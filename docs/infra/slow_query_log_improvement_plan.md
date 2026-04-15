# GraphDB 慢查询日志改进方案

## 一、现状分析

### 1.1 当前实现

**架构**：
- 使用 `StatsManager::write_slow_query_log()` 方法
- 通过 `log::warn!` 宏写入主日志文件（`logs/graphdb.log`）
- 没有独立的慢查询日志文件
- 同步写入，阻塞查询执行

**配置**：
```rust
pub struct MonitoringConfig {
    pub enabled: bool,
    pub memory_cache_size: usize,
    pub slow_query_threshold_ms: u64,  // 默认 1000ms
}
```

**日志格式**：
```
慢查询 [trace_id=xxx] [session_id=xxx] [duration=xxxms] [status=xxx]
查询：SELECT ...
阶段统计：parse=xxxms validate=xxxms plan=xxxms optimize=xxxms execute=xxxms
结果数：xxx 执行器数：xxx 执行器总时间：xxxms
执行器详情：xxx
```

### 1.2 主要问题

| 问题 | 严重程度 | 影响 |
|------|----------|------|
| ❌ 没有独立日志文件 | 高 | 难以分析和定位慢查询 |
| ❌ 没有异步写入 | 高 | 阻塞查询，影响性能 |
| ❌ 没有日志轮转 | 高 | 日志文件可能无限增长 |
| ❌ 缺少 I/O 统计 | 中 | 无法分析存储性能瓶颈 |
| ❌ 格式不够结构化 | 中 | 不利于程序分析 |
| ❌ 缺少执行计划记录 | 中 | 难以优化查询 |

---

## 二、改进目标

### 2.1 核心目标（P0）

1. **独立日志文件**：`logs/slow_query.log`
2. **异步写入**：使用 channel + 后台线程
3. **日志轮转**：自动管理文件大小和数量
4. **可配置格式**：支持标准格式和详细格式

### 2.2 增强目标（P1）

5. **执行计划记录**：类似 PostgreSQL 的 auto_explain
6. **I/O 统计**：记录存储层访问指标
7. **JSON 格式支持**：便于程序分析

### 2.3 高级目标（P2）

8. **聚合统计组件**：类似 pg_stat_statements
9. **采样机制**：减少高负载下的开销
10. **分析工具**：提供日志分析脚本

---

## 三、详细设计方案

### 3.1 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                     Query Execution                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    StatsManager                             │
│  - 判断是否为慢查询                                          │
│  - 发送到慢查询日志器                                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼ (异步 channel)
┌─────────────────────────────────────────────────────────────┐
│                  SlowQueryLogger                            │
│  - 格式化日志条目                                            │
│  - 后台线程异步写入                                          │
│  - 管理日志轮转                                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              慢查询日志文件 (独立)                           │
│  logs/slow_query.log                                         │
│  logs/slow_query.log.1                                       │
│  logs/slow_query.log.2                                       │
│  ...                                                         │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 配置设计

```rust
/// 慢查询日志配置
#[derive(Debug, Clone)]
pub struct SlowQueryConfig {
    /// 是否启用
    pub enabled: bool,
    /// 慢查询阈值（毫秒）
    pub threshold_ms: u64,
    /// 日志文件路径
    pub log_file_path: String,
    /// 单个文件最大大小（MB）
    pub max_file_size_mb: u64,
    /// 最大保留文件数
    pub max_files: u32,
    /// 是否使用详细格式
    pub verbose_format: bool,
    /// 异步写入缓冲区大小
    pub buffer_size: usize,
    /// 是否使用 JSON 格式
    pub json_format: bool,
}

impl Default for SlowQueryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold_ms: 1000,
            log_file_path: "logs/slow_query.log".to_string(),
            max_file_size_mb: 100,
            max_files: 5,
            verbose_format: false,
            buffer_size: 100,
            json_format: false,
        }
    }
}
```

