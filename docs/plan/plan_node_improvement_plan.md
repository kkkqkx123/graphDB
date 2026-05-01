# PlanNode 体系改进方案

## 概述

本文档针对 `src/query/planning/plan` 目录中 PlanNode 体系存在的问题，提出系统性的改进方案。改进分为三个阶段：

1. **功能补全** - 补充缺失的节点类型
2. **架构优化** - 改进代码结构和可维护性
3. **运行时验证** - 增强安全性和健壮性

---

## 第一阶段：功能补全

### 1.1 连接类型补全

#### 1.1.1 RightJoin（右连接）

**现状**：缺少 RightJoin 节点

**实现方案**：

```rust
// 文件：src/query/planning/plan/core/nodes/join/join_node.rs

define_join_node! {
    pub struct RightJoinNode {
    }
    enum: RightJoin
}

impl RightJoinNode {
    pub fn new(
        left: PlanNodeEnum,
        right: PlanNodeEnum,
        hash_keys: Vec<ContextualExpression>,
        probe_keys: Vec<ContextualExpression>,
    ) -> Result<Self, PlannerError> {
        let mut col_names = left.col_names().to_vec();
        for col in right.col_names() {
            if !col_names.contains(col) {
                col_names.push(col.clone());
            } else {
                let mut idx = 1;
                let mut new_col = format!("{}_{}", col, idx);
                while col_names.contains(&new_col) {
                    idx += 1;
                    new_col = format!("{}_{}", col, idx);
                }
                col_names.push(new_col);
            }
        }

        Ok(Self {
            id: -1,
            left: Box::new(left),
            right: Box::new(right),
            hash_keys,
            probe_keys,
            deps: vec![],
            output_var: None,
            col_names,
        })
    }
}
```

**优化器转换规则**：

```rust
// RightJoin 可以转换为 LeftJoin（交换左右输入）
// A RIGHT JOIN B ON A.id = B.id <=> B LEFT JOIN A ON A.id = B.id
impl RightJoinNode {
    pub fn to_left_join(self) -> LeftJoinNode {
        LeftJoinNode::new(
            *self.right,  // 交换
            *self.left,   // 交换
            self.probe_keys,  // 交换
            self.hash_keys,   // 交换
        )
    }
}
```

#### 1.1.2 SemiJoin（半连接）

**用途**：用于 `EXISTS` 子查询和 `IN` 子查询优化

**实现方案**：

```rust
// 文件：src/query/planning/plan/core/nodes/join/join_node.rs

define_join_node! {
    pub struct SemiJoinNode {
        /// 是否返回匹配的行（true）还是不匹配的行（false）
        /// true = SemiJoin, false = AntiJoin
        anti: bool,
    }
    enum: SemiJoin
}

impl SemiJoinNode {
    /// 创建半连接节点
    /// 用于 EXISTS 子查询：只返回左表中在右表有匹配的行
    pub fn new_semi(
        left: PlanNodeEnum,
        right: PlanNodeEnum,
        hash_keys: Vec<ContextualExpression>,
        probe_keys: Vec<ContextualExpression>,
    ) -> Result<Self, PlannerError> {
        Self::new(left, right, hash_keys, probe_keys, false)
    }

    /// 创建半连接节点（公开接口）
    pub fn new(
        left: PlanNodeEnum,
        right: PlanNodeEnum,
        hash_keys: Vec<ContextualExpression>,
        probe_keys: Vec<ContextualExpression>,
        anti: bool,
    ) -> Result<Self, PlannerError> {
        let col_names = left.col_names().to_vec();

        Ok(Self {
            id: -1,
            left: Box::new(left),
            right: Box::new(right),
            hash_keys,
            probe_keys,
            anti,
            deps: vec![],
            output_var: None,
            col_names,
        })
    }

    pub fn is_anti(&self) -> bool {
        self.anti
    }
}
```

#### 1.1.3 AntiJoin（反连接）

**用途**：用于 `NOT EXISTS` 子查询和 `NOT IN` 子查询优化

**实现方案**：

```rust
// AntiJoin 可以复用 SemiJoinNode，设置 anti = true
// 或者单独定义一个类型别名
pub type AntiJoinNode = SemiJoinNode;

impl AntiJoinNode {
    /// 创建反连接节点
    /// 用于 NOT EXISTS 子查询：只返回左表中在右表没有匹配的行
    pub fn new_anti(
        left: PlanNodeEnum,
        right: PlanNodeEnum,
        hash_keys: Vec<ContextualExpression>,
        probe_keys: Vec<ContextualExpression>,
    ) -> Result<Self, PlannerError> {
        SemiJoinNode::new(left, right, hash_keys, probe_keys, true)
    }
}
```

#### 1.1.4 FullOuterJoin 完善

**现状**：已定义但实现可能不完整

**完善方案**：

```rust
define_join_node! {
    pub struct FullOuterJoinNode {
    }
    enum: FullOuterJoin
}

impl FullOuterJoinNode {
    pub fn new(
        left: PlanNodeEnum,
        right: PlanNodeEnum,
        hash_keys: Vec<ContextualExpression>,
        probe_keys: Vec<ContextualExpression>,
    ) -> Result<Self, PlannerError> {
        let mut col_names = left.col_names().to_vec();
        for col in right.col_names() {
            if !col_names.contains(col) {
                col_names.push(col.clone());
            } else {
                let mut idx = 1;
                let mut new_col = format!("{}_{}", col, idx);
                while col_names.contains(&new_col) {
                    idx += 1;
                    new_col = format!("{}_{}", col, idx);
                }
                col_names.push(new_col);
            }
        }

        Ok(Self {
            id: -1,
            left: Box::new(left),
            right: Box::new(right),
            hash_keys,
            probe_keys,
            deps: vec![],
            output_var: None,
            col_names,
        })
    }
}
```

