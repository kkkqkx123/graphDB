# Explain/Profile 功能参考信息收集

## 1. PostgreSQL EXPLAIN ANALYZE 实现机制

### 1.1 核心概念

PostgreSQL的`EXPLAIN ANALYZE`是业界最成熟的查询分析实现之一，其核心特点是：

- **实际执行**: 不仅生成计划，还实际执行查询
- **真实统计**: 收集每个节点的实际行数和执行时间
- **预估对比**: 显示优化器预估 vs 实际执行数据
- **微秒精度**: 时间统计精确到微秒级别

### 1.2 输出格式示例

```sql
EXPLAIN ANALYZE SELECT * FROM tenk1 t1, tenk2 t2 
WHERE t1.unique1 < 100 AND t1.unique2 = t2.unique2;
```

```
QUERY PLAN
-------------------------------------------------------------------
 Hash Join  (cost=226.23..709.73 rows=100 width=488) 
            (actual time=0.515..2.920 rows=100 loops=1)
   Hash Cond: (t2.unique2 = t1.unique2)
   ->  Seq Scan on tenk2 t2  
       (cost=0.00..445.00 rows=10000 width=244) 
       (actual time=0.026..1.790 rows=10000 loops=1)
   ->  Hash  
       (cost=224.98..224.98 rows=100 width=244) 
       (actual time=0.476..0.477 rows=100 loops=1)
         Buckets: 1024  Batches: 1  Memory Usage: 35kB
         ->  Bitmap Heap Scan on tenk1 t1  
             (cost=5.06..224.98 rows=100 width=244) 
             (actual time=0.030..0.450 rows=100 loops=1)
               Recheck Cond: (unique1 < 100)
               Heap Blocks: exact=90
               ->  Bitmap Index Scan on tenk1_unique1  
                   (cost=0.00..5.04 rows=100 width=0) 
                   (actual time=0.013..0.013 rows=100 loops=1)
                     Index Cond: (unique1 < 100)
 Planning Time: 0.187 ms
 Execution Time: 3.036 ms
```

### 1.3 关键字段说明

| 字段 | 说明 |
|------|------|
| `cost=XX..YY` | 优化器预估的启动成本和总成本 |
| `rows=NN` | 优化器预估的输出行数 |
| `width=NN` | 优化器预估的平均行宽度(字节) |
| `actual time=XX..YY` | 实际首次输出行时间和总时间 |
| `rows=NN` (actual) | 实际输出行数 |
| `loops=N` | 执行器被调用的次数 |

### 1.4 BUFFERS选项

```sql
EXPLAIN (ANALYZE, BUFFERS) SELECT * FROM tenk1 WHERE unique1 < 100;
```

```
QUERY PLAN
-------------------------------------------------------------------
 Bitmap Heap Scan on tenk1
   Buffers: shared hit=14 read=3
   ->  Bitmap Index Scan on tenk1_unique1
         Buffers: shared hit=4 read=3
```

**Buffers统计**：
- `shared hit`: 从共享缓存读取的块数
- `shared read`: 从磁盘读取的块数
- `shared dirtied`: 被修改的块数
- `shared written`: 被写入磁盘的块数

### 1.5 实现原理

PostgreSQL通过在执行器中植入**Instrumentation**结构实现统计收集：

```c
typedef struct Instrumentation {
    bool        running;        /* 是否正在计时 */
    struct timeval starttime;   /* 开始时间 */
    struct timeval counter;     /* 累计时间 */
    
    double      firsttuple;     /* 首次输出行时间 */
    double      tuplecount;     /* 输出行数 */
    
    /* 缓冲区统计 */
    BufferUsage bufusage_start;
    BufferUsage bufusage;
} Instrumentation;
```

每个PlanState节点都有一个`instrument`字段，在执行前后记录统计信息。

---

## 2. 其他数据库实现对比

### 2.1 MySQL/MariaDB

```sql
EXPLAIN ANALYZE SELECT * FROM employees WHERE salary > 50000;
```

特点：
- 支持`EXPLAIN ANALYZE`（MySQL 8.0.18+）
- 显示实际时间和行数
- 支持`FORMAT=TREE`和`FORMAT=JSON`

```
-> Filter: (employees.salary > 50000)  
   (cost=100.50 rows=1000) (actual time=0.523..2.920 rows=100 loops=1)
    -> Table scan on employees  
       (cost=100.50 rows=10000) (actual time=0.026..1.790 rows=10000 loops=1)
```

### 2.2 SQL Server

```sql
SET STATISTICS PROFILE ON;
SET STATISTICS TIME ON;
SET STATISTICS IO ON;
SELECT * FROM Employees WHERE Salary > 50000;
SET STATISTICS PROFILE OFF;
```

