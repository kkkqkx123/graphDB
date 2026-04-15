# 慢查询日志系统设计

## 一、概述

慢查询日志是性能监控系统的重要组成部分，用于记录执行时间超过阈值的查询，帮助识别和优化性能瓶颈。

### 1.1 设计目标

1. **独立性**: 与主日志系统（flexi_logger）分离，独立写入文件
2. **低开销**: 异步写入，不阻塞查询执行流程
3. **可配置**: 支持动态调整阈值和日志配置
4. **易分析**: 结构化的日志格式，便于后续分析和工具处理

### 1.2 日志文件

| 日志类型 | 文件路径 | 用途 |
|---------|---------|------|
| 主日志 | `logs/graphdb.log` | 系统运行日志 |
| 慢查询日志 | `logs/slow_query.log` | 慢查询记录 |
| 错误日志 | `logs/error.log` | 错误信息 |

---

## 二、日志格式

### 2.1 标准格式

```
[时间戳] [SLOW_QUERY] [trace_id=xxx] [session_id=xxx] [duration=xxxms] [status=xxx] 查询文本
```

### 2.2 详细格式

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

### 2.3 示例

#### 2.3.1 标准格式示例

```
[2026-04-15 10:23:45.123] [SLOW_QUERY] [trace_id=550e8400-e29b-41d4-a716-446655440000] [session_id=12345] [duration=1523ms] [status=success] MATCH (n:User) WHERE n.age > 18 RETURN n
```

#### 2.3.2 失败查询示例

```
[2026-04-15 10:24:12.456] [SLOW_QUERY] [trace_id=660e8400-e29b-41d4-a716-446655440001] [session_id=12346] [duration=2045ms] [status=failed] [error_type=ExecutionError] [error_phase=Execute] MATCH (n:Invalid) RETURN n
```

---

## 三、架构设计

### 3.1 组件结构

```
┌─────────────────────────────────────────────────────────────┐
│                     Query Execution                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    StatsManager                             │
│  - 记录查询画像                                              │
│  - 判断是否为慢查询                                          │
│  - 发送到慢查询日志器                                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  SlowQueryLogger                            │
│  - 格式化日志条目                                            │
│  - 异步写入文件                                              │
│  - 日志轮转管理                                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   慢查询日志文件                             │
│  logs/slow_query.log                                        │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 核心组件

#### 3.2.1 慢查询日志配置

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
    /// 最大文件大小（MB）
    pub max_file_size_mb: u64,
    /// 保留的文件数量
    pub max_files: u32,
    /// 是否使用详细格式
    pub verbose_format: bool,
    /// 异步写入缓冲区大小
    pub buffer_size: usize,
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
        }
    }
}
```

#### 3.2.2 慢查询日志记录器

