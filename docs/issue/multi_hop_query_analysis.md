# 多跳 MATCH 查询问题分析报告

## 问题概述

`test_complex_social_network_query` 测试失败，多跳 MATCH 查询返回 6 行而非预期的 1 行。

## 测试场景

```cypher
MATCH (a:Person)-[:KNOWS]->(b:Person)-[:KNOWS]->(c:Person)
WHERE a.name == 'Alice'
RETURN a.name, b.name, c.name
```

**测试数据：**
- Alice (ID 1) -> Bob (ID 2)
- Alice (ID 1) -> Charlie (ID 3)
- Bob (ID 2) -> David (ID 4)
- Charlie (ID 3) -> David (ID 4)

**预期结果：** 1 行 (Alice, Bob, David)
**实际结果：** 6 行

## 问题分析

### 1. 查询计划结构问题

通过调试发现，查询计划存在以下结构：

```
HashInnerJoinNode (b == src)
├── CrossJoinNode
│   ├── ScanVerticesNode (a) - 产生 4 行（所有 Person 顶点）
│   └── ExpandAllNode (a -> b) - 预期 2 行，实际参与笛卡尔积
└── ExpandAllNode (b -> c) - 产生 14 行（异常）
```

### 2. 关键问题点

#### 问题 1：CrossJoinNode 产生笛卡尔积

`cross_join_plans` 方法将第一个节点的扫描结果（4 行）与边扩展结果（2 行）进行交叉连接，产生 8 行的笛卡尔积，而非预期的 2 行。

**原因：**
- `plan_path_pattern` 方法中，第一个节点使用 `ScanVerticesNode` 扫描所有顶点
- 边扩展使用 `ExpandAllNode` 进行路径扩展
- 两者通过 `CrossJoinNode` 连接，产生笛卡尔积

#### 问题 2：中间节点处理逻辑错误

在多跳查询中，中间节点（如 `b`）应该：
1. 不作为独立的扫描操作
2. 使用前一个边扩展的 `dst` 列作为输入

但当前实现中，中间节点可能被当作独立节点处理，导致额外的扫描操作。

#### 问题 3：ExpandAllExecutor 输入处理异常

第二条边扩展产生了 14 行，远超预期的 2 行。可能原因：
- 输入变量绑定错误，获取了错误的输入数据
- `ExpandAllExecutor` 被多次执行
- 缓存未正确清理（已在 `execute` 方法开头添加清理代码）

#### 问题 4：连接操作被执行多次

调试显示 `InnerJoinExecutor::execute` 被调用了 5 次，表明：
- 查询计划中可能存在多余的连接节点
- 或者某些节点被重复执行

### 3. 数据流分析

**预期数据流：**
```
ScanVerticesNode(a) - 1 行（Alice）
    -> ExpandAllNode(a->b) - 2 行（Alice->Bob, Alice->Charlie）
        -> ExpandAllNode(b->c) - 2 行（Bob->David, Charlie->David）
            -> Filter(b.name == 'Bob') - 1 行
```

**实际数据流：**
```
ScanVerticesNode(a) - 4 行（所有 Person）
    -> CrossJoinNode - 8 行（笛卡尔积）
        -> HashInnerJoinNode - 多次执行
            -> 最终结果 6 行
```

## 根本原因

1. **查询计划生成逻辑错误**：`plan_path_pattern` 和 `cross_join_plans` 方法没有正确处理多跳查询中的节点依赖关系
2. **CrossJoin 误用**：应该使用 HashInnerJoin 来连接边扩展结果，而不是 CrossJoin
3. **变量绑定问题**：中间节点的变量没有正确绑定到边扩展的输出列

## 修复建议

### 方案 1：修复 plan_path_pattern 方法

修改 `plan_path_pattern` 方法，确保：
1. 第一个节点扫描所有顶点
2. 后续节点使用前一个边扩展的 `dst` 列作为输入
3. 中间节点不作为独立扫描操作
4. 使用 HashInnerJoin 而非 CrossJoin 连接边扩展结果

### 方案 2：优化 cross_join_plans 方法

对于多跳查询中的第一个边扩展，应该：
1. 直接将 `ScanVerticesNode` 作为 `ExpandAllNode` 的输入
2. 避免使用 `CrossJoinNode` 产生笛卡尔积

### 方案 3：修复 ExpandAllExecutor 输入处理

确保 `ExpandAllExecutor` 正确从 `ExecutionContext` 获取输入：
1. 验证 `input_var` 是否正确设置
2. 验证从 `DataSet` 提取顶点的列索引是否正确
3. 确保缓存清理逻辑正常工作

## 相关文件

- `src/query/planning/statements/match_statement_planner.rs` - 查询计划生成
- `src/query/executor/data_processing/graph_traversal/expand_all.rs` - 边扩展执行器
- `src/query/executor/data_processing/join/inner_join.rs` - 连接操作执行器
- `src/query/executor/factory/executors/plan_executor.rs` - 执行器构建

## 后续工作

1. 重新设计多跳查询的计划生成逻辑
2. 添加更多单元测试验证修复结果
3. 考虑优化查询计划，减少不必要的连接操作
