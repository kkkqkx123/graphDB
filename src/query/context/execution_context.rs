//! 查询执行上下文模块 - 管理查询执行期间的上下文信息
//! 对应原C++中的ExecutionContext.h/cpp
//!
//! 注意：这是查询级别的执行上下文，不同于应用级别的 services::context::ExecutionContext

use std::collections::HashMap;
use std::sync::{RwLock, Arc};
use crate::core::{Result, Value};

/// 查询执行上下文
/// 
/// 每个查询请求的执行上下文，存储查询变量值和查询结果的多版本历史
/// 对应原C++中的ExecutionContext类
/// 
/// 与 services::context::ExecutionContext 的区别：
/// - services::context::ExecutionContext: 应用级，追踪单个操作的超时和统计
/// - QueryExecutionContext: 查询级，管理查询变量的多版本
#[derive(Debug, Clone)]
pub struct QueryExecutionContext {
    // name -> 多版本结果列表 (最新版本在前，最老版本在后)
    value_map: Arc<RwLock<HashMap<String, Vec<Result>>>>,
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
        let mut value_map = self.value_map.write()
            .expect("Failed to acquire write lock");
        value_map.entry(name.to_string()).or_insert_with(Vec::new);
    }

    /// 获取变量的最新值
    pub fn get_value(&self, name: &str) -> std::result::Result<Value, String> {
        let value_map = self.value_map.read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        if let Some(results) = value_map.get(name) {
            if let Some(result) = results.first() {
                return Ok(result.value().clone());
            } else {
                return Err("No results found for variable".to_string());
            }
        } else {
            Err("Variable not found".to_string())
        }
    }

    /// 获取变量的最新结果
    pub fn get_result(&self, name: &str) -> std::result::Result<Result, String> {
        let value_map = self.value_map.read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        if let Some(results) = value_map.get(name) {
            if let Some(result) = results.first() {
                return Ok(result.clone());
            } else {
                return Err("No results found for variable".to_string());
            }
        } else {
            Err("Variable not found".to_string())
        }
    }

    /// 获取变量的指定版本结果
    pub fn get_versioned_result(&self, name: &str, version: i64) -> std::result::Result<Result, String> {
        let value_map = self.value_map.read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        if let Some(results) = value_map.get(name) {
            // 版本处理：0是最新的，1是第二个最新，等等；-n表示第n个元素（与正索引访问相同元素）
            let index = if version >= 0 {
                // 从最新开始计数，0是最新，1是第二个最新，等等
                version as usize
            } else {
                // 负数索引与正数索引访问相同的元素：-1与1访问同一元素，等等
                (-version) as usize
            };

            // 检查索引范围
            if index >= results.len() {
                return Err("Version index out of range".to_string());
            }

            if let Some(result) = results.get(index) {
                Ok(result.clone())
            } else {
                Err("Version index out of range".to_string())
            }
        } else {
            Err("Variable not found".to_string())
        }
    }

    /// 设置变量的指定版本结果
    pub fn set_versioned_result(&self, name: &str, result: Result, version: i64) -> std::result::Result<(), String> {
        let mut value_map = self.value_map.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        let results = value_map.entry(name.to_string()).or_insert_with(Vec::new);

        // 版本处理：0是最新的，-1是前一个，依此类推；1是最老的，2是第二个老的，依此类推
        let index = if version >= 0 {
            // 从最新开始计数
            version as usize
        } else {
            // 从末尾开始计数
            if results.len() as i64 >= (-version) {
                results.len() - (-version) as usize
            } else {
                return Err("Version index out of range".to_string());
            }
        };

        if index < results.len() {
            results[index] = result;
        } else if index == results.len() {
            results.push(result);
        } else {
            return Err("Version index out of range".to_string());
        }

        Ok(())
    }

    /// 获取变量的版本数量
    pub fn num_versions(&self, name: &str) -> std::result::Result<usize, String> {
        let value_map = self.value_map.read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        if let Some(results) = value_map.get(name) {
            Ok(results.len())
        } else {
            Err("Variable not found".to_string())
        }
    }

    /// 获取变量的所有历史结果（最新的在前，最老的在后）
    pub fn get_history(&self, name: &str) -> std::result::Result<Vec<Result>, String> {
        let value_map = self.value_map.read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;

        if let Some(results) = value_map.get(name) {
            Ok(results.clone())
        } else {
            Err("Variable not found".to_string())
        }
    }

    /// 设置变量的最新值
    pub fn set_value(&self, name: &str, value: Value) -> std::result::Result<(), String> {
        let mut value_map = self.value_map.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        let results = value_map.entry(name.to_string()).or_insert_with(Vec::new);
        let result = Result::new(value, crate::core::ResultState::Success);
        results.insert(0, result); // 插入到最前面（最新位置）

        Ok(())
    }

    /// 设置变量的最新结果
    pub fn set_result(&self, name: &str, result: Result) -> std::result::Result<(), String> {
        let mut value_map = self.value_map.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        let results = value_map.entry(name.to_string()).or_insert_with(Vec::new);
        results.insert(0, result); // 插入到最前面（最新位置）

        Ok(())
    }

    /// 删除变量的结果
    pub fn drop_result(&self, name: &str) -> std::result::Result<(), String> {
        let mut value_map = self.value_map.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;

        value_map.remove(name);
        Ok(())
    }

    /// 只保留最近几个版本的结果
    pub fn trunc_history(&self, name: &str, num_versions_to_keep: usize) -> std::result::Result<(), String> {
        let mut value_map = self.value_map.write()
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
        ctx.set_value("test_var", value.clone()).unwrap();
        let retrieved_value = ctx.get_value("test_var").unwrap();
        assert_eq!(retrieved_value, value);
        
        // 测试结果操作
        let result = Result::new(Value::String("test_result".to_string()), crate::core::ResultState::Success);
        ctx.set_result("result_var", result.clone()).unwrap();
        let retrieved_result = ctx.get_result("result_var").unwrap();
        assert_eq!(retrieved_result, result);
    }

    #[test]
    fn test_versioned_operations() {
        let ctx = QueryExecutionContext::new();
        
        // 创建一些测试结果
        let result1 = Result::new(Value::Int(1), crate::core::ResultState::Success);
        let result2 = Result::new(Value::Int(2), crate::core::ResultState::Success);
        let result3 = Result::new(Value::Int(3), crate::core::ResultState::Success);
        
        // 设置不同版本的结果
        ctx.set_result("versioned_var", result1.clone()).unwrap();
        ctx.set_result("versioned_var", result2.clone()).unwrap();
        ctx.set_result("versioned_var", result3.clone()).unwrap();
        
        // 检查版本数量
        assert_eq!(ctx.num_versions("versioned_var").unwrap(), 3);
        
        // 获取历史记录
        let history = ctx.get_history("versioned_var").unwrap();
        assert_eq!(history.len(), 3);
        // 注意：历史记录是最新在前
        assert_eq!(history[0], result3); // 最新
        assert_eq!(history[1], result2);
        assert_eq!(history[2], result1); // 最老
        
        // 获取指定版本（0是最新的）
        assert_eq!(ctx.get_versioned_result("versioned_var", 0).unwrap(), result3);
        assert_eq!(ctx.get_versioned_result("versioned_var", -1).unwrap(), result2);
        assert_eq!(ctx.get_versioned_result("versioned_var", 1).unwrap(), result2);
        assert_eq!(ctx.get_versioned_result("versioned_var", 2).unwrap(), result1);
        
        // 版本截断
        ctx.trunc_history("versioned_var", 2).unwrap();
        assert_eq!(ctx.num_versions("versioned_var").unwrap(), 2);
    }
}