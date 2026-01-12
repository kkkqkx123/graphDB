//! 查询执行上下文模块 - 管理查询执行期间的上下文信息
//!
//! 提供查询执行期间的变量管理和结果历史记录功能

use crate::core::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 查询执行上下文
///
/// 每个查询请求的执行上下文，存储查询变量值和查询结果的多版本历史
///
/// 与应用级执行上下文的区别：
/// - 应用级执行上下文: 应用级，追踪单个操作的超时和统计
/// - QueryExecutionContext: 查询级，管理查询变量的多版本
#[derive(Debug, Clone)]
pub struct QueryExecutionContext {
    // name -> 多版本值列表 (最新版本在前，最老版本在后)
    value_map: Arc<RwLock<HashMap<String, Vec<Value>>>>,
}

impl QueryExecutionContext {
    /// 创建新的查询执行上下文
    pub fn new() -> Self {
        Self {
            value_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 初始化变量
    pub fn init_var(&self, name: &str) {
        let mut value_map = self
            .value_map
            .write()
            .expect("Failed to acquire write lock");
        value_map.entry(name.to_string()).or_insert_with(Vec::new);
    }

    /// 获取变量的最新值
    pub fn get_value(&self, name: &str) -> std::result::Result<Value, String> {
        let value_map = self
            .value_map
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        if let Some(values) = value_map.get(name) {
            if let Some(value) = values.first() {
                return Ok(value.clone());
            } else {
                return Err("No values found for variable".to_string());
            }
        } else {
            Err("Variable not found".to_string())
        }
    }

    /// 获取变量的指定版本值
    pub fn get_versioned_value(
        &self,
        name: &str,
        version: i64,
    ) -> std::result::Result<Value, String> {
        let value_map = self
            .value_map
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        if let Some(values) = value_map.get(name) {
            // 版本处理：0是最新的，1是第二个最新，等等；-n表示第n个元素（与正索引访问相同元素）
            let index = if version >= 0 {
                // 从最新开始计数，0是最新，1是第二个最新，等等
                version as usize
            } else {
                // 负数索引与正数索引访问相同的元素：-1与1访问同一元素，等等
                (-version) as usize
            };

            // 检查索引范围
            if index >= values.len() {
                return Err("Version index out of range".to_string());
            }

            if let Some(value) = values.get(index) {
                Ok(value.clone())
            } else {
                Err("Version index out of range".to_string())
            }
        } else {
            Err("Variable not found".to_string())
        }
    }

    /// 设置变量的指定版本值
    pub fn set_versioned_value(
        &self,
        name: &str,
        value: Value,
        version: i64,
    ) -> std::result::Result<(), String> {
        let mut value_map = self
            .value_map
            .write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        let values = value_map.entry(name.to_string()).or_insert_with(Vec::new);

        // 版本处理：0是最新的，-1是前一个，依此类推；1是最老的，2是第二个老的，依此类推
        let index = if version >= 0 {
            // 从最新开始计数
            version as usize
        } else {
            // 从末尾开始计数
            if values.len() as i64 >= (-version) {
                values.len() - (-version) as usize
            } else {
                return Err("Version index out of range".to_string());
            }
        };

        if index < values.len() {
            values[index] = value;
        } else if index == values.len() {
            values.push(value);
        } else {
            return Err("Version index out of range".to_string());
        }

        Ok(())
    }

    /// 获取变量的版本数量
    pub fn num_versions(&self, name: &str) -> std::result::Result<usize, String> {
        let value_map = self
            .value_map
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        if let Some(results) = value_map.get(name) {
            Ok(results.len())
        } else {
            Err("Variable not found".to_string())
        }
    }

    /// 获取变量的所有历史值（最新的在前，最老的在后）
    pub fn get_history(&self, name: &str) -> std::result::Result<Vec<Value>, String> {
        let value_map = self
            .value_map
            .read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        if let Some(values) = value_map.get(name) {
            Ok(values.clone())
        } else {
            Err("Variable not found".to_string())
        }
    }

    /// 设置变量的最新值
    pub fn set_value(&self, name: &str, value: Value) -> std::result::Result<(), String> {
        let mut value_map = self
            .value_map
            .write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        let values = value_map.entry(name.to_string()).or_insert_with(Vec::new);
        values.insert(0, value); // 插入到最前面（最新位置）

        Ok(())
    }

    /// 删除变量的结果
    pub fn drop_result(&self, name: &str) -> std::result::Result<(), String> {
        let mut value_map = self
            .value_map
            .write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        value_map.remove(name);
        Ok(())
    }

    /// 只保留最近几个版本的结果
    pub fn trunc_history(
        &self,
        name: &str,
        num_versions_to_keep: usize,
    ) -> std::result::Result<(), String> {
        let mut value_map = self
            .value_map
            .write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        if let Some(results) = value_map.get_mut(name) {
            if results.len() > num_versions_to_keep {
                results.truncate(num_versions_to_keep);
            }
            Ok(())
        } else {
            Err("Variable not found".to_string())
        }
    }

    /// 检查变量是否存在
    pub fn exists(&self, name: &str) -> bool {
        let value_map = match self.value_map.read() {
            Ok(map) => map,
            Err(_) => return false, // 如果无法获取读锁，返回false
        };

        value_map.contains_key(name)
    }

    /// 获取变量数量
    pub fn variable_count(&self) -> usize {
        let value_map = match self.value_map.read() {
            Ok(map) => map,
            Err(_) => return 0, // 如果无法获取读锁，返回0
        };

        value_map.len()
    }
}

impl Default for QueryExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_execution_context() {
        let ctx = QueryExecutionContext::new();

        // 测试初始化变量
        ctx.init_var("test_var");
        assert!(ctx.exists("test_var"));

        // 测试设置和获取值
        let value = Value::Int(42);
        ctx.set_value("test_var", value.clone())
            .expect("Expected successful setting of test variable");
        let retrieved_value = ctx
            .get_value("test_var")
            .expect("Expected successful retrieval of test value");
        assert_eq!(retrieved_value, value);
    }

    #[test]
    fn test_versioned_operations() {
        let ctx = QueryExecutionContext::new();

        // 创建一些测试值
        let value1 = Value::Int(1);
        let value2 = Value::Int(2);
        let value3 = Value::Int(3);

        // 设置不同版本的值
        ctx.set_value("versioned_var", value1.clone())
            .expect("Expected successful setting of version 1");
        ctx.set_value("versioned_var", value2.clone())
            .expect("Expected successful setting of version 2");
        ctx.set_value("versioned_var", value3.clone())
            .expect("Expected successful setting of version 3");

        // 检查版本数量
        assert_eq!(
            ctx.num_versions("versioned_var")
                .expect("Expected successful version count check"),
            3
        );

        // 获取历史记录
        let history = ctx
            .get_history("versioned_var")
            .expect("Expected successful history retrieval");
        assert_eq!(history.len(), 3);
        // 注意：历史记录是最新在前
        assert_eq!(history[0], value3); // 最新
        assert_eq!(history[1], value2);
        assert_eq!(history[2], value1); // 最老

        // 获取指定版本（0是最新的）
        assert_eq!(
            ctx.get_versioned_value("versioned_var", 0)
                .expect("Expected successful retrieval of version 0"),
            value3
        );
        assert_eq!(
            ctx.get_versioned_value("versioned_var", -1)
                .expect("Expected successful retrieval of version -1"),
            value2
        );
        assert_eq!(
            ctx.get_versioned_value("versioned_var", 1)
                .expect("Expected successful retrieval of version 1"),
            value2
        );
        assert_eq!(
            ctx.get_versioned_value("versioned_var", 2)
                .expect("Expected successful retrieval of version 2"),
            value1
        );

        // 版本截断
        ctx.trunc_history("versioned_var", 2)
            .expect("Expected successful truncation of history");
        assert_eq!(
            ctx.num_versions("versioned_var")
                .expect("Expected successful version count after truncation"),
            2
        );
    }

    #[test]
    fn test_variable_count() {
        let ctx = QueryExecutionContext::new();

        // 初始变量数量应为0
        assert_eq!(ctx.variable_count(), 0);

        // 添加变量
        ctx.set_value("var1", Value::Int(1))
            .expect("Expected successful setting of var1");
        assert_eq!(ctx.variable_count(), 1);

        // 添加更多变量
        ctx.set_value("var2", Value::String("test".to_string()))
            .expect("Expected successful setting of var2");
        ctx.set_value("var3", Value::Bool(true))
            .expect("Expected successful setting of var3");
        assert_eq!(ctx.variable_count(), 3);

        // 删除变量
        ctx.drop_result("var2")
            .expect("Expected successful dropping of var2");
        assert_eq!(ctx.variable_count(), 2);

        // 删除不存在的变量不应影响计数
        ctx.drop_result("non_existent")
            .expect("Expected successful dropping of non-existent var");
        assert_eq!(ctx.variable_count(), 2);
    }
}
