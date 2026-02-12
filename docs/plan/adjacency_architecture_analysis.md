# graphDB与nebula-graph邻接处理架构对比分析

## 文档信息
- **创建日期**: 2026-02-12
- **分析对象**: graphDB vs nebula-graph 邻接边和节点处理架构
- **分析范围**: 架构设计、功能实现（不涉及语言差异）

---

## 一、核心数据结构对比

### 1.1 顶点（Vertex）结构

#### nebula-graph
- **实现位置**: `src/storage/exec/TagNode.h`
- **特点**:
  - 使用 `TagNode` 处理顶点标签数据
  - 顶点ID（VertexID）是字符串或整数类型
  - 标签（Tag）通过 `TagID` 标识，支持多标签
  - 属性通过 `PropContext` 定义，支持属性在键中的特殊位置（VID、TAG、SRC等）

#### graphDB
- **实现位置**: `src/core/vertex_edge_path.rs`
- **结构**:
```rust
pub struct Vertex {
    pub vid: Box<Value>,      // 用户可见的顶点ID，可以是任意Value类型
    pub id: i64,              // 内部整数ID，用于索引和快速查找
    pub tags: Vec<Tag>,       // 多标签支持
    pub properties: HashMap<String, Value>,  // 顶点级属性
}
```
- **特点**:
  - 双ID设计：用户可见的 `vid`（任意Value类型）和内部 `id`（i64）
  - 标签存储在 `Vec<Tag>` 中，每个Tag包含名称和属性

### 1.2 边（Edge）结构

#### nebula-graph
- **实现位置**: `src/storage/exec/EdgeNode.h`
- **特点**:
  - 使用 `EdgeNode` 模板类处理边数据
  - 支持 `EdgeType`（边类型ID）标识
  - 边键包含：srcId + edgeType + rank + dstId
  - 支持TTL（生存时间）检查
  - 可以跳过解码（skipDecode）优化性能

#### graphDB
- **实现位置**: `src/core/vertex_edge_path.rs`
- **结构**:
```rust
pub struct Edge {
    pub src: Box<Value>,      // 源顶点ID
    pub dst: Box<Value>,      // 目标顶点ID
    pub edge_type: String,    // 边类型名称（字符串）
    pub ranking: i64,         // 边排序值
    pub id: i64,              // 内部整数ID
    pub props: HashMap<String, Value>,  // 边属性
}
```
- **特点**:
  - 使用字符串标识边类型（而非整数ID）
  - 支持ranking排序
  - 双ID设计类似Vertex

---

## 二、邻接查询架构对比

### 2.1 nebula-graph的DAG执行架构

#### 核心组件

1. **RelNode** (`src/storage/exec/RelNode.h`)
   - 关系代数节点基类
   - 支持依赖关系（dependencies_）
   - 模板化设计支持不同输入类型
   - 执行时间统计

2. **QueryNode/IterateNode**
   - `QueryNode<T>`：单次查询节点
   - `IterateNode<T>`：可迭代节点，继承自StorageIterator

3. **专用节点类型**:
   - `GetNeighborsNode`: 邻居查询核心节点
   - `HashJoinNode`: 标签和边的哈希连接
   - `TagNode`: 顶点标签处理
   - `EdgeNode/SingleEdgeNode`: 边处理
   - `FilterNode`: 过滤节点
   - `AggregateNode`: 聚合统计

#### 执行流程
```
GetNeighborsRequest → GetNeighborsProcessor → buildPlan
→ HashJoinNode (TagNode + SingleEdgeNode[])
→ GetNeighborsNode → iterateEdges → 结果DataSet
```

### 2.2 graphDB的执行器架构

#### 核心组件

1. **ExpandExecutor** (`src/query/executor/data_processing/graph_traversal/expand.rs`)
   - 路径扩展执行器
   - 支持多步扩展（max_depth）
   - 支持采样（sampling）
   - 邻接关系缓存（adjacency_cache）

2. **迭代器体系**:
   - `GetNeighborsIter`: 复杂嵌套迭代器
   - `GetNeighborsIterator`: 简化版迭代器