```rust
/// 慢查询日志记录器
pub struct SlowQueryLogger {
    config: SlowQueryConfig,
    /// 异步写入通道
    tx: mpsc::Sender<String>,
    /// 后台写入线程句柄
    writer_handle: Option<thread::JoinHandle<()>>,
    /// 当前文件大小（字节）
    current_file_size: AtomicU64,
    /// 当前文件路径
    current_file_path: Mutex<PathBuf>,
}

impl SlowQueryLogger {
    /// 创建新的慢查询日志记录器
    pub fn new(config: SlowQueryConfig) -> Result<Self, SlowQueryError> {
        // 创建日志目录
        if let Some(parent) = Path::new(&config.log_file_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 创建异步通道
        let (tx, rx) = mpsc::channel::<String>(config.buffer_size);
        
        // 启动后台写入线程
        let writer_handle = Some(Self::spawn_writer_thread(
            rx,
            config.clone(),
        ));
        
        Ok(Self {
            config,
            tx,
            writer_handle,
            current_file_size: AtomicU64::new(0),
            current_file_path: Mutex::new(PathBuf::from(&config.log_file_path)),
        })
    }
    
    /// 记录慢查询
    pub fn log_slow_query(&self, profile: &QueryProfile) {
        if !self.config.enabled {
            return;
        }
        
        if profile.total_duration_ms < self.config.threshold_ms {
            return;
        }
        
        // 格式化日志条目
        let log_entry = if self.config.verbose_format {
            self.format_verbose_log(profile)
        } else {
            self.format_simple_log(profile)
        };
        
        // 异步发送（非阻塞）
        let _ = self.tx.try_send(log_entry);
    }
    
    /// 格式化简单日志
    fn format_simple_log(&self, profile: &QueryProfile) -> String {
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
            profile.total_duration_ms,
            status_str,
            profile.query_text
        )
    }
    
    /// 格式化详细日志
    fn format_verbose_log(&self, profile: &QueryProfile) -> String {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let status_str = match profile.status {
            QueryStatus::Success => "success",
            QueryStatus::Failed => "failed",
        };
        
        let mut log = String::new();
        log.push_str(&format!("[{}] [SLOW_QUERY]\n", timestamp));
        log.push_str(&format!("  trace_id: {}\n", profile.trace_id));
        log.push_str(&format!("  session_id: {}\n", profile.session_id));
        log.push_str(&format!("  query_text: {}\n", profile.query_text));
        log.push_str(&format!("  duration: {}ms\n", profile.total_duration_ms));
        log.push_str(&format!("  status: {}\n", status_str));
        
        if let Some(ref error_info) = profile.error_info {
            log.push_str(&format!("  error_type: {}\n", error_info.error_type));
            log.push_str(&format!("  error_phase: {}\n", error_info.error_phase));
            log.push_str(&format!("  error_message: {}\n", error_info.error_message));
        }
        
        log.push_str("  stages:\n");
        log.push_str(&format!("    parse: {}ms\n", profile.stages.parse_ms));
        log.push_str(&format!("    validate: {}ms\n", profile.stages.validate_ms));
        log.push_str(&format!("    plan: {}ms\n", profile.stages.plan_ms));
        log.push_str(&format!("    optimize: {}ms\n", profile.stages.optimize_ms));
        log.push_str(&format!("    execute: {}ms\n", profile.stages.execute_ms));
        
        log.push_str(&format!("  result_count: {}\n", profile.result_count));
        
        if !profile.executor_stats.is_empty() {
            log.push_str("  executor_stats:\n");
            for stat in &profile.executor_stats {
                log.push_str(&format!(
                    "    - executor: {}, duration: {}ms, rows: {}\n",
                    stat.executor_type,
                    stat.duration_ms,
                    stat.rows_processed
                ));
            }
        }
        
        log.push('\n');
        log
    }
    
    /// 启动后台写入线程
    fn spawn_writer_thread(
        rx: mpsc::Receiver<String>,
        config: SlowQueryConfig,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut file = match File::create(&config.log_file_path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to create slow query log file: {}", e);
                    return;
                }
            };
            
            let mut writer = BufWriter::new(file);
            let max_size_bytes = config.max_file_size_mb * 1024 * 1024;
            let mut current_size = 0u64;
            
            while let Ok(log_entry) = rx.recv() {
                // 检查是否需要日志轮转
                if current_size + log_entry.len() as u64 > max_size_bytes {
                    let _ = writer.flush();
                    drop(writer);
                    
                    // 执行日志轮转
                    if let Err(e) = Self::rotate_logs(&config) {
                        eprintln!("Failed to rotate slow query log: {}", e);
                    }
                    
                    // 重新创建文件
                    match File::create(&config.log_file_path) {
                        Ok(f) => {
                            writer = BufWriter::new(f);
                            current_size = 0;
                        }
                        Err(e) => {
                            eprintln!("Failed to recreate slow query log file: {}", e);
                            continue;
                        }
                    }
                }
                
                // 写入日志
                if let Err(e) = writer.write_all(log_entry.as_bytes()) {
                    eprintln!("Failed to write slow query log: {}", e);
                    continue;
                }
                
                current_size += log_entry.len() as u64;
                
                // 定期刷新
                if current_size % 4096 == 0 {
                    let _ = writer.flush();
                }
            }
            
            // 确保所有数据写入
            let _ = writer.flush();
        })
    }
    
    /// 日志轮转
    fn rotate_logs(config: &SlowQueryConfig) -> Result<(), std::io::Error> {
        let base_path = Path::new(&config.log_file_path);
        
        // 删除最旧的日志
        let oldest_path = format!("{}.{}", base_path.display(), config.max_files);
        if Path::new(&oldest_path).exists() {
            fs::remove_file(&oldest_path)?;
        }
        
        // 重命名现有日志
        for i in (1..config.max_files).rev() {
            let old_path = format!("{}.{}", base_path.display(), i);
            let new_path = format!("{}.{}", base_path.display(), i + 1);
            
            if Path::new(&old_path).exists() {
                fs::rename(&old_path, &new_path)?;
            }
        }
        
        // 重命名当前日志
        let new_path = format!("{}.1", base_path.display());
        if base_path.exists() {
            fs::rename(base_path, &new_path)?;
        }
        
        Ok(())
    }
    
    /// 更新配置
    pub fn update_config(&self, config: SlowQueryConfig) {
        // 可以通过 channel 发送配置更新消息
        // 简化实现：直接替换（实际需要考虑线程安全）
    }
}

impl Drop for SlowQueryLogger {
    fn drop(&mut self) {
        // 等待后台线程完成
        if let Some(handle) = self.writer_handle.take() {
            let _ = handle.join();
        }
    }
}
```

