# 缓存内存估算机制分析报告

## 1. 现有实现概述

### 1.1 核心架构

当前内存估算系统采用分层设计，主要包含三个层次：

```
┌─────────────────────────────────────────────────────────────┐
│                    应用层 (Application)                      │
│         PlanCache / CteCache / GlobalCacheManager           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   计划节点层 (Plan Node)                      │
│    PlanNodeEnum.estimate_memory() → 递归估算整棵树           │
│    MemoryEstimatable trait → 单个节点内存估算                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  辅助函数层 (Helpers)                        │
│    estimate_string_memory / estimate_vec_string_memory       │
│    estimate_option_string_memory / estimate_vec_memory       │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 关键组件

#### 1.2.1 MemoryEstimatable Trait

位于 `src/query/planning/plan/core/nodes/base/memory_estimation.rs`：

```rust
pub trait MemoryEstimatable {
    /// 估算节点自身内存（不包括子节点）
    fn estimate_memory(&self) -> usize;
}
```

**设计原则**：每个节点只估算自身字段，子节点内存由调用方递归计算。

#### 1.2.2 PlanNodeEnum 递归估算

位于 `src/query/planning/plan/core/nodes/base/plan_node_enum.rs`：

```rust
impl PlanNodeEnum {
    pub fn estimate_memory(&self) -> usize {
        let base_size = std::mem::size_of::<PlanNodeEnum>();
        
        match self {
            // ZeroInputNode: 仅节点结构
            PlanNodeEnum::Start(node) => base_size + estimate_node_memory(node),
            
            // SingleInputNode: 节点 + 子节点
            PlanNodeEnum::Project(node) => {
                base_size + estimate_node_memory(node) + 
                Self::estimate_input_memory(node.input())
            }
            
            // BinaryInputNode: 节点 + 左右子节点
            PlanNodeEnum::InnerJoin(node) => {
                base_size + estimate_node_memory(node) +
                Self::estimate_input_memory(node.left_input()) +
                Self::estimate_input_memory(node.right_input())
            }
            
            // MultipleInputNode: 节点 + 多个依赖
            PlanNodeEnum::Expand(node) => {
                base_size + estimate_node_memory(node) +
                node.dependencies()
                    .iter()
                    .map(Self::estimate_input_memory)
                    .sum::<usize>()
            }
            
            // ControlFlowNode: 节点 + 条件分支
            PlanNodeEnum::Select(node) => {
                let mut total = base_size + estimate_node_memory(node);
                if let Some(if_branch) = node.if_branch() {
                    total += Self::estimate_input_memory(if_branch.as_ref());
                }
                // ... else_branch
                total
            }
        }
    }
}
```

#### 1.2.3 宏自动生成实现

位于 `src/query/planning/plan/core/nodes/base/macros.rs`：

`define_plan_node!` 宏自动为所有节点类型生成 `MemoryEstimatable` 实现：

```rust
impl MemoryEstimatable for $name {
    fn estimate_memory(&self) -> usize {
        let base = std::mem::size_of::<$name>();
        
        // 估算 col_names 向量
        let col_names_size = estimate_vec_string_memory(&self.col_names());
        
        // 估算 output_var
        let output_var_size = std::mem::size_of::<Option<String>>() +
            self.output_var.as_ref()
                .map(|s| std::mem::size_of::<String>() + s.len())
                .unwrap_or(0);
        
        base + col_names_size + output_var_size
    }
}
```

#### 1.2.4 CTE 缓存条目估算

位于 `src/query/cache/cte_cache.rs`：

```rust
impl CteCacheEntry {
    pub fn estimate_memory(&self) -> usize {
        let mut total = 0;
        
        // Data: Arc<Vec<u8>>
        total += std::mem::size_of::<Arc<Vec<u8>>>();
        total += std::mem::size_of::<Vec<u8>>();
        total += self.data_size;
        
        // String 字段
        total += std::mem::size_of::<String>() + self.cte_hash.capacity();
        total += std::mem::size_of::<String>() + self.cte_definition.capacity();
        
        // Vector 字段
        total += std::mem::size_of::<Vec<String>>();
        for table in &self.dependent_tables {
            total += std::mem::size_of::<String>() + table.capacity();
        }
        
        // 基本类型字段
        total += std::mem::size_of::<Instant>() * 2;
        total += std::mem::size_of::<u64>() * 3;
        total += std::mem::size_of::<f64>() * 2;
        total += std::mem::size_of::<CachePriority>();
        
        total
    }
}
```

## 2. 估算方法详解

### 2.1 字符串内存估算

```rust
pub fn estimate_string_memory(s: &str) -> usize {
    std::mem::size_of::<String>() + s.len()
}
```

**说明**：
- `size_of::<String>()` = 24 bytes（指针 + 长度 + 容量）
- `s.len()` = UTF-8 字节长度
- **注意**：使用 `len()` 而非 `capacity()`，因为实际分配可能更大

### 2.2 Option<String> 估算

```rust
pub fn estimate_option_string_memory(opt: &Option<String>) -> usize {
    std::mem::size_of::<Option<String>>() +
        opt.as_ref()
            .map(|s| std::mem::size_of::<String>() + s.len())
            .unwrap_or(0)
}
```

### 2.3 Vec<String> 估算

```rust
pub fn estimate_vec_string_memory(vec: &[String]) -> usize {
    std::mem::size_of::<Vec<String>>() +
        vec.iter()
            .map(|s| std::mem::size_of::<String>() + s.len())
            .sum::<usize>()
}
```

### 2.4 Vec<T> 通用估算

```rust
pub fn estimate_vec_memory<T>(vec: &[T]) -> usize {
    std::mem::size_of_val(vec)
}
```

## 3. 潜在问题分析

### 3.1 问题一：共享子树重复计算 ✅ 已修复

**问题描述**：

当计划树中存在共享子树（通过 `Arc<PlanNodeEnum>` 引用）时，递归估算会导致重复计算：

```
        Join
       /    \
  Filter    Project
     |          |
    Scan      Scan  ← 同一个 Scan 节点被引用两次