3. **遍历执行器**:
   - `TraverseExecutor`: 通用图遍历
   - `AllPathsExecutor`: 所有路径查询
   - `ShortestPathExecutor`: 最短路径查询
   - `ExpandAllExecutor`: 全扩展查询

#### 执行流程
```
输入节点 → ExpandExecutor.expand_multi_step
→ expand_step → get_neighbors → 邻居节点集合
```

---

## 三、关键功能差异

| 功能特性 | nebula-graph | graphDB |
|---------|-------------|---------|
| **执行模型** | DAG（有向无环图）节点执行 | 迭代器+执行器模式 |
| **边方向支持** | 通过EdgeType正负值区分（正=out，负=in） | `EdgeDirection`枚举（Out/In/Both） |
| **多标签处理** | HashJoinNode合并多个TagNode | Vertex.tags: Vec<Tag> |
| **属性过滤** | FilterNode支持表达式过滤 | 执行器内部处理 |
| **统计聚合** | AggregateNode专门处理 | AggregateExecutor已实现 |
| **采样支持** | GetNeighborsSampleNode（水库采样） | ExpandExecutor支持采样 |
| **自环边处理** | 去重机制（visitedSelfReflectiveEdges_） | 未实现 |
| **TTL支持** | 完整支持（EdgeNode/TagNode） | 未实现 |
| **内存锁** | VerticesMemLock/EdgesMemLock | 未实现 |

---

## 四、架构设计差异总结

### 4.1 nebula-graph优势
1. **成熟的DAG执行引擎**：节点职责单一，易于扩展和优化
2. **完善的统计功能**：AggregateNode支持SUM/COUNT/AVG/MAX/MIN
3. **分布式支持**：Processor支持多线程执行
4. **内存管理**：专门的内存锁机制防止并发冲突
5. **性能优化**：支持skipDecode、TTL检查、自环边去重

### 4.2 graphDB现状
1. **简化设计**：单节点架构，移除了分布式复杂性
2. **统一类型系统**：使用Value枚举统一处理各种数据类型
3. **迭代器丰富**：多种迭代器支持不同查询场景
4. **缓存机制**：ExpandExecutor内置邻接关系缓存
5. **聚合功能**：AggregateExecutor已实现基础聚合功能

### 4.3 数据共享机制

graphDB已有一套成熟的并发控制体系：

1. **存储层**：使用 `Arc<Mutex<S>>` 包装StorageClient
2. **调度层**：`AsyncScheduler` 使用 `safe_lock` 保护执行状态
3. **并发模型**：`concurrency.rs` 提供了三级并发策略
   - `LocalSymbolTable`：单查询模式（RefCell）
   - `SharedSymbolTable`：多查询模式（RwLock）
   - `Arc<RwLock>`：跨查询共享

---

## 五、功能必要性分析

### 5.1 TTL支持
- **建议程度**: ⭐⭐⭐（中等优先级）
- **分析**: 对于单节点数据库，TTL功能并非必需，但有用。适用场景包括会话数据、临时缓存、日志数据等
- **结论**: 不是过度设计，但优先级不高

### 5.2 自环边去重
- **建议程度**: ⭐⭐⭐⭐（较高优先级）
- **分析**: nebula-graph在 `GetNeighborsNode` 中专门处理了自环边去重。在图遍历中，自环边（A->A）会被重复访问，导致结果膨胀。这是数据正确性问题，不是性能问题
- **结论**: 不是过度设计，应该实现

### 5.3 更复杂的过滤节点
- **建议程度**: ⭐⭐（低优先级）
- **分析**: graphDB当前在执行器内部处理过滤。nebula-graph的 `FilterNode` 是DAG架构的一部分，职责分离更清晰。但graphDB采用的是迭代器+执行器模式，将过滤内联在执行器中可以減少抽象层次
- **结论**: 可能是过度设计，当前架构下过滤逻辑内联在执行器中是合理的

