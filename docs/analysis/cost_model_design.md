# GraphDB 代价模型设计文档

## 1. 概述

本文档定义 GraphDB 的代价模型设计，参考 PostgreSQL 的代价模型，并针对图数据库特性进行扩展。

## 2. 设计原则

1. **与 PostgreSQL 兼容**：基础参数（seq_page_cost, cpu_tuple_cost 等）与 PostgreSQL 保持一致
2. **图特性支持**：针对图遍历、多跳查询等特有操作设计专门代价公式
3. **硬件适配**：支持 SSD、HDD、内存等不同存储介质的配置
4. **可扩展性**：预留参数便于未来扩展（并行查询、分布式等）

## 3. 代价参数设计

### 3.1 基础 I/O 代价参数（与 PostgreSQL 一致）

| 参数名 | 默认值 | 说明 | PostgreSQL 对应 |
|--------|--------|------|-----------------|
| `seq_page_cost` | 1.0 | 顺序读取一页的代价 | seq_page_cost |
| `random_page_cost` | 4.0 | 随机读取一页的代价 | random_page_cost |
| `cpu_tuple_cost` | 0.01 | 处理一行数据的 CPU 代价 | cpu_tuple_cost |
| `cpu_index_tuple_cost` | 0.005 | 处理索引项的 CPU 代价 | cpu_index_tuple_cost |
| `cpu_operator_cost` | 0.0025 | 执行操作符的 CPU 代价 | cpu_operator_cost |

### 3.2 图数据库特有参数

| 参数名 | 默认值 | 说明 |
|--------|--------|------|
| `edge_traversal_cost` | 0.02 | 边遍历代价（比顶点处理更复杂） |
| `multi_hop_penalty` | 1.2 | 多跳遍历每步递增系数 |
| `neighbor_lookup_cost` | 0.015 | 邻居节点查找代价 |
| `effective_cache_pages` | 10000 | 有效缓存大小（页数） |
| `cache_hit_cost_factor` | 0.1 | 缓存命中时代价系数 |
| `shortest_path_base_cost` | 10.0 | 最短路径算法基础代价 |
| `path_enumeration_factor` | 2.0 | 路径枚举指数系数 |
| `super_node_threshold` | 10000 | 超级节点阈值（度数） |
| `super_node_penalty` | 2.0 | 超级节点额外代价系数 |

### 3.3 算法相关参数

| 参数名 | 默认值 | 说明 |
|--------|--------|------|
| `hash_build_overhead` | 0.1 | 哈希构建开销系数 |
| `sort_comparison_cost` | 1.0 | 排序比较代价系数 |
| `memory_sort_threshold` | 10000 | 内存排序阈值（行数） |
| `external_sort_page_cost` | 2.0 | 外部排序页代价 |

## 4. 代价计算公式

### 4.1 扫描操作

```
全表扫描顶点 = rows × cpu_tuple_cost
全表扫描边 = rows × cpu_tuple_cost

索引扫描 = 索引访问代价 + 回表代价
索引访问代价 = index_pages × seq_page_cost + index_rows × cpu_index_tuple_cost
回表代价 = matches × random_page_cost + matches × cpu_tuple_cost
```

### 4.2 图遍历操作

```
单步扩展 = start_nodes × avg_degree × (seq_page_cost + edge_traversal_cost)

多步遍历 = Σ(step=1 to n)[rows_step × edge_traversal_cost × multi_hop_penalty^(step-1)]
其中 rows_step = start_nodes × avg_degree^step

全扩展(ExpandAll) = 单步扩展 × 1.5

邻居查找 = start_nodes × neighbor_lookup_cost × avg_degree
```

### 4.3 连接操作

```
哈希连接 = 构建代价 + 探测代价 + 哈希开销
构建代价 = left_rows × cpu_tuple_cost
探测代价 = right_rows × cpu_tuple_cost
哈希开销 = left_rows × hash_build_overhead × cpu_operator_cost

嵌套循环连接 = 外表代价 + 内表循环代价
外表代价 = left_rows × cpu_tuple_cost
内表循环代价 = left_rows × right_rows × cpu_tuple_cost

交叉连接 = left_rows × right_rows × cpu_tuple_cost
```

### 4.4 排序和聚合

```
排序 = rows × log₂(rows) × sort_columns × cpu_operator_cost × sort_comparison_cost

TopN = rows × log₂(limit) × cpu_operator_cost

聚合 = rows × agg_functions × cpu_operator_cost

去重 = rows × cpu_operator_cost × 2.0
```

### 4.5 图算法

```
最短路径 = start_nodes × branching_factor^max_depth × cpu_tuple_cost + shortest_path_base_cost

所有路径 = 最短路径 × path_enumeration_factor

多源最短路径 = 最短路径 × 1.5
```

## 5. 硬件环境配置

### 5.1 HDD 配置（默认）
```rust
random_page_cost = 4.0
seq_page_cost = 1.0
```

### 5.2 SSD 配置
```rust
random_page_cost = 1.1  // SSD 随机访问接近顺序访问
seq_page_cost = 1.0
```

### 5.3 内存数据库配置
```rust
random_page_cost = 0.1
seq_page_cost = 0.1
// CPU 代价相对更重要
cpu_tuple_cost = 0.01
cache_hit_cost_factor = 0.01  // 缓存命中几乎无代价
```

## 6. 选择性估计

### 6.1 等值条件
```
有统计信息: 1 / distinct_values
无统计信息: 0.1 (默认值)
```

### 6.2 范围条件
```
有直方图: 使用直方图计算
无直方图: 0.333 (默认 1/3)
```

### 6.3 LIKE 条件
```
前缀匹配 (xxx%): 0.1
后缀匹配 (%xxx): 0.2
包含匹配 (%xxx%): 0.5
精确匹配: 0.05
```

## 7. 超级节点处理

当节点的度数超过 `super_node_threshold` 时，应用额外代价惩罚：

```
实际代价 = 基础代价 × super_node_penalty
```

这可以避免优化器低估涉及超级节点的查询代价。

## 8. 缓存感知

根据有效缓存大小调整 I/O 代价：

```
如果访问的数据页数 < effective_cache_pages:
    io_cost = pages × seq_page_cost × cache_hit_cost_factor
否则:
    io_cost = effective_cache_pages × seq_page_cost × cache_hit_cost_factor +
              (pages - effective_cache_pages) × seq_page_cost
```

## 9. 与 PostgreSQL 的差异

| 方面 | PostgreSQL | GraphDB |
|------|------------|---------|
| 并行代价 | 支持 | 暂不支持（单节点） |
| 分区代价 | 支持 | 暂不支持 |
| 图遍历 | 无 | 专门支持 |
| 多跳查询 | 无 | 专门支持 |
| 最短路径 | 无 | 专门支持 |
| 超级节点 | 无 | 专门处理 |

## 10. 未来扩展

1. **并行查询**：添加 parallel_tuple_cost、parallel_setup_cost
2. **自适应优化**：根据实际执行反馈调整代价参数
3. **机器学习**：使用 ML 模型预测复杂查询的代价
4. **分布式**：添加网络传输代价参数