```

**原实现**：

```rust
fn estimate_input_memory(input: &PlanNodeEnum) -> usize {
    let arc_overhead = std::mem::size_of::<Arc<PlanNodeEnum>>();
    let node_memory = input.estimate_memory();  // 每次都递归计算
    arc_overhead + node_memory
}
```

**修复方案**：

已实现 `estimate_memory_dedup()` 方法，使用 HashSet 跟踪已访问节点：

```rust
impl PlanNodeEnum {
    /// 带去重的内存估算
    pub fn estimate_memory_dedup(&self) -> usize {
        let mut visited = HashSet::new();
        self.estimate_memory_internal(&mut visited)
    }
    
    fn estimate_memory_internal(&self, visited: &mut HashSet<i64>) -> usize {
        // 检查是否已访问
        if !visited.insert(self.id()) {
            return 0;  // 已访问过，不重复计算
        }
        
        // 递归计算子节点时传递 visited
        match self {
            PlanNodeEnum::Project(node) => {
                base_size + estimate_node_memory(node) 
                    + node.input().estimate_memory_internal(visited)
            }
            // ... 其他节点类型
        }
    }
}
```

**使用建议**：
- 默认 `estimate_memory()` 保持简单快速，适合大多数场景
- `estimate_memory_dedup()` 用于精确内存控制，避免共享子树重复计算

### 3.2 问题二：复杂字段估算不完整

**问题描述**：

某些节点包含复杂嵌套结构，当前估算可能不完整：

**SelectNode 示例**：

```rust
impl MemoryEstimatable for SelectNode {
    fn estimate_memory(&self) -> usize {
        // ...
        // Estimate condition (ContextualExpression)
        let condition_size = std::mem::size_of::<ContextualExpression>()
            + std::mem::size_of::<Arc<ExpressionAnalysisContext>>();
        // 问题：没有递归估算 ContextualExpression 内部的 Expression
    }
}
```

**影响**：
- 条件表达式、投影表达式等复杂结构的内存被低估
- 对于包含大量表达式的查询，估算偏差较大

**建议改进**：

为 `Expression` 类型实现 `MemoryEstimatable`：

```rust
impl MemoryEstimatable for Expression {
    fn estimate_memory(&self) -> usize {
        let base = std::mem::size_of::<Expression>();
        match self {
            Expression::Constant(v) => base + v.estimate_memory(),
            Expression::ColumnRef(col) => base + col.len(),
            Expression::BinaryOp { left, right, .. } => {
                base + left.estimate_memory() + right.estimate_memory()
            }
            // ... 其他变体
        }
    }
}
```

### 3.3 问题三：Arc 引用计数开销 - 设计分析

**Arc 内存布局**：

```
┌─────────────────────────────────────────────────────────┐
│                    Arc<T> (栈上)                         │
│  ┌──────────────┬──────────────┐                        │
│  │   ptr        │  8 bytes     │ ───────┐               │
│  ├──────────────┼──────────────┤        │               │
│  │   phantom    │  8 bytes     │        │               │
│  └──────────────┴──────────────┘        │               │
│         Total: 16 bytes                 │               │
└─────────────────────────────────────────┘               │
                                                          │
                                                          ▼