**执行器实现要点**：

```rust
// 执行器需要处理三种情况：
// 1. 两边都匹配：输出连接结果
// 2. 左边无匹配：输出左边 + NULL
// 3. 右边无匹配：输出 NULL + 右边
impl FullOuterJoinExecutor {
    fn execute(&self) -> Result<Dataset, ExecutionError> {
        let left_data = self.left_input.execute()?;
        let right_data = self.right_input.execute()?;

        let mut result = Dataset::new();
        let mut left_matched = HashSet::new();
        let mut right_matched = HashSet::new();

        // Phase 1: Inner Join
        for left_row in &left_data {
            for right_row in &right_data {
                if self.matches(left_row, right_row) {
                    result.push(self.merge_rows(left_row, right_row));
                    left_matched.insert(left_row.id());
                    right_matched.insert(right_row.id());
                }
            }
        }

        // Phase 2: Left unmatched
        for left_row in &left_data {
            if !left_matched.contains(&left_row.id()) {
                result.push(self.merge_with_null_right(left_row));
            }
        }

        // Phase 3: Right unmatched
        for right_row in &right_data {
            if !right_matched.contains(&right_row.id()) {
                result.push(self.merge_with_null_left(right_row));
            }
        }

        Ok(result)
    }
}
```

### 1.2 双向遍历节点

#### 1.2.1 BiExpand（双向扩展）

**用途**：从两个方向同时扩展边，用于路径查找优化

**实现方案**：

```rust
// 文件：src/query/planning/plan/core/nodes/traversal/traversal_node.rs

define_plan_node_with_deps! {
    pub struct BiExpandNode {
        /// 左侧扩展方向
        left_direction: EdgeDirection,
        /// 右侧扩展方向
        right_direction: EdgeDirection,
        /// 边类型过滤
        edge_types: Vec<String>,
        /// 最大跳数
        max_hops: usize,
    }
    enum: BiExpand
    input: BinaryInputNode
}

impl BiExpandNode {
    pub fn new(
        left_input: PlanNodeEnum,
        right_input: PlanNodeEnum,
        left_direction: EdgeDirection,
        right_direction: EdgeDirection,
        edge_types: Vec<String>,
        max_hops: usize,
    ) -> Result<Self, PlannerError> {
        Ok(Self {
            id: -1,
            left: Box::new(left_input),
            right: Box::new(right_input),
            left_direction,
            right_direction,
            edge_types,
            max_hops,
            deps: vec![],
            output_var: None,
            col_names: vec![],
        })
    }
}
```

**执行策略**：

```rust
// 双向 BFS 扩展，直到两个方向相遇
impl BiExpandExecutor {
    fn execute(&self) -> Result<Dataset, ExecutionError> {
        let left_start = self.left_input.execute()?;
        let right_start = self.right_input.execute()?;

        let mut left_frontier = left_start;
        let mut right_frontier = right_start;
        let mut left_visited = HashMap::new();
        let mut right_visited = HashMap::new();
        let mut paths = Vec::new();

        for hop in 0..self.max_hops {
            // 左侧扩展一步
            left_frontier = self.expand_step(&left_frontier, self.left_direction)?;
            for vertex in &left_frontier {
                left_visited.insert(vertex.id(), hop + 1);
            }

            // 检查是否相遇
            if let Some(meeting_points) = self.find_meeting_points(&left_frontier, &right_frontier) {
                paths.extend(self.build_paths(&meeting_points, &left_visited, &right_visited)?);
            }

            // 右侧扩展一步
            right_frontier = self.expand_step(&right_frontier, self.right_direction)?;
            for vertex in &right_frontier {
                right_visited.insert(vertex.id(), hop + 1);
            }

            // 再次检查是否相遇
            if let Some(meeting_points) = self.find_meeting_points(&left_frontier, &right_frontier) {
                paths.extend(self.build_paths(&meeting_points, &left_visited, &right_visited)?);
            }
        }

        Ok(Dataset::from_rows(paths))
    }
}
```

#### 1.2.2 BiTraverse（双向遍历）

**用途**：双向遍历并返回完整路径

**实现方案**：

```rust
define_plan_node_with_deps! {
    pub struct BiTraverseNode {
        /// 左侧起始点变量
        left_src_var: String,
        /// 右侧起始点变量
        right_src_var: String,
        /// 边类型
        edge_types: Vec<String>,
        /// 方向（通常两侧方向相反）
        left_direction: EdgeDirection,
        right_direction: EdgeDirection,
        /// 步数范围
        min_hops: usize,
        max_hops: usize,
        /// 输出路径变量
        path_var: String,
    }
    enum: BiTraverse
    input: BinaryInputNode
}

impl BiTraverseNode {
    pub fn new(
        left_input: PlanNodeEnum,
        right_input: PlanNodeEnum,
        config: BiTraverseConfig,
    ) -> Result<Self, PlannerError> {
        Ok(Self {
            id: -1,
            left: Box::new(left_input),
            right: Box::new(right_input),
            left_src_var: config.left_src_var,
            right_src_var: config.right_src_var,
            edge_types: config.edge_types,
            left_direction: config.left_direction,
            right_direction: config.right_direction,
            min_hops: config.min_hops,
            max_hops: config.max_hops,
            path_var: config.path_var,
            deps: vec![],
            output_var: Some(config.path_var),
            col_names: vec![config.path_var.clone()],
        })
    }
}
```