**配置文件示例（config.toml）**：
```toml
[monitoring]
enabled = true
memory_cache_size = 1000
slow_query_threshold_ms = 1000

[monitoring.slow_query_log]
enabled = true
log_file_path = "logs/slow_query.log"
max_file_size_mb = 100
max_files = 5
verbose_format = false
buffer_size = 100
json_format = false
```

### 3.3 日志格式设计

#### 3.3.1 标准格式（默认）

```
[时间戳] [SLOW_QUERY] [trace_id=xxx] [session_id=xxx] [duration=xxxms] [status=xxx] 查询文本
```

**示例**：
```
[2026-04-15 10:23:45.123] [SLOW_QUERY] [trace_id=550e8400-e29b-41d4-a716-446655440000] 
[session_id=12345] [duration=1523ms] [status=success] MATCH (n:User) WHERE n.age > 18 RETURN n
```

#### 3.3.2 详细格式

```
[时间戳] [SLOW_QUERY]
  trace_id: xxx
  session_id: xxx
  query_text: xxx
  duration: xxxms
  status: success/failed
  error_type: xxx (可选)
  error_phase: xxx (可选)
  stages:
    parse: xxxms
    validate: xxxms
    plan: xxxms
    optimize: xxxms
    execute: xxxms
  result_count: xxx
  executor_stats:
    - executor: xxx, duration: xxxms, rows: xxx
    - executor: xxx, duration: xxxms, rows: xxx
```

#### 3.3.3 JSON 格式

```json
{
  "timestamp": "2026-04-15T10:23:45.123+08:00",
  "trace_id": "550e8400-e29b-41d4-a716-446655440000",
  "session_id": 12345,
  "query_text": "MATCH (n:User) WHERE n.age > 18 RETURN n",
  "duration_ms": 1523,
  "status": "success",
  "stages": {
    "parse_ms": 12,
    "validate_ms": 5,
    "plan_ms": 45,
    "optimize_ms": 23,
    "execute_ms": 1438
  },
  "result_count": 15234,
  "executor_stats": [
    {
      "executor_type": "ScanVerticesExecutor",
      "duration_ms": 823,
      "rows_processed": 50000
    },
    {
      "executor_type": "FilterExecutor",
      "duration_ms": 615,
      "rows_processed": 15234
    }
  ]
}
```

### 3.4 核心组件实现

#### 3.4.1 SlowQueryLogger

