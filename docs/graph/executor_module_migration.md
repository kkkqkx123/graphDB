# NebulaGraph Executor 模块迁移方案

## 文档概述

本文档分析了 NebulaGraph 3.8.0 中 `src/graph/executor` 目录的所有模块，并为新架构中的迁移提供详细的映射方案。

**分析日期**: 2025-12-09  
**源代码**: `nebula-3.8.0/src/graph/executor`  
**目标架构**: 新 Rust GraphDB 单机版

---

## 目录结构对比

### NebulaGraph 3.8.0 执行器目录结构

```
src/graph/executor/
├── admin/           # 管理功能执行器 (20+ 个执行器)
├── algo/            # 算法执行器 (最短路径、全路径等)
├── logic/           # 逻辑控制执行器 (START、SELECT、LOOP等)
├── maintain/        # 维护执行器 (标签、边、索引管理)
├── mutate/          # 数据修改执行器 (INSERT、UPDATE、DELETE)
├── query/           # 查询执行器 (40+ 种查询操作)
├── test/            # 测试文件
├── Executor.h/cpp           # 基础执行器类
├── ExecutionError.h         # 错误定义
└── StorageAccessExecutor.h/cpp  # 存储访问基类
```

### 新 Rust 架构执行器组织

```
src/query/executor/
├── base.rs                  # 基础执行器接口和类型
├── data_access.rs           # 数据访问执行器
├── data_modification.rs      # 数据修改执行器
├── data_processing.rs       # 数据处理执行器
├── result_processing.rs     # 结果处理执行器
└── mod.rs                   # 模块导出
```

---

## 详细映射方案

### 一、核心基础模块

#### 1. Executor.h/cpp → `src/query/executor/base.rs`

**优先级**: ⭐⭐⭐ (最高)

| 原始类型 | 说明 | 迁移内容 |
|---------|------|--------|
| `Executor` | 基础执行器抽象类 | 定义 Rust trait 或 enum |
| `next()` | 获取下一行结果 | 异步结果迭代方法 |
| `getProps()` | 获取输出属性 | 结果集结构定义 |
| `finish()` | 执行完成清理 | Drop trait 实现 |

**关键设计决策**：
- 使用 Rust trait 替代虚基类
- 基于 async/await 的异步执行
- 使用 Result<T> 处理错误而非异常

```rust
/// 新架构基础执行器 trait
pub trait Executor: Send + Sync {
    /// 获取下一批结果
    async fn next(&mut self) -> Result<Vec<Row>>;
    
    /// 获取输出列信息
    fn output_schema(&self) -> &Schema;
    
    /// 执行完成或取消时清理资源
    async fn close(&mut self) -> Result<()>;
}
```

#### 2. StorageAccessExecutor.h/cpp → `src/query/executor/data_access.rs`

**优先级**: ⭐⭐⭐ (最高)

| 原始类型 | 说明 | 迁移内容 |
|---------|------|--------|
| `StorageAccessExecutor` | 存储访问基类 | 数据访问执行器基类 |
| 存储接口 | 与存储层通信 | 定义 StorageClient trait |

**实现模块**:
- `GetNeighborsExecutor` - 获取相邻节点
- `GetVerticesExecutor` - 获取节点
- `GetEdgesExecutor` - 获取边
- `IndexScanExecutor` - 索引扫描

---

### 二、查询执行器 (query/)

**目标模块**: `src/query/executor/data_access.rs` 和 `data_processing.rs`

#### 2.1 数据访问类执行器

| ExecutorName | 优先级 | 映射位置 | 依赖关系 | 说明 |
|---|---|---|---|---|
| `GetNeighborsExecutor` | ⭐⭐⭐ | `data_access.rs` | 存储引擎 | 图遍历的基础操作 |
| `GetVerticesExecutor` | ⭐⭐⭐ | `data_access.rs` | 存储引擎 | 按ID获取节点及属性 |
| `GetEdgesExecutor` | ⭐⭐⭐ | `data_access.rs` | 存储引擎 | 按ID获取边及属性 |
| `GetPropExecutor` | ⭐⭐⭐ | `data_access.rs` | GetVertices/GetEdges | 属性获取优化 |
| `IndexScanExecutor` | ⭐⭐ | `data_access.rs` | 索引引擎 | 索引扫描查询 |
| `ScanVerticesExecutor` | ⭐⭐ | `data_access.rs` | 存储引擎 | 全表扫描节点 |
| `ScanEdgesExecutor` | ⭐⭐ | `data_access.rs` | 存储引擎 | 全表扫描边 |
| `FulltextIndexScanExecutor` | ⭐ | `data_access.rs` | 全文索引引擎 | 全文索引查询 |