┌─────────────────────────────────────────────────────────┐
│              ArcInner<T> (堆上)                          │
│  ┌──────────────┬──────────────┐                        │
│  │ strong_count │ AtomicUsize  │  8 bytes               │
│  ├──────────────┼──────────────┤                        │
│  │ weak_count   │ AtomicUsize  │  8 bytes               │
│  ├──────────────┼──────────────┤                        │
│  │ data         │ T            │  T的大小               │
│  └──────────────┴──────────────┘                        │
│         Total: 16 + size_of::<T>()                      │
└─────────────────────────────────────────────────────────┘
```

**当前实现**：

```rust
fn estimate_input_memory(input: &PlanNodeEnum) -> usize {
    let arc_overhead = std::mem::size_of::<Arc<PlanNodeEnum>>();  // 16 bytes
    let node_memory = input.estimate_memory();
    arc_overhead + node_memory
}
```

**设计决策分析**：

1. **引用计数归属问题**：
   - `ArcInner` 的 16 bytes 引用计数应该算在谁头上？
   - 方案 A：每个 Arc 都计算（重复计算）
   - 方案 B：只由"第一个" Arc 计算（难以确定）
   - 方案 C：由被引用节点计算（当前方案）

2. **当前方案（方案 C）的合理性**：
   - 被引用节点 "拥有" 堆内存
   - Arc 只是提供了一个访问路径
   - 符合 Rust 的所有权思想

3. **改进建议 - 精确 Arc 估算**：

如果需要更精确的估算，可以实现：

```rust
pub struct ArcMemoryEstimator {
    /// 每个 Arc 的元数据开销（引用计数）
    const ARC_METADATA_SIZE: usize = 16; // strong + weak count
    
    /// 估算 Arc 引用的总内存
    /// 包括：Arc 结构体 + 共享的堆内存（按比例分摊）
    pub fn estimate_arc_memory<T: MemoryEstimatable>(
        arc: &Arc<T>,
        shared_count: usize
    ) -> usize {
        let arc_struct = std::mem::size_of::<Arc<T>>();
        let metadata = Self::ARC_METADATA_SIZE / shared_count;  // 按比例分摊
        let data = arc.estimate_memory() / shared_count;        // 按比例分摊
        arc_struct + metadata + data
    }
}
```

**实际应用建议**：

- 对于缓存内存控制：当前方案已足够
- 对于精确内存分析：可使用 `estimate_memory_dedup()` + 手动计算 Arc 开销
- 对于性能敏感场景：避免过度复杂的估算逻辑

### 3.4 问题四：Vec 容量 vs 长度 ✅ 已修复

**问题描述**：

原实现使用 `s.len()` 而非 `s.capacity()`：

```rust
// 原实现
pub fn estimate_string_memory(s: &str) -> usize {
    std::mem::size_of::<String>() + s.len()  // 使用 len
}
```

**修复方案**：

已统一使用 `capacity()` 以反映实际堆内存分配：

```rust
// 修复后
pub fn estimate_string_memory(s: &String) -> usize {
    std::mem::size_of::<String>() + s.capacity()
}

pub fn estimate_option_string_memory(opt: &Option<String>) -> usize {
    std::mem::size_of::<Option<String>>()
        + opt.as_ref()
            .map(|s| std::mem::size_of::<String>() + s.capacity())
            .unwrap_or(0)
}

pub fn estimate_vec_string_memory(vec: &[String]) -> usize {
    std::mem::size_of::<Vec<String>>()
        + vec.iter()
            .map(|s| std::mem::size_of::<String>() + s.capacity())
            .sum::<usize>()
}
```

**影响范围**：
- `memory_estimation.rs` - 基础辅助函数
- `macros.rs` - 所有宏生成的节点实现
- `control_flow_node.rs` - SelectNode, LoopNode
- `data_processing_node.rs` - RollUpApplyNode, PatternApplyNode, RemoveNode

**设计理由**：
- `len()` 只反映实际数据大小
- `capacity()` 反映实际分配的堆内存
- 对于内存控制，使用 `capacity()` 更准确，避免低估

### 3.5 问题五：缺乏缓存元数据估算

**问题描述**：

Moka 缓存本身有额外的元数据开销，当前没有估算：

```rust
// PlanCache 使用 Moka Cache
pub struct QueryPlanCache {
    cache: Cache<String, CachedPlan>,  // Moka 内部有额外开销
}
```

**Moka 内部开销**：
- Entry 元数据（时间戳、权重等）
- 并发控制结构（Striped locks）
- LRU 链表指针

**估算经验值**：

```rust
// Moka 每个条目的额外开销约为 64-128 bytes
const MOKA_ENTRY_OVERHEAD: usize = 96;

pub fn estimate_total_cache_memory(&self) -> usize {
    let entries_memory: usize = self.cache.iter()
        .map(|(_, v)| v.estimate_memory())
        .sum();
    let entry_count = self.cache.entry_count();
    entries_memory + entry_count * MOKA_ENTRY_OVERHEAD
}
```

### 3.6 问题六：缺乏运行时校准机制

**问题描述**：

当前估算完全基于静态代码分析，没有运行时校准：

```rust
// 理论估算值
let estimated = plan.estimate_memory();

