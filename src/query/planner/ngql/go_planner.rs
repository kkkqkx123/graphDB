//! GO语句规划器
//! 处理Nebula GO查询的规划

use crate::query::context::ast::{AstContext, GoContext};
use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::common::{EdgeProp, TagProp};
use crate::query::planner::plan::core::plan_node_traits::{PlanNodeDependencies, PlanNodeMutable};
use crate::query::planner::plan::core::{
    ArgumentNode, DedupNode, ExpandNode, ExpandAllNode, FilterNode, InnerJoinNode, ProjectNode,
};
use crate::query::planner::plan::utils::join_params::JoinParams;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use std::sync::Arc;

/// GO查询规划器
/// 负责将GO语句转换为执行计划
#[derive(Debug)]
pub struct GoPlanner;

impl GoPlanner {
    /// 创建新的GO规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配GO查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "GO"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
            priority: 100,
        }
    }
}

impl Planner for GoPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 从ast_ctx创建GoContext
        let go_ctx = GoContext::new(ast_ctx.clone());

        // 实现GO查询的规划逻辑
        println!("Processing GO query planning: {:?}", go_ctx);

        // 创建执行计划节点
        // 1. 创建参数节点（如果需要）
        let arg_node = Arc::new(ArgumentNode::new(1, &go_ctx.from.user_defined_var_name));

        // 2. 创建扩展节点
        let mut _edge_types = go_ctx.over.edge_types.clone();
        // 如果是双向扩展，设置边类型
        if go_ctx.over.direction == "both" {
            _edge_types = go_ctx.over.edge_types.clone();
        } else if go_ctx.over.direction == "in" {
            // 对于入边，边类型取负值
            _edge_types = go_ctx
                .over
                .edge_types
                .iter()
                .map(|et| format!("-{}", et))
                .collect();
        } else {
            // 默认是出边
            _edge_types = go_ctx.over.edge_types.clone();
        }

        let _expand_node = Arc::new(ExpandNode::new(1, _edge_types.clone(), "out"));

        // 3. 创建ExpandAll节点进行多步扩展
        let direction = if go_ctx.over.direction == "both" {
            "both"
        } else if go_ctx.over.direction == "in" {
            "in"
        } else {
            "out"
        };

        let edge_types = go_ctx.over.edge_types.clone(); // 正确初始化edge_types变量
        let expand_all_node = Arc::new(ExpandAllNode::new(1, edge_types, direction));

        // 4. 如果有JOIN操作，创建JOIN节点
        let join_node = if go_ctx.join_dst {
            // 使用InnerJoinNode替代HashLeftJoin
            use crate::graph::expression::Expression;
            let join_key = Expression::Variable("_expandall_vid".to_string());
            let join = Arc::new(InnerJoinNode::new(
                expand_all_node.clone(),
                arg_node.clone(),
                vec![join_key.clone()],
                vec![join_key],
            ).unwrap());
            Some(join)
        } else {
            None
        };

        // 5. 创建过滤节点（如果有过滤条件）
        let filter_node = if let Some(ref condition) = go_ctx.filter {
            use crate::graph::expression::Expression;
            let expr = Expression::Variable(condition.clone());
            let dependency_node: Arc<dyn crate::query::planner::plan::core::PlanNode> =
                if let Some(ref join_ref) = join_node {
                    join_ref.clone()
                } else {
                    expand_all_node.clone()
                };
            let filter = Arc::new(FilterNode::new(dependency_node, expr).unwrap());
            Some(filter)
        } else {
            None
        };

        // 6. 创建投影节点
        use crate::query::validator::YieldColumn;
        use crate::graph::expression::Expression;
        let yield_columns = vec![YieldColumn {
            expr: Expression::Variable(go_ctx.yield_expr.clone().unwrap_or("DEFAULT".to_string())),
            alias: "project_result".to_string(),
            is_matched: false,
        }];
        
        let last_node: Arc<dyn crate::query::planner::plan::core::PlanNode> =
            if let Some(ref filter_ref) = filter_node {
                filter_ref.clone()
            } else if let Some(ref join_ref) = join_node {
                join_ref.clone()
            } else {
                expand_all_node.clone()
            };

        let project_node = Arc::new(ProjectNode::new(last_node, yield_columns).unwrap());

        // 7. 如果需要去重，创建去重节点
        let final_node: Arc<dyn crate::query::planner::plan::core::PlanNode> = if go_ctx.distinct {
            let dedup_node = Arc::new(DedupNode::new(project_node).unwrap());
            dedup_node
        } else {
            project_node
        };

        // 创建SubPlan
        let sub_plan = SubPlan {
            root: Some(final_node),
            tail: Some(arg_node),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for GoPlanner {
    fn default() -> Self {
        Self::new()
    }
}