#### 2.2 数据处理类执行器

| ExecutorName | 优先级 | 映射位置 | 依赖关系 | 说明 |
|---|---|---|---|---|
| `FilterExecutor` | ⭐⭐⭐ | `data_processing.rs` | 表达式计算 | WHERE/FILTER 条件执行 |
| `ExpandExecutor` | ⭐⭐⭐ | `data_processing.rs` | GetNeighbors | 路径扩展，关键的图遍历操作 |
| `ExpandAllExecutor` | ⭐⭐⭐ | `data_processing.rs` | GetNeighbors | 返回所有路径 |
| `TraverseExecutor` | ⭐⭐⭐ | `data_processing.rs` | Expand | 完整的图遍历执行 |
| `JoinExecutor` | ⭐⭐ | `data_processing.rs` | 比较操作 | INNER JOIN 执行 |
| `LeftJoinExecutor` | ⭐⭐ | `data_processing.rs` | 比较操作 | LEFT OUTER JOIN 执行 |
| `InnerJoinExecutor` | ⭐⭐ | `data_processing.rs` | 比较操作 | INNER JOIN 执行 |
| `UnionExecutor` | ⭐⭐ | `data_processing.rs` | 数据合并 | UNION 执行（去重） |
| `UnionAllVersionVarExecutor` | ⭐ | `data_processing.rs` | 数据合并 | UNION ALL 执行 |
| `UnwindExecutor` | ⭐⭐ | `data_processing.rs` | 列表展开 | UNWIND 执行 |
| `IntersectExecutor` | ⭐⭐ | `data_processing.rs` | 集合运算 | INTERSECT 执行 |
| `MinusExecutor` | ⭐⭐ | `data_processing.rs` | 集合运算 | MINUS/EXCEPT 执行 |
| `AppendVerticesExecutor` | ⭐⭐ | `data_processing.rs` | GetVertices | 追加顶点信息 |
| `AssignExecutor` | ⭐⭐ | `data_processing.rs` | 变量赋值 | 变量赋值操作 |
| `ValueExecutor` | ⭐ | `data_processing.rs` | 常量评估 | 返回常量值 |
| `PatternApplyExecutor` | ⭐ | `data_processing.rs` | Pattern匹配 | 模式匹配应用 |
| `RollUpApplyExecutor` | ⭐ | `data_processing.rs` | 聚合 | ROLLUP 操作 |

#### 2.3 结果处理类执行器

| ExecutorName | 优先级 | 映射位置 | 依赖关系 | 说明 |
|---|---|---|---|---|
| `ProjectExecutor` | ⭐⭐⭐ | `result_processing.rs` | 列选择 | 选择和投影输出列 |
| `AggregateExecutor` | ⭐⭐⭐ | `result_processing.rs` | 聚合函数 | COUNT/SUM/AVG/MAX/MIN |
| `SortExecutor` | ⭐⭐⭐ | `result_processing.rs` | 比较操作 | ORDER BY 执行 |
| `LimitExecutor` | ⭐⭐⭐ | `result_processing.rs` | 结果限制 | LIMIT/OFFSET 执行 |
| `TopNExecutor` | ⭐⭐ | `result_processing.rs` | 排序限制 | TOP N 优化 |
| `DedupExecutor` | ⭐⭐ | `result_processing.rs` | 集合操作 | DISTINCT 去重执行 |
| `DataCollectExecutor` | ⭐⭐ | `result_processing.rs` | 结果收集 | 收集所有结果 |
| `SetExecutor` | ⭐ | `result_processing.rs` | 变量设置 | SET 语句执行 |
| `SampleExecutor` | ⭐ | `result_processing.rs` | 随机采样 | 随机采样结果 |

---

### 三、数据修改执行器 (mutate/)

**目标模块**: `src/query/executor/data_modification.rs`

