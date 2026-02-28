//! FETCH EDGES查询规划器
//! 处理FETCH EDGES查询的规划

use crate::query::QueryContext;
use crate::query::parser::ast::{FetchTarget, Stmt};
use crate::query::planner::plan::core::nodes::{
    ArgumentNode, FilterNode, GetEdgesNode, ProjectNode,
};
use crate::query::planner::plan::core::PlanNodeEnum;
use crate::query::planner::plan::execution_plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};
use std::sync::Arc;

/// FETCH EDGES查询规划器
/// 负责将FETCH EDGES查询转换为执行计划
#[derive(Debug, Clone)]
pub struct FetchEdgesPlanner;

impl FetchEdgesPlanner {
    /// 创建新的FETCH EDGES规划器
    pub fn new() -> Self {
        Self
    }
}

impl Planner for FetchEdgesPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let fetch_stmt = match &validated.stmt {
            Stmt::Fetch(fetch_stmt) => fetch_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "FetchEdgesPlanner 需要 Fetch 语句".to_string()
                ));
            }
        };

        // 检查是否是 FETCH EDGES
        let (src, dst, edge_type, rank) = match &fetch_stmt.target {
            FetchTarget::Edges { src, dst, edge_type, rank, .. } => (src, dst, edge_type, rank),
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "FetchEdgesPlanner 需要 FETCH EDGES 语句".to_string()
                ));
            }
        };

        let var_name = "e";

        // 1. 创建参数节点，获取边的条件
        let arg_node = ArgumentNode::new(1, var_name);

        // 从表达式中提取字符串值
        let src_str = extract_string_from_expr(src)?;
        let dst_str = extract_string_from_expr(dst)?;
        let rank_str = rank.as_ref().map(|r| extract_string_from_expr(r)).transpose()?.unwrap_or_else(|| "0".to_string());

        // 2. 创建获取边的节点
        let get_edges_node = PlanNodeEnum::GetEdges(GetEdgesNode::new(
            1, // space_id
            &src_str,
            edge_type,
            &rank_str,
            &dst_str,
        ));

        // 3. 创建过滤空边的节点
        use std::sync::Arc;
        let ctx = Arc::new(crate::core::types::ExpressionContext::new());
        let filter_node = match FilterNode::from_expression(
            get_edges_node.clone(),
            crate::core::Expression::Variable(format!("{} IS NOT EMPTY", var_name)),
            ctx,
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

        // 5. 创建SubPlan
        let arg_node = PlanNodeEnum::Argument(arg_node);
        let sub_plan = SubPlan::new(Some(project_node), Some(arg_node));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Fetch(fetch_stmt) => {
                matches!(&fetch_stmt.target, FetchTarget::Edges { .. })
            }
            _ => false,
        }
    }
}

/// 从表达式中提取字符串值
fn extract_string_from_expr(expr: &crate::core::Expression) -> Result<String, PlannerError> {
    match expr {
        crate::core::Expression::Variable(s) => Ok(s.clone()),
        crate::core::Expression::Literal(v) => {
            match v {
                crate::core::Value::String(s) => Ok(s.clone()),
                crate::core::Value::Int(i) => Ok(i.to_string()),
                crate::core::Value::Float(f) => Ok(f.to_string()),
                crate::core::Value::Bool(b) => Ok(b.to_string()),
                _ => Err(PlannerError::InvalidOperation(format!("无法从字面量提取字符串: {:?}", v))),
            }
        }
        _ => Err(PlannerError::InvalidOperation(format!("无法从表达式提取字符串: {:?}", expr))),
    }
}

impl Default for FetchEdgesPlanner {
    fn default() -> Self {
        Self::new()
    }
}
