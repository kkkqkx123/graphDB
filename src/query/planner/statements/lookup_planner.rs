//! LOOKUP语句规划器
//! 处理Nebula LOOKUP查询的规划
//!
//! ## 改进说明
//!
//! - 统一导入路径
//! - 完善表达式解析
//! - 添加属性索引选择逻辑

use crate::core::types::expression::Expression;
use crate::query::context::ast::{AstContext, LookupContext};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

pub use crate::query::planner::plan::algorithms::IndexScan;
pub use crate::query::planner::plan::core::nodes::{
    ArgumentNode, DedupNode, FilterNode, GetEdgesNode, GetVerticesNode, HashInnerJoinNode,
    ProjectNode,
};
pub use crate::query::planner::plan::core::PlanNodeEnum;

/// LOOKUP查询规划器
/// 负责将LOOKUP语句转换为执行计划
#[derive(Debug)]
pub struct LookupPlanner {
    query_context: AstContext,
}

impl LookupPlanner {
    /// 创建新的LOOKUP规划器
    pub fn new() -> Self {
        Self {
            query_context: AstContext::from_strings("LOOKUP", "LOOKUP ON player WHERE player.name == 'test'"),
        }
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配LOOKUP查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "LOOKUP"
    }
}

impl Planner for LookupPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let lookup_ctx = LookupContext::new(ast_ctx.clone());

        let space_id = ast_ctx.space().space_id.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("Space ID is required for LOOKUP query".to_string())
        })?;

        if space_id == 0 {
            return Err(PlannerError::PlanGenerationFailed(
                "Invalid space ID: 0".to_string(),
            ));
        }

        if lookup_ctx.schema_id == 0 {
            return Err(PlannerError::PlanGenerationFailed(
                "Invalid schema ID: 0".to_string(),
            ));
        }

        let index_scan_type = if lookup_ctx.is_fulltext_index {
            "FULLTEXT".to_string()
        } else {
            "RANGE".to_string()
        };

        let index_scan_node = IndexScan::new(
            -1,
            space_id as i32,
            lookup_ctx.schema_id,
            lookup_ctx.schema_id,
            &index_scan_type,
        );
        let mut current_node: PlanNodeEnum = PlanNodeEnum::IndexScan(index_scan_node);

        if lookup_ctx.is_fulltext_index && lookup_ctx.has_score {
            let id_expr = Expression::Variable("id".to_string());

            let get_node = if lookup_ctx.is_edge {
                let get_edges = GetEdgesNode::new(space_id as i32, "", "", "", "");
                PlanNodeEnum::GetEdges(get_edges)
            } else {
                let get_vertices = GetVerticesNode::new(space_id as i32, "");
                PlanNodeEnum::GetVertices(get_vertices)
            };

            let argument_node = ArgumentNode::new(-1, "id");
            let argument_enum = PlanNodeEnum::Argument(argument_node);

            let hash_join =
                HashInnerJoinNode::new(get_node, argument_enum, vec![id_expr.clone()], vec![id_expr])
                    .map_err(|e| {
                        PlannerError::PlanGenerationFailed(format!(
                            "Failed to create HashInnerJoinNode: {}",
                            e
                        ))
                    })?;

            current_node = PlanNodeEnum::HashInnerJoin(hash_join);
        } else if lookup_ctx.is_fulltext_index {
            let get_node = if lookup_ctx.is_edge {
                let get_edges = GetEdgesNode::new(space_id as i32, "", "", "", "");
                PlanNodeEnum::GetEdges(get_edges)
            } else {
                let get_vertices = GetVerticesNode::new(space_id as i32, "");
                PlanNodeEnum::GetVertices(get_vertices)
            };

            current_node = get_node;
        }

        if let Some(ref condition) = lookup_ctx.filter {
            let expr = Self::parse_filter_expression(condition)?;
            let filter_node = FilterNode::new(current_node, expr).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create FilterNode: {}", e))
            })?;
            current_node = PlanNodeEnum::Filter(filter_node);
        }

        let yield_columns = Self::build_yield_columns(&lookup_ctx)?;
        let project_node = ProjectNode::new(current_node, yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
        })?;
        current_node = PlanNodeEnum::Project(project_node);

        let final_node = if lookup_ctx.dedup {
            match DedupNode::new(current_node.clone()) {
                Ok(dedup) => PlanNodeEnum::Dedup(dedup),
                Err(_) => current_node,
            }
        } else {
            current_node
        };

        let arg_node = ArgumentNode::new(0, "lookup_input");
        let sub_plan = SubPlan {
            root: Some(final_node),
            tail: Some(PlanNodeEnum::Argument(arg_node)),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl LookupPlanner {
    /// 解析过滤条件表达式
    fn parse_filter_expression(condition: &str) -> Result<Expression, PlannerError> {
        if condition.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "Filter condition is empty".to_string(),
            ));
        }

        if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim().trim_matches('"');
                return Ok(Expression::Function {
                    name: "eq".to_string(),
                    args: vec![
                        Expression::Property {
                            object: Box::new(Expression::Variable(left.to_string())),
                            property: left.to_string(),
                        },
                        Expression::Literal(crate::core::Value::from(right.to_string())),
                    ],
                });
            }
        } else if condition.contains("=") {
            let parts: Vec<&str> = condition.split("=").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim().trim_matches('"');
                return Ok(Expression::Function {
                    name: "eq".to_string(),
                    args: vec![
                        Expression::Property {
                            object: Box::new(Expression::Variable(left.to_string())),
                            property: left.to_string(),
                        },
                        Expression::Literal(crate::core::Value::from(right.to_string())),
                    ],
                });
            }
        }

        Ok(Expression::Variable(condition.to_string()))
    }

    /// 构建YIELD列
    fn build_yield_columns(
        lookup_ctx: &LookupContext,
    ) -> Result<Vec<crate::query::validator::YieldColumn>, PlannerError> {
        let mut columns = Vec::new();

        if let Some(ref yield_expr) = lookup_ctx.yield_expr {
            for col in &yield_expr.columns {
                columns.push(crate::query::validator::YieldColumn {
                    expr: Self::parse_yield_expression(&col.name(), lookup_ctx.is_edge)?,
                    alias: col.alias.clone(),
                    is_matched: false,
                });
            }
        } else {
            columns.push(crate::query::validator::YieldColumn {
                expr: Expression::Variable("*".to_string()),
                alias: "result".to_string(),
                is_matched: false,
            });
        }

        if columns.is_empty() {
            columns.push(crate::query::validator::YieldColumn {
                expr: Expression::Variable("*".to_string()),
                alias: "result".to_string(),
                is_matched: false,
            });
        }

        Ok(columns)
    }

    /// 解析YIELD表达式
    fn parse_yield_expression(name: &str, _is_edge: bool) -> Result<Expression, PlannerError> {
        if name.contains(".") {
            let parts: Vec<&str> = name.split(".").collect();
            if parts.len() == 2 {
                return Ok(Expression::Property {
                    object: Box::new(Expression::Variable(parts[0].to_string())),
                    property: parts[1].to_string(),
                });
            }
        }

        Ok(Expression::Variable(name.to_string()))
    }
}

impl Default for LookupPlanner {
    fn default() -> Self {
        Self::new()
    }
}
