# Explain功能实现分析

## 概述

本文档分析了nebula-graph中explain功能的实现机制，为graphDB项目提供实现参考。explain功能用于生成查询执行计划的描述，帮助用户理解查询的执行过程和优化器生成的执行计划。

## 核心数据结构

### 1. PlanNodeDescription

位置：`nebula-3.8.0/src/common/graph/Response.h`

```cpp
struct PlanNodeDescription {
  std::string name;                                          // 节点名称
  int64_t id{-1};                                           // 节点ID
  std::string outputVar;                                     // 输出变量名
  std::unique_ptr<std::vector<Pair>> description{nullptr};  // 节点描述键值对
  std::unique_ptr<std::vector<ProfilingStats>> profiles{nullptr};  // 性能统计
  std::unique_ptr<PlanNodeBranchInfo> branchInfo{nullptr}; // 分支信息
  std::unique_ptr<std::vector<int64_t>> dependencies{nullptr}; // 依赖节点ID列表

  folly::dynamic toJson() const;  // 转换为JSON格式
};
```

**字段说明：**
- `name`: 节点类型名称（如"Filter"、"Project"、"GetNeighbors"等）
- `id`: 节点的唯一标识符
- `outputVar`: 该节点输出数据的变量名
- `description`: 节点特定的描述信息（如过滤条件、投影列等）
- `profiles`: 执行后的性能统计数据（行数、执行时间等）
- `branchInfo`: 用于Select/Loop等控制流节点的分支信息
- `dependencies`: 依赖的子节点ID列表

### 2. PlanDescription

位置：`nebula-3.8.0/src/common/graph/Response.h`

```cpp
struct PlanDescription {
  std::vector<PlanNodeDescription> planNodeDescs;  // 所有节点的描述
  std::unordered_map<int64_t, int64_t> nodeIndexMap; // 节点ID到索引的映射
  std::string format;                               // 输出格式（如"dot"）
  int32_t optimize_time_in_us{0};                   // 优化耗时

  folly::dynamic toJson() const;  // 转换为JSON格式
};
```

**字段说明：**
- `planNodeDescs`: 执行计划中所有节点的描述列表
- `nodeIndexMap`: 快速查找节点描述的索引映射
- `format`: 输出格式（支持文本、dot图等）
- `optimize_time_in_us`: 查询优化器生成执行计划的耗时

### 3. PlanNodeBranchInfo

```cpp
struct PlanNodeBranchInfo {
  bool isDoBranch;      // 是否为执行分支
  int64_t conditionNodeId;  // 条件节点的ID
};
```

用于Select和Loop节点描述分支结构。

## Explain方法调用流程

### 1. 入口：ExecutionPlan::describe()

位置：`nebula-3.8.0/src/graph/planner/plan/ExecutionPlan.cpp:82-86`

```cpp
void ExecutionPlan::describe(PlanDescription* planDesc) {
  planDescription_ = DCHECK_NOTNULL(planDesc);
  planDescription_->optimize_time_in_us = optimizeTimeInUs_;
  planDescription_->format = explainFormat_;
  makePlanNodeDesc(root_);  // 从根节点开始遍历
}
```

**流程说明：**
1. 设置优化时间和输出格式
2. 从执行计划的根节点开始递归遍历所有节点
3. 调用`makePlanNodeDesc()`生成节点描述

### 2. 核心方法：makePlanNodeDesc()

位置：`nebula-3.8.0/src/graph/planner/plan/ExecutionPlan.cpp:25-50`

```cpp
uint64_t ExecutionPlan::makePlanNodeDesc(const PlanNode* node) {
  // 检查节点是否已处理（避免重复处理）
  auto found = planDescription_->nodeIndexMap.find(node->id());
  if (found != planDescription_->nodeIndexMap.end()) {
    return found->second;
  }

  // 生成节点描述
  size_t planNodeDescPos = planDescription_->planNodeDescs.size();
  planDescription_->nodeIndexMap.emplace(node->id(), planNodeDescPos);
  planDescription_->planNodeDescs.emplace_back(std::move(*node->explain()));
  auto& planNodeDesc = planDescription_->planNodeDescs.back();
  planNodeDesc.profiles = std::make_unique<std::vector<ProfilingStats>>();

  // 处理控制流节点（Select/Loop）
  if (node->kind() == PlanNode::Kind::kSelect) {
    auto select = static_cast<const Select*>(node);
    setPlanNodeDeps(select, &planNodeDesc);
    descBranchInfo(select->then(), true, select->id());
    descBranchInfo(select->otherwise(), false, select->id());
  } else if (node->kind() == PlanNode::Kind::kLoop) {
    auto loop = static_cast<const Loop*>(node);
    setPlanNodeDeps(loop, &planNodeDesc);
    descBranchInfo(loop->body(), true, loop->id());
  }

  // 递归处理所有依赖节点
  for (size_t i = 0; i < node->numDeps(); ++i) {
    makePlanNodeDesc(node->dep(i));
  }

  return planNodeDescPos;
}
```

