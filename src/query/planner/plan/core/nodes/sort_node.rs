//! 排序节点实现
//!
//! SortNode 用于对输入数据进行排序操作

use crate::core::types::graph_schema::OrderDirection;
use crate::define_plan_node_with_deps;

/// 排序项定义
/// 包含列名和排序方向
#[derive(Debug, Clone, PartialEq)]
pub struct SortItem {
    /// 排序列名
    pub column: String,
    /// 排序方向
    pub direction: OrderDirection,
}

impl SortItem {
    /// 创建新的排序项
    pub fn new(column: String, direction: OrderDirection) -> Self {
        Self { column, direction }
    }

    /// 创建升序排序项
    pub fn asc(column: String) -> Self {
        Self::new(column, OrderDirection::Asc)
    }

    /// 创建降序排序项
    pub fn desc(column: String) -> Self {
        Self::new(column, OrderDirection::Desc)
    }
}

define_plan_node_with_deps! {
    pub struct SortNode {
        sort_items: Vec<SortItem>,
        limit: Option<i64>,
    }
    enum: Sort
    input: SingleInputNode
}

impl SortNode {
    /// 创建新的排序节点
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        sort_items: Vec<SortItem>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            sort_items,
            limit: None,
            output_var: None,
            col_names,
        })
    }

    /// 获取排序字段
    pub fn sort_items(&self) -> &[SortItem] {
        &self.sort_items
    }

    /// 获取限制数量
    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    /// 设置限制数量
    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }
}

define_plan_node_with_deps! {
    pub struct LimitNode {
        offset: i64,
        count: i64,
    }
    enum: Limit
    input: SingleInputNode
}

impl LimitNode {
    /// 创建新的限制节点
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        offset: i64,
        count: i64,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            offset,
            count,
            output_var: None,
            col_names,
        })
    }

    /// 获取偏移量
    pub fn offset(&self) -> i64 {
        self.offset
    }

    /// 获取计数
    pub fn count(&self) -> i64 {
        self.count
    }
}

define_plan_node_with_deps! {
    pub struct TopNNode {
        sort_items: Vec<SortItem>,
        limit: i64,
    }
    enum: TopN
    input: SingleInputNode
}

impl TopNNode {
    /// 创建新的TopN节点
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        sort_items: Vec<SortItem>,
        limit: i64,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            sort_items,
            limit,
            output_var: None,
            col_names,
        })
    }

    /// 获取排序字段
    pub fn sort_items(&self) -> &[SortItem] {
        &self.sort_items
    }

    /// 获取限制数量
    pub fn limit(&self) -> i64 {
        self.limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_sort_node_creation() {
        let start_node = PlanNodeEnum::Start(StartNode::new());

        let sort_items = vec![
            SortItem::asc("name".to_string()),
            SortItem::desc("age".to_string()),
        ];

        let sort_node =
            SortNode::new(start_node, sort_items).expect("SortNode creation should succeed");

        assert_eq!(sort_node.type_name(), "SortNode");
        assert_eq!(sort_node.dependencies().len(), 1);
        assert_eq!(sort_node.sort_items().len(), 2);
        assert_eq!(sort_node.sort_items()[0].direction, OrderDirection::Asc);
        assert_eq!(sort_node.sort_items()[1].direction, OrderDirection::Desc);
    }

    #[test]
    fn test_limit_node_creation() {
        let start_node = PlanNodeEnum::Start(StartNode::new());

        let limit_node =
            LimitNode::new(start_node, 10, 100).expect("Limit node should be created successfully");

        assert_eq!(limit_node.type_name(), "LimitNode");
        assert_eq!(limit_node.dependencies().len(), 1);
        assert_eq!(limit_node.offset(), 10);
        assert_eq!(limit_node.count(), 100);
    }

    #[test]
    fn test_topn_node_creation() {
        let start_node = PlanNodeEnum::Start(StartNode::new());

        let sort_items = vec![
            SortItem::asc("name".to_string()),
            SortItem::desc("age".to_string()),
        ];
        let topn_node = TopNNode::new(start_node, sort_items, 10)
            .expect("TopN node should be created successfully");

        assert_eq!(topn_node.type_name(), "TopNNode");
        assert_eq!(topn_node.dependencies().len(), 1);
        assert_eq!(topn_node.sort_items().len(), 2);
        assert_eq!(topn_node.limit(), 10);
        assert_eq!(topn_node.sort_items()[0].direction, OrderDirection::Asc);
        assert_eq!(topn_node.sort_items()[1].direction, OrderDirection::Desc);
    }
}
