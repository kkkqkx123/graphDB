//! E2E 测试共享工具模块
//!
//! 提供 E2E 测试基础设施和辅助函数

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use graphdb::api::service::{GraphService, StatsManager};
use graphdb::api::session::{ClientSession, GraphSessionManager};
use graphdb::config::Config;
use graphdb::core::{Value, DataSet, Row};
use graphdb::storage::redb_storage::{RedbStorage, DefaultStorage};

/// E2E 测试上下文
///
/// 维护完整的测试环境，包括服务、会话和存储
pub struct E2eTestContext {
    service: Arc<GraphService<DefaultStorage>>,
    session_manager: Arc<GraphSessionManager>,
    storage: Arc<DefaultStorage>,
    temp_path: PathBuf,
    current_space: Mutex<Option<String>>,
}

impl E2eTestContext {
    /// 创建新的 E2E 测试上下文
    pub async fn new() -> anyhow::Result<Self> {
        let temp_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("e2e-test-temp");
        
        std::fs::create_dir_all(&temp_dir)?;
        
        let unique_id = format!(
            "e2e_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        );
        let temp_path = temp_dir.join(&unique_id);
        std::fs::create_dir_all(&temp_path)?;
        
        let db_path = temp_path.join("test.db");
        
        let mut config = Config::default();
        config.storage_path = db_path.to_string_lossy().to_string();
        
        let storage = Arc::new(DefaultStorage::new_with_path(db_path)?);
        let stats_manager = Arc::new(StatsManager::new());
        let session_manager = Arc::new(GraphSessionManager::new(
            format!("{}:{}", config.host, config.port),
            config.max_connections,
            3600,
        ));
        
        let service = Arc::new(GraphService::new(config, storage.clone()));
        
        Ok(Self {
            service,
            session_manager,
            storage,
            temp_path,
            current_space: Mutex::new(None),
        })
    }
    
    /// 创建新会话
    pub async fn create_session(&self, username: &str) -> anyhow::Result<ClientSession> {
        self.session_manager
            .create_session(username.to_string(), "127.0.0.1".to_string())
            .map_err(|e| anyhow::anyhow!("创建会话失败: {}", e))
    }
    
    /// 执行查询
    pub async fn execute_query(&self, query: &str) -> anyhow::Result<QueryResult> {
        let session = self.create_session("e2e_test").await?;
        let session_id = session.id();
        
        let start = Instant::now();
        let result = self.service.execute(session_id, query).await;
        let duration = start.elapsed();
        
        match result {
            Ok(data_set) => Ok(QueryResult {
                success: true,
                data: Some(data_set),
                error: None,
                execution_time: duration,
            }),
            Err(e) => Ok(QueryResult {
                success: false,
                data: None,
                error: Some(e.to_string()),
                execution_time: duration,
            }),
        }
    }
    
    /// 执行查询并返回成功结果
    pub async fn execute_query_ok(&self, query: &str) -> anyhow::Result<DataSet> {
        let result = self.execute_query(query).await?;
        if result.success {
            Ok(result.data.expect("成功结果应包含数据"))
        } else {
            Err(anyhow::anyhow!(
                "查询执行失败: {}",
                result.error.unwrap_or_default()
            ))
        }
    }
    
    /// 获取存储实例
    pub fn storage(&self) -> Arc<DefaultStorage> {
        self.storage.clone()
    }
    
    /// 获取服务实例
    pub fn service(&self) -> Arc<GraphService<DefaultStorage>> {
        self.service.clone()
    }
    
    /// 设置当前图空间
    pub async fn use_space(&self, space_name: &str) -> anyhow::Result<()> {
        let query = format!("USE {}", space_name);
        self.execute_query_ok(&query).await?;
        *self.current_space.lock().await = Some(space_name.to_string());
        Ok(())
    }
    
    /// 获取当前图空间
    pub async fn current_space(&self) -> Option<String> {
        self.current_space.lock().await.clone()
    }
}

impl Drop for E2eTestContext {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.temp_path);
    }
}

impl Clone for E2eTestContext {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            session_manager: self.session_manager.clone(),
            storage: self.storage.clone(),
            temp_path: self.temp_path.clone(),
            current_space: Mutex::new(None),
        }
    }
}

