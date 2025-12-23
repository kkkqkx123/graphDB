# Input 和 Dependencies 统一设计分析

## 当前问题分析

通过分析各种节点类型的实现，我发现了以下几种不同的输入/依赖处理模式：

### 1. 单输入节点（使用 input + deps）

**ProjectNode**:
- 使用 `input: Box<PlanNodeEnum>` 存储单个输入
- 没有独立的 deps 字段
- 通过 `input()` 方法访问输入

**FilterNode**:
- 使用 `input: Box<PlanNodeEnum>` 存储单个输入
- `dependencies()` 方法返回 `std::slice::from_ref(&self.input)`
- 没有独立的 deps 字段

**SortNode/LimitNode/TopNNode**:
- 同时使用 `input: Box<PlanNodeEnum>` 和 `deps: Vec<Box<PlanNodeEnum>>`
- deps 中包含与 input 相同的数据，造成重复

**AggregateNode**:
- 同时使用 `input: Box<PlanNodeEnum>` 和 `deps: Vec<Box<PlanNodeEnum>>`
- deps 中包含与 input 相同的数据，造成重复

### 2. 多输入节点（使用具体字段 + deps）

**InnerJoinNode/LeftJoinNode/CrossJoinNode**:
- 使用具体字段 `left: Box<PlanNodeEnum>` 和 `right: Box<PlanNodeEnum>`
- 同时维护 `inner_deps: Vec<Box<PlanNodeEnum>>`
- inner_deps 包含与 left/right 相同的数据，造成重复

### 3. 无输入节点

**StartNode**:
- 没有输入字段
- 维护空的 `dependencies_vec: Vec<PlanNodeEnum>`

### 4. 管理节点

**用户管理节点 (CreateUser, DropUser 等)**:
- 使用 `deps: Vec<PlanNodeEnum>` 存储依赖
- 实现 PlanNode trait 的 `dependencies()` 方法

**插入节点 (InsertVertices, InsertEdges)**:
- 没有明确的输入/依赖字段
- 使用 Arc 包装，结构不同

## 设计问题

1. **数据重复**: input 和 deps 存储相同的数据，浪费内存且容易导致不一致
2. **接口不统一**: 不同节点类型使用不同的方式存储和访问输入/依赖
3. **复杂性增加**: 维护两套数据结构增加了代码复杂性
4. **类型不一致**: 有些使用 `Vec<PlanNodeEnum>`，有些使用 `Vec<Box<PlanNodeEnum>>`

## 推荐的统一设计方案

### 方案一：统一使用 deps（推荐）

```rust
pub trait PlanNode {
    fn id(&self) -> i64;
    fn name(&self) -> &'static str;
    fn output_var(&self) -> Option<&Variable>;
    fn col_names(&self) -> &[String];
    fn cost(&self) -> f64;
    
    // 统一使用 dependencies 方法
    fn dependencies(&self) -> &[Box<PlanNodeEnum>];
    
    // 提供便捷方法访问特定输入
    fn input(&self) -> Option<&PlanNodeEnum> {
        // 对于单输入节点，返回第一个依赖
        self.dependencies().first().map(|boxed| boxed.as_ref())
    }
    
    fn left_input(&self) -> Option<&PlanNodeEnum> {
        // 对于双输入节点，返回左输入
        self.dependencies().get(0).map(|boxed| boxed.as_ref())
    }
    
    fn right_input(&self) -> Option<&PlanNodeEnum> {
        // 对于双输入节点，返回右输入
        self.dependencies().get(1).map(|boxed| boxed.as_ref())
    }
    
    fn set_output_var(&mut self, var: Variable);
    fn set_col_names(&mut self, names: Vec<String>);
    fn into_enum(self) -> PlanNodeEnum;
}
```

### 方案二：根据节点类型使用不同的输入访问方式

