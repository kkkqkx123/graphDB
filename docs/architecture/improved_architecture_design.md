# 改进后的查询模块架构设计

## 一、设计目标

本文档描述 GraphDB 查询模块改进后的架构设计。改进设计旨在解决当前架构中存在的问题，包括枚举定义碎片化、处理链条不完整、动态分发性能问题等，通过系统性的架构优化提升代码的可维护性、可扩展性和系统稳定性。

改进设计的核心目标包括：

1. **统一类型管理**：建立从 AST 到执行器的统一操作类型体系
2. **完整性保障**：确保处理链条的每个环节都能正确处理所有支持的查询类型
3. **性能优化**：完成静态分发改造，消除不必要的运行时开销
4. **可维护性提升**：通过访问者模式统一遍历逻辑，减少重复代码
5. **可扩展性增强**：支持方便地添加新的语句类型、计划节点和执行器

## 二、总体架构

### 2.1 改进后的架构概览

改进后的查询模块架构采用分层设计，从上到下依次为：

```
┌─────────────────────────────────────────────────────────────────┐
│                     Presentation Layer                          │
│                    （用户接口层 - CLI/REST）                      │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      Query Pipeline                             │
│                    （查询处理管道）                               │
├─────────────────────────────────────────────────────────────────┤
│  Parser → Validator → Planner → Optimizer → Executor → Scheduler │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    Storage Layer                                │
│                    （存储引擎层）                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 核心组件关系

改进后的架构定义了以下核心组件：

**类型系统组件**

- `CoreOperationKind`：核心操作类型枚举，统一所有模块的操作类型
- `PlanNodeVisitor`：统一的计划节点访问者接口
- `AstVisitor`：统一的 AST 访问者接口

**处理管道组件**

- `QueryPipeline`：查询处理管道，协调各阶段的处理
- `QueryContext`：查询执行上下文，传递各阶段的状态信息
- `ExecutionPlan`：执行计划，包含优化后的计划节点序列

**执行组件**

- `ExecutorEnum`：执行器枚举，替代 `Box<dyn Executor<S>>`
- `QueryScheduler`：查询调度器，管理执行器的执行顺序
- `ExecutionEngine`：执行引擎，负责具体的查询执行

### 2.3 架构改进要点

**改进一：统一类型定义**

```
之前：
  Stmt (25) → StatementType (35) → SentenceKind (~30) → PlanNodeEnum (60)

之后：
  CoreOperationKind (~50) → PlanNodeEnum (60) → ExecutorEnum (40)
```

改进后，操作类型定义更加集中，减少了重复和不一致。

**改进二：处理链条完整性**

```
之前：
  Parser (完整) → Validator (不完整) → Planner (基本完整) → Optimizer (部分) → Executor (部分) → Scheduler (简单)

之后：
  Parser (完整) → Validator (完整) → Planner (完整) → Optimizer (完整) → Executor (完整) → Scheduler (增强)
```

改进后，处理链条的每个环节都完整可用。

**改进三：静态分发**

```
之前：
  InputExecutor: Box<dyn Executor<S>>（动态分发）
  ChainableExecutor: Box<dyn Executor<S>>（动态分发）

之后：
  InputExecutor: ExecutorEnum<S>（静态分发）
  ChainableExecutor: ExecutorEnum<S>（静态分发）
```

改进后，执行器调用使用静态分发，消除了虚函数调用的开销。

**改进四：访问者模式**

```
之前：
  ExpressionVisitor（已有）
  PlanNodeVisitor（缺失）

之后：
  ExpressionVisitor（已有）
  PlanNodeVisitor（新增）
  AstVisitor（新增）
```

改进后，遍历和处理 AST/计划节点更加统一和简洁。

## 三、核心类型定义

### 3.1 CoreOperationKind 枚举

`CoreOperationKind` 是查询系统的核心操作类型枚举，定义了所有支持的查询操作：

```rust
/// 核心操作类型枚举 - 查询系统的类型基础
///
/// 此枚举统一了查询系统中的所有操作类型，贯穿 Parser、Validator、Planner、Optimizer 和 Executor 五个模块。
/// 通过统一的类型定义，减少了各模块之间的类型映射复杂性，提高了代码的可维护性。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreOperationKind {
    // ==================== 数据查询操作 ====================
    
    /// MATCH 查询 - 图模式匹配查询
    Match,
    
    /// GO 查询 - 简单的图遍历查询
    Go,
    
    /// LOOKUP 查询 - 基于索引的查找查询
    Lookup,
    
    /// FIND PATH 查询 - 查找两点之间的路径
    FindPath,
    
    /// GET SUBGRAPH 查询 - 获取子图
    GetSubgraph,
    
    // ==================== 数据访问操作 ====================
    
    /// 扫描所有顶点
    ScanVertices,
    
    /// 扫描所有边
    ScanEdges,
    
    /// 获取指定顶点
    GetVertices,
    
    /// 获取指定边
    GetEdges,
    
    /// 获取邻居节点
    GetNeighbors,
    
    // ==================== 数据转换操作 ====================
    
    /// 项目操作 - 选择输出列
    Project,
    
    /// 过滤操作 - 根据条件筛选行
    Filter,
    
    /// 排序操作 - 对结果排序
    Sort,
    
    /// 限制操作 - 限制返回行数
    Limit,
    
    /// TopN 操作 - 获取前 N 行
    TopN,
    
    /// 采样操作 - 随机采样
    Sample,
    
    /// 展开操作 - 将数组展开为行
    Unwind,
    
    // ==================== 数据聚合操作 ====================
    
    /// 聚合操作 - 分组聚合
    Aggregate,
    
    /// 分组操作 - GROUP BY
    GroupBy,
    
    /// HAVING 操作 - 分组后过滤
    Having,
    
    /// 去重操作 - 去除重复行
    Dedup,
    
    // ==================== 连接操作 ====================
    
    /// 内连接 - INNER JOIN
    InnerJoin,
    
    /// 左连接 - LEFT JOIN
    LeftJoin,
    
    /// 交叉连接 - CROSS JOIN
    CrossJoin,
    
    /// 哈希连接 - HASH JOIN
    HashJoin,
    
    // ==================== 图遍历操作 ====================
    
    /// 遍历操作 - 广度优先遍历
    Traverse,
    
    /// 扩展操作 - 扩展到邻居节点
    Expand,
    
    /// 全扩展操作 - 扩展到所有层级的邻居
    ExpandAll,
    
    /// 最短路径 - 单源最短路径
    ShortestPath,
    
    /// 所有路径 - 查找所有路径
    AllPaths,
    
    // ==================== 数据修改操作 ====================
    
    /// 插入操作 - INSERT
    Insert,
    
    /// 更新操作 - UPDATE
    Update,
    
    /// 删除操作 - DELETE
    Delete,
    
    /// 合并操作 - MERGE
    Merge,
    
    // ==================== 模式匹配操作 ====================
    
    /// 模式应用 - PATTERN APPLY
    PatternApply,
    
    /// 卷起应用 - ROLL UP APPLY
    RollUpApply,
    
    // ==================== 循环控制操作 ====================
    
    /// 循环 - LOOP
    Loop,
    
    /// FOR 循环 - FOR LOOP
    ForLoop,
    
    /// WHILE 循环 - WHILE LOOP
    WhileLoop,
    
    // ==================== 空间管理操作 ====================
    
    /// 创建空间 - CREATE SPACE
    CreateSpace,
    
    /// 删除空间 - DROP SPACE
    DropSpace,
    
    /// 描述空间 - DESCRIBE SPACE
    DescribeSpace,
    
    /// 使用空间 - USE SPACE
    UseSpace,
    
    // ==================== 标签管理操作 ====================
    
    /// 创建标签 - CREATE TAG
    CreateTag,
    
    /// 修改标签 - ALTER TAG
    AlterTag,
    
    /// 删除标签 - DROP TAG
    DropTag,
    
    /// 描述标签 - DESCRIBE TAG
    DescribeTag,
    
    // ==================== 边类型管理操作 ====================
    
    /// 创建边类型 - CREATE EDGE
    CreateEdge,
    
    /// 修改边类型 - ALTER EDGE
    AlterEdge,
    
    /// 删除边类型 - DROP EDGE
    DropEdge,
    
    /// 描述边类型 - DESCRIBE EDGE
    DescribeEdge,
    
    // ==================== 索引管理操作 ====================
    
    /// 创建索引 - CREATE INDEX
    CreateIndex,
    
    /// 删除索引 - DROP INDEX
    DropIndex,
    
    /// 描述索引 - DESCRIBE INDEX
    DescribeIndex,
    
    /// 重建索引 - REBUILD INDEX
    RebuildIndex,
    
    /// 全文索引扫描 - FULLTEXT INDEX SCAN
    FulltextIndexScan,
    
    // ==================== 用户管理操作 ====================
    
    /// 创建用户 - CREATE USER
    CreateUser,
    
    /// 修改用户 - ALTER USER
    AlterUser,
    
    /// 删除用户 - DROP USER
    DropUser,
    
    /// 修改密码 - CHANGE PASSWORD
    ChangePassword,
    
    // ==================== 其他操作 ====================
    
    /// 设置操作 - SET
    Set,
    
    /// 解释执行 - EXPLAIN
    Explain,
    
    /// 显示操作 - SHOW
    Show,
    
    /// 分配操作 - ASSIGNMENT
    Assignment,
}

