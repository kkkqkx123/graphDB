//! 标签索引查找规划器
//! 根据标签索引进行查找
//! 负责规划基于标签索引的查找操作

use crate::graph::expression::Expression;
use crate::query::planner::plan::core::{PlanNode, PlanNodeMutable};
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::path_structs::NodeInfo;
use std::sync::Arc;

/// 标签索引查找元数据
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

/// 标签索引查找规划器
/// 负责规划基于标签索引的查找操作
#[derive(Debug)]
pub struct LabelIndexSeek {
    node_info: NodeInfo,
}

impl LabelIndexSeek {
    pub fn new(node_info: NodeInfo) -> Self {
        Self { node_info }
    }

    /// 构建标签索引查找计划
    ///
    /// 返回一个执行计划，包含：
    /// 1. IndexScan节点：执行索引扫描获取顶点ID
    /// 2. 可选的Filter节点：对属性和条件进行过滤
    pub fn build_plan(&self) -> Result<SubPlan, PlannerError> {
        // 验证基本条件
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

        // 创建索引扫描节点
        let index_scan_node = Arc::new(SingleInputNode::new(
            PlanNodeKind::IndexScan,
            create_start_node()?,
        ));

        // 处理多标签情况：使用第一个标签作为主标签
        let label_id = self.node_info.tids[0];
        let label_name = &self.node_info.labels[0];

        // 创建索引ID（在实际实现中应从元数据获取）
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

        // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
        // 我们需要创建一个新的节点来设置属性
        let mut new_index_scan_node = (*index_scan_node).clone();
        new_index_scan_node.set_output_var(variable);
        new_index_scan_node.set_col_names(vec!["vid".to_string()]);
        let index_scan_node = Arc::new(new_index_scan_node);

        // 构建索引元数据用于执行器
        let mut metadata =
            IndexScanMetadata::new(vec![label_id], vec![label_name.clone()], vec![index_id]);

        // 处理节点属性过滤
        let mut root: Arc<dyn PlanNode> = index_scan_node.clone();
        if let Some(props) = &self.node_info.props {
            metadata.set_property_filter(props.clone());
        }

        // 处理节点过滤条件 - 创建独立的Filter节点而不是修改IndexScan
        if let Some(_filter) = &self.node_info.filter {
            // 创建Filter节点来处理过滤条件
            let filter_node = Arc::new(SingleInputNode::new(
                PlanNodeKind::Filter,
                index_scan_node.clone(),
            ));

            // 设置Filter节点的输出变量
            let filter_var_name = format!("filtered_{}", label_name);
            let filter_variable = crate::query::context::validate::types::Variable {
                name: filter_var_name,
                columns: vec![crate::query::context::validate::types::Column {
                    name: "vid".to_string(),
                    type_: "Vertex".to_string(),
                }],
            };

            // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
            // 我们需要创建一个新的节点来设置属性
            let mut new_filter_node = (*filter_node).clone();
            new_filter_node.set_output_var(filter_variable);
            new_filter_node.set_col_names(vec!["vid".to_string()]);
            let filter_node = Arc::new(new_filter_node);

            root = filter_node;
        }

        // 对于标签索引查找，tail应该是IndexScan节点
        // root可能是Filter节点（如果有过滤条件）或IndexScan节点
        let tail = index_scan_node.clone();
        Ok(SubPlan::new(Some(root), Some(tail)))
    }

    /// 检查是否可以使用标签索引查找
    ///
    /// 条件：
    /// 1. 节点有标签
    /// 2. 节点有对应的标签ID
    pub fn match_node(&self) -> bool {
        !self.node_info.labels.is_empty() && !self.node_info.tids.is_empty()
    }

