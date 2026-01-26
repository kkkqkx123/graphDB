//! 连接节点实现
//!
//! 包含各种连接节点类型，如内连接、左连接等

use crate::core::Expression;
use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;

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

    pub fn add_dependency(&mut self, _dep: super::plan_node_enum::PlanNodeEnum) -> Result<(), PlannerError> {
        Err(PlannerError::InvalidOperation(
            "内连接节点不支持添加依赖，它需要恰好两个输入".to_string(),
        ))
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

// 为 InnerJoinNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for InnerJoinNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.type_name()
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn cost(&self) -> f64 {
        self.cost()
    }

    fn set_output_var(&mut self, var: Variable) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::InnerJoin(self)
    }
}

// 为 InnerJoinNode 实现 BinaryInputNode trait
impl super::plan_node_traits::BinaryInputNode for InnerJoinNode {
    fn left_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.left
    }

    fn right_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.right
    }

    fn set_left_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.left = Box::new(input);
        self.inner_deps[0] = self.left.clone();
    }

    fn set_right_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.right = Box::new(input);
        self.inner_deps[1] = self.right.clone();
    }
}

// 为 InnerJoinNode 实现 JoinNode trait
impl super::plan_node_traits::JoinNode for InnerJoinNode {
    fn hash_keys(&self) -> &[Expression] {
        &self.hash_keys
    }

    fn probe_keys(&self) -> &[Expression] {
        &self.probe_keys
    }
}

// 为 InnerJoinNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for InnerJoinNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
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

    pub fn add_dependency(&mut self, _dep: super::plan_node_enum::PlanNodeEnum) -> Result<(), PlannerError> {
        Err(PlannerError::InvalidOperation(
            "左连接节点不支持添加依赖，它需要恰好两个输入".to_string(),
        ))
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

// 为 LeftJoinNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for LeftJoinNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.type_name()
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn cost(&self) -> f64 {
        self.cost()
    }

    fn set_output_var(&mut self, var: Variable) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::LeftJoin(self)
    }
}

// 为 LeftJoinNode 实现 BinaryInputNode trait
impl super::plan_node_traits::BinaryInputNode for LeftJoinNode {
    fn left_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.left
    }

    fn right_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.right
    }

    fn set_left_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.left = Box::new(input);
        self.inner_deps[0] = self.left.clone();
    }

    fn set_right_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.right = Box::new(input);
        self.inner_deps[1] = self.right.clone();
    }
}

// 为 LeftJoinNode 实现 JoinNode trait
impl super::plan_node_traits::JoinNode for LeftJoinNode {
    fn hash_keys(&self) -> &[Expression] {
        &self.hash_keys
    }

    fn probe_keys(&self) -> &[Expression] {
        &self.probe_keys
    }
}

// 为 LeftJoinNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for LeftJoinNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
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

    pub fn add_dependency(&mut self, _dep: super::plan_node_enum::PlanNodeEnum) -> Result<(), PlannerError> {
        Err(PlannerError::InvalidOperation(
            "交叉连接节点不支持添加依赖，它需要恰好两个输入".to_string(),
        ))
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

// 为 CrossJoinNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for CrossJoinNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.type_name()
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn cost(&self) -> f64 {
        self.cost()
    }

    fn set_output_var(&mut self, var: Variable) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::CrossJoin(self)
    }
}

// 为 CrossJoinNode 实现 BinaryInputNode trait
impl super::plan_node_traits::BinaryInputNode for CrossJoinNode {
    fn left_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.left
    }

    fn right_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.right
    }

    fn set_left_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.left = Box::new(input);
        self.inner_deps[0] = self.left.clone();
    }

    fn set_right_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.right = Box::new(input);
        self.inner_deps[1] = self.right.clone();
    }
}

// 为 CrossJoinNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for CrossJoinNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

/// 哈希内连接节点
///
/// 使用哈希连接算法对两个输入进行内连接
#[derive(Debug, Clone)]
pub struct HashInnerJoinNode {
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

impl HashInnerJoinNode {
    /// 创建新的哈希内连接节点
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
        "HashInnerJoin"
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