**流程说明：**
1. **去重检查**：通过`nodeIndexMap`避免重复处理同一节点
2. **生成描述**：调用节点的`explain()`方法获取`PlanNodeDescription`
3. **初始化性能统计**：为每个节点创建空的`ProfilingStats`向量
4. **处理控制流**：对Select/Loop节点设置分支信息
5. **递归遍历**：处理所有依赖的子节点

### 3. 节点explain()方法实现

#### 基类实现

位置：`nebula-3.8.0/src/graph/planner/plan/PlanNode.cpp:418-424`

```cpp
std::unique_ptr<PlanNodeDescription> PlanNode::explain() const {
  auto desc = std::make_unique<PlanNodeDescription>();
  desc->id = id_;
  desc->name = toString(kind_);
  desc->outputVar = folly::toJson(util::toJson(outputVar_));
  return desc;
}
```

#### 单输入节点实现

位置：`nebula-3.8.0/src/graph/planner/plan/PlanNode.cpp:446-450`

```cpp
std::unique_ptr<PlanNodeDescription> SingleInputNode::explain() const {
  auto desc = SingleDependencyNode::explain();
  addDescription("inputVar", inputVar(), desc.get());
  return desc;
}
```

#### 双输入节点实现

位置：`nebula-3.8.0/src/graph/planner/plan/PlanNode.cpp:494-502`

```cpp
std::unique_ptr<PlanNodeDescription> BinaryInputNode::explain() const {
  auto desc = PlanNode::explain();
  DCHECK(desc->dependencies == nullptr);
  desc->dependencies.reset(new std::vector<int64_t>{left()->id(), right()->id()});
  folly::dynamic inputVar = folly::dynamic::object();
  inputVar.insert("leftVar", leftInputVar());
  inputVar.insert("rightVar", rightInputVar());
  addDescription("inputVar", folly::toJson(inputVar), desc.get());
  return desc;
}
```

#### 具体节点实现示例

位置：`nebula-3.8.0/src/graph/planner/plan/Query.cpp`

```cpp
// Explore节点
std::unique_ptr<PlanNodeDescription> Explore::explain() const {
  auto desc = SingleInputNode::explain();
  addDescription("space", folly::to<std::string>(space_), desc.get());
  addDescription("dedup", folly::to<std::string>(dedup_), desc.get());
  addDescription("limit", limit_ ? limit_->toString() : "", desc.get());
  addDescription("filter", filter_ ? filter_->toString() : "", desc.get());
  addDescription("orderBy", folly::toJson(util::toJson(orderBy_)), desc.get());
  return desc;
}

// GetNeighbors节点
std::unique_ptr<PlanNodeDescription> GetNeighbors::explain() const {
  auto desc = Explore::explain();
  addDescription("src", src_ ? src_->toString() : "", desc.get());
  addDescription("edgeTypes", folly::toJson(util::toJson(edgeTypes_)), desc.get());
  addDescription("edgeDirection", apache::thrift::util::enumNameSafe(edgeDirection_), desc.get());
  addDescription("random", folly::to<std::string>(random_), desc.get());
  return desc;
}

// Filter节点
std::unique_ptr<PlanNodeDescription> Filter::explain() const {
  auto desc = SingleInputNode::explain();
  addDescription("condition", condition_ ? condition_->toString() : "", desc.get());
  addDescription("needStableFilter", folly::to<std::string>(needStableFilter_), desc.get());
  return desc;
}
```

### 4. 辅助方法

#### addDescription()

位置：`nebula-3.8.0/src/graph/planner/plan/PlanNode.cpp:398-402`

```cpp
void PlanNode::addDescription(std::string key, std::string value, PlanNodeDescription* desc) {
  if (desc->description == nullptr) {
    desc->description = std::make_unique<std::vector<Pair>>();
  }
  desc->description->emplace_back(Pair{std::move(key), std::move(value)});
}
```

#### setPlanNodeDeps()

位置：`nebula-3.8.0/src/graph/planner/plan/ExecutionPlan.cpp:52-58`

```cpp
void ExecutionPlan::setPlanNodeDeps(const PlanNode* node, PlanNodeDescription* planNodeDesc) const {
  auto deps = std::make_unique<std::vector<int64_t>>();
  for (size_t i = 0; i < node->numDeps(); ++i) {
    deps->emplace_back(node->dep(i)->id());
  }
  planNodeDesc->dependencies = std::move(deps);
}
```

