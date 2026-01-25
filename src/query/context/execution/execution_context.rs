//! 执行上下文模块
//!
//! 包含查询执行上下文、执行计划和执行响应。

use crate::core::Value;
use std::collections::HashMap;

/// 执行上下文
///
/// 存储查询执行过程中的变量和中间结果。
/// 对应原Nebula-Graph的ResultMap设计。
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    variables: HashMap<String, Value>,
}

impl ExecutionContext {
    /// 创建新的执行上下文
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// 设置变量值
    pub fn set_value(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// 获取变量值
    pub fn get_value(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    /// 获取可变变量值
    pub fn get_value_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.variables.get_mut(name)
    }

    /// 移除变量
    pub fn remove_value(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }

    /// 检查变量是否存在
    pub fn exists(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    /// 获取变量数量
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    /// 清空所有变量
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// 获取所有变量
    pub fn variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行计划
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub plan_id: i64,
    pub root_node: Option<PlanNode>,
    pub is_profile_enabled: bool,
}

impl ExecutionPlan {
    pub fn new(plan_id: i64) -> Self {
        Self {
            plan_id,
            root_node: None,
            is_profile_enabled: false,
        }
    }

    pub fn id(&self) -> i64 {
        self.plan_id
    }

    pub fn is_profile_enabled(&self) -> bool {
        self.is_profile_enabled
    }

    pub fn enable_profile(&mut self) {
        self.is_profile_enabled = true;
    }

    pub fn set_root_node(&mut self, node: PlanNode) {
        self.root_node = Some(node);
    }

    pub fn root_node(&self) -> Option<&PlanNode> {
        self.root_node.as_ref()
    }
}

/// 计划节点
#[derive(Debug, Clone)]
pub struct PlanNode {
    pub node_id: i64,
    pub node_type: String,
    pub children: Vec<PlanNode>,
    pub properties: HashMap<String, Value>,
}

impl PlanNode {
    pub fn new(node_id: i64, node_type: String) -> Self {
        Self {
            node_id,
            node_type,
            children: Vec::new(),
            properties: HashMap::new(),
        }
    }

    pub fn add_child(&mut self, child: PlanNode) {
        self.children.push(child);
    }

    pub fn set_property(&mut self, key: String, value: Value) {
        self.properties.insert(key, value);
    }
}

/// 执行响应
#[derive(Debug, Clone)]
pub struct ExecutionResponse {
    pub success: bool,
    pub data: Option<Value>,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
}

impl ExecutionResponse {
    pub fn new(success: bool) -> Self {
        Self {
            success,
            data: None,
            error_code: None,
            error_message: None,
            execution_time_ms: 0,
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_error(mut self, code: i32, message: String) -> Self {
        self.error_code = Some(code);
        self.error_message = Some(message);
        self.success = false;
        self
    }

    pub fn set_execution_time(&mut self, ms: u64) {
        self.execution_time_ms = ms;
    }
}
