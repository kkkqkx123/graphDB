//! 连接节点实现
//!
//! 包含各种连接节点类型，如内连接、左连接等

use crate::core::Expression;
use crate::query::context::validate::types::Variable;

/// 内连接节点
///
/// 根据指定的连接键对两个输入进行内连接
#[derive(Debug, Clone)]
pub struct InnerJoinNode {
    id: i64,
    left: Box<super::plan_node_enum::PlanNodeEnum>,
    right: Box<super::plan_node_enum::PlanNodeEnum>,
    hash_keys: Vec<Expression>,
    probe_keys: Vec<Expression>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    // 内部存储的依赖向量，用于快速访问
    inner_deps: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
}

impl InnerJoinNode {
    /// 创建新的内连接节点
    pub fn new(
        left: super::plan_node_enum::PlanNodeEnum,
        right: super::plan_node_enum::PlanNodeEnum,
        hash_keys: Vec<Expression>,
        probe_keys: Vec<Expression>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = left.col_names().to_vec();
        col_names.extend(right.col_names().iter().cloned());

        let inner_deps = vec![Box::new(left.clone()), Box::new(right.clone())];

        Ok(Self {
            id: -1,
            left: Box::new(left),
            right: Box::new(right),
            hash_keys,
            probe_keys,
            output_var: None,
            col_names,
            cost: 0.0,
            inner_deps,
        })
    }

    /// 获取哈希键
    pub fn hash_keys(&self) -> &[Expression] {
        &self.hash_keys
    }