```rust
use std::sync::mpsc;
use std::thread;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::Mutex;
use chrono::Local;
use serde::Serialize;

/// 慢查询日志器
pub struct SlowQueryLogger {
    config: SlowQueryConfig,
    tx: mpsc::Sender<String>,
    writer_handle: Option<thread::JoinHandle<()>>,
    current_file_size: AtomicU64,
    current_file_path: Mutex<PathBuf>,
}

impl SlowQueryLogger {
    /// 创建新的慢查询日志器
    pub fn new(config: SlowQueryConfig) -> Result<Self, std::io::Error> {
        // 创建日志目录
        if let Some(parent) = Path::new(&config.log_file_path).parent() {
            fs::create_dir_all(parent)?;
        }

        // 创建 channel
        let (tx, rx) = mpsc::channel::<String>();

        // 初始化日志文件
        let initial_path = PathBuf::from(&config.log_file_path);
        let file_size = if initial_path.exists() {
            fs::metadata(&initial_path)?.len()
        } else {
            0
        };

        // 生成后台写入线程
        let writer_handle = Self::spawn_writer_thread(
            rx,
            config.clone(),
            AtomicU64::new(file_size),
            Mutex::new(initial_path.clone()),
        );

        Ok(Self {
            config,
            tx,
            writer_handle: Some(writer_handle),
            current_file_size: AtomicU64::new(file_size),
            current_file_path: Mutex::new(initial_path),
        })
    }

    /// 记录慢查询
    pub fn log(&self, profile: &QueryProfile) {
        let log_entry = if self.config.json_format {
            self.format_json_log(profile)
        } else if self.config.verbose_format {
            self.format_verbose_log(profile)
        } else {
            self.format_standard_log(profile)
        };

        // 异步发送（非阻塞）
        let _ = self.tx.send(log_entry);
    }

    /// 格式化标准日志
    fn format_standard_log(&self, profile: &QueryProfile) -> String {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let status_str = match profile.status {
            QueryStatus::Success => "success",
            QueryStatus::Failed => "failed",
        };

        format!(
            "[{}] [SLOW_QUERY] [trace_id={}] [session_id={}] [duration={}ms] [status={}] {}\n",
            timestamp,
            profile.trace_id,
            profile.session_id,
            micros_to_millis(profile.total_duration_us),
            status_str,
            profile.query_text
        )
    }

    /// 格式化详细日志
    fn format_verbose_log(&self, profile: &QueryProfile) -> String {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let mut log = String::new();

        log.push_str(&format!("[{}] [SLOW_QUERY]\n", timestamp));
        log.push_str(&format!("  trace_id: {}\n", profile.trace_id));
        log.push_str(&format!("  session_id: {}\n", profile.session_id));
        log.push_str(&format!("  query_text: {}\n", profile.query_text));
        log.push_str(&format!("  duration: {}ms\n", micros_to_millis(profile.total_duration_us)));
        
        let status_str = match profile.status {
            QueryStatus::Success => "success",
            QueryStatus::Failed => "failed",
        };
        log.push_str(&format!("  status: {}\n", status_str));

        if let Some(ref error_info) = profile.error_info {
            log.push_str(&format!("  error_type: {}\n", error_info.error_type));
            log.push_str(&format!("  error_phase: {}\n", error_info.error_phase));
            log.push_str(&format!("  error_message: {}\n", error_info.error_message));
        }

        // 阶段统计
        log.push_str("  stages:\n");
        log.push_str(&format!("    parse: {}ms\n", profile.stages.parse_ms()));
        log.push_str(&format!("    validate: {}ms\n", profile.stages.validate_ms()));
        log.push_str(&format!("    plan: {}ms\n", profile.stages.plan_ms()));
        log.push_str(&format!("    optimize: {}ms\n", profile.stages.optimize_ms()));
        log.push_str(&format!("    execute: {}ms\n", profile.stages.execute_ms()));

        log.push_str(&format!("  result_count: {}\n", profile.result_count));

        // 执行器统计
        if !profile.executor_stats.is_empty() {
            log.push_str("  executor_stats:\n");
            for stat in &profile.executor_stats {
                log.push_str(&format!(
                    "    - executor: {}, duration: {}ms, rows: {}\n",
                    stat.executor_type,
                    stat.duration_ms(),
                    stat.rows_processed()
                ));
            }
        }

        log.push('\n');
        log
    }

    /// 格式化 JSON 日志
    fn format_json_log(&self, profile: &QueryProfile) -> String {
        #[derive(Serialize)]
        struct SlowQueryLog {
            timestamp: String,
            trace_id: String,
            session_id: i64,
            query_text: String,
            duration_ms: f64,
            status: String,
            stages: StageStats,
            result_count: usize,
            executor_stats: Vec<ExecutorStatOutput>,
        }

        #[derive(Serialize)]
        struct StageStats {
            parse_ms: f64,
            validate_ms: f64,
            plan_ms: f64,
            optimize_ms: f64,
            execute_ms: f64,
        }

        #[derive(Serialize)]
        struct ExecutorStatOutput {
            executor_type: String,
            duration_ms: f64,
            rows_processed: usize,
        }

        let log_entry = SlowQueryLog {
            timestamp: Local::now().to_rfc3339(),
            trace_id: profile.trace_id.clone(),
            session_id: profile.session_id,
            query_text: profile.query_text.clone(),
            duration_ms: micros_to_millis(profile.total_duration_us),
            status: match profile.status {
                QueryStatus::Success => "success",
                QueryStatus::Failed => "failed",
            }.to_string(),
            stages: StageStats {
                parse_ms: profile.stages.parse_ms(),
                validate_ms: profile.stages.validate_ms(),
                plan_ms: profile.stages.plan_ms(),
                optimize_ms: profile.stages.optimize_ms(),
                execute_ms: profile.stages.execute_ms(),
            },
            result_count: profile.result_count,
            executor_stats: profile.executor_stats.iter().map(|stat| {
                ExecutorStatOutput {
                    executor_type: stat.executor_type.clone(),
                    duration_ms: stat.duration_ms(),
                    rows_processed: stat.rows_processed(),
                }
            }).collect(),
        };

        serde_json::to_string(&log_entry).unwrap_or_default() + "\n"
    }

    /// 生成后台写入线程
    fn spawn_writer_thread(
        rx: mpsc::Receiver<String>,
        config: SlowQueryConfig,
        file_size: AtomicU64,
        file_path: Mutex<PathBuf>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut writer = BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&config.log_file_path)
                    .expect("Failed to open slow query log file")
            );

            loop {
                match rx.recv() {
                    Ok(log_entry) => {
                        let bytes = log_entry.as_bytes();
                        let current_size = file_size.load(Ordering::Relaxed);

                        // 检查是否需要轮转
                        if current_size + bytes.len() as u64 > config.max_file_size_mb * 1024 * 1024 {
                            // 执行日志轮转
                            if let Err(e) = Self::rotate_logs(&config) {
                                eprintln!("Failed to rotate slow query log: {}", e);
                            }
                            
                            // 重新打开新文件
                            writer = BufWriter::new(
                                OpenOptions::new()
                                    .create(true)
                                    .write(true)
                                    .truncate(true)
                                    .open(&config.log_file_path)
                                    .expect("Failed to open new slow query log file")
                            );
                            file_size.store(0, Ordering::Relaxed);
                        }

                        // 写入日志
                        if let Err(e) = writer.write_all(bytes) {
                            eprintln!("Failed to write slow query log: {}", e);
                        }
                        
                        // 定期刷新
                        if file_size.load(Ordering::Relaxed) % 1024 == 0 {
                            let _ = writer.flush();
                        }

                        file_size.fetch_add(bytes.len() as u64, Ordering::Relaxed);
                    }
                    Err(_) => {
                        // channel 关闭，退出线程
                        break;
                    }
                }
            }

            // 确保所有数据写入
            let _ = writer.flush();
        })
    }

    /// 日志轮转
    fn rotate_logs(config: &SlowQueryConfig) -> std::io::Result<()> {
        let base_path = Path::new(&config.log_file_path);
        
        // 删除最旧的文件
        let oldest_path = format!("{}.{}", base_path.display(), config.max_files);
        if Path::new(&oldest_path).exists() {
            fs::remove_file(&oldest_path)?;
        }

        // 轮转现有文件
        for i in (1..config.max_files).rev() {
            let old_path = if i == 1 {
                base_path.to_path_buf()
            } else {
                PathBuf::from(format!("{}.{}", base_path.display(), i))
            };
            
            let new_path = PathBuf::from(format!("{}.{}", base_path.display(), i + 1));
            
            if old_path.exists() {
                fs::rename(&old_path, &new_path)?;
            }
        }

        Ok(())
    }
}

impl Drop for SlowQueryLogger {
    fn drop(&mut self) {
        // 等待所有日志写入完成
        drop(self.tx.clone());
        if let Some(handle) = self.writer_handle.take() {
            let _ = handle.join();
        }
    }
}
```

