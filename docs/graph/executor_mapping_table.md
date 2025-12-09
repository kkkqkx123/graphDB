# Executor 模块映射表（更新版）

## 文档概述

本文档基于 `executor_refactoring_plan.md` 和当前 `src/query/executor/` 目录的实际结构重新编写，提供 NebulaGraph 执行器到新 Rust 架构的完整映射表。

**更新日期**: 2025-12-09  
**应用版本**: GraphDB Rust 版  
**状态**: 已实施

---

## 快速导航

### 当前目录结构（已实施）

```
src/query/executor/
├── base.rs                           # 基础执行器接口和类型
├── data_access.rs                    # 数据访问执行器
├── data_modification.rs              # 数据修改执行器
├── data_processing/                  # 数据处理执行器（目录结构）
│   ├── mod.rs                        # 模块导出
│   ├── filter.rs                     # FilterExecutor
│   ├── loops.rs                      # LoopExecutor
│   ├── graph_traversal/
│   │   ├── mod.rs
│   │   ├── expand.rs                 # ExpandExecutor
│   │   ├── expand_all.rs             # ExpandAllExecutor
│   │   ├── traverse.rs               # TraverseExecutor
│   │   └── shortest_path.rs          # ShortestPathExecutor
│   ├── join/
│   │   └── mod.rs                    # 为后续 JOIN 操作预留
│   ├── set_operations/
│   │   └── mod.rs                    # 为后续集合运算预留
│   └── transformations/
│       └── mod.rs                    # 为后续数据转换预留
├── result_processing/                # 结果处理执行器（目录结构）
│   ├── mod.rs                        # 模块导出
│   ├── projection.rs                 # ProjectExecutor
│   ├── aggregation.rs                # AggregateExecutor
│   ├── sorting.rs                    # SortExecutor
│   ├── limiting.rs                   # LimitExecutor, OffsetExecutor
│   ├── dedup.rs                      # DistinctExecutor
│   ├── sampling.rs                   # SampleExecutor
│   └── topn.rs                       # TopNExecutor
└── mod.rs                            # 模块导出
```

---

## 按优先级排序

### ⭐⭐⭐ 第一优先级（已实施）

已实现的核心执行器：

| 执行器类名 | 文件位置 | 描述 |
|---|---|---|
| Executor | `base.rs` | 基础执行器接口（trait） |
| StartExecutor | `base.rs` | 查询起点执行器 |
| GetNeighborsExecutor | `data_access.rs` | 获取相邻节点 |
| GetVerticesExecutor | `data_access.rs` | 获取节点 |
| GetEdgesExecutor | `data_access.rs` | 获取边 |
| FilterExecutor | `data_processing/filter.rs` | WHERE/FILTER 执行 |
| ExpandExecutor | `data_processing/graph_traversal/expand.rs` | 路径扩展 |
| ExpandAllExecutor | `data_processing/graph_traversal/expand_all.rs` | 返回所有路径 |
| TraverseExecutor | `data_processing/graph_traversal/traverse.rs` | 图遍历 |
| ShortestPathExecutor | `data_processing/graph_traversal/shortest_path.rs` | 最短路径 |
| ProjectExecutor | `result_processing/projection.rs` | 列投影 |
| AggregateExecutor | `result_processing/aggregation.rs` | 聚合函数 |
| SortExecutor | `result_processing/sorting.rs` | ORDER BY |
| LimitExecutor | `result_processing/limiting.rs` | LIMIT 执行 |
| OffsetExecutor | `result_processing/limiting.rs` | OFFSET 执行 |
| InsertExecutor | `data_modification.rs` | 插入数据 |
| UpdateExecutor | `data_modification.rs` | 更新数据 |
| DeleteExecutor | `data_modification.rs` | 删除数据 |

### ⭐⭐ 第二优先级（待实施）

需要迁移的重要执行器：

