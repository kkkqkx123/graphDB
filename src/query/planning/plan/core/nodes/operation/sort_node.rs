//! Implementation of sorting nodes
//!
//! The `SortNode` is used to perform sorting operations on the input data.

use crate::core::types::graph_schema::OrderDirection;
use crate::define_plan_node_with_deps;

/// Sorting item definition
/// Includes column names and sorting direction.
#[derive(Debug, Clone, PartialEq)]
pub struct SortItem {
    /// Sort column names
    pub column: String,
    /// Sorting direction
    pub direction: OrderDirection,
}

impl SortItem {
    /// Create a new sorting item.
    pub fn new(column: String, direction: OrderDirection) -> Self {
        Self { column, direction }
    }

    /// Create items for ascending sorting.
    pub fn asc(column: String) -> Self {
        Self::new(column, OrderDirection::Asc)
    }

    /// Create descending order sorting items
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
    /// Create a new sorting node.
    pub fn new(
        input: crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum,
        sort_items: Vec<SortItem>,
    ) -> Result<Self, crate::query::planning::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![input],
            sort_items,
            limit: None,
            output_var: None,
            col_names,
        })
    }

    /// Obtain the sorted fields
    pub fn sort_items(&self) -> &[SortItem] {
        &self.sort_items
    }

    /// Obtain a limited quantity.
    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    /// Set a limit on the number of items.
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
    /// Create a new restriction node.
    pub fn new(
        input: crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum,
        offset: i64,
        count: i64,
    ) -> Result<Self, crate::query::planning::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![input],
            offset,
            count,
            output_var: None,
            col_names,
        })
    }

    /// Obtain the offset value.
    pub fn offset(&self) -> i64 {
        self.offset
    }

    /// Obtain the count.
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
    /// Create new TopN nodes.
    pub fn new(
        input: crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum,
        sort_items: Vec<SortItem>,
        limit: i64,
    ) -> Result<Self, crate::query::planning::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![input],
            sort_items,
            limit,
            output_var: None,
            col_names,
        })
    }

    /// Get Sorted Fields
    pub fn sort_items(&self) -> &[SortItem] {
        &self.sort_items
    }

    /// Access to restricted quantities
    pub fn limit(&self) -> i64 {
        self.limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum;
    use crate::query::planning::plan::core::nodes::control_flow::start_node::StartNode;

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