#### 3.4.2 StatsManager 集成

```rust
pub struct StatsManager {
    // ... 现有字段
    slow_query_logger: Option<Arc<SlowQueryLogger>>,
}

impl StatsManager {
    /// 使用慢查询日志器创建 StatsManager
    pub fn with_slow_query_logger(
        config: MonitoringConfig,
        slow_query_config: SlowQueryConfig,
    ) -> Result<Self, std::io::Error> {
        let logger = Arc::new(SlowQueryLogger::new(slow_query_config)?);
        
        Ok(Self {
            // ... 初始化其他字段
            slow_query_logger: Some(logger),
            // ...
        })
    }

    fn write_slow_query_log(&self, profile: &QueryProfile) {
        if let Some(ref logger) = self.slow_query_logger {
            logger.log(profile);
        } else {
            // 回退到旧的 log::warn 方式
            log::warn!("慢查询：{}", profile.query_text);
        }
    }
}
```

### 3.5 执行计划记录（可选增强）

```rust
/// 执行计划记录配置
#[derive(Debug, Clone)]
pub struct ExplainLogConfig {
    pub enabled: bool,
    pub threshold_ms: u64,  // 超过此阈值才记录执行计划
    pub include_actual_rows: bool,
    pub include_memory_usage: bool,
}

/// 执行计划记录
pub struct ExplainLogger {
    config: ExplainLogConfig,
    tx: mpsc::Sender<String>,
}

impl ExplainLogger {
    pub fn log(&self, plan: &ExecutionPlan, profile: &QueryProfile) {
        if !self.config.enabled {
            return;
        }

        if profile.total_duration_us < self.config.threshold_ms * 1000 {
            return;
        }

        let explain_text = self.format_explain(plan, profile);
        let _ = self.tx.send(explain_text);
    }

    fn format_explain(&self, plan: &ExecutionPlan, profile: &QueryProfile) -> String {
        let mut output = String::new();
        
        output.push_str("=== Execution Plan ===\n");
        output.push_str(&format!("Query: {}\n", profile.query_text));
        output.push_str(&format!("Duration: {}ms\n\n", micros_to_millis(profile.total_duration_us)));
        
        // 格式化执行计划树
        self.format_plan_node(plan.root(), &mut output, 0);
        
        output
    }

    fn format_plan_node(&self, node: &PlanNode, output: &mut String, depth: usize) {
        let indent = "  ".repeat(depth);
        
        output.push_str(&format!("{}{} (cost={:.2}..{:.2}, rows={})\n", 
            indent,
            node.node_type(),
            node.estimated_cost(),
            node.actual_cost().unwrap_or(0.0),
            node.estimated_rows()
        ));

        if self.config.include_actual_rows {
            if let Some(actual) = node.actual_rows() {
                output.push_str(&format!("{}  Actual Rows: {}\n", indent, actual));
            }
        }

        if self.config.include_memory_usage {
            if let Some(mem) = node.memory_used() {
                output.push_str(&format!("{}  Memory: {} KB\n", indent, mem / 1024));
            }
        }

        // 递归处理子节点
        for child in node.children() {
            self.format_plan_node(child, output, depth + 1);
        }
    }
}
```

