# Nebula-Graph Executor 架构对比分析

## 概述

本文档对比分析 Nebula-Graph 3.8.0 的 executor 模块与新 GraphDB 架构的对应关系，重点关注语言无关的功能性/架构区别。

---

## 一、模块映射关系

### 1.1 Nebula-Graph Executor 目录结构

```
nebula-3.8.0/src/graph/executor/
├── admin/           # 管理操作（用户、主机、配置、作业、会话等）
├── algo/            # 算法执行器（最短路径、子图、笛卡尔积等）
├── logic/           # 逻辑控制执行器（循环、选择、参数传递等）
├── maintain/       # 维护操作（Tag、Edge、Index 的创建/修改/删除）
├── mutate/          # 数据修改操作（插入、更新、删除）
├── query/           # 查询执行器（过滤、连接、聚合、扫描等）
└── test/            # 测试
```

### 1.2 GraphDB Executor 目录结构

```
src/query/executor/
├── admin/           # 管理操作
│   ├── edge/        # 边管理
│   ├── index/       # 索引管理
│   ├── space/       # 空间管理
│   ├── tag/         # 标签管理
│   └── user/        # 用户管理
├── base/            # 基础执行器
├── data_processing/ # 数据处理
│   ├── graph_traversal/  # 图遍历
│   ├── join/             # 连接
│   └── set_operations/   # 集合操作
├── logic/           # 逻辑控制
├── result_processing/ # 结果处理
│   └── transformations/ # 转换
├── data_access.rs       # 数据访问执行器
├── data_modification.rs # 数据修改执行器
└── special_executors.rs # 特殊执行器
```

---

## 二、详细模块对比

### 2.1 管理操作

#### Nebula-Graph (admin/)

| 模块 | 功能 | 对应 GraphDB 模块 |
|------|------|------------------|
| SpaceExecutor | 空间管理（创建、删除、切换） | `admin/space/` |
| TagExecutor | 标签管理 | `admin/tag/` |
| EdgeExecutor | 边类型管理 | `admin/edge/` |
| TagIndexExecutor, EdgeIndexExecutor | 索引管理 | `admin/index/` |
| CreateUserExecutor, DropUserExecutor, UpdateUserExecutor | 用户管理 | `admin/user/` |
| ConfigExecutor | 配置管理 | 未实现（单节点不需要） |
| AddHostsExecutor, DropHostsExecutor, ShowHostsExecutor | 主机管理 | 未实现（单节点不需要） |
| SubmitJobExecutor | 作业提交 | 未实现（分布式特性） |
| SnapshotExecutor | 快照 | 未实现（分布式特性） |
| ZoneExecutor | 分区管理 | 未实现（分布式特性） |
| ListenerExecutor | 监听器 | 未实现（分布式特性） |
| SessionExecutor, ShowQueriesExecutor, KillQueryExecutor | 会话管理 | 部分在 `api/session/` |

#### 架构区别

**Nebula-Graph:**
- 包含大量分布式系统相关的管理功能
- 主机管理、分区管理、快照等都是分布式架构必需的
- 配置管理涉及多节点协调
- 作业系统支持分布式任务执行

**GraphDB:**
- 专注于单节点场景，移除了所有分布式相关功能
- 空间、标签、边类型、索引的管理功能保留
- 用户管理简化，仅保留基础认证
- 会话管理移至 API 层，与执行器解耦

---

### 2.2 算法执行器

#### Nebula-Graph (algo/)

| 模块 | 功能 | 对应 GraphDB 模块 |
|------|------|------------------|
| ShortestPathExecutor | 最短路径 | `data_processing/graph_traversal/shortest_path.rs` |
| BFSShortestPathExecutor | BFS 最短路径 | 同上 |
| AllPathsExecutor | 所有路径 | 未实现 |
| SubgraphExecutor | 子图提取 | 未实现 |
| CartesianProductExecutor | 笛卡尔积 | `data_processing/join/cross_join.rs` |
| MultiShortestPathExecutor | 多源最短路径 | 未实现 |

#### 架构区别

**Nebula-Graph:**
- 提供多种路径算法（BFS、多源、所有路径）
- 算法执行器直接继承 Executor 基类
- 算法与查询执行紧密耦合

**GraphDB:**
- 仅实现最短路径算法，采用更通用的遍历框架
- 算法作为图遍历的一部分，通过 `GraphTraversal` trait 实现
- 更好的扩展性，便于添加新算法

---

### 2.3 逻辑控制执行器

#### Nebula-Graph (logic/)