impl CoreOperationKind {
    /// 获取操作类别的名称
    pub fn category(&self) -> &'static str {
        match self {
            // 数据查询
            Self::Match | Self::Go | Self::Lookup | Self::FindPath | Self::GetSubgraph => "DATA_QUERY",
            
            // 数据访问
            Self::ScanVertices | Self::ScanEdges | Self::GetVertices | Self::GetEdges | Self::GetNeighbors => "DATA_ACCESS",
            
            // 数据转换
            Self::Project | Self::Filter | Self::Sort | Self::Limit | Self::TopN | Self::Sample | Self::Unwind => "DATA_TRANSFORMATION",
            
            // 数据聚合
            Self::Aggregate | Self::GroupBy | Self::Having | Self::Dedup => "DATA_AGGREGATION",
            
            // 连接操作
            Self::InnerJoin | Self::LeftJoin | Self::CrossJoin | Self::HashJoin => "JOIN",
            
            // 图遍历
            Self::Traverse | Self::Expand | Self::ExpandAll | Self::ShortestPath | Self::AllPaths => "GRAPH_TRAVERSAL",
            
            // 数据修改
            Self::Insert | Self::Update | Self::Delete | Self::Merge => "DATA_MODIFICATION",
            
            // 模式匹配
            Self::PatternApply | Self::RollUpApply => "PATTERN_MATCHING",
            
            // 循环控制
            Self::Loop | Self::ForLoop | Self::WhileLoop => "LOOP_CONTROL",
            
            // 空间管理
            Self::CreateSpace | Self::DropSpace | Self::DescribeSpace | Self::UseSpace => "SPACE_MANAGEMENT",
            
            // 标签管理
            Self::CreateTag | Self::AlterTag | Self::DropTag | Self::DescribeTag => "TAG_MANAGEMENT",
            
            // 边类型管理
            Self::CreateEdge | Self::AlterEdge | Self::DropEdge | Self::DescribeEdge => "EDGE_MANAGEMENT",
            
            // 索引管理
            Self::CreateIndex | Self::DropIndex | Self::DescribeIndex | Self::RebuildIndex | Self::FulltextIndexScan => "INDEX_MANAGEMENT",
            
            // 用户管理
            Self::CreateUser | Self::AlterUser | Self::DropUser | Self::ChangePassword => "USER_MANAGEMENT",
            
            // 其他
            Self::Set | Self::Explain | Self::Show | Self::Assignment => "OTHER",
        }
    }
    
    /// 判断是否为只读操作
    pub fn is_read_only(&self) -> bool {
        matches!(
            self,
            Self::Match | Self::Go | Self::Lookup | Self::FindPath | Self::GetSubgraph
                | Self::ScanVertices | Self::ScanEdges | Self::GetVertices | Self::GetEdges | Self::GetNeighbors
                | Self::Project | Self::Filter | Self::Sort | Self::Limit | Self::TopN | Self::Sample | Self::Unwind
                | Self::Aggregate | Self::GroupBy | Self::Having | Self::Dedup
                | Self::InnerJoin | Self::LeftJoin | Self::CrossJoin | Self::HashJoin
                | Self::Traverse | Self::Expand | Self::ExpandAll | Self::ShortestPath | Self::AllPaths
                | Self::DescribeSpace | Self::DescribeTag | Self::DescribeEdge | Self::DescribeIndex
                | Self::Show | Self::Explain
        )
    }
    
    /// 判断是否为元数据操作
    pub fn is_metadata_operation(&self) -> bool {
        matches!(
            self,
            Self::CreateSpace | Self::DropSpace | Self::DescribeSpace | Self::UseSpace
                | Self::CreateTag | Self::AlterTag | Self::DropTag | Self::DescribeTag
                | Self::CreateEdge | Self::AlterEdge | Self::DropEdge | Self::DescribeEdge
                | Self::CreateIndex | Self::DropIndex | Self::DescribeIndex | Self::RebuildIndex
                | Self::CreateUser | Self::AlterUser | Self::DropUser | Self::ChangePassword
                | Self::Show | Self::Explain
        )
    }
}
```

### 3.2 类型转换实现

为各模块的枚举实现与 `CoreOperationKind` 的转换：

```rust
// 从 Stmt 到 CoreOperationKind 的转换
impl From<&Stmt> for CoreOperationKind {
    fn from(stmt: &Stmt) -> Self {
        match stmt {
            Stmt::Match(_) => CoreOperationKind::Match,
            Stmt::Go(_) => CoreOperationKind::Go,
            Stmt::Lookup(_) => CoreOperationKind::Lookup,
            Stmt::FindPath(_) => CoreOperationKind::FindPath,
            Stmt::Subgraph(_) => CoreOperationKind::GetSubgraph,
            Stmt::Insert(_) => CoreOperationKind::Insert,
            Stmt::Update(_) => CoreOperationKind::Update,
            Stmt::Delete(_) => CoreOperationKind::Delete,
            // ... 其他语句类型的转换
        }
    }
}

