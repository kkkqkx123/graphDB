# Executor 模块拆分与重构方案

## 文档概述

本文档定义了对 `src/query/executor/` 目录下已过大单文件的拆分方案，用于改进代码可维护性、模块化程度和后续扩展性。该方案基于 `executor_module_migration.md` 和 `executor_mapping_table.md` 的指导。

**制定日期**: 2025-12-09  
**适用版本**: GraphDB Rust 版  
**状态**: 待实施

---

## 现状分析

### 当前问题

#### 1. **文件过大**
- `data_processing.rs`: ~650 行，包含 8 个执行器
  - FilterExecutor、ProjectExecutor、SortExecutor、ExpandExecutor
  - ExpandAllExecutor、TraverseExecutor、ShortestPathExecutor、AggregateExecutor
- `result_processing.rs`: ~548 行，包含 5 个执行器
  - LimitExecutor、OffsetExecutor、DistinctExecutor、SampleExecutor、TopNExecutor

#### 2. **功能分类混乱**
- `data_processing.rs` 中混合了：
  - **图遍历操作**：ExpandExecutor、ExpandAllExecutor、TraverseExecutor、ShortestPathExecutor
  - **集合/聚合操作**：FilterExecutor
  - **投影和排序**：ProjectExecutor、SortExecutor
  - **聚合函数**：AggregateExecutor
  
- `result_processing.rs` 中应该只有结果处理，但包含了：
  - **限制操作**：LimitExecutor、OffsetExecutor
  - **去重操作**：DistinctExecutor
  - **采样操作**：SampleExecutor
  - **排序优化**：TopNExecutor

#### 3. **缺乏扩展性**
- 根据 `executor_mapping_table.md`：
  - `data_processing.rs` 应包含 27 个执行器，现仅 8 个
  - 需要添加：JoinExecutor、LeftJoinExecutor、InnerJoinExecutor、UnionExecutor、UnwindExecutor、IntersectExecutor、MinusExecutor、AppendVerticesExecutor、AssignExecutor、ValueExecutor、PatternApplyExecutor、RollUpApplyExecutor、LoopExecutor、AllPathsExecutor、SubgraphExecutor 等
  - 这将导致文件更加臃肿

#### 4. **依赖关系复杂**
- 多个执行器有相似的实现模式
- 图遍历相关的执行器之间存在层级关系

---

## 拆分方案

### 目标

1. **按功能将 `data_processing.rs` 拆分为多个子模块**
2. **按功能将 `result_processing.rs` 拆分为多个子模块**
3. **建立清晰的模块层级和依赖关系**
4. **为后续的 27 个执行器预留扩展空间**

### 原则

1. **单一职责**：每个文件/目录专注于一类功能
2. **合理粒度**：避免过度拆分，每个文件 200-400 行为宜
3. **清晰命名**：模块名称直观反映职责
4. **易于导出**：统一的 `mod.rs` 或 `mod/` 目录模式

---

## 详细拆分方案

### 第一步：拆分 `data_processing.rs`

#### 新目录结构

```
src/query/executor/data_processing/
├── mod.rs                    # 模块导出和重新导出
├── filter.rs                 # FilterExecutor （条件过滤）
├── graph_traversal/          # 图遍历相关执行器
│   ├── mod.rs
│   ├── expand.rs             # ExpandExecutor（单步扩展）
│   ├── expand_all.rs         # ExpandAllExecutor（全路径扩展）
│   ├── traverse.rs           # TraverseExecutor（完整遍历）
│   └── shortest_path.rs      # ShortestPathExecutor + 相关算法
├── set_operations/           # 集合运算（并、交、差）
│   ├── mod.rs
│   ├── union.rs              # UnionExecutor
│   ├── union_all.rs          # UnionAllVersionVarExecutor
│   ├── intersect.rs          # IntersectExecutor
│   └── minus.rs              # MinusExecutor（EXCEPT）
├── join/                     # JOIN 操作（已有空目录，迁移至此）
│   ├── mod.rs
│   ├── inner_join.rs         # InnerJoinExecutor / JoinExecutor
│   ├── left_join.rs          # LeftJoinExecutor
│   └── cross_join.rs         # CartesianProductExecutor
├── transformations/          # 数据转换操作
│   ├── mod.rs
│   ├── assign.rs             # AssignExecutor（变量赋值）
│   ├── append_vertices.rs    # AppendVerticesExecutor
│   ├── unwind.rs             # UnwindExecutor（列表展开）
│   ├── pattern_apply.rs      # PatternApplyExecutor
│   └── rollup.rs             # RollUpApplyExecutor
└── loops.rs                  # LoopExecutor（循环控制）
```

