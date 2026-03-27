# 查询优化器代价评估指标分析报告

## 概述

本文档基于 `src/core/value`、`src/core/types`、`src/query/planning/plan/core/nodes/base/memory_estimation.rs` 目录实现的内存估算功能，分析 `src/query/optimizer` 模块可以纳入代价评判的指标。

## 一、现有内存估算能力分析

### 1.1 Value 类型内存估算

位于 `src/core/value/memory_estimation.rs`，实现了 `MemoryEstimatable` trait：

| 类型类别 | 具体类型 | 估算方式 |
|---------|---------|---------|
| 固定大小类型 | `Empty`, `Null`, `Bool`, 整数类型, `Float` | 仅基础大小 `size_of::<Value>()` |
| 变长字符串 | `String`, `FixedString` | 基础大小 + `capacity()` |
| 二进制数据 | `Blob` | 基础大小 + `capacity()` |
| 复杂类型 | `Decimal128`, `Geography`, `DateTime` | 基础大小 + 内部数据大小 |
| 图类型 | `Vertex`, `Edge`, `Path` | 基础大小 + 引用大小 |
| 集合类型 | `List`, `Map`, `Set` | 递归估算，累加元素内存 |
| 数据集 | `DataSet` | 基础大小 + 内部数据大小 |

### 1.2 Expression 类型内存估算

位于 `src/core/types/expr/memory_estimation.rs`：

| 表达式类型 | 估算内容 |
|-----------|---------|
| 叶子节点 | `Literal`, `Variable`, `Label`, `Parameter` - 基础大小 + 数据 |
| 一元操作 | `Unary`, `TypeCast`, `Aggregate` - 基础大小 + 操作数 |
| 二元操作 | `Binary`, `Subscript` - 基础大小 + 两个操作数 |
| 属性访问 | `Property`, `TagProperty`, `EdgeProperty` - 基础大小 + 字符串 |
| 集合类型 | `List`, `Map`, `Path` - 递归累加元素 |
| 函数调用 | `Function`, `Predicate` - 基础大小 + 名称 + 参数 |
| 条件表达式 | `Case` - 基础大小 + 条件 + 结果 + 默认值 |
| 列表推导 | `ListComprehension`, `Reduce` - 基础大小 + 变量 + 表达式 |

### 1.3 Plan Node 内存估算基础

位于 `src/query/planning/plan/core/nodes/base/memory_estimation.rs`：

提供了基础 trait 和宏：
- `MemoryEstimatable` trait - 统一内存估算接口
- `estimate_string_memory` - 字符串内存估算
- `estimate_vec_string_memory` - 字符串向量估算
- `impl_default_estimate_memory!` 宏 - 为 plan node 提供默认实现

## 二、现有代价评估体系分析

### 2.1 代价模型配置

位于 `src/query/optimizer/cost/config.rs`，包含以下参数：

#### I/O 成本参数
- `seq_page_cost`: 顺序页读取成本 (默认 1.0)
- `random_page_cost`: 随机页读取成本 (默认 4.0)
- `effective_cache_pages`: 有效缓存页数
- `cache_hit_cost_factor`: 缓存命中成本系数

#### CPU 成本参数
- `cpu_tuple_cost`: 每行处理成本 (默认 0.01)
- `cpu_index_tuple_cost`: 索引行处理成本 (默认 0.005)
- `cpu_operator_cost`: 操作符计算成本 (默认 0.0025)

#### 算法成本参数
- `hash_build_overhead`: 哈希表构建开销 (默认 0.1)
- `sort_comparison_cost`: 排序比较成本 (默认 1.0)
- `memory_sort_threshold`: 内存排序阈值 (默认 10000 行)
- `external_sort_page_cost`: 外部排序页成本 (默认 2.0)

#### 图数据库特有参数
- `edge_traversal_cost`: 边遍历成本 (默认 0.02)
- `multi_hop_penalty`: 多跳惩罚系数 (默认 1.2)
- `neighbor_lookup_cost`: 邻居查找成本 (默认 0.015)
- `shortest_path_base_cost`: 最短路径基础成本 (默认 10.0)
- `path_enumeration_factor`: 路径枚举系数 (默认 2.0)
- `super_node_threshold`: 超级节点阈值 (默认 10000)
- `super_node_penalty`: 超级节点惩罚 (默认 2.0)

### 2.2 节点代价估算器

#### 扫描操作估算器 (`scan.rs`)
- `ScanVertices` - 基于顶点数量 × CPU 成本
- `ScanEdges` - 基于边数量 × CPU 成本
- `IndexScan` - 基于选择性 × 表行数
- `EdgeIndexScan` - 基于选择性 × 边数量

