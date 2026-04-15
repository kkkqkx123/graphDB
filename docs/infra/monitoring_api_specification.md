# 性能监控 API 设计规范

## 一、概述

本文档定义性能监控系统的 API 接口规范，用于获取系统性能指标、查询画像和健康状态信息。

### 1.1 设计原则

1. **内部模块隔离**: 内部模块不直接与外部交互，仅提供 API 接口
2. **统一接口风格**: 所有 API 遵循一致的命名和返回格式
3. **线程安全**: 所有 API 支持多线程并发调用
4. **低开销**: API 调用本身不应该显著影响系统性能
5. **可扩展性**: 支持未来添加新的监控指标

### 1.2 API 分类

监控 API 分为三大类：

| 类别 | 用途 | 调用频率 |
|------|------|---------|
| Metrics API | 获取聚合指标 | 高频（每秒多次） |
| Profile API | 获取查询详情 | 中频（调试时使用） |
| Health API | 健康检查 | 高频（健康检查） |

---

## 二、Metrics API

### 2.1 数据结构

#### 2.1.1 主指标结构

```rust
/// 监控指标快照
#[derive(Debug, Clone)]
pub struct MonitoringMetrics {
    /// 查询指标
    pub query_metrics: QueryMetricsSnapshot,
    /// 存储指标
    pub storage_metrics: StorageMetricsSnapshot,
    /// 同步指标
    pub sync_metrics: SyncMetricsSnapshot,
    /// 延迟分布
    pub latency_histogram: LatencySnapshot,
    /// 内存指标
    pub memory_metrics: MemoryMetricsSnapshot,
}
```

#### 2.1.2 查询指标

```rust
/// 查询指标快照
#[derive(Debug, Clone, Default)]
pub struct QueryMetricsSnapshot {
    /// 总查询数
    pub total_queries: u64,
    /// 活跃查询数
    pub active_queries: u64,
    /// 失败查询数
    pub failed_queries: u64,
    /// 平均解析时间（微秒）
    pub avg_parse_time_us: f64,
    /// 平均验证时间（微秒）
    pub avg_validate_time_us: f64,
    /// 平均计划时间（微秒）
    pub avg_plan_time_us: f64,
    /// 平均优化时间（微秒）
    pub avg_optimize_time_us: f64,
    /// 平均执行时间（微秒）
    pub avg_execute_time_us: f64,
    /// 平均总时间（微秒）
    pub avg_total_time_us: f64,
    /// 平均结果行数
    pub avg_result_rows: f64,
    /// 平均计划节点数
    pub avg_plan_node_count: f64,
}
```

#### 2.1.3 存储指标

```rust
/// 存储指标快照
#[derive(Debug, Clone, Default)]
pub struct StorageMetricsSnapshot {
    /// 扫描项数
    pub items_scanned: u64,
    /// 返回项数
    pub items_returned: u64,
    /// 缓存命中数
    pub cache_hits: u64,
    /// 缓存未命中数
    pub cache_misses: u64,
    /// 缓存命中率（0.0-1.0）
    pub cache_hit_rate: f64,
    /// 扫描效率（返回/扫描）
    pub scan_efficiency: f64,
    /// I/O 读取次数
    pub io_reads: u64,
    /// I/O 读取字节数
    pub io_read_bytes: u64,
    /// I/O 写入次数
    pub io_writes: u64,
    /// I/O 写入字节数
    pub io_write_bytes: u64,
    /// I/O 总时间（微秒）
    pub io_time_us: u64,
}
```

#### 2.1.4 同步指标