### 1.3 子查询支持节点

#### 1.3.1 Apply 节点（相关子查询）

**用途**：处理相关子查询，对每一行执行子查询

**实现方案**：

```rust
// 文件：src/query/planning/plan/core/nodes/graph_operations/apply_node.rs

define_plan_node_with_deps! {
    pub struct ApplyNode {
        /// 子查询计划
        subquery: Box<PlanNodeEnum>,
        /// 关联变量（从外部传入子查询）
        correlation_vars: Vec<String>,
        /// Apply 类型
        apply_type: ApplyType,
    }
    enum: Apply
    input: SingleInputNode
}

/// Apply 类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyType {
    /// 普通应用：返回子查询的所有结果
    All,
    /// 存在性检查：返回布尔值
    Exists,
    /// 唯一值检查：返回单个值
    SingleValue,
    /// 集合包含检查
    In,
}

impl ApplyNode {
    pub fn new(
        input: PlanNodeEnum,
        subquery: PlanNodeEnum,
        correlation_vars: Vec<String>,
        apply_type: ApplyType,
    ) -> Result<Self, PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input)),
            subquery: Box::new(subquery),
            correlation_vars,
            apply_type,
            deps: vec![],
            output_var: None,
            col_names,
        })
    }

    pub fn subquery(&self) -> &PlanNodeEnum {
        &self.subquery
    }

    pub fn correlation_vars(&self) -> &[String] {
        &self.correlation_vars
    }

    pub fn apply_type(&self) -> ApplyType {
        self.apply_type
    }
}
```

**执行器实现**：

```rust
impl ApplyExecutor {
    fn execute(&self) -> Result<Dataset, ExecutionError> {
        let outer_data = self.input.execute()?;
        let mut result = Dataset::new();

        for outer_row in outer_data {
            // 设置关联变量
            for var in &self.correlation_vars {
                self.context.set_variable(var, outer_row.get(var)?);
            }

            // 执行子查询
            let subquery_result = self.subquery.execute()?;

            // 根据类型处理结果
            match self.apply_type {
                ApplyType::All => {
                    for sub_row in subquery_result {
                        result.push(self.merge_rows(&outer_row, &sub_row)?);
                    }
                }
                ApplyType::Exists => {
                    let exists = !subquery_result.is_empty();
                    result.push(self.add_column(&outer_row, "exists", Value::Bool(exists))?);
                }
                ApplyType::SingleValue => {
                    if subquery_result.len() == 1 {
                        let value = subquery_result.first()?.get(0)?;
                        result.push(self.add_column(&outer_row, "value", value)?);
                    } else if subquery_result.is_empty() {
                        result.push(self.add_column(&outer_row, "value", Value::Null)?);
                    } else {
                        return Err(ExecutionError::CardinalityViolation(
                            "Subquery returned more than one row".to_string()
                        ));
                    }
                }
                ApplyType::In => {
                    // 检查外部值是否在子查询结果中
                    let outer_value = outer_row.get(self.check_var)?;
                    let found = subquery_result.iter().any(|r| r.get(0)? == outer_value);
                    result.push(self.add_column(&outer_row, "in_result", Value::Bool(found))?);
                }
            }
        }

        Ok(result)
    }
}
```

#### 1.3.2 子查询优化规则

**优化转换**：

```rust
// 相关子查询 -> SemiJoin/AntiJoin
pub fn optimize_apply_to_join(plan: &mut PlanNodeEnum) -> bool {
    if let PlanNodeEnum::Apply(apply) = plan {
        match apply.apply_type {
            ApplyType::Exists => {
                // EXISTS 子查询 -> SemiJoin
                *plan = PlanNodeEnum::SemiJoin(
                    SemiJoinNode::new_semi(
                        apply.input.take(),
                        *apply.subquery.clone(),
                        apply.correlation_vars.clone(),
                        vec![],
                        false,
                    )
                );
                return true;
            }
            ApplyType::In if !apply.correlation_vars.is_empty() => {
                // IN 子查询 -> SemiJoin
                // ...
            }
            _ => {}
        }
    }
    false
}
```

### 1.4 事务控制节点

#### 1.4.1 事务控制节点定义

**实现方案**：

```rust
// 文件：src/query/planning/plan/core/nodes/control_flow/transaction_node.rs

define_plan_node! {
    pub struct BeginTransactionNode {
        /// 事务隔离级别
        isolation_level: IsolationLevel,
        /// 访问模式
        access_mode: AccessMode,
    }
    enum: BeginTransaction
    input: ZeroInputNode
}

define_plan_node_with_deps! {
    pub struct CommitTransactionNode {
        /// 事务ID（可选，用于显式提交）
        transaction_id: Option<u64>,
    }
    enum: CommitTransaction
    input: SingleInputNode
}

define_plan_node_with_deps! {
    pub struct RollbackTransactionNode {
        /// 事务ID（可选）
        transaction_id: Option<u64>,
        /// 是否回滚到保存点
        savepoint: Option<String>,
    }
    enum: RollbackTransaction
    input: SingleInputNode
}

define_plan_node! {
    pub struct SavepointNode {
        /// 保存点名称
        name: String,
    }
    enum: Savepoint
    input: ZeroInputNode
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    ReadOnly,
    ReadWrite,
}
```

#### 1.4.2 事务块节点