/// 查询结果
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub success: bool,
    pub data: Option<DataSet>,
    pub error: Option<String>,
    pub execution_time: Duration,
}

/// 断言工具
pub mod assertions {
    use super::*;
    
    /// 断言查询成功
    pub fn assert_query_success(result: &QueryResult) {
        assert!(
            result.success,
            "查询应该成功，但失败: {:?}",
            result.error
        );
    }
    
    /// 断言查询失败
    pub fn assert_query_failed(result: &QueryResult) {
        assert!(!result.success, "查询应该失败，但成功了");
    }
    
    /// 断言结果集行数
    pub fn assert_row_count(data: &DataSet, expected: usize) {
        assert_eq!(
            data.rows.len(),
            expected,
            "结果集行数不匹配，期望 {}，实际 {}",
            expected,
            data.rows.len()
        );
    }
    
    /// 断言结果集非空
    pub fn assert_not_empty(data: &DataSet) {
        assert!(!data.rows.is_empty(), "结果集不应为空");
    }
    
    /// 断言结果集为空
    pub fn assert_empty(data: &DataSet) {
        assert!(data.rows.is_empty(), "结果集应为空");
    }
    
    /// 断言执行时间在限制内
    pub fn assert_execution_time(result: &QueryResult, max_duration: Duration) {
        assert!(
            result.execution_time <= max_duration,
            "执行时间 {:?} 超过限制 {:?}",
            result.execution_time,
            max_duration
        );
    }
    
    /// 断言行包含特定值
    pub fn assert_row_contains(row: &Row, column: &str, expected: &Value) {
        let col_index = row
            .columns
            .iter()
            .position(|c| c == column)
            .expect(&format!("列 '{}' 不存在", column));
        
        assert_eq!(
            &row.values[col_index], expected,
            "列 '{}' 的值不匹配",
            column
        );
    }
    
    /// 断言结果包含列
    pub fn assert_has_column(data: &DataSet, column: &str) {
        assert!(
            data.columns.iter().any(|c| c.name == column),
            "结果集应包含列 '{}'",
            column
        );
    }
}

/// 性能分析工具
pub struct PerformanceProfiler {
    measurements: Vec<PerformanceMeasurement>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    pub query: String,
    pub duration: Duration,
    pub timestamp: Instant,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            measurements: Vec::new(),
        }
    }
    
    pub fn record(&mut self, query: &str, duration: Duration) {
        self.measurements.push(PerformanceMeasurement {
            query: query.to_string(),
            duration,
            timestamp: Instant::now(),
        });
    }
    
    pub fn average_duration(&self) -> Option<Duration> {
        if self.measurements.is_empty() {
            return None;
        }
        
        let total: Duration = self.measurements.iter().map(|m| m.duration).sum();
        Some(total / self.measurements.len() as u32)
    }
    
    pub fn p99_duration(&self) -> Option<Duration> {
        if self.measurements.is_empty() {
            return None;
        }
        
        let mut durations: Vec<Duration> =
            self.measurements.iter().map(|m| m.duration).collect();
        durations.sort();
        
        let index = (durations.len() as f64 * 0.99) as usize;
        durations.get(index).copied()
    }
    
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== 性能测试报告 ===\n\n");
        
        if let Some(avg) = self.average_duration() {
            report.push_str(&format!("平均执行时间: {:?}\n", avg));
        }
        
        if let Some(p99) = self.p99_duration() {
            report.push_str(&format!("P99 执行时间: {:?}\n", p99));
        }
        
        report.push_str(&format!("总查询次数: {}\n", self.measurements.len()));
        
        report
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试超时包装器
pub async fn with_timeout<F, T>(duration: Duration, f: F) -> anyhow::Result<T>
where
    F: std::future::Future<Output = anyhow::Result<T>>,
{
    tokio::time::timeout(duration, f)
        .await
        .map_err(|_| anyhow::anyhow!("测试超时"))?
}

/// 重试执行
pub async fn retry<F, Fut, T>(mut f: F, max_retries: u32) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let mut last_error = None;
    
    for i in 0..max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if i < max_retries - 1 {
                    tokio::time::sleep(Duration::from_millis(100 * (i + 1) as u64)).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}