| 模块 | 功能 | 对应 GraphDB 模块 |
|------|------|------------------|
| LoopExecutor | 循环执行 | `logic/loops.rs` |
| SelectExecutor | 条件选择 | 未实现（使用条件表达式） |
| ArgumentExecutor | 参数传递 | `special_executors.rs` |
| StartExecutor | 起始节点 | 未实现（简化） |
| PassThroughExecutor | 直通 | `special_executors.rs` |

#### 架构区别

**Nebula-Graph:**
- 完整的逻辑控制结构（循环、选择、参数）
- 支持复杂的执行流控制
- 每个逻辑操作都是独立的执行器

**GraphDB:**
- 仅实现循环功能，选择逻辑通过条件表达式实现
- 逻辑控制更简化，减少执行器数量
- 参数传递和直通作为特殊执行器处理

---

### 2.4 维护操作

#### Nebula-Graph (maintain/)

| 模块 | 功能 | 对应 GraphDB 模块 |
|------|------|------------------|
| TagExecutor | 标签维护 | `admin/tag/` |
| EdgeExecutor | 边类型维护 | `admin/edge/` |
| TagIndexExecutor | 标签索引维护 | `admin/index/tag_index.rs` |
| EdgeIndexExecutor | 边索引维护 | `admin/index/edge_index.rs` |
| FTIndexExecutor | 全文索引 | 未实现 |

#### 架构区别

**Nebula-Graph:**
- 维护操作作为独立的执行器
- 支持全文索引（FTIndexExecutor）

**GraphDB:**
- 维护操作整合到 admin 模块
- 按功能分类（tag、edge、index），结构更清晰
- 暂未实现全文索引

---

### 2.5 数据修改操作

#### Nebula-Graph (mutate/)

| 模块 | 功能 | 对应 GraphDB 模块 |
|------|------|------------------|
| InsertExecutor | 插入数据 | `data_modification.rs` (InsertExecutor) |
| UpdateExecutor | 更新数据 | `data_modification.rs` (UpdateExecutor) |
| DeleteExecutor | 删除数据 | `data_modification.rs` (DeleteExecutor) |

#### 架构区别

**Nebula-Graph:**
- 每个操作都是独立的执行器
- 支持批量操作和条件更新/删除
- 与存储层通过 RPC 交互

**GraphDB:**
- 所有修改操作集中在 `data_modification.rs` 文件
- 使用 `StorageProcessorExecutor` 统一处理存储访问
- 支持条件表达式过滤
- 批量操作通过 `BatchOptimizer` 优化

---

### 2.6 查询执行器

#### Nebula-Graph (query/)

| 模块 | 功能 | 对应 GraphDB 模块 |
|------|------|------------------|
| ScanVerticesExecutor | 扫描顶点 | `data_access.rs` (GetVerticesExecutor) |
| ScanEdgesExecutor | 扫描边 | `data_access.rs` (ScanEdgesExecutor) |
| GetVerticesExecutor | 获取顶点 | `data_access.rs` (GetVerticesExecutor) |
| GetEdgesExecutor | 获取边 | `data_access.rs` (GetEdgesExecutor) |
| GetNeighborsExecutor | 获取邻居 | `data_access.rs` (GetNeighborsExecutor) |
| IndexScanExecutor | 索引扫描 | `optimizer/rules/index/` |
| FilterExecutor | 过滤 | `result_processing/filter.rs` |
| ProjectExecutor | 投影 | `result_processing/projection.rs` |
| AggregateExecutor | 聚合 | `result_processing/aggregation.rs` |
| SortExecutor | 排序 | `result_processing/sort.rs` |
| LimitExecutor | 限制 | `result_processing/limit.rs` |
| TopNExecutor | TopN | `result_processing/topn.rs` |
| ExpandExecutor | 扩展 | `data_processing/graph_traversal/expand.rs` |
| ExpandAllExecutor | 全扩展 | `data_processing/graph_traversal/expand_all.rs` |
| TraverseExecutor | 遍历 | `data_processing/graph_traversal/traverse.rs` |
| JoinExecutor, InnerJoinExecutor, LeftJoinExecutor | 连接 | `data_processing/join/` |
| UnionExecutor, UnionAllVersionVarExecutor | 并集 | `data_processing/set_operations/union.rs` |
| IntersectExecutor | 交集 | `data_processing/set_operations/intersect.rs` |
| MinusExecutor | 差集 | `data_processing/set_operations/minus.rs` |
| DedupExecutor | 去重 | `result_processing/dedup.rs` |
| AppendVerticesExecutor | 追加顶点 | `result_processing/transformations/append_vertices.rs` |
| AssignExecutor | 赋值 | `result_processing/transformations/assign.rs` |
| UnwindExecutor | 展开 | `result_processing/transformations/unwind.rs` |
| PatternApplyExecutor | 模式应用 | `result_processing/transformations/pattern_apply.rs` |
| RollUpApplyExecutor | 滚动应用 | `result_processing/transformations/rollup_apply.rs` |
| SampleExecutor | 采样 | `result_processing/sample.rs` |
| SetExecutor | 集合操作 | 未单独实现 |
| DataCollectExecutor | 数据收集 | `special_executors.rs` |
| ValueExecutor | 值执行器 | 未实现 |
| GetPropExecutor | 获取属性 | `data_access.rs` (GetPropExecutor) |
| FulltextIndexScanExecutor | 全文索引扫描 | 未实现 |