#### 图遍历估算器 (`graph_traversal.rs`)
- `Expand` - 考虑倾斜感知的扩展成本
- `ExpandAll` - 包含顶点信息的扩展
- `Traverse` - 多步遍历成本（含多跳惩罚）
- `AppendVertices` - 附加顶点成本
- `GetNeighbors` - 邻居查找成本
- `GetVertices/GetEdges` - 基于 limit 的成本

#### 连接操作估算器 (`join.rs`)
- `HashInnerJoin` - 哈希表构建 + 探测成本
- `HashLeftJoin` - 保留左表所有行
- `InnerJoin/LeftJoin` - 嵌套循环成本
- `CrossJoin` - 笛卡尔积成本
- `FullOuterJoin` - 全外连接成本

#### 数据处理估算器 (`data_processing.rs`)
- `Filter` - 基于条件数量和选择性
- `Project` - 基于列数 × 行数
- `Unwind` - 基于列表大小 × 行数
- `DataCollect` - 数据收集成本

#### 排序限制估算器 (`sort_limit.rs`)
- `Sort` - O(n log n) 或外部排序
- `Limit` - 基于 offset + limit
- `TopN` - 堆排序 O(n log k)
- `Aggregate` - 基于聚合函数数和分组键
- `Dedup` - 哈希去重成本
- `Sample` - 采样成本

## 三、可纳入代价评判的新指标

基于内存估算能力和现有代价体系，以下指标可以纳入代价评判：

### 3.1 内存使用成本指标

#### 3.1.1 节点内存占用成本
```rust
// 新增配置参数
pub struct CostModelConfig {
    // ... 现有参数
    
    /// 每字节内存使用成本（用于估算内存压力）
    pub memory_byte_cost: f64, // 默认 0.0001
    
    /// 内存压力阈值（超过此值增加成本惩罚）
    pub memory_pressure_threshold: usize, // 默认 100MB (100 * 1024 * 1024)
    
    /// 内存压力惩罚系数
    pub memory_pressure_penalty: f64, // 默认 2.0
}
```

**应用场景**：
- 聚合操作需要存储中间结果（HashMap）
- 排序操作需要存储待排序数据
- 连接操作需要存储哈希表
- 图遍历需要存储访问过的节点

#### 3.1.2 中间结果大小估算
```rust
/// 中间结果内存估算
pub fn estimate_intermediate_memory(&self, node: &PlanNodeEnum, input_rows: u64) -> usize {
    match node {
        PlanNodeEnum::Aggregate(n) => {
            // 估算聚合哈希表大小
            let group_count = self.estimate_group_count(n, input_rows);
            let row_size = self.estimate_row_size(n.output_schema());
            group_count * row_size
        }
        PlanNodeEnum::Sort(n) => {
            // 估算排序缓冲区大小
            input_rows as usize * self.estimate_row_size(n.output_schema())
        }
        PlanNodeEnum::HashInnerJoin(_) => {
            // 估算哈希表大小（基于左表）
            input_rows as usize * self.estimate_avg_row_size()
        }
        _ => 0
    }
}
```

### 3.2 表达式复杂度成本

#### 3.2.1 表达式计算成本
```rust
// 新增配置参数
pub struct CostModelConfig {
    /// 简单表达式（叶子节点）成本
    pub simple_expression_cost: f64, // 默认 0.001
    
    /// 函数调用基础成本
    pub function_call_base_cost: f64, // 默认 0.01
    
    /// 复杂表达式每层嵌套成本
    pub expression_nesting_cost: f64, // 默认 0.005
}
```

**基于 Expression 内存估算的复杂度指标**：

| 指标 | 计算方式 | 用途 |
|-----|---------|-----|
| `node_count` | 表达式树节点总数 | 评估表达式复杂度 |
| `max_depth` | 表达式树最大深度 | 评估栈溢出风险 |
| `memory_size` | `estimate_memory()` 结果 | 评估内存占用 |
| `is_simple` | 是否为叶子节点 | 快速路径判断 |

#### 3.2.2 表达式成本计算实现
```rust
impl CostCalculator {
    /// 计算表达式评估成本
    pub fn calculate_expression_cost(&self, expr: &Expression) -> f64 {
        let node_count = expr.node_count() as f64;
        let memory_factor = expr.estimate_memory() as f64 / 1000.0; // 每KB成本
        
        if expr.is_simple() {
            self.config.simple_expression_cost
        } else {
            node_count * self.config.cpu_operator_cost 
                + memory_factor * self.config.memory_byte_cost
        }
    }
    
    /// 计算过滤条件成本（增强版）
    pub fn calculate_filter_cost_enhanced(
        &self, 
        input_rows: u64, 
        conditions: &[Expression]
    ) -> f64 {
        let base_cost = input_rows as f64 * self.config.cpu_operator_cost;
        let expression_cost: f64 = conditions
            .iter()
            .map(|e| self.calculate_expression_cost(e))
            .sum();
        
        base_cost + expression_cost * input_rows as f64
    }
}
```

