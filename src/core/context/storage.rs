//! 存储上下文模块
//!
//! 提供存储层操作的上下文管理，整合自expression/context/storage.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::base::ContextType;
use super::traits::BaseContext;
use crate::core::Value;

/// 存储上下文
///
/// 管理存储层操作的上下文信息
#[derive(Debug, Clone)]
pub struct StorageContext {
    /// 上下文ID
    pub id: String,

    /// 存储空间ID
    pub space_id: i32,

    /// 会话ID
    pub session_id: i64,

    /// 事务ID（如果有）
    pub transaction_id: Option<i64>,

    /// 只读标志
    pub read_only: bool,

    /// 一致性级别
    pub consistency_level: ConsistencyLevel,

    /// 超时时间（毫秒）
    pub timeout_ms: u64,

    /// 重试次数
    pub retry_count: u32,

    /// 变量绑定
    pub variables: HashMap<String, Value>,

    /// 版本化变量
    pub versioned_variables: HashMap<String, Vec<Value>>,

    /// 内部变量
    pub inner_variables: HashMap<String, Value>,

    /// 自定义属性
    pub attributes: HashMap<String, Value>,

    /// 创建时间
    pub created_at: std::time::SystemTime,

    /// 最后更新时间
    pub updated_at: std::time::SystemTime,

    /// 是否有效
    pub valid: bool,
}

/// 一致性级别
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    /// 强一致性
    Strong,
    /// 最终一致性
    Eventual,
    /// 会话一致性
    Session,
    /// 单调读
    MonotonicRead,
    /// 单调写
    MonotonicWrite,
}

impl StorageContext {
    /// 创建新的存储上下文
    pub fn new(id: String, space_id: i32, session_id: i64) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            id,
            space_id,
            session_id,
            transaction_id: None,
            read_only: false,
            consistency_level: ConsistencyLevel::Strong,
            timeout_ms: 30000, // 30秒
            retry_count: 3,
            variables: HashMap::new(),
            versioned_variables: HashMap::new(),
            inner_variables: HashMap::new(),
            attributes: HashMap::new(),
            created_at: now,
            updated_at: now,
            valid: true,
        }
    }

    /// 设置事务ID
    pub fn set_transaction_id(&mut self, transaction_id: i64) {
        self.transaction_id = Some(transaction_id);
        self.touch();
    }

    /// 设置只读标志
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
        self.touch();
    }

    /// 设置一致性级别
    pub fn set_consistency_level(&mut self, level: ConsistencyLevel) {
        self.consistency_level = level;
        self.touch();
    }

    /// 设置超时时间
    pub fn set_timeout_ms(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
        self.touch();
    }

    /// 设置重试次数
    pub fn set_retry_count(&mut self, retry_count: u32) {
        self.retry_count = retry_count;
        self.touch();
    }

    /// 获取变量值（最新版本）
    pub fn get_var(&self, name: &str) -> Result<&Value, String> {
        self.variables
            .get(name)
            .ok_or_else(|| format!("Variable '{}' not found", name))
    }

    /// 获取指定版本的变量值
    pub fn get_versioned_var(&self, name: &str, version: i64) -> Result<&Value, String> {
        let versions = self
            .versioned_variables
            .get(name)
            .ok_or_else(|| format!("Versioned variable '{}' not found", name))?;

        let index = if version >= 0 {
            version as usize
        } else {
            // 负数索引从末尾开始
            let abs_version = (-version) as usize;
            if abs_version > versions.len() {
                return Err(format!(
                    "Version index {} out of range for variable '{}'",
                    version, name
                ));
            }
            versions.len() - abs_version
        };

        versions
            .get(index)
            .ok_or_else(|| format!("Version {} not found for variable '{}'", version, name))
    }

    /// 设置变量值
    pub fn set_var(&mut self, name: &str, value: Value) -> Result<(), String> {
        self.variables.insert(name.to_string(), value);

        // 同时添加到版本化变量
        let versions = self
            .versioned_variables
            .entry(name.to_string())
            .or_insert_with(Vec::new);
        versions.insert(0, self.variables[name].clone());

        self.touch();
        Ok(())
    }

    /// 设置表达式内部变量
    pub fn set_inner_var(&mut self, var: &str, value: Value) {
        self.inner_variables.insert(var.to_string(), value);
        self.touch();
    }

    /// 获取表达式内部变量
    pub fn get_inner_var(&self, var: &str) -> Option<&Value> {
        self.inner_variables.get(var)
    }

    /// 获取变量属性值
    pub fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, String> {
        let var_value = self.get_var(var)?;

        match var_value {
            Value::Map(props) => props
                .get(prop)
                .cloned()
                .ok_or_else(|| format!("Property '{}' not found in variable '{}'", prop, var)),
            _ => Err(format!(
                "Variable '{}' is not a map, cannot get property '{}'",
                var, prop
            )),
        }
    }

    /// 获取目标顶点属性值
    pub fn get_dst_prop(&self, tag: &str, prop: &str) -> Result<Value, String> {
        let dst_var = format!("{}_dst", tag);
        self.get_var_prop(&dst_var, prop)
    }

    /// 获取输入属性值
    pub fn get_input_prop(&self, prop: &str) -> Result<Value, String> {
        self.get_var_prop("__input__", prop)
    }

    /// 获取输入属性索引
    pub fn get_input_prop_index(&self, prop: &str) -> Result<usize, String> {
        let input_props = self.get_var("__input_props__")?;

        match input_props {
            Value::List(props) => {
                for (i, p) in props.iter().enumerate() {
                    if let Value::String(prop_name) = p {
                        if prop_name == prop {
                            return Ok(i);
                        }
                    }
                }
                Err(format!("Property '{}' not found in input properties", prop))
            }
            _ => Err("__input_props__ is not a list".to_string()),
        }
    }

    /// 按列索引获取值
    pub fn get_column(&self, index: i32) -> Result<Value, String> {
        let columns = self.get_var("__columns__")?;

        match columns {
            Value::List(cols) => {
                let idx = if index >= 0 {
                    index as usize
                } else {
                    cols.len() - (-index) as usize
                };
                cols.get(idx)
                    .cloned()
                    .ok_or_else(|| format!("Column index {} out of range", index))
            }
            _ => Err("__columns__ is not a list".to_string()),
        }
    }

    /// 获取标签属性值
    pub fn get_tag_prop(&self, tag: &str, prop: &str) -> Result<Value, String> {
        let tag_var = format!("{}_tag", tag);
        self.get_var_prop(&tag_var, prop)
    }

    /// 获取边属性值
    pub fn get_edge_prop(&self, edge: &str, prop: &str) -> Result<Value, String> {
        self.get_var_prop(edge, prop)
    }

    /// 获取源顶点属性值
    pub fn get_src_prop(&self, tag: &str, prop: &str) -> Result<Value, String> {
        let src_var = format!("{}_src", tag);
        self.get_var_prop(&src_var, prop)
    }

    /// 获取顶点
    pub fn get_vertex(&self, name: &str) -> Result<Value, String> {
        self.get_var(name).map(|v| v.clone())
    }

    /// 获取边
    pub fn get_edge(&self) -> Result<Value, String> {
        self.get_var("__edge__").map(|v| v.clone())
    }

    /// 初始化变量
    pub fn init_var(&mut self, name: &str) {
        self.variables
            .entry(name.to_string())
            .or_insert(Value::Null(Default::default()));
        self.versioned_variables
            .entry(name.to_string())
            .or_insert_with(Vec::new);
        self.touch();
    }

    /// 删除变量
    pub fn remove_var(&mut self, name: &str) -> Option<Value> {
        let removed = self.variables.remove(name);
        self.versioned_variables.remove(name);
        self.touch();
        removed
    }

    /// 清空所有变量
    pub fn clear_vars(&mut self) {
        self.variables.clear();
        self.versioned_variables.clear();
        self.inner_variables.clear();
        self.touch();
    }

    /// 获取变量数量
    pub fn var_count(&self) -> usize {
        self.variables.len()
    }

    /// 获取所有变量名
    pub fn var_names(&self) -> Vec<&str> {
        self.variables.keys().map(|k| k.as_str()).collect()
    }

    /// 检查变量是否存在
    pub fn has_var(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 获取变量的版本数量
    pub fn var_version_count(&self, name: &str) -> Option<usize> {
        self.versioned_variables.get(name).map(|v| v.len())
    }
}

