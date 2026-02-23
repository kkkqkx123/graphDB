# PostgreSQL 查询优化器设计参考

本文档基于 PostgreSQL 16 官方文档，详细介绍 PostgreSQL 查询优化器的架构设计、代价模型和优化策略。

---

## 一、优化器架构概述

### 1.1 Planner/Optimizer 职责

PostgreSQL 的查询优化器（planner/optimizer）负责创建最优执行计划。一个给定的 SQL 查询可以以多种不同的方式执行，每种方式产生相同的结果集。优化器的任务是检查这些可能的执行计划，最终选择预期运行最快的计划。

```
查询处理流程：
SQL → Parser → Rewrite → Planner/Optimizer → Executor
                ↓
            统计信息 (pg_statistic/pg_class)
```

### 1.2 Path 与 Plan 的区别

优化器的搜索过程实际上使用称为 **Path** 的数据结构：
- **Path**：精简的计划表示，仅包含优化器决策所需的信息
- **Plan**：完整的计划树，包含执行器运行所需的全部细节

优化器首先确定最便宜的 Path，然后构建完整的 Plan Tree 传递给执行器。

### 1.3 优化器类型

PostgreSQL 使用两种优化器：

#### 标准优化器（近穷举搜索）
- 基于 IBM System R 数据库引入的算法
- 对连接策略空间进行近穷举搜索
- 产生接近最优的连接顺序
- 适用于大多数常规查询

#### 遗传查询优化器（GEQO）
- 当连接数超过阈值（`geqo_threshold`，默认 12）时启用
- 使用遗传算法解决连接排序问题
- 适用于涉及大量表连接的复杂查询
- 配置参数：
  ```sql
  SET geqo = on;                    -- 启用 GEQO
  SET geqo_threshold = 12;          -- 触发阈值
  SET geqo_effort = 5;              -- 优化努力程度 (1-10)
  SET geqo_pool_size = 100;         -- 种群大小
  SET geqo_generations = 100;       -- 迭代代数
  SET geqo_selection_bias = 2.00;   -- 选择偏差
  SET geqo_seed = 0.5;              -- 随机种子
  ```

---

## 二、核心代价参数

### 2.1 磁盘 I/O 代价

| 参数名 | 默认值 | 说明 |
|--------|--------|------|
| `seq_page_cost` | 1.0 | 顺序读取一个磁盘页面的成本 |
| `random_page_cost` | 4.0 | 随机读取一个磁盘页面的成本 |

**配置示例**：
```sql
ALTER SYSTEM SET seq_page_cost = 1.0;
ALTER SYSTEM SET random_page_cost = 4.0;
```

**SSD 环境调优**：
- 在 SSD 上，随机读取和顺序读取性能接近
- 建议降低 `random_page_cost`（如设置为 1.1 或 1.2）

### 2.2 CPU 处理代价

| 参数名 | 默认值 | 说明 |
|--------|--------|------|
| `cpu_tuple_cost` | 0.01 | 处理每一行数据的 CPU 成本 |
| `cpu_index_tuple_cost` | 0.005 | 处理每个索引项的 CPU 成本 |
| `cpu_operator_cost` | 0.0025 | 执行每个操作符或函数的 CPU 成本 |

### 2.3 缓存相关参数

| 参数名 | 说明 |
|--------|------|
| `effective_cache_size` | 优化器估计的缓存大小，影响索引使用决策 |

```sql
SET effective_cache_size = '1GB';
```

---

## 三、统计信息系统

### 3.1 统计信息收集

**ANALYZE 命令**：
```sql
-- 分析所有表
ANALYZE;

-- 分析特定表
ANALYZE table_name;

-- 分析特定列
ANALYZE table_name(column1, column2);
```

ANALYZE 收集表的统计信息并存储在 `pg_statistic` 系统目录中，查询规划器使用这些统计信息帮助确定最有效的执行计划。

### 3.2 表级统计信息（pg_class）

```sql
SELECT relname, relkind, reltuples, relpages
FROM pg_class
WHERE relname LIKE 'tenk1%';
```

- **relname**：对象名称
- **relkind**：对象类型（r=表，i=索引）
- **reltuples**：表的估计行数
- **relpages**：表占用的磁盘页数

**注意**：
- `reltuples` 和 `relpages` 不会实时更新
- 由 `VACUUM`、`ANALYZE` 和某些 DDL 命令（如 `CREATE INDEX`）更新
- 规划器会缩放这些值以匹配当前物理表大小

### 3.3 列级统计信息（pg_stats）

```sql
SELECT 
    schemaname,
    tablename,
    attname,
    null_frac,
    n_distinct,
    most_common_vals,
    most_common_freqs,
    histogram_bounds,
    correlation
FROM pg_stats
WHERE schemaname = 'your_schema' AND tablename = 'your_table';
```

