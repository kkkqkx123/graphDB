//! 连接节点实现
//!
//! 包含各种连接节点类型，如内连接、左连接等

use crate::core::types::ContextualExpression;
use crate::define_join_node;
use crate::define_binary_input_node;

define_join_node! {
    pub struct InnerJoinNode {
    }
    enum: InnerJoin
}

impl InnerJoinNode {
    pub fn new(
        left: super::plan_node_enum::PlanNodeEnum,
        right: super::plan_node_enum::PlanNodeEnum,
        hash_keys: Vec<ContextualExpression>,
        probe_keys: Vec<ContextualExpression>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = left.col_names().to_vec();
        col_names.extend(right.col_names().iter().cloned());

        let deps = vec![Box::new(left.clone()), Box::new(right.clone())];

        Ok(Self {
            id: -1,
            left: Box::new(left),
            right: Box::new(right),
            hash_keys,
            probe_keys,
            deps,
            output_var: None,
            col_names,
        })
    }
}

define_join_node! {
    pub struct LeftJoinNode {
    }
    enum: LeftJoin
}

impl LeftJoinNode {
    pub fn new(
        left: super::plan_node_enum::PlanNodeEnum,
        right: super::plan_node_enum::PlanNodeEnum,
        hash_keys: Vec<ContextualExpression>,
        probe_keys: Vec<ContextualExpression>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = left.col_names().to_vec();
        col_names.extend(right.col_names().iter().cloned());

        let deps = vec![Box::new(left.clone()), Box::new(right.clone())];

        Ok(Self {
            id: -1,
            left: Box::new(left),
            right: Box::new(right),
            hash_keys,
            probe_keys,
            deps,
            output_var: None,
            col_names,
        })
    }
}

define_binary_input_node! {
    pub struct CrossJoinNode {
    }
    enum: CrossJoin
    input: BinaryInputNode
}

impl CrossJoinNode {
    pub fn new(
        left: super::plan_node_enum::PlanNodeEnum,
        right: super::plan_node_enum::PlanNodeEnum,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = left.col_names().to_vec();
        col_names.extend(right.col_names().iter().cloned());

        let deps = vec![Box::new(left.clone()), Box::new(right.clone())];

        Ok(Self {
            id: -1,
            left: Box::new(left),
            right: Box::new(right),
            deps,
            output_var: None,
            col_names,
        })
    }
}

define_join_node! {
    pub struct HashInnerJoinNode {
    }
    enum: HashInnerJoin
}

impl HashInnerJoinNode {
    pub fn new(
        left: super::plan_node_enum::PlanNodeEnum,
        right: super::plan_node_enum::PlanNodeEnum,
        hash_keys: Vec<ContextualExpression>,
        probe_keys: Vec<ContextualExpression>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = left.col_names().to_vec();
        col_names.extend(right.col_names().iter().cloned());

        let deps = vec![Box::new(left.clone()), Box::new(right.clone())];

        Ok(Self {
            id: -1,
            left: Box::new(left),
            right: Box::new(right),
            hash_keys,
            probe_keys,
            deps,
            output_var: None,
            col_names,
        })
    }
}

define_join_node! {
    pub struct HashLeftJoinNode {
    }
    enum: HashLeftJoin
}

impl HashLeftJoinNode {
    pub fn new(
        left: super::plan_node_enum::PlanNodeEnum,
        right: super::plan_node_enum::PlanNodeEnum,
        hash_keys: Vec<ContextualExpression>,
        probe_keys: Vec<ContextualExpression>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = left.col_names().to_vec();
        col_names.extend(right.col_names().iter().cloned());

        let deps = vec![Box::new(left.clone()), Box::new(right.clone())];

        Ok(Self {
            id: -1,
            left: Box::new(left),
            right: Box::new(right),
            hash_keys,
            probe_keys,
            deps,
            output_var: None,
            col_names,
        })
    }
}

define_join_node! {
    pub struct FullOuterJoinNode {
    }
    enum: FullOuterJoin
}

impl FullOuterJoinNode {
    pub fn new(
        left: super::plan_node_enum::PlanNodeEnum,
        right: super::plan_node_enum::PlanNodeEnum,
        hash_keys: Vec<ContextualExpression>,
        probe_keys: Vec<ContextualExpression>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = left.col_names().to_vec();
        col_names.extend(right.col_names().iter().cloned());

        let deps = vec![Box::new(left.clone()), Box::new(right.clone())];

        Ok(Self {
            id: -1,
            left: Box::new(left),
            right: Box::new(right),
            hash_keys,
            probe_keys,
            deps,
            output_var: None,
            col_names,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;

    fn create_test_start_node(_id: i64) -> PlanNodeEnum {
        PlanNodeEnum::Start(StartNode::new())
    }

    #[test]
    fn test_inner_join_node_creation() {
        let left = create_test_start_node(1);
        let right = create_test_start_node(2);
        let hash_keys = vec![];
        let probe_keys = vec![];

        let node = InnerJoinNode::new(left, right, hash_keys, probe_keys);
        assert!(node.is_ok());
        let node = node.expect("InnerJoinNode创建应该成功");
        assert_eq!(node.type_name(), "InnerJoinNode");
        assert_eq!(node.id(), -1);
    }

    #[test]
    fn test_left_join_node_creation() {
        let left = create_test_start_node(1);
        let right = create_test_start_node(2);
        let hash_keys = vec![];
        let probe_keys = vec![];

        let node = LeftJoinNode::new(left, right, hash_keys, probe_keys);
        assert!(node.is_ok());
        let node = node.expect("LeftJoinNode创建应该成功");
        assert_eq!(node.type_name(), "LeftJoinNode");
        assert_eq!(node.id(), -1);
    }

    #[test]
    fn test_cross_join_node_creation() {
        let left = create_test_start_node(1);
        let right = create_test_start_node(2);

        let node = CrossJoinNode::new(left, right);
        assert!(node.is_ok());
        let node = node.expect("CrossJoinNode创建应该成功");
        assert_eq!(node.type_name(), "CrossJoinNode");
        assert_eq!(node.id(), -1);
    }

    #[test]
    fn test_hash_inner_join_node_creation() {
        let left = create_test_start_node(1);
        let right = create_test_start_node(2);
        let hash_keys = vec![];
        let probe_keys = vec![];

        let node = HashInnerJoinNode::new(left, right, hash_keys, probe_keys);
        assert!(node.is_ok());
        let node = node.expect("HashInnerJoinNode创建应该成功");
        assert_eq!(node.type_name(), "HashInnerJoinNode");
        assert_eq!(node.id(), -1);
    }

    #[test]
    fn test_hash_left_join_node_creation() {
        let left = create_test_start_node(1);
        let right = create_test_start_node(2);
        let hash_keys = vec![];
        let probe_keys = vec![];

        let node = HashLeftJoinNode::new(left, right, hash_keys, probe_keys);
        assert!(node.is_ok());
        let node = node.expect("HashLeftJoinNode创建应该成功");
        assert_eq!(node.type_name(), "HashLeftJoinNode");
        assert_eq!(node.id(), -1);
    }
}