**优先级**: ⭐⭐⭐ (最高)

| ExecutorName | 说明 | 实现细节 |
|---|---|---|
| `InsertExecutor` | 插入节点和边 | 调用存储引擎的写入接口 |
| `UpdateExecutor` | 更新节点/边属性 | 支持条件更新和批量更新 |
| `DeleteExecutor` | 删除节点和边 | 处理级联删除逻辑 |

**关键考虑**:
- 事务支持（原子性）
- 约束检查（唯一性、引用完整性等）
- 错误恢复和回滚机制

---

### 四、算法执行器 (algo/)

**目标模块**: `src/query/executor/data_processing.rs` 或独立模块 `src/query/algorithms/`

| ExecutorName | 优先级 | 说明 | 复杂度 |
|---|---|---|---|
| `ShortestPathExecutor` | ⭐⭐⭐ | Dijkstra/BFS最短路径 | O(V+E) |
| `SingleShortestPath` | ⭐⭐⭐ | 单源最短路径辅助类 | 支持类 |
| `BFSShortestPathExecutor` | ⭐⭐⭐ | BFS最短路径优化版 | O(V+E) |
| `MultiShortestPathExecutor` | ⭐⭐ | 多源最短路径 | O(K*(V+E)) |
| `BatchShortestPath` | ⭐⭐ | 批量最短路径 | 优化版 |
| `AllPathsExecutor` | ⭐⭐ | 查找所有路径 | O(2^V) |
| `SubgraphExecutor` | ⭐⭐ | 子图提取 | O(V+E) |
| `CartesianProductExecutor` | ⭐ | 笛卡尔积运算 | O(n*m) |

**建议**：将算法实现为独立的 trait，可组合到不同的执行器中。

---

### 五、维护执行器 (maintain/)

**目标模块**：`src/index/` 或需要移除

| ExecutorName | 优先级 | 新架构处理 | 说明 |
|---|---|---|---|
| `TagExecutor` | ❌ | 删除 | 分布式元数据，单机不需 |
| `EdgeExecutor` | ❌ | 删除 | 分布式元数据，单机不需 |
| `TagIndexExecutor` | ⭐⭐ | `src/index/tag_index.rs` | 标签索引管理 |
| `EdgeIndexExecutor` | ⭐⭐ | `src/index/edge_index.rs` | 边索引管理 |
| `FTIndexExecutor` | ⭐ | `src/index/fulltext_index.rs` | 全文索引管理 |

**说明**：分布式管理相关的执行器应删除，索引管理应迁移到 `src/index/` 模块。

---

### 六、管理执行器 (admin/)

**目标模块**：`src/api/` 或 `src/commands/`

#### 6.1 需要保留（适配单机）

| ExecutorName | 优先级 | 映射位置 | 说明 |
|---|---|---|---|
| `SessionExecutor` | ⭐⭐ | `src/api/session.rs` | 会话管理 |
| `SpaceExecutor` | ⭐⭐ | `src/api/space.rs` | 图空间管理 |
| `ConfigExecutor` | ⭐ | `src/api/config.rs` | 配置管理 |
| `ShowQueriesExecutor` | ⭐ | `src/api/show_queries.rs` | 显示查询统计 |
| `ShowStatsExecutor` | ⭐ | `src/api/show_stats.rs` | 显示统计信息 |
| `CharsetExecutor` | ⭐ | `src/api/charset.rs` | 字符集管理 |
| `ChangePasswordExecutor` | ⭐ | `src/api/security.rs` | 修改密码 |

#### 6.2 需要删除（分布式特定）

以下执行器为分布式特定功能，单机版不需要：

- `ShowHostsExecutor` - 显示主机列表
- `AddHostsExecutor` - 添加主机
- `DropHostsExecutor` - 删除主机
- `ZoneExecutor` - 区域管理
- `PartExecutor` - 分区管理
- `ListenerExecutor` - 监听器管理
- `SnapshotExecutor` - 快照管理
- `SubmitJobExecutor` - 分布式任务
- `KillQueryExecutor` - 删除分布式查询
- `ShowMetaLeaderExecutor` - 显示Meta Leader
- `SignInServiceExecutor` - 服务登录
- `SignOutServiceExecutor` - 服务登出
- `ShowServiceClientsExecutor` - 显示服务客户端

