# PlanNode 依赖关系分析文档

## 概述

本文档描述 GraphDB 查询计划节点（PlanNode）之间的依赖关系体系，帮助理解执行计划的拓扑结构和数据流。

## 依赖关系类型

根据节点的输入特性，PlanNode 分为以下几类：

### 1. 零输入节点（ZeroInputNode）

**定义**：没有输入依赖的节点，作为执行计划的起始点。

**节点列表**：
| 节点类型 | 说明 | 依赖 |
|---------|------|-----|
| StartNode | 执行计划入口 | 无 |
| ScanVerticesNode | 全表扫描顶点 | 无 |
| ScanEdgesNode | 全表扫描边 | 无 |

**特点**：
- 作为叶子节点出现在执行计划中
- 直接从存储层读取数据
- 可被优化器并行化

### 2. 单输入节点（SingleInputNode）

**定义**：只有一个上游输入节点的节点。

**节点列表**：
| 节点类型 | 说明 | 输入依赖 |
|---------|------|---------|
| FilterNode | 条件过滤 | 任意单输入节点 |
| ProjectNode | 投影/列选择 | 任意单输入节点 |
| AggregateNode | 聚合运算 | 任意单输入节点 |
| SortNode | 排序 | 任意单输入节点 |
| LimitNode | 限制返回行数 | 任意单输入节点 |
| TopNNode | Top N 排序 | 任意单输入节点 |
| SampleNode | 采样 | 任意单输入节点 |
| DedupNode | 去重 | 任意单输入节点 |
| ExpandNode | 边扩展 | 顶点相关节点 |
| ExpandAllNode | 全扩展 | 顶点相关节点 |
| TraverseNode | 遍历 | 顶点/边节点 |
| AppendVerticesNode | 追加顶点 | 遍历结果 |
| ArgumentNode | 参数传递 | 特定依赖 |
| PassThroughNode | 直通传递 | 任意单输入节点 |

**特点**：
- 构成执行计划的主体
- 数据流从叶子节点流向根节点
- 支持管道化执行

### 3. 双输入节点（BinaryInputNode）

**定义**：有两个上游输入节点的节点，通常用于连接操作。

**节点列表**：
| 节点类型 | 说明 | 输入依赖 |
|---------|------|---------|
| InnerJoinNode | 内连接 | 两个输入流 |
| LeftJoinNode | 左连接 | 两个输入流 |
| CrossJoinNode | 交叉连接 | 两个输入流 |
| HashInnerJoinNode | 哈希内连接 | 两个输入流 |
| HashLeftJoinNode | 哈希左连接 | 两个输入流 |

**特点**：
- 需要协调两个输入流
- 可能导致数据倾斜
- 优化器需要考虑连接顺序

### 4. 多输入节点（MultipleInputNode）

**定义**：有多个上游输入节点的节点。

**节点列表**：
| 节点类型 | 说明 | 输入依赖 |
|---------|------|---------|
| UnionNode | 并集操作 | 多个输入流 |
| DataCollectNode | 数据收集 | 多个输入流 |

**特点**：
- 输入数量不固定
- 需要处理不同输入的模式兼容
- 支持并行收集

### 5. 特殊节点

| 节点类型 | 特点 |
|---------|------|
| LoopNode | 包含循环体依赖 |
| SelectNode | 包含多分支依赖 |
| PatternApplyNode | 模式匹配依赖 |
| RollUpApplyNode | 上卷聚合依赖 |
| UnwindNode | 数组展开依赖 |
| AssignNode | 变量赋值依赖 |

## 依赖关系图示

### 典型查询计划结构

```
MATCH (n) WHERE n.age > 20 RETURN n.name
│
├── ScanVerticesNode (Start)
│       │
│       ▼
├── FilterNode (条件过滤)
│       │
│       ▼
├── ProjectNode (投影)
│       │
│       ▼
└── LimitNode (结果限制)
```

### 连接查询结构

```
MATCH (n)-[e]->(m) WHERE n.age > 20 RETURN n.name, m.name
│
├── ScanVerticesNode (n)
│       │
│       ▼
├── ExpandNode (n → e)
│       │
│       ▼
├── GetNeighborsNode (e → m)
│       │
│       ▼
├── HashInnerJoinNode (合并结果)
│       │
│       ▼
├── FilterNode
│       │
│       ▼
└── ProjectNode
```

## 依赖关系验证

### 规则1：类型兼容性

连接节点的两个输入必须有兼容的模式（schema）：

```rust
impl HashInnerJoinNode {
    pub fn new(
        left_input: PlanNodeEnum,
        right_input: PlanNodeEnum,
        join_keys: Vec<Expression>,
    ) -> Result<Self, PlannerError> {
        // 验证输入模式兼容性
        let left_schema = left_input.output_schema()?;
        let right_schema = right_input.output_schema()?;
        
        if !schemas_compatible(&left_schema, &right_schema) {
            return Err(PlannerError::SchemaMismatch(
                "Join inputs have incompatible schemas".to_string()
            ));
        }
        
        Ok(Self {
            id: -1,
            left_input: Box::new(left_input),
            right_input: Box::new(right_input),
            join_keys,
            output_var: None,
            col_names: vec![],
            cost: 0.0,
        })
    }
}
```

### 规则2：循环依赖检测

计划节点不能形成循环依赖：

```rust
pub fn detect_cycle(node: &PlanNodeEnum) -> bool {
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    
    fn dfs(
        node: &PlanNodeEnum,
        visited: &mut HashSet<i64>,
        stack: &mut HashSet<i64>,
    ) -> bool {
        if stack.contains(&node.id()) {
            return true; // 检测到循环
        }
        
        if visited.contains(&node.id()) {
            return false;
        }
        
        visited.insert(node.id());
        stack.insert(node.id());
        
        for child in node.dependencies() {
            if dfs(child, visited, stack) {
                return true;
            }
        }
        
        stack.remove(&node.id());
        false
    }
    
    dfs(node, &mut visited, &mut stack)
}
```

## 优化器依赖处理

### 1. 下推过滤

尽可能将 FilterNode 下推到访问层：

```rust
pub fn push_down_filter(plan: &mut ExecutionPlan) {
    if let Some(filter) = plan.root_mut().as_filter_mut() {
        if let Some(scan) = filter.input_mut().as_scan_vertices_mut() {
            // 将过滤条件下推到扫描节点
            scan.add_filter(filter.condition().clone());
            // 用扫描节点替换过滤节点
            *plan.root_mut() = filter.input_mut().clone();
        }
    }
}
```

### 2. 连接重排

根据代价模型重排连接顺序：

```rust
pub fn reorder_joins(plan: &mut ExecutionPlan) {
    if let Some(join) = plan.root_mut().as_hash_inner_join_mut() {
        let left_cost = estimate_cost(join.left_input());
        let right_cost = estimate_cost(join.right_input());
        
        // 代价小的表作为构建表（哈希表）
        if left_cost > right_cost {
            std::mem::swap(&mut join.left_input, &mut join.right_input);
        }
    }
}
```

### 3. 子计划合并

合并连续的同类操作：

```rust
pub fn merge_consecutive_projects(plan: &mut ExecutionPlan) {
    if let (Some(outer), Some(inner)) = (
        plan.root().as_project(),
        plan.root().input().as_project()
    ) {
        // 如果两个 Project 相邻，合并列表达式
        let merged_columns = merge_columns(
            outer.columns(),
            inner.columns()
        );
        // 用合并后的 Project 替换
    }
}
```
