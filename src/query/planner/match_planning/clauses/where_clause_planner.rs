//! WHERE 子句规划器
//! 实现新的 CypherClausePlanner 接口
//! 
//! WHERE 子句是 Cypher 查询的过滤子句，负责根据指定的条件过滤输入数据流。

use crate::query::planner::match_planning::core::cypher_clause_planner::{
    CypherClausePlanner, ClauseType, PlanningContext, VariableRequirement, VariableProvider,
};
use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::match_planning::paths::match_path_planner::MatchPathPlanner;
use crate::query::planner::match_planning::utils::connection_strategy::UnifiedConnector;
use crate::query::planner::plan::{PlanNodeKind, SubPlan};
use crate::query::planner::plan::core::nodes::{PlanNodeFactory, StartNode};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use crate::query::validator::structs::CypherClauseKind;
use crate::query::parser::ast::expr::Expr;
use std::sync::Arc;
use std::collections::HashSet;

/// WHERE 子句规划器
/// 
/// 负责规划 WHERE 子句的执行。WHERE 子句是一个转换子句，
/// 它需要输入数据流并根据指定的过滤条件对结果进行过滤。
/// 
/// # 示例
/// 
/// ```cypher
/// MATCH (n:Person)
/// WHERE n.age > 25 AND n.name STARTS WITH 'John'
/// RETURN n.name, n.age
/// ```
/// 
/// 在上面的例子中，WHERE 子句会过滤出年龄大于25且姓名以'John'开头的人员。
#[derive(Debug)]
pub struct WhereClausePlanner {
    need_stable_filter: bool, // 是否需要稳定的过滤器（用于ORDER BY场景）
}

impl WhereClausePlanner {
    /// 创建新的 WHERE 子句规划器
    /// 
    /// # 参数
    /// 
    /// * `need_stable_filter` - 是否需要稳定的过滤器，用于ORDER BY场景
    pub fn new(need_stable_filter: bool) -> Self {
        Self { need_stable_filter }
    }

    /// 构建 WHERE 子句的执行计划
    /// 
    /// # 参数
    /// 
    /// * `where_clause_ctx` - WHERE 子句的上下文信息
    /// * `input_plan` - 输入的执行计划
    /// * `context` - 规划上下文
    /// 
    /// # 返回值
    /// 
    /// 返回包含 WHERE 子句执行计划的 SubPlan
    fn build_where(
        &self,
        where_clause_ctx: &crate::query::validator::structs::clause_structs::WhereClauseContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 处理路径表达式（模式谓词）
        let mut plan = if !where_clause_ctx.paths.is_empty() {
            let mut paths_plan = SubPlan::new(None, None);

            // 为模式表达式构建计划
            for path in &where_clause_ctx.paths {
                let mut path_planner = MatchPathPlanner::new(
                    // 这里需要创建一个临时的MatchClauseContext
                    crate::query::validator::structs::clause_structs::MatchClauseContext {
                        paths: vec![path.clone()],
                        aliases_available: where_clause_ctx.aliases_available.clone(),
                        aliases_generated: where_clause_ctx.aliases_generated.clone(),
                        where_clause: None,
                        is_optional: false,
                        skip: None,
                        limit: None,
                    },
                    path.clone(),
                );

                // 暂时使用旧接口，因为 MatchPathPlanner 还没有更新
                let path_plan = path_planner.transform(None, &mut HashSet::new())?;

                // 使用新的统一连接器
                let mut intersected_aliases = HashSet::new();
                // 添加路径中的别名
                for node_info in &path.node_infos {
                    if !node_info.alias.is_empty() {
                        intersected_aliases.insert(node_info.alias.clone());
                    }
                }
                for edge_info in &path.edge_infos {
                    if !edge_info.alias.is_empty() {
                        intersected_aliases.insert(edge_info.alias.clone());
                    }
                }

                if path.is_pred {
                    // 构建模式谓词的计划
                    paths_plan = UnifiedConnector::pattern_apply(
                        &crate::query::context::ast::base::AstContext::new("WHERE", "test"),
                        &paths_plan,
                        &path_plan,
                        intersected_aliases,
                    )?;
                } else {
                    // 构建路径收集的计划
                    paths_plan = UnifiedConnector::roll_up_apply(
                        &crate::query::context::ast::base::AstContext::new("WHERE", "test"),
                        &paths_plan,
                        &path_plan,
                        intersected_aliases,
                    )?;
                }
            }

            paths_plan
        } else {
            SubPlan::new(None, None)
        };

        // 处理过滤条件
        if let Some(filter) = &where_clause_ctx.filter {
            let mut where_plan = SubPlan::new(None, None);

            // 创建起始节点作为输入
            let start_node = PlanNodeFactory::create_start_node()?;
            
            // 创建过滤器节点 - 将 Expression 转换为 Expr
            let expr = convert_expression_to_expr(filter);
            let filter_node = PlanNodeFactory::create_filter(
                start_node,
                expr,
            )?;

            where_plan = SubPlan::from_single_node(filter_node);

            if plan.root.is_none() {
                return Ok(where_plan);
            }

            plan = UnifiedConnector::add_input(
                &crate::query::context::ast::base::AstContext::new("WHERE", "test"),
                &where_plan,
                &plan,
                true,
            )?;
        }

        Ok(plan)
    }
}

