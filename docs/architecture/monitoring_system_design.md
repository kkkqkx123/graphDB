# GraphDB 监控系统设计方案（简化版）

## 一、设计目标

解决当前监控数据孤岛问题，实现：
1. 执行器指标的统一收集和查询
2. 慢查询的自动识别和记录
3. 历史查询性能的可追溯

## 二、架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────┐
│            数据消费层                    │
│  ┌─────────────┐    ┌─────────────┐    │
│  │ SHOW STATS  │    │ 慢查询日志   │    │
│  │   命令      │    │  (文件)     │    │
│  └─────────────┘    └─────────────┘    │
├─────────────────────────────────────────┤
│            数据服务层                    │
│      ┌─────────────────────┐            │
│      │   StatsManager      │            │
│      │  (扩展现有实现)      │            │
│      │                     │            │
│      │  - 查询指标聚合      │            │
│      │  - 执行器指标收集    │            │
│      │  - 历史数据查询      │            │
│      └─────────────────────┘            │
├─────────────────────────────────────────┤
│            数据存储层                    │
│  ┌─────────────┐    ┌─────────────┐    │
│  │  内存缓存    │    │  文件日志    │    │
│  │ (最近N条)   │    │ (慢查询)    │    │
│  └─────────────┘    └─────────────┘    │
└─────────────────────────────────────────┘
```

### 2.2 模块目录结构

```
src/
├── api/
│   └── service/
│       ├── stats_manager.rs          # 扩展现有实现
│       └── mod.rs
├── storage/
│   └── monitoring/                   # 存储层监控
│       ├── mod.rs
│       └── storage_metrics.rs
└── query/
    └── executor/
        └── base/
            └── executor_stats.rs     # 扩展收集接口
```

## 三、详细设计

### 3.1 数据模型

#### QueryProfile（查询画像）

```
QueryProfile
├── trace_id: UUID              # 查询追踪ID
├── session_id: i64             # 会话ID
├── query_text: String          # 查询文本（摘要）
├── start_time: Timestamp       # 开始时间
├── total_duration_ms: u64      # 总耗时
├── stages: StageMetrics        # 各阶段耗时
│   ├── parse_ms: u64
│   ├── validate_ms: u64
│   ├── plan_ms: u64
│   ├── optimize_ms: u64
│   └── execute_ms: u64
├── executor_stats: Vec<ExecutorStat>  # 执行器统计列表
├── result_count: usize         # 结果行数
├── status: QueryStatus         # 成功/失败
└── error_message: Option<String>
```

#### ExecutorStat（执行器统计）

```
ExecutorStat
├── executor_type: String       # 执行器类型
├── executor_id: i64            # 执行器ID
├── duration_ms: u64            # 执行耗时
├── rows_processed: usize       # 处理行数
└── memory_used: usize          # 内存使用
```

### 3.2 数据收集流程

#### 查询级收集

```
QueryPipelineManager::execute_query_with_metrics()
    │
    ├── 创建 QueryProfile（生成 trace_id）
    │
    ├── 解析阶段
    │   ├── 记录开始时间
    │   ├── Parser::parse()
    │   └── 记录解析耗时
    │
    ├── 验证阶段
    │   ├── 记录开始时间
    │   ├── Validator::validate()
    │   └── 记录验证耗时
    │
    ├── 规划阶段
    │   ├── 记录开始时间
    │   ├── Planner::create_plan()
    │   └── 记录规划耗时
    │
    ├── 优化阶段
    │   ├── 记录开始时间
    │   ├── Optimizer::find_best_plan()
    │   └── 记录优化耗时
    │
    ├── 执行阶段
    │   ├── 记录开始时间
    │   ├── 创建 InstrumentedExecutor（包装执行器）
    │   ├── 执行查询
    │   │   └── 各执行器通过 ExecutorHook 上报统计
    │   └── 记录执行耗时和结果行数
    │
    └── 完成收集
        ├── 判断是否慢查询（阈值可配置，默认10秒）
        │   └── 是 → 写入慢查询日志
        ├── 保存到内存缓存
        └── 更新全局统计
