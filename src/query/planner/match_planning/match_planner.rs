//! MATCH查询主规划器
//! 负责将MATCH查询转换为执行计划

use crate::query::context::AstContext;
use crate::query::planner::match_planning::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::match_planning::match_clause_planner::MatchClausePlanner;
use crate::query::planner::match_planning::return_clause_planner::ReturnClausePlanner;
use crate::query::planner::match_planning::segments_connector::SegmentsConnector;
use crate::query::planner::match_planning::unwind_clause_planner::UnwindClausePlanner;
use crate::query::planner::match_planning::where_clause_planner::WhereClausePlanner;
use crate::query::planner::match_planning::with_clause_planner::WithClausePlanner;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::{PlanNodeKind, SingleDependencyNode};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::validator::structs::{
    alias_structs::QueryPart,
    clause_structs::{
        MatchClauseContext, ReturnClauseContext, WhereClauseContext, WithClauseContext,
    },
    CypherClauseContext, CypherClauseContext as ClauseContext, CypherClauseKind,
};
use std::collections::HashSet;

/// MATCH查询规划器
/// 处理Cypher MATCH语句的转换为执行计划
#[derive(Debug)]
pub struct MatchPlanner {
    tail_connected: bool,
}

impl MatchPlanner {
    /// 创建新的MATCH规划器
    pub fn new() -> Self {
        Self {
            tail_connected: false,
        }
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配MATCH查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "MATCH"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }

    /// 生成子句计划
    fn gen_plan(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        match clause_ctx.kind() {
            CypherClauseKind::Match => {
                let mut planner = MatchClausePlanner::new();
                planner.transform(clause_ctx)
            }
            CypherClauseKind::Unwind => {
                let mut planner = UnwindClausePlanner::new();
                planner.transform(clause_ctx)
            }
            CypherClauseKind::With => {
                let mut planner = WithClausePlanner::new();
                planner.transform(clause_ctx)
            }
            CypherClauseKind::Return => {
                let mut planner = ReturnClausePlanner::new();
                planner.transform(clause_ctx)
            }
            _ => Err(PlannerError::UnsupportedOperation(
                "Unsupported clause type in MATCH query".to_string(),
            )),
        }
    }

    /// 连接MATCH计划到之前的查询计划
    fn connect_match_plan(
        &mut self,
        query_plan: &mut SubPlan,
        match_ctx: &MatchClauseContext,
    ) -> Result<(), PlannerError> {
        // 生成当前MATCH计划
        let match_plan = self.gen_plan(&CypherClauseContext::Match(match_ctx.clone()))?;

        if query_plan.root.is_none() {
            *query_plan = match_plan;
            return Ok(());
        }

        // 找到交集别名
        let mut inter_aliases = HashSet::new();
        for (alias, _) in &match_ctx.aliases_generated {
            if match_ctx.aliases_available.contains_key(alias) {
                inter_aliases.insert(alias.clone());
                // TODO: 检查类型兼容性
            }
        }

        let connector = SegmentsConnector::new();

        if !inter_aliases.is_empty() {
            if match_ctx.is_optional {
                // 处理可选MATCH的左连接
                if let Some(where_ctx) = &match_ctx.where_clause {
                    // 连接WHERE过滤条件
                    let where_plan =
                        self.gen_plan(&CypherClauseContext::Where(where_ctx.clone()))?;
                    let match_plan_with_where = connector.add_input(where_plan, match_plan, true);
                    *query_plan = connector.left_join(
                        query_plan.clone(),
                        match_plan_with_where,
                        inter_aliases.into_iter().collect(),
                    );
                } else {
                    *query_plan = connector.left_join(
                        query_plan.clone(),
                        match_plan,
                        inter_aliases.into_iter().collect(),
                    );
                }
            } else {
                // 内连接
                *query_plan = connector.inner_join(
                    query_plan.clone(),
                    match_plan,
                    inter_aliases.into_iter().collect(),
                );
            }
        } else {
            // 笛卡尔积
            *query_plan = connector.cartesian_product(query_plan.clone(), match_plan);
        }

        Ok(())
    }

    /// 生成查询部分计划
    fn gen_query_part_plan(
        &mut self,
        query_plan: &mut SubPlan,
        query_part: &QueryPart,
    ) -> Result<(), PlannerError> {
        // 为MATCH子句生成计划
        for match_ctx in &query_part.matchs {
            self.connect_match_plan(query_plan, match_ctx)?;

            // 连接MATCH过滤条件（非可选MATCH）
            if let Some(where_ctx) = &match_ctx.where_clause {
                if !match_ctx.is_optional {
                    let where_plan =
                        self.gen_plan(&CypherClauseContext::Where(where_ctx.clone()))?;
                    let connector = SegmentsConnector::new();
                    *query_plan = connector.add_input(where_plan, query_plan.clone(), true);
                }
            }
        }

        // 设置边界子句的输入列名
        if let Some(boundary) = &query_part.boundary {
            if let Some(root) = &query_plan.root {
                // TODO: 设置输入列名
            }
        }

        // 为边界子句生成计划
        if let Some(boundary) = &query_part.boundary {
            let boundary_plan = match boundary {
                crate::query::validator::structs::alias_structs::BoundaryClauseContext::With(
                    with_ctx,
                ) => self.gen_plan(&CypherClauseContext::With(with_ctx.clone()))?,
                crate::query::validator::structs::alias_structs::BoundaryClauseContext::Unwind(
                    unwind_ctx,
                ) => self.gen_plan(&CypherClauseContext::Unwind(unwind_ctx.clone()))?,
            };

            if query_plan.root.is_none() {
                *query_plan = boundary_plan;
            } else {
                let connector = SegmentsConnector::new();
                *query_plan = connector.add_input(boundary_plan, query_plan.clone(), false);
            }
        }

        // TODO: 为所有查询计划尾部生成变量
        if let Some(tail) = &query_plan.tail {
            if tail.kind() == PlanNodeKind::Argument {
                // 设置输入变量
            }
        }

        if !self.tail_connected {
            self.tail_connected = true;
            // TODO: 添加起始节点
        }

        Ok(())
    }
}

impl Planner for MatchPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 验证这是MATCH语句
        if !Self::match_ast_ctx(ast_ctx) {
            return Err(PlannerError::InvalidAstContext(
                "Only MATCH statements are accepted by MatchPlanner".to_string(),
            ));
        }

        // TODO: 从AST上下文中提取Cypher上下文
        // 这里需要解析AST并构建相应的Cypher上下文结构

        // 临时创建一个简单的查询计划作为示例
        let mut query_plan = SubPlan::new(None, None);

        // 创建起始节点
        let start_node = Box::new(SingleDependencyNode {
            id: -1,
            kind: PlanNodeKind::Start,
            dependencies: vec![],
            output_var: None,
            col_names: vec![],
            cost: 0.0,
        });

        // 创建获取邻居节点
        let get_neighbors_node = Box::new(SingleDependencyNode {
            id: -1,
            kind: PlanNodeKind::GetNeighbors,
            dependencies: vec![start_node],
            output_var: None,
            col_names: vec!["vertex".to_string()],
            cost: 1.0,
        });

        query_plan.root = Some(get_neighbors_node.clone());
        query_plan.tail = Some(get_neighbors_node);

        Ok(query_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for MatchPlanner {
    fn default() -> Self {
        Self::new()
    }
}