#### 迁移映射

| 原位置 | 新位置 | 执行器名称 |
|-------|-------|---------|
| data_processing.rs | data_processing/filter.rs | FilterExecutor |
| data_processing.rs | data_processing/graph_traversal/expand.rs | ExpandExecutor |
| data_processing.rs | data_processing/graph_traversal/expand_all.rs | ExpandAllExecutor |
| data_processing.rs | data_processing/graph_traversal/traverse.rs | TraverseExecutor |
| data_processing.rs | data_processing/graph_traversal/shortest_path.rs | ShortestPathExecutor, ShortestPathAlgorithm |
| result_processing.rs | data_processing/set_operations/union.rs | 待添加（迁移自新实现） |
| result_processing.rs | data_processing/transformations/assign.rs | 待添加（迁移自新实现） |
| - | data_processing/mod.rs | 重新导出所有子模块 |

#### `data_processing/mod.rs` 示例

```rust
// 图遍历执行器
pub mod graph_traversal;
pub use graph_traversal::{
    ExpandExecutor, ExpandAllExecutor, TraverseExecutor,
    ShortestPathExecutor, ShortestPathAlgorithm,
};

// 集合运算执行器
pub mod set_operations;
pub use set_operations::{
    UnionExecutor, UnionAllVersionVarExecutor,
    IntersectExecutor, MinusExecutor,
};

// JOIN 执行器
pub mod join;
pub use join::{
    JoinExecutor, InnerJoinExecutor, LeftJoinExecutor,
    CartesianProductExecutor,
};

// 条件过滤
pub mod filter;
pub use filter::FilterExecutor;

// 数据转换
pub mod transformations;
pub use transformations::{
    AssignExecutor, AppendVerticesExecutor, UnwindExecutor,
    PatternApplyExecutor, RollUpApplyExecutor,
};

// 循环控制
pub mod loops;
pub use loops::LoopExecutor;
```

---

### 第二步：拆分 `result_processing.rs`

#### 新目录结构

```
src/query/executor/result_processing/
├── mod.rs                    # 模块导出和重新导出
├── projection.rs             # ProjectExecutor（列投影）
├── aggregation.rs            # AggregateExecutor（聚合函数）
├── sorting.rs                # SortExecutor（排序操作）
├── limiting.rs               # LimitExecutor、OffsetExecutor（结果限制）
├── dedup.rs                  # DistinctExecutor（去重）
├── sampling.rs               # SampleExecutor（采样）
└── topn.rs                   # TopNExecutor（排序优化）
```

#### 迁移映射

| 原位置 | 新位置 | 执行器名称 |
|-------|-------|---------|
| result_processing.rs | result_processing/projection.rs | ProjectExecutor |
| result_processing.rs | result_processing/aggregation.rs | AggregateExecutor |
| result_processing.rs | result_processing/sorting.rs | SortExecutor |
| result_processing.rs | result_processing/limiting.rs | LimitExecutor, OffsetExecutor |
| result_processing.rs | result_processing/dedup.rs | DistinctExecutor |
| result_processing.rs | result_processing/sampling.rs | SampleExecutor |
| result_processing.rs | result_processing/topn.rs | TopNExecutor |

#### `result_processing/mod.rs` 示例