特点：
- 通过`SET`语句启用统计
- 显示详细的I/O统计
- 支持图形化执行计划

### 2.3 Oracle

```sql
EXPLAIN PLAN FOR SELECT * FROM employees WHERE salary > 50000;
SELECT * FROM TABLE(DBMS_XPLAN.DISPLAY);

-- 实际执行统计
SELECT /*+ GATHER_PLAN_STATISTICS */ * FROM employees WHERE salary > 50000;
SELECT * FROM TABLE(DBMS_XPLAN.DISPLAY_CURSOR(format=>'ALLSTATS LAST'));
```

特点：
- 需要`GATHER_PLAN_STATISTICS` hint
- 支持`DBMS_XPLAN`包格式化输出
- 显示A-Rows(实际)和E-Rows(预估)对比

### 2.4 MongoDB

```javascript
db.collection.explain("executionStats").find({age: {$gt: 18}})
```

特点：
- `queryPlanner`: 仅显示计划
- `executionStats`: 显示执行统计
- `allPlansExecution`: 显示所有候选计划

```json
{
  "executionStats": {
    "executionSuccess": true,
    "nReturned": 1000,
    "executionTimeMillis": 15,
    "totalKeysExamined": 1000,
    "totalDocsExamined": 1000,
    "executionStages": {
      "stage": "IXSCAN",
      "nReturned": 1000,
      "executionTimeMillisEstimate": 10,
      "works": 1001,
      "advanced": 1000,
      "keysExamined": 1000
    }
  }
}
```

---

## 3. 图数据库特定考量

### 3.1 NebulaGraph实现

NebulaGraph作为本项目的参考实现，其Explain/Profile功能包括：

```ngql
EXPLAIN {FORMAT=row} GO FROM "player100" OVER follow YIELD dst(edge);
PROFILE {FORMAT=dot} GO FROM "player100" OVER follow YIELD dst(edge);
```

**特点**：
- 支持row/dot两种输出格式
- Profile实际执行并收集统计
- 显示每个执行器的输出行数
- 支持依赖关系图展示

**执行计划节点类型**：
- `GetNeighbors`: 获取邻居节点
- `GetVertices`: 获取顶点属性
- `Filter`: 过滤数据
- `Project`: 投影列
- `Loop`: 循环执行（用于多跳查询）

### 3.2 Neo4j PROFILE

```cypher
PROFILE MATCH (p:Person)-[:KNOWS]->(f:Person)
WHERE p.name = 'Alice'
RETURN f.name
```

输出包含：
- `Operator`: 操作符类型
- `Estimated Rows`: 预估行数
- `Rows`: 实际行数
- `DB Hits`: 数据库访问次数
- `Memory`: 内存使用
- `Page Cache Hits/Misses`: 页缓存统计

### 3.3 Amazon Neptune

Neptune支持`explain`和`profile`两种模式：

```sparql
# explain mode
curl -X POST https://neptune-endpoint/sparql \
  -d "query=SELECT * WHERE {?s ?p ?o}" \
  -d "explain=static"

# profile mode  
curl -X POST https://neptune-endpoint/sparql \
  -d "query=SELECT * WHERE {?s ?p ?o}" \
  -d "explain=dynamic"
```

---

## 4. 关键设计模式

### 4.1 Visitor模式（计划描述）

```rust
// 使用Visitor模式遍历计划树并生成描述
pub trait PlanNodeVisitor {
    fn visit_scan(&mut self, node: &ScanNode);
    fn visit_filter(&mut self, node: &FilterNode);
    fn visit_join(&mut self, node: &JoinNode);
    // ...
}

pub struct DescribeVisitor {
    descriptions: Vec<PlanNodeDescription>,
}

impl PlanNodeVisitor for DescribeVisitor {
    fn visit_scan(&mut self, node: &ScanNode) {
        let desc = PlanNodeDescription::new("Scan", node.id())
            .with_description("table", node.table_name())
            .with_description("index", node.index_name());
        self.descriptions.push(desc);
    }
    // ...
}
```

### 4.2 Decorator模式（统计包装）

```rust
// 使用Decorator模式包装执行器以收集统计
pub struct InstrumentedExecutor<E> {
    inner: E,
    stats: ExecutionStats,
}

impl<E: Executor> Executor for InstrumentedExecutor<E> {
    fn execute(&mut self) -> Result<ExecutionResult> {
        let start = Instant::now();
        let result = self.inner.execute()?;
        self.stats.record_execution(start.elapsed());
        self.stats.record_rows(result.len());
        Ok(result)
    }
}
```

### 4.3 Context模式（全局统计）