#### 架构区别

**Nebula-Graph:**
- 所有查询操作都是独立的执行器
- 执行器之间通过 ExecutionContext 传递数据
- 支持全文索引扫描
- 每个操作都有专门的执行器类

**GraphDB:**
- 按功能分类组织（数据访问、数据处理、结果处理）
- 使用 trait 定义执行器接口，更灵活
- 数据访问执行器集中在 `data_access.rs`
- 数据处理（遍历、连接、集合操作）在 `data_processing/`
- 结果处理（过滤、聚合、排序等）在 `result_processing/`
- 转换操作单独在 `transformations/` 子目录
- 未实现全文索引

---

## 三、核心架构区别

### 3.1 执行器基类设计

#### Nebula-Graph

```cpp
class Executor {
    virtual Status open();
    virtual folly::Future<Status> execute() = 0;
    virtual Status close();
    
    QueryContext *qctx_;
    ExecutionContext *ectx_;
    std::set<Executor *> depends_;
    std::set<Executor *> successors_;
};
```

**特点：**
- 使用 C++ 继承
- 基于 folly::Future 的异步执行
- 依赖关系通过 depends_/successors_ 管理
- 内存管理通过 MemoryTracker

#### GraphDB

```rust
pub trait Executor<S: StorageClient>: Send + Sync {
    async fn execute(&mut self) -> DBResult<ExecutionResult>;
    fn open(&mut self) -> DBResult<()>;
    fn close(&mut self) -> DBResult<()>;
    fn is_open(&self) -> bool;
    fn id(&self) -> i64;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn stats(&self) -> &ExecutorStats;
    fn stats_mut(&mut self) -> &mut ExecutorStats;
}
```

**特点：**
- 使用 Rust trait 定义接口
- 基于 async/await 的异步执行
- 更灵活的组合方式
- 统一的错误处理（DBResult）

---

### 3.2 数据流管理

#### Nebula-Graph

- 使用 ExecutionContext 存储中间结果
- 通过变量名引用传递数据
- 支持数据移动优化（movable）

#### GraphDB

- 使用 ExecutionResult 直接传递
- 更类型化的数据流
- 支持批量优化（BatchOptimizer）

---

### 3.3 存储访问

#### Nebula-Graph

- 通过 StorageClient RPC 访问存储层
- 存储访问执行器继承 StorageAccessExecutor
- 支持分布式存储访问

#### GraphDB

- 使用 StorageProcessorExecutor 统一处理存储访问
- 批量操作通过 BatchOptimizer 优化
- 单节点存储，无需 RPC

---

### 3.4 并行执行

#### Nebula-Graph

- 使用 folly::Executor 和 folly::Future
- runMultiJobs 方法支持并行执行
- 通过 folly::collectAll 收集结果

#### GraphDB

- 使用 Rust 的 async/await 和 tokio
- 并行连接执行器（join/parallel.rs）
- 更轻量级的并发模型

---

### 3.5 内存管理

#### Nebula-Graph

- 使用 MemoryTracker 跟踪内存使用
- checkMemoryWatermark 检查内存水位
- 支持内存超限错误

#### GraphDB

- 依赖 Rust 的所有权系统
- 使用 Arc<Mutex<>> 共享存储
- 更安全的内存管理

---

## 四、设计理念差异

### 4.1 Nebula-Graph 设计理念

1. **分布式优先**: 所有设计都考虑分布式场景
2. **执行器粒度细**: 每个操作都是独立执行器
3. **RPC 通信**: 通过 RPC 与存储层交互
4. **复杂拓扑**: 支持复杂的执行计划拓扑
5. **资源管理**: 完善的内存和资源管理

### 4.2 GraphDB 设计理念