#### descBranchInfo()

位置：`nebula-3.8.0/src/graph/planner/plan/ExecutionPlan.cpp:60-66`

```cpp
void ExecutionPlan::descBranchInfo(const PlanNode* node, bool isDoBranch, int64_t id) {
  auto pos = makePlanNodeDesc(node);
  auto info = std::make_unique<PlanNodeBranchInfo>();
  info->isDoBranch = isDoBranch;
  info->conditionNodeId = id;
  planDescription_->planNodeDescs[pos].branchInfo = std::move(info);
}
```

## 完整调用流程图

```
用户执行EXPLAIN查询
        ↓
GraphService::execute()
        ↓
ExecutionPlan::describe(planDesc)
        ↓
makePlanNodeDesc(root_)
        ↓
┌─────────────────────────────────────┐
│ 1. 检查节点是否已处理                │
│    (通过nodeIndexMap)               │
└─────────────────────────────────────┘
        ↓
┌─────────────────────────────────────┐
│ 2. 调用node->explain()              │
│    - 生成PlanNodeDescription        │
│    - 设置name, id, outputVar       │
│    - 添加节点特定描述               │
└─────────────────────────────────────┘
        ↓
┌─────────────────────────────────────┐
│ 3. 初始化profiles向量               │
└─────────────────────────────────────┘
        ↓
┌─────────────────────────────────────┐
│ 4. 处理控制流节点                   │
│    (Select/Loop的分支信息)          │
└─────────────────────────────────────┘
        ↓
┌─────────────────────────────────────┐
│ 5. 递归处理所有依赖节点             │
│    for (i = 0; i < node->numDeps()) │
│        makePlanNodeDesc(dep(i))     │
└─────────────────────────────────────┘
        ↓
返回节点索引位置
        ↓
所有节点处理完成
        ↓
planDesc->toJson()
        ↓
返回JSON格式的执行计划描述
```

## 输出格式示例

```json
{
  "planNodeDescs": [
    {
      "name": "Start",
      "id": 1,
      "outputVar": "__Start_1",
      "description": [],
      "profiles": [],
      "dependencies": []
    },
    {
      "name": "Filter",
      "id": 2,
      "outputVar": "__Filter_2",
      "description": [
        {"key": "condition", "value": "v.age > 18"},
        {"key": "inputVar", "value": "__Start_1"}
      ],
      "profiles": [],
      "dependencies": [1]
    },
    {
      "name": "Project",
      "id": 3,
      "outputVar": "__Project_3",
      "description": [
        {"key": "columns", "value": "[v.name, v.age]"},
        {"key": "inputVar", "value": "__Filter_2"}
      ],
      "profiles": [],
      "dependencies": [2]
    }
  ],
  "nodeIndexMap": {
    "1": 0,
    "2": 1,
    "3": 2
  },
  "format": "",
  "optimize_time_in_us": 1500
}
```

## graphDB实现建议

### 1. 定义核心数据结构

```rust
// src/query/planner/plan/core/explain.rs

/// 节点描述键值对
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pair {
    pub key: String,
    pub value: String,
}

/// 分支信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanNodeBranchInfo {
    pub is_do_branch: bool,
    pub condition_node_id: i64,
}

/// 性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingStats {
    pub rows: i64,
    pub exec_duration_in_us: i64,
    pub total_duration_in_us: i64,
    pub other_stats: std::collections::HashMap<String, String>,
}

/// 计划节点描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanNodeDescription {
    pub name: String,
    pub id: i64,
    pub output_var: String,
    pub description: Option<Vec<Pair>>,
    pub profiles: Option<Vec<ProfilingStats>>,
    pub branch_info: Option<PlanNodeBranchInfo>,
    pub dependencies: Option<Vec<i64>>,
}

impl PlanNodeDescription {
    pub fn add_description(&mut self, key: String, value: String) {
        if self.description.is_none() {
            self.description = Some(Vec::new());
        }
        self.description.as_mut().unwrap().push(Pair { key, value });
    }
}

/// 执行计划描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDescription {
    pub plan_node_descs: Vec<PlanNodeDescription>,
    pub node_index_map: std::collections::HashMap<i64, usize>,
    pub format: String,
    pub optimize_time_in_us: i32,
}
```

### 2. 在PlanNode trait中添加explain方法

```rust
// src/query/planner/plan/core/nodes/plan_node_traits.rs

pub trait PlanNode {
    // ... 现有方法 ...

    /// 生成节点描述
    fn explain(&self) -> PlanNodeDescription;
}
```

### 3. 实现ExecutionPlan