```rust
// 列投影
pub mod projection;
pub use projection::ProjectExecutor;

// 聚合函数
pub mod aggregation;
pub use aggregation::AggregateExecutor;

// 排序
pub mod sorting;
pub use sorting::SortExecutor;

// 结果限制
pub mod limiting;
pub use limiting::{LimitExecutor, OffsetExecutor};

// 去重
pub mod dedup;
pub use dedup::DistinctExecutor;

// 采样
pub mod sampling;
pub use sampling::SampleExecutor;

// TOP N 优化
pub mod topn;
pub use topn::TopNExecutor;
```

---

### 第三步：更新 `executor/mod.rs`

#### 当前结构
```rust
pub mod base;
pub mod data_access;
pub mod data_processing;
pub mod data_modification;
pub mod result_processing;
```

#### 新结构（无需改变，模块内部拆分透明）
```rust
pub mod base;
pub mod data_access;
pub mod data_processing;      // 现为目录，内部自动拆分
pub mod data_modification;
pub mod result_processing;    // 现为目录，内部自动拆分
```

#### 不变的公共导出
```rust
// Re-export the base types
pub use base::{
    Executor, ExecutionResult, ExecutionContext, BaseExecutor,
    InputExecutor, ChainableExecutor, EdgeDirection, StartExecutor
};

// Re-export data access executors
pub use data_access::{
    GetVerticesExecutor, GetEdgesExecutor, GetNeighborsExecutor, GetPropExecutor
};

// Re-export data processing executors
pub use data_processing::{
    FilterExecutor, ProjectExecutor, SortExecutor, AggregateExecutor,
    ExpandExecutor, ExpandAllExecutor, TraverseExecutor, ShortestPathExecutor,
    ShortestPathAlgorithm
};

// Re-export data modification executors
pub use data_modification::{
    InsertExecutor, UpdateExecutor, DeleteExecutor,
    CreateIndexExecutor, DropIndexExecutor, VertexUpdate, EdgeUpdate, IndexType
};

// Re-export result processing executors
pub use result_processing::{
    LimitExecutor, OffsetExecutor, DistinctExecutor, SampleExecutor, TopNExecutor,
    ProjectExecutor, AggregateExecutor, SortExecutor
};
```

---

## 实施步骤

### 阶段 1：基础设施（1 天）

1. **创建目录结构**
   ```bash
   mkdir -p src/query/executor/data_processing/graph_traversal
   mkdir -p src/query/executor/data_processing/set_operations
   mkdir -p src/query/executor/data_processing/transformations
   mkdir -p src/query/executor/result_processing
   ```

2. **保留 `join_ops/` 和 `set_ops/` 目录** （已存在）
   - 将被重构为 `data_processing/join/` 和 `data_processing/set_operations/`

### 阶段 2：迁移 `data_processing.rs` （2-3 天）

#### 步骤 1：图遍历模块
1. 创建 `data_processing/graph_traversal/mod.rs`
2. 迁移 `ExpandExecutor` → `data_processing/graph_traversal/expand.rs`
3. 迁移 `ExpandAllExecutor` → `data_processing/graph_traversal/expand_all.rs`
4. 迁移 `TraverseExecutor` → `data_processing/graph_traversal/traverse.rs`
5. 迁移 `ShortestPathExecutor` + `ShortestPathAlgorithm` → `data_processing/graph_traversal/shortest_path.rs`
6. 验证编译

#### 步骤 2：其他模块
1. 创建 `data_processing/filter.rs` 并迁移 `FilterExecutor`
2. 创建 `data_processing/set_operations/` 目录（为后续 Union、Intersect、Minus 预留）
3. 创建 `data_processing/join/` 目录（为后续 Join 操作预留）
4. 创建 `data_processing/transformations/` 目录（为后续 Assign、AppendVertices 等预留）
5. 创建 `data_processing/loops.rs` 预留空间

#### 步骤 3：更新 mod 文件
1. 创建/更新 `data_processing/mod.rs`
2. 验证所有导出正确

### 阶段 3：迁移 `result_processing.rs` （2-3 天）