// 从 PlanNodeEnum 到 CoreOperationKind 的转换
impl From<&PlanNodeEnum> for CoreOperationKind {
    fn from(node: &PlanNodeEnum) -> Self {
        match node {
            PlanNodeEnum::Start(_) => CoreOperationKind::ScanVertices,
            PlanNodeEnum::Project(_) => CoreOperationKind::Project,
            PlanNodeEnum::Filter(_) => CoreOperationKind::Filter,
            PlanNodeEnum::Sort(_) => CoreOperationKind::Sort,
            PlanNodeEnum::Limit(_) => CoreOperationKind::Limit,
            // ... 其他计划节点的转换
        }
    }
}

// 从 ExecutorEnum 到 CoreOperationKind 的转换
impl<S: StorageEngine + Send + 'static> From<&ExecutorEnum<S>> for CoreOperationKind {
    fn from(exec: &ExecutorEnum<S>) -> Self {
        match exec {
            ExecutorEnum::Project(_) => CoreOperationKind::Project,
            ExecutorEnum::Filter(_) => CoreOperationKind::Filter,
            ExecutorEnum::Sort(_) => CoreOperationKind::Sort,
            ExecutorEnum::Limit(_) => CoreOperationKind::Limit,
            // ... 其他执行器的转换
        }
    }
}
```

## 四、PlanNodeVisitor 设计

### 4.1 访问者接口定义

```rust
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::admin_node::*;
use crate::query::planner::plan::core::nodes::data_access_node::*;
use crate::query::planner::plan::core::nodes::data_modification_node::*;
use crate::query::planner::plan::core::nodes::dql_node::*;

/// PlanNode 访问者 trait
///
/// 提供统一的 PlanNode 遍历接口，简化优化规则和数据转换的实现。
/// 访问者模式使得可以在不修改节点结构的情况下对节点进行操作。
pub trait PlanNodeVisitor {
    /// 访问结果的类型
    type Result;
    
    /// 访问开始节点
    fn visit_start(&mut self, node: &StartNode) -> Self::Result;
    
    /// 访问项目节点
    fn visit_project(&mut self, node: &ProjectNode) -> Self::Result;
    
    /// 访问过滤节点
    fn visit_filter(&mut self, node: &FilterNode) -> Self::Result;
    
    /// 访问排序节点
    fn visit_sort(&mut self, node: &SortNode) -> Self::Result;
    
    /// 访问限制节点
    fn visit_limit(&mut self, node: &LimitNode) -> Self::Result;
    
    /// 访问 TopN 节点
    fn visit_topn(&mut self, node: &TopNNode) -> Self::Result;
    
    /// 访问采样节点
    fn visit_sample(&mut self, node: &SampleNode) -> Self::Result;
    
    /// 访问去重节点
    fn visit_dedup(&mut self, node: &DedupNode) -> Self::Result;
    
    /// 访问获取顶点节点
    fn visit_get_vertices(&mut self, node: &GetVerticesNode) -> Self::Result;
    
    /// 访问获取邻居节点
    fn visit_get_neighbors(&mut self, node: &GetNeighborsNode) -> Self::Result;
    
    /// 访问扫描顶点节点
    fn visit_scan_vertices(&mut self, node: &ScanVerticesNode) -> Self::Result;
    
    /// 访问扩展节点
    fn visit_expand(&mut self, node: &ExpandNode) -> Self::Result;
    
    /// 访问全扩展节点
    fn visit_expand_all(&mut self, node: &ExpandAllNode) -> Self::Result;
    
    /// 访问遍历节点
    fn visit_traverse(&mut self, node: &TraverseNode) -> Self::Result;
    
    /// 访问内连接节点
    fn visit_inner_join(&mut self, node: &InnerJoinNode) -> Self::Result;
    
    /// 访问左连接节点
    fn visit_left_join(&mut self, node: &LeftJoinNode) -> Self::Result;
    
    /// 访问交叉连接节点
    fn visit_cross_join(&mut self, node: &CrossJoinNode) -> Self::Result;
    
    /// 访问哈希内连接节点
    fn visit_hash_inner_join(&mut self, node: &HashInnerJoinNode) -> Self::Result;
    
    /// 访问聚合节点
    fn visit_aggregate(&mut self, node: &AggregateNode) -> Self::Result;
    
    /// 访问分组节点
    fn visit_group_by(&mut self, node: &GroupByNode) -> Self::Result;
    
    /// 访问 Having 节点
    fn visit_having(&mut self, node: &HavingNode) -> Self::Result;
    
    /// 访问展开节点
    fn visit_unwind(&mut self, node: &UnwindNode) -> Self::Result;
    
    /// 访问追加顶点节点
    fn visit_append_vertices(&mut self, node: &AppendVerticesNode) -> Self::Result;
    
    /// 访问模式应用节点
    fn visit_pattern_apply(&mut self, node: &PatternApplyNode) -> Self::Result;
    
    /// 访问卷起应用节点
    fn visit_rollup_apply(&mut self, node: &RollUpApplyNode) -> Self::Result;
    
    /// 访问循环节点
    fn visit_loop(&mut self, node: &LoopNode) -> Self::Result;
    
    /// 访问 For 循环节点
    fn visit_for_loop(&mut self, node: &ForLoopNode) -> Self::Result;
    
    /// 访问 While 循环节点
    fn visit_while_loop(&mut self, node: &WhileLoopNode) -> Self::Result;
    
    /// 访问分配节点
    fn visit_assign(&mut self, node: &AssignNode) -> Self::Result;
    
    /// 访问创建空间节点
    fn visit_create_space(&mut self, node: &CreateSpaceNode) -> Self::Result;
    
    /// 访问删除空间节点
    fn visit_drop_space(&mut self, node: &DropSpaceNode) -> Self::Result;
    
    /// 访问描述空间节点
    fn visit_desc_space(&mut self, node: &DescSpaceNode) -> Self::Result;
    
    /// 访问创建标签节点
    fn visit_create_tag(&mut self, node: &CreateTagNode) -> Self::Result;
    
    /// 访问修改标签节点
    fn visit_alter_tag(&mut self, node: &AlterTagNode) -> Self::Result;
    
    /// 访问删除标签节点
    fn visit_drop_tag(&mut self, node: &DropTagNode) -> Self::Result;
    
    /// 访问创建边节点
    fn visit_create_edge(&mut self, node: &CreateEdgeNode) -> Self::Result;
    
    /// 访问修改边节点
    fn visit_alter_edge(&mut self, node: &AlterEdgeNode) -> Self::Result;
    
    /// 访问删除边节点
    fn visit_drop_edge(&mut self, node: &DropEdgeNode) -> Self::Result;
    
    /// 访问插入顶点节点
    fn visit_insert_vertex(&mut self, node: &InsertVertexNode) -> Self::Result;
    
    /// 访问插入边节点
    fn visit_insert_edge(&mut self, node: &InsertEdgeNode) -> Self::Result;
    
    /// 访问更新节点
    fn visit_update(&mut self, node: &UpdateNode) -> Self::Result;
    
    /// 访问最短路径节点
    fn visit_shortest_path(&mut self, node: &ShortestPathNode) -> Self::Result;
    
    /// 访问所有路径节点
    fn visit_all_paths(&mut self, node: &AllPathsNode) -> Self::Result;
    