impl ClausePlanner for WhereClausePlanner {
    fn name(&self) -> &'static str {
        "WhereClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Where
    }
}

impl CypherClausePlanner for WhereClausePlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 验证输入
        self.validate_input(input_plan)?;
        
        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::missing_input("WHERE clause requires input".to_string())
        })?;
        
        // 验证上下文类型
        if !matches!(clause_ctx.kind(), CypherClauseKind::Where) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for WhereClausePlanner".to_string(),
            ));
        }

        let where_clause_ctx = match clause_ctx {
            CypherClauseContext::Where(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected WhereClauseContext".to_string(),
                ))
            }
        };

        // 构建 WHERE 子句的执行计划
        self.build_where(where_clause_ctx, input_plan, context)
    }
    
    fn validate_input(&self, input_plan: Option<&SubPlan>) -> Result<(), PlannerError> {
        if input_plan.is_none() {
            return Err(PlannerError::missing_input(
                "WHERE clause requires input from previous clauses".to_string()
            ));
        }
        Ok(())
    }
    
    fn clause_type(&self) -> ClauseType {
        ClauseType::Transform
    }
    
    fn can_start_flow(&self) -> bool {
        false  // WHERE 不能开始数据流
    }
    
    fn requires_input(&self) -> bool {
        true   // WHERE 需要输入
    }
    
    fn input_requirements(&self) -> Vec<VariableRequirement> {
        // WHERE 子句需要输入数据，但不强制要求特定变量
        vec![]
    }
    
    fn output_provides(&self) -> Vec<VariableProvider> {
        // WHERE 子句不产生新的变量，只是过滤输入
        vec![]
    }
}

/// 将 Expression 转换为 Expr
fn convert_expression_to_expr(expr: &crate::graph::expression::Expression) -> Expr {
    use crate::query::parser::ast::expr::*;
    use crate::query::parser::ast::types::Span;
    
    match expr {
        crate::graph::expression::Expression::Variable(name) => {
            Expr::Variable(VariableExpr::new(name.clone(), Span::default()))
        }
        crate::graph::expression::Expression::Literal(val) => {
            use crate::core::Value;
            let const_val = match val {
                crate::graph::expression::expression::LiteralValue::String(s) => Value::String(s.clone()),
                crate::graph::expression::expression::LiteralValue::Int(i) => Value::Int(*i),
                crate::graph::expression::expression::LiteralValue::Float(f) => Value::Float(*f),
                crate::graph::expression::expression::LiteralValue::Bool(b) => Value::Bool(*b),
                crate::graph::expression::expression::LiteralValue::Null => Value::Null,
            };
            Expr::Constant(ConstantExpr::new(const_val, Span::default()))
        }
        _ => {
            // 其他表达式类型暂时使用默认的 true 表达式
            Expr::Constant(ConstantExpr::new(crate::core::Value::Bool(true), Span::default()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::match_planning::core::ClauseType;
    
    #[test]
    fn test_where_clause_planner_interface() {
        let planner = WhereClausePlanner::new(false);
        assert_eq!(planner.clause_type(), ClauseType::Transform);
        assert!(!planner.can_start_flow());
        assert!(planner.requires_input());
    }
    
    #[test]
    fn test_where_clause_planner_validate_input() {
        let planner = WhereClausePlanner::new(false);
        
        // 测试没有输入的情况
        let result = planner.validate_input(None);
        assert!(result.is_err());
        
        // 测试有输入的情况
        let dummy_plan = SubPlan::new(None, None);
        let result = planner.validate_input(Some(&dummy_plan));
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_where_clause_planner_stable_filter() {
        let planner = WhereClausePlanner::new(true);
        assert!(planner.need_stable_filter);
        
        let planner = WhereClausePlanner::new(false);
        assert!(!planner.need_stable_filter);
    }
}