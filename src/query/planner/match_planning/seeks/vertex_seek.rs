use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::PlanNodeKind;
//! 顶点查找规划器
//! 根据顶点ID进行查找
//! 负责规划基于顶点ID的查找操作，包括固定ID和可变ID

use crate::graph::expression::Expression;
use crate::query::planner::plan::core::nodes::PlanNodeFactory;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::path_structs::NodeInfo;
use crate::query::planner::match_planning::seeks::seek_strategy::SeekStrategy;
use crate::query::planner::match_planning::utils::node_factory::create_start_node;
use std::sync::Arc;

/// 顶点查找类型
#[derive(Debug, Clone)]
pub enum VertexSeekType {
    /// 固定顶点ID列表
    Fixed(Vec<String>),
    /// 可变顶点ID表达式
    Variable(Expression),
}

/// 顶点查找规划器
/// 负责规划基于顶点ID的查找操作，支持固定ID和可变ID两种模式
#[derive(Debug)]
pub struct VertexSeek {
    node_info: NodeInfo,
    seek_type: VertexSeekType,
}

impl VertexSeek {
    /// 创建固定顶点ID查找规划器
    pub fn new_fixed(node_info: NodeInfo, vids: Vec<String>) -> Self {
        Self {
            node_info,
            seek_type: VertexSeekType::Fixed(vids),
        }
    }

    /// 创建可变顶点ID查找规划器
    pub fn new_variable(node_info: NodeInfo, vid_expr: Expression) -> Self {
        Self {
            node_info,
            seek_type: VertexSeekType::Variable(vid_expr),
        }
    }

    /// 构建顶点查找计划
    pub fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        // 创建获取顶点节点
        let get_vertices_node = Arc::new(PlanNodeFactory::create_placeholder_node()??,
        ));

        // 根据查找类型设置不同的参数
        match &self.seek_type {
            VertexSeekType::Fixed(vids) => {
                // TODO: 设置固定顶点ID列表
                // 这里需要根据vids设置要查找的顶点ID
                if vids.is_empty() {
                    return Err(PlannerError::InvalidAstContext(
                        "顶点ID列表不能为空".to_string(),
                    ));
                }
            }
            VertexSeekType::Variable(vid_expr) => {
                // TODO: 设置可变顶点ID表达式
                // 这里需要根据vid_expr设置要查找的顶点ID表达式
                if !self.is_valid_variable_expression(vid_expr) {
                    return Err(PlannerError::InvalidAstContext(
                        "无效的顶点ID表达式".to_string(),
                    ));
                }
            }
        }

        Ok(SubPlan::new(
            Some(get_vertices_node.clone()),
            Some(get_vertices_node),
        ))
    }

    /// 检查是否可以使用顶点查找
    pub fn match_node(&self) -> bool {
        match &self.seek_type {
            VertexSeekType::Fixed(vids) => !vids.is_empty(),
            VertexSeekType::Variable(vid_expr) => self.is_valid_variable_expression(vid_expr),
        }
    }

    /// 检查是否是有效的变量表达式
    fn is_valid_variable_expression(&self, expr: &Expression) -> bool {
        matches!(
            expr,
            Expression::Label(_) | Expression::Variable(_)
        )
    }

    /// 获取查找类型
    pub fn seek_type(&self) -> &VertexSeekType {
        &self.seek_type
    }

    /// 获取节点信息
    pub fn node_info(&self) -> &NodeInfo {
        &self.node_info
    }
}

