# 性能分析设计方案

## 1. 概述

### 1.1 目标

为 GraphDB CLI 提供查询性能分析功能，帮助用户了解查询执行计划、执行时间和资源消耗，优化查询性能。

### 1.2 参考实现

- **psql**：`EXPLAIN` 和 `EXPLAIN ANALYZE` 命令，`\timing` 元命令
- **MySQL**：`EXPLAIN`、`EXPLAIN FORMAT=JSON`，`SHOW PROFILE`
- **neo4j-cli**：`PROFILE` 和 `EXPLAIN` 命令

## 2. 功能需求

### 2.1 核心功能

| 功能            | 说明                                     |
| --------------- | ---------------------------------------- |
| 执行计划展示    | 显示查询的执行计划树，包括算子和成本估算 |
| 执行时间统计    | 显示查询执行的总时间、各阶段耗时         |
| 资源消耗统计    | 显示扫描行数、返回行数、内存使用等       |
| `\timing` 命令  | 切换是否在每次查询后显示执行时间         |
| `\explain` 命令 | 显示查询的执行计划而不实际执行           |
| `\profile` 命令 | 执行查询并显示详细的性能分析数据         |

### 2.2 输出格式

```
QUERY PLAN
─────────────────────────────────────────────────────────────
IndexScan on person  (cost=0.00..35.50 rows=100 width=128)
  Index Cond: (name = 'Alice')
  Filter: (age > 25)
─────────────────────────────────────────────────────────────
Execution Time: 2.345 ms
Rows Scanned: 150
Rows Returned: 100
```

## 3. 架构设计

### 3.1 模块结构

```
src/
├── analysis/
│   ├── mod.rs              # 模块导出
│   ├── explain.rs          # 执行计划解析和格式化
│   ├── profile.rs          # 性能分析数据收集
│   └── timing.rs           # 计时工具
└── command/
    └── executor.rs         # 集成性能分析命令
```

### 3.2 核心数据结构

```rust
pub struct QueryPlan {
    pub plan_type: PlanType,
    pub cost: f64,
    pub rows: usize,
    pub width: usize,
    pub children: Vec<QueryPlan>,
    pub details: HashMap<String, String>,
}

pub enum PlanType {
    IndexScan,
    SeqScan,
    Filter,
    Project,
    Join,
    Aggregate,
    Sort,
    Limit,
    Unknown(String),
}

pub struct ExecutionStats {
    pub total_time_ms: u64,
    pub planning_time_ms: u64,
    pub execution_time_ms: u64,
    pub rows_scanned: u64,
    pub rows_returned: usize,
    pub memory_used_bytes: Option<u64>,
    pub index_hits: Option<u64>,
    pub cache_hits: Option<u64>,
    pub cache_misses: Option<u64>,
}

pub struct ProfileResult {
    pub query: String,
    pub plan: QueryPlan,
    pub stats: ExecutionStats,
    pub warnings: Vec<String>,
}
```

### 3.3 API 扩展

需要在 HTTP 客户端中添加新的 API 调用：

```rust
impl GraphDBHttpClient {
    pub async fn explain(&self, query: &str, session_id: i64) -> Result<QueryPlan> {
        let url = format!("{}/query/explain", self.base_url);
        let request = ExplainRequest {
            query: query.to_string(),
            session_id,
        };
        let response = self.client.post(&url).json(&request).send().await?;
        let plan: QueryPlan = response.json().await?;
        Ok(plan)
    }

    pub async fn profile(&self, query: &str, session_id: i64) -> Result<ProfileResult> {
        let url = format!("{}/query/profile", self.base_url);
        let request = ProfileRequest {
            query: query.to_string(),
            session_id,
        };
        let response = self.client.post(&url).json(&request).send().await?;
        let result: ProfileResult = response.json().await?;
        Ok(result)
    }
}
```

## 4. 元命令设计

### 4.1 `\timing` 命令

**功能**：切换执行时间显示模式

**实现**：

```rust
MetaCommand::Timing => {
    let current = self.formatter.timing_enabled();
    self.formatter.set_timing(!current);
    self.write_output(&format!(
        "Timing {}.",
        if !current { "enabled" } else { "disabled" }
    ))?;
    Ok(true)
}
```

**输出示例**：

```
graphdb(root:test)> \timing
Timing enabled.

graphdb(root:test)> MATCH (p:person) RETURN p LIMIT 10;
...
(10 rows)
Time: 15.234 ms
```

### 4.2 `\explain` 命令