---

## 四、与 StatsManager 集成

### 4.1 集成方式

```rust
// src/core/stats/manager.rs

impl StatsManager {
    pub fn with_slow_query_logger(
        config: crate::config::MonitoringConfig,
        slow_query_config: SlowQueryConfig,
    ) -> Self {
        let cache_size = config.memory_cache_size;
        let slow_query_logger = Arc::new(SlowQueryLogger::new(slow_query_config).unwrap());
        
        Self {
            metrics: Arc::new(DashMap::new()),
            space_metrics: Arc::new(DashMap::new()),
            last_query_metrics: Arc::new(Mutex::new(None)),
            query_profiles: Arc::new(Mutex::new(VecDeque::with_capacity(cache_size))),
            config,
            error_stats: ErrorStatsManager::new(),
            slow_query_logger: Some(slow_query_logger),
        }
    }
    
    pub fn record_query_profile(&self, profile: QueryProfile) {
        if !self.config.enabled {
            return;
        }
        
        // 记录到内存缓存
        {
            let mut profiles = self.query_profiles.lock();
            if profiles.len() >= self.config.memory_cache_size {
                profiles.pop_front();
            }
            profiles.push_back(profile.clone());
        }
        
        // 记录到慢查询日志
        if let Some(ref logger) = self.slow_query_logger {
            logger.log_slow_query(&profile);
        }
        
        // 如果超过慢查询阈值，也记录到日志
        if profile.total_duration_ms >= self.config.slow_query_threshold_ms {
            self.write_slow_query_log(&profile);
        }
    }
}
```

---

## 五、日志分析工具

### 5.1 简单的日志分析脚本

```python
#!/usr/bin/env python3
"""
慢查询日志分析工具
"""

import re
from collections import defaultdict
from datetime import datetime

# 日志格式正则表达式
LOG_PATTERN = re.compile(
    r'\[(?P<timestamp>[\d\-\s:.]+)\] \[SLOW_QUERY\] '
    r'\[trace_id=(?P<trace_id>[\w\-]+)\] '
    r'\[session_id=(?P<session_id>\d+)\] '
    r'\[duration=(?P<duration>\d+)ms\] '
    r'\[status=(?P<status>\w+)\] '
    r'(?P<query>.+)$'
)

def parse_log_file(log_path):
    """解析慢查询日志文件"""
    slow_queries = []
    
    with open(log_path, 'r') as f:
        for line in f:
            match = LOG_PATTERN.match(line.strip())
            if match:
                slow_queries.append({
                    'timestamp': match.group('timestamp'),
                    'trace_id': match.group('trace_id'),
                    'session_id': int(match.group('session_id')),
                    'duration': int(match.group('duration')),
                    'status': match.group('status'),
                    'query': match.group('query'),
                })
    
    return slow_queries

def analyze_slow_queries(queries):
    """分析慢查询"""
    print(f"=== 慢查询分析报告 ===\n")
    print(f"总慢查询数：{len(queries)}\n")
    
    # 按持续时间排序
    top_10 = sorted(queries, key=lambda x: x['duration'], reverse=True)[:10]
    
    print("Top 10 最慢查询:")
    for i, q in enumerate(top_10, 1):
        print(f"{i}. [{q['duration']}ms] {q['query'][:80]}...")
    
    # 按查询模式分组
    pattern_counts = defaultdict(int)
    for q in queries:
        # 简化查询模式（移除具体值）
        pattern = re.sub(r"'[^']*'", "'?'", q['query'])
        pattern = re.sub(r'\d+', '?', pattern)
        pattern_counts[pattern] += 1
    
    print("\n最常见的查询模式:")
    for pattern, count in sorted(pattern_counts.items(), key=lambda x: x[1], reverse=True)[:5]:
        print(f"  {count}x: {pattern[:60]}...")
    
    # 错误查询统计
    error_queries = [q for q in queries if q['status'] == 'failed']
    print(f"\n失败查询数：{len(error_queries)}")
    if error_queries:
        print("失败查询示例:")
        for q in error_queries[:3]:
            print(f"  - {q['query'][:60]}...")

def main():
    import sys
    
    if len(sys.argv) < 2:
        print("用法：python analyze_slow_query.py <log_file>")
        sys.exit(1)
    
    log_path = sys.argv[1]
    queries = parse_log_file(log_path)
    analyze_slow_queries(queries)

if __name__ == '__main__':
    main()
```