```rust
/// 同步指标快照
#[derive(Debug, Clone, Default)]
pub struct SyncMetricsSnapshot {
    /// 已提交事务数
    pub transactions_committed: u64,
    /// 已回滚事务数
    pub transactions_rolled_back: u64,
    /// 活跃事务数
    pub active_transactions: u64,
    /// 索引操作总数
    pub index_operations_total: u64,
    /// 索引插入操作数
    pub index_operations_insert: u64,
    /// 索引更新操作数
    pub index_operations_update: u64,
    /// 索引删除操作数
    pub index_operations_delete: u64,
    /// 重试尝试次数
    pub retry_attempts_total: u64,
    /// 重试成功次数
    pub retry_successes: u64,
    /// 重试失败次数
    pub retry_failures: u64,
    /// 重试成功率（0.0-1.0）
    pub retry_success_rate: f64,
    /// 死信队列大小
    pub dead_letter_queue_size: usize,
    /// 总处理时间（毫秒）
    pub total_processing_time_ms: u64,
    /// 平均处理时间（毫秒）
    pub avg_processing_time_ms: f64,
}
```

#### 2.1.5 延迟分布

```rust
/// 延迟分布快照
#[derive(Debug, Clone, Default)]
pub struct LatencySnapshot {
    /// P50 延迟（微秒）
    pub p50_us: u64,
    /// P95 延迟（微秒）
    pub p95_us: u64,
    /// P99 延迟（微秒）
    pub p99_us: u64,
    /// 平均延迟（微秒）
    pub mean_us: f64,
    /// 最大延迟（微秒）
    pub max_us: u64,
    /// 最小延迟（微秒）
    pub min_us: u64,
    /// 样本数量
    pub count: u64,
    /// 标准差
    pub std_dev_us: f64,
}
```

#### 2.1.6 内存指标

```rust
/// 内存指标快照
#[derive(Debug, Clone, Default)]
pub struct MemoryMetricsSnapshot {
    /// 当前使用内存（字节）
    pub current_bytes: usize,
    /// 峰值内存（字节）
    pub peak_bytes: usize,
    /// 总分配内存（字节）
    pub allocated_bytes: usize,
    /// 分配次数
    pub allocation_count: usize,
    /// 释放次数
    pub deallocation_count: usize,
    /// 当前使用（MB）
    pub current_mb: f64,
    /// 峰值使用（MB）
    pub peak_mb: f64,
}
```

### 2.2 API 接口

#### 2.2.1 获取所有指标

