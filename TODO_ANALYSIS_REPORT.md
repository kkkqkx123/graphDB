# Factory.rs TODO 项目可行性分析报告

## 概述

本报告分析了 `src/query/executor/factory.rs` 文件中的 TODO 项目，评估其是否已经可以实现，以及相关模块的实现完整性。

## TODO 项目分析

### 1. ScanVerticesCreator 中的 TODO (第128行)

```rust
// TODO: 这里需要解析顶点ID和标签过滤条件
// 暂时使用None
(vertex_ids, tag_filter) = (None, None);
```

**可行性评估：可以实现 ✅**

**实现方案：**
- 从 `ScanVertices` 计划节点中提取 `tag_filter` 和 `vertex_filter` 字段
- 使用 `parse_expression_from_string` 函数解析过滤条件字符串为表达式
- 将解析后的表达式传递给 `GetVerticesExecutor`

**所需依赖：**
- `src/query/parser/expressions/expression_converter.rs` 中的 `parse_expression_from_string` 函数已实现
- `ScanVertices` 计划节点已定义 `tag_filter` 和 `vertex_filter` 字段

### 2. ScanEdgesCreator 中的 TODO (第159行)

```rust
// TODO: 这里需要解析边过滤条件
// 暂时使用None
edge_filter = None
```

**可行性评估：可以实现 ✅**

**实现方案：**
- 从 `ScanEdges` 计划节点中提取 `filter` 字段
- 使用 `parse_expression_from_string` 函数解析过滤条件字符串为表达式
- 将解析后的表达式传递给 `GetEdgesExecutor`

**所需依赖：**
- `parse_expression_from_string` 函数已实现
- `ScanEdges` 计划节点已定义 `filter` 字段

### 3. FilterCreator 中的 TODO (第192行)

```rust
// TODO: 这里需要实现表达式解析器
// 暂时使用简单的true表达式
Expression::literal(true)
```

**可行性评估：可以实现 ✅**

**实现方案：**
- 从 `Filter` 计划节点中提取 `condition` 字段
- 使用 `parse_expression_from_string` 函数解析条件字符串为表达式
- 将解析后的表达式传递给 `FilterExecutor`

**所需依赖：**
- `parse_expression_from_string` 函数已实现
- `Filter` 计划节点已定义 `condition` 字段

### 4. ProjectCreator 中的 TODO (第225行)

```rust
// TODO: 这里需要实现表达式解析器
// 暂时根据yield_expr创建简单的投影列
```

**可行性评估：可以实现 ✅**

**实现方案：**
- 从 `Project` 计划节点中提取 `yield_expr` 字段
- 解析投影表达式字符串，支持逗号分隔的多列
- 使用 `parse_expression_from_string` 函数解析每个表达式
- 创建对应的 `ProjectionColumn` 对象

**所需依赖：**
- `parse_expression_from_string` 函数已实现
- `Project` 计划节点已定义 `yield_expr` 字段

### 5. SortCreator 中的 TODO (第301行)

```rust
// TODO: 这里需要解析排序方向，暂时默认为升序
SortKey::new(Expression::variable(item.clone()), SortOrder::Asc)
```

**可行性评估：可以实现 ✅**

**实现方案：**
- 解析排序项字符串，提取排序方向（ASC/DESC）
- 根据方向创建对应的 `SortOrder` 枚举值
- 使用 `parse_expression_from_string` 函数解析排序表达式

**所需依赖：**
- `parse_expression_from_string` 函数已实现
- `Sort` 计划节点已定义 `sort_items` 字段

### 6. AggregateCreator 中的 TODO (第340行)

```rust
// TODO: 这里需要解析分组键和聚合函数
// 暂时使用空列表
(group_keys, agg_funcs) = (vec![], vec![])
```

**可行性评估：可以实现 ✅**

**实现方案：**
- 从 `Aggregate` 计划节点中提取 `group_keys` 和 `agg_exprs` 字段
- 使用 `parse_expression_from_string` 函数解析聚合表达式
- 将解析后的表达式传递给 `AggregateExecutor`