| 执行器类名 | 目标位置 | 描述 | 状态 |
|---|---|---|---|
| JoinExecutor | `data_processing/join/inner_join.rs` | INNER JOIN | 预留 |
| LeftJoinExecutor | `data_processing/join/left_join.rs` | LEFT OUTER JOIN | 预留 |
| InnerJoinExecutor | `data_processing/join/inner_join.rs` | INNER JOIN | 预留 |
| UnionExecutor | `data_processing/set_operations/union.rs` | UNION（去重） | 预留 |
| UnionAllVersionVarExecutor | `data_processing/set_operations/union_all.rs` | UNION ALL | 预留 |
| UnwindExecutor | `data_processing/transformations/unwind.rs` | UNWIND | 预留 |
| IntersectExecutor | `data_processing/set_operations/intersect.rs` | INTERSECT | 预留 |
| MinusExecutor | `data_processing/set_operations/minus.rs` | MINUS/EXCEPT | 预留 |
| AppendVerticesExecutor | `data_processing/transformations/append_vertices.rs` | 追加顶点 | 预留 |
| AssignExecutor | `data_processing/transformations/assign.rs` | 变量赋值 | 预留 |
| LoopExecutor | `data_processing/loops.rs` | 循环执行 | 预留 |
| AllPathsExecutor | `data_processing/graph_traversal/all_paths.rs` | 所有路径 | 预留 |
| SubgraphExecutor | `data_processing/graph_traversal/subgraph.rs` | 子图提取 | 预留 |
| DistinctExecutor | `result_processing/dedup.rs` | DISTINCT 去重 | ✓ 已实施 |
| TopNExecutor | `result_processing/topn.rs` | TOP N 优化 | ✓ 已实施 |
| SampleExecutor | `result_processing/sampling.rs` | 随机采样 | ✓ 已实施 |
| GetPropExecutor | `data_access.rs` | 获取属性 | 预留 |
| IndexScanExecutor | `data_access.rs` | 索引扫描 | 预留 |
| ScanVerticesExecutor | `data_access.rs` | 全表扫描节点 | 预留 |
| ScanEdgesExecutor | `data_access.rs` | 全表扫描边 | 预留 |

### ⭐ 第三优先级（可选/简化）

高级和可选的执行器：

| 执行器类名 | 目标位置 | 描述 |
|---|---|---|
| ValueExecutor | `data_processing/value.rs` | 常量值 |
| PatternApplyExecutor | `data_processing/transformations/pattern_apply.rs` | 模式匹配 |
| RollUpApplyExecutor | `data_processing/transformations/rollup.rs` | ROLLUP 操作 |
| CartesianProductExecutor | `data_processing/join/cross_join.rs` | 笛卡尔积 |
| BFSShortestPathExecutor | `data_processing/graph_traversal/shortest_path.rs` | BFS最短路径 |
| MultiShortestPathExecutor | `data_processing/graph_traversal/shortest_path.rs` | 多源最短路径 |
| BatchShortestPath | `data_processing/graph_traversal/shortest_path.rs` | 批量最短路径 |
| SetExecutor | `result_processing/set.rs` | SET 语句 |
| DataCollectExecutor | `result_processing/collect.rs` | 收集结果 |
| FulltextIndexScanExecutor | `data_access.rs` | 全文索引扫描 |

---

## 按类别分类

### 数据访问执行器

**文件**: `src/query/executor/data_access.rs`

| 执行器 | 优先级 | 状态 |
|---|---|---|
| GetVerticesExecutor | ⭐⭐⭐ | ✓ |
| GetEdgesExecutor | ⭐⭐⭐ | ✓ |
| GetNeighborsExecutor | ⭐⭐⭐ | ✓ |
| GetPropExecutor | ⭐⭐ | 待实施 |
| IndexScanExecutor | ⭐⭐ | 待实施 |
| ScanVerticesExecutor | ⭐⭐ | 待实施 |
| ScanEdgesExecutor | ⭐⭐ | 待实施 |
| FulltextIndexScanExecutor | ⭐ | 待实施 |

### 数据处理执行器

**目录**: `src/query/executor/data_processing/`