```rust
impl MonitoringApi {
    /// 获取所有监控指标
    /// 
    /// # 返回
    /// 返回完整的监控指标快照
    /// 
    /// # 性能特征
    /// - 时间复杂度：O(1)
    /// - 空间复杂度：O(1)
    /// - 线程安全：是
    /// 
    /// # 示例
    /// ```
    /// let metrics = monitoring_api.get_metrics();
    /// println!("Total queries: {}", metrics.query_metrics.total_queries);
    /// println!("P95 latency: {} us", metrics.latency_histogram.p95_us);
    /// ```
    pub fn get_metrics(&self) -> MonitoringMetrics;
}
```

#### 2.2.2 获取查询指标

```rust
impl MonitoringApi {
    /// 获取查询指标
    /// 
    /// # 返回
    /// 返回查询相关的指标快照
    /// 
    /// # 示例
    /// ```
    /// let query_metrics = monitoring_api.get_query_metrics();
    /// println!("Active queries: {}", query_metrics.active_queries);
    /// ```
    pub fn get_query_metrics(&self) -> QueryMetricsSnapshot;
}
```

#### 2.2.3 获取存储指标

```rust
impl MonitoringApi {
    /// 获取存储指标
    /// 
    /// # 返回
    /// 返回存储引擎的指标快照
    pub fn get_storage_metrics(&self) -> StorageMetricsSnapshot;
}
```

#### 2.2.4 获取同步指标

```rust
impl MonitoringApi {
    /// 获取同步指标
    /// 
    /// # 返回
    /// 返回同步模块的指标快照
    pub fn get_sync_metrics(&self) -> SyncMetricsSnapshot;
}
```

#### 2.2.5 获取延迟分布

```rust
impl MonitoringApi {
    /// 获取延迟分布
    /// 
    /// # 返回
    /// 返回查询延迟的分布统计
    /// 
    /// # 示例
    /// ```
    /// let latency = monitoring_api.get_latency_snapshot();
    /// println!("P50: {} us", latency.p50_us);
    /// println!("P95: {} us", latency.p95_us);
    /// println!("P99: {} us", latency.p99_us);
    /// ```
    pub fn get_latency_snapshot(&self) -> LatencySnapshot;
}
```

#### 2.2.6 获取内存指标

```rust
impl MonitoringApi {
    /// 获取内存指标
    /// 
    /// # 返回
    /// 返回内存使用情况的快照
    pub fn get_memory_metrics(&self) -> MemoryMetricsSnapshot;
}
```

#### 2.2.7 重置指标

```rust
impl MonitoringApi {
    /// 重置所有指标
    /// 
    /// # 注意
    /// 此操作会清空所有累计指标，谨慎使用
    pub fn reset_metrics(&self);
}
```

---

## 三、Profile API

### 3.1 数据结构

#### 3.1.1 查询画像

使用现有的 `QueryProfile` 结构：

```rust
#[derive(Debug, Clone)]
pub struct QueryProfile {
    pub trace_id: String,
    pub session_id: i64,
    pub query_text: String,
    pub start_time: Instant,
    pub total_duration_ms: u64,
    pub stages: StageMetrics,
    pub executor_stats: Vec<ExecutorStat>,
    pub result_count: usize,
    pub status: QueryStatus,
    pub error_message: Option<String>,
    pub error_info: Option<ErrorInfo>,
}
```

#### 3.1.2 查询画像过滤条件

```rust
/// 查询画像过滤条件
#[derive(Debug, Clone, Default)]
pub struct ProfileFilter {
    /// 最小执行时间（毫秒）
    pub min_duration_ms: Option<u64>,
    /// 最大执行时间（毫秒）
    pub max_duration_ms: Option<u64>,
    /// 查询状态
    pub status: Option<QueryStatus>,
    /// 查询文本包含
    pub query_text_contains: Option<String>,
    /// Trace ID
    pub trace_id: Option<String>,
    /// Session ID
    pub session_id: Option<i64>,
    /// 开始时间
    pub start_time_from: Option<Instant>,
    /// 结束时间
    pub start_time_to: Option<Instant>,
}
```

#### 3.1.3 查询画像排序选项

```rust
/// 排序字段
#[derive(Debug, Clone, Copy)]
pub enum ProfileSortField {
    Duration,
    StartTime,
    ResultCount,
}

