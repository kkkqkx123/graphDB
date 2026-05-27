# 当前问题与待执行任务汇总

## 一、已修复的问题

### 1. Schema Manager 传播问题 (EXPLAIN with LOOKUP)
- **状态**: 已修复
- **修复内容**: 为9个验证器添加了 `set_schema_manager` 方法，并更新了 Validator enum 以委托给它们
- **相关文件**:
  - `src/query/validator/statements/*.rs`
  - `src/query/validator/validator_enum.rs`

### 2. Vertex ID 类型不匹配问题
- **状态**: 已修复
- **修复内容**: 修改了 insert_edges_validator、delete_validator、fetch_vertices_validator、update_validator，使其从 schema manager 动态获取 vid_type，而不是硬编码为 String
- **相关文件**:
  - `src/query/validator/statements/insert_edges_validator.rs`
  - `src/query/validator/statements/delete_validator.rs`
  - `src/query/validator/statements/fetch_vertices_validator.rs`
  - `src/query/validator/statements/update_validator.rs`

### 3. GO Traversal 变量绑定问题 (UndefinedVariable)
- **状态**: 部分修复
- **修复内容**: 
  - 在 ProjectExecutor 中添加了 GO 查询特殊变量的绑定 ($$, $^, target, edge, 边类型名)
  - 在 FilterExecutor 中添加了 GO 查询特殊变量的绑定
- **相关文件**:
  - `src/query/executor/relational_algebra/projection.rs`
  - `src/query/executor/relational_algebra/selection/filter.rs`

---

## 二、当前遇到的阻塞问题

### 问题: FilterExecutor 未被包含在执行链中

#### 现象
- `ProjectNode::new` 的输入是 `Filter` 节点
- 但 `ProjectExecutor` 的输入执行器是 `ExpandAllExecutor`，而不是 `FilterExecutor`
- 导致 WHERE 子句没有被执行，查询返回了未过滤的结果

#### 调试信息
```
[ProjectNode::new] Created ProjectNode, input type: Some("Filter")
[build_executor_chain] Building executor for node: "Project"
[build_executor_chain] Node has 1 children
[build_executor_chain] Child 0: "ExpandAll"  <-- 应该是 "Filter"
```

#### 根因分析
问题出在 `ProjectNode` 的 `children()` 方法返回了 `ExpandAll` 节点，而不是 `Filter` 节点。

`ProjectNode` 使用 `define_plan_node_with_deps!` 宏定义，`SingleInputNode::input()` 方法返回 `self.input.as_ref()`。

`ProjectNode::new` 设置了 `input: Some(Box::new(input.clone()))`，其中 `input` 是 `filter_node`。

但 `build_executor_chain` 的调试输出显示 `Child 0: "ExpandAll"`，这意味着 `ProjectNode` 的 `input` 字段在创建后被修改为 `ExpandAll` 节点。

#### 可能的原因
1. **优化器重写了计划节点**: `impl_single_input_rewrite!` 宏在优化过程中调用了 `set_input`，可能将 `ProjectNode` 的输入从 `Filter` 改为了 `ExpandAll`
2. **`FilterNode` 被优化器移除**: 优化器可能认为 `FilterNode` 不必要，将其移除并直接连接到 `ExpandAll`
3. **`deps` 和 `input` 字段不一致**: `ProjectNode` 的 `deps` 字段和 `input` 字段可能指向不同的节点

#### 相关代码位置
- `src/query/planning/plan/core/nodes/operation/project_node.rs` - ProjectNode 定义
- `src/query/planning/plan/core/nodes/operation/filter_node.rs` - FilterNode 定义
- `src/query/planning/plan/core/nodes/base/plan_node_children.rs` - children() 方法
- `src/query/optimizer/heuristic/visitor.rs` - 优化器重写逻辑
- `src/query/executor/factory/engine.rs` - build_executor_chain 方法

#### 需要进一步调查
1. 检查优化器是否移除了 `FilterNode`
2. 检查 `ProjectNode` 的 `input` 和 `deps` 字段是否一致
3. 检查 `SingleInputNode::input()` 方法返回的是哪个字段

---

## 三、待修复的问题

### 1. Edge 持久化问题 (最高优先级)
- **状态**: 未开始
- **描述**: INSERT EDGE 报告成功但边不可检索。所有依赖边的操作（MATCH 遍历、GO、FIND PATH、DELETE EDGE、UPDATE EDGE）均返回 0 结果或失败。影响 35+ 个测试。
- **详细分析**: `docs/issue/dml_edge_persistence.md`
- **相关文件**:
  - `crates/graphdb-query/src/query/executor/dml/insert_edge_executor.rs`
  - `crates/graphdb-storage/src/storage/edge/edge_table.rs`
  - `crates/graphdb-storage/src/storage/vertex/vertex_table.rs`
  - `crates/graphdb-storage/src/storage/edge/adjacency_list.rs`

### 2. FIND PATH 与 Graph Traversal 执行问题
- **状态**: 未开始（依赖问题 1 的修复）
- **描述**: 所有 FIND SHORTEST/ALL PATH 和 MATCH/GO 带边遍历的查询返回 0 行。Parser 测试全部通过，执行器始终返回空结果。
- **详细分析**: `docs/issue/dql_find_path_and_traversal.md`

