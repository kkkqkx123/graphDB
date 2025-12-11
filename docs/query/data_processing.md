# 数据处理模块分析与实现方案

## 1. 模块概览

数据处理模块（`src\query\executor\data_processing`）是图查询引擎的核心组件之一，负责处理查询中间结果的转换、过滤、聚合等操作。该模块通过一系列执行器（Executor）实现不同的数据处理功能。

### 1.1 当前实现功能

当前实现包括以下主要功能：

- **基础数据处理**：过滤（Filter）、去重（Dedup）、采样（Sample）
- **图遍历操作**：扩展（Expand）、遍历（Traverse）、最短路径（ShortestPath）
- **集合操作**：并集（Union）、交集（Intersect）、差集（Minus）
- **连接操作**：内连接（InnerJoin）、左连接（LeftJoin）、右连接（RightJoin）、全外连接（FullOuterJoin）、笛卡尔积（CrossJoin）
- **数据转换**：赋值（Assign）、展开（Unwind）、模式匹配（PatternApply）
- **控制流**：循环（Loop）
- **聚合操作**：分组（GroupBy）、聚合（Aggregate）、条件过滤（Having）
- **排序操作**：排序（Sort）
- **分页操作**：限制（Limit）

## 2. 与 NebulaGraph 的对比分析

### 2.1 完整性分析

当前实现已包含 NebulaGraph 数据处理的大部分核心功能，实现了之前缺失的关键功能：

#### 已实现功能：
- 基础过滤操作（WHERE 子句）
- 数据去重
- 不同策略的采样
- 图遍历算法
- 基础集合操作
- JOIN 操作（内连接、左连接、右连接、全外连接）
- 数据转换操作
- 循环控制
- 聚合操作（GROUP BY、HAVING、聚合函数）
- 排序操作（ORDER BY）
- 分页操作（LIMIT、OFFSET）

#### 部分实现功能：
- 路径匹配（仅基本模式，缺少高级路径匹配）

#### 新增已实现功能：
- 聚合操作（GROUP BY、HAVING、聚合函数）
- 排序操作（ORDER BY）
- 分页操作（LIMIT、OFFSET）
- 右连接和全外连接

#### 仍缺失功能：
- 窗口函数
- 递归 CTE
- 高级图算法（PageRank、社区检测等）

### 2.2 性能优化对比

当前实现在性能优化方面：

**已实现**：
- 并行处理（JOIN 操作的并行化）
- 内存限制（去重执行器的内存控制）
- 哈希表优化（JOIN 操作）
- 排序操作的内存管理及溢出机制

**缺失**：
- 向量化执行
- 更多 JOIN 算法（Sort-Merge Join、Index Join）
- 磁盘溢出（Spill-to-disk）功能
- 查询执行统计和监控

## 3. 新增实现功能详细说明

### 3.1 聚合操作框架

**已实现组件**：
- `GroupByExecutor` - GROUP BY 操作
- `AggregateExecutor` - SUM、COUNT、AVG、MIN、MAX 等聚合函数
- `HavingExecutor` - HAVING 子句过滤

**实现原理**：
```rust
// 示例：GroupByExecutor 结构
pub struct GroupByExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    group_keys: Vec<Expression>,      // 分组键
    aggregate_exprs: Vec<AggregateExpression>, // 聚合表达式
    input_executor: Option<Box<dyn Executor<S>>>,
    // 内部分组哈希表
    group_table: HashMap<List, AggregateState>,
}

pub struct AggregateState {
    counts: Vec<i64>,
    sums: Vec<f64>,
    mins: Vec<Value>,
    maxs: Vec<Value>,
    // ... 其他聚合状态
}
```

**实现特点**：
- 支持多键分组
- 内存限制控制
- 并行处理分组操作
- 多种聚合函数支持

### 3.2 排序与分页

**已实现组件**：
- `SortExecutor` - ORDER BY 操作
- `LimitExecutor` - LIMIT 操作
- `SortOrder` - 排序方向（升序、降序）

