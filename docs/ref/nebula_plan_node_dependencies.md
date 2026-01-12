# NebulaGraph 中 dependencies 对优化器遍历的作用

## 概述

在 NebulaGraph 的执行计划（PlanNode）设计中，`dependencies` 是一个核心属性，用于维护执行计划树的拓扑结构。它对于优化器的遍历和规则应用至关重要。

## 数据结构

```cpp
// NebulaGraph PlanNode.h
class PlanNode {
  // 存储所有依赖节点的指针
  const auto& dependencies() const {
    return dependencies_;
  }
  
  auto& dependencies() {
    return dependencies_;
  }
  
  void addDep(const PlanNode* dep) {
    dependencies_.emplace_back(dep);
  }
  
  size_t numDeps() const {
    return dependencies_.size();
  }
private:
  std::vector<const PlanNode*> dependencies_;
};
```

## 核心作用

### 1. 执行计划树的拓扑维护

**定义上游节点关系**：每个节点的 dependencies 包含所有必须先执行的上游节点。

- 单输入节点（Filter、Project、Sort）：dependencies.size() == 1
- 双输入节点（Join）：dependencies.size() == 2
- 多输入节点（Union）：dependencies 包含所有输入分支

示例：
```
Plan Tree:        Select
                    |
                  Filter (dependencies = [GetVertices])
                    |
              GetVertices (dependencies = [])
```

### 2. 优化器规则的模式匹配

优化器通过遍历 dependencies 来识别可优化的计划片段。

**模式匹配示例** - 合并 Project + GetVertices：

```cpp
// src/graph/optimizer/rule/MergeGetVerticesAndProjectRule.cpp
const OptGroupNode *optProj = matched.dependencies.back().node;

// 检查 Project 的输入是否为 GetVertices
for (auto dep : optProj->dependencies()) {
  if (dep->node()->kind() == PlanNode::Kind::kGetVertices) {
    // 模式匹配成功，应用优化规则
    applyOptimization();
  }
}
```

### 3. 递归遍历和深度优化

许多优化规则需要递归地遍历整个计划树，dependencies 提供了遍历的导航路径。

**深度优化示例** - 向下推送过滤条件：

```cpp
// src/graph/optimizer/rule/PushFilterDownTraverseRule.cpp
// 从 Filter 节点开始
auto filterNode = matched.dependencies[0].node;  // 获取依赖

// 继续遍历其他依赖
for (auto dep : filterNode->dependencies()) {
  // 递归处理下游节点
  processNode(dep);
}
```

### 4. 链式依赖追踪

在某些优化中，需要追踪长链路上的节点关系。

```cpp
// src/graph/validator/SetValidator.cpp
// 一直向下遍历到叶节点
while (node->dependencies()[0]->dependencies().size() > 0UL) {
  node = const_cast<PlanNode*>(node->dependencies()[0]);
}
```

## 关键使用场景

### 场景 1：优化规则应用

```cpp
// 定位可优化的计划片段
OptGroupNode* matchedNode = ...;
const auto& deps = matchedNode->dependencies();  // 获取所有依赖

// 验证模式
if (deps.size() == 1) {
  auto childNode = deps[0].node;
  if (childNode->node()->kind() == expectedKind) {
    // 应用优化规则
  }
}
```

### 场景 2：成本估算

```cpp
// 计算整个计划树的成本
double calculateCost(PlanNode* node) {
  double cost = node->cost();
  for (auto dep : node->dependencies()) {
    cost += calculateCost(dep);
  }
  return cost;
}
```

### 场景 3：计划序列化

```cpp
// src/graph/planner/plan/ExecutionPlan.cpp
// 将执行计划转换为可序列化的格式，需要遍历 dependencies
planNodeDesc->dependencies = std::move(deps);
```

### 场景 4：执行调度

```cpp
// src/graph/scheduler/Scheduler.cpp
// 确定节点执行顺序：只有所有 dependencies 都完成后，才能执行该节点
for (auto dep : currentNode->dependencies()) {
  if (!isCompleted(dep)) {
    continue;  // 等待依赖完成
  }
}
execute(currentNode);
```

## 与 GraphDB 的对比

### NebulaGraph 的设计

- **dependencies 必需**：完整的分布式优化器设计，需要灵活遍历计划树
- **支持复杂优化**：级联过滤下推、连接顺序优化、索引选择优化等
- **成本驱动优化**：需要遍历所有可能的计划变体并比较成本

### GraphDB 的简化设计

在我们的单节点图数据库中：

- **input 字段的作用**：存储单个上游节点的直接引用
- **使用场景**：
  1. **计划树构建**：规划器通过 `add_dependency(dep)` 将新节点的输入连接到前置节点
  2. **计划获取**：`input()` 方法返回直接的上游节点用于遍历
  3. **节点修改**：`remove_dependency(id)` 检查输入节点是否被移除
  4. **计划克隆**：`clone_plan_node()` 时自动复制整个输入树结构
  5. **依赖推导**：`plan_node_enum.rs` 中通过 `node.input()` 推导出该节点的所有依赖

示例：顺序连接两个操作

```rust
// Order By 连接到 Project 上
let order_by_plan = ...;      // root = Order By Node
let project_plan = ...;        // root = Project Node

// 通过 add_input 实现连接
let combined = UnifiedConnector::add_input(&qctx, &order_by_plan, &project_plan, true)?;
// 结果：Order By.input -> Project Node.input -> ... （整个上游树）
```

- **优化需求简单**：暂无分布式优化、复杂的多路径选择
- **性能优先**：减少间接引用，提高克隆效率

**决策**：在 ProjectNode 等简单节点上移除 dependencies，保留结构化的 input 字段。

## 未来考虑

如果 GraphDB 需要支持以下功能，应重新引入 dependencies：

1. **复杂查询优化**
   - 多路径连接顺序优化
   - 基于成本的规则应用

2. **执行计划分析**
   - EXPLAIN 命令的详细计划展示
   - 成本模型验证

3. **分布式扩展**
   - 计划分片和分布式调度
   - 跨节点的依赖管理

## 参考实现

- PlanNode 定义：nebula-3.8.0/src/graph/planner/plan/PlanNode.h:249-273
- 优化规则示例：nebula-3.8.0/src/graph/optimizer/rule/*.cpp
- 调度器：nebula-3.8.0/src/graph/scheduler/Scheduler.cpp:44