### 3. Optimizer Visitor panic (DELETE with pipe/MATCH)
- **状态**: 未开始
- **描述**: MATCH/PIPE DELETE 执行时在 `heuristic/visitor.rs:185` panic (`visit_default should not be called`)
- **影响**: 10 个 DELETE 测试全部崩溃（而非优雅报错）
- **详细分析**: `docs/issue/optimizer_visitor_panic.md`
- **修复建议**: 临时将 `unreachable!()` 替换为 `self.visit_children(node)` 可避免 panic

### 4. UPSERT 语法不支持
- **状态**: 未开始
- **描述**: `UPSERT VERTEX ... ON DUPLICATE ...` 中 parser 拒绝 `ON` 关键字 — `Unexpected token in expression: On`
- **影响**: 10 个 UPSERT 测试失败
- **注意**: `MERGE VERTEX` (无 ON DUPLICATE) 正常工作

### 5. DELETE EDGE 语法问题
- **状态**: 未开始
- **描述**: `DELETE EDGE 1 -> 2` 中 parser 期望 identifier 但找到 IntegerLiteral(1)
- **影响**: 2 个 parser 测试失败

### 6. UPDATE EDGE 语法问题
- **状态**: 未开始
- **描述**: `UPDATE EDGE ON 1 -> 2` 中 parser 期望 `OF` 关键字
- **影响**: 2 个测试失败

### 7. DDL Constraint 执行失效
- **状态**: 未开始
- **描述**: DEFAULT 值在 INSERT 时不生效，NOT NULL 不拒绝 NULL。Schema 层约束被正确解析和存储但 DML 执行层不强制执行。
- **影响**: 5 个 DDL 测试失败
- **相关测试**: `tests/ddl/constraints.rs`

### 8. Aggregation 类型错误
- **状态**: 未开始
- **描述**: SUM/MIN/MAX 返回 `Value::String("30.0")` 而非数值类型
- **影响**: 3 个 DQL 测试失败
- **示例**:
  - `SUM(price)` → `String("30.0")` (应为 `Double(30.0)` 或 `BigInt`)
  - `MIN(age)` → `String("25")` (应为 `Int(25)`)

### 9. MATCH ORDER BY 变量解析错误
- **状态**: 未开始
- **描述**: `MATCH (v:Person) RETURN v.name ORDER BY v.name` → `UndefinedVariable: v`
- **影响**: 1 个测试失败

### 10. 权限架构缺口
- **状态**: 部分修复
- **描述**: `extract_permission_from_statement` 将 CREATE（含 TAG/EDGE/SPACE）分类为 `Write` 而非 `Schema`。`PermissionManager` 与存储层用户数据不同步（GRANT 通过 pipeline 执行不更新 PermissionManager）。
- **已修复**: USE 语句跳过权限检查（session 级操作）

### 11. DCL 执行未实现
- **状态**: 确认已知限制
- **描述**: CreateUser/DropUser/AlterUser 被 parser 正确解析但 `MaintainPlanner` 拒绝执行：`Statement CreateUser is not supported by MaintainPlanner`
- **影响**: 23 个 DCL 执行测试失败

---

## 四、待执行的任务

### P0 — 核心修复
- [ ] 修复 Edge 持久化问题（详见 `dml_edge_persistence.md`）
- [ ] 修复 Optimizer visitor panic（临时或永久方案）

### P1 — 查询执行
- [ ] 修复 FIND PATH / MATCH / GO 遍历执行（依赖 P0 Edge 修复）
- [ ] 修复 Aggregation 类型转换

### P2 — 语法 & 约束
- [ ] 修复 UPSERT parser `ON` 关键字支持
- [ ] 修复 DELETE EDGE parser identifier 支持
- [ ] 修复 UPDATE EDGE parser `OF` 关键字支持
- [ ] 修复 DDL Constraint 执行（DEFAULT, NOT NULL）

### P3 — 架构
- [ ] 重新设计 `extract_permission_from_statement` 确保 Schema/Write 分类正确
- [ ] 实现 PermissionManager 与存储层用户数据同步
- [ ] 实现 DCL 语句的 pipeline 执行支持

---

## 五、调试代码位置 (需要清理)

以下文件添加了调试输出，修复完成后需要清理：

1. `src/query/planning/plan/core/nodes/operation/filter_node.rs` - FilterNode::new
2. `src/query/planning/plan/core/nodes/operation/project_node.rs` - ProjectNode::new
3. `src/query/executor/factory/engine.rs` - build_executor_chain
4. `src/query/executor/relational_algebra/projection.rs` - ProjectExecutor::execute
5. `src/query/executor/relational_algebra/selection/filter.rs` - FilterExecutor::execute

---

## 六、相关文档

- `docs/issue/e2e_test_report_2025_04_28.md` - E2E 测试报告
- `docs/issue/test_failure_summary_2026_05_27.md` - 集成测试失败汇总（新版）
- `docs/issue/dml_edge_persistence.md` - Edge 持久化问题分析
- `docs/issue/dql_find_path_and_traversal.md` - Graph 遍历执行分析
- `docs/issue/optimizer_visitor_panic.md` - Optimizer Visitor panic 分析
- `docs/issue/code_analysis_go_traversal_variable_binding.md` - GO 遍历变量绑定分析
- `docs/issue/code_analysis_match_variable_binding.md` - MATCH 变量绑定分析
- `docs/issue/code_analysis_optimizer_plan_operators.md` - 优化器计划算子分析
