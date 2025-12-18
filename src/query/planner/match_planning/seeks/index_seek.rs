use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::PlanNodeKind;
/// 索引查找规划器
/// 根据标签索引和属性索引进行查找
/// 负责规划基于索引的查找操作，包括标签索引、属性索引和可变属性索引

use crate::graph::expression::Expression;
use crate::query::planner::match_planning::seeks::seek_strategy::SeekStrategy;
use crate::query::planner::plan::core::{PlanNode, PlanNodeMutable};
use crate::query::planner::plan::core::nodes::PlanNodeFactory;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::path_structs::NodeInfo;
use std::sync::Arc;

/// 索引查找元数据
/// 存储IndexScan节点执行所需的索引相关信息
#[derive(Debug, Clone)]
pub struct IndexScanMetadata {
    /// 标签ID列表
    pub label_ids: Vec<i32>,
    /// 标签名称列表
    pub label_names: Vec<String>,
    /// 索引ID列表
    pub index_ids: Vec<i32>,
    /// 是否有属性过滤
    pub has_property_filter: bool,
    /// 属性过滤表达式（如果存在）
    pub property_filter: Option<Expression>,
}

impl IndexScanMetadata {
    /// 创建新的IndexScan元数据
    pub fn new(label_ids: Vec<i32>, label_names: Vec<String>, index_ids: Vec<i32>) -> Self {
        Self {
            label_ids,
            label_names,
            index_ids,
            has_property_filter: false,
            property_filter: None,
        }
    }

    /// 设置属性过滤表达式
    pub fn set_property_filter(&mut self, filter: Expression) {
        self.has_property_filter = true;
        self.property_filter = Some(filter);
    }
}

/// 索引查找类型
#[derive(Debug, Clone)]
pub enum IndexSeekType {
    /// 标签索引查找
    Label,
    /// 属性索引查找
    Property(Vec<Expression>),
    /// 可变属性索引查找
    VariableProperty(Vec<Expression>),
}

/// 索引查找规划器
/// 负责规划基于索引的查找操作，支持标签索引、属性索引和可变属性索引
#[derive(Debug)]
pub struct IndexSeek {
    node_info: NodeInfo,
    seek_type: IndexSeekType,
}

impl IndexSeek {
    /// 创建标签索引查找规划器
    pub fn new_label(node_info: NodeInfo) -> Self {
        Self {
            node_info,
            seek_type: IndexSeekType::Label,
        }
    }

    /// 创建属性索引查找规划器
    pub fn new_property(node_info: NodeInfo, prop_exprs: Vec<Expression>) -> Self {
        Self {
            node_info,
            seek_type: IndexSeekType::Property(prop_exprs),
        }
    }

    /// 创建可变属性索引查找规划器
    pub fn new_variable_property(node_info: NodeInfo, prop_exprs: Vec<Expression>) -> Self {
        Self {
            node_info,
            seek_type: IndexSeekType::VariableProperty(prop_exprs),
        }
    }

    /// 构建索引查找计划
    pub fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        // 验证基本条件
        self.validate_conditions()?;

        // 创建索引扫描节点
        let index_scan_node = PlanNodeFactory::create_placeholder_node()?;

        // 根据查找类型设置不同的参数
        let (label_id, label_name) = self.get_primary_label_info()?;
        let index_id = label_id;

        // 设置IndexScan节点的输出变量
        let var_name = format!("index_scan_{}", label_name);
        let variable = crate::query::context::validate::types::Variable {
            name: var_name,
            columns: vec![crate::query::context::validate::types::Column {
                name: "vid".to_string(),
                type_: "Vertex".to_string(),
            }],
        };

        // 由于不能直接修改 Arc<dyn PlanNode>，我们使用占位符
        let mut metadata =
            IndexScanMetadata::new(vec![label_id], vec![label_name.clone()], vec![index_id]);