    /// 访问最短路径（多源）节点
    fn visit_multi_shortest_path(&mut self, node: &MultiShortestPathNode) -> Self::Result;
    
    /// 访问求差节点
    fn visit_minus(&mut self, node: &MinusNode) -> Self::Result;
    
    /// 访问交集节点
    fn visit_intersect(&mut self, node: &IntersectNode) -> Self::Result;
    
    /// 访问并集节点
    fn visit_union(&mut self, node: &UnionNode) -> Self::Result;
    
    /// 访问全并集节点
    fn visit_union_all(&mut self, node: &UnionAllNode) -> Self::Result;
    
    /// 统一的访问入口
    fn visit(&mut self, node: &PlanNodeEnum) -> Self::Result {
        match node {
            PlanNodeEnum::Start(n) => self.visit_start(n),
            PlanNodeEnum::Project(n) => self.visit_project(n),
            PlanNodeEnum::Filter(n) => self.visit_filter(n),
            PlanNodeEnum::Sort(n) => self.visit_sort(n),
            PlanNodeEnum::Limit(n) => self.visit_limit(n),
            PlanNodeEnum::TopN(n) => self.visit_topn(n),
            PlanNodeEnum::Sample(n) => self.visit_sample(n),
            PlanNodeEnum::Dedup(n) => self.visit_dedup(n),
            PlanNodeEnum::GetVertices(n) => self.visit_get_vertices(n),
            PlanNodeEnum::GetNeighbors(n) => self.visit_get_neighbors(n),
            PlanNodeEnum::ScanVertices(n) => self.visit_scan_vertices(n),
            PlanNodeEnum::Expand(n) => self.visit_expand(n),
            PlanNodeEnum::ExpandAll(n) => self.visit_expand_all(n),
            PlanNodeEnum::Traverse(n) => self.visit_traverse(n),
            PlanNodeEnum::InnerJoin(n) => self.visit_inner_join(n),
            PlanNodeEnum::LeftJoin(n) => self.visit_left_join(n),
            PlanNodeEnum::CrossJoin(n) => self.visit_cross_join(n),
            PlanNodeEnum::HashInnerJoin(n) => self.visit_hash_inner_join(n),
            PlanNodeEnum::Aggregate(n) => self.visit_aggregate(n),
            PlanNodeEnum::GroupBy(n) => self.visit_group_by(n),
            PlanNodeEnum::Having(n) => self.visit_having(n),
            PlanNodeEnum::Unwind(n) => self.visit_unwind(n),
            PlanNodeEnum::AppendVertices(n) => self.visit_append_vertices(n),
            PlanNodeEnum::PatternApply(n) => self.visit_pattern_apply(n),
            PlanNodeEnum::RollUpApply(n) => self.visit_rollup_apply(n),
            PlanNodeEnum::Loop(n) => self.visit_loop(n),
            PlanNodeEnum::ForLoop(n) => self.visit_for_loop(n),
            PlanNodeEnum::WhileLoop(n) => self.visit_while_loop(n),
            PlanNodeEnum::Assign(n) => self.visit_assign(n),
            PlanNodeEnum::CreateSpace(n) => self.visit_create_space(n),
            PlanNodeEnum::DropSpace(n) => self.visit_drop_space(n),
            PlanNodeEnum::DescSpace(n) => self.visit_desc_space(n),
            PlanNodeEnum::CreateTag(n) => self.visit_create_tag(n),
            PlanNodeEnum::AlterTag(n) => self.visit_alter_tag(n),
            PlanNodeEnum::DropTag(n) => self.visit_drop_tag(n),
            PlanNodeEnum::CreateEdge(n) => self.visit_create_edge(n),
            PlanNodeEnum::AlterEdge(n) => self.visit_alter_edge(n),
            PlanNodeEnum::DropEdge(n) => self.visit_drop_edge(n),
            PlanNodeEnum::InsertVertex(n) => self.visit_insert_vertex(n),
            PlanNodeEnum::InsertEdge(n) => self.visit_insert_edge(n),
            PlanNodeEnum::Update(n) => self.visit_update(n),
            PlanNodeEnum::ShortestPath(n) => self.visit_shortest_path(n),
            PlanNodeEnum::AllPaths(n) => self.visit_all_paths(n),
            PlanNodeEnum::MultiShortestPath(n) => self.visit_multi_shortest_path(n),
            PlanNodeEnum::Minus(n) => self.visit_minus(n),
            PlanNodeEnum::Intersect(n) => self.visit_intersect(n),
            PlanNodeEnum::Union(n) => self.visit_union(n),
            PlanNodeEnum::UnionAll(n) => self.visit_union_all(n),
        }
    }
}
```

### 4.2 默认访问者实现

```rust
/// 默认的 PlanNode 访问者实现
///
/// 提供所有访问方法的默认实现，子类可以只重写需要定制的方法。
pub struct DefaultPlanNodeVisitor;

impl DefaultPlanNodeVisitor {
    pub fn new() -> Self {
        Self
    }
}

impl PlanNodeVisitor for DefaultPlanNodeVisitor {
    type Result = ();
    
    fn visit_start(&mut self, _node: &StartNode) {}
    fn visit_project(&mut self, _node: &ProjectNode) {}
    fn visit_filter(&mut self, _node: &FilterNode) {}
    fn visit_sort(&mut self, _node: &SortNode) {}
    fn visit_limit(&mut self, _node: &LimitNode) {}
    fn visit_topn(&mut self, _node: &TopNNode) {}
    fn visit_sample(&mut self, _node: &SampleNode) {}
    fn visit_dedup(&mut self, _node: &DedupNode) {}
    fn visit_get_vertices(&mut self, _node: &GetVerticesNode) {}
    fn visit_get_neighbors(&mut self, _node: &GetNeighborsNode) {}
    fn visit_scan_vertices(&mut self, _node: &ScanVerticesNode) {}
    fn visit_expand(&mut self, _node: &ExpandNode) {}
    fn visit_expand_all(&mut self, _node: &ExpandAllNode) {}
    fn visit_traverse(&mut self, _node: &TraverseNode) {}
    fn visit_inner_join(&mut self, _node: &InnerJoinNode) {}
    fn visit_left_join(&mut self, _node: &LeftJoinNode) {}
    fn visit_cross_join(&mut self, _node: &CrossJoinNode) {}
    fn visit_hash_inner_join(&mut self, _node: &HashInnerJoinNode) {}
    fn visit_aggregate(&mut self, _node: &AggregateNode) {}
    fn visit_group_by(&mut self, _node: &GroupByNode) {}
    fn visit_having(&mut self, _node: &HavingNode) {}
    fn visit_unwind(&mut self, _node: &UnwindNode) {}
    fn visit_append_vertices(&mut self, _node: &AppendVerticesNode) {}
    fn visit_pattern_apply(&mut self, _node: &PatternApplyNode) {}
    fn visit_rollup_apply(&mut self, _node: &RollUpApplyNode) {}
    fn visit_loop(&mut self, _node: &LoopNode) {}
    fn visit_for_loop(&mut self, _node: &ForLoopNode) {}
    fn visit_while_loop(&mut self, _node: &WhileLoopNode) {}
    fn visit_assign(&mut self, _node: &AssignNode) {}
    fn visit_create_space(&mut self, _node: &CreateSpaceNode) {}
    fn visit_drop_space(&mut self, _node: &DropSpaceNode) {}
    fn visit_desc_space(&mut self, _node: &DescSpaceNode) {}
    fn visit_create_tag(&mut self, _node: &CreateTagNode) {}
    fn visit_alter_tag(&mut self, _node: &AlterTagNode) {}
    fn visit_drop_tag(&mut self, _node: &DropTagNode) {}
    fn visit_create_edge(&mut self, _node: &CreateEdgeNode) {}
    fn visit_alter_edge(&mut self, _node: &AlterEdgeNode) {}
    fn visit_drop_edge(&mut self, _node: &DropEdgeNode) {}
    fn visit_insert_vertex(&mut self, _node: &InsertVertexNode) {}
    fn visit_insert_edge(&mut self, _node: &InsertEdgeNode) {}
    fn visit_update(&mut self, _node: &UpdateNode) {}
    fn visit_shortest_path(&mut self, _node: &ShortestPathNode) {}
    fn visit_all_paths(&mut self, _node: &AllPathsNode) {}
    fn visit_multi_shortest_path(&mut self, _node: &MultiShortestPathNode) {}
    fn visit_minus(&mut self, _node: &MinusNode) {}
    fn visit_intersect(&mut self, _node: &IntersectNode) {}
    fn visit_union(&mut self, _node: &UnionNode) {}
    fn visit_union_all(&mut self, _node: &UnionAllNode) {}
}
```

### 4.3 使用示例

```rust
/// 统计计划节点类型的访问者
pub struct NodeTypeCounter {
    counts: HashMap<&'static str, usize>,
}