```rust
define_plan_node_with_deps! {
    pub struct TransactionBlockNode {
        /// 事务体
        body: Box<PlanNodeEnum>,
        /// 事务配置
        config: TransactionConfig,
    }
    enum: TransactionBlock
    input: SingleInputNode
}

impl TransactionBlockNode {
    pub fn new(body: PlanNodeEnum, config: TransactionConfig) -> Self {
        Self {
            id: -1,
            input: None,
            body: Box::new(body),
            config,
            deps: vec![],
            output_var: None,
            col_names: vec![],
        }
    }

    pub fn body(&self) -> &PlanNodeEnum {
        &self.body
    }
}
```

---

## 第二阶段：架构优化

### 2.1 宏定义统一

#### 2.1.1 当前问题

各子模块有独立的 `macros.rs`：

- `src/query/planning/plan/core/nodes/operation/macros.rs`
- `src/query/planning/plan/core/nodes/join/macros.rs`
- `src/query/planning/plan/core/nodes/traversal/macros.rs`

#### 2.1.2 统一方案

**文件结构**：

```
src/query/planning/plan/core/nodes/base/
├── mod.rs
├── macros.rs              # 统一的宏定义
│   ├── define_plan_node!
│   ├── define_plan_node_with_deps!
│   ├── define_binary_input_node!
│   ├── define_join_node!
│   └── 其他通用宏
├── plan_node_traits.rs
├── plan_node_enum.rs
└── ...
```

**统一宏定义**：

```rust
// 文件：src/query/planning/plan/core/nodes/base/macros.rs

/// 统一的计划节点定义宏
///
/// 支持所有输入类型：ZeroInput, SingleInput, BinaryInput, MultipleInput
#[macro_export]
macro_rules! define_plan_node {
    // 零输入节点
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident: $field_ty:ty,
            )*
        }
        enum: $enum_variant:ident
        input: ZeroInputNode
    ) => {
        $(#[$outer])*
        $vis struct $name {
            pub id: i64,
            $(
                $(#[$field_meta])*
                $field_vis $field: $field_ty,
            )*
            pub output_var: Option<String>,
            pub col_names: Vec<String>,
        }

        impl $crate::query::planning::plan::core::nodes::base::plan_node_traits::PlanNode for $name {
            fn id(&self) -> i64 { self.id }
            fn name(&self) -> &'static str { stringify!($name) }
            fn category(&self) -> $crate::query::planning::plan::core::nodes::base::plan_node_category::PlanNodeCategory {
                $crate::query::planning::plan::core::nodes::base::plan_node_category::PlanNodeCategory::Access
            }
            fn output_var(&self) -> Option<&str> { self.output_var.as_deref() }
            fn col_names(&self) -> &[String] { &self.col_names }
            fn set_output_var(&mut self, var: String) { self.output_var = Some(var); }
            fn set_col_names(&mut self, names: Vec<String>) { self.col_names = names; }
            fn into_enum(self) -> $crate::query::planning::plan::PlanNodeEnum {
                $crate::query::planning::plan::PlanNodeEnum::$enum_variant(self)
            }
        }

        impl $crate::query::planning::plan::core::nodes::base::plan_node_traits::ZeroInputNode for $name {}
    };

    // 单输入节点
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident: $field_ty:ty,
            )*
        }
        enum: $enum_variant:ident
        input: SingleInputNode
    ) => {
        // ... 类似实现
    };

    // 双输入节点
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident: $field_ty:ty,
            )*
        }
        enum: $enum_variant:ident
        input: BinaryInputNode
    ) => {
        // ... 类似实现
    };

    // 多输入节点
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident: $field_ty:ty,
            )*
        }
        enum: $enum_variant:ident
        input: MultipleInputNode
    ) => {
        // ... 类似实现
    };
}
```

### 2.2 PlanNodeEnum 分层重构

#### 2.2.1 当前问题

`PlanNodeEnum` 有 90+ 个变体，导致：

- 编译时间长
- match 语句冗长
- 难以维护

#### 2.2.2 分层方案