```rust
// 使用Context模式管理全局统计
pub struct ExecutionContext {
    node_stats: HashMap<NodeId, NodeStats>,
    global_stats: GlobalStats,
    config: ExecutionConfig,
}

impl ExecutionContext {
    pub fn record_node_start(&self, node_id: NodeId) {
        // 记录节点开始执行
    }
    
    pub fn record_node_end(&self, node_id: NodeId, stats: NodeStats) {
        // 记录节点执行完成
    }
}
```

---

## 5. 性能考量

### 5.1 统计收集开销

PostgreSQL文档明确指出：

> "EXPLAIN ANALYZE adds profiling overhead to query execution."

**开销来源**：
1. 计时器调用（gettimeofday）
2. 行数计数器递增
3. 缓冲区统计跟踪
4. 内存分配跟踪

**优化策略**：
- 使用CPU周期计数器（rdtsc）代替系统调用
- 批量更新统计（而非每行更新）
- 可选的轻量级统计模式

### 5.2 精度与开销权衡

| 精度级别 | 时间精度 | 行数统计 | 内存追踪 | I/O统计 | 典型开销 |
|---------|---------|---------|---------|---------|---------|
| 基础 | 毫秒 | 估算 | 无 | 无 | <1% |
| 标准 | 微秒 | 精确 | 峰值 | 块级 | 2-5% |
| 详细 | 纳秒 | 精确 | 详细 | 字节级 | 5-10% |
| 全量 | 纳秒 | 逐行 | 分配追踪 | 完整 | 10-20% |

### 5.3 生产环境建议

1. **采样统计**: 仅对一定比例的查询收集详细统计
2. **异步收集**: 将统计写入异步队列，避免阻塞执行
3. **阈值触发**: 仅对执行时间超过阈值的查询收集统计
4. **动态开关**: 支持运行时启用/禁用统计收集

---

## 6. 测试验证方法

### 6.1 功能测试

```rust
#[test]
fn test_explain_basic() {
    let result = execute("EXPLAIN MATCH (n) RETURN n");
    assert!(result.contains("ScanVertices"));
    assert!(result.contains("Project"));
}

#[test]
fn test_explain_analyze_actual_rows() {
    // 插入100行数据
    execute("INSERT VERTEX Person(name) VALUES 1..100:(\"test\")");
    
    let result = execute("EXPLAIN ANALYZE MATCH (p:Person) RETURN p");
    
    // 验证实际行数等于插入的行数
    assert!(result.contains("actual rows: 100"));
}
```

### 6.2 性能测试

```rust
#[test]
fn test_profile_overhead() {
    let query = "MATCH (p:Person)-[:KNOWS]->(f:Person) RETURN f";
    
    // 无统计的执行时间
    let time_without_stats = benchmark(|| execute(query));
    
    // 有统计的执行时间
    let time_with_stats = benchmark(|| execute(&format!("PROFILE {}", query)));
    
    // 统计开销应小于10%
    let overhead = (time_with_stats - time_without_stats) / time_without_stats;
    assert!(overhead < 0.10);
}
```

---

## 7. 参考资源

### 7.1 官方文档

- [PostgreSQL EXPLAIN文档](https://www.postgresql.org/docs/current/sql-explain.html)
- [PostgreSQL EXPLAIN使用指南](https://www.postgresql.org/docs/current/using-explain.html)
- [MySQL EXPLAIN文档](https://dev.mysql.com/doc/refman/8.0/en/explain.html)
- [SQL Server执行计划文档](https://docs.microsoft.com/en-us/sql/relational-databases/performance/execution-plans)
- [Oracle执行计划文档](https://docs.oracle.com/en/database/oracle/oracle-database/19/tgsql/generating-and-displaying-execution-plans.html)

### 7.2 学术论文

- "Efficient Query Profiling for Database Applications" - VLDB 2014
- "Automatic Performance Diagnosis in Database Systems" - SIGMOD 2016
- "Query Plan Visualization and Analysis" - CIDR 2018

### 7.3 开源实现

- PostgreSQL: `src/backend/commands/explain.c`
- MySQL: `sql/opt_explain.cc`
- DuckDB: `src/main/query_profiler.cpp`

---

## 8. 总结

通过研究PostgreSQL等成熟数据库的实现，我们可以得出以下关键结论：

1. **Instrumentation是核心**: 在执行器中植入统计收集点是实现Explain/Profile的基础

2. **预估vs实际对比**: 显示优化器预估和实际执行数据的差异是诊断性能问题的关键

3. **多维度统计**: 时间、行数、内存、I/O等多维度统计提供全面的执行分析

4. **精度与开销权衡**: 需要根据使用场景选择合适的统计精度，平衡信息丰富度和性能开销

5. **灵活的输出格式**: 支持Table、Dot、JSON等多种格式，适应不同使用场景

这些经验为本项目的Explain/Profile功能实现提供了重要的参考和指导。