    pub fn add_dependency(&mut self, _dep: super::plan_node_enum::PlanNodeEnum) -> Result<(), PlannerError> {
        Err(PlannerError::InvalidOperation(
            "哈希内连接节点不支持添加依赖，它需要恰好两个输入".to_string(),
        ))
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
        super::plan_node_enum::PlanNodeEnum::HashInnerJoin(Self {
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
        super::plan_node_enum::PlanNodeEnum::HashInnerJoin(cloned)
    }
}

// 为 HashInnerJoinNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for HashInnerJoinNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.type_name()
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn cost(&self) -> f64 {
        self.cost()
    }

    fn set_output_var(&mut self, var: Variable) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::HashInnerJoin(self)
    }
}

// 为 HashInnerJoinNode 实现 BinaryInputNode trait
impl super::plan_node_traits::BinaryInputNode for HashInnerJoinNode {
    fn left_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.left
    }

    fn right_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.right
    }

    fn set_left_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.left = Box::new(input);
        self.inner_deps[0] = self.left.clone();
    }

    fn set_right_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.right = Box::new(input);
        self.inner_deps[1] = self.right.clone();
    }
}

// 为 HashInnerJoinNode 实现 JoinNode trait
impl super::plan_node_traits::JoinNode for HashInnerJoinNode {
    fn hash_keys(&self) -> &[Expression] {
        &self.hash_keys
    }

    fn probe_keys(&self) -> &[Expression] {
        &self.probe_keys
    }
}

// 为 HashInnerJoinNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for HashInnerJoinNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

/// 哈希左连接节点
///
/// 使用哈希连接算法对两个输入进行左连接
#[derive(Debug, Clone)]
pub struct HashLeftJoinNode {
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

impl HashLeftJoinNode {
    /// 创建新的哈希左连接节点
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
        "HashLeftJoin"
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

    pub fn add_dependency(&mut self, _dep: super::plan_node_enum::PlanNodeEnum) -> Result<(), PlannerError> {
        Err(PlannerError::InvalidOperation(
            "哈希左连接节点不支持添加依赖，它需要恰好两个输入".to_string(),
        ))
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
        super::plan_node_enum::PlanNodeEnum::HashLeftJoin(Self {
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
        super::plan_node_enum::PlanNodeEnum::HashLeftJoin(cloned)
    }
}

// 为 HashLeftJoinNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for HashLeftJoinNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.type_name()
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn cost(&self) -> f64 {
        self.cost()
    }

    fn set_output_var(&mut self, var: Variable) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::HashLeftJoin(self)
    }
}

// 为 HashLeftJoinNode 实现 BinaryInputNode trait
impl super::plan_node_traits::BinaryInputNode for HashLeftJoinNode {
    fn left_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.left
    }

    fn right_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.right
    }

    fn set_left_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.left = Box::new(input);
        self.inner_deps[0] = self.left.clone();
    }

    fn set_right_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.right = Box::new(input);
        self.inner_deps[1] = self.right.clone();
    }
}

// 为 HashLeftJoinNode 实现 JoinNode trait
impl super::plan_node_traits::JoinNode for HashLeftJoinNode {
    fn hash_keys(&self) -> &[Expression] {
        &self.hash_keys
    }

    fn probe_keys(&self) -> &[Expression] {
        &self.probe_keys
    }
}

// 为 HashLeftJoinNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for HashLeftJoinNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

/// 连接器
///
/// 用于连接两个子计划的工具
#[derive(Debug)]
pub struct JoinConnector;

impl JoinConnector {
    pub fn new() -> Self {
        Self
    }

    pub fn cartesian_product(
        _qctx: &crate::query::context::ast::base::AstContext,
        left: &SubPlan,
        right: &SubPlan,
    ) -> Result<SubPlan, crate::query::planner::planner::PlannerError> {
        if left.root.is_none() || right.root.is_none() {
            return Ok(if left.root.is_some() { left.clone() } else { right.clone() });
        }

        let left_root = left.root.as_ref().expect("Left plan root should exist");
        let right_root = right.root.as_ref().expect("Right plan root should exist");

        let cross_join_node = CrossJoinNode::new(left_root.clone(), right_root.clone())
            .map_err(|e| crate::query::planner::planner::PlannerError::PlanGenerationFailed(format!("Failed to create cross join node: {}", e)))?;

        let cross_join_enum = super::plan_node_enum::PlanNodeEnum::CrossJoin(cross_join_node);

        Ok(SubPlan::new(
            Some(cross_join_enum.clone()),
            Some(cross_join_enum),
        ))
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