**核心字段**：

| 字段名 | 说明 |
|--------|------|
| `null_frac` | 该列中空值的比例 |
| `n_distinct` | 不同值的数量（负数表示比例） |
| `most_common_vals` | 最常见值列表（MCV） |
| `most_common_freqs` | 对应 MCV 的出现频率 |
| `histogram_bounds` | 直方图边界值 |
| `correlation` | 列值与物理存储顺序的相关性（-1.0 ~ 1.0） |

**注意**：
- `pg_statistic` 只能由超级用户读取
- `pg_stats` 对所有用户可读，但只显示用户有权限读取的表

---

## 四、选择性估计

### 4.1 等值条件选择性

#### 使用 MCV（最常见值）

如果查询值在 MCV 列表中，直接使用对应的频率作为选择性。

**示例**：
```sql
-- 查询
EXPLAIN SELECT * FROM tenk1 WHERE stringu1 = 'CRAAAA';

-- 查看统计信息
SELECT null_frac, n_distinct, most_common_vals, most_common_freqs 
FROM pg_stats
WHERE tablename='tenk1' AND attname='stringu1';
```

**计算**：
```
selectivity = mcf[3] = 0.003
rows = 10000 * 0.003 = 30
```

#### 非 MCV 值

使用均匀分布假设：
```
selectivity = (1 - sum(mcv_freqs)) / (num_distinct - num_mcv)
```

**示例计算**：
```
selectivity = (1 - (0.00333333 + 0.003 + ... + 0.003)) / (676 - 10)
            = 0.0014559
```

### 4.2 范围条件选择性

使用直方图进行估计：

1. 确定查询值落在哪个直方图桶中
2. 计算该桶内的比例
3. 累加前面所有完整桶的比例

```
selectivity = (前面完整桶数 + 当前桶内比例) / 总桶数
```

**注意**：
- 直方图不包含 MCV 部分的数据
- 先计算 MCV 部分的选择性，再用直方图计算非 MCV 部分，最后合并

### 4.3 连接选择性

**等值连接**：
```
selectivity = 1 / max(左表不同值数, 右表不同值数)
```

**考虑空值**：
```
selectivity = (1 - 左表空值率) × (1 - 右表空值率) × 基础选择性
```

### 4.4 多列相关性统计

**创建多列统计**：
```sql
CREATE TABLE t2 (
    a   int,
    b   int
);

INSERT INTO t2 SELECT mod(i,100), mod(i,100)
FROM generate_series(1,1000000) s(i);

-- 创建 MCV 统计
CREATE STATISTICS s2 (mcv) ON a, b FROM t2;

ANALYZE t2;
```

多列 MCV 统计可以改进相关列的查询估计：
```sql
-- 有效组合（在 MCV 中找到）
EXPLAIN ANALYZE SELECT * FROM t2 WHERE (a = 1) AND (b = 1);

-- 无效组合（不在 MCV 中）
EXPLAIN ANALYZE SELECT * FROM t2 WHERE (a = 1) AND (b = 2);
```

---

## 五、代价计算原理

### 5.1 顺序扫描代价

```
总代价 = 磁盘 I/O 代价 + CPU 处理代价
       = (页面数 × seq_page_cost) + (行数 × cpu_tuple_cost)
```

### 5.2 索引扫描代价

```
总代价 = 索引访问代价 + 回表代价

索引访问代价 = (索引页数 × seq_page_cost)  -- 顺序读取索引页
             + (索引行数 × cpu_index_tuple_cost)
             + 条件评估代价

回表代价 = (选择性 × 表行数 × random_page_cost)
         + (选择性 × 表行数 × cpu_tuple_cost)
```

**C 代码实现**：
```c
/*
 * 通用假设：索引页按顺序读取，每页成本为 seq_page_cost
 * 对每行索引数据评估索引条件
 * 所有成本在扫描过程中逐步支付
 */
cost_qual_eval(&index_qual_cost, path->indexquals, root);
*indexStartupCost = index_qual_cost.startup;
*indexTotalCost = seq_page_cost * numIndexPages +
    (cpu_index_tuple_cost + index_qual_cost.per_tuple) * numIndexTuples;
```

### 5.3 连接操作代价

#### 嵌套循环连接（Nested Loop Join）

```
代价 = 外表扫描代价 + (外表行数 × 内表扫描代价)
```

**特点**：
- 内表侧总是非并行计划
- 如果内表是索引扫描，效率较高
- 外表元组在协作进程间分配

**示例**：
```sql
EXPLAIN SELECT *
FROM tenk1 t1, tenk2 t2
WHERE t1.unique1 < 10 AND t1.unique2 = t2.unique2;
-- 使用 Bitmap Heap Scan + Index Scan 的嵌套循环
```