**功能**：显示查询执行计划

**语法**：

- `\explain <query>` - 显示执行计划
- `\explain analyze <query>` - 执行查询并显示实际执行统计
- `\explain format=json <query>` - 以 JSON 格式输出

**实现**：

```rust
MetaCommand::Explain { query, analyze, format } => {
    if !self.conditional_stack.is_active() {
        return Ok(true);
    }

    if analyze {
        let result = session_mgr.profile(&query).await?;
        let output = self.formatter.format_profile(&result, format);
        self.write_output(&output)?;
    } else {
        let plan = session_mgr.explain(&query).await?;
        let output = self.formatter.format_explain(&plan, format);
        self.write_output(&output)?;
    }
    Ok(true)
}
```

### 4.3 `\profile` 命令

**功能**：执行查询并显示详细性能分析

**语法**：`\profile <query>`

**实现**：

```rust
MetaCommand::Profile { query } => {
    if !self.conditional_stack.is_active() {
        return Ok(true);
    }

    let result = session_mgr.profile(&query).await?;
    let output = self.formatter.format_profile(&result, OutputFormat::Table);
    self.write_output(&output)?;
    Ok(true)
}
```

## 5. 执行计划格式化

### 5.1 树形格式化

```rust
impl QueryPlan {
    pub fn format_tree(&self, indent: usize) -> String {
        let mut output = String::new();
        let prefix = " ".repeat(indent * 2);

        output.push_str(&format!(
            "{}{}  (cost={:.2}..{:.2} rows={} width={})\n",
            prefix,
            self.plan_type.as_str(),
            self.cost,
            self.cost + self.rows as f64 * 0.1,
            self.rows,
            self.width
        ));

        for (key, value) in &self.details {
            output.push_str(&format!("{}  {}: {}\n", prefix, key, value));
        }

        for child in &self.children {
            output.push_str(&child.format_tree(indent + 1));
        }

        output
    }
}
```

### 5.2 JSON 格式化

```rust
impl QueryPlan {
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "type": self.plan_type.as_str(),
            "cost": self.cost,
            "rows": self.rows,
            "width": self.width,
            "details": self.details,
            "children": self.children.iter().map(|c| c.to_json()).collect::<Vec<_>>()
        })
    }
}
```

## 6. 计时器实现

### 6.1 高精度计时

```rust
use std::time::{Duration, Instant};

pub struct QueryTimer {
    start: Instant,
    phases: Vec<(String, Duration)>,
}

impl QueryTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            phases: Vec::new(),
        }
    }

    pub fn record_phase(&mut self, name: &str) {
        let elapsed = self.start.elapsed();
        let last = self.phases.last().map(|(_, d)| d).unwrap_or(&Duration::ZERO);
        let phase_time = elapsed - *last;
        self.phases.push((name.to_string(), phase_time));
    }

    pub fn total_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    pub fn phase_ms(&self, name: &str) -> Option<u64> {
        self.phases
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, d)| d.as_millis() as u64)
    }
}
```

### 6.2 集成到查询执行

```rust
async fn execute_query(&mut self, query: &str, session_mgr: &mut SessionManager) -> Result<bool> {
    let mut timer = QueryTimer::new();

    let result = session_mgr.execute_query(query).await?;
    timer.record_phase("execution");

    let output = self.formatter.format_result(&result);
    self.write_output(&output)?;

    if self.formatter.timing_enabled() {
        let time_str = format!("Time: {:.3} ms", timer.total_ms() as f64);
        self.write_output(&time_str)?;
    }

    Ok(true)
}
```

## 7. 性能统计展示

### 7.1 统计信息格式化

```rust
impl ExecutionStats {
    pub fn format_summary(&self) -> String {
        let mut output = String::new();

        output.push_str("─────────────────────────────────────────────────────────────\n");
        output.push_str("Execution Statistics\n");
        output.push_str("─────────────────────────────────────────────────────────────\n");
        output.push_str(&format!("Total Time:      {:.3} ms\n", self.total_time_ms as f64));
        output.push_str(&format!("Planning Time:   {:.3} ms\n", self.planning_time_ms as f64));
        output.push_str(&format!("Execution Time:  {:.3} ms\n", self.execution_time_ms as f64));
        output.push_str(&format!("Rows Scanned:    {}\n", self.rows_scanned));
        output.push_str(&format!("Rows Returned:   {}\n", self.rows_returned));

        if let Some(mem) = self.memory_used_bytes {
            output.push_str(&format!("Memory Used:     {:.2} MB\n", mem as f64 / 1024.0 / 1024.0));
        }

        if let (Some(hits), Some(misses)) = (self.cache_hits, self.cache_misses) {
            let total = hits + misses;
            let hit_rate = if total > 0 { hits as f64 / total as f64 * 100.0 } else { 0.0 };
            output.push_str(&format!("Cache Hit Rate:  {:.1}%\n", hit_rate));
        }

        output
    }
}
```

