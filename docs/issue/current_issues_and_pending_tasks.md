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

### 1. MATCH with edge - variable binding in edge expansion
- **状态**: 未开始
- **描述**: MATCH 查询中边扩展的变量绑定问题
- **相关文件**: 待分析

### 2. Optimizer plan operators issue (DescribeVisitor)
- **状态**: 未开始
- **描述**: 优化器计划算子问题
- **相关文件**: 待分析

### 3. 其他失败的测试
- **状态**: 未开始
- **包括**:
  - fetch 测试
  - yield where 测试
  - find path 测试
  - dangling edges 测试

---

## 四、待执行的任务

### 1. 修复当前阻塞问题 (FilterExecutor 不在执行链中)
- [ ] 调查优化器是否移除了 FilterNode
- [ ] 检查 ProjectNode 的 input 和 deps 字段一致性
- [ ] 修复 build_executor_chain 或优化器逻辑

### 2. 修复其他查询问题
- [ ] MATCH with edge - variable binding
- [ ] Optimizer plan operators (DescribeVisitor)
- [ ] 其他失败测试 (fetch, yield where, find path, dangling edges)

### 3. 更新测试
- [ ] 更新单元测试
- [ ] 更新 tests/ 目录下的集成测试

### 4. 验证修复
- [ ] 运行 cargo test 验证所有修复

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
- `docs/issue/code_analysis_go_traversal_variable_binding.md` - GO 遍历变量绑定分析
- `docs/issue/code_analysis_match_variable_binding.md` - MATCH 变量绑定分析
- `docs/issue/code_analysis_optimizer_plan_operators.md` - 优化器计划算子分析