### 5.2 使用示例

```bash
# 分析慢查询日志
python analyze_slow_query.py logs/slow_query.log

# 输出示例：
# === 慢查询分析报告 ===
# 
# 总慢查询数：156
# 
# Top 10 最慢查询:
# 1. [5234ms] MATCH (n:User) WHERE n.age > 18 RETURN n...
# 2. [3421ms] MATCH (n:Product)-[:BELONGS_TO]->(c:Category) RETURN n, c...
# 3. [2987ms] GO FROM "player100" OVER follow RECURSIVELY...
# 
# 最常见的查询模式:
#   45x: MATCH (n:User) WHERE n.age > ? RETURN n
#   32x: MATCH (n:Product)-[:BELONGS_TO]->(c:Category) RETURN n, c
#   28x: GO FROM ? OVER follow RECURSIVELY
# 
# 失败查询数：12
# 失败查询示例:
#   - MATCH (n:Invalid) RETURN n...
```

---

## 六、配置示例

### 6.1 配置文件（toml）

```toml
# config.toml

[monitoring]
enabled = true
memory_cache_size = 100
slow_query_threshold_ms = 1000

[monitoring.slow_query_log]
enabled = true
log_file_path = "logs/slow_query.log"
max_file_size_mb = 100
max_files = 5
verbose_format = false
buffer_size = 100
```

### 6.2 动态配置更新

```rust
// 运行时更新慢查询日志配置
let new_config = SlowQueryConfig {
    threshold_ms: 2000,  // 调整为 2 秒
    verbose_format: true, // 启用详细格式
    ..Default::default()
};

slow_query_logger.update_config(new_config);
```

---

## 七、最佳实践

### 7.1 配置建议

| 场景 | 阈值 | 详细格式 | 缓冲区大小 |
|------|------|---------|-----------|
| 开发环境 | 100ms | true | 50 |
| 测试环境 | 500ms | false | 100 |
| 生产环境（低负载） | 1000ms | false | 100 |
| 生产环境（高负载） | 2000ms | false | 200 |

### 7.2 日志轮转策略

建议使用日志轮转，保留最近 N 个文件：
- `slow_query.log` - 当前日志
- `slow_query.log.1` - 最近的归档
- `slow_query.log.2` - 次近的归档
- ...

### 7.3 性能优化

1. **异步写入**: 使用 channel 和后台线程异步写入
2. **批量刷新**: 定期批量刷新，减少 I/O 次数
3. **缓冲区**: 设置合适的缓冲区大小，平衡内存和性能
4. **采样**: 高负载场景可以考虑采样记录

---

## 八、监控和维护

### 8.1 监控指标

建议监控以下指标：
- 慢查询数量（每分钟）
- 慢查询平均持续时间
- 慢查询日志文件大小
- 异步写入队列长度

### 8.2 告警规则

```rust
// 示例告警规则
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

---

## 九、总结

慢查询日志系统提供以下核心能力：

1. ✅ **独立日志**: 与主日志分离，便于分析
2. ✅ **异步写入**: 不阻塞查询执行
3. ✅ **可配置**: 支持动态调整阈值和格式
4. ✅ **日志轮转**: 自动管理日志文件大小
5. ✅ **易于分析**: 结构化的日志格式

通过慢查询日志，可以：
- 识别性能瓶颈查询
- 分析查询模式趋势
- 发现异常查询行为
- 优化查询执行计划