#### 步骤 1：分离文件
1. 创建 `result_processing/projection.rs` 并迁移 `ProjectExecutor`
2. 创建 `result_processing/aggregation.rs` 并迁移 `AggregateExecutor`
3. 创建 `result_processing/sorting.rs` 并迁移 `SortExecutor`
4. 创建 `result_processing/limiting.rs` 并迁移 `LimitExecutor`、`OffsetExecutor`
5. 创建 `result_processing/dedup.rs` 并迁移 `DistinctExecutor`
6. 创建 `result_processing/sampling.rs` 并迁移 `SampleExecutor`
7. 创建 `result_processing/topn.rs` 并迁移 `TopNExecutor`

#### 步骤 2：更新 mod 文件
1. 创建/更新 `result_processing/mod.rs`
2. 验证所有导出正确

### 阶段 4：验证与测试（1 天）

1. **检查编译**
   ```bash
   cargo check
   cargo build
   ```

2. **运行测试**
   ```bash
   cargo test
   ```

3. **检查导出**
   - 验证所有公共 API 仍然可访问
   - 验证模块重导出正确

4. **删除原文件**
   - 删除 `data_processing.rs`
   - 删除 `result_processing.rs`

---

## 后续扩展规划

### 为 `data_processing` 添加新执行器

当添加以下执行器时，应放在相应的子模块中：

#### 集合运算（`set_operations/`）
- `UnionExecutor` → `union.rs`
- `UnionAllVersionVarExecutor` → `union_all.rs`
- `IntersectExecutor` → `intersect.rs`
- `MinusExecutor` → `minus.rs`

#### JOIN 操作（`join/`）
- `JoinExecutor` / `InnerJoinExecutor` → `inner_join.rs`
- `LeftJoinExecutor` → `left_join.rs`
- `CartesianProductExecutor` → `cross_join.rs`

#### 数据转换（`transformations/`）
- `AssignExecutor` → `assign.rs`
- `AppendVerticesExecutor` → `append_vertices.rs`
- `UnwindExecutor` → `unwind.rs`
- `PatternApplyExecutor` → `pattern_apply.rs`
- `RollUpApplyExecutor` → `rollup.rs`

#### 其他
- `ValueExecutor` → 新文件 `data_processing/value.rs`
- `LoopExecutor` → `loops.rs`
- `AllPathsExecutor` → `graph_traversal/all_paths.rs`
- `SubgraphExecutor` → `graph_traversal/subgraph.rs`

### 为 `result_processing` 添加新执行器

未来如添加其他结果处理执行器，在 `result_processing/` 中新建对应文件：
- `SetExecutor` → `set.rs`
- `DataCollectExecutor` → `collect.rs`

---

## 文件清单

### 需要创建的文件

#### data_processing 子目录
```
src/query/executor/data_processing/
├── mod.rs                                    (新建)
├── filter.rs                                 (从 data_processing.rs 迁移)
├── graph_traversal/
│   ├── mod.rs                                (新建)
│   ├── expand.rs                             (从 data_processing.rs 迁移)
│   ├── expand_all.rs                         (从 data_processing.rs 迁移)
│   ├── traverse.rs                           (从 data_processing.rs 迁移)
│   └── shortest_path.rs                      (从 data_processing.rs 迁移)
├── set_operations/
│   └── mod.rs                                (新建，为后续扩展预留)
├── join/
│   └── mod.rs                                (新建，为后续扩展预留)
├── transformations/
│   └── mod.rs                                (新建，为后续扩展预留)
└── loops.rs                                  (新建，为后续扩展预留)
```

#### result_processing 子目录
```
src/query/executor/result_processing/
├── mod.rs                                    (新建)
├── projection.rs                             (从 result_processing.rs 迁移)
├── aggregation.rs                            (从 result_processing.rs 迁移)
├── sorting.rs                                (从 result_processing.rs 迁移)
├── limiting.rs                               (从 result_processing.rs 迁移)
├── dedup.rs                                  (从 result_processing.rs 迁移)
├── sampling.rs                               (从 result_processing.rs 迁移)
└── topn.rs                                   (从 result_processing.rs 迁移)
```