**实现原理**：
```rust
// 示例：SortExecutor 结构
pub struct SortExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    sort_keys: Vec<(Expression, SortOrder)>, // 排序键和顺序
    limit: Option<usize>,                   // 限制数量
    input_executor: Option<Box<dyn Executor<S>>>,
    // 内存限制和磁盘溢出处理
    memory_limit: usize,
    use_disk: bool,
}

pub enum SortOrder {
    Asc,
    Desc,
}
```

**实现特点**：
- 支持多字段排序
- 内存限制管理
- TOP-N 优化排序
- 灵活的分页控制

### 3.3 完整 JOIN 操作

**已实现组件**：
- `RightJoinExecutor` - 右外连接
- `FullOuterJoinExecutor` - 全外连接

**实现原理**：
```rust
// 示例：RightJoinExecutor 结构
pub struct RightJoinExecutor<S: StorageEngine> {
    base: BaseJoinExecutor<S>,
    // 右表构建哈希表，左表探测
    // 右表列数用于填充 NULL
}
```

**实现特点**：
- 完整的四种 JOIN 类型支持
- 高效的哈希连接算法
- 正确的 NULL 填充机制

### 3.4 聚合函数支持

新增了多种聚合函数：

- COUNT - 计算元素数量
- SUM - 数值求和
- AVG - 平均值计算
- MIN - 最小值
- MAX - 最大值
- COLLECT - 收集所有值到列表

## 4. 实现详情

### 4.1 聚合操作实现

**GroupByExecutor 实现**：
1. 读取输入数据
2. 提取分组键值
3. 使用哈希表对数据进行分组
4. 为每组计算聚合状态
5. 输出分组结果

**关键设计考虑**：
- 支持多键分组
- 内存限制和磁盘溢出
- 并行处理分组操作

### 4.2 排序与分页实现

**SortExecutor 实现**：
1. 评估排序键
2. 根据内存限制选择排序策略
3. 内存充足时使用标准排序
4. 内存不足时使用外部排序（磁盘溢出）
5. 实现 LIMIT 优化（Top-N 排序）

**关键设计考虑**：
- 内存管理
- 磁盘溢出机制
- 排序算法优化

### 4.3 完整 JOIN 实现

**RightJoinExecutor 实现**：
1. 将左表构建为哈希表
2. 探测右表并查找匹配
3. 对未匹配的右表行填充 NULL

**FullOuterJoinExecutor 实现**：
1. 构建左右两表的哈希表
2. 实现双向探测和匹配
3. 处理所有未匹配的行

## 5. 实现优先级回顾

### 5.1 已完成的高优先级功能
1. **聚合操作框架** - GROUP BY、聚合函数、HAVING（✓ 已实现）
2. **排序与分页** - ORDER BY、LIMIT（✓ 已实现）
3. **右连接和全外连接** - 完善 JOIN 功能（✓ 已实现）

### 5.2 仍待实现的中低优先级功能
1. **窗口函数** - ROW_NUMBER、RANK、DENSE_RANK 等
2. **高级图算法** - PageRank、路径算法
3. **性能优化** - 向量化执行、JOIN 优化
4. **递归 CTE** - 递归查询
5. **地理空间函数** - 空间数据处理
6. **监控统计** - 查询执行监控

## 6. 实现注意事项

### 6.1 架构一致性
- 保持与现有执行器模式的一致性
- 遵循现有的错误处理模式
- 保持异步执行接口

### 6.2 性能考虑
- 内存使用优化
- 并行处理支持
- 磁盘溢出机制

### 6.3 测试覆盖
- 单元测试覆盖所有功能
- 集成测试验证复杂查询
- 性能测试确保优化效果

## 7. 总结

经过本次更新，数据处理模块已成功实现了之前缺失的关键功能，包括聚合操作框架、排序和分页操作以及完整的JOIN操作。当前模块已基本实现了NebulaGraph的核心数据处理功能，能够支持复杂的SQL-like查询操作。

目前仍待实现的功能主要包括窗口函数、递归CTE和高级图算法等高级功能，这些功能可根据业务需求按优先级逐步添加。整个实现过程保持了与现有架构的一致性，并充分考虑了性能优化和测试覆盖。