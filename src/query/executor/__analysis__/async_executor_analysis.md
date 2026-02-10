# 异步执行器分析报告 (更新版)

## 概述

本报告分析了图数据库中执行器的并行处理需求，基于以下原则：
- 轻量级、高性能的单节点图数据库
- 只对真正需要的执行器引入并行处理
- 避免过度设计和不必要的复杂性

## 执行器分类与开销分析

### 1. CPU 密集型执行器 - 推荐 Rayon

以下执行器主要涉及 CPU 密集型计算，适合使用 Rayon 进行并行化：

| 执行器 | 主要开销 | 推荐策略 |
|-------|---------|---------|
| `SortExecutor` | CPU (排序 O(n log n)) | `rayon::par_sort()` |
| `TopNExecutor` | CPU (堆操作) | Rayon 分区排序 |
| `DedupExecutor` | CPU + 内存 (HashSet) | Rayon 预分组 |
| `FilterExecutor` | CPU (表达式求值) | `par_iter().filter()` |
| `AggregationExecutor` | CPU (分组聚合) | Rayon Map-Reduce |

**并行化收益**：当数据集 > 10000 条时，收益明显。

### 2. I/O 密集型执行器 - 推荐存储层异步化

以下执行器主要涉及存储 I/O 操作：

| 执行器 | 主要开销 | 推荐策略 |
|-------|---------|---------|
| `GetVerticesExecutor` | I/O (存储读取) | 存储层异步接口 |
| `GetEdgesExecutor` | I/O (存储读取) | 存储层异步接口 |
| `ScanVerticesExecutor` | I/O (顺序扫描) | 存储层异步接口 |
| `ScanEdgesExecutor` | I/O (顺序扫描) | 存储层异步接口 |
| `GetNeighborsExecutor` | I/O (多键查找) | 存储层异步接口 |
| `BatchInsertExecutor` | I/O (批量写入) | 异步批量 I/O |
| `BatchUpdateExecutor` | I/O + CPU | 异步 I/O + CPU 并行 |
| `BatchDeleteExecutor` | I/O (多键删除) | 异步 I/O |

**并行化收益**：当有多个存储分片时，并行预取可获得接近线性的加速。

### 3. 图算法类执行器 - 推荐 Rayon

| 执行器 | 主要开销 | 推荐策略 |
|-------|---------|---------|
| `BFSShortestExecutor` | CPU (队列遍历) | Rayon (宽图) |
| `AllPathsExecutor` | CPU (指数级) | Rayon + 剪枝 |
| `ShortestPathExecutor` | CPU (堆操作) | Rayon |
| `PageRankExecutor` | CPU (迭代) | Rayon (迭代并行) |
| `ConnectedComponentsExecutor` | CPU | Rayon |

**并行化收益**：
- PageRank：迭代内完全并行，顶点数 > 10万时收益明显
- BFS：宽图 (分支因子 > 50) 时收益高

### 4. 适合同步实现的执行器

以下执行器没有明显的并行收益，应保持同步实现：

#### 简单查询处理类
- `ProjectExecutor` - 投影操作 (纯数据复制)
- `LimitExecutor` - 限制结果数量
- `GroupByExecutor` - 分组操作 (依赖累积)

#### 管理类执行器
- `CreateTagExecutor` - 创建标签
- `DropTagExecutor` - 删除标签
- `CreateEdgeExecutor` - 创建边类型
- `DropEdgeExecutor` - 删除边类型
- `CreateSpaceExecutor` - 创建空间
- `DropSpaceExecutor` - 删除空间

#### 其他
- `GetPropExecutor` - 获取属性值 (简单查找)
- `JoinExecutor` - 需根据具体场景决定

## 并行处理策略

### 策略选择原则

```
数据规模判断：
├── < 1000 条: 顺序执行，无并行
├── 1000 - 10000 条: 顺序执行，可考虑 Rayon
└── > 10000 条: 优先使用 Rayon
```

### 推荐的实现方式

```
┌─────────────────────────────────────────────────────────────┐
│                   存储层 (Storage)                            │
│         所有 I/O 操作 → Async + Tokio (未来规划)              │
└─────────────────────────────────────────────────────────────┘
                              ↑
                              │ async_read()/async_write()
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                   执行器层 (Executor)                        │
├─────────────────┬─────────────────┬─────────────────────────┤
│   CPU 密集型     │   I/O 密集型     │     混合型              │
│   (Sort, Filter,│   (Scan, Batch) │   (Update)             │
│    PageRank)     │                 │                        │
├─────────────────┼─────────────────┼─────────────────────────┤
│   Rayon         │   顺序执行       │   分离 I/O 和 CPU       │
│   (par_iter)    │   (或 spawn_    │   (如需要则 Rayon)      │
│                 │    blocking)    │                        │
└─────────────────┴─────────────────┴─────────────────────────┘
```

### 线程池使用规则

**允许使用线程池的情况**：
1. CPU 密集型计算：使用 Rayon
2. 阻塞 I/O 操作：使用 `tokio::task::spawn_blocking`

**禁止使用线程池的情况**：
1. 简单数据处理 (投影、限制、去重小数据集)
2. 管理类操作 (创建/删除 schema)
3. 任何不需要并行的操作

## 总结

### 执行器实现策略

| 执行器类别 | 推荐实现 | 是否需要 Rayon |
|-----------|---------|---------------|
| **CPU 密集型** | Rayon | 是 |
| **I/O 密集型** | 顺序或 spawn_blocking | 否 |
| **图算法类** | Rayon | 是 |
| **简单处理类** | 顺序执行 | 否 |
| **管理类** | 顺序执行 | 否 |

### 后续行动

1. 对 SortExecutor、FilterExecutor、DedupExecutor 等引入 Rayon
2. 对 Batch*Executor 考虑使用 `spawn_blocking` 或未来异步存储
3. 移除其他执行器中的所有线程池操作
4. 保持代码简单，避免过度设计