### 7.2 警告信息

```rust
impl ProfileResult {
    pub fn analyze_warnings(&mut self) {
        if self.stats.rows_scanned > 10000 && self.stats.rows_returned < 100 {
            self.warnings.push(
                "High scan-to-result ratio. Consider adding an index.".to_string()
            );
        }

        if self.stats.execution_time_ms > 1000 {
            self.warnings.push(
                "Query execution time exceeds 1 second. Consider optimization.".to_string()
            );
        }

        if let Some(misses) = self.stats.cache_misses {
            if misses > 1000 {
                self.warnings.push(
                    "High cache miss count. Data may not be in memory.".to_string()
                );
            }
        }
    }
}
```

## 8. 命令解析器扩展

### 8.1 新增元命令

```rust
pub enum MetaCommand {
    Timing,
    Explain {
        query: String,
        analyze: bool,
        format: ExplainFormat,
    },
    Profile {
        query: String,
    },
}

pub enum ExplainFormat {
    Text,
    Json,
    Dot,
}

fn parse_meta_command(input: &str) -> Result<Command> {
    let trimmed = input.trim_start_matches('\\');

    match trimmed.split_whitespace().next() {
        Some("timing") => Ok(Command::MetaCommand(MetaCommand::Timing)),
        Some("explain") => parse_explain_command(trimmed),
        Some("profile") => parse_profile_command(trimmed),
        _ => parse_other_meta_command(trimmed),
    }
}

fn parse_explain_command(input: &str) -> Result<Command> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let mut analyze = false;
    let mut format = ExplainFormat::Text;
    let mut query_start = 1;

    for (i, part) in parts.iter().skip(1).enumerate() {
        if *part == "analyze" {
            analyze = true;
            query_start += 1;
        } else if part.starts_with("format=") {
            format = match part.split('=').nth(1) {
                Some("json") => ExplainFormat::Json,
                Some("dot") => ExplainFormat::Dot,
                _ => ExplainFormat::Text,
            };
            query_start += 1;
        } else {
            break;
        }
    }

    let query = parts[query_start..].join(" ");
    Ok(Command::MetaCommand(MetaCommand::Explain { query, analyze, format }))
}
```

## 9. 测试用例

### 9.1 Timing 功能

| 操作           | 预期输出              |
| -------------- | --------------------- |
| `\timing`      | "Timing enabled."     |
| 执行查询       | 显示 "Time: X.XXX ms" |
| 再次 `\timing` | "Timing disabled."    |
| 执行查询       | 不显示时间            |

### 9.2 Explain 功能

| 输入                                 | 预期输出               |
| ------------------------------------ | ---------------------- |
| `\explain MATCH (p:person) RETURN p` | 显示执行计划树         |
| `\explain analyze MATCH ...`         | 显示执行计划和实际统计 |
| `\explain format=json MATCH ...`     | 以 JSON 格式输出       |

### 9.3 Profile 功能

| 输入                            | 预期输出                   |
| ------------------------------- | -------------------------- |
| `\profile MATCH (p:person) ...` | 显示查询结果和详细性能分析 |

## 10. 实现步骤

### Step 1: 实现 Timing 功能（0.5 天）

- 在 `OutputFormatter` 中添加 timing 开关
- 在查询执行后显示时间
- 添加 `\timing` 元命令

### Step 2: 实现执行计划数据结构（1 天）

- 定义 `QueryPlan`、`ExecutionStats`、`ProfileResult`
- 实现序列化/反序列化
- 实现树形格式化

### Step 3: 扩展 HTTP 客户端（1 天）

- 添加 `explain()` 和 `profile()` API
- 处理服务端响应

### Step 4: 实现元命令（1 天）

- 添加 `\explain` 命令解析
- 添加 `\profile` 命令解析
- 集成到命令执行器

### Step 5: 实现格式化和警告（0.5 天）

- 实现多种输出格式
- 实现性能警告检测

### Step 6: 测试（0.5 天）

- 单元测试
- 集成测试
- 文档更新
