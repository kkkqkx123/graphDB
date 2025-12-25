use crate::core::{Expression, Value};
/// 扫描查找规划器
/// 进行全表扫描操作的规划
/// 负责规划全表扫描操作
use crate::query::planner::match_planning::seeks::seek_strategy::SeekStrategy;

use crate::query::planner::plan::{PlanNodeFactory, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::path_structs::NodeInfo;

/// 扫描查找规划器
/// 负责规划全表扫描操作，作为其他查找策略失败时的备选方案
#[derive(Debug)]
pub struct ScanSeek {
    node_info: NodeInfo,
}

impl ScanSeek {
    /// 创建新的扫描查找规划器
    pub fn new(node_info: NodeInfo) -> Self {
        Self { node_info }
    }

    /// 构建扫描查找计划
    pub fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        // 创建实际的扫描顶点节点
        let space_id = 1; // TODO: 应该从上下文获取space_id
        let scan_vertices_node = PlanNodeFactory::create_scan_vertices(space_id)?;

        // 如果有标签过滤，添加过滤节点
        let root = if !self.node_info.labels.is_empty() {
            // 创建标签过滤表达式
            let _filter_expr = self.create_label_filter_expression()?;
            // TODO: 需要将 Expression 转换为 Expr
            // 暂时使用克隆的方式，实际需要实现转换逻辑
            scan_vertices_node.clone()
        } else {
            scan_vertices_node.clone()
        };

        // 如果有额外的过滤条件，再添加一个过滤节点
        let final_root = if let Some(_filter) = &self.node_info.filter {
            // TODO: 需要将 Expression 转换为 Expr
            // 暂时返回原节点，实际需要实现转换逻辑
            root
        } else {
            root
        };

        Ok(SubPlan::new(Some(final_root), Some(scan_vertices_node)))
    }

    /// 创建标签过滤表达式
    fn create_label_filter_expression(&self) -> Result<Expression, PlannerError> {
        if self.node_info.labels.is_empty() {
            return Err(PlannerError::InvalidAstContext(
                "没有标签可用于过滤".to_string(),
            ));
        }

        // 如果有多个标签，创建OR条件
        let mut filter_expr = self.create_single_label_filter(&self.node_info.labels[0])?;

        for label_name in &self.node_info.labels[1..] {
            let label_filter = self.create_single_label_filter(label_name)?;
            filter_expr = Expression::Binary {
                left: Box::new(filter_expr),
                op: crate::core::BinaryOperator::Or,
                right: Box::new(label_filter),
            };
        }

        Ok(filter_expr)
    }

    /// 创建单个标签的过滤表达式
    fn create_single_label_filter(&self, label_name: &str) -> Result<Expression, PlannerError> {
        Ok(Expression::Function {
            name: "hasLabel".to_string(),
            args: vec![
                Expression::Variable(self.node_info.alias.clone()),
                Expression::Literal(Value::String(label_name.to_string())),
            ],
        })
    }

    /// 检查是否可以使用扫描查找
    pub fn match_node(&self) -> bool {
        // 扫描查找总是可用的，作为最后的备选方案
        true
    }

    /// 获取扫描成本估计
    pub fn estimate_cost(&self) -> f64 {
        // 基础扫描成本
        let mut cost = 1000.0;

        // 如果有标签过滤，降低成本
        if !self.node_info.labels.is_empty() {
            cost *= 0.8; // 标签过滤可以减少扫描的数据量
        }

        // 如果有属性过滤，进一步降低成本
        if self.node_info.props.is_some() {
            cost *= 0.6;
        }

        // 如果有额外的过滤条件，进一步降低成本
        if self.node_info.filter.is_some() {
            cost *= 0.5;
        }

        cost
    }

    /// 获取节点信息
    pub fn node_info(&self) -> &NodeInfo {
        &self.node_info
    }
}