        // 处理节点属性过滤
        let mut root: Arc<dyn PlanNode> = index_scan_node.clone_plan_node();
        if let Some(props) = &self.node_info.props {
            metadata.set_property_filter(props.clone());
        }

        // 处理属性表达式
        match &self.seek_type {
            IndexSeekType::Property(prop_exprs) | IndexSeekType::VariableProperty(prop_exprs) => {
                if !prop_exprs.is_empty() {
                    // TODO: 设置属性索引表达式
                    // 这里需要根据prop_exprs设置要扫描的属性索引表达式
                }
            }
            IndexSeekType::Label => {
                // 标签索引查找不需要额外的属性表达式
            }
        }

        // 处理节点过滤条件 - 创建独立的Filter节点而不是修改IndexScan
        if let Some(_filter) = &self.node_info.filter {
            // 创建Filter节点来处理过滤条件
            let filter_node = PlanNodeFactory::create_placeholder_node()?;

            // 设置Filter节点的输出变量
            let filter_var_name = format!("filtered_{}", label_name);
            let filter_variable = crate::query::context::validate::types::Variable {
                name: filter_var_name,
                columns: vec![crate::query::context::validate::types::Column {
                    name: "vid".to_string(),
                    type_: "Vertex".to_string(),
                }],
            };

            // 由于不能直接修改 Arc<dyn PlanNode>，我们使用占位符
            root = filter_node.clone_plan_node();
        }