impl NodeTypeCounter {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }
    
    pub fn get_counts(&self) -> &HashMap<&'static str, usize> {
        &self.counts
    }
}

impl PlanNodeVisitor for NodeTypeCounter {
    type Result = ();
    
    fn visit_start(&mut self, node: &StartNode) {
        *self.counts.entry("Start").or_insert(0) += 1;
    }
    
    fn visit_project(&mut self, node: &ProjectNode) {
        *self.counts.entry("Project").or_insert(0) += 1;
    }
    
    fn visit_filter(&mut self, node: &FilterNode) {
        *self.counts.entry("Filter").or_insert(0) += 1;
    }
    
    // ... 其他节点类型的统计
    
    fn visit(&mut self, node: &PlanNodeEnum) -> Self::Result {
        // 使用统一的访问入口
        PlanNodeVisitor::visit(self, node);
    }
}

// 使用示例
let plan = /* 获取执行计划 */;
let mut counter = NodeTypeCounter::new();
for node in plan.nodes() {
    counter.visit(node);
}
println!("节点类型统计: {:?}", counter.get_counts());
```

## 五、静态分发改造

### 5.1 InputExecutor 改造

```rust
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::storage::StorageEngine;

/// 输入执行器 trait - 统一输入处理机制
///
/// 需要访问输入数据的执行器应实现此 trait。
/// 使用 ExecutorEnum 替代 Box<dyn Executor<S>>，实现静态分发。
pub trait InputExecutor<S: StorageEngine> {
    /// 设置输入数据
    fn set_input(&mut self, input: ExecutorEnum<S>);
    
    /// 获取输入数据
    fn get_input(&self) -> Option<&ExecutorEnum<S>>;
    
    /// 获取可变的输入数据
    fn get_input_mut(&mut self) -> Option<&mut ExecutorEnum<S>>;
}

