# MATCH 查询路径遍历问题分析

## 问题概述

当前实现的 MATCH 查询在处理包含边遍历的复杂路径模式时存在问题。基本的节点查询（如 `MATCH (n:Person) RETURN n`）可以正常工作，但涉及边遍历的查询（如 `MATCH (a)-[]->(b) RETURN a, b`）无法正确执行。

## 已修复的问题

### 1. 变量查找失败问题 ✅

**问题描述**：在执行 HashInnerJoin 时，出现 "Undefined variable" 错误，无法找到连接键对应的变量。

**根本原因**：
- 当输入是 `Vertices` 类型时，`check_input_datasets` 方法将列名设置为 `self.col_names`（合并后的列名），而不是实际的列名
- 例如，右输入的列名应该是 `["b"]`，但被设置为 `["a", "e", "b"]`

**修复方案**：
- 修改 `base_join.rs` 中的 `check_input_datasets` 方法，当输入是 `Vertices` 时，使用 `["_vertex"]` 作为默认列名
- 修改 `inner_join.rs` 中的 `execute_single_key_join` 方法，当数据集只有一列 `"_vertex"` 时，根据 key 表达式的变量名设置正确的列名

**相关文件**：
- `src/query/executor/data_processing/join/base_join.rs`
- `src/query/executor/data_processing/join/inner_join.rs`

## 待修复的问题

### 1. 路径遍历逻辑错误 ❌

**问题描述**：对于查询 `MATCH (a:Person)-[:KNOWS]->(b:Person)-[:KNOWS]->(c:Person)`，执行结果为空或不正确。

**根本原因分析**：

当前的执行计划构建逻辑存在以下问题：

1. **ExpandAll 输出处理不当**
   - ExpandAll 返回的是完整路径（包含起点、边、终点）
   - 但执行计划构建逻辑没有从路径中提取终点节点
   - 而是直接将路径与现有计划 CrossJoin，导致数据结构混乱

2. **HashInnerJoin 连接键选择错误**
   - 当前逻辑使用起点节点（如 `a`）作为 hash_key
   - 但实际上应该使用边的终点节点（从路径中提取）作为连接键
   - 这导致 HashInnerJoin 无法正确匹配节点

3. **执行计划结构问题**

以查询 `(a:Person)-[:KNOWS]->(b:Person)-[:KNOWS]->(c:Person)` 为例：

**当前执行计划**：
```
HashInnerJoin(hash_key="b", probe_key="c")
├── CrossJoin
│   ├── HashInnerJoin(hash_key="a", probe_key="b")
│   │   ├── CrossJoin
│   │   │   ├── ScanVertices(a)
│   │   │   └── ExpandAll(e1)  // 返回路径，未提取终点
│   │   └── Filter(ScanVertices(b))
│   │   └── ExpandAll(e2)  // 返回路径，未提取终点
│   └── Filter(ScanVertices(c))
```

**期望执行计划**：
```
HashInnerJoin(hash_key="b", probe_key="c")
├── Project(提取终点节点)
│   └── ExpandAll(e2)  // 从 b 扩展到 c
├── Filter(ScanVertices(c))

// 或者更清晰的结构
HashInnerJoin(hash_key="b_id", probe_key="c_id")
├── Project(提取 e2 的终点)
│   └── ExpandAll(e2)
└── ScanVertices(c)
```

**具体代码位置**：
- `src/query/planning/statements/match_statement_planner.rs`
  - `plan_path_pattern` 方法：处理路径模式的循环
  - `cross_join_plans` 方法：创建 CrossJoin 节点
  - `join_node_plans` 方法：创建 HashInnerJoin 节点

**建议修复方案**：

1. **添加路径提取节点**
   - 创建新的计划节点（如 `PathExtractNode`），用于从路径中提取特定元素（终点节点、边等）
   - 在 ExpandAll 之后添加 PathExtractNode，提取终点节点

2. **修改执行计划构建逻辑**
   - 在 `plan_path_pattern` 中，处理边模式时：
     - 创建 ExpandAll 节点
     - 添加 PathExtractNode 提取终点节点
     - 使用提取的终点节点与下一个节点的 ScanVertices 连接

3. **示例代码结构**：
```rust
// 处理边模式
PathElement::Edge(edge) => {
    let edge_plan = self.plan_pattern_edge(edge, space_id)?;
    
    // 从路径中提取终点节点
    let extract_plan = self.plan_extract_destination(edge_plan)?;
    
    plan = if let Some(existing_root) = plan.root.take() {
        // 使用 HashInnerJoin 连接，使用前一节点和提取的终点
        self.join_node_plans(
            SubPlan::new(Some(existing_root), plan.tail),
            extract_plan,
            prev_node_alias.as_deref().unwrap(),
            &edge.destination_variable, // 需要添加这个字段
            ...
        )?
    } else {
        extract_plan
    };
}
```