#### 图遍历子模块
**文件**: `data_processing/graph_traversal/`

| 执行器 | 文件 | 优先级 | 状态 |
|---|---|---|---|
| ExpandExecutor | `expand.rs` | ⭐⭐⭐ | ✓ |
| ExpandAllExecutor | `expand_all.rs` | ⭐⭐⭐ | ✓ |
| TraverseExecutor | `traverse.rs` | ⭐⭐⭐ | ✓ |
| ShortestPathExecutor | `shortest_path.rs` | ⭐⭐⭐ | ✓ |
| BFSShortestPathExecutor | `shortest_path.rs` | ⭐ | 待实施 |
| MultiShortestPathExecutor | `shortest_path.rs` | ⭐ | 待实施 |
| BatchShortestPath | `shortest_path.rs` | ⭐ | 待实施 |
| AllPathsExecutor | `all_paths.rs` | ⭐⭐ | 待实施 |
| SubgraphExecutor | `subgraph.rs` | ⭐⭐ | 待实施 |

#### JOIN 子模块
**文件**: `data_processing/join/`

| 执行器 | 文件 | 优先级 | 状态 |
|---|---|---|---|
| InnerJoinExecutor | `inner_join.rs` | ⭐⭐ | 待实施 |
| LeftJoinExecutor | `left_join.rs` | ⭐⭐ | 待实施 |
| CartesianProductExecutor | `cross_join.rs` | ⭐ | 待实施 |

#### 集合运算子模块
**文件**: `data_processing/set_operations/`

| 执行器 | 文件 | 优先级 | 状态 |
|---|---|---|---|
| UnionExecutor | `union.rs` | ⭐⭐ | 待实施 |
| UnionAllVersionVarExecutor | `union_all.rs` | ⭐⭐ | 待实施 |
| IntersectExecutor | `intersect.rs` | ⭐⭐ | 待实施 |
| MinusExecutor | `minus.rs` | ⭐⭐ | 待实施 |

#### 数据转换子模块
**文件**: `data_processing/transformations/`

| 执行器 | 文件 | 优先级 | 状态 |
|---|---|---|---|
| AssignExecutor | `assign.rs` | ⭐⭐ | 待实施 |
| AppendVerticesExecutor | `append_vertices.rs` | ⭐⭐ | 待实施 |
| UnwindExecutor | `unwind.rs` | ⭐⭐ | 待实施 |
| PatternApplyExecutor | `pattern_apply.rs` | ⭐ | 待实施 |
| RollUpApplyExecutor | `rollup.rs` | ⭐ | 待实施 |

#### 其他
**文件**: `data_processing/`

| 执行器 | 文件 | 优先级 | 状态 |
|---|---|---|---|
| FilterExecutor | `filter.rs` | ⭐⭐⭐ | ✓ |
| LoopExecutor | `loops.rs` | ⭐⭐ | 待实施 |
| ValueExecutor | `value.rs` | ⭐ | 待实施 |

### 结果处理执行器

**目录**: `src/query/executor/result_processing/`

| 执行器 | 文件 | 优先级 | 状态 |
|---|---|---|---|
| ProjectExecutor | `projection.rs` | ⭐⭐⭐ | ✓ |
| AggregateExecutor | `aggregation.rs` | ⭐⭐⭐ | ✓ |
| SortExecutor | `sorting.rs` | ⭐⭐⭐ | ✓ |
| LimitExecutor | `limiting.rs` | ⭐⭐⭐ | ✓ |
| OffsetExecutor | `limiting.rs` | ⭐⭐⭐ | ✓ |
| DistinctExecutor | `dedup.rs` | ⭐⭐ | ✓ |
| TopNExecutor | `topn.rs` | ⭐⭐ | ✓ |
| SampleExecutor | `sampling.rs` | ⭐ | ✓ |
| DataCollectExecutor | `collect.rs` | ⭐⭐ | 待实施 |
| SetExecutor | `set.rs` | ⭐ | 待实施 |

### 数据修改执行器