impl SeekStrategy for VertexSeek {
    fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        self.build_plan()
    }

    fn match_node(&self) -> bool {
        self.match_node()
    }

    fn name(&self) -> &'static str {
        match &self.seek_type {
            VertexSeekType::Fixed(_) => "VertexIdSeek",
            VertexSeekType::Variable(_) => "VariableVertexIdSeek",
        }
    }

    fn estimate_cost(&self) -> f64 {
        match &self.seek_type {
            VertexSeekType::Fixed(vids) => {
                // 固定ID查找的成本很低，且与ID数量成正比
                vids.len() as f64 * 0.1
            }
            VertexSeekType::Variable(_) => {
                // 可变ID查找的成本稍高
                5.0
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::Expression;

    fn create_test_node_info() -> NodeInfo {
        NodeInfo {
            alias: "n".to_string(),
            labels: vec![],
            props: None,
            anonymous: false,
            filter: None,
            tids: vec![],
            label_props: vec![],
        }
    }

    #[test]
    fn test_vertex_seek_new_fixed() {
        let node_info = create_test_node_info();
        let vids = vec!["1".to_string(), "2".to_string()];
        let seeker = VertexSeek::new_fixed(node_info, vids.clone());

        match seeker.seek_type() {
            VertexSeekType::Fixed(stored_vids) => {
                assert_eq!(stored_vids, &vids);
            }
            _ => panic!("Expected Fixed seek type"),
        }
    }

    #[test]
    fn test_vertex_seek_new_variable() {
        let node_info = create_test_node_info();
        let vid_expr = Expression::Variable("x".to_string());
        let seeker = VertexSeek::new_variable(node_info, vid_expr.clone());

        match seeker.seek_type() {
            VertexSeekType::Variable(stored_expr) => {
                assert_eq!(stored_expr, &vid_expr);
            }
            _ => panic!("Expected Variable seek type"),
        }
    }

    #[test]
    fn test_match_node_fixed() {
        let node_info = create_test_node_info();
        let seeker = VertexSeek::new_fixed(node_info.clone(), vec!["1".to_string()]);
        assert!(seeker.match_node());

        let empty_seeker = VertexSeek::new_fixed(node_info, vec![]);
        assert!(!empty_seeker.match_node());
    }

    #[test]
    fn test_match_node_variable() {
        let node_info = create_test_node_info();
        let valid_expr = Expression::Variable("x".to_string());
        let seeker = VertexSeek::new_variable(node_info.clone(), valid_expr);
        assert!(seeker.match_node());

        let invalid_expr = Expression::Literal(
            crate::graph::expression::expression::LiteralValue::String("test".to_string()),
        );
        let invalid_seeker = VertexSeek::new_variable(node_info, invalid_expr);
        assert!(!invalid_seeker.match_node());
    }

    #[test]
    fn test_seek_strategy_name() {
        let node_info = create_test_node_info();
        let fixed_seeker = VertexSeek::new_fixed(node_info.clone(), vec!["1".to_string()]);
        assert_eq!(fixed_seeker.name(), "VertexIdSeek");

        let variable_seeker = VertexSeek::new_variable(
            node_info,
            Expression::Variable("x".to_string()),
        );
        assert_eq!(variable_seeker.name(), "VariableVertexIdSeek");
    }

    #[test]
    fn test_estimate_cost() {
        let node_info = create_test_node_info();
        let single_id_seeker = VertexSeek::new_fixed(node_info.clone(), vec!["1".to_string()]);
        assert!((single_id_seeker.estimate_cost() - 0.1).abs() < f64::EPSILON);

        let multi_id_seeker = VertexSeek::new_fixed(node_info.clone(), vec!["1".to_string(), "2".to_string(), "3".to_string()]);
        // 使用近似比较处理浮点数精度问题
        let expected_cost = 0.3;
        let actual_cost = multi_id_seeker.estimate_cost();
        assert!((actual_cost - expected_cost).abs() < 1e-10, "Expected {}, but got {}", expected_cost, actual_cost);

        let variable_seeker = VertexSeek::new_variable(
            node_info,
            Expression::Variable("x".to_string()),
        );
        assert!((variable_seeker.estimate_cost() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_build_plan_fixed() {
        let node_info = create_test_node_info();
        let seeker = VertexSeek::new_fixed(node_info, vec!["1".to_string()]);
        let result = seeker.build_plan();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_plan_variable() {
        let node_info = create_test_node_info();
        let seeker = VertexSeek::new_variable(
            node_info,
            Expression::Variable("x".to_string()),
        );
        let result = seeker.build_plan();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_plan_fixed_empty() {
        let node_info = create_test_node_info();
        let seeker = VertexSeek::new_fixed(node_info, vec![]);
        let result = seeker.build_plan();
        assert!(result.is_err());
    }

    #[test]
    fn test_build_plan_variable_invalid() {
        let node_info = create_test_node_info();
        let seeker = VertexSeek::new_variable(
            node_info,
            Expression::Literal(
                crate::graph::expression::expression::LiteralValue::String("test".to_string()),
            ),
        );
        let result = seeker.build_plan();
        assert!(result.is_err());
    }
}