---

## 四、实施计划

### 4.1 阶段一：核心功能（P0）- 1-2 周

**目标**：实现独立的慢查询日志系统

**任务**：
1. ✅ 创建 `SlowQueryConfig` 配置结构
2. ✅ 实现 `SlowQueryLogger` 核心组件
3. ✅ 实现异步写入机制
4. ✅ 实现日志轮转
5. ✅ 集成到 `StatsManager`
6. ✅ 更新配置文件支持

**验收标准**：
- [ ] 慢查询写入独立文件 `logs/slow_query.log`
- [ ] 异步写入，不阻塞查询
- [ ] 日志文件自动轮转
- [ ] 支持配置阈值和格式

### 4.2 阶段二：增强功能（P1）- 1-2 周

**目标**：提升可分析性和诊断能力

**任务**：
1. ✅ 实现 JSON 格式输出
2. ✅ 实现详细格式输出
3. ✅ 增加 I/O 统计
4. ✅ 实现执行计划记录（类似 auto_explain）
5. ✅ 添加日志分析脚本（Python）

**验收标准**：
- [ ] 支持 JSON 格式输出
- [ ] 日志包含 I/O 统计信息
- [ ] 慢查询自动记录执行计划
- [ ] 提供日志分析工具

### 4.3 阶段三：高级功能（P2）- 2-3 周

**目标**：实现聚合统计和性能优化

**任务**：
1. ✅ 实现聚合统计组件（类似 pg_stat_statements）
2. ✅ 实现采样机制
3. ✅ 提供 HTTP API 查询统计
4. ✅ 集成 Prometheus 指标
5. ✅ 实现告警功能

**验收标准**：
- [ ] 提供查询聚合统计 API
- [ ] 支持采样减少开销
- [ ] 集成监控系统
- [ ] 支持告警配置

---

## 五、测试计划

### 5.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slow_query_logger_creation() {
        let config = SlowQueryConfig::default();
        let logger = SlowQueryLogger::new(config);
        assert!(logger.is_ok());
    }

    #[test]
    fn test_log_rotation() {
        // 测试日志轮转逻辑
    }

    #[test]
    fn test_async_writing() {
        // 测试异步写入不阻塞
    }
}
```

### 5.2 集成测试

```rust
#[test]
fn test_slow_query_logging_integration() {
    // 创建测试数据库
    // 执行慢查询
    // 验证日志文件内容
    // 验证日志轮转
}
```

### 5.3 性能测试

```rust
#[test]
fn test_performance_overhead() {
    // 测量开启慢查询日志前后的性能差异
    // 确保开销 < 5%
}
```

---

## 六、配置示例

### 6.1 开发环境

```toml
[monitoring]
enabled = true
memory_cache_size = 100
slow_query_threshold_ms = 100

[monitoring.slow_query_log]
enabled = true
log_file_path = "logs/slow_query.log"
max_file_size_mb = 50
max_files = 3
verbose_format = true
buffer_size = 50
json_format = false
```

### 6.2 生产环境（低负载）

```toml
[monitoring]
enabled = true
memory_cache_size = 1000
slow_query_threshold_ms = 1000

[monitoring.slow_query_log]
enabled = true
log_file_path = "logs/slow_query.log"
max_file_size_mb = 100
max_files = 5
verbose_format = false
buffer_size = 100
json_format = true
```

### 6.3 生产环境（高负载）

```toml
[monitoring]
enabled = true
memory_cache_size = 1000
slow_query_threshold_ms = 2000

[monitoring.slow_query_log]
enabled = true
log_file_path = "logs/slow_query.log"
max_file_size_mb = 200
max_files = 10
verbose_format = false
buffer_size = 200
json_format = true
```

---

## 七、监控和维护

### 7.1 监控指标

建议监控以下指标：

- **慢查询数量**（每分钟）
- **慢查询平均持续时间**
- **慢查询日志文件大小**
- **异步写入队列长度**
- **日志轮转次数**

### 7.2 告警规则

```rust
let alert_rules = vec![
    // 慢查询数量突增
    AlertRule {
        name: "high_slow_query_rate",
        condition: "slow_query_count_per_minute > 100",
        level: AlertLevel::Warning,
    },
    // 慢查询日志文件过大
    AlertRule {
        name: "large_slow_query_log",
        condition: "slow_query_log_size_mb > 500",
        level: AlertLevel::Warning,
    },
    // 异步写入队列堆积
    AlertRule {
        name: "write_queue_backlog",
        condition: "write_queue_size > 1000",
        level: AlertLevel::Critical,
    },
];
```

### 7.3 维护建议

1. **定期检查**：
   - 每周查看慢查询日志
   - 分析 Top 10 慢查询
   - 识别查询模式趋势

2. **日志清理**：
   - 配置自动轮转
   - 设置合理的保留期限
   - 定期归档旧日志

3. **性能调优**：
   - 根据负载调整阈值
   - 优化慢查询
   - 调整缓冲区大小

---

## 八、总结

本改进方案参考了 PostgreSQL、MySQL、MongoDB 等主流数据库的慢查询日志实现，结合 GraphDB 的实际情况，提供了完整的改进方案：

**核心改进**：
1. ✅ 独立日志文件
2. ✅ 异步写入机制
3. ✅ 日志轮转
4. ✅ 多种日志格式
5. ✅ 可配置性强

**增强功能**：
6. ✅ 执行计划记录
7. ✅ I/O 统计
8. ✅ 聚合统计
9. ✅ 分析工具

通过分阶段实施，可以在保证系统稳定性的前提下，逐步提升 GraphDB 的慢查询监控和诊断能力。
