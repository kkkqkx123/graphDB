# 执行器工厂模块集成完成方案

## 文档信息

- **创建日期**：2026-03-09
- **版本**：v1.0
- **作者**：GraphDB 项目组
- **状态**：待实施

---

## 1. 当前集成状态分析

### 1.1 已完成的集成

工厂模块已成功集成约 **55+** 种计划节点类型，覆盖以下类别：

| 类别 | 已完成 | 状态 |
|------|--------|------|
| 数据访问 | 6种 | ✅ 完整 |
| 数据处理 | 8种 | ✅ 完整 |
| 连接操作 | 6种 | ✅ 完整 |
| 集合操作 | 3种 | ✅ 完整 |
| 图遍历 | 3种 | ⚠️ 部分 |
| 数据转换 | 5种 | ✅ 完整 |
| 控制流 | 5种 | ✅ 完整 |
| 管理操作 | 25+种 | ⚠️ 部分 |

### 1.2 缺失的集成

通过对比 [PlanNodeEnum](file:///d:/项目/database/graphDB/src/query/planner/plan/core/nodes/plan_node_enum.rs) 和 [ExecutorFactory](file:///d:/项目/database/graphDB/src/query/executor/factory/executor_factory.rs)，发现以下 **16种** 计划节点类型尚未在工厂中处理：

#### 优先级 P0 - 核心功能缺失

| 节点类型 | 对应执行器 | 说明 | 影响 |
|---------|-----------|------|------|
| `InsertVertices` | `InsertExecutor` | 插入顶点 | DML核心功能 |
| `InsertEdges` | `InsertExecutor` | 插入边 | DML核心功能 |
| `GetEdges` | `GetEdgesExecutor` | 获取边 | 数据访问 |
| `Remove` | `RemoveExecutor` | 删除数据 | DML核心功能 |

#### 优先级 P1 - 图算法功能

| 节点类型 | 对应执行器 | 说明 | 影响 |
|---------|-----------|------|------|
| `AllPaths` | `AllPathsExecutor` | 所有路径查询 | 图算法 |
| `BFSShortest` | `BFSShortestExecutor` | BFS最短路径 | 图算法 |
| `MultiShortestPath` | - | 多源最短路径 | 图算法 |
| `ShortestPath` | `ShortestPathExecutor` | 最短路径 | 图算法 |
| `IndexScan` | `IndexScanExecutor` | 索引扫描 | 数据访问 |

#### 优先级 P2 - 管理功能完善

| 节点类型 | 对应执行器 | 说明 | 影响 |
|---------|-----------|------|------|
| `GrantRole` | `GrantRoleExecutor` | 授权角色 | 权限管理 |
| `RevokeRole` | `RevokeRoleExecutor` | 撤销角色 | 权限管理 |
| `ShowStats` | `ShowStatsExecutor` | 显示统计 | 管理功能 |
| `SwitchSpace` | `SwitchSpaceExecutor` | 切换空间 | 空间管理 |
| `AlterSpace` | `AlterSpaceExecutor` | 修改空间 | 空间管理 |
| `ClearSpace` | `ClearSpaceExecutor` | 清空空间 | 空间管理 |

#### 优先级 P3 - 其他

| 节点类型 | 对应执行器 | 说明 | 影响 |
|---------|-----------|------|------|
| `Materialize` | - | 物化节点 | 查询优化 |

---

## 2. 分阶段实施计划

### 阶段一：核心DML功能完善（P0）

**目标**：完成数据操作语言的核心功能

**预计工作量**：3-4天

#### 2.1.1 新增 DataModificationBuilder

创建 `src/query/executor/factory/builders/data_modification_builder.rs`：

```rust
//! 数据修改执行器构建器
//!
//! 负责创建数据修改类型的执行器（InsertVertices, InsertEdges, Remove）

use crate::core::error::QueryError;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::data_modification::{InsertExecutor, RemoveExecutor};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planner::plan::core::nodes::{
    InsertVerticesNode, InsertEdgesNode, RemoveNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 数据修改执行器构建器
pub struct DataModificationBuilder<S: StorageClient + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + 'static> DataModificationBuilder<S> {
    /// 创建新的数据修改构建器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// 构建 InsertVertices 执行器
    pub fn build_insert_vertices(
        &self,
        node: &InsertVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // 实现逻辑
    }

    /// 构建 InsertEdges 执行器
    pub fn build_insert_edges(
        &self,
        node: &InsertEdgesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // 实现逻辑
    }

    /// 构建 Remove 执行器
    pub fn build_remove(
        &self,
        node: &RemoveNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // 实现逻辑
    }
}
```

#### 2.1.2 更新 DataAccessBuilder

在 `data_access_builder.rs` 中添加 `GetEdges` 支持：

```rust
/// 构建 GetEdges 执行器
pub fn build_get_edges(
    &self,
    node: &GetEdgesNode,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    // 实现逻辑
}
```

#### 2.1.3 更新 Builders 集合

修改 `src/query/executor/factory/builders/mod.rs`：

```rust
pub mod data_modification_builder;

pub use data_modification_builder::DataModificationBuilder;

pub struct Builders<S: StorageClient + 'static> {
    // ... 现有字段
    data_modification: DataModificationBuilder<S>,
}

impl<S: StorageClient + 'static> Builders<S> {
    pub fn new() -> Self {
        Self {
            // ... 现有初始化
            data_modification: DataModificationBuilder::new(),
        }
    }

    /// 获取数据修改构建器
    pub fn data_modification(&self) -> &DataModificationBuilder<S> {
        &self.data_modification
    }
}
```

#### 2.1.4 更新 ExecutorFactory

在 `executor_factory.rs` 的 `create_executor` 方法中添加：

```rust
// 数据修改执行器
PlanNodeEnum::InsertVertices(node) => {
    self.builders.data_modification().build_insert_vertices(node, storage, context)
}
PlanNodeEnum::InsertEdges(node) => {
    self.builders.data_modification().build_insert_edges(node, storage, context)
}
PlanNodeEnum::Remove(node) => {
    self.builders.data_modification().build_remove(node, storage, context)
}

// 数据访问执行器 - 新增
PlanNodeEnum::GetEdges(node) => {
    self.builders.data_access().build_get_edges(node, storage, context)
}
```

#### 2.1.5 更新 ExecutorEnum

确保 `ExecutorEnum` 包含以下变体：

```rust
pub enum ExecutorEnum<S: StorageClient + Send + 'static> {
    // ... 现有变体
    InsertVertices(InsertExecutor<S>),
    InsertEdges(InsertExecutor<S>),
    GetEdges(GetEdgesExecutor<S>),
    Remove(RemoveExecutor<S>),
}
```

**验收标准**：
- [ ] `InsertVertices` 节点可以正确创建执行器
- [ ] `InsertEdges` 节点可以正确创建执行器
- [ ] `GetEdges` 节点可以正确创建执行器
- [ ] `Remove` 节点可以正确创建执行器
- [ ] 所有相关测试通过

---

### 阶段二：图算法功能完善（P1）

**目标**：完成图遍历算法相关功能

**预计工作量**：2-3天

#### 2.2.1 更新 TraversalBuilder

扩展 `traversal_builder.rs`：

```rust
use crate::query::executor::data_processing::graph_traversal::{
    AllPathsExecutor, ShortestPathExecutor,
};
use crate::query::planner::plan::algorithms::{
    AllPaths, BFSShortest, MultiShortestPath, ShortestPath,
};

/// 构建 AllPaths 执行器
pub fn build_all_paths(
    &self,
    node: &AllPaths,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    // 实现逻辑
}

/// 构建 ShortestPath 执行器
pub fn build_shortest_path(
    &self,
    node: &ShortestPath,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    // 实现逻辑
}

/// 构建 BFSShortest 执行器
pub fn build_bfs_shortest(
    &self,
    node: &BFSShortest,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    // 实现逻辑
}

/// 构建 MultiShortestPath 执行器
pub fn build_multi_shortest_path(
    &self,
    node: &MultiShortestPath,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    // 实现逻辑或返回不支持错误
}
```

#### 2.2.2 更新 DataAccessBuilder

添加 `IndexScan` 支持：

```rust
use crate::query::planner::plan::algorithms::IndexScan;

/// 构建 IndexScan 执行器
pub fn build_index_scan(
    &self,
    node: &IndexScan,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    // 实现逻辑
}
```

#### 2.2.3 更新 ExecutorFactory

```rust
// 图遍历执行器 - 新增
PlanNodeEnum::AllPaths(node) => {
    self.builders.traversal().build_all_paths(node, storage, context)
}
PlanNodeEnum::ShortestPath(node) => {
    self.builders.traversal().build_shortest_path(node, storage, context)
}
PlanNodeEnum::BFSShortest(node) => {
    self.builders.traversal().build_bfs_shortest(node, storage, context)
}
PlanNodeEnum::MultiShortestPath(node) => {
    self.builders.traversal().build_multi_shortest_path(node, storage, context)
}

// 数据访问执行器 - 新增
PlanNodeEnum::IndexScan(node) => {
    self.builders.data_access().build_index_scan(node, storage, context)
}
```

**验收标准**：
- [ ] `AllPaths` 节点可以正确创建执行器
- [ ] `ShortestPath` 节点可以正确创建执行器
- [ ] `BFSShortest` 节点可以正确创建执行器
- [ ] `IndexScan` 节点可以正确创建执行器
- [ ] `MultiShortestPath` 有明确的处理策略（实现或报错）

---

### 阶段三：管理功能完善（P2）

**目标**：完成管理操作相关功能

**预计工作量**：2天

#### 2.3.1 更新 AdminBuilder

扩展 `admin_builder.rs`：

```rust
use crate::query::executor::admin::{
    GrantRoleExecutor, RevokeRoleExecutor, ShowStatsExecutor,
    SwitchSpaceExecutor, AlterSpaceExecutor, ClearSpaceExecutor,
};
use crate::query::planner::plan::core::nodes::{
    GrantRoleNode, RevokeRoleNode, ShowStatsNode,
    SwitchSpaceNode, AlterSpaceNode, ClearSpaceNode,
};

// ========== 权限管理执行器 ==========

/// 构建 GrantRole 执行器
pub fn build_grant_role(
    &self,
    node: &GrantRoleNode,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    // 实现逻辑
}

/// 构建 RevokeRole 执行器
pub fn build_revoke_role(
    &self,
    node: &RevokeRoleNode,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    // 实现逻辑
}

// ========== 空间管理执行器（补充） ==========

/// 构建 SwitchSpace 执行器
pub fn build_switch_space(
    &self,
    node: &SwitchSpaceNode,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    // 实现逻辑
}

/// 构建 AlterSpace 执行器
pub fn build_alter_space(
    &self,
    node: &AlterSpaceNode,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    // 实现逻辑
}

/// 构建 ClearSpace 执行器
pub fn build_clear_space(
    &self,
    node: &ClearSpaceNode,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    // 实现逻辑
}

// ========== 查询管理执行器 ==========

/// 构建 ShowStats 执行器
pub fn build_show_stats(
    &self,
    node: &ShowStatsNode,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    // 实现逻辑
}
```

#### 2.3.2 更新 ExecutorFactory

```rust
// 管理执行器 - 权限管理
PlanNodeEnum::GrantRole(node) => {
    self.builders.admin().build_grant_role(node, storage, context)
}
PlanNodeEnum::RevokeRole(node) => {
    self.builders.admin().build_revoke_role(node, storage, context)
}

// 管理执行器 - 空间管理（补充）
PlanNodeEnum::SwitchSpace(node) => {
    self.builders.admin().build_switch_space(node, storage, context)
}
PlanNodeEnum::AlterSpace(node) => {
    self.builders.admin().build_alter_space(node, storage, context)
}
PlanNodeEnum::ClearSpace(node) => {
    self.builders.admin().build_clear_space(node, storage, context)
}

// 管理执行器 - 查询管理
PlanNodeEnum::ShowStats(node) => {
    self.builders.admin().build_show_stats(node, storage, context)
}
```

**验收标准**：
- [ ] `GrantRole` 节点可以正确创建执行器
- [ ] `RevokeRole` 节点可以正确创建执行器
- [ ] `SwitchSpace` 节点可以正确创建执行器
- [ ] `AlterSpace` 节点可以正确创建执行器
- [ ] `ClearSpace` 节点可以正确创建执行器
- [ ] `ShowStats` 节点可以正确创建执行器

---

### 阶段四：物化节点支持（P3）

**目标**：处理物化节点

**预计工作量**：1天

#### 2.4.1 决策方案

对于 `Materialize` 节点，有两种处理策略：

**方案A - 透传处理**：
```rust
PlanNodeEnum::Materialize(node) => {
    // 直接创建子执行器，物化逻辑在计划层处理
    let child = node.child()
        .ok_or_else(|| QueryError::ExecutionError("Materialize节点缺少子节点".to_string()))?;
    self.create_executor(child, storage, context)
}
```

**方案B - 创建专用执行器**：
```rust
PlanNodeEnum::Materialize(node) => {
    self.builders.data_processing().build_materialize(node, storage, context)
}
```

**推荐**：采用方案A，因为物化是计划优化层面的概念，执行器层面不需要特殊处理。

---

## 3. 实施检查清单

### 3.1 代码修改清单

#### 新增文件
- [ ] `src/query/executor/factory/builders/data_modification_builder.rs`

#### 修改文件
- [ ] `src/query/executor/factory/builders/mod.rs` - 添加 DataModificationBuilder
- [ ] `src/query/executor/factory/builders/data_access_builder.rs` - 添加 GetEdges, IndexScan
- [ ] `src/query/executor/factory/builders/data_processing_builder.rs` - 添加 Materialize（如需要）
- [ ] `src/query/executor/factory/builders/traversal_builder.rs` - 添加图算法执行器
- [ ] `src/query/executor/factory/builders/admin_builder.rs` - 添加管理执行器
- [ ] `src/query/executor/factory/executor_factory.rs` - 添加所有缺失的匹配分支
- [ ] `src/query/executor/executor_enum.rs` - 确保所有变体存在

### 3.2 测试检查清单

- [ ] 单元测试：每个新增构建方法都有对应的单元测试
- [ ] 集成测试：端到端的执行器创建测试
- [ ] 边界测试：错误处理、空值处理
- [ ] 性能测试：大规模执行器创建性能

### 3.3 文档检查清单

- [ ] 更新 `docs/query/factory_refactoring_plan.md`
- [ ] 更新 `docs/query/factory_refactoring_remaining_tasks.md`
- [ ] 更新模块文档注释

---

## 4. 风险评估与缓解策略

### 4.1 风险识别

| 风险 | 可能性 | 影响 | 缓解策略 |
|------|--------|------|----------|
| 节点类型与执行器不匹配 | 中 | 高 | 仔细核对节点字段和执行器参数 |
| 循环依赖 | 低 | 高 | 使用递归检测器验证 |
| 性能退化 | 低 | 中 | 进行基准测试对比 |
| 编译错误 | 高 | 中 | 分阶段编译验证 |

### 4.2 回滚策略

每个阶段独立提交，如发现问题可单独回滚该阶段的修改。

---

## 5. 时间线

| 阶段 | 预计时间 | 开始日期 | 结束日期 |
|------|----------|----------|----------|
| 阶段一：核心DML功能 | 3-4天 | - | - |
| 阶段二：图算法功能 | 2-3天 | - | - |
| 阶段三：管理功能 | 2天 | - | - |
| 阶段四：物化节点 | 1天 | - | - |
| **总计** | **8-10天** | - | - |

---

## 6. 附录

### 6.1 节点类型与执行器映射表

| PlanNodeEnum 变体 | ExecutorEnum 变体 | 构建器 | 状态 |
|-------------------|-------------------|--------|------|
| InsertVertices | InsertVertices | DataModificationBuilder | 待实现 |
| InsertEdges | InsertEdges | DataModificationBuilder | 待实现 |
| GetEdges | GetEdges | DataAccessBuilder | 待实现 |
| Remove | Remove | DataModificationBuilder | 待实现 |
| AllPaths | AllPaths | TraversalBuilder | 待实现 |
| BFSShortest | BFSShortest | TraversalBuilder | 待实现 |
| MultiShortestPath | - | TraversalBuilder | 待决策 |
| ShortestPath | ShortestPath | TraversalBuilder | 待实现 |
| IndexScan | IndexScan | DataAccessBuilder | 待实现 |
| GrantRole | GrantRole | AdminBuilder | 待实现 |
| RevokeRole | RevokeRole | AdminBuilder | 待实现 |
| ShowStats | ShowStats | AdminBuilder | 待实现 |
| SwitchSpace | SwitchSpace | AdminBuilder | 待实现 |
| AlterSpace | AlterSpace | AdminBuilder | 待实现 |
| ClearSpace | ClearSpace | AdminBuilder | 待实现 |
| Materialize | - | - | 待决策 |

### 6.2 相关代码引用

- [PlanNodeEnum 定义](file:///d:/项目/database/graphDB/src/query/planner/plan/core/nodes/plan_node_enum.rs)
- [ExecutorEnum 定义](file:///d:/项目/database/graphDB/src/query/executor/executor_enum.rs)
- [ExecutorFactory 实现](file:///d:/项目/database/graphDB/src/query/executor/factory/executor_factory.rs)
- [Builders 集合](file:///d:/项目/database/graphDB/src/query/executor/factory/builders/mod.rs)