```

#### 执行器级收集

```
InstrumentedExecutor::execute()
    │
    ├── 记录开始时间
    │
    ├── 调用原始执行器::execute()
    │   └── 执行实际逻辑
    │
    ├── 获取执行结果
    │   └── 计算结果行数
    │
    ├── 计算执行耗时
    │
    └── 上报统计
        └── StatsManager::record_executor_stat()
            └── 关联到当前 QueryProfile
```

### 3.3 数据存储

#### 内存缓存

- 使用 VecDeque 实现循环缓冲区
- 容量：默认 1000 条查询画像
- 支持按时间范围、会话ID、用户过滤
- 数据保留：先进先出，自动淘汰旧数据

#### 慢查询日志

- 文件位置：`<data_dir>/logs/slow_queries.log`
- 记录条件：查询耗时超过阈值（默认 10 秒）
- 格式：JSON Lines，每行一个查询画像
- 轮转：按天轮转，保留 7 天

### 3.4 数据消费

#### SHOW STATS 命令增强

```sql
-- 显示全局统计
SHOW STATS;

-- 显示最近查询列表
SHOW STATS QUERIES [LIMIT n];

-- 显示慢查询列表（从内存缓存中筛选）
SHOW SLOW QUERIES [LIMIT n];

-- 显示执行器统计
SHOW STATS EXECUTORS;

-- 显示指定查询的详细画像
SHOW STATS QUERY <trace_id>;
```

## 四、实现步骤

### 步骤1：扩展 StatsManager

位置：`src/api/service/stats_manager.rs`

添加功能：
1. 内存缓存（VecDeque<QueryProfile>）
2. 慢查询日志写入
3. 查询画像查询接口
4. 执行器统计收集接口

### 步骤2：创建执行器包装器

位置：`src/query/executor/base/executor_stats.rs`

添加功能：
1. InstrumentedExecutor 结构体
2. 包装现有执行器的工厂方法
3. 执行时自动上报统计

### 步骤3：修改 QueryPipelineManager

位置：`src/query/query_pipeline_manager.rs`

修改内容：
1. 在执行流程中插入监控点
2. 创建和完成 QueryProfile
3. 使用 InstrumentedExecutor 包装执行器

### 步骤4：扩展 SHOW STATS 执行器

位置：`src/query/executor/admin/query_management/show_stats.rs`

添加功能：
1. 解析新的子命令
2. 调用 StatsManager 查询接口
3. 格式化输出结果

### 步骤5：配置支持

在 `config.toml` 中添加监控配置：

```toml
[monitoring]
enabled = true                          # 总开关
memory_cache_size = 1000                # 内存缓存条数
slow_query_threshold_ms = 10000         # 慢查询阈值
slow_query_log_dir = "logs"             # 慢查询日志目录
slow_query_log_retention_days = 7       # 日志保留天数
```

## 五、性能考虑

### 5.1 开销控制

| 监控点 | 开销 | 控制措施 |
|--------|------|---------|
| 阶段计时 | < 1% | 仅记录时间戳 |
| 执行器统计 | < 2% | 异步批量上报 |
| 内存缓存 | 固定 | 限制容量，自动淘汰 |
| 慢查询日志 | 仅慢查询 | 仅记录超过阈值的查询 |

### 5.2 资源使用

- 内存：固定 1000 条 × 平均 2KB = 约 2MB
- 磁盘：取决于慢查询数量，每天最多几百MB
- CPU：监控本身开销 < 3%

## 六、测试计划

1. **单元测试**：
   - StatsManager 缓存和查询功能
   - 执行器包装器正确性
   - 慢查询检测逻辑

2. **集成测试**：
   - 完整查询流程监控
   - SHOW STATS 命令输出
   - 慢查询日志生成

3. **性能测试**：
   - 监控开启前后的性能对比
   - 内存使用监控

## 七、后续扩展

简化设计保留了扩展性，后续可按需添加：

1. **阶段2**：添加本地持久化存储（SQLite）
2. **阶段3**：添加告警功能（慢查询通知）
3. **阶段4**：添加外部存储适配器（Prometheus）