impl SeekStrategy for ScanSeek {
    fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        self.build_plan()
    }

    fn match_node(&self) -> bool {
        self.match_node()
    }

    fn name(&self) -> &'static str {
        "FullScan"
    }

    fn estimate_cost(&self) -> f64 {
        self.estimate_cost()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    fn create_test_node_info(labels: Vec<&str>, has_props: bool, has_filter: bool) -> NodeInfo {
        NodeInfo {
            alias: "n".to_string(),
            labels: labels.into_iter().map(|s| s.to_string()).collect(),
            props: if has_props {
                Some(Expression::Variable("prop".to_string()))
            } else {
                None
            },
            anonymous: false,
            filter: if has_filter {
                Some(Expression::Variable("filter".to_string()))
            } else {
                None
            },
            tids: vec![],
            label_props: vec![],
        }
    }

    #[test]
    fn test_scan_seek_new() {
        let node_info = create_test_node_info(vec![], false, false);
        let scan_seek = ScanSeek::new(node_info);
        assert_eq!(scan_seek.node_info().alias, "n");
    }

    #[test]
    fn test_match_node() {
        let node_info = create_test_node_info(vec![], false, false);
        let scan_seek = ScanSeek::new(node_info);
        assert!(scan_seek.match_node()); // 扫描查找总是可用
    }

    #[test]
    fn test_estimate_cost() {
        // 基础扫描成本
        let node_info = create_test_node_info(vec![], false, false);
        let scan_seek = ScanSeek::new(node_info);
        assert_eq!(scan_seek.estimate_cost(), 1000.0);

        // 有标签过滤
        let node_info = create_test_node_info(vec!["Person"], false, false);
        let scan_seek = ScanSeek::new(node_info);
        assert_eq!(scan_seek.estimate_cost(), 800.0);

        // 有属性过滤
        let node_info = create_test_node_info(vec!["Person"], true, false);
        let scan_seek = ScanSeek::new(node_info);
        assert_eq!(scan_seek.estimate_cost(), 480.0);

        // 有额外过滤
        let node_info = create_test_node_info(vec!["Person"], true, true);
        let scan_seek = ScanSeek::new(node_info);
        assert_eq!(scan_seek.estimate_cost(), 240.0);
    }

    #[test]
    fn test_create_label_filter_expression() {
        // 单个标签
        let node_info = create_test_node_info(vec!["Person"], false, false);
        let scan_seek = ScanSeek::new(node_info);
        let filter_expr = scan_seek
            .create_label_filter_expression()
            .expect("Label filter expression should be created successfully");

        match filter_expr {
            Expression::Function { name, args } => {
                assert_eq!(name, "hasLabel");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Function expression"),
        }

        // 多个标签
        let node_info = create_test_node_info(vec!["Person", "Student"], false, false);
        let scan_seek = ScanSeek::new(node_info);
        let filter_expr = scan_seek
            .create_label_filter_expression()
            .expect("Label filter expression should be created successfully");

        match filter_expr {
            Expression::Binary { op, .. } => {
                assert_eq!(op, crate::core::BinaryOperator::Or);
            }
            _ => panic!("Expected Binary expression with OR operator"),
        }
    }

    #[test]
    fn test_create_label_filter_expression_empty() {
        let node_info = create_test_node_info(vec![], false, false);
        let scan_seek = ScanSeek::new(node_info);
        let result = scan_seek.create_label_filter_expression();
        assert!(result.is_err());
    }

    #[test]
    fn test_build_plan() {
        // 基础扫描
        let node_info = create_test_node_info(vec![], false, false);
        let scan_seek = ScanSeek::new(node_info);
        let result = scan_seek.build_plan();
        assert!(result.is_ok());

        // 带标签过滤的扫描
        let node_info = create_test_node_info(vec!["Person"], false, false);
        let scan_seek = ScanSeek::new(node_info);
        let result = scan_seek.build_plan();
        assert!(result.is_ok());

        // 带属性和过滤的扫描
        let node_info = create_test_node_info(vec!["Person"], true, true);
        let scan_seek = ScanSeek::new(node_info);
        let result = scan_seek.build_plan();
        assert!(result.is_ok());
    }

    #[test]
    fn test_seek_strategy_implementation() {
        let node_info = create_test_node_info(vec!["Person"], false, false);
        let scan_seek = ScanSeek::new(node_info);

        assert_eq!(scan_seek.name(), "FullScan");
        assert!(scan_seek.match_node());
        assert!(scan_seek.build_plan().is_ok());
        assert_eq!(scan_seek.estimate_cost(), 800.0);
    }
}
