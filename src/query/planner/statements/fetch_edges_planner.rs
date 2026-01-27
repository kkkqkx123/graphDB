//! FETCH EDGES查询规划器
//! 处理FETCH EDGES查询的规划

use crate::query::context::ast::{AstContext, FetchEdgesContext};
use crate::query::planner::plan::core::nodes::{
    ArgumentNode, DedupNode, FilterNode, GetEdgesNode, ProjectNode,
};
use crate::query::planner::plan::core::PlanNodeEnum;
use crate::query::planner::plan::execution_plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

/// FETCH EDGES查询规划器
/// 负责将FETCH EDGES查询转换为执行计划
#[derive(Debug, Clone)]
pub struct FetchEdgesPlanner;

impl FetchEdgesPlanner {
    /// 创建新的FETCH EDGES规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配FETCH EDGES查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "FETCH EDGES"
    }

    /// 获取匹配和实例化函数（静态注册版本）
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiateEnum {
        crate::query::planner::planner::MatchAndInstantiateEnum::FetchEdges(Self::new())
    }
}

impl Planner for FetchEdgesPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 从ast_ctx创建FetchEdgesContext
        let fetch_ctx = FetchEdgesContext::new(ast_ctx.clone());

        // 实现FETCH EDGES查询的规划逻辑
        println!("Processing FETCH EDGES query planning: {:?}", fetch_ctx);

        // 1. 创建参数节点，获取边的条件
        let arg_node = ArgumentNode::new(1, &fetch_ctx.input_var_name);

        // 2. 创建获取边的节点
        let get_edges_node = PlanNodeEnum::GetEdges(GetEdgesNode::new(
            1, // space_id
            &fetch_ctx.src.clone().unwrap_or_default(),
            &fetch_ctx.edge_type.clone().unwrap_or_default(),
            &fetch_ctx.rank.clone().unwrap_or_default(),
            &fetch_ctx.dst.clone().unwrap_or_default(),
        ));

        // 3. 创建过滤空边的节点
        let filter_node = match FilterNode::new(
            get_edges_node.clone(),
            crate::core::Expression::Variable(format!("{} IS NOT EMPTY", fetch_ctx.edge_name)),
        ) {
            Ok(node) => PlanNodeEnum::Filter(node),
            Err(_) => get_edges_node.clone(),
        };

        // 4. 创建投影节点
        let project_node = match ProjectNode::new(filter_node.clone(), vec![]) {
            Ok(node) => PlanNodeEnum::Project(node),
            Err(e) => {
                println!("Failed to create project node: {:?}", e);
                filter_node
            }
        };

        // 5. 如果需要去重，创建去重节点
        let final_node = if fetch_ctx.distinct {
            match DedupNode::new(project_node.clone()) {
                Ok(node) => PlanNodeEnum::Dedup(node),
                Err(_) => project_node.clone(),
            }
        } else {
            project_node
        };

        // 创建SubPlan
        let arg_node = PlanNodeEnum::Argument(arg_node);
        let sub_plan = SubPlan::new(Some(final_node), Some(arg_node));

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for FetchEdgesPlanner {
    fn default() -> Self {
        Self::new()
    }
}