```rust
pub trait PlanNode {
    fn id(&self) -> i64;
    fn name(&self) -> &'static str;
    fn output_var(&self) -> Option<&Variable>;
    fn col_names(&self) -> &[String];
    fn cost(&self) -> f64;
    
    // 移除 dependencies 方法，使用更具体的输入访问方法
    fn input_count(&self) -> usize;
    
    fn set_output_var(&mut self, var: Variable);
    fn set_col_names(&mut self, names: Vec<String>);
    fn into_enum(self) -> PlanNodeEnum;
}

// 为不同类型的节点提供特定的 trait
pub trait SingleInputNode: PlanNode {
    fn input(&self) -> &PlanNodeEnum;
    fn set_input(&mut self, input: PlanNodeEnum);
}

pub trait DualInputNode: PlanNode {
    fn left_input(&self) -> &PlanNodeEnum;
    fn right_input(&self) -> &PlanNodeEnum;
    fn set_left_input(&mut self, input: PlanNodeEnum);
    fn set_right_input(&mut self, input: PlanNodeEnum);
}

pub trait MultiInputNode: PlanNode {
    fn inputs(&self) -> &[Box<PlanNodeEnum>];
    fn add_input(&mut self, input: PlanNodeEnum);
    fn remove_input(&mut self, index: usize) -> Option<Box<PlanNodeEnum>>;
}
```

## 推荐方案

我推荐**方案一**，原因如下：

1. **统一接口**: 所有节点都使用相同的 `dependencies()` 方法，简化了遍历和操作
2. **灵活性**: 可以轻松支持任意数量的输入
3. **向后兼容**: 可以通过便捷方法（如 `input()`, `left_input()`）提供特定访问方式
4. **减少重复**: 只维护一个数据结构，避免数据不一致
5. **简化实现**: 节点实现更简单，只需要维护一个 deps 字段

## 实现建议

1. **移除重复字段**: 移除所有节点中的 `input` 字段，统一使用 `deps` 或 `dependencies` 字段
2. **统一类型**: 所有依赖使用 `Vec<Box<PlanNodeEnum>>` 类型
3. **提供便捷方法**: 在 trait 中提供便捷方法访问特定输入
4. **更新节点实现**: 
   - 单输入节点：`input()` 方法返回第一个依赖
   - 双输入节点：`left_input()` 和 `right_input()` 方法分别返回第一个和第二个依赖
   - 多输入节点：通过索引访问特定输入

## 具体实现示例

### 单输入节点（如 ProjectNode）

```rust
#[derive(Debug, Clone)]
pub struct ProjectNode {
    id: i64,
    deps: Vec<Box<PlanNodeEnum>>,  // 只保留 deps
    columns: Vec<YieldColumn>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl ProjectNode {
    pub fn input(&self) -> &PlanNodeEnum {
        &self.deps[0]  // 第一个依赖作为输入
    }
    
    pub fn set_input(&mut self, input: PlanNodeEnum) {
        self.deps.clear();
        self.deps.push(Box::new(input));
    }
}
```

### 双输入节点（如 InnerJoinNode）

```rust
#[derive(Debug, Clone)]
pub struct InnerJoinNode {
    id: i64,
    deps: Vec<Box<PlanNodeEnum>>,  // 只保留 deps
    hash_keys: Vec<Expression>,
    probe_keys: Vec<Expression>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl InnerJoinNode {
    pub fn left_input(&self) -> &PlanNodeEnum {
        &self.deps[0]  // 第一个依赖作为左输入
    }
    
    pub fn right_input(&self) -> &PlanNodeEnum {
        &self.deps[1]  // 第二个依赖作为右输入
    }
    
    pub fn set_left_input(&mut self, input: PlanNodeEnum) {
        self.deps[0] = Box::new(input);
    }
    
    pub fn set_right_input(&mut self, input: PlanNodeEnum) {
        self.deps[1] = Box::new(input);
    }
}
```

## 迁移计划

1. **第一阶段**: 更新 PlanNode trait，添加便捷方法
2. **第二阶段**: 逐个更新节点实现，移除重复字段
3. **第三阶段**: 更新 plan_node_operations.rs，使用新的统一接口
4. **第四阶段**: 更新所有使用这些节点的代码

这样的设计既保持了统一性，又提供了便捷的访问方式，同时避免了数据重复的问题。