```rust
// 文件：src/query/planning/plan/core/nodes/base/plan_node_enum.rs

/// 访问层节点枚举
#[derive(Debug, Clone)]
pub enum AccessNodeEnum {
    Start(StartNode),
    ScanVertices(ScanVerticesNode),
    ScanEdges(ScanEdgesNode),
    GetVertices(GetVerticesNode),
    GetEdges(GetEdgesNode),
    GetNeighbors(GetNeighborsNode),
    IndexScan(IndexScanNode),
    EdgeIndexScan(EdgeIndexScanNode),
}

/// 操作层节点枚举
#[derive(Debug, Clone)]
pub enum OperationNodeEnum {
    Filter(FilterNode),
    Project(ProjectNode),
    Sort(SortNode),
    Limit(LimitNode),
    TopN(TopNNode),
    Sample(SampleNode),
    Aggregate(AggregateNode),
    Dedup(DedupNode),
}

/// 连接层节点枚举
#[derive(Debug, Clone)]
pub enum JoinNodeEnum {
    InnerJoin(InnerJoinNode),
    LeftJoin(LeftJoinNode),
    RightJoin(RightJoinNode),
    CrossJoin(CrossJoinNode),
    HashInnerJoin(HashInnerJoinNode),
    HashLeftJoin(HashLeftJoinNode),
    FullOuterJoin(FullOuterJoinNode),
    SemiJoin(SemiJoinNode),
}

/// 遍历层节点枚举
#[derive(Debug, Clone)]
pub enum TraversalNodeEnum {
    Expand(ExpandNode),
    ExpandAll(ExpandAllNode),
    Traverse(TraverseNode),
    AppendVertices(AppendVerticesNode),
    BiExpand(BiExpandNode),
    BiTraverse(BiTraverseNode),
}

/// 控制流节点枚举
#[derive(Debug, Clone)]
pub enum ControlFlowNodeEnum {
    Argument(ArgumentNode),
    Loop(LoopNode),
    PassThrough(PassThroughNode),
    Select(SelectNode),
    Apply(ApplyNode),
    BeginTransaction(BeginTransactionNode),
    CommitTransaction(CommitTransactionNode),
    RollbackTransaction(RollbackTransactionNode),
}

/// 数据处理节点枚举
#[derive(Debug, Clone)]
pub enum DataProcessingNodeEnum {
    DataCollect(DataCollectNode),
    Union(UnionNode),
    Minus(MinusNode),
    Intersect(IntersectNode),
    Unwind(UnwindNode),
    Assign(AssignNode),
    PatternApply(PatternApplyNode),
    RollUpApply(RollUpApplyNode),
    Materialize(MaterializeNode),
    Remove(RemoveNode),
}

/// 算法节点枚举
#[derive(Debug, Clone)]
pub enum AlgorithmNodeEnum {
    ShortestPath(ShortestPathNode),
    AllPaths(AllPathsNode),
    BFSShortest(BFSShortestNode),
    MultiShortestPath(MultiShortestPathNode),
}

/// 管理节点枚举
#[derive(Debug, Clone)]
pub enum ManagementNodeEnum {
    // Space
    CreateSpace(CreateSpaceNode),
    DropSpace(DropSpaceNode),
    DescSpace(DescSpaceNode),
    ShowSpaces(ShowSpacesNode),
    SwitchSpace(SwitchSpaceNode),
    AlterSpace(AlterSpaceNode),
    ClearSpace(ClearSpaceNode),
    // Tag
    CreateTag(CreateTagNode),
    AlterTag(AlterTagNode),
    DescTag(DescTagNode),
    DropTag(DropTagNode),
    ShowTags(ShowTagsNode),
    // Edge
    CreateEdge(CreateEdgeNode),
    AlterEdge(AlterEdgeNode),
    DescEdge(DescEdgeNode),
    DropEdge(DropEdgeNode),
    ShowEdges(ShowEdgesNode),
    // Index
    CreateTagIndex(CreateTagIndexNode),
    DropTagIndex(DropTagIndexNode),
    DescTagIndex(DescTagIndexNode),
    ShowTagIndexes(ShowTagIndexesNode),
    CreateEdgeIndex(CreateEdgeIndexNode),
    DropEdgeIndex(DropEdgeIndexNode),
    DescEdgeIndex(DescEdgeIndexNode),
    ShowEdgeIndexes(ShowEdgeIndexesNode),
    RebuildTagIndex(RebuildTagIndexNode),
    RebuildEdgeIndex(RebuildEdgeIndexNode),
    // User
    CreateUser(CreateUserNode),
    AlterUser(AlterUserNode),
    DropUser(DropUserNode),
    ChangePassword(ChangePasswordNode),
    GrantRole(GrantRoleNode),
    RevokeRole(RevokeRoleNode),
}

/// 数据修改节点枚举
#[derive(Debug, Clone)]
pub enum DataModificationNodeEnum {
    InsertVertices(InsertVerticesNode),
    InsertEdges(InsertEdgesNode),
    DeleteVertices(DeleteVerticesNode),
    DeleteEdges(DeleteEdgesNode),
    Update(UpdateNode),
    UpdateVertices(UpdateVerticesNode),
    UpdateEdges(UpdateEdgesNode),
}

/// 搜索节点枚举
#[derive(Debug, Clone)]
pub enum SearchNodeEnum {
    FulltextSearch(FulltextSearchNode),
    FulltextLookup(FulltextLookupNode),
    MatchFulltext(MatchFulltextNode),
    VectorSearch(VectorSearchNode),
    VectorLookup(VectorLookupNode),
    VectorMatch(VectorMatchNode),
}

/// 顶层计划节点枚举（分层后的版本）
#[derive(Debug, Clone)]
pub enum PlanNodeEnum {
    Access(AccessNodeEnum),
    Operation(OperationNodeEnum),
    Join(JoinNodeEnum),
    Traversal(TraversalNodeEnum),
    ControlFlow(ControlFlowNodeEnum),
    DataProcessing(DataProcessingNodeEnum),
    Algorithm(AlgorithmNodeEnum),
    Management(ManagementNodeEnum),
    DataModification(DataModificationNodeEnum),
    Search(SearchNodeEnum),
}
```

#### 2.2.3 兼容性处理

```rust
// 提供向后兼容的转换方法
impl PlanNodeEnum {
    /// 从旧版枚举转换为新版枚举
    pub fn from_legacy(legacy: LegacyPlanNodeEnum) -> Self {
        match legacy {
            LegacyPlanNodeEnum::Start(n) => PlanNodeEnum::Access(AccessNodeEnum::Start(n)),
            LegacyPlanNodeEnum::Filter(n) => PlanNodeEnum::Operation(OperationNodeEnum::Filter(n)),
            // ... 其他转换
        }
    }

    /// 转换为旧版枚举
    pub fn to_legacy(&self) -> Option<LegacyPlanNodeEnum> {
        match self {
            PlanNodeEnum::Access(AccessNodeEnum::Start(n)) => Some(LegacyPlanNodeEnum::Start(n.clone())),
            // ... 其他转换
        }
    }
}
```

---

## 第三阶段：运行时验证

