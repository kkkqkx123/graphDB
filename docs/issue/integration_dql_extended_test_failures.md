# Integration DQL Extended 测试失败分析报告

## 概述

在运行 `tests\integration_dql_extended.rs` 测试文件时，发现 2 个测试用例失败。本文档分析失败原因并提供修复建议。

## 测试环境

- 测试文件: `tests\integration_dql_extended.rs`
- 通过测试: 19 个
- 失败测试: 2 个

## 失败的测试用例

### 1. test_complex_social_network_query

**测试描述**: 测试多跳 MATCH 查询，查找朋友的朋友

**测试数据**:
```
顶点:
  1: Alice (name='Alice', age=30, city='NYC')
  2: Bob (name='Bob', age=25, city='LA')
  3: Charlie (name='Charlie', age=35, city='NYC')
  4: David (name='David', age=28, city='LA')

边:
  1 -> 2 (Alice -> Bob)
  1 -> 3 (Alice -> Charlie)
  2 -> 4 (Bob -> David)
  3 -> 4 (Charlie -> David)
```

**查询**:
```sql
MATCH (a:Person)-[:KNOWS]->(b:Person)-[:KNOWS]->(c:Person)
WHERE a.name == 'Alice' AND c.city == 'LA'
RETURN c.name, c.age
```

**预期结果**: 1 行 (David, 28)

**实际结果**: 8 行，所有行都是 (Bob, 25)

**问题分析**:

1. **变量绑定错误**: 变量 `c` 实际上指向的是中间节点 `b`，而不是第二个边的目标节点
2. **WHERE 子句过滤失败**: 条件 `c.city == 'LA'` 没有被正确应用
3. **重复结果**: 返回了 8 行重复的结果，说明连接操作产生了笛卡尔积

**根本原因**:

在 `match_statement_planner.rs` 的 `plan_path_pattern` 方法中，处理多跳查询时：

1. 第一个节点 `(a)` 扫描所有 Person 顶点
2. 边 `[:KNOWS]` 使用 `ExpandAll` 扩展，输出列 `["src", "edge", "dst"]`
3. 第二个节点 `(b)` 扫描所有 Person 顶点，然后与 `dst` 列连接
4. 第二个边 `[:KNOWS]` 再次扩展
5. 第三个节点 `(c)` 扫描所有 Person 顶点，然后与 `dst` 列连接

问题在于每次遇到节点时都扫描所有顶点，而不是使用边扩展的结果。此外，变量绑定逻辑有误，导致 `c` 指向了错误的节点。

**修复建议**:

1. 修改 `plan_path_pattern` 方法，对于中间节点不扫描所有顶点，而是直接使用边扩展的结果
2. 修复变量绑定逻辑，确保变量正确绑定到对应的节点
3. 确保 WHERE 子句的过滤条件正确应用到所有相关列

**相关代码**:
- `src\query\planning\statements\match_statement_planner.rs`
- `src\query\executor\data_processing\join\inner_join.rs`

---

### 2. test_aggregation_query

**测试描述**: 测试聚合查询，按类别统计产品数量

**测试数据**:
```
顶点:
  1: Laptop (category='Electronics', price=999.99)
  2: Mouse (category='Electronics', price=29.99)
  3: Keyboard (category='Electronics', price=79.99)
  4: Desk (category='Furniture', price=299.99)
```

**查询**:
```sql
MATCH (p:Product)
RETURN p.category, count(*) AS count
ORDER BY count DESC
```

**预期结果**: 2 行
- (Electronics, 3)
- (Furniture, 1)

**实际结果**: 4 行
- (Electronics, 1) x 3
- (Furniture, 1) x 1

**问题分析**:

1. **GROUP BY 未实现**: 查询没有按 `p.category` 分组
2. **聚合函数错误**: `count(*)` 返回了每行 1，而不是分组后的计数
3. **结果未合并**: 相同类别的行没有被合并成一行

**根本原因**:

在 `match_statement_planner.rs` 中：

1. `plan_project` 方法只创建了 `ProjectNode`，没有处理聚合函数
2. 没有实现 `GROUP BY` 子句的处理逻辑
3. 聚合函数 `count(*)` 被当作普通表达式处理，没有触发聚合逻辑

**修复建议**:

1. 实现 `GROUP BY` 子句的处理逻辑
2. 修改 `plan_project` 方法，识别聚合函数并创建相应的聚合节点
3. 实现聚合执行器，支持 `count`, `sum`, `avg`, `min`, `max` 等聚合函数
4. 在查询计划中添加分组操作节点

**相关代码**:
- `src\query\planning\statements\match_statement_planner.rs`
- `src\query\executor\result_processing\projection.rs`

---

## 修复优先级

| 问题 | 优先级 | 复杂度 | 影响范围 |
|------|--------|--------|----------|
| test_complex_social_network_query | 高 | 高 | MATCH 多跳查询 |
| test_aggregation_query | 中 | 高 | 聚合查询 |

## 建议修复顺序

1. **先修复 test_complex_social_network_query**
   - 这是基础功能，影响 MATCH 查询的核心逻辑
   - 修复后可以使多跳查询正常工作

2. **后修复 test_aggregation_query**
   - 这是高级功能，需要实现完整的聚合框架
   - 涉及 GROUP BY、聚合函数、分组执行器等

## 相关文件

### 规划器
- `src\query\planning\statements\match_statement_planner.rs`

### 执行器
- `src\query\executor\data_processing\join\inner_join.rs`
- `src\query\executor\result_processing\projection.rs`
- `src\query\executor\result_processing\filter.rs`

### 测试
- `tests\integration_dql_extended.rs`
- `tests\common\test_scenario.rs`

## 备注

这两个问题都涉及复杂的查询执行逻辑，需要对查询引擎有较深入的理解。建议在修复前仔细阅读相关代码，并考虑添加更多的单元测试来验证修复效果。