#### 6.3 权限管理（简化）

单机版应简化权限系统：

| ExecutorName | 处理方式 | 说明 |
|---|---|---|
| `CreateUserExecutor` | 简化 | 仅保留基本用户管理 |
| `DropUserExecutor` | 简化 | 简化实现 |
| `UpdateUserExecutor` | 简化 | 简化实现 |
| `DescribeUserExecutor` | 简化 | 简化实现 |
| `ListUsersExecutor` | 简化 | 简化实现 |
| `GrantRoleExecutor` | 删除 | 单机可不需 |
| `RevokeRoleExecutor` | 删除 | 单机可不需 |
| `ListUserRolesExecutor` | 删除 | 单机可不需 |
| `ListRolesExecutor` | 删除 | 单机可不需 |

---

### 七、逻辑控制执行器 (logic/)

**目标模块**：`src/query/scheduler/` 或 `src/query/executor/base.rs`

| ExecutorName | 优先级 | 映射位置 | 说明 |
|---|---|---|---|
| `StartExecutor` | ⭐⭐⭐ | `src/query/executor/base.rs` | 查询开始点，执行逻辑入口 |
| `SelectExecutor` | ⭐⭐⭐ | `src/query/scheduler/` | 执行计划选择和分发 |
| `LoopExecutor` | ⭐⭐ | `src/query/executor/data_processing.rs` | 循环执行（for循环结构） |
| `ArgumentExecutor` | ⭐⭐ | `src/query/executor/base.rs` | 参数传递和处理 |
| `PassThroughExecutor` | ⭐ | `src/query/executor/base.rs` | 直通执行器 |

---

## 迁移优先级与时间表

### 第一阶段：核心基础（1-2周）

**目标**: 建立基本的执行框架

1. **基础执行器框架** (`base.rs`)
   - Executor trait 定义
   - 基础错误处理
   - 执行上下文定义

2. **数据访问执行器** (`data_access.rs`)
   - GetNeighborsExecutor
   - GetVerticesExecutor
   - GetEdgesExecutor

3. **数据修改执行器** (`data_modification.rs`)
   - InsertExecutor
   - UpdateExecutor
   - DeleteExecutor

### 第二阶段：查询能力（2-3周）

**目标**: 实现完整的查询处理链

1. **数据处理执行器** (`data_processing.rs`)
   - FilterExecutor
   - ExpandExecutor / TraverseExecutor
   - JoinExecutor / UnionExecutor

2. **结果处理执行器** (`result_processing.rs`)
   - ProjectExecutor
   - AggregateExecutor
   - SortExecutor
   - LimitExecutor

### 第三阶段：高级功能（3-4周）

**目标**: 完善图算法和高级功能

1. **图算法** (`src/query/algorithms/`)
   - ShortestPathExecutor
   - AllPathsExecutor
   - SubgraphExecutor

2. **索引管理** (`src/index/`)
   - 迁移索引相关执行器

3. **API管理** (`src/api/`)
   - 管理命令执行器的适配

### 第四阶段：优化与测试（1-2周）

**目标**: 性能优化和综合测试

1. 执行器性能优化
2. 内存管理优化
3. 集成测试
4. 文档完善

---

## 架构设计建议

### 1. 执行器基类设计

```rust
/// 执行器基类
pub trait Executor: Send + Sync {
    /// 初始化执行器
    async fn init(&mut self) -> Result<()>;
    
    /// 获取下一批结果
    async fn next(&mut self) -> Result<Option<Vec<Row>>>;
    
    /// 获取输出模式
    fn output_schema(&self) -> &Schema;
    
    /// 获取执行统计信息
    fn stats(&self) -> ExecutionStats;
    
    /// 关闭执行器并释放资源
    async fn close(&mut self) -> Result<()>;
}

/// 执行上下文
pub struct ExecutionContext {
    /// 存储引擎
    storage: Arc<dyn StorageEngine>,
    
    /// 索引引擎
    index: Arc<dyn IndexEngine>,
    
    /// 变量作用域
    scope: HashMap<String, Value>,
    
    /// 执行统计
    stats: ExecutionStats,
}
```

### 2. 执行器组合模式