### 3.1 循环依赖检测

#### 3.1.1 实现方案

```rust
// 文件：src/query/planning/plan/core/validators/cycle_detector.rs

use std::collections::{HashSet, VecDeque};
use crate::query::planning::plan::PlanNodeEnum;

/// 循环依赖检测器
pub struct CycleDetector {
    visited: HashSet<i64>,
    recursion_stack: HashSet<i64>,
}

impl CycleDetector {
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
            recursion_stack: HashSet::new(),
        }
    }

    /// 检测计划中是否存在循环依赖
    pub fn detect_cycle(root: &PlanNodeEnum) -> Result<(), PlanValidationError> {
        let mut detector = Self::new();
        detector.dfs(root)
    }

    fn dfs(&mut self, node: &PlanNodeEnum) -> Result<(), PlanValidationError> {
        let node_id = node.id();

        // 如果在递归栈中，说明检测到循环
        if self.recursion_stack.contains(&node_id) {
            return Err(PlanValidationError::CycleDetected {
                node_id,
                node_name: node.name().to_string(),
            });
        }

        // 如果已访问过，跳过
        if self.visited.contains(&node_id) {
            return Ok(());
        }

        self.visited.insert(node_id);
        self.recursion_stack.insert(node_id);

        // 递归检查所有子节点
        for child in node.children() {
            self.dfs(child)?;
        }

        self.recursion_stack.remove(&node_id);
        Ok(())
    }

    /// 使用迭代方式检测循环（避免栈溢出）
    pub fn detect_cycle_iterative(root: &PlanNodeEnum) -> Result<(), PlanValidationError> {
        let mut visited = HashSet::new();
        let mut stack = vec![(root.clone(), false)];

        while let Some((node, processed)) = stack.pop() {
            let node_id = node.id();

            if processed {
                // 后序处理，从递归栈中移除
                visited.remove(&node_id);
                continue;
            }

            if visited.contains(&node_id) {
                // 检测到循环
                return Err(PlanValidationError::CycleDetected {
                    node_id,
                    node_name: node.name().to_string(),
                });
            }

            visited.insert(node_id);
            stack.push((node.clone(), true));

            for child in node.children() {
                stack.push((child.clone(), false));
            }
        }

        Ok(())
    }
}
```

### 3.2 Schema 兼容性检查

#### 3.2.1 Schema 定义

```rust
// 文件：src/query/planning/plan/core/schema.rs

use std::collections::HashMap;

/// 列定义
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

/// Schema 定义
#[derive(Debug, Clone)]
pub struct Schema {
    columns: Vec<ColumnDef>,
    name_to_index: HashMap<String, usize>,
}

impl Schema {
    pub fn new(columns: Vec<ColumnDef>) -> Self {
        let name_to_index = columns
            .iter()
            .enumerate()
            .map(|(i, c)| (c.name.clone(), i))
            .collect();
        Self { columns, name_to_index }
    }

    pub fn columns(&self) -> &[ColumnDef] {
        &self.columns
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn get_column(&self, name: &str) -> Option<&ColumnDef> {
        self.name_to_index.get(name).map(|&i| &self.columns[i])
    }

    /// 检查两个 Schema 是否兼容（可以连接）
    pub fn is_compatible_with(&self, other: &Schema) -> bool {
        // 检查列名冲突
        for col in &self.columns {
            if other.get_column(&col.name).is_some() {
                // 列名冲突，需要重命名
                return false;
            }
        }
        true
    }

    /// 合并两个 Schema（用于 Join）
    pub fn merge_with(&self, other: &Schema) -> Schema {
        let mut columns = self.columns.clone();
        for col in &other.columns {
            if !self.name_to_index.contains_key(&col.name) {
                columns.push(col.clone());
            }
        }
        Schema::new(columns)
    }
}
```

#### 3.2.2 Schema 推导

```rust
// 文件：src/query/planning/plan/core/schema_inference.rs

use crate::query::planning::plan::PlanNodeEnum;

/// Schema 推导器
pub struct SchemaInference;

impl SchemaInference {
    /// 推导节点的输出 Schema
    pub fn infer_schema(node: &PlanNodeEnum) -> Result<Schema, PlanValidationError> {
        match node {
            PlanNodeEnum::Access(access) => Self::infer_access_schema(access),
            PlanNodeEnum::Operation(op) => Self::infer_operation_schema(op),
            PlanNodeEnum::Join(join) => Self::infer_join_schema(join),
            PlanNodeEnum::Traversal(traversal) => Self::infer_traversal_schema(traversal),
            // ... 其他类型
        }
    }

    fn infer_access_schema(node: &AccessNodeEnum) -> Result<Schema, PlanValidationError> {
        match node {
            AccessNodeEnum::ScanVertices(n) => {
                let columns = n.tag_props().iter().map(|tp| ColumnDef {
                    name: format!("{}.{}", n.tag_name(), tp.name()),
                    data_type: tp.data_type(),
                    nullable: true,
                }).collect();
                Ok(Schema::new(columns))
            }
            AccessNodeEnum::GetNeighbors(n) => {
                // ... 推导邻居节点 Schema
            }
            // ... 其他访问节点
        }
    }

    fn infer_operation_schema(node: &OperationNodeEnum) -> Result<Schema, PlanValidationError> {
        match node {
            OperationNodeEnum::Project(n) => {
                let input_schema = Self::infer_schema(n.input())?;
                let columns = n.columns().iter().map(|col| {
                    ColumnDef {
                        name: col.alias().unwrap_or_else(|| col.expr().to_string()),
                        data_type: Self::infer_expression_type(col.expr(), &input_schema)?,
                        nullable: true,
                    }
                }).collect();
                Ok(Schema::new(columns))
            }
            OperationNodeEnum::Filter(n) => {
                // Filter 不改变 Schema
                Self::infer_schema(n.input())
            }
            // ... 其他操作节点
        }
    }

    fn infer_join_schema(node: &JoinNodeEnum) -> Result<Schema, PlanValidationError> {
        match node {
            JoinNodeEnum::InnerJoin(n) => {
                let left_schema = Self::infer_schema(n.left_input())?;
                let right_schema = Self::infer_schema(n.right_input())?;
                Ok(left_schema.merge_with(&right_schema))
            }
            JoinNodeEnum::SemiJoin(n) => {
                // SemiJoin 只返回左表的列
                Self::infer_schema(n.left_input())
            }
            // ... 其他连接节点
        }
    }
}
```