**文件**: `src/query/executor/data_modification.rs`

| 执行器 | 优先级 | 状态 |
|---|---|---|
| InsertExecutor | ⭐⭐⭐ | ✓ |
| UpdateExecutor | ⭐⭐⭐ | ✓ |
| DeleteExecutor | ⭐⭐⭐ | ✓ |

### 基础执行器

**文件**: `src/query/executor/base.rs`

| 执行器 | 优先级 | 状态 |
|---|---|---|
| Executor | ⭐⭐⭐ | ✓ |
| StartExecutor | ⭐⭐⭐ | ✓ |
| ArgumentExecutor | ⭐ | 待实施 |
| PassThroughExecutor | ⭐ | 待实施 |

---

## 统计信息

### 实施状态

| 类别 | 总数 | ✓ 已实施 | 待实施 |
|---|---|---|---|
| 数据访问 | 8 | 3 | 5 |
| 数据处理 | 27 | 5 | 22 |
| 结果处理 | 9 | 7 | 2 |
| 数据修改 | 3 | 3 | 0 |
| 基础/逻辑 | 4 | 2 | 2 |
| **总计** | **52** | **20** | **32** |

### 优先级分布

| 优先级 | 总数 | 已实施 | 完成度 |
|---|---|---|---|
| ⭐⭐⭐ | 18 | 17 | 94% |
| ⭐⭐ | 19 | 3 | 16% |
| ⭐ | 15 | 0 | 0% |

---

## 迁移指南

### 已实施的执行器使用方式

已实施的执行器可直接从 `src/query/executor` 模块导出使用：

```rust
use graphdb::query::executor::{
    // 基础
    Executor, StartExecutor,
    // 数据访问
    GetVerticesExecutor, GetEdgesExecutor, GetNeighborsExecutor,
    // 数据处理
    FilterExecutor, ExpandExecutor, TraverseExecutor, ShortestPathExecutor,
    // 结果处理
    ProjectExecutor, AggregateExecutor, SortExecutor, 
    LimitExecutor, OffsetExecutor, DistinctExecutor,
    // 数据修改
    InsertExecutor, UpdateExecutor, DeleteExecutor,
};
```

### 待实施的执行器路线

待实施的执行器应按照以下步骤添加：

1. **在对应子模块中创建文件**
   ```
   data_processing/join/inner_join.rs
   data_processing/set_operations/union.rs
   等等
   ```

2. **实现执行器结构和逻辑**
   - 继承或实现 `Executor` trait
   - 编写核心算法逻辑
   - 添加单元测试

3. **在子模块 `mod.rs` 中导出**
   ```rust
   mod inner_join;
   pub use inner_join::InnerJoinExecutor;
   ```

4. **在上级 `mod.rs` 中重导出**
   ```rust
   pub use join::{InnerJoinExecutor, LeftJoinExecutor, ...};
   ```

5. **验证和测试**
   ```bash
   cargo build
   cargo test
   ```

---

## NebulaGraph 到新架构的完整映射表

### 第一部分：NebulaGraph 映射（按优先级）

#### ⭐⭐⭐ 第一优先级（已完成）