#### 哈希连接（Hash Join）

```
代价 = 构建哈希表代价 + 探测代价
     = 左输入代价 + 右输入代价 + 哈希构建开销
```

**特点**：
- 非并行哈希连接：每个进程构建相同的哈希表副本
- 并行哈希连接：协作进程共享哈希表构建工作

**示例**：
```sql
EXPLAIN SELECT *
FROM tenk1 t1, tenk2 t2
WHERE t1.unique1 < 100 AND t1.unique2 = t2.unique2;
-- 使用 Hash Join
```

#### 归并连接（Merge Join）

```
代价 = 左输入排序代价 + 右输入排序代价 + 归并代价
```

**特点**：
- 内表侧总是非并行计划
- 可能需要排序，工作和数据在每个进程中重复

---

## 六、连接优化策略

### 6.1 连接方法选择

PostgreSQL 支持三种连接方法：

1. **嵌套循环连接（Nested Loop）**
   - 对内表进行迭代，对外表的每一行执行一次
   - 适用于小表连接或大表有索引的情况

2. **哈希连接（Hash Join）**
   - 对一个表进行哈希，用另一个表探测
   - 适用于中等大小表连接，选择性差异大

3. **归并连接（Merge Join）**
   - 对两个表在连接属性上排序，然后合并
   - 适用于已排序数据或需要排序输出的情况

### 6.2 连接顺序优化

**穷举搜索**：
- 标准优化器对连接策略空间进行近穷举搜索
- 产生接近最优的连接顺序
- 时间复杂度随连接数指数增长

**遗传算法（GEQO）**：
- 当连接数超过 `geqo_threshold` 时启用
- 使用启发式搜索算法
- 在合理时间内找到合理（不一定最优）的计划

---

## 七、索引成本估算

### 7.1 索引成本估算函数

索引成本估算函数必须用 C 编写（不能用 SQL 或过程语言），因为它们需要访问规划器/优化器的内部数据结构。

### 7.2 核心估算参数

| 参数 | 说明 |
|------|------|
| `indexSelectivity` | 预计检索的父表行比例 |
| `indexCorrelation` | 索引顺序与表顺序的相关性（-1.0 ~ 1.0） |
| `indexPages` | 叶子页数，用于估计并行索引扫描的工作进程数 |
| `loop_count` | 当大于 1 时，返回单次扫描的平均值 |

### 7.3 成本计算要素

索引访问成本应使用 `src/backend/optimizer/path/costsize.c` 中的参数：
- 顺序磁盘块获取成本：`seq_page_cost`
- 非顺序获取成本：`random_page_cost`
- 处理一个索引行的成本：`cpu_index_tuple_cost`
- 比较操作符成本：`cpu_operator_cost` 的适当倍数

**注意**：
- 访问成本包括扫描索引本身的所有磁盘和 CPU 成本
- 不包括检索或处理父表行的成本
- 启动成本是在获取第一行之前必须支出的总扫描成本部分

---

## 八、对 GraphDB 的启示

### 8.1 架构设计建议

1. **分层代价模型**
   - 将 I/O 和 CPU 代价分离
   - 针对不同硬件环境可调优

2. **丰富的统计信息**
   - MCV 和直方图结合
   - 兼顾常见值和分布形状

3. **选择性估计的统一框架**
   - 不同类型条件使用一致的估计方法
   - 支持多列相关性统计

4. **统计信息的自动维护**
   - 减少人工干预
   - 保持统计信息新鲜度

5. **可扩展的设计**
   - 代价参数可配置
   - 适应不同部署场景

### 8.2 实现要点

| 组件 | 建议 |
|------|------|
| 代价参数 | 参考 PostgreSQL 的 `seq_page_cost`、`random_page_cost` 等 |
| 统计信息 | 实现类似 `pg_class` 和 `pg_stats` 的统计体系 |
| 选择性估计 | 实现 MCV + 直方图的混合策略 |
| 连接优化 | 支持多种连接算法，考虑遗传优化器处理复杂查询 |
| 索引选择 | 基于成本估算选择最优索引 |

---

## 参考文档

- [PostgreSQL 16 Documentation - Planner/Optimizer](https://www.postgresql.org/docs/16/planner-optimizer.html)
- [PostgreSQL 16 Documentation - Statistics Used by the Planner](https://www.postgresql.org/docs/16/planner-stats.html)
- [PostgreSQL 16 Documentation - Row Estimation Examples](https://www.postgresql.org/docs/16/row-estimation-examples.html)
- [PostgreSQL 16 Documentation - Index Cost Estimation](https://www.postgresql.org/docs/16/index-cost-estimation.html)
- [PostgreSQL 16 Documentation - Genetic Query Optimizer](https://www.postgresql.org/docs/16/geqo.html)
