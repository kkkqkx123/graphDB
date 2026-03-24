//! 预编译语句核心模块
//!
//! 提供预编译语句的核心功能，包括参数绑定、查询执行等

use crate::api::core::{CoreError, CoreResult, QueryApi, QueryRequest};
use crate::api::embedded::result::QueryResult;
use crate::api::embedded::precompiled::config::{ExecutionStats, ParameterInfo, StatementConfig};
use crate::api::embedded::precompiled::parameter_extractor::{extract_parameters, type_matches};
use crate::core::{DataType, Value};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 预编译语句
///
/// 预编译的查询语句，可以重复执行并绑定不同的参数，提高性能。
/// 支持查询计划缓存、类型检查、批量执行等功能。
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
///
/// // 查看执行统计
/// let stats = stmt.stats();
/// println!("执行次数: {}", stats.execution_count);
/// # Ok(())
/// # }
/// ```
pub struct PreparedStatement<S: StorageClient + 'static> {
    query_api: Arc<Mutex<QueryApi<S>>>,
    query: String,
    parameter_types: HashMap<String, DataType>,
    bound_params: HashMap<String, Value>,
    space_id: Option<u64>,
    config: StatementConfig,
    stats: ExecutionStats,
    execution_history: Vec<Duration>,
}

impl<S: StorageClient + Clone + 'static> PreparedStatement<S> {
    /// 创建新的预编译语句
    pub(crate) fn new(
        query_api: Arc<Mutex<QueryApi<S>>>,
        query: String,
        space_id: Option<u64>,
    ) -> CoreResult<Self> {
        Self::with_config(query_api, query, space_id, StatementConfig::default())
    }

    /// 使用配置创建预编译语句
    pub(crate) fn with_config(
        query_api: Arc<Mutex<QueryApi<S>>>,
        query: String,
        space_id: Option<u64>,
        config: StatementConfig,
    ) -> CoreResult<Self> {
        let parameter_types = extract_parameters(&query)?;

        Ok(Self {
            query_api,
            query,
            parameter_types,
            bound_params: HashMap::new(),
            space_id,
            config,
            stats: ExecutionStats::new(),
            execution_history: Vec::new(),
        })
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
        if !self.parameter_types.contains_key(name) {
            return Err(CoreError::InvalidParameter(format!("未知参数: {}", name)));
        }

        if self.config.enable_type_check {
            if let Some(expected_type) = self.parameter_types.get(name) {
                if !type_matches(&value, expected_type) {
                    return Err(CoreError::InvalidParameter(format!(
                        "类型不匹配: 期望 {:?}, 实际 {:?}",
                        expected_type, value
                    )));
                }
            }
        }

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
    pub fn execute(&mut self) -> CoreResult<QueryResult> {
        self.check_all_parameters_bound()?;

        let start = Instant::now();

        let ctx = QueryRequest {
            space_id: self.space_id,
            auto_commit: true,
            transaction_id: None,
            parameters: Some(self.bound_params.clone()),
        };

        let result = {
            let mut query_api = self.query_api.lock();
            query_api.execute(&self.query, ctx)?
        };

        let duration = start.elapsed();
        self.record_execution(duration);

        Ok(QueryResult::from_core(result))
    }

    /// 执行更新（返回影响行数）
    ///
    /// # 返回
    /// - 成功时返回影响的行数
    /// - 失败时返回错误
    pub fn execute_update(&mut self) -> CoreResult<usize> {
        let result = self.execute()?;
        Ok(result.len())
    }

    /// 批量执行
    ///
    /// 使用不同的参数多次执行同一查询
    ///
    /// # 参数
    /// - `param_batches` - 参数批次列表，每个批次是一组参数
    ///
    /// # 返回
    /// - 成功时返回每批次的执行结果
    /// - 失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use graphdb::api::embedded::GraphDatabase;
    /// use std::collections::HashMap;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = GraphDatabase::open("my_db")?;
    /// let mut stmt = db.prepare("INSERT VERTEX user(id, name) VALUES $id:($name)")?;
    ///
    /// let batches = vec![
    ///     {
    ///         let mut params = HashMap::new();
    ///         params.insert("id".to_string(), Value::Int(1));
    ///         params.insert("name".to_string(), Value::String("Alice".to_string()));
    ///         params
    ///     },
    ///     {
    ///         let mut params = HashMap::new();
    ///         params.insert("id".to_string(), Value::Int(2));
    ///         params.insert("name".to_string(), Value::String("Bob".to_string()));
    ///         params
    ///     },
    /// ];
    ///
    /// let results = stmt.execute_batch(&batches)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn execute_batch(
        &mut self,
        param_batches: &[HashMap<String, Value>],
    ) -> CoreResult<Vec<QueryResult>> {
        let mut results = Vec::with_capacity(param_batches.len());

        for params in param_batches {
            self.reset();
            self.bind_many(params.clone())?;
            let result = self.execute()?;
            results.push(result);
        }

        Ok(results)
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

    /// 获取参数信息列表
    pub fn parameter_info(&self) -> Vec<ParameterInfo> {
        self.parameter_types
            .iter()
            .map(|(name, data_type)| ParameterInfo {
                name: name.clone(),
                data_type: data_type.clone(),
                required: true,
                default_value: None,
            })
            .collect()
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
        for name in self.parameter_types.keys() {
            if !self.bound_params.contains_key(name) {
                return Err(CoreError::InvalidParameter(format!("参数未绑定: {}", name)));
            }
        }
        Ok(())
    }

    /// 获取执行统计信息
    pub fn stats(&self) -> &ExecutionStats {
        &self.stats
    }

    /// 获取执行历史
    pub fn execution_history(&self) -> &[Duration] {
        &self.execution_history
    }

    /// 记录执行
    fn record_execution(&mut self, duration: Duration) {
        self.stats.record_execution(duration);
        self.execution_history.push(duration);

        if self.execution_history.len() > self.config.max_history_size {
            self.execution_history.remove(0);
        }
    }
}
