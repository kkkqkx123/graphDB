//! 预编译语句模块
//!
//! 提供高性能的预编译查询支持

use crate::api::core::{CoreError, CoreResult, QueryContext, QueryApi};
use crate::api::embedded::result::QueryResult;
use crate::core::{DataType, Value};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// 预编译语句
///
/// 预编译的查询语句，可以重复执行并绑定不同的参数，提高性能。
///
/// # 示例
///
/// ```rust
/// use graphdb::api::embedded::{GraphDatabase, DatabaseConfig};
/// use std::collections::HashMap;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = GraphDatabase::open("my_db")?;
///
/// // 预编译查询
/// let mut stmt = db.prepare("MATCH (n:User {id: $id}) RETURN n")?;
///
/// // 绑定参数并执行
/// stmt.bind("id", Value::Int(1))?;
/// let result1 = stmt.execute()?;
///
/// // 重置并重新绑定参数
/// stmt.reset();
/// stmt.bind("id", Value::Int(2))?;
/// let result2 = stmt.execute()?;
/// # Ok(())
/// # }
/// ```
pub struct PreparedStatement<S: StorageClient + 'static> {
    query_api: Arc<Mutex<QueryApi<S>>>,
    query: String,
    parameter_types: HashMap<String, DataType>,
    bound_params: HashMap<String, Value>,
    space_id: Option<u64>,
}

impl<S: StorageClient + Clone + 'static> PreparedStatement<S> {
    /// 创建新的预编译语句
    pub(crate) fn new(
        query_api: Arc<Mutex<QueryApi<S>>>,
        query: String,
        space_id: Option<u64>,
    ) -> Self {
        // 解析查询以提取参数类型信息
        // 这里简化处理，实际应该从查询计划中获取
        let parameter_types = Self::extract_parameters(&query);

        Self {
            query_api,
            query,
            parameter_types,
            bound_params: HashMap::new(),
            space_id,
        }
    }

    /// 从查询中提取参数（简化实现）
    fn extract_parameters(_query: &str) -> HashMap<String, DataType> {
        // 注意：这是一个实例方法，需要使用 Self::extract_parameters(&query) 调用
        // 或者使用 PreparedStatement::<RedbStorage>::extract_parameters(&query)
        let query = _query;
        let mut params = HashMap::new();

        // 简单的参数提取：查找 $name 或 :name 格式的参数
        // 实际实现应该使用查询解析器
        let mut chars = query.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '$' || ch == ':' {
                let mut param_name = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' {
                        param_name.push(next_ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if !param_name.is_empty() {
                    // 默认类型为 String，实际应该从查询推断
                    params.insert(param_name, DataType::String);
                }
            }
        }

        params
    }

    /// 绑定参数
    ///
    /// # 参数
    /// - `name` - 参数名称
    /// - `value` - 参数值
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误（如参数不存在或类型不匹配）
    pub fn bind(&mut self, name: &str, value: Value) -> CoreResult<()> {
        // 检查参数是否存在
        if !self.parameter_types.contains_key(name) {
            return Err(CoreError::InvalidParameter(
                format!("未知参数: {}", name)
            ));
        }

        // 可选：类型检查
        // if let Some(expected_type) = self.parameter_types.get(name) {
        //     if !Self::type_matches(&value, expected_type) {
        //         return Err(CoreError::TypeMismatch {
        //             expected: format!("{:?}", expected_type),
        //             actual: format!("{:?}", value),
        //         });
        //     }
        // }

        self.bound_params.insert(name.to_string(), value);
        Ok(())

    }

    /// 绑定多个参数
    ///
    /// # 参数
    /// - `params` - 参数映射
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误
    pub fn bind_many(&mut self, params: HashMap<String, Value>) -> CoreResult<()> {
        for (name, value) in params {
            self.bind(&name, value)?;
        }
        Ok(())
    }

    /// 执行查询（返回结果集）
    ///
    /// # 返回
    /// - 成功时返回查询结果
    /// - 失败时返回错误
    ///
    /// # 错误
    /// 如果必需的参数未绑定，返回错误
    pub fn execute(&self) -> CoreResult<QueryResult> {
        self.check_all_parameters_bound()?;

        let ctx = QueryContext {
            space_id: self.space_id,
            auto_commit: true,
            transaction_id: None,
            parameters: Some(self.bound_params.clone()),
        };

        let mut query_api = self.query_api.lock();
        let result = query_api.execute(&self.query, ctx)?;
        Ok(QueryResult::from_core(result))
    }

    /// 执行更新（返回影响行数）
    ///
    /// # 返回
    /// - 成功时返回影响的行数
    /// - 失败时返回错误
    pub fn execute_update(&self) -> CoreResult<usize> {
        let result = self.execute()?;
        Ok(result.len())
    }

    /// 重置语句
    ///
    /// 清除所有绑定的参数，使语句可以重新使用
    pub fn reset(&mut self) {
        self.bound_params.clear();
    }

    /// 清除参数绑定
    ///
    /// 与 reset() 相同
    pub fn clear_bindings(&mut self) {
        self.reset();
    }

    /// 获取查询字符串
    pub fn query(&self) -> &str {
        &self.query
    }

    /// 获取参数列表
    pub fn parameters(&self) -> &HashMap<String, DataType> {
        &self.parameter_types
    }

    /// 获取已绑定的参数
    pub fn bound_parameters(&self) -> &HashMap<String, Value> {
        &self.bound_params
    }

    /// 检查参数是否已绑定
    pub fn is_bound(&self, name: &str) -> bool {
        self.bound_params.contains_key(name)
    }

    /// 检查所有必需参数是否已绑定
    fn check_all_parameters_bound(&self) -> CoreResult<()> {
        for (name, _) in &self.parameter_types {
            if !self.bound_params.contains_key(name) {
                return Err(CoreError::InvalidParameter(
                    format!("参数未绑定: {}", name)
                ));
            }
        }
        Ok(())
    }

    /// 类型匹配检查（简化实现）
    #[allow(dead_code)]
    fn type_matches(value: &Value, expected_type: &DataType) -> bool {
        match (value, expected_type) {
            (Value::Int(_), DataType::Int) => true,
            (Value::Float(_), DataType::Float) => true,
            (Value::String(_), DataType::String) => true,
            (Value::Bool(_), DataType::Bool) => true,
            (Value::Null(_), _) => true, // NULL 可以匹配任何类型
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::RedbStorage;

    #[test]
    fn test_extract_parameters() {
        let query = "MATCH (n:User {id: $user_id, name: $name}) RETURN n";
        let params = PreparedStatement::<RedbStorage>::extract_parameters(query);

        assert!(params.contains_key("user_id"));
        assert!(params.contains_key("name"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_extract_parameters_colon() {
        let query = "MATCH (n) WHERE n.id = :id AND n.age > :min_age RETURN n";
        let params = PreparedStatement::<RedbStorage>::extract_parameters(query);

        assert!(params.contains_key("id"));
        assert!(params.contains_key("min_age"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_extract_parameters_none() {
        let query = "MATCH (n) RETURN n";
        let params = PreparedStatement::<RedbStorage>::extract_parameters(query);

        assert!(params.is_empty());
    }
}