    /// 获取索引扫描元数据
    /// 用于在执行期间获取索引信息
    pub fn get_index_metadata(&self) -> Result<IndexScanMetadata, PlannerError> {
        if self.node_info.labels.is_empty() || self.node_info.tids.is_empty() {
            return Err(PlannerError::InvalidAstContext(
                "无效的节点信息：标签或标签ID为空".to_string(),
            ));
        }

        let label_id = self.node_info.tids[0];
        let label_name = self.node_info.labels[0].clone();
        let index_id = label_id;

        let mut metadata = IndexScanMetadata::new(vec![label_id], vec![label_name], vec![index_id]);

        // 如果有属性过滤表达式，记录在元数据中
        if let Some(props) = &self.node_info.props {
            metadata.set_property_filter(props.clone());
        }

        Ok(metadata)
    }
}

/// 创建起始节点
fn create_start_node() -> Result<Arc<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
    use crate::query::planner::plan::SingleDependencyNode;

    Ok(Arc::new(SingleDependencyNode {
        id: -1,
        kind: PlanNodeKind::Start,
        dependencies: vec![],
        output_var: None,
        col_names: vec![],
        cost: 0.0,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::Expression;
    use crate::query::validator::structs::path_structs::NodeInfo;

    /// 创建测试用的节点信息
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
    fn test_label_index_seek_new() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = LabelIndexSeek::new(node_info);

        // 验证创建的实例
        assert_eq!(seeker.node_info.alias, "n");
        assert_eq!(seeker.node_info.labels.len(), 1);
        assert_eq!(seeker.node_info.labels[0], "Person");
    }

    #[test]
    fn test_match_node_with_labels() {
        let node_info = create_test_node_info(vec!["Person", "User"], vec![1, 2]);
        let seeker = LabelIndexSeek::new(node_info);

        // 有标签和标签ID的节点应该匹配
        assert!(seeker.match_node());
    }

    #[test]
    fn test_match_node_without_labels() {
        let node_info = create_test_node_info(vec![], vec![1]);
        let seeker = LabelIndexSeek::new(node_info);

        // 没有标签的节点不应该匹配
        assert!(!seeker.match_node());
    }

    #[test]
    fn test_match_node_without_tids() {
        let node_info = create_test_node_info(vec!["Person"], vec![]);
        let seeker = LabelIndexSeek::new(node_info);

        // 没有标签ID的节点不应该匹配
        assert!(!seeker.match_node());
    }

    #[test]
    fn test_match_node_with_single_label() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = LabelIndexSeek::new(node_info);

        // 有单个标签的节点应该匹配
        assert!(seeker.match_node());
    }

    #[test]
    fn test_build_plan_success() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = LabelIndexSeek::new(node_info);

        let result = seeker.build_plan();

        // 构建计划应该成功
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
        assert!(subplan.tail.is_some());

        // 验证尾节点（IndexScan）
        if let Some(tail) = &subplan.tail {
            assert_eq!(tail.kind(), PlanNodeKind::IndexScan);
        }

        // 验证根节点（IndexScan，或Filter如果有过滤条件）
        if let Some(root) = &subplan.root {
            assert_eq!(root.kind(), PlanNodeKind::IndexScan);
        }
    }

    #[test]
    fn test_build_plan_with_multiple_labels() {
        let node_info = create_test_node_info(vec!["Person", "User", "Admin"], vec![1, 2, 3]);
        let seeker = LabelIndexSeek::new(node_info);

        let result = seeker.build_plan();

        // 构建计划应该成功（使用第一个标签）
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
        assert!(subplan.tail.is_some());
    }

    #[test]
    fn test_build_plan_without_labels() {
        let node_info = create_test_node_info(vec![], vec![1]);
        let seeker = LabelIndexSeek::new(node_info);

        let result = seeker.build_plan();

        // 没有标签应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_build_plan_without_tids() {
        let node_info = create_test_node_info(vec!["Person"], vec![]);
        let seeker = LabelIndexSeek::new(node_info);

        let result = seeker.build_plan();

        // 没有标签ID应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_create_start_node() {
        let result = create_start_node();

        // 创建起始节点应该成功
        assert!(result.is_ok());

        let start_node = result.unwrap();
        assert_eq!(start_node.kind(), PlanNodeKind::Start);
        assert_eq!(start_node.id(), -1);
        assert_eq!(start_node.dependencies().len(), 0);
        assert_eq!(start_node.cost(), 0.0);
    }

    #[test]
    fn test_build_plan_with_properties() {
        let mut node_info = create_test_node_info(vec!["Person"], vec![1]);
        node_info.props = Some(Expression::Literal(
            crate::graph::expression::expression::LiteralValue::String("test".to_string()),
        ));

        let seeker = LabelIndexSeek::new(node_info);

        // 有属性的节点应该仍然匹配
        assert!(seeker.match_node());

        // 构建计划应该成功
        let result = seeker.build_plan();
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
        assert!(subplan.tail.is_some());
    }

    #[test]
    fn test_build_plan_with_filter() {
        let mut node_info = create_test_node_info(vec!["Person"], vec![1]);
        node_info.filter = Some(Expression::Variable("x".to_string()));

        let seeker = LabelIndexSeek::new(node_info);

        // 有过滤条件的节点应该仍然匹配
        assert!(seeker.match_node());

        // 构建计划应该成功
        let result = seeker.build_plan();
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
        assert!(subplan.tail.is_some());

        // 验证根节点应该是Filter节点
        if let Some(root) = &subplan.root {
            assert_eq!(root.kind(), PlanNodeKind::Filter);
        }

        // 验证尾节点应该是IndexScan节点
        if let Some(tail) = &subplan.tail {
            assert_eq!(tail.kind(), PlanNodeKind::IndexScan);
        }
    }

    #[test]
    fn test_label_index_seek_debug() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = LabelIndexSeek::new(node_info);

        let debug_str = format!("{:?}", seeker);
        assert!(debug_str.contains("LabelIndexSeek"));
    }

    #[test]
    fn test_index_scan_metadata_creation() {
        let metadata = IndexScanMetadata::new(vec![1], vec!["Person".to_string()], vec![1]);

        assert_eq!(metadata.label_ids, vec![1]);
        assert_eq!(metadata.label_names, vec!["Person".to_string()]);
        assert_eq!(metadata.index_ids, vec![1]);
        assert!(!metadata.has_property_filter);
        assert!(metadata.property_filter.is_none());
    }

    #[test]
    fn test_index_scan_metadata_with_filter() {
        let mut metadata = IndexScanMetadata::new(vec![1], vec!["Person".to_string()], vec![1]);

        let filter = Expression::Literal(
            crate::graph::expression::expression::LiteralValue::String("test".to_string()),
        );
        metadata.set_property_filter(filter);

        assert!(metadata.has_property_filter);
        assert!(metadata.property_filter.is_some());
    }

    #[test]
    fn test_get_index_metadata() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = LabelIndexSeek::new(node_info);

        let result = seeker.get_index_metadata();
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.label_ids, vec![1]);
        assert_eq!(metadata.label_names, vec!["Person".to_string()]);
        assert_eq!(metadata.index_ids, vec![1]);
    }

    #[test]
    fn test_get_index_metadata_without_labels() {
        let node_info = create_test_node_info(vec![], vec![1]);
        let seeker = LabelIndexSeek::new(node_info);

        let result = seeker.get_index_metadata();
        assert!(result.is_err());
    }

    #[test]
    fn test_subplan_structure() {
        let node_info = create_test_node_info(vec!["Person"], vec![1]);
        let seeker = LabelIndexSeek::new(node_info);

        let result = seeker.build_plan();
        assert!(result.is_ok());

        let subplan = result.unwrap();

        // 验证 SubPlan 结构
        assert!(subplan.root().is_some());
        assert!(subplan.tail().is_some());

        // 验证尾节点总是IndexScan
        if let Some(tail) = subplan.tail() {
            assert_eq!(tail.kind(), PlanNodeKind::IndexScan);
        }
    }
}