使用组合模式构建复杂查询：

```rust
pub struct PipelineExecutor {
    /// 执行器链
    executors: Vec<Box<dyn Executor>>,
    
    /// 上下文
    context: Arc<ExecutionContext>,
}
```

### 3. 异步执行设计

- 使用 Tokio 或 async-std 进行异步操作
- 支持流式处理和批处理
- 实现取消令牌（CancellationToken）支持

### 4. 错误处理

统一的错误类型：

```rust
pub enum ExecutionError {
    StorageError(String),
    TypeError(String),
    IndexError(String),
    QueryError(String),
    Cancelled,
}
```

---

## 文件迁移清单

### 需要迁移的核心文件

```
nebula-3.8.0/src/graph/executor/
├── Executor.h/cpp                    → src/query/executor/base.rs
├── ExecutionError.h                  → src/core/error.rs (扩展)
├── StorageAccessExecutor.h/cpp       → src/query/executor/data_access.rs
│
├── query/
│   ├── GetNeighborsExecutor.*        → data_access.rs
│   ├── GetVerticesExecutor.*         → data_access.rs
│   ├── GetEdgesExecutor.*            → data_access.rs
│   ├── FilterExecutor.*              → data_processing.rs
│   ├── ExpandExecutor.*              → data_processing.rs
│   ├── ProjectExecutor.*             → result_processing.rs
│   ├── AggregateExecutor.*           → result_processing.rs
│   ├── SortExecutor.*                → result_processing.rs
│   ├── LimitExecutor.*               → result_processing.rs
│   └── [其他查询执行器]              → 相应模块
│
├── mutate/
│   ├── InsertExecutor.*              → data_modification.rs
│   ├── UpdateExecutor.*              → data_modification.rs
│   └── DeleteExecutor.*              → data_modification.rs
│
├── algo/
│   ├── ShortestPathExecutor.*        → src/query/algorithms/shortest_path.rs
│   ├── AllPathsExecutor.*            → src/query/algorithms/all_paths.rs
│   └── [其他算法]                    → algorithms/
│
└── logic/
    ├── StartExecutor.*               → executor/base.rs (修改)
    ├── SelectExecutor.*              → scheduler/
    └── [其他逻辑]                    → 相应模块
```

### 需要删除的文件

所有分布式相关的执行器：

```
maintain/TagExecutor.*          ❌ 删除
maintain/EdgeExecutor.*         ❌ 删除
admin/ShowHostsExecutor.*       ❌ 删除
admin/ZoneExecutor.*            ❌ 删除
admin/PartExecutor.*            ❌ 删除
[其他分布式执行器]              ❌ 删除
```

---

## 依赖关系图

```
Executor (base trait)
├── StorageAccessExecutor (trait)
│   ├── GetNeighborsExecutor
│   ├── GetVerticesExecutor
│   ├── GetEdgesExecutor
│   └── IndexScanExecutor
│
├── DataProcessingExecutor
│   ├── FilterExecutor
│   ├── ExpandExecutor
│   ├── TraverseExecutor
│   ├── JoinExecutor
│   ├── UnionExecutor
│   └── LoopExecutor
│
├── ResultProcessingExecutor
│   ├── ProjectExecutor
│   ├── AggregateExecutor
│   ├── SortExecutor
│   ├── LimitExecutor
│   └── DedupExecutor
│
├── DataModificationExecutor
│   ├── InsertExecutor
│   ├── UpdateExecutor
│   └── DeleteExecutor
│
└── AlgorithmExecutor
    ├── ShortestPathExecutor
    ├── AllPathsExecutor
    └── SubgraphExecutor
```

---

## 测试策略

### 单元测试

- 为每个执行器编写单元测试
- 测试正常路径和错误路径
- 验证输出结果的正确性

### 集成测试

- 测试执行器链（Pipeline）
- 测试复杂查询执行
- 性能基准测试

### 回归测试

- 使用原 NebulaGraph 的测试用例进行对比
- 验证结果一致性

---

## 参考资源

- NebulaGraph 3.8.0 源代码文档
- Rust 异步编程最佳实践
- 相关设计文档：
  - `src/query/executor/mod.rs`
  - `src/query/scheduler/mod.rs`
  - `src/core/types.rs`