    /// 获取探测键
    pub fn probe_keys(&self) -> &[Expression] {
        &self.probe_keys
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "InnerJoin"
    }

    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.inner_deps
    }

    pub fn add_dependency(&mut self, _dep: super::plan_node_enum::PlanNodeEnum) {
        // 内连接节点不支持添加依赖，它需要恰好两个输入
        // 在实际使用中，内连接节点在创建时就确定了依赖
        panic!("内连接节点不支持添加依赖，它需要恰好两个输入")
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        let initial_len = self.inner_deps.len();
        self.inner_deps.retain(|dep| dep.id() != id);
        let final_len = self.inner_deps.len();

        if initial_len != final_len {
            // 更新 left 和 right 输入，如果原来的输入被移除
            if self.left.id() == id {
                if let Some(new_left) = self.inner_deps.get(0) {
                    self.left = new_left.clone();
                }
            }
            if self.right.id() == id {
                if let Some(new_right) = self.inner_deps.get(1) {
                    self.right = new_right.clone();
                }
            }
            true
        } else {
            false
        }
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::InnerJoin(Self {
            id: self.id,
            left: self.left.clone(),
            right: self.right.clone(),
            hash_keys: self.hash_keys.clone(),
            probe_keys: self.probe_keys.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            inner_deps: self.inner_deps.clone(),
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::InnerJoin(cloned)
    }
}

/// 左连接节点
///
/// 根据指定的连接键对两个输入进行左连接
#[derive(Debug, Clone)]
pub struct LeftJoinNode {
    id: i64,
    left: Box<super::plan_node_enum::PlanNodeEnum>,
    right: Box<super::plan_node_enum::PlanNodeEnum>,
    hash_keys: Vec<Expression>,
    probe_keys: Vec<Expression>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    inner_deps: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
}

impl LeftJoinNode {
    /// 创建新的左连接节点
    pub fn new(
        left: super::plan_node_enum::PlanNodeEnum,
        right: super::plan_node_enum::PlanNodeEnum,
        hash_keys: Vec<Expression>,
        probe_keys: Vec<Expression>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = left.col_names().to_vec();
        col_names.extend(right.col_names().iter().cloned());

        let inner_deps = vec![Box::new(left.clone()), Box::new(right.clone())];

        Ok(Self {
            id: -1,
            left: Box::new(left),
            right: Box::new(right),
            hash_keys,
            probe_keys,
            output_var: None,
            col_names,
            cost: 0.0,
            inner_deps,
        })
    }

    /// 获取哈希键
    pub fn hash_keys(&self) -> &[Expression] {
        &self.hash_keys
    }

    /// 获取探测键
    pub fn probe_keys(&self) -> &[Expression] {
        &self.probe_keys
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "LeftJoin"
    }

    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.inner_deps
    }

    pub fn add_dependency(&mut self, _dep: super::plan_node_enum::PlanNodeEnum) {
        panic!("左连接节点不支持添加依赖，它需要恰好两个输入")
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        let initial_len = self.inner_deps.len();
        self.inner_deps.retain(|dep| dep.id() != id);
        let final_len = self.inner_deps.len();

        if initial_len != final_len {
            if self.left.id() == id {
                if let Some(new_left) = self.inner_deps.get(0) {
                    self.left = new_left.clone();
                }
            }
            if self.right.id() == id {
                if let Some(new_right) = self.inner_deps.get(1) {
                    self.right = new_right.clone();
                }
            }
            true
        } else {
            false
        }
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::LeftJoin(Self {
            id: self.id,
            left: self.left.clone(),
            right: self.right.clone(),
            hash_keys: self.hash_keys.clone(),
            probe_keys: self.probe_keys.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            inner_deps: self.inner_deps.clone(),
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::LeftJoin(cloned)
    }
}

/// 交叉连接节点
///
/// 对两个输入进行笛卡尔积连接
#[derive(Debug, Clone)]
pub struct CrossJoinNode {
    id: i64,
    left: Box<super::plan_node_enum::PlanNodeEnum>,
    right: Box<super::plan_node_enum::PlanNodeEnum>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    inner_deps: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
}

impl CrossJoinNode {
    /// 创建新的交叉连接节点
    pub fn new(
        left: super::plan_node_enum::PlanNodeEnum,
        right: super::plan_node_enum::PlanNodeEnum,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = left.col_names().to_vec();
        col_names.extend(right.col_names().iter().cloned());

        let inner_deps = vec![Box::new(left.clone()), Box::new(right.clone())];

        Ok(Self {
            id: -1,
            left: Box::new(left),
            right: Box::new(right),
            output_var: None,
            col_names,
            cost: 0.0,
            inner_deps,
        })
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "CrossJoin"
    }

    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.inner_deps
    }

    pub fn add_dependency(&mut self, _dep: super::plan_node_enum::PlanNodeEnum) {
        panic!("交叉连接节点不支持添加依赖，它需要恰好两个输入")
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        let initial_len = self.inner_deps.len();
        self.inner_deps.retain(|dep| dep.id() != id);
        let final_len = self.inner_deps.len();

        if initial_len != final_len {
            if self.left.id() == id {
                if let Some(new_left) = self.inner_deps.get(0) {
                    self.left = new_left.clone();
                }
            }
            if self.right.id() == id {
                if let Some(new_right) = self.inner_deps.get(1) {
                    self.right = new_right.clone();
                }
            }
            true
        } else {
            false
        }
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::CrossJoin(Self {
            id: self.id,
            left: self.left.clone(),
            right: self.right.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            inner_deps: self.inner_deps.clone(),
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::CrossJoin(cloned)
    }
}

#[cfg(test)]
mod tests {
    use super::super::start_node::StartNode;
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_inner_join_node_creation() {
        let left_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                StartNode::new(),
            );
        let right_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                StartNode::new(),
            );

        let hash_keys = vec![Expression::Variable("key".to_string())];
        let probe_keys = vec![Expression::Variable("key".to_string())];

        let join_node = InnerJoinNode::new(left_node, right_node, hash_keys, probe_keys)
            .expect("Join node should be created successfully");

        assert_eq!(join_node.type_name(), "InnerJoin");
        assert_eq!(join_node.dependencies().len(), 2);
        assert_eq!(join_node.hash_keys().len(), 1);
        assert_eq!(join_node.probe_keys().len(), 1);
    }

    #[test]
    fn test_inner_join_node_dependencies() {
        let left_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                StartNode::new(),
            );
        let right_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                StartNode::new(),
            );

        let hash_keys = vec![Expression::Variable("key".to_string())];
        let probe_keys = vec![Expression::Variable("key".to_string())];

        let join_node =
            InnerJoinNode::new(left_node.clone(), right_node.clone(), hash_keys, probe_keys)
                .expect("Join node should be created successfully");

        // 测试依赖管理
        assert_eq!(join_node.dependencies().len(), 2);
    }
}