```rust
// src/query/planner/plan/core/execution_plan.rs

pub struct ExecutionPlan {
    root: PlanNodeEnum,
    optimize_time_in_us: i32,
    explain_format: String,
}

impl ExecutionPlan {
    pub fn describe(&self) -> PlanDescription {
        let mut plan_desc = PlanDescription {
            plan_node_descs: Vec::new(),
            node_index_map: std::collections::HashMap::new(),
            format: self.explain_format.clone(),
            optimize_time_in_us: self.optimize_time_in_us,
        };
        self.make_plan_node_desc(&self.root, &mut plan_desc);
        plan_desc
    }

    fn make_plan_node_desc(
        &self,
        node: &PlanNodeEnum,
        plan_desc: &mut PlanDescription,
    ) -> usize {
        let node_id = node.id();
        
        // 检查是否已处理
        if let Some(&pos) = plan_desc.node_index_map.get(&node_id) {
            return pos;
        }

        // 生成节点描述
        let pos = plan_desc.plan_node_descs.len();
        plan_desc.node_index_map.insert(node_id, pos);
        
        let mut node_desc = node.explain();
        node_desc.profiles = Some(Vec::new());

        // 处理控制流节点
        match node {
            PlanNodeEnum::Select(select_node) => {
                node_desc.dependencies = Some(select_node.dependencies().iter().map(|n| n.id()).collect());
                // 处理分支
                self.desc_branch_info(&select_node.then_node(), true, node_id, plan_desc);
                self.desc_branch_info(&select_node.else_node(), false, node_id, plan_desc);
            }
            PlanNodeEnum::Loop(loop_node) => {
                node_desc.dependencies = Some(loop_node.dependencies().iter().map(|n| n.id()).collect());
                self.desc_branch_info(&loop_node.body_node(), true, node_id, plan_desc);
            }
            _ => {
                // 设置依赖
                node_desc.dependencies = Some(node.dependencies().iter().map(|n| n.id()).collect());
            }
        }

        plan_desc.plan_node_descs.push(node_desc);

        // 递归处理依赖节点
        for dep in node.dependencies() {
            self.make_plan_node_desc(dep, plan_desc);
        }

        pos
    }

    fn desc_branch_info(
        &self,
        node: &PlanNodeEnum,
        is_do_branch: bool,
        condition_node_id: i64,
        plan_desc: &mut PlanDescription,
    ) {
        let pos = self.make_plan_node_desc(node, plan_desc);
        plan_desc.plan_node_descs[pos].branch_info = Some(PlanNodeBranchInfo {
            is_do_branch,
            condition_node_id,
        });
    }
}
```

### 4. 为各节点实现explain方法

```rust
// StartNode
impl PlanNode for StartNode {
    fn explain(&self) -> PlanNodeDescription {
        PlanNodeDescription {
            name: "Start".to_string(),
            id: self.id(),
            output_var: self.output_var().clone(),
            description: Some(Vec::new()),
            profiles: None,
            branch_info: None,
            dependencies: Some(Vec::new()),
        }
    }
}

// FilterNode
impl PlanNode for FilterNode {
    fn explain(&self) -> PlanNodeDescription {
        let mut desc = PlanNodeDescription {
            name: "Filter".to_string(),
            id: self.id(),
            output_var: self.output_var().clone(),
            description: Some(Vec::new()),
            profiles: None,
            branch_info: None,
            dependencies: None,
        };
        desc.add_description("condition".to_string(), self.condition().to_string());
        desc.add_description("inputVar".to_string(), self.input_var().to_string());
        desc
    }
}

// ProjectNode
impl PlanNode for ProjectNode {
    fn explain(&self) -> PlanNodeDescription {
        let mut desc = PlanNodeDescription {
            name: "Project".to_string(),
            id: self.id(),
            output_var: self.output_var().clone(),
            description: Some(Vec::new()),
            profiles: None,
            branch_info: None,
            dependencies: None,
        };
        desc.add_description("columns".to_string(), self.columns().to_string());
        desc.add_description("inputVar".to_string(), self.input_var().to_string());
        desc
    }
}
```

## 总结

nebula-graph的explain功能实现具有以下特点：

1. **分层设计**：通过PlanNodeDescription和PlanDescription两层结构组织数据
2. **递归遍历**：从根节点开始递归处理整个执行计划树
3. **去重机制**：使用nodeIndexMap避免重复处理共享节点
4. **扩展性**：每个节点类型可以自定义explain方法添加特定描述
5. **性能统计**：预留profiles字段用于执行后的性能分析

graphDB可以参考此设计，利用Rust的类型系统和trait机制实现类似功能，同时考虑使用serde库实现JSON序列化。