#### 3.2.3 Schema 验证

```rust
// 文件：src/query/planning/plan/core/validators/schema_validator.rs

use crate::query::planning::plan::PlanNodeEnum;
use super::super::schema::Schema;
use super::super::schema_inference::SchemaInference;

/// Schema 验证器
pub struct SchemaValidator;

impl SchemaValidator {
    /// 验证整个计划树的 Schema 一致性
    pub fn validate(root: &PlanNodeEnum) -> Result<(), PlanValidationError> {
        Self::validate_node(root)
    }

    fn validate_node(node: &PlanNodeEnum) -> Result<(), PlanValidationError> {
        match node {
            PlanNodeEnum::Operation(OperationNodeEnum::Project(n)) => {
                let input_schema = SchemaInference::infer_schema(n.input())?;

                // 验证投影表达式中的列是否存在
                for col in n.columns() {
                    Self::validate_expression(col.expr(), &input_schema)?;
                }

                // 递归验证子节点
                Self::validate_node(n.input())
            }

            PlanNodeEnum::Operation(OperationNodeEnum::Filter(n)) => {
                let input_schema = SchemaInference::infer_schema(n.input())?;

                // 验证过滤条件中的列是否存在
                Self::validate_expression(n.condition(), &input_schema)?;

                // 递归验证子节点
                Self::validate_node(n.input())
            }

            PlanNodeEnum::Join(join) => {
                let left_schema = SchemaInference::infer_schema(Self::left_input(join))?;
                let right_schema = SchemaInference::infer_schema(Self::right_input(join))?;

                // 验证连接键是否存在于各自的 Schema 中
                for key in Self::hash_keys(join) {
                    Self::validate_expression(key, &left_schema)?;
                }
                for key in Self::probe_keys(join) {
                    Self::validate_expression(key, &right_schema)?;
                }

                // 递归验证子节点
                Self::validate_node(Self::left_input(join))?;
                Self::validate_node(Self::right_input(join))
            }

            // ... 其他节点类型的验证
            _ => Ok(()),
        }
    }

    fn validate_expression(
        expr: &ContextualExpression,
        schema: &Schema,
    ) -> Result<(), PlanValidationError> {
        // 遍历表达式中的所有列引用，检查是否存在于 Schema 中
        for col_ref in expr.column_references() {
            if schema.get_column(&col_ref).is_none() {
                return Err(PlanValidationError::ColumnNotFound {
                    column: col_ref,
                    available_columns: schema.columns().iter().map(|c| c.name.clone()).collect(),
                });
            }
        }
        Ok(())
    }
}
```

### 3.3 统一验证框架

```rust
// 文件：src/query/planning/plan/core/validators/mod.rs

pub mod cycle_detector;
pub mod schema_validator;

use crate::query::planning::plan::PlanNodeEnum;

/// 计划验证错误
#[derive(Debug, thiserror::Error)]
pub enum PlanValidationError {
    #[error("Cycle detected in plan: node {node_id} ({node_name})")]
    CycleDetected {
        node_id: i64,
        node_name: String,
    },

    #[error("Column not found: {column}. Available columns: {available_columns:?}")]
    ColumnNotFound {
        column: String,
        available_columns: Vec<String>,
    },

    #[error("Schema mismatch: {details}")]
    SchemaMismatch {
        details: String,
    },

    #[error("Invalid input count: expected {expected}, got {actual}")]
    InvalidInputCount {
        expected: usize,
        actual: usize,
    },
}

/// 计划验证器
pub struct PlanValidator {
    check_cycle: bool,
    check_schema: bool,
}

impl PlanValidator {
    pub fn new() -> Self {
        Self {
            check_cycle: true,
            check_schema: true,
        }
    }

    pub fn with_cycle_check(mut self, check: bool) -> Self {
        self.check_cycle = check;
        self
    }

    pub fn with_schema_check(mut self, check: bool) -> Self {
        self.check_schema = check;
        self
    }

    /// 验证执行计划
    pub fn validate(&self, plan: &PlanNodeEnum) -> Result<(), PlanValidationError> {
        if self.check_cycle {
            cycle_detector::CycleDetector::detect_cycle(plan)?;
        }

        if self.check_schema {
            schema_validator::SchemaValidator::validate(plan)?;
        }

        Ok(())
    }
}

impl Default for PlanValidator {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## 实施计划

### 阶段一：功能补全（预计 2-3 周）

| 任务                   | 优先级 | 预计时间 |
| ---------------------- | ------ | -------- |
| RightJoin 实现         | 高     | 2 天     |
| SemiJoin/AntiJoin 实现 | 高     | 3 天     |
| FullOuterJoin 完善     | 中     | 2 天     |
| BiExpand 实现          | 中     | 3 天     |
| BiTraverse 实现        | 中     | 3 天     |
| Apply 节点实现         | 高     | 4 天     |
| 事务控制节点           | 低     | 2 天     |

### 阶段二：架构优化（预计 1-2 周）

| 任务                  | 优先级 | 预计时间 |
| --------------------- | ------ | -------- |
| 宏定义统一            | 中     | 2 天     |
| PlanNodeEnum 分层重构 | 高     | 5 天     |
| 兼容性处理            | 中     | 2 天     |

### 阶段三：运行时验证（预计 1 周）

| 任务              | 优先级 | 预计时间 |
| ----------------- | ------ | -------- |
| 循环依赖检测      | 高     | 2 天     |
| Schema 定义和推导 | 高     | 3 天     |
| Schema 验证       | 中     | 2 天     |

---

## 测试计划

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_right_join_creation() {
        let left = create_test_scan_node();
        let right = create_test_scan_node();
        let join = RightJoinNode::new(
            left, right,
            vec![create_key("id")],
            vec![create_key("id")],
        ).unwrap();
        assert_eq!(join.name(), "RightJoinNode");
    }

    #[test]
    fn test_semi_join_optimization() {
        // 测试 EXISTS 子查询转换为 SemiJoin
    }

    #[test]
    fn test_cycle_detection() {
        // 构造一个有循环的计划
        let plan = create_cyclic_plan();
        let result = CycleDetector::detect_cycle(&plan);
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_validation() {
        // 测试 Schema 验证
    }
}
```