        // 对于索引查找，tail应该是IndexScan节点
        // root可能是Filter节点（如果有过滤条件）或IndexScan节点
        let tail = index_scan_node.clone_plan_node();
        Ok(SubPlan::new(Some(root), Some(tail)))
    }

    /// 验证查找条件
    fn validate_conditions(&self) -> Result<(), PlannerError> {
        match &self.seek_type {
            IndexSeekType::Label => {
                if self.node_info.labels.is_empty() {
                    return Err(PlannerError::InvalidAstContext(
                        "节点必须有标签才能使用标签索引查找".to_string(),
                    ));
                }
                if self.node_info.tids.is_empty() {
                    return Err(PlannerError::InvalidAstContext(
                        "节点标签ID列表不能为空".to_string(),
                    ));
                }
            }
            IndexSeekType::Property(prop_exprs) => {
                if prop_exprs.is_empty() {
                    return Err(PlannerError::UnsupportedOperation(
                        "No property expressions for index seek".to_string(),
                    ));
                }
            }
            IndexSeekType::VariableProperty(prop_exprs) => {
                if prop_exprs.is_empty() {
                    return Err(PlannerError::UnsupportedOperation(
                        "No property expressions for variable index seek".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// 获取主要标签信息
    fn get_primary_label_info(&self) -> Result<(i32, String), PlannerError> {
        match &self.seek_type {
            IndexSeekType::Label => {
                if self.node_info.labels.is_empty() || self.node_info.tids.is_empty() {
                    return Err(PlannerError::InvalidAstContext(
                        "无效的节点信息：标签或标签ID为空".to_string(),
                    ));
                }
                let label_id = self.node_info.tids[0];
                let label_name = self.node_info.labels[0].clone();
                Ok((label_id, label_name))
            }
            IndexSeekType::Property(_) | IndexSeekType::VariableProperty(_) => {
                // 对于属性索引查找，如果有标签信息则使用，否则使用默认值
                if !self.node_info.labels.is_empty() && !self.node_info.tids.is_empty() {
                    let label_id = self.node_info.tids[0];
                    let label_name = self.node_info.labels[0].clone();
                    Ok((label_id, label_name))
                } else {
                    // 使用默认标签信息
                    Ok((0, "default".to_string()))
                }
            }
        }
    }

    /// 检查是否可以使用索引查找
    pub fn match_node(&self) -> bool {
        match &self.seek_type {
            IndexSeekType::Label => {
                !self.node_info.labels.is_empty() && !self.node_info.tids.is_empty()
            }
            IndexSeekType::Property(prop_exprs) => !prop_exprs.is_empty(),
            IndexSeekType::VariableProperty(prop_exprs) => {
                !prop_exprs.is_empty()
                    && prop_exprs
                        .iter()
                        .any(|expr| matches!(expr, Expression::Label(_) | Expression::Variable(_)))
            }
        }
    }

    /// 获取索引扫描元数据
    pub fn get_index_metadata(&self) -> Result<IndexScanMetadata, PlannerError> {
        let (label_id, label_name) = self.get_primary_label_info()?;
        let index_id = label_id;

        let mut metadata = IndexScanMetadata::new(vec![label_id], vec![label_name], vec![index_id]);

        // 如果有属性过滤表达式，记录在元数据中
        if let Some(props) = &self.node_info.props {
            metadata.set_property_filter(props.clone());
        }

        Ok(metadata)
    }

    /// 获取查找类型
    pub fn seek_type(&self) -> &IndexSeekType {
        &self.seek_type
    }

    /// 获取节点信息
    pub fn node_info(&self) -> &NodeInfo {
        &self.node_info
    }
}

impl SeekStrategy for IndexSeek {
    fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        self.build_plan()
    }

    fn match_node(&self) -> bool {
        self.match_node()
    }

    fn name(&self) -> &'static str {
        match &self.seek_type {
            IndexSeekType::Label => "LabelIndexSeek",
            IndexSeekType::Property(_) => "PropIndexSeek",
            IndexSeekType::VariableProperty(_) => "VariablePropIndexSeek",
        }
    }

    fn estimate_cost(&self) -> f64 {
        match &self.seek_type {
            IndexSeekType::Label => 50.0,
            IndexSeekType::Property(prop_exprs) => {
                // 属性索引查找的成本与属性数量成正比
                prop_exprs.len() as f64 * 10.0
            }
            IndexSeekType::VariableProperty(prop_exprs) => {
                // 可变属性索引查找的成本更高
                prop_exprs.len() as f64 * 20.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::Expression;

    fn create_test_node_info(labels: Vec<&str>, tids: Vec<i32>) -> NodeInfo {
        NodeInfo {
            alias: "n".to_string(),
            labels: labels.into_iter().map(|s| s.to_string()).collect(),
            props: None,
            anonymous: false,
            filter: None,
            tids,
            label_props: vec![None],
        }
    }

    #[test]
    fn test_index_seek_new_label() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = IndexSeek::new_label(node_info);

        match seeker.seek_type() {
            IndexSeekType::Label => {
                // 预期的类型
            }
            _ => panic!("Expected Label seek type"),
        }
    }

    #[test]
    fn test_index_seek_new_property() {
        let node_info = create_test_node_info(vec![], vec![]);
        let prop_exprs = vec![Expression::Variable("x".to_string())];
        let seeker = IndexSeek::new_property(node_info, prop_exprs.clone());

        match seeker.seek_type() {
            IndexSeekType::Property(stored_exprs) => {
                assert_eq!(stored_exprs, &prop_exprs);
            }
            _ => panic!("Expected Property seek type"),
        }
    }

    #[test]
    fn test_index_seek_new_variable_property() {
        let node_info = create_test_node_info(vec![], vec![]);
        let prop_exprs = vec![Expression::Variable("x".to_string())];
        let seeker = IndexSeek::new_variable_property(node_info, prop_exprs.clone());

        match seeker.seek_type() {
            IndexSeekType::VariableProperty(stored_exprs) => {
                assert_eq!(stored_exprs, &prop_exprs);
            }
            _ => panic!("Expected VariableProperty seek type"),
        }
    }

    #[test]
    fn test_match_node_label() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = IndexSeek::new_label(node_info);
        assert!(seeker.match_node());

        let empty_node_info = create_test_node_info(vec![], vec![1]);
        let empty_seeker = IndexSeek::new_label(empty_node_info);
        assert!(!empty_seeker.match_node());
    }

    #[test]
    fn test_match_node_property() {
        let node_info = create_test_node_info(vec![], vec![]);
        let prop_exprs = vec![Expression::Variable("x".to_string())];
        let seeker = IndexSeek::new_property(node_info.clone(), prop_exprs);
        assert!(seeker.match_node());

        let empty_seeker = IndexSeek::new_property(node_info, vec![]);
        assert!(!empty_seeker.match_node());
    }

    #[test]
    fn test_match_node_variable_property() {
        let node_info = create_test_node_info(vec![], vec![]);
        let valid_exprs = vec![Expression::Variable("x".to_string())];
        let seeker = IndexSeek::new_variable_property(node_info.clone(), valid_exprs);
        assert!(seeker.match_node());

        let invalid_exprs = vec![Expression::Literal(
            crate::graph::expression::expression::LiteralValue::String("test".to_string()),
        )];
        let invalid_seeker = IndexSeek::new_variable_property(node_info, invalid_exprs);
        assert!(!invalid_seeker.match_node());
    }

    #[test]
    fn test_seek_strategy_name() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let label_seeker = IndexSeek::new_label(node_info);
        assert_eq!(label_seeker.name(), "LabelIndexSeek");

        let node_info2 = create_test_node_info(vec![], vec![]);
        let prop_seeker =
            IndexSeek::new_property(node_info2.clone(), vec![Expression::Variable("x".to_string())]);
        assert_eq!(prop_seeker.name(), "PropIndexSeek");

        let var_prop_seeker = IndexSeek::new_variable_property(
            node_info2,
            vec![Expression::Variable("x".to_string())],
        );
        assert_eq!(var_prop_seeker.name(), "VariablePropIndexSeek");
    }

    #[test]
    fn test_estimate_cost() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let label_seeker = IndexSeek::new_label(node_info);
        assert_eq!(label_seeker.estimate_cost(), 50.0);

        let node_info2 = create_test_node_info(vec![], vec![]);
        let single_prop_seeker =
            IndexSeek::new_property(node_info2.clone(), vec![Expression::Variable("x".to_string())]);
        assert_eq!(single_prop_seeker.estimate_cost(), 10.0);

        let multi_prop_seeker = IndexSeek::new_property(
            node_info2.clone(),
            vec![
                Expression::Variable("x".to_string()),
                Expression::Variable("y".to_string()),
            ],
        );
        assert_eq!(multi_prop_seeker.estimate_cost(), 20.0);

        let var_prop_seeker = IndexSeek::new_variable_property(
            node_info2,
            vec![Expression::Variable("x".to_string())],
        );
        assert_eq!(var_prop_seeker.estimate_cost(), 20.0);
    }

    #[test]
    fn test_build_plan_label() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = IndexSeek::new_label(node_info);
        let result = seeker.build_plan();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_plan_property() {
        let node_info = create_test_node_info(vec![], vec![]);
        let seeker =
            IndexSeek::new_property(node_info, vec![Expression::Variable("x".to_string())]);
        let result = seeker.build_plan();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_plan_variable_property() {
        let node_info = create_test_node_info(vec![], vec![]);
        let seeker = IndexSeek::new_variable_property(
            node_info,
            vec![Expression::Variable("x".to_string())],
        );
        let result = seeker.build_plan();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_plan_label_without_labels() {
        let node_info = create_test_node_info(vec![], vec![1]);
        let seeker = IndexSeek::new_label(node_info);
        let result = seeker.build_plan();
        assert!(result.is_err());
    }

    #[test]
    fn test_build_plan_property_without_exprs() {
        let node_info = create_test_node_info(vec![], vec![]);
        let seeker = IndexSeek::new_property(node_info, vec![]);
        let result = seeker.build_plan();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_index_metadata() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = IndexSeek::new_label(node_info);
        let result = seeker.get_index_metadata();
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.label_ids, vec![1]);
        assert_eq!(metadata.label_names, vec!["Person".to_string()]);
        assert_eq!(metadata.index_ids, vec![1]);
    }
}