### 3.3 数据类型相关成本

#### 3.3.1 类型处理成本
```rust
// 新增配置参数
pub struct CostModelConfig {
    /// 固定大小类型处理成本系数
    pub fixed_type_cost_factor: f64, // 默认 1.0
    
    /// 变长类型处理成本系数
    pub variable_type_cost_factor: f64, // 默认 1.5
    
    /// 复杂类型处理成本系数
    pub complex_type_cost_factor: f64, // 默认 2.0
    
    /// 图类型处理成本系数
    pub graph_type_cost_factor: f64, // 默认 3.0
}
```

**基于 Value 类型的成本分类**：

| 类型类别 | 代表类型 | 成本系数 | 原因 |
|---------|---------|---------|-----|
| 固定大小 | `Int`, `Float`, `Bool` | 1.0 | CPU缓存友好，无需堆分配 |
| 变长类型 | `String`, `Blob` | 1.5 | 需要堆分配和内存管理 |
| 复杂类型 | `List`, `Map`, `Set` | 2.0 | 需要递归处理，内存不连续 |
| 图类型 | `Vertex`, `Edge`, `Path` | 3.0 | 包含多个属性，结构复杂 |
| 数据集 | `DataSet` | 2.5 | 包含多行数据，需要迭代处理 |

#### 3.3.2 行大小估算
```rust
/// 估算行的平均大小
pub fn estimate_row_size(&self, schema: &Schema) -> usize {
    schema.columns()
        .iter()
        .map(|col| self.estimate_type_size(&col.data_type))
        .sum()
}

/// 估算数据类型大小
pub fn estimate_type_size(&self, data_type: &DataType) -> usize {
    match data_type {
        DataType::Int | DataType::Float | DataType::Bool => 8,
        DataType::String => 32, // 假设平均长度
        DataType::List(inner) => 16 + self.estimate_type_size(inner) * 3, // 假设3个元素
        DataType::Map => 64, // 假设平均大小
        DataType::Vertex | DataType::Edge => 128, // 图类型较大
        _ => 16, // 默认值
    }
}
```

### 3.4 缓存友好性成本

#### 3.4.1 数据局部性成本
```rust
// 新增配置参数
pub struct CostModelConfig {
    /// 缓存未命中成本
    pub cache_miss_cost: f64, // 默认 0.1
    
    /// 顺序访问成本系数
    pub sequential_access_factor: f64, // 默认 0.8
    
    /// 随机访问成本系数
    pub random_access_factor: f64, // 默认 1.5
}
```

**应用场景**：
- 全表扫描 vs 索引扫描的缓存行为差异
- 图遍历中顺序访问邻居 vs 随机访问
- 排序后数据 vs 未排序数据的访问效率

### 3.5 并发与并行成本

#### 3.5.1 并行执行成本
```rust
// 新增配置参数
pub struct CostModelConfig {
    /// 并行化启动成本
    pub parallel_startup_cost: f64, // 默认 100.0
    
    /// 每线程处理成本系数
    pub per_thread_cost_factor: f64, // 默认 0.9
    
    /// 线程间通信成本
    pub thread_communication_cost: f64, // 默认 0.05
    
    /// 最大并行度
    pub max_parallel_degree: usize, // 默认 8
}
```

**应用场景**：
- 大数据量排序的并行化决策
- 聚合操作的并行化收益评估
- 连接操作的并行哈希构建

## 四、成本估算增强建议

### 4.1 增强 NodeCostEstimate

```rust
/// 增强的节点成本估算结果
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeCostEstimate {
    /// 节点自身成本（不含子节点）
    pub node_cost: f64,
    /// 总成本（含所有子节点）
    pub total_cost: f64,
    /// 估算输出行数
    pub output_rows: u64,
    
    // ===== 新增字段 =====
    /// 估算内存使用（字节）
    pub memory_usage: usize,
    /// 表达式计算成本
    pub expression_cost: f64,
    /// I/O 成本细分
    pub io_cost: IoCostBreakdown,
    /// CPU 成本细分
    pub cpu_cost: CpuCostBreakdown,
}

/// I/O 成本细分
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IoCostBreakdown {
    pub sequential_io: f64,
    pub random_io: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// CPU 成本细分
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CpuCostBreakdown {
    pub tuple_processing: f64,
    pub expression_evaluation: f64,
    pub hash_computation: f64,
    pub comparison_operations: f64,
}
```