/// 排序方向
#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// 排序选项
#[derive(Debug, Clone, Default)]
pub struct ProfileSort {
    pub field: ProfileSortField,
    pub order: SortOrder,
}
```

### 3.2 API 接口

#### 3.2.1 获取最近的查询画像

```rust
impl ProfileApi {
    /// 获取最近的查询画像
    /// 
    /// # 参数
    /// - `limit`: 返回的最大数量
    /// 
    /// # 返回
    /// 返回最近的查询画像列表，按开始时间降序排列
    /// 
    /// # 示例
    /// ```
    /// let profiles = profile_api.get_recent_profiles(10);
    /// for profile in profiles {
    ///     println!("[{}] {} - {}ms", 
    ///         profile.trace_id, 
    ///         profile.query_text,
    ///         profile.total_duration_ms
    ///     );
    /// }
    /// ```
    pub fn get_recent_profiles(&self, limit: usize) -> Vec<QueryProfile>;
}
```

#### 3.2.2 根据条件查询画像

```rust
impl ProfileApi {
    /// 根据条件查询画像
    /// 
    /// # 参数
    /// - `filter`: 过滤条件
    /// - `sort`: 排序选项
    /// - `limit`: 返回的最大数量
    /// 
    /// # 返回
    /// 返回符合条件的查询画像列表
    /// 
    /// # 示例
    /// ```
    /// let filter = ProfileFilter {
    ///     min_duration_ms: Some(1000),
    ///     status: Some(QueryStatus::Failed),
    ///     ..Default::default()
    /// };
    /// let sort = ProfileSort {
    ///     field: ProfileSortField::Duration,
    ///     order: SortOrder::Desc,
    /// };
    /// let profiles = profile_api.query_profiles(filter, sort, 20);
    /// ```
    pub fn query_profiles(
        &self,
        filter: ProfileFilter,
        sort: ProfileSort,
        limit: usize,
    ) -> Vec<QueryProfile>;
}
```

#### 3.2.3 根据 trace_id 获取画像

```rust
impl ProfileApi {
    /// 根据 trace_id 获取查询画像
    /// 
    /// # 参数
    /// - `trace_id`: 查询的 trace ID
    /// 
    /// # 返回
    /// 如果找到则返回对应的查询画像
    /// 
    /// # 示例
    /// ```
    /// if let Some(profile) = profile_api.get_profile_by_trace_id("abc-123") {
    ///     println!("Query: {}", profile.query_text);
    ///     println!("Duration: {}ms", profile.total_duration_ms);
    /// }
    /// ```
    pub fn get_profile_by_trace_id(&self, trace_id: &str) -> Option<QueryProfile>;
}
```

#### 3.2.4 获取慢查询

```rust
impl ProfileApi {
    /// 获取慢查询画像
    /// 
    /// # 参数
    /// - `threshold_ms`: 慢查询阈值（毫秒）
    /// - `limit`: 返回的最大数量
    /// 
    /// # 返回
    /// 返回执行时间超过阈值的查询画像
    /// 
    /// # 示例
    /// ```
    /// let slow_queries = profile_api.get_slow_queries(1000, 10);
    /// for query in slow_queries {
    ///     println!("Slow query: {} ({}ms)", 
    ///         query.query_text,
    ///         query.total_duration_ms
    ///     );
    /// }
    /// ```
    pub fn get_slow_queries(&self, threshold_ms: u64, limit: usize) -> Vec<QueryProfile>;
}
```

#### 3.2.5 获取失败的查询

```rust
impl ProfileApi {
    /// 获取失败的查询画像
    /// 
    /// # 参数
    /// - `limit`: 返回的最大数量
    /// 
    /// # 返回
    /// 返回执行失败的查询画像
    pub fn get_failed_queries(&self, limit: usize) -> Vec<QueryProfile>;
}
```

#### 3.2.6 清除画像缓存

```rust
impl ProfileApi {
    /// 清除所有缓存的查询画像
    /// 
    /// # 注意
    /// 此操作会清空所有历史查询画像
    pub fn clear_profiles(&self);
}
```

---

## 四、Health API

### 4.1 数据结构

#### 4.1.1 健康状态

```rust
/// 健康状态
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// 健康状态
    pub status: HealthState,
    /// 状态描述
    pub message: String,
    /// 详细指标
    pub metrics: HealthMetrics,
    /// 告警列表
    pub alerts: Vec<HealthAlert>,
}
```

#### 4.1.2 健康状态枚举

```rust
/// 健康状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    /// 健康
    Healthy,
    /// 降级（部分功能受影响）
    Degraded,
    /// 不健康（严重问题）
    Unhealthy,
}
```

#### 4.1.3 健康指标

```rust
/// 健康指标
#[derive(Debug, Clone, Default)]
pub struct HealthMetrics {
    /// 活跃查询数
    pub active_queries: u64,
    /// 错误率（0.0-1.0）
    pub error_rate: f64,
    /// P99 延迟（毫秒）
    pub p99_latency_ms: f64,
    /// 内存使用（MB）
    pub memory_used_mb: f64,
    /// 存储缓存命中率（0.0-1.0）
    pub cache_hit_rate: f64,
    /// 活跃事务数
    pub active_transactions: u64,
    /// 死信队列大小
    pub dead_letter_queue_size: usize,
}
```

#### 4.1.4 健康告警

```rust
/// 健康告警
#[derive(Debug, Clone)]
pub struct HealthAlert {
    /// 告警级别
    pub level: AlertLevel,
    /// 告警类型
    pub alert_type: AlertType,
    /// 告警消息
    pub message: String,
    /// 当前值
    pub current_value: f64,
    /// 阈值
    pub threshold_value: f64,
}
```

#### 4.1.5 告警级别

```rust
/// 告警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertLevel {
    /// 警告
    Warning,
    /// 严重
    Critical,
}
```

#### 4.1.6 告警类型

```rust
/// 告警类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertType {
    /// 高延迟
    HighLatency,
    /// 高错误率
    HighErrorRate,
    /// 高活跃查询数
    HighActiveQueries,
    /// 高内存使用
    HighMemoryUsage,
    /// 低缓存命中率
    LowCacheHitRate,
    /// 死信队列堆积
    DeadLetterQueueBacklog,
}
```

#### 4.1.7 健康阈值配置

```rust
/// 健康阈值配置
#[derive(Debug, Clone)]
pub struct HealthThresholds {
    /// 最大活跃查询数
    pub max_active_queries: u64,
    /// 最大错误率（0.0-1.0）
    pub max_error_rate: f64,
    /// 最大 P99 延迟（毫秒）
    pub max_p99_latency_ms: f64,
    /// 最大内存使用（MB）
    pub max_memory_mb: f64,
    /// 最小缓存命中率（0.0-1.0）
    pub min_cache_hit_rate: f64,
    /// 最大死信队列大小
    pub max_dead_letter_queue_size: usize,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            max_active_queries: 100,
            max_error_rate: 0.05,
            max_p99_latency_ms: 500.0,
            max_memory_mb: 1024.0,
            min_cache_hit_rate: 0.8,
            max_dead_letter_queue_size: 100,
        }
    }
}
```

### 4.2 API 接口

#### 4.2.1 检查健康状态

```rust
impl HealthApi {
    /// 检查系统健康状态
    /// 
    /// # 返回
    /// 返回完整的健康状态信息
    /// 
    /// # 示例
    /// ```
    /// let health = health_api.check_health();
    /// println!("Status: {:?}", health.status);
    /// println!("Message: {}", health.message);
    /// 
    /// if health.status != HealthState::Healthy {
    ///     for alert in &health.alerts {
    ///         eprintln!("Alert: {:?} - {}", alert.level, alert.message);
    ///     }
    /// }
    /// ```
    pub fn check_health(&self) -> HealthStatus;
}
```

#### 4.2.2 获取健康指标

```rust
impl HealthApi {
    /// 获取健康指标
    /// 
    /// # 返回
    /// 返回健康相关的指标
    pub fn get_health_metrics(&self) -> HealthMetrics;
}
```

#### 4.2.3 更新阈值配置

```rust
impl HealthApi {
    /// 更新健康阈值配置
    /// 
    /// # 参数
    /// - `thresholds`: 新的阈值配置
    /// 
    /// # 示例
    /// ```
    /// let thresholds = HealthThresholds {
    ///     max_p99_latency_ms: 1000.0,
    ///     max_error_rate: 0.1,
    ///     ..Default::default()
    /// };
    /// health_api.update_thresholds(thresholds);
    /// ```
    pub fn update_thresholds(&self, thresholds: HealthThresholds);
}
```

#### 4.2.4 获取当前阈值

```rust
impl HealthApi {
    /// 获取当前阈值配置
    /// 
    /// # 返回
    /// 返回当前的阈值配置
    pub fn get_thresholds(&self) -> HealthThresholds;
}
```

---

## 五、配置 API

### 5.1 监控配置

```rust
/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// 监控级别
    pub level: MonitoringLevel,
    /// 采样率（0.0-1.0）
    pub sampling_rate: f64,
    /// 慢查询阈值（毫秒）
    pub slow_query_threshold_ms: u64,
    /// 是否启用直方图统计
    pub enable_histogram: bool,
    /// 是否启用 I/O 统计
    pub enable_io_stats: bool,
    /// 是否启用内存统计
    pub enable_memory_stats: bool,
    /// 画像缓存大小
    pub profile_cache_size: usize,
}
```

### 5.2 API 接口

#### 5.2.1 更新监控配置

```rust
impl MonitoringApi {
    /// 更新监控配置
    /// 
    /// # 参数
    /// - `config`: 新的监控配置
    /// 
    /// # 示例
    /// ```
    /// let config = MonitoringConfig {
    ///     level: MonitoringLevel::Minimal,
    ///     sampling_rate: 0.1,
    ///     slow_query_threshold_ms: 2000,
    ///     ..Default::default()
    /// };
    /// monitoring_api.update_config(config);
    /// ```
    pub fn update_config(&self, config: MonitoringConfig);
}
```

#### 5.2.2 获取当前配置

```rust
impl MonitoringApi {
    /// 获取当前监控配置
    /// 
    /// # 返回
    /// 返回当前的监控配置
    pub fn get_config(&self) -> MonitoringConfig;
}
```

---

## 六、使用示例

### 6.1 完整使用流程

```rust
use graphdb::infra::monitoring::api::{MonitoringApi, ProfileApi, HealthApi};
use graphdb::core::stats::StatsManager;
use std::sync::Arc;