| NebulaGraph 文件 | 类名 | 新架构位置 |
|---|---|---|
| `executor/Executor.h/cpp` | `Executor` | `src/query/executor/base.rs` |
| `executor/StorageAccessExecutor.h/cpp` | `StorageAccessExecutor` | `src/query/executor/data_access.rs` |
| `executor/query/GetNeighborsExecutor.h/cpp` | `GetNeighborsExecutor` | `data_access.rs` |
| `executor/query/GetVerticesExecutor.h/cpp` | `GetVerticesExecutor` | `data_access.rs` |
| `executor/query/GetEdgesExecutor.h/cpp` | `GetEdgesExecutor` | `data_access.rs` |
| `executor/query/FilterExecutor.h/cpp` | `FilterExecutor` | `data_processing/filter.rs` |
| `executor/query/ExpandExecutor.h/cpp` | `ExpandExecutor` | `data_processing/graph_traversal/expand.rs` |
| `executor/query/ExpandAllExecutor.h/cpp` | `ExpandAllExecutor` | `data_processing/graph_traversal/expand_all.rs` |
| `executor/query/TraverseExecutor.h/cpp` | `TraverseExecutor` | `data_processing/graph_traversal/traverse.rs` |
| `executor/query/ProjectExecutor.h/cpp` | `ProjectExecutor` | `result_processing/projection.rs` |
| `executor/query/AggregateExecutor.h/cpp` | `AggregateExecutor` | `result_processing/aggregation.rs` |
| `executor/query/SortExecutor.h/cpp` | `SortExecutor` | `result_processing/sorting.rs` |
| `executor/query/LimitExecutor.h/cpp` | `LimitExecutor` | `result_processing/limiting.rs` |
| `executor/mutate/InsertExecutor.h/cpp` | `InsertExecutor` | `data_modification.rs` |
| `executor/mutate/UpdateExecutor.h/cpp` | `UpdateExecutor` | `data_modification.rs` |
| `executor/mutate/DeleteExecutor.h/cpp` | `DeleteExecutor` | `data_modification.rs` |
| `executor/logic/StartExecutor.h/cpp` | `StartExecutor` | `base.rs` |
| `executor/algo/ShortestPathExecutor.h/cpp` | `ShortestPathExecutor` | `data_processing/graph_traversal/shortest_path.rs` |

#### ⭐⭐ 第二优先级（部分实施）

| NebulaGraph 文件 | 类名 | 新架构位置 | 状态 |
|---|---|---|---|
| `executor/query/JoinExecutor.h/cpp` | `JoinExecutor` | `data_processing/join/inner_join.rs` | 待实施 |
| `executor/query/LeftJoinExecutor.h/cpp` | `LeftJoinExecutor` | `data_processing/join/left_join.rs` | 待实施 |
| `executor/query/UnionExecutor.h/cpp` | `UnionExecutor` | `data_processing/set_operations/union.rs` | 待实施 |
| `executor/query/UnwindExecutor.h/cpp` | `UnwindExecutor` | `data_processing/transformations/unwind.rs` | 待实施 |
| `executor/query/DedupExecutor.h/cpp` | `DedupExecutor` | `result_processing/dedup.rs` | ✓ |
| `executor/query/TopNExecutor.h/cpp` | `TopNExecutor` | `result_processing/topn.rs` | ✓ |
| `executor/query/AssignExecutor.h/cpp` | `AssignExecutor` | `data_processing/transformations/assign.rs` | 待实施 |
| `executor/query/IntersectExecutor.h/cpp` | `IntersectExecutor` | `data_processing/set_operations/intersect.rs` | 待实施 |
| `executor/query/MinusExecutor.h/cpp` | `MinusExecutor` | `data_processing/set_operations/minus.rs` | 待实施 |
| `executor/query/AppendVerticesExecutor.h/cpp` | `AppendVerticesExecutor` | `data_processing/transformations/append_vertices.rs` | 待实施 |
| `executor/query/GetPropExecutor.h/cpp` | `GetPropExecutor` | `data_access.rs` | 待实施 |
| `executor/query/IndexScanExecutor.h/cpp` | `IndexScanExecutor` | `data_access.rs` | 待实施 |
| `executor/query/ScanVerticesExecutor.h/cpp` | `ScanVerticesExecutor` | `data_access.rs` | 待实施 |
| `executor/query/ScanEdgesExecutor.h/cpp` | `ScanEdgesExecutor` | `data_access.rs` | 待实施 |
| `executor/logic/LoopExecutor.h/cpp` | `LoopExecutor` | `data_processing/loops.rs` | 待实施 |
| `executor/algo/AllPathsExecutor.h/cpp` | `AllPathsExecutor` | `data_processing/graph_traversal/all_paths.rs` | 待实施 |
| `executor/algo/SubgraphExecutor.h/cpp` | `SubgraphExecutor` | `data_processing/graph_traversal/subgraph.rs` | 待实施 |
| `executor/query/SampleExecutor.h/cpp` | `SampleExecutor` | `result_processing/sampling.rs` | ✓ |