### 集成测试

```rust
#[test]
fn test_biexpand_shortest_path() {
    // 测试双向扩展查找最短路径
    let query = "MATCH p = (a)-[*]-(b) WHERE a.id = 1 AND b.id = 100 RETURN p";
    let plan = planner.plan(query).unwrap();

    // 验证计划包含 BiExpand 节点
    assert!(plan.contains_node_type("BiExpand"));

    // 执行并验证结果
    let result = executor.execute(plan).unwrap();
    assert!(result.len() > 0);
}
```

---

## 风险评估

| 风险                              | 影响 | 缓解措施                   |
| --------------------------------- | ---- | -------------------------- |
| PlanNodeEnum 重构导致大量代码修改 | 高   | 提供兼容层，逐步迁移       |
| 新节点执行器实现复杂度            | 中   | 参考现有实现，编写充分测试 |
| Schema 推导不完整                 | 中   | 先实现核心节点，逐步完善   |
| 性能影响                          | 低   | 验证逻辑可配置开关         |

---

## 总结

本改进方案系统性地解决了 PlanNode 体系的三个主要问题：

1. **功能补全**：补充 RightJoin、SemiJoin、AntiJoin、BiExpand、BiTraverse、Apply 等关键节点
2. **架构优化**：统一宏定义，分层重构 PlanNodeEnum
3. **运行时验证**：实现循环依赖检测和 Schema 兼容性检查

通过分阶段实施，可以在保证系统稳定性的同时，逐步提升查询计划体系的完整性和健壮性。

---

## 实施状态

### 已完成 ✅

| 阶段 | 任务              | 状态      | 完成日期   |
| ---- | ----------------- | --------- | ---------- |
| 1.1  | RightJoin 节点    | ✅ 已完成 | 2026-05-01 |
| 1.2  | SemiJoin 节点     | ✅ 已完成 | 2026-05-01 |
| 1.3  | AntiJoin 节点     | ✅ 已完成 | 2026-05-01 |
| 1.4  | BiExpand 节点     | ✅ 已完成 | 2026-05-01 |
| 1.5  | BiTraverse 节点   | ✅ 已完成 | 2026-05-01 |
| 1.6  | Apply 节点        | ✅ 已完成 | 2026-05-01 |
| 1.7  | 事务控制节点      | ✅ 已完成 | 2026-05-01 |
| 2.1  | 宏定义统一        | ✅ 已完成 | 2026-05-01 |
| 3.1  | 循环检测验证器    | ✅ 已完成 | 2026-05-01 |
| 3.2  | Schema 兼容性验证 | ✅ 已完成 | 2026-05-01 |

### 待定 ⏳

| 阶段 | 任务                  | 状态    | 说明                   |
| ---- | --------------------- | ------- | ---------------------- |
| 2.2  | PlanNodeEnum 分层重构 | ⏳ 暂缓 | 风险较高，需进一步评估 |

### 新增文件

- `src/query/planning/plan/validation/mod.rs` - 验证模块入口
- `src/query/planning/plan/validation/cycle_detection.rs` - 循环检测器
- `src/query/planning/plan/validation/schema_validation.rs` - Schema 验证器

### 修改文件

- `src/query/planning/plan/core/nodes/join/join_node.rs` - 添加 RightJoin、SemiJoin、AntiJoin
- `src/query/planning/plan/core/nodes/traversal/traversal_node.rs` - 添加 BiExpand、BiTraverse
- `src/query/planning/plan/core/nodes/graph_operations/graph_operations_node.rs` - 添加 Apply 节点
- `src/query/planning/plan/core/nodes/control_flow/control_flow_node.rs` - 添加事务控制节点
- `src/query/planning/plan/core/nodes/base/macros.rs` - 统一宏定义
- `src/query/planning/plan/core/nodes/base/plan_node_enum.rs` - 添加新节点枚举
- `src/query/executor/factory/builders/control_flow_builder.rs` - 添加事务执行器构建