### 需要删除的文件

```
src/query/executor/data_processing.rs         (删除，内容已拆分)
src/query/executor/result_processing.rs       (删除，内容已拆分)
```

### 需要保留的文件

```
src/query/executor/mod.rs                     (更新导出)
src/query/executor/base.rs                    (保持不变)
src/query/executor/data_access.rs             (保持不变)
src/query/executor/data_modification.rs       (保持不变)
```

---

## 风险评估与缓解

### 潜在风险

| 风险 | 概率 | 影响 | 缓解措施 |
|-----|------|------|--------|
| 循环依赖 | 中 | 编译失败 | 确保子模块间单向依赖；使用 trait 解耦 |
| 导出遗漏 | 中 | 外部使用失败 | 逐一检查所有公共 API 导出 |
| 迁移错误 | 低 | 代码损坏 | 使用 git 进行版本控制；逐步迁移 |
| 测试覆盖不足 | 中 | 功能回归 | 运行完整测试套件；必要时添加新测试 |

---

## 验收标准

1. **编译通过**
   - `cargo check` 无错误
   - `cargo build` 成功
   - `cargo build --release` 成功

2. **功能完整**
   - 所有原有执行器功能保持
   - 所有公共 API 仍可访问
   - 所有现有测试通过

3. **代码质量**
   - 无 clippy 警告
   - 代码格式正确（`cargo fmt`）
   - 文档注释完整

4. **可维护性提升**
   - 每个文件 ≤ 400 行
   - 模块职责清晰
   - 易于后续扩展

---

## 后续文档更新

完成重构后，应更新以下文档：

1. **README.md** - 更新项目结构说明
2. **executor_module_migration.md** - 补充实际实现细节
3. **本文档** - 标记为"已完成"并记录完成日期

---

## 附录：迁移检查清单

### data_processing 迁移

- [ ] 创建 `graph_traversal/` 目录
- [ ] 创建 `graph_traversal/mod.rs`
- [ ] 迁移 `ExpandExecutor` → `graph_traversal/expand.rs`
- [ ] 迁移 `ExpandAllExecutor` → `graph_traversal/expand_all.rs`
- [ ] 迁移 `TraverseExecutor` → `graph_traversal/traverse.rs`
- [ ] 迁移 `ShortestPathExecutor` + `ShortestPathAlgorithm` → `graph_traversal/shortest_path.rs`
- [ ] 创建并迁移 `FilterExecutor` → `filter.rs`
- [ ] 创建 `set_operations/mod.rs`
- [ ] 创建 `join/mod.rs`
- [ ] 创建 `transformations/mod.rs`
- [ ] 创建 `loops.rs`
- [ ] 创建 `data_processing/mod.rs`
- [ ] 验证编译
- [ ] 运行测试
- [ ] 删除原 `data_processing.rs`

### result_processing 迁移

- [ ] 创建 `projection.rs` 并迁移 `ProjectExecutor`
- [ ] 创建 `aggregation.rs` 并迁移 `AggregateExecutor`
- [ ] 创建 `sorting.rs` 并迁移 `SortExecutor`
- [ ] 创建 `limiting.rs` 并迁移 `LimitExecutor`、`OffsetExecutor`
- [ ] 创建 `dedup.rs` 并迁移 `DistinctExecutor`
- [ ] 创建 `sampling.rs` 并迁移 `SampleExecutor`
- [ ] 创建 `topn.rs` 并迁移 `TopNExecutor`
- [ ] 创建 `result_processing/mod.rs`
- [ ] 验证编译
- [ ] 运行测试
- [ ] 删除原 `result_processing.rs`

### 最终验证

- [ ] `cargo check` 无错误
- [ ] `cargo test` 通过
- [ ] `cargo fmt` 格式检查
- [ ] `cargo clippy` 无警告
- [ ] 所有公共 API 仍可访问
- [ ] 文档更新完成
- [ ] 提交代码到 git

---

**文档版本**: v1.0  
**最后更新**: 2025-12-09  
**维护者**: GraphDB 开发团队