#### ⭐ 第三优先级（可选）

| NebulaGraph 文件 | 类名 | 新架构位置 | 状态 |
|---|---|---|---|
| `executor/query/ValueExecutor.h/cpp` | `ValueExecutor` | `data_processing/value.rs` | 待实施 |
| `executor/query/SetExecutor.h/cpp` | `SetExecutor` | `result_processing/set.rs` | 待实施 |
| `executor/query/PatternApplyExecutor.h/cpp` | `PatternApplyExecutor` | `data_processing/transformations/pattern_apply.rs` | 待实施 |
| `executor/query/RollUpApplyExecutor.cpp` | `RollUpApplyExecutor` | `data_processing/transformations/rollup.rs` | 待实施 |
| `executor/query/FulltextIndexScanExecutor.h/cpp` | `FulltextIndexScanExecutor` | `data_access.rs` | 待实施 |
| `executor/logic/ArgumentExecutor.h/cpp` | `ArgumentExecutor` | `base.rs` | 待实施 |
| `executor/logic/PassThroughExecutor.h/cpp` | `PassThroughExecutor` | `base.rs` | 待实施 |
| `executor/algo/BFSShortestPathExecutor.h/cpp` | `BFSShortestPathExecutor` | `data_processing/graph_traversal/shortest_path.rs` | 待实施 |
| `executor/algo/MultiShortestPathExecutor.h/cpp` | `MultiShortestPathExecutor` | `data_processing/graph_traversal/shortest_path.rs` | 待实施 |
| `executor/algo/BatchShortestPath.h/cpp` | `BatchShortestPath` | `data_processing/graph_traversal/shortest_path.rs` | 待实施 |
| `executor/algo/CartesianProductExecutor.h/cpp` | `CartesianProductExecutor` | `data_processing/join/cross_join.rs` | 待实施 |

---

## 后续实施任务清单

### 阶段 1：数据处理执行器补充（优先）

- [ ] 集合运算模块
  - [ ] `union.rs` - UnionExecutor
  - [ ] `union_all.rs` - UnionAllVersionVarExecutor
  - [ ] `intersect.rs` - IntersectExecutor
  - [ ] `minus.rs` - MinusExecutor

- [ ] JOIN 操作模块
  - [ ] `inner_join.rs` - InnerJoinExecutor, JoinExecutor
  - [ ] `left_join.rs` - LeftJoinExecutor
  - [ ] `cross_join.rs` - CartesianProductExecutor

- [ ] 数据转换模块
  - [ ] `assign.rs` - AssignExecutor
  - [ ] `append_vertices.rs` - AppendVerticesExecutor
  - [ ] `unwind.rs` - UnwindExecutor
  - [ ] `pattern_apply.rs` - PatternApplyExecutor
  - [ ] `rollup.rs` - RollUpApplyExecutor

### 阶段 2：结果处理补充

- [ ] `result_processing/collect.rs` - DataCollectExecutor
- [ ] `result_processing/set.rs` - SetExecutor

### 阶段 3：数据访问补充

- [ ] `GetPropExecutor`
- [ ] `IndexScanExecutor`
- [ ] `ScanVerticesExecutor`
- [ ] `ScanEdgesExecutor`
- [ ] `FulltextIndexScanExecutor`

### 阶段 4：高级功能

- [ ] 图遍历扩展
  - [ ] `all_paths.rs` - AllPathsExecutor
  - [ ] `subgraph.rs` - SubgraphExecutor

- [ ] 最短路径优化
  - [ ] BFS优化版本
  - [ ] 多源最短路径
  - [ ] 批量最短路径

---

## 参考文档

- `executor_refactoring_plan.md` - 拆分方案和实施步骤
- `executor_module_migration.md` - NebulaGraph 迁移详细指南
- `src/query/executor/mod.rs` - 当前模块导出配置