1. **单节点简化**: 移除分布式相关功能
2. **模块化组织**: 按功能分类组织执行器
3. **直接访问**: 直接访问存储层，无需 RPC
4. **类型安全**: 利用 Rust 类型系统保证安全
5. **性能优先**: 批量优化和轻量级并发

---

## 五、功能对比总结

### 5.1 已实现功能

| 功能类别 | Nebula-Graph | GraphDB | 实现度 |
|---------|--------------|---------|--------|
| 数据扫描 | ✓ | ✓ | 100% |
| 数据过滤 | ✓ | ✓ | 100% |
| 数据投影 | ✓ | ✓ | 100% |
| 聚合 | ✓ | ✓ | 100% |
| 排序 | ✓ | ✓ | 100% |
| 限制 | ✓ | ✓ | 100% |
| 连接 | ✓ | ✓ | 100% |
| 集合操作 | ✓ | ✓ | 100% |
| 图遍历 | ✓ | ✓ | 80% |
| 最短路径 | ✓ | ✓ | 100% |
| 数据修改 | ✓ | ✓ | 100% |
| 索引管理 | ✓ | ✓ | 80% |
| 用户管理 | ✓ | ✓ | 60% |

### 5.2 未实现功能

| 功能 | Nebula-Graph | GraphDB | 原因 |
|------|--------------|---------|------|
| 全文索引 | ✓ | ✗ | 非核心功能 |
| 所有路径算法 | ✓ | ✗ | 非核心功能 |
| 多源最短路径 | ✓ | ✗ | 非核心功能 |
| 子图提取 | ✓ | ✗ | 非核心功能 |
| 主机管理 | ✓ | ✗ | 分布式功能 |
| 分区管理 | ✓ | ✗ | 分布式功能 |
| 快照 | ✓ | ✗ | 分布式功能 |
| 作业系统 | ✓ | ✗ | 分布式功能 |
| 配置管理 | ✓ | ✗ | 简化 |

### 5.3 架构优化

| 优化项 | Nebula-Graph | GraphDB | 优势 |
|--------|--------------|---------|------|
| 执行器组织 | 扁平化 | 模块化 | GraphDB 更清晰 |
| 存储访问 | RPC | 直接访问 | GraphDB 更高效 |
| 并发模型 | folly::Future | async/await | GraphDB 更轻量 |
| 内存管理 | 手动跟踪 | 所有权系统 | GraphDB 更安全 |
| 批量优化 | 部分支持 | 统一优化 | GraphDB 更完善 |
| 类型安全 | 运行时检查 | 编译时检查 | GraphDB 更安全 |

---

## 六、关键设计决策

### 6.1 为什么移除分布式功能？

**原因：**
1. 项目定位为单节点图数据库
2. 减少外部依赖
3. 简化架构，提高性能
4. 降低开发和维护成本

### 6.2 为什么重新组织执行器？

**原因：**
1. Nebula-Graph 的执行器数量过多（50+）
2. 按功能分类更易于理解和维护
3. 减少代码重复
4. 提高可扩展性

### 6.3 为什么使用 trait 而不是继承？

**原因：**
1. Rust 不支持传统继承
2. trait 提供更灵活的组合方式
3. 可以实现多个 trait
4. 更符合 Rust 的设计哲学

### 6.4 为什么引入 BatchOptimizer？

**原因：**
1. 单节点场景下批量操作更高效
2. 减少存储访问次数
3. 提高查询性能
4. 统一的批量优化接口

---

## 七、未来改进方向

### 7.1 短期改进

1. **完善图遍历**: 实现更多遍历算法
2. **优化连接**: 改进连接性能
3. **增强索引**: 支持更多索引类型
4. **完善测试**: 增加测试覆盖率

### 7.2 中期改进

1. **查询优化**: 完善查询优化器
2. **统计信息**: 收集和使用统计信息
3. **缓存机制**: 实现结果缓存
4. **性能分析**: 增强性能分析工具

### 7.3 长期改进

1. **全文索引**: 实现全文搜索
2. **高级算法**: 实现更多图算法
3. **流式处理**: 支持流式查询
4. **插件系统**: 支持自定义扩展

---

## 八、结论

GraphDB 在保留 Nebula-Graph 核心功能的基础上，通过以下方式实现了架构简化：

1. **移除分布式功能**: 专注于单节点场景
2. **重新组织执行器**: 按功能分类，结构更清晰
3. **优化存储访问**: 直接访问存储层，提高效率
4. **利用 Rust 特性**: 类型安全、内存安全、并发安全
5. **统一批量优化**: 提高查询性能

这些改进使得 GraphDB 更适合个人使用和小规模应用场景，同时保持了高性能和易用性。