/// 可链式执行的执行器 trait
///
/// 支持链式组合的执行器可以实现此 trait。
pub trait ChainableExecutor<S: StorageEngine + Send + 'static>:
    Executor<S> + InputExecutor<S>
{
    /// 链式执行 - 将当前执行器与下一个执行器链接
    fn chain(mut self, next: ExecutorEnum<S>) -> ExecutorEnum<S>
    where
        Self: Sized + 'static,
    {
        self.set_input(next);
        ExecutorEnum::from(self)
    }
}
```

### 5.2 ExecutorEnum 实现 InputExecutor

```rust
#[async_trait]
impl<S: StorageEngine + Send + 'static> InputExecutor<S> for ExecutorEnum<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        match self {
            ExecutorEnum::Filter(exec) => exec.set_input(input),
            ExecutorEnum::Project(exec) => exec.set_input(input),
            ExecutorEnum::Limit(exec) => exec.set_input(input),
            ExecutorEnum::Sort(exec) => exec.set_input(input),
            ExecutorEnum::TopN(exec) => exec.set_input(input),
            ExecutorEnum::Sample(exec) => exec.set_input(input),
            ExecutorEnum::Aggregate(exec) => exec.set_input(input),
            ExecutorEnum::GroupBy(exec) => exec.set_input(input),
            ExecutorEnum::Having(exec) => exec.set_input(input),
            ExecutorEnum::Dedup(exec) => exec.set_input(input),
            ExecutorEnum::Unwind(exec) => exec.set_input(input),
            ExecutorEnum::Assign(exec) => exec.set_input(input),
            ExecutorEnum::AppendVertices(exec) => exec.set_input(input),
            ExecutorEnum::PatternApply(exec) => exec.set_input(input),
            ExecutorEnum::RollUpApply(exec) => exec.set_input(input),
            ExecutorEnum::Loop(exec) => exec.set_input(input),
            ExecutorEnum::ForLoop(exec) => exec.set_input(input),
            ExecutorEnum::WhileLoop(exec) => exec.set_input(input),
            ExecutorEnum::InnerJoin(exec) => exec.set_input(input),
            ExecutorEnum::HashInnerJoin(exec) => exec.set_input(input),
            ExecutorEnum::LeftJoin(exec) => exec.set_input(input),
            ExecutorEnum::HashLeftJoin(exec) => exec.set_input(input),
            ExecutorEnum::CrossJoin(exec) => exec.set_input(input),
            ExecutorEnum::Traverse(exec) => exec.set_input(input),
            ExecutorEnum::Expand(exec) => exec.set_input(input),
            ExecutorEnum::ExpandAll(exec) => exec.set_input(input),
            ExecutorEnum::ShortestPath(exec) => exec.set_input(input),
            ExecutorEnum::MultiShortestPath(exec) => exec.set_input(input),
            ExecutorEnum::AllPaths(exec) => exec.set_input(input),
            _ => {}
        }
    }
    
    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        match self {
            ExecutorEnum::Filter(exec) => exec.get_input(),
            ExecutorEnum::Project(exec) => exec.get_input(),
            ExecutorEnum::Limit(exec) => exec.get_input(),
            ExecutorEnum::Sort(exec) => exec.get_input(),
            ExecutorEnum::TopN(exec) => exec.get_input(),
            ExecutorEnum::Sample(exec) => exec.get_input(),
            ExecutorEnum::Aggregate(exec) => exec.get_input(),
            ExecutorEnum::GroupBy(exec) => exec.get_input(),
            ExecutorEnum::Having(exec) => exec.get_input(),
            ExecutorEnum::Dedup(exec) => exec.get_input(),
            ExecutorEnum::Unwind(exec) => exec.get_input(),
            ExecutorEnum::Assign(exec) => exec.get_input(),
            ExecutorEnum::AppendVertices(exec) => exec.get_input(),
            ExecutorEnum::PatternApply(exec) => exec.set_input_input(),
            ExecutorEnum::RollUpApply(exec) => exec.get_input(),
            ExecutorEnum::Loop(exec) => exec.get_input(),
            ExecutorEnum::ForLoop(exec) => exec.get_input(),
            ExecutorEnum::WhileLoop(exec) => exec.get_input(),
            ExecutorEnum::InnerJoin(exec) => exec.get_input(),
            ExecutorEnum::HashInnerJoin(exec) => exec.get_input(),
            ExecutorEnum::LeftJoin(exec) => exec.get_input(),
            ExecutorEnum::HashLeftJoin(exec) => exec.get_input(),
            ExecutorEnum::CrossJoin(exec) => exec.get_input(),
            ExecutorEnum::Traverse(exec) => exec.get_input(),
            ExecutorEnum::Expand(exec) => exec.get_input(),
            ExecutorEnum::ExpandAll(exec) => exec.get_input(),
            ExecutorEnum::ShortestPath(exec) => exec.get_input(),
            ExecutorEnum::MultiShortestPath(exec) => exec.get_input(),
            ExecutorEnum::AllPaths(exec) => exec.get_input(),
            _ => None,
        }
    }
    
    fn get_input_mut(&mut self) -> Option<&mut ExecutorEnum<S>> {
        match self {
            ExecutorEnum::Filter(exec) => exec.get_input_mut(),
            ExecutorEnum::Project(exec) => exec.get_input_mut(),
            ExecutorEnum::Limit(exec) => exec.get_input_mut(),
            ExecutorEnum::Sort(exec) => exec.get_input_mut(),
            ExecutorEnum::TopN(exec) => exec.get_input_mut(),
            ExecutorEnum::Sample(exec) => exec.get_input_mut(),
            ExecutorEnum::Aggregate(exec) => exec.get_input_mut(),
            ExecutorEnum::GroupBy(exec) => exec.get_input_mut(),
            ExecutorEnum::Having(exec) => exec.get_input_mut(),
            ExecutorEnum::Dedup(exec) => exec.get_input_mut(),
            ExecutorEnum::Unwind(exec) => exec.get_input_mut(),
            ExecutorEnum::Assign(exec) => exec.get_input_mut(),
            ExecutorEnum::AppendVertices(exec) => exec.get_input_mut(),
            ExecutorEnum::PatternApply(exec) => exec.get_input_mut(),
            ExecutorEnum::RollUpApply(exec) => exec.get_input_mut(),
            ExecutorEnum::Loop(exec) => exec.get_input_mut(),
            ExecutorEnum::ForLoop(exec) => exec.get_input_mut(),
            ExecutorEnum::WhileLoop(exec) => exec.get_input_mut(),
            ExecutorEnum::InnerJoin(exec) => exec.get_input_mut(),
            ExecutorEnum::HashInnerJoin(exec) => exec.get_input_mut(),
            ExecutorEnum::LeftJoin(exec) => exec.get_input_mut(),
            ExecutorEnum::HashLeftJoin(exec) => exec.get_input_mut(),
            ExecutorEnum::CrossJoin(exec) => exec.get_input_mut(),
            ExecutorEnum::Traverse(exec) => exec.get_input_mut(),
            ExecutorEnum::Expand(exec) => exec.get_input_mut(),
            ExecutorEnum::ExpandAll(exec) => exec.get_input_mut(),
            ExecutorEnum::ShortestPath(exec) => exec.get_input_mut(),
            ExecutorEnum::MultiShortestPath(exec) => exec.get_input_mut(),
            ExecutorEnum::AllPaths(exec) => exec.get_input_mut(),
            _ => None,
        }
    }
}
```

## 六、QueryPipeline 设计

### 6.1 查询管道接口

```rust
use crate::query::context::execution::QueryContext;
use crate::query::planner::ExecutionPlan;
use crate::query::executor::ExecutionResult;
use crate::storage::StorageEngine;
use async_trait::async_trait;

/// 查询处理管道
///
/// 协调 Parser、Validator、Planner、Optimizer、Executor 和 Scheduler 的交互，
/// 提供统一的查询处理接口。
pub struct QueryPipeline<S: StorageEngine> {
    query_context: QueryContext,
}

impl<S: StorageEngine> QueryPipeline<S> {
    /// 创建新的查询管道
    pub fn new() -> Self {
        Self {
            query_context: QueryContext::new(),
        }
    }
    
    /// 处理查询
    pub async fn process(&mut self, query: &str) -> Result<ExecutionResult, QueryError> {
        // 1. 解析查询
        let ast = self.parse(query)?;
        
        // 2. 验证查询
        let validated_ast = self.validate(&ast)?;
        
        // 3. 规划查询
        let plan = self.plan(&validated_ast)?;
        
        // 4. 优化查询
        let optimized_plan = self.optimize(plan)?;
        
        // 5. 执行查询
        let result = self.execute(optimized_plan).await?;
        
        Ok(result)
    }
    
    /// 解析查询
    fn parse(&mut self, query: &str) -> Result<Stmt, QueryError> {
        // 使用 Parser 解析查询
        crate::query::parser::parse_query(query)
            .map_err(|e| QueryError::ParseError(e.to_string()))
    }
    
    /// 验证查询
    fn validate(&mut self, ast: &Stmt) -> Result<ValidatedAst, QueryError> {
        // 使用 Validator 验证查询
        let validator = ValidationFactory::create(ast.kind());
        validator.validate(ast, &mut self.query_context)
            .map_err(|e| QueryError::ValidationError(e.to_string()))
    }
    
    /// 规划查询
    fn plan(&mut self, validated_ast: &ValidatedAst) -> Result<ExecutionPlan, QueryError> {
        // 使用 Planner 规划查询
        let planner = PlannerRegistry::create(validated_ast.statement_type());
        planner.plan(validated_ast, &mut self.query_context)
            .map_err(|e| QueryError::PlanningError(e.to_string()))
    }
    
    /// 优化查询
    fn optimize(&mut self, plan: ExecutionPlan) -> Result<ExecutionPlan, QueryError> {
        // 使用 Optimizer 优化查询
        let optimizer = Optimizer::new();
        optimizer.optimize(plan, &mut self.query_context)
            .map_err(|e| QueryError::OptimizationError(e.to_string()))
    }
    
    /// 执行查询
    async fn execute(&mut self, plan: ExecutionPlan) -> Result<ExecutionResult, QueryError> {
        // 使用 Executor 执行查询
        let executor_factory = ExecutorFactory::new();
        let executors = executor_factory.create_executors(&plan)?;
        
        let scheduler = QueryScheduler::new();
        scheduler.schedule(executors).await
            .map_err(|e| QueryError::ExecutionError(e.to_string()))
    }
}
```

### 6.2 执行计划优化器

```rust
use crate::query::optimizer::OptContext;
use crate::query::optimizer::engine::Optimizer;
use crate::query::optimizer::rule_traits::BaseOptRule;

/// 优化的执行计划
///
/// 包含优化后的计划节点序列，以及相关的元数据。
pub struct OptimizedExecutionPlan {
    /// 计划节点序列
    nodes: Vec<PlanNodeEnum>,
    
    /// 优化统计信息
    stats: OptimizationStats,
    
    /// 使用的优化规则
    applied_rules: Vec<&'static str>,
}