### 4.2 内存感知成本计算

```rust
impl CostCalculator {
    /// 计算内存感知成本
    pub fn calculate_memory_aware_cost(
        &self,
        base_cost: f64,
        memory_usage: usize,
    ) -> f64 {
        let memory_cost = memory_usage as f64 * self.config.memory_byte_cost;
        
        // 检查内存压力
        if memory_usage > self.config.memory_pressure_threshold {
            let pressure_factor = memory_usage as f64 / self.config.memory_pressure_threshold as f64;
            let penalty = pressure_factor * self.config.memory_pressure_penalty;
            base_cost * penalty + memory_cost
        } else {
            base_cost + memory_cost
        }
    }
    
    /// 增强的聚合成本计算
    pub fn calculate_aggregate_cost_enhanced(
        &self,
        input_rows: u64,
        agg_functions: usize,
        group_by_keys: usize,
        group_by_exprs: &[Expression],
    ) -> f64 {
        // 基础成本
        let base_cost = self.calculate_aggregate_cost(input_rows, agg_functions, group_by_keys);
        
        // 估算内存使用
        let estimated_groups = (input_rows / 2_u64.pow(group_by_keys as u32)).max(10);
        let row_size = group_by_exprs
            .iter()
            .map(|e| e.estimate_memory())
            .sum::<usize>();
        let memory_usage = estimated_groups as usize * row_size;
        
        // 计算内存感知成本
        self.calculate_memory_aware_cost(base_cost, memory_usage)
    }
}
```

### 4.3 统计信息增强

```rust
/// 增强的统计信息
pub struct EnhancedStatistics {
    // 现有统计信息...
    
    /// 平均行大小
    pub avg_row_size: usize,
    /// 数据倾斜度
    pub skewness: f64,
    /// 缓存命中率估计
    pub estimated_cache_hit_rate: f64,
    /// 列的直方图统计
    pub column_histograms: HashMap<String, Histogram>,
}

impl StatisticsManager {
    /// 获取平均行大小
    pub fn get_avg_row_size(&self, tag_name: &str) -> usize {
        self.tag_stats
            .get(tag_name)
            .map(|s| s.avg_row_size)
            .unwrap_or(64) // 默认值
    }
    
    /// 获取列的选择性（基于直方图）
    pub fn get_column_selectivity(
        &self, 
        tag_name: &str, 
        column: &str,
        range: &RangeCondition,
    ) -> f64 {
        if let Some(histogram) = self.get_histogram(tag_name, column) {
            histogram.estimate_selectivity(range)
        } else {
            0.1 // 默认选择性
        }
    }
}
```

## 五、实施优先级建议

### 5.1 高优先级（立即实施）

1. **表达式计算成本**
   - 利用现有的 `Expression::node_count()` 方法
   - 添加简单的表达式复杂度惩罚
   - 影响：Filter、Project、Aggregate 等节点

2. **基础内存估算**
   - 为 Aggregate、Sort、Join 节点添加内存使用估算
   - 利用现有的 `MemoryEstimatable` trait
   - 影响：内存压力大的查询场景

### 5.2 中优先级（短期实施）

1. **数据类型成本系数**
   - 根据 Value 类型分类添加成本系数
   - 影响行大小估算的准确性
   - 影响：所有涉及类型处理的节点

2. **增强统计信息**
   - 添加平均行大小统计
   - 添加列级直方图
   - 影响：选择性估算的准确性

### 5.3 低优先级（长期规划）

1. **缓存感知成本**
   - 需要硬件性能监控支持
   - 影响：I/O 密集型查询优化

2. **并行成本模型**
   - 需要并行执行框架支持
   - 影响：大数据量查询性能

## 六、总结

通过分析现有代码，可以得出以下可纳入代价评判的指标：

| 指标类别 | 具体指标 | 数据来源 | 实施难度 |
|---------|---------|---------|---------|
| 内存成本 | 中间结果内存占用 | `MemoryEstimatable` trait | 低 |
| 表达式成本 | 节点数、嵌套深度 | `Expression::node_count()` | 低 |
| 类型成本 | 类型处理系数 | `Value` 类型分类 | 低 |
| 行大小 | 平均行大小 | 统计信息 + 类型估算 | 中 |
| 缓存成本 | 缓存命中率 | 统计信息 + 访问模式 | 中 |
| 并行成本 | 并行化收益 | 成本模型 + 硬件信息 | 高 |

这些指标的纳入将显著提升查询优化器的决策质量，特别是在内存密集型操作（如大表聚合、复杂排序）和计算密集型操作（如复杂表达式过滤）的场景下。