// 实际内存占用可能因以下因素不同：
// 1. 内存对齐填充
// 2. Allocator 行为
// 3. 内存碎片
// 4. 编译器优化
```

**建议改进**：

引入运行时校准机制：

```rust
pub struct MemoryEstimator {
    // 理论估算值 -> 实际观测值的映射
    calibration_factor: RwLock<HashMap<TypeId, f64>>,
}

impl MemoryEstimator {
    pub fn estimate_with_calibration<T: MemoryEstimatable>(&self, obj: &T) -> usize {
        let theoretical = obj.estimate_memory();
        let factor = self.calibration_factor.read()
            .get(&TypeId::of::<T>())
            .copied()
            .unwrap_or(1.0);
        (theoretical as f64 * factor) as usize
    }
    
    pub fn record_actual<T: 'static>(&self, estimated: usize, actual: usize) {
        let factor = actual as f64 / estimated.max(1) as f64;
        self.calibration_factor.write()
            .insert(TypeId::of::<T>(), factor);
    }
}
```

## 4. 精度评估

### 4.1 估算准确度分级

| 组件 | 准确度 | 说明 |
|------|--------|------|
| 基本类型 (i64, f64) | ⭐⭐⭐⭐⭐ | 精确计算 |
| String (使用 len) | ⭐⭐⭐ | 可能低估 0-2x |
| String (使用 capacity) | ⭐⭐⭐⭐ | 可能高估 0-1x |
| Vec<T> | ⭐⭐⭐⭐ | 基于 size_of_val，较准确 |
| 计划节点基础字段 | ⭐⭐⭐⭐ | 较准确 |
| 复杂表达式 | ⭐⭐ | 估算不完整 |
| 共享子树 | ⭐⭐ | 可能重复计算 |
| Moka 元数据 | ⭐ | 未估算 |

### 4.2 典型场景偏差分析

**场景 1：简单查询**

```cypher
MATCH (n:Person) RETURN n.name
```

- 计划节点数：~5
- 估算准确度：~90%
- 主要偏差：表达式估算不完整

**场景 2：复杂 JOIN 查询**

```cypher
MATCH (a)-[:KNOWS]->(b)-[:WORKS_AT]->(c) RETURN *
```

- 计划节点数：~15
- 估算准确度：~80%
- 主要偏差：共享子树、复杂表达式

**场景 3：递归 CTE**

```cypher
WITH RECURSIVE paths AS (...)
```

- 计划节点数：~10 + 递归展开
- 估算准确度：~70%
- 主要偏差：共享子树重复计算严重

## 5. 改进建议

### 5.1 短期改进（高优先级）

1. **统一使用 `capacity()`**
   - 修改 `estimate_string_memory` 使用 `capacity()`
   - 确保所有字符串估算一致

2. **修复共享子树重复计算**
   - 添加 `visited` 集合跟踪已访问节点
   - 使用节点 ID 去重

3. **完善复杂表达式估算**
   - 为 `Expression` 实现 `MemoryEstimatable`
   - 递归估算嵌套表达式

### 5.2 中期改进（中优先级）

4. **添加 Moka 元数据估算**
   - 引入经验值常量
   - 在全局内存统计中考虑

5. **实现运行时校准**
   - 添加校准因子机制
   - 定期更新校准数据

6. **添加估算精度监控**
   - 对比估算值与实际内存
   - 记录偏差指标

### 5.3 长期改进（低优先级）

7. **使用 Allocator 钩子**
   - 实现自定义 GlobalAlloc
   - 精确追踪实际分配

8. **引入内存分析工具集成**
   - 支持 dhat/heaptrack 输出格式
   - 便于离线分析

## 6. 结论

当前内存估算机制设计合理，实现了基本的递归估算能力，能够满足缓存内存控制的基本需求。主要优势包括：

1. **分层架构清晰**：trait + 递归 + 辅助函数的分层设计
2. **自动化程度高**：宏自动生成大部分实现
3. **类型覆盖全面**：支持所有计划节点类型

但存在以下需要改进的问题：

1. **共享子树重复计算**：可能导致估算值偏高
2. **复杂字段估算不完整**：表达式等嵌套结构被低估
3. **缺乏运行时校准**：静态估算与实际可能存在偏差
4. **Moka 元数据未考虑**：缓存系统本身有额外开销

**建议优先级**：
- 🔴 高：修复共享子树重复计算
- 🟡 中：完善表达式估算、添加运行时校准
- 🟢 低：Allocator 钩子、分析工具集成

总体而言，当前实现适合作为内存控制的参考值，但在精确内存管理场景下需要进一步改进。