impl OptimizedExecutionPlan {
    /// 创建新的优化执行计划
    pub fn new(nodes: Vec<PlanNodeEnum>) -> Self {
        Self {
            nodes,
            stats: OptimizationStats::new(),
            applied_rules: Vec::new(),
        }
    }
    
    /// 获取计划节点
    pub fn nodes(&self) -> &[PlanNodeEnum] {
        &self.nodes
    }
    
    /// 获取优化统计
    pub fn stats(&self) -> &OptimizationStats {
        &self.stats
    }
    
    /// 获取应用的规则
    pub fn applied_rules(&self) -> &[&'static str] {
        &self.applied_rules
    }
}

/// 优化统计信息
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    /// 原始节点数
    original_node_count: usize,
    
    /// 优化后节点数
    optimized_node_count: usize,
    
    /// 优化耗时（毫秒）
    optimization_time_ms: u64,
    
    /// 成本估计变化
    cost_estimate: Option<f64>,
}

impl OptimizationStats {
    pub fn new() -> Self {
        Self {
            original_node_count: 0,
            optimized_node_count: 0,
            optimization_time_ms: 0,
            cost_estimate: None,
        }
    }
    
    pub fn set_original_node_count(&mut self, count: usize) {
        self.original_node_count = count;
    }
    
    pub fn set_optimized_node_count(&mut self, count: usize) {
        self.optimized_node_count = count;
    }
    
    pub fn set_optimization_time(&mut self, time_ms: u64) {
        self.optimization_time_ms = time_ms;
    }
    
    pub fn set_cost_estimate(&mut self, cost: f64) {
        self.cost_estimate = Some(cost);
    }
}
```

## 七、模块集成方案

### 7.1 Parser 模块集成

```rust
// 在 parser/mod.rs 中添加
pub use ast::stmt::Stmt;
pub use ast::stmt::StmtKind;

/// 将 Stmt 转换为 CoreOperationKind
impl From<&Stmt> for CoreOperationKind {
    fn from(stmt: &Stmt) -> Self {
        match stmt {
            Stmt::Match(_) => CoreOperationKind::Match,
            Stmt::Go(_) => CoreOperationKind::Go,
            Stmt::Lookup(_) => CoreOperationKind::Lookup,
            Stmt::FindPath(_) => CoreOperationKind::FindPath,
            Stmt::Subgraph(_) => CoreOperationKind::GetSubgraph,
            Stmt::Insert(insert_stmt) => {
                match &insert_stmt.target {
                    InsertTarget::Vertex { .. } => CoreOperationKind::Insert,
                    InsertTarget::Edge { .. } => CoreOperationKind::Insert,
                }
            }
            Stmt::Update(_) => CoreOperationKind::Update,
            Stmt::Delete(_) => CoreOperationKind::Delete,
            Stmt::Create(create_stmt) => {
                match &create_stmt.target {
                    CreateTarget::Space { .. } => CoreOperationKind::CreateSpace,
                    CreateTarget::Tag { .. } => CoreOperationKind::CreateTag,
                    CreateTarget::EdgeType { .. } => CoreOperationKind::CreateEdge,
                    _ => CoreOperationKind::CreateSpace,
                }
            }
            Stmt::Drop(drop_stmt) => {
                match &drop_stmt.target {
                    DropTarget::Space { .. } => CoreOperationKind::DropSpace,
                    DropTarget::Tag { .. } => CoreOperationKind::DropTag,
                    DropTarget::EdgeType { .. } => CoreOperationKind::DropEdge,
                    _ => CoreOperationKind::DropSpace,
                }
            }
            Stmt::Use(_) => CoreOperationKind::UseSpace,
            Stmt::Show(show_stmt) => {
                match &show_stmt.target {
                    ShowTarget::Spaces => CoreOperationKind::Show,
                    ShowTarget::Tags => CoreOperationKind::Show,
                    ShowTarget::Edges => CoreOperationKind::Show,
                    _ => CoreOperationKind::Show,
                }
            }
            Stmt::Desc(desc_stmt) => {
                match &desc_stmt.target {
                    DescTarget::Space { .. } => CoreOperationKind::DescribeSpace,
                    DescTarget::Tag { .. } => CoreOperationKind::DescribeTag,
                    DescTarget::Edge { .. } => CoreOperationKind::DescribeEdge,
                    _ => CoreOperationKind::DescribeSpace,
                }
            }
            Stmt::Alter(alter_stmt) => {
                match &alter_stmt.target {
                    AlterTarget::Tag { .. } => CoreOperationKind::AlterTag,
                    AlterTarget::Edge { .. } => CoreOperationKind::AlterEdge,
                    _ => CoreOperationKind::AlterTag,
                }
            }
            Stmt::ChangePassword(_) => CoreOperationKind::ChangePassword,
            _ => CoreOperationKind::Match,
        }
    }
}
```

### 7.2 Validator 模块集成

```rust
// 在 validator/mod.rs 中添加
pub use validation_factory::{ValidationFactory, ValidatorRegistry, StatementType};

/// 从 CoreOperationKind 转换为 StatementType
impl From<CoreOperationKind> for StatementType {
    fn from(kind: CoreOperationKind) -> Self {
        match kind {
            CoreOperationKind::Match => StatementType::Match,
            CoreOperationKind::Go => StatementType::Go,
            CoreOperationKind::Lookup => StatementType::Lookup,
            CoreOperationKind::FindPath => StatementType::FindPath,
            CoreOperationKind::GetSubgraph => StatementType::GetSubgraph,
            CoreOperationKind::Insert => StatementType::InsertVertices, // 或 InsertEdges
            CoreOperationKind::Update => StatementType::Update,
            CoreOperationKind::Delete => StatementType::Delete,
            CoreOperationKind::CreateSpace => StatementType::CreateSpace,
            CoreOperationKind::DropSpace => StatementType::DropSpace,
            CoreOperationKind::DescribeSpace => StatementType::DescribeSpace,
            CoreOperationKind::UseSpace => StatementType::Use,
            CoreOperationKind::CreateTag => StatementType::CreateTag,
            CoreOperationKind::AlterTag => StatementType::AlterTag,
            CoreOperationKind::DropTag => StatementType::DropTag,
            CoreOperationKind::DescribeTag => StatementType::DescribeTag,
            CoreOperationKind::CreateEdge => StatementType::CreateEdge,
            CoreOperationKind::AlterEdge => StatementType::AlterEdge,
            CoreOperationKind::DropEdge => StatementType::DropEdge,
            CoreOperationKind::DescribeEdge => StatementType::DescribeEdge,
            _ => StatementType::Match,
        }
    }
}
```

### 7.3 Executor 模块集成

```rust
// 在 executor/mod.rs 中添加
pub use executor_enum::ExecutorEnum;
pub use traits::{Executor, InputExecutor, ChainableExecutor};