fn main() {
    // 初始化 StatsManager
    let stats_manager = Arc::new(StatsManager::new());
    
    // 创建监控 API
    let monitoring_api = Arc::new(MonitoringApi::new(stats_manager.clone()));
    let profile_api = ProfileApi::new(stats_manager.clone());
    let health_api = HealthApi::new(monitoring_api.clone());
    
    // 1. 获取所有监控指标
    let metrics = monitoring_api.get_metrics();
    println!("=== 监控指标 ===");
    println!("总查询数：{}", metrics.query_metrics.total_queries);
    println!("活跃查询数：{}", metrics.query_metrics.active_queries);
    println!("P95 延迟：{} us", metrics.latency_histogram.p95_us);
    println!("P99 延迟：{} us", metrics.latency_histogram.p99_us);
    
    // 2. 获取慢查询
    println!("\n=== 慢查询 ===");
    let slow_queries = profile_api.get_slow_queries(1000, 5);
    for query in slow_queries {
        println!(
            "[{}] {} - {}ms",
            query.trace_id,
            query.query_text,
            query.total_duration_ms
        );
    }
    
    // 3. 健康检查
    println!("\n=== 健康状态 ===");
    let health = health_api.check_health();
    println!("状态：{:?}", health.status);
    println!("描述：{}", health.message);
    
    if !health.alerts.is_empty() {
        println!("告警：");
        for alert in &health.alerts {
            println!("  [{:?}] {} - 当前值：{}, 阈值：{}", 
                alert.level,
                alert.message,
                alert.current_value,
                alert.threshold_value
            );
        }
    }
    
    // 4. 调整监控配置
    println!("\n=== 调整监控配置 ===");
    let config = MonitoringConfig {
        level: MonitoringLevel::Standard,
        sampling_rate: 0.5,  // 50% 采样
        slow_query_threshold_ms: 2000,
        enable_histogram: true,
        enable_io_stats: false,
        enable_memory_stats: false,
        profile_cache_size: 100,
    };
    monitoring_api.update_config(config);
    println!("配置已更新");
}
```

### 6.2 性能监控仪表板

```rust
/// 简单的性能监控仪表板
struct PerformanceDashboard {
    monitoring_api: Arc<MonitoringApi>,
    health_api: Arc<HealthApi>,
}