### 2. 列名传播问题 ⚠️

**问题描述**：在复杂的执行计划中，列名传播不正确，导致后续操作无法找到正确的变量。

**具体表现**：
- CrossJoin 合并列名时，可能出现重复的列名
- 例如：`["a", "e", "b", "e", "c"]` 中有两个 `"e"`

**根本原因**：
- `cross_join_plans` 方法简单地合并左右子计划的列名
- 没有处理列名冲突的情况

**建议修复方案**：
- 在合并列名时，检查是否有重复的列名
- 如果有重复，添加后缀或前缀来区分
- 或者使用更清晰的列名命名规范

### 3. 结果集结构问题 ⚠️

**问题描述**：查询结果包含嵌套的 DataSet，结构混乱。

**具体表现**：
```
rows: [[DataSet(DataSet { col_names: ["a", "e", "b"], rows: [...] })]]
```

**根本原因**：
- Project 节点或其他结果处理节点没有正确展平结果
- 可能是由于执行计划结构不正确导致的

**建议修复方案**：
- 检查 Project 节点的实现，确保正确展平结果
- 在执行计划构建时，确保每个节点的输出结构正确

## 调试信息

### 当前执行计划的调试输出

对于查询 `MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN b.name`：

```
[InnerJoinExecutor] hash_key: Variable("a"), probe_key: Variable("b")
[InnerJoinExecutor] build_col_names: ["a", "e", "b"], probe_col_names: ["a", "e", "b"]
[InnerJoinExecutor] processing build row: [...]
[InnerJoinExecutor] context created with col_names: ["a", "e", "b"]
```

问题：`hash_key` 是 `Variable("a")`，但 `build_col_names` 是 `["a", "e", "b"]`，这导致上下文查找变量 `a` 时失败。

### ExpandAll 的输出格式

ExpandAll 返回 `ExecutionResult::Values(path_values)`，其中每个路径是：
```
Value::List([src_vertex, edge, dst_vertex])
```

例如：
```
List([
    Vertex(Alice),
    Edge(KNOWS: Alice -> Bob),
    Vertex(Bob)
])
```

### CrossJoin 的列名合并

当前 CrossJoin 的列名合并逻辑：
```rust
let col_names = [
    left_col_names.clone(),
    right_col_names.clone()
].concat();
```

这导致列名重复，例如：`["a", "e", "b", "e", "c"]`

## 测试状态

### 通过的测试 ✅
- `test_match_basic_with_data`：基本的节点查询

### 失败的测试 ❌
- `test_match_with_edge_traversal`：单边遍历查询
- `test_complex_social_network_query`：多跳路径查询

## 修复优先级

1. **高优先级**：路径遍历逻辑错误
   - 这是核心功能，影响所有涉及边遍历的查询
   
2. **中优先级**：列名传播问题
   - 影响查询的正确性和可读性
   
3. **低优先级**：结果集结构问题
   - 主要是结果展示问题，不影响核心功能

## 相关文件

### 执行计划构建
- `src/query/planning/statements/match_statement_planner.rs`
- `src/query/planning/plan/core/nodes/traversal/traversal_node.rs`
- `src/query/planning/plan/core/nodes/join/join_node.rs`

### 执行器
- `src/query/executor/data_processing/join/inner_join.rs`
- `src/query/executor/data_processing/join/base_join.rs`
- `src/query/executor/data_processing/join/cross_join.rs`
- `src/query/executor/data_processing/graph_traversal/expand_all.rs`

### 测试
- `tests/integration_dql_extended.rs`

## 备注

当前的修复解决了变量查找失败的问题，使基本的查询可以正常工作。但路径遍历的核心逻辑问题需要更深入的重构，建议作为后续的主要工作项。

### 关键修改点

1. **base_join.rs** (已修改)
   - `check_input_datasets` 方法：当输入是 `Vertices` 时，使用 `["_vertex"]` 作为默认列名

2. **inner_join.rs** (已修改)
   - `execute_single_key_join` 方法：添加 `get_col_names` 辅助函数，当数据集只有一列 `"_vertex"` 时，根据 key 表达式的变量名设置正确的列名

3. **match_statement_planner.rs** (需要修改)
   - `plan_path_pattern` 方法：需要添加路径提取逻辑
   - `cross_join_plans` 方法：需要修改列名合并逻辑

4. **expand_all.rs** (可能需要修改)
   - 可能需要添加方法来提取路径中的特定元素（终点节点、边等）