/// 从 PlanNodeEnum 转换为 CoreOperationKind
impl From<&PlanNodeEnum> for CoreOperationKind {
    fn from(node: &PlanNodeEnum) -> Self {
        match node {
            PlanNodeEnum::Start(_) => CoreOperationKind::ScanVertices,
            PlanNodeEnum::Project(_) => CoreOperationKind::Project,
            PlanNodeEnum::Filter(_) => CoreOperationKind::Filter,
            PlanNodeEnum::Sort(_) => CoreOperationKind::Sort,
            PlanNodeEnum::Limit(_) => CoreOperationKind::Limit,
            PlanNodeEnum::TopN(_) => CoreOperationKind::TopN,
            PlanNodeEnum::Sample(_) => CoreOperationKind::Sample,
            PlanNodeEnum::Dedup(_) => CoreOperationKind::Dedup,
            PlanNodeEnum::GetVertices(_) => CoreOperationKind::GetVertices,
            PlanNodeEnum::GetNeighbors(_) => CoreOperationKind::GetNeighbors,
            PlanNodeEnum::ScanVertices(_) => CoreOperationKind::ScanVertices,
            PlanNodeEnum::ScanEdges(_) => CoreOperationKind::ScanEdges,
            PlanNodeEnum::Expand(_) => CoreOperationKind::Expand,
            PlanNodeEnum::ExpandAll(_) => CoreOperationKind::ExpandAll,
            PlanNodeEnum::Traverse(_) => CoreOperationKind::Traverse,
            PlanNodeEnum::InnerJoin(_) => CoreOperationKind::InnerJoin,
            PlanNodeEnum::LeftJoin(_) => CoreOperationKind::LeftJoin,
            PlanNodeEnum::CrossJoin(_) => CoreOperationKind::CrossJoin,
            PlanNodeEnum::HashInnerJoin(_) => CoreOperationKind::HashJoin,
            PlanNodeEnum::Aggregate(_) => CoreOperationKind::Aggregate,
            PlanNodeEnum::GroupBy(_) => CoreOperationKind::GroupBy,
            PlanNodeEnum::Having(_) => CoreOperationKind::Having,
            PlanNodeEnum::Unwind(_) => CoreOperationKind::Unwind,
            PlanNodeEnum::AppendVertices(_) => CoreOperationKind::AppendVertices,
            PlanNodeEnum::PatternApply(_) => CoreOperationKind::PatternApply,
            PlanNodeEnum::RollUpApply(_) => CoreOperationKind::RollUpApply,
            PlanNodeEnum::Loop(_) => CoreOperationKind::Loop,
            PlanNodeEnum::ForLoop(_) => CoreOperationKind::ForLoop,
            PlanNodeEnum::WhileLoop(_) => CoreOperationKind::WhileLoop,
            PlanNodeEnum::Assign(_) => CoreOperationKind::Assignment,
            PlanNodeEnum::CreateSpace(_) => CoreOperationKind::CreateSpace,
            PlanNodeEnum::DropSpace(_) => CoreOperationKind::DropSpace,
            PlanNodeEnum::DescSpace(_) => CoreOperationKind::DescribeSpace,
            PlanNodeEnum::CreateTag(_) => CoreOperationKind::CreateTag,
            PlanNodeEnum::AlterTag(_) => CoreOperationKind::AlterTag,
            PlanNodeEnum::DropTag(_) => CoreOperationKind::DropTag,
            PlanNodeEnum::DescTag(_) => CoreOperationKind::DescribeTag,
            PlanNodeEnum::CreateEdge(_) => CoreOperationKind::CreateEdge,
            PlanNodeEnum::AlterEdge(_) => CoreOperationKind::AlterEdge,
            PlanNodeEnum::DropEdge(_) => CoreOperationKind::DropEdge,
            PlanNodeEnum::DescEdge(_) => CoreOperationKind::DescribeEdge,
            PlanNodeEnum::InsertVertex(_) => CoreOperationKind::Insert,
            PlanNodeEnum::InsertEdge(_) => CoreOperationKind::Insert,
            PlanNodeEnum::Update(_) => CoreOperationKind::Update,
            PlanNodeEnum::ShortestPath(_) => CoreOperationKind::ShortestPath,
            PlanNodeEnum::AllPaths(_) => CoreOperationKind::AllPaths,
            PlanNodeEnum::MultiShortestPath(_) => CoreOperationKind::AllPaths,
            PlanNodeEnum::Minus(_) => CoreOperationKind::Delete,
            PlanNodeEnum::Intersect(_) => CoreOperationKind::Filter,
            PlanNodeEnum::Union(_) => CoreOperationKind::Project,
            PlanNodeEnum::UnionAll(_) => CoreOperationKind::Project,
            _ => CoreOperationKind::Project,
        }
    }
}
```

## 八、性能优化考量

### 8.1 静态分发的性能优势

使用 `ExecutorEnum` 替代 `Box<dyn Executor<S>>` 可以获得以下性能优势：

1. **消除虚函数调用开销**：静态分发在编译时确定调用目标，无需运行时查找
2. **内联优化**：编译器可以内联小型的执行器方法
3. **更好的分支预测**：match 表达式可以被编译器优化

```rust
// 动态分发（之前）
async fn execute(&mut self) -> DBResult<ExecutionResult> {
    self.inner.execute().await  // 虚函数调用
}

// 静态分发（之后）
async fn execute(&mut self) -> DBResult<ExecutionResult> {
    match self {
        ExecutorEnum::Filter(exec) => exec.execute().await,
        ExecutorEnum::Project(exec) => exec.execute().await,
        // ... 所有分支在编译时已知
    }
}
```

### 8.2 访问者模式的性能优化

`PlanNodeVisitor` 的设计考虑了性能优化：

1. **零成本抽象**：trait 的默认实现不引入额外开销
2. **避免递归**：使用迭代方式遍历大型计划树
3. **缓存访问结果**：避免重复访问相同的节点

```rust
/// 优化：使用迭代方式遍历计划树
pub fn collect_all_nodes(plan: &ExecutionPlan) -> Vec<&PlanNodeEnum> {
    let mut nodes = Vec::new();
    let mut stack = vec![plan.root()];
    
    while let Some(node) = stack.pop() {
        nodes.push(node);
        // 添加子节点到栈中
        for child in node.children() {
            stack.push(child);
        }
    }
    
    nodes
}
```

### 8.3 内存优化

1. **小对象优化**：对于小型的执行器，使用栈分配而非堆分配
2. **对象池**：重复使用的执行器从对象池获取
3. **引用计数**：共享不可变数据使用 Arc

## 九、总结

本文档描述了 GraphDB 查询模块改进后的架构设计。改进设计主要包括以下方面：

1. **统一类型定义**：引入 `CoreOperationKind` 枚举，统一各模块的操作类型定义
2. **完整处理链条**：确保 Parser → Validator → Planner → Optimizer → Executor → Scheduler 的每个环节都完整可用
3. **静态分发改造**：将 `InputExecutor` 和 `ChainableExecutor` 改造为使用 `ExecutorEnum`，消除动态分发的开销
4. **访问者模式**：引入 `PlanNodeVisitor`，统一 PlanNode 的遍历和处理逻辑
5. **QueryPipeline**：设计统一的查询处理管道，简化查询处理流程

这些改进将显著提升代码的可维护性、可扩展性和系统性能。

## 十、参考文档

- [查询模块操作类型分析](query_operation_type_analysis.md)
- [处理链条完整性分析](processing_chain_integrity_analysis.md)
- [分阶段修改计划](phased_modification_plan.md)
- [模块问题与解决方案](modules_issues_and_solutions.md)