impl BaseContext for StorageContext {
    fn id(&self) -> &str {
        &self.id
    }

    fn context_type(&self) -> ContextType {
        ContextType::Storage
    }

    fn created_at(&self) -> std::time::SystemTime {
        self.created_at
    }

    fn updated_at(&self) -> std::time::SystemTime {
        self.updated_at
    }

    fn is_valid(&self) -> bool {
        self.valid
    }

    fn touch(&mut self) {
        self.updated_at = std::time::SystemTime::now();
    }

    fn invalidate(&mut self) {
        self.valid = false;
        self.updated_at = std::time::SystemTime::now();
    }

    fn revalidate(&mut self) -> bool {
        self.valid = true;
        self.updated_at = std::time::SystemTime::now();
        true
    }

    fn parent_id(&self) -> Option<&str> {
        None
    }

    fn depth(&self) -> usize {
        1
    }

    fn get_attribute(&self, key: &str) -> Option<Value> {
        self.attributes.get(key).cloned()
    }

    fn set_attribute(&mut self, key: String, value: Value) {
        self.attributes.insert(key, value);
        self.updated_at = std::time::SystemTime::now();
    }

    fn attribute_keys(&self) -> Vec<String> {
        self.attributes.keys().cloned().collect()
    }

    fn remove_attribute(&mut self, key: &str) -> Option<Value> {
        let removed = self.attributes.remove(key);
        self.updated_at = std::time::SystemTime::now();
        removed
    }

    fn clear_attributes(&mut self) {
        self.attributes.clear();
        self.updated_at = std::time::SystemTime::now();
    }
}

impl Default for ConsistencyLevel {
    fn default() -> Self {
        Self::Strong
    }
}

impl Default for StorageContext {
    fn default() -> Self {
        Self::new("default_storage".to_string(), 0, 0)
    }
}