### 5.4 内存锁机制
- **建议程度**: ⭐（极低优先级）
- **分析**: graphDB已经有一套成熟的并发控制体系。作为单节点数据库，其并发模型已经足够：
  - 查询级并发：通过 `Arc<Mutex<StorageClient>>` 实现
  - 执行器状态：通过调度器统一管理
- **结论**: 完全过度设计，不需要nebula-graph那样的细粒度内存锁

---

## 六、执行器架构对比分析

### 6.1 当前架构特点

graphDB已有约70+个执行器：
- `TraverseExecutor`、`AllPathsExecutor`、`ShortestPathExecutor` - 图遍历
- `ExpandExecutor`、`ExpandAllExecutor` - 路径扩展
- `AggregateExecutor`、`GroupByExecutor` - 聚合
- `InnerJoinExecutor`、`HashInnerJoinExecutor` - 连接
- `FilterExecutor`、`ProjectExecutor` - 数据处理

**调度方式**:
```rust
pub struct AsyncScheduler {
    execution_state: Arc<Mutex<ExecutionState>>,
    completion_notifier: Arc<(Mutex<bool>, Condvar)>,
}
```

**优势**:
1. 职责清晰：每个执行器专注单一功能
2. 易于测试：独立执行器可单元测试
3. 组合灵活：通过 `ExecutorEnum` 组合
4. 调度成熟：`AsyncScheduler` 支持异步并行执行

### 6.2 DAG架构特点

**优势**:
1. 执行计划可视化：DAG结构清晰
2. 细粒度优化：可对单个节点优化
3. 延迟执行：按需拉取数据

**劣势**:
1. 抽象复杂：需要大量节点类型
2. 调试困难：执行流分散在多个节点
3. 不适合Rust：所有权和生命周期管理复杂

### 6.3 是否值得改造为DAG架构？

**结论：不值得**

| 维度 | 当前架构 | DAG架构 | 评估 |
|------|---------|---------|------|
| **代码量** | 已有70+执行器，结构清晰 | 需要重构所有执行器为节点 | 改造成本极高 |
| **性能** | AsyncScheduler已支持并行 | 理论上更优，但单节点差异小 | 收益有限 |
| **维护性** | 执行器职责单一，易于维护 | 节点间依赖复杂 | 当前更优 |
| **Rust适配** | 所有权模型自然 | 生命周期管理复杂 | 当前更优 |
| **功能完整性** | 已覆盖所有查询类型 | 无新增功能 | 无收益 |

**关键理由**:
1. graphDB的执行器模式本质上已经是DAG的变体 - `AsyncScheduler` 管理执行顺序和依赖
2. 单节点场景下，执行器间的数据传递开销远低于分布式网络通信，不需要DAG的细粒度优化
3. Rust的所有权系统使得DAG节点的生命周期管理复杂，容易引入内存安全问题

---

## 七、总结

### 7.1 核心结论
1. **graphDB不应该改造为DAG架构** - 当前执行器架构在单节点场景下已经足够优秀，且更适合Rust语言特性
2. **应该优先实现自环边去重** - 以保证图遍历的数据正确性
3. **内存锁机制是过度设计** - 当前并发模型已足够

### 7.2 建议优先级

| 功能 | 是否必要 | 优先级 | 理由 |
|------|---------|--------|------|
| **改造为DAG架构** | ❌ 不推荐 | 无 | 当前架构已足够，改造成本高收益低 |
| **自环边去重** | ✅ 必要 | 高 | 数据正确性问题 |
| **TTL支持** | ⏳ 可选 | 中 | 非核心功能，但有用 |
| **过滤节点提取** | ❌ 不需要 | 低 | 当前内联方式更适合单节点场景 |
| **内存锁机制** | ❌ 不需要 | 无 | 已存在完善的并发模型 |

---

## 参考文档
- `docs/adjacency_analysis.md` - 原始邻接分析文档
- `docs/plan/adjacency_analysis_comparison.md` - 邻接分析对比
- `docs/plan/adjacency_processing_plan.md` - 邻接处理计划