**所需依赖：**
- `parse_expression_from_string` 函数已实现
- `Aggregate` 计划节点已定义 `group_keys` 和 `agg_exprs` 字段

### 7. JoinCreator 中的 TODO (第366行和第373行)

```rust
// TODO: 修复 Join 导入问题
// TODO: 这里需要解析连接条件
```

**可行性评估：部分可以实现 ⚠️**

**问题分析：**
1. Join 计划节点导入问题：当前代码中没有定义通用的 `Join` 计划节点，只有具体的 `HashJoin`、`HashLeftJoin` 等实现
2. 连接条件解析：需要从具体的 Join 节点中提取连接条件

**实现方案：**
1. 创建一个通用的 `Join` 计划节点，或者修改代码以支持具体的 Join 类型
2. 从 Join 节点中提取连接条件参数
3. 使用 `parse_expression_from_string` 函数解析连接条件

**所需依赖：**
- 需要定义或导入适当的 Join 计划节点
- `parse_expression_from_string` 函数已实现

### 8. ExpandCreator 中的 TODO (第426行)

```rust
// TODO: 这里需要解析展开参数
// 暂时使用默认值
(direction, edge_types, vertex_filter) = (EdgeDirection::Both, None, None)
```

**可行性评估：可以实现 ✅**

**实现方案：**
- 从 `Expand` 计划节点中提取 `direction`、`edge_types` 字段
- 解析方向字符串为 `EdgeDirection` 枚举值
- 使用 `parse_expression_from_string` 函数解析顶点过滤条件

**所需依赖：**
- `parse_expression_from_string` 函数已实现
- `Expand` 计划节点已定义相关字段

## 相关模块实现完整性分析

### 1. 表达式解析器模块

**状态：已实现 ✅**

- `src/query/parser/expressions/expression_converter.rs` 提供了 `parse_expression_from_string` 函数
- 支持从字符串解析表达式并转换为 `Expression` 对象
- 支持二元表达式、一元表达式、函数调用、属性访问等多种表达式类型

### 2. 计划节点定义模块

**状态：基本完整 ✅**

- `src/query/planner/plan/operations/` 目录下定义了各种计划节点
- 每个计划节点都包含了必要的字段和方法
- 支持图扫描操作、数据处理操作、排序操作、聚合操作、遍历操作、连接操作等

### 3. 执行器实现模块

**状态：基本完整 ✅**

- `src/query/executor/` 目录下实现了各种执行器
- 支持数据访问、数据处理、结果处理等操作
- 执行器接口定义清晰，易于扩展

### 4. 表达式评估模块

**状态：已实现 ✅**

- `src/graph/expression/evaluator.rs` 提供了表达式评估功能
- 支持各种表达式类型的评估
- 提供了完整的错误处理机制

## 实现优先级建议

### 高优先级（立即实现）

1. **FilterCreator 中的表达式解析**：这是查询过滤的核心功能
2. **ProjectCreator 中的表达式解析**：这是结果投影的核心功能
3. **ScanVerticesCreator 和 ScanEdgesCreator 中的过滤条件解析**：这是数据扫描的基础功能

### 中优先级（近期实现）

1. **SortCreator 中的排序方向解析**：提升排序功能的完整性
2. **AggregateCreator 中的分组键和聚合函数解析**：支持聚合查询
3. **ExpandCreator 中的展开参数解析**：支持图遍历操作

### 低优先级（后续实现）

1. **JoinCreator 中的连接条件解析**：需要先解决 Join 计划节点的导入问题

## 实现建议

1. **统一表达式解析接口**：创建一个统一的表达式解析工具函数，避免在各处重复代码
2. **错误处理增强**：为表达式解析添加更详细的错误信息和恢复机制
3. **性能优化**：考虑缓存解析结果，避免重复解析相同的表达式
4. **测试覆盖**：为每个实现的功能添加单元测试和集成测试

## 结论

大部分 TODO 项目已经可以实现，主要依赖的表达式解析器和计划节点定义都已经实现。唯一需要额外工作的是 JoinCreator 中的连接条件解析，需要先解决 Join 计划节点的导入问题。

建议按照优先级逐步实现这些 TODO 项目，以提升查询执行器的功能完整性。