impl PerformanceDashboard {
    fn new(monitoring_api: Arc<MonitoringApi>, health_api: Arc<HealthApi>) -> Self {
        Self {
            monitoring_api,
            health_api,
        }
    }
    
    /// 打印仪表板信息
    fn print_dashboard(&self) {
        let metrics = self.monitoring_api.get_metrics();
        let health = self.health_api.check_health();
        
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║              性能监控仪表板                              ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        
        // 健康状态
        let status_icon = match health.status {
            HealthState::Healthy => "✓",
            HealthState::Degraded => "⚠",
            HealthState::Unhealthy => "✗",
        };
        println!("║ 健康状态：{} {:<50} ║", status_icon, format!("{:?}", health.status));
        
        // 查询指标
        println!("╠──────────────────────────────────────────────────────────╣");
        println!("║ 查询统计：                                              ║");
        println!("║   总查询数：{:<45} ║", metrics.query_metrics.total_queries);
        println!("║   活跃查询：{:<45} ║", metrics.query_metrics.active_queries);
        println!("║   失败查询：{:<45} ║", metrics.query_metrics.failed_queries);
        
        // 延迟统计
        println!("╠──────────────────────────────────────────────────────────╣");
        println!("║ 延迟统计：                                              ║");
        println!("║   P50: {:>12} us                                      ║", metrics.latency_histogram.p50_us);
        println!("║   P95: {:>12} us                                      ║", metrics.latency_histogram.p95_us);
        println!("║   P99: {:>12} us                                      ║", metrics.latency_histogram.p99_us);
        println!("║   平均：{:>12.2} us                                    ║", metrics.latency_histogram.mean_us);
        
        // 存储指标
        println!("╠──────────────────────────────────────────────────────────╣");
        println!("║ 存储统计：                                              ║");
        println!("║   缓存命中率：{:.2}%                                    ║", metrics.storage_metrics.cache_hit_rate * 100.0);
        println!("║   扫描效率：{:.2}%                                      ║", metrics.storage_metrics.scan_efficiency * 100.0);
        
        println!("╚══════════════════════════════════════════════════════════╝");
    }
}
```

---

## 七、性能考虑

### 7.1 API 调用开销

| API | 预估开销 | 优化建议 |
|-----|---------|---------|
| `get_metrics()` | < 10us | 返回快照，无锁读取 |
| `get_query_metrics()` | < 5us | 原子操作读取 |
| `get_recent_profiles()` | O(n) | 限制最大返回数量 |
| `get_slow_queries()` | O(n) | 使用索引加速 |
| `check_health()` | < 50us | 并行收集指标 |

### 7.2 最佳实践

1. **避免频繁调用 Profile API**
   - Profile API 用于调试，不建议在生产环境高频调用
   - 建议调用频率：< 1 次/秒

2. **使用 Metrics API 进行监控**
   - Metrics API 设计为轻量级，支持高频调用
   - 建议调用频率：1-10 次/秒

3. **合理设置采样率**
   - 高负载场景降低采样率（0.1-0.5）
   - 低负载场景使用全量采样（1.0）

4. **定期清理画像缓存**
   - 使用 `clear_profiles()` 定期清理
   - 或设置合理的 `profile_cache_size`

---

## 八、错误处理

### 8.1 错误类型

```rust
/// 监控 API 错误
#[derive(Debug, Clone)]
pub enum MonitoringApiError {
    /// 配置错误
    ConfigError(String),
    /// 数据不可用
    DataNotAvailable(String),
    /// 内部错误
    InternalError(String),
}

impl std::fmt::Display for MonitoringApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitoringApiError::ConfigError(msg) => write!(f, "配置错误：{}", msg),
            MonitoringApiError::DataNotAvailable(msg) => write!(f, "数据不可用：{}", msg),
            MonitoringApiError::InternalError(msg) => write!(f, "内部错误：{}", msg),
        }
    }
}

impl std::error::Error for MonitoringApiError {}
```

### 8.2 错误处理示例

```rust
match profile_api.get_profile_by_trace_id(trace_id) {
    Some(profile) => Ok(profile),
    None => Err(MonitoringApiError::DataNotAvailable(
        format!("未找到 trace_id 为 {} 的查询画像", trace_id)
    )),
}
```

---

## 九、总结

本 API 设计规范提供了：

1. **Metrics API** - 用于获取系统性能指标，支持高频调用
2. **Profile API** - 用于获取查询详情，支持调试和分析
3. **Health API** - 用于健康检查，支持告警和监控
4. **配置 API** - 用于动态调整监控行为

所有 API 遵循以下原则：
- 线程安全，支持并发调用
- 低开销，不影响主流程性能
- 易于使用，提供清晰的文档和示例
- 可扩展，支持未来添加新指标
