# Executor 模块迁移实施指南

## 文档概述

本文档基于已实施的 `executor_refactoring_plan.md` 提供具体的迁移实施指南，指导如何继续添加新的执行器到现有的模块化架构中。

**更新日期**: 2025-12-09  
**源代码**: `nebula-3.8.0/src/graph/executor`  
**目标架构**: 新 Rust GraphDB 单机版  
**状态**: 进行中（第一阶段已完成）

---

## 架构现状回顾

### 已完成的拆分结构

当前目录结构已按照规划完全实施：

```
src/query/executor/
├── base.rs                           # ✓ 基础执行器
├── data_access.rs                    # ✓ 数据访问（3个已实施）
├── data_modification.rs              # ✓ 数据修改（3个已实施）
├── data_processing/                  # ✓ 目录已建
│   ├── mod.rs                        # ✓ 模块导出
│   ├── filter.rs                     # ✓ FilterExecutor
│   ├── loops.rs                      # 预留
│   ├── graph_traversal/              # ✓ 子目录已建
│   │   ├── mod.rs                    # ✓ 导出
│   │   ├── expand.rs                 # ✓ ExpandExecutor
│   │   ├── expand_all.rs             # ✓ ExpandAllExecutor
│   │   ├── traverse.rs               # ✓ TraverseExecutor
│   │   └── shortest_path.rs          # ✓ ShortestPathExecutor
│   ├── join/                         # 预留
│   ├── set_operations/               # 预留
│   └── transformations/              # 预留
├── result_processing/                # ✓ 目录已建
│   ├── mod.rs                        # ✓ 导出
│   ├── projection.rs                 # ✓ ProjectExecutor
│   ├── aggregation.rs                # ✓ AggregateExecutor
│   ├── sorting.rs                    # ✓ SortExecutor
│   ├── limiting.rs                   # ✓ LimitExecutor, OffsetExecutor
│   ├── dedup.rs                      # ✓ DistinctExecutor
│   ├── sampling.rs                   # ✓ SampleExecutor
│   └── topn.rs                       # ✓ TopNExecutor
└── mod.rs                            # ✓ 顶级导出
```

**已实施统计**：
- ✓ 20 个执行器已实现
- 32 个执行器待实施

---

## 已实施执行器概览

### 第一优先级（全部完成）

| 执行器 | 位置 | 说明 |
|---|---|---|
| Executor | `base.rs` | 基础 trait |
| StartExecutor | `base.rs` | 查询入口 |
| GetVerticesExecutor | `data_access.rs` | 获取节点 |
| GetEdgesExecutor | `data_access.rs` | 获取边 |
| GetNeighborsExecutor | `data_access.rs` | 获取邻接 |
| FilterExecutor | `data_processing/filter.rs` | 条件过滤 |
| ExpandExecutor | `data_processing/graph_traversal/expand.rs` | 图展开 |
| ExpandAllExecutor | `data_processing/graph_traversal/expand_all.rs` | 全路径 |
| TraverseExecutor | `data_processing/graph_traversal/traverse.rs` | 图遍历 |
| ShortestPathExecutor | `data_processing/graph_traversal/shortest_path.rs` | 最短路径 |
| ProjectExecutor | `result_processing/projection.rs` | 列投影 |
| AggregateExecutor | `result_processing/aggregation.rs` | 聚合函数 |
| SortExecutor | `result_processing/sorting.rs` | 排序 |
| LimitExecutor | `result_processing/limiting.rs` | 限制行数 |
| OffsetExecutor | `result_processing/limiting.rs` | 跳过行数 |
| InsertExecutor | `data_modification.rs` | 插入数据 |
| UpdateExecutor | `data_modification.rs` | 更新数据 |
| DeleteExecutor | `data_modification.rs` | 删除数据 |
| DistinctExecutor | `result_processing/dedup.rs` | 去重 |
| TopNExecutor | `result_processing/topn.rs` | TOP N |
| SampleExecutor | `result_processing/sampling.rs` | 采样 |

---

## 继续迁移指南

### 迁移流程（通用步骤）

所有新执行器的迁移应遵循以下流程：

#### 步骤 1：创建文件和实现执行器

在对应的子模块目录中创建文件：

```rust
// 示例：data_processing/join/inner_join.rs
use crate::query::executor::base::{Executor, ExecutionContext, ExecutionResult};
use async_trait::async_trait;

/// INNER JOIN 执行器
pub struct InnerJoinExecutor {
    // 字段定义
    left_input: Box<dyn Executor>,
    right_input: Box<dyn Executor>,
    join_condition: String,
    // ... 其他必要字段
}

#[async_trait]
impl Executor for InnerJoinExecutor {
    async fn next(&mut self) -> ExecutionResult<Vec<Row>> {
        // 实现 JOIN 逻辑
    }
    
    fn output_schema(&self) -> &Schema {
        // 返回输出列信息
    }
    
    async fn close(&mut self) -> ExecutionResult<()> {
        // 清理资源
    }
}
```

#### 步骤 2：在子模块 `mod.rs` 中导出

编辑对应的 `mod.rs` 文件，添加模块声明和导出：

```rust
// 示例：data_processing/join/mod.rs
mod inner_join;
mod left_join;    // 之后添加
mod cross_join;   // 之后添加

pub use inner_join::InnerJoinExecutor;
pub use left_join::LeftJoinExecutor;
pub use cross_join::CartesianProductExecutor;
```

#### 步骤 3：在上级 `mod.rs` 中重导出

更新 `data_processing/mod.rs`（或其他上级模块）：

```rust
// data_processing/mod.rs 示例片段
pub mod join;
pub use join::{InnerJoinExecutor, LeftJoinExecutor, CartesianProductExecutor};
```

#### 步骤 4：在顶级 `executor/mod.rs` 中导出

最后更新 `src/query/executor/mod.rs` 的公共导出：

```rust
// 添加到 Re-export data processing executors 部分
pub use data_processing::{
    FilterExecutor, ProjectExecutor, SortExecutor, AggregateExecutor,
    ExpandExecutor, ExpandAllExecutor, TraverseExecutor, ShortestPathExecutor,
    ShortestPathAlgorithm,
    // 新添加
    InnerJoinExecutor, LeftJoinExecutor, CartesianProductExecutor,
};
```

#### 步骤 5：编写测试

为新执行器添加单元测试和集成测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_inner_join() {
        // 测试逻辑
    }
}
```

#### 步骤 6：验证编译和测试

```bash
# 检查编译
cargo check

# 编译代码
cargo build

# 运行测试
cargo test

# 检查代码风格
cargo fmt
cargo clippy
```

---

## 分阶段迁移计划

### 阶段 1：集合运算执行器（优先级：⭐⭐）

**目标目录**: `src/query/executor/data_processing/set_operations/`

#### 1.1 UnionExecutor（UNION 去重）

**文件**: `set_operations/union.rs`

```rust
/// UNION 执行器 - 合并两个结果集并去重
pub struct UnionExecutor {
    left_input: Box<dyn Executor>,
    right_input: Box<dyn Executor>,
    seen: HashSet<Row>,  // 用于去重
}

#[async_trait]
impl Executor for UnionExecutor {
    async fn next(&mut self) -> ExecutionResult<Vec<Row>> {
        // 从左输入获取行，添加到 seen
        // 从右输入获取行，检查是否在 seen 中
        // 返回未见过的行
    }
}
```

**关键考虑**:
- 去重需要对比整行数据
- 需要实现 Hash 和 Eq trait 用于去重
- 可考虑使用外部排序用于大数据集

#### 1.2 UnionAllExecutor（UNION ALL 不去重）

**文件**: `set_operations/union_all.rs`

```rust
/// UNION ALL 执行器 - 合并两个结果集（不去重）
pub struct UnionAllVersionVarExecutor {
    left_input: Box<dyn Executor>,
    right_input: Box<dyn Executor>,
    current_input: CurrentInput,  // 追踪当前输入
}

enum CurrentInput {
    Left,
    Right,
    Done,
}
```

**区别于 Union**:
- 不需要去重
- 实现更简单，性能更高
- 按顺序返回所有行

#### 1.3 IntersectExecutor（交集）

**文件**: `set_operations/intersect.rs`

```rust
/// INTERSECT 执行器 - 两个结果集的交集
pub struct IntersectExecutor {
    left_input: Box<dyn Executor>,
    right_input: Box<dyn Executor>,
    left_rows: HashSet<Row>,  // 缓存左输入
    right_rows: HashSet<Row>, // 缓存右输入
}
```

**实现策略**:
- 先读取左输入到集合
- 读取右输入，检查交集
- 返回既在左又在右的行

#### 1.4 MinusExecutor（差集 / EXCEPT）

**文件**: `set_operations/minus.rs`

```rust
/// MINUS/EXCEPT 执行器 - 从第一个集合中移除第二个集合的元素
pub struct MinusExecutor {
    left_input: Box<dyn Executor>,
    right_input: Box<dyn Executor>,
    right_rows: HashSet<Row>,  // 缓存右输入
}
```

**实现策略**:
- 先读取右输入到集合
- 读取左输入，过滤掉在右集合中的行
- 返回只在左存在的行

### 阶段 2：JOIN 执行器（优先级：⭐⭐）

**目标目录**: `src/query/executor/data_processing/join/`

#### 2.1 InnerJoinExecutor（内连接）

**文件**: `join/inner_join.rs`

```rust
/// INNER JOIN 执行器
pub struct InnerJoinExecutor {
    left_input: Box<dyn Executor>,
    right_input: Box<dyn Executor>,
    join_keys: Vec<String>,  // JOIN 条件的列名
    join_condition: Option<Expression>,  // 额外的 ON 条件
}

#[async_trait]
impl Executor for InnerJoinExecutor {
    async fn next(&mut self) -> ExecutionResult<Vec<Row>> {
        // 实现嵌套循环 JOIN 或哈希 JOIN
        // 只返回两边都匹配的行
    }
}
```

**优化策略**:
- **嵌套循环 JOIN** - 简单但低效，用于小数据集
- **哈希 JOIN** - 构建右表的哈希表，扫描左表
- **排序-合并 JOIN** - 如果两表都已排序

**推荐实现**: 哈希 JOIN（缓存右表）

#### 2.2 LeftJoinExecutor（左外连接）

**文件**: `join/left_join.rs`

```rust
/// LEFT OUTER JOIN 执行器
pub struct LeftJoinExecutor {
    left_input: Box<dyn Executor>,
    right_input: Box<dyn Executor>,
    join_keys: Vec<String>,
    join_condition: Option<Expression>,
    unmatched_right_filled: bool,
}
```

**区别于 InnerJoin**:
- 保留左表中所有行
- 右表不匹配的部分填充 NULL
- 需要追踪哪些右表行被匹配过

#### 2.3 CartesianProductExecutor（笛卡尔积）

**文件**: `join/cross_join.rs`

```rust
/// 笛卡尔积执行器 - CROSS JOIN
pub struct CartesianProductExecutor {
    left_input: Box<dyn Executor>,
    right_input: Box<dyn Executor>,
    left_rows: Vec<Row>,  // 缓存左表
    current_left_idx: usize,
    current_right: Option<Row>,
}
```

**特点**:
- 无 JOIN 条件
- 返回左表行数 × 右表行数 的结果
- 内存消耗大，应限制表大小

### 阶段 3：数据转换执行器（优先级：⭐⭐）

**目标目录**: `src/query/executor/data_processing/transformations/`

#### 3.1 AssignExecutor（变量赋值）

**文件**: `transformations/assign.rs`

```rust
/// ASSIGN 执行器 - 为变量赋值
pub struct AssignExecutor {
    input: Box<dyn Executor>,
    assignments: Vec<(String, Expression)>,  // 变量名 -> 表达式
}

#[async_trait]
impl Executor for AssignExecutor {
    async fn next(&mut self) -> ExecutionResult<Vec<Row>> {
        let mut input_rows = self.input.next().await?;
        
        for row in &mut input_rows {
            for (var_name, expr) in &self.assignments {
                let value = expr.evaluate(row)?;
                // 将值存储到执行上下文或行的变量部分
            }
        }
        
        Ok(input_rows)
    }
}
```

**功能**:
- 计算表达式值
- 将结果赋给变量
- 变量存储在执行上下文中

#### 3.2 AppendVerticesExecutor（追加顶点属性）

**文件**: `transformations/append_vertices.rs`

```rust
/// 追加顶点属性执行器
pub struct AppendVerticesExecutor {
    input: Box<dyn Executor>,
    vertex_ids: Vec<String>,  // 要追加的顶点 ID
    storage: Arc<dyn StorageEngine>,
}
```

**功能**:
- 获取输入中的顶点 ID
- 从存储中获取完整的顶点属性
- 合并到输出行中

#### 3.3 UnwindExecutor（展开列表）

**文件**: `transformations/unwind.rs`

```rust
/// UNWIND 执行器 - 展开列表成多行
pub struct UnwindExecutor {
    input: Box<dyn Executor>,
    unwinding_column: String,  // 要展开的列
    new_column_name: String,   // 展开后的新列名
}

#[async_trait]
impl Executor for UnwindExecutor {
    async fn next(&mut self) -> ExecutionResult<Vec<Row>> {
        let input_rows = self.input.next().await?;
        let mut output_rows = Vec::new();
        
        for row in input_rows {
            if let Some(Value::List(items)) = row.get(&self.unwinding_column) {
                for item in items {
                    let mut new_row = row.clone();
                    new_row.insert(self.new_column_name.clone(), item.clone());
                    output_rows.push(new_row);
                }
            }
        }
        
        Ok(output_rows)
    }
}
```

**示例**:
```
输入: [id: 1, tags: ["a", "b"]]
输出: [id: 1, tags: "a"]
      [id: 1, tags: "b"]
```

#### 3.4 PatternApplyExecutor（模式匹配）

**文件**: `transformations/pattern_apply.rs`

```rust
/// 模式匹配应用执行器
pub struct PatternApplyExecutor {
    input: Box<dyn Executor>,
    pattern: Pattern,  // 图模式
}
```

**复杂度高** - 涉及复杂的模式匹配算法

#### 3.5 RollUpApplyExecutor（ROLLUP 操作）

**文件**: `transformations/rollup.rs`

```rust
/// ROLLUP 操作执行器
pub struct RollUpApplyExecutor {
    input: Box<dyn Executor>,
    group_by_cols: Vec<String>,
    aggregations: Vec<AggregateFunc>,
}
```

### 阶段 4：图遍历扩展（优先级：⭐⭐）

**目标目录**: `src/query/executor/data_processing/graph_traversal/`

#### 4.1 AllPathsExecutor（所有路径）

**文件**: `graph_traversal/all_paths.rs`

```rust
/// 所有路径执行器 - 找出两点之间的所有简单路径
pub struct AllPathsExecutor {
    input: Box<dyn Executor>,
    start_vertex: String,
    end_vertex: String,
    max_length: Option<usize>,
    storage: Arc<dyn StorageEngine>,
}

#[async_trait]
impl Executor for AllPathsExecutor {
    async fn next(&mut self) -> ExecutionResult<Vec<Row>> {
        // 使用 DFS 找出所有路径
        // 需要处理循环避免
        // 可能结果很多
    }
}
```

**算法**: DFS（深度优先搜索）
- 维护访问过的顶点集合
- 回溯时移除访问标记

#### 4.2 SubgraphExecutor（子图提取）

**文件**: `graph_traversal/subgraph.rs`

```rust
/// 子图提取执行器
pub struct SubgraphExecutor {
    input: Box<dyn Executor>,
    vertex_filter: Option<Expression>,
    edge_filter: Option<Expression>,
    storage: Arc<dyn StorageEngine>,
}
```

**功能**:
- 根据条件过滤顶点
- 根据条件过滤边
- 返回满足条件的子图

### 阶段 5：结果处理补充（优先级：⭐⭐）

**目标目录**: `src/query/executor/result_processing/`

#### 5.1 DataCollectExecutor（结果收集）

**文件**: `result_processing/collect.rs`

```rust
/// 结果收集执行器 - 收集所有结果到内存
pub struct DataCollectExecutor {
    input: Box<dyn Executor>,
    collected_data: Vec<Row>,
}

#[async_trait]
impl Executor for DataCollectExecutor {
    async fn next(&mut self) -> ExecutionResult<Vec<Row>> {
        // 一次性返回所有结果
        // 适用于需要完整结果集的场景
    }
}
```

**注意**: 可能内存消耗大，应限制结果集大小

#### 5.2 SetExecutor（SET 语句）

**文件**: `result_processing/set.rs`

```rust
/// SET 执行器 - 设置变量或配置
pub struct SetExecutor {
    variable_name: String,
    value: Expression,
}
```

### 阶段 6：数据访问补充（优先级：⭐⭐）

**目标文件**: `src/query/executor/data_access.rs`

添加以下执行器到现有文件：

#### 6.1 GetPropExecutor（获取属性）

```rust
/// 获取属性执行器 - 优化版本的属性获取
pub struct GetPropExecutor {
    vertex_ids: Vec<String>,
    properties: Vec<String>,
    storage: Arc<dyn StorageEngine>,
}
```

#### 6.2 IndexScanExecutor（索引扫描）

```rust
/// 索引扫描执行器
pub struct IndexScanExecutor {
    index_name: String,
    predicates: Vec<Predicate>,
    index_engine: Arc<dyn IndexEngine>,
}
```

#### 6.3 ScanVerticesExecutor 和 ScanEdgesExecutor（全表扫描）

```rust
/// 全表扫描顶点执行器
pub struct ScanVerticesExecutor {
    vertex_label: String,
    filter: Option<Expression>,
    storage: Arc<dyn StorageEngine>,
}
```

---

## 实现要点和最佳实践

### 1. 异步编程

所有执行器应使用 async/await 模式：

```rust
#[async_trait]
impl Executor for MyExecutor {
    async fn next(&mut self) -> ExecutionResult<Vec<Row>> {
        // 异步操作
    }
}
```

### 2. 错误处理

使用 `ExecutionResult<T>` 处理错误：

```rust
pub type ExecutionResult<T> = Result<T, ExecutionError>;

pub enum ExecutionError {
    StorageError(String),
    TypeError(String),
    IndexError(String),
    QueryError(String),
}
```

### 3. 内存管理

- 对大数据集进行流式处理
- 必要时使用外部排序或哈希表
- 及时释放不需要的数据

### 4. 执行上下文

所有执行器应能访问执行上下文：

```rust
pub struct ExecutionContext {
    pub storage: Arc<dyn StorageEngine>,
    pub index: Arc<dyn IndexEngine>,
    pub scope: HashMap<String, Value>,
}
```

### 5. 性能考虑

- **选择合适的算法**: 根据数据规模选择 JOIN 算法
- **索引利用**: 在可能的地方使用索引
- **批处理**: 返回批量行而非单行
- **内存限制**: 避免一次性加载全部数据

### 6. 代码组织

按功能分组相关的执行器：
- 相同类别的执行器放在同一文件或子目录
- 共享的代码提取到公共模块
- 明确的模块导出和重导出

---

## 测试策略

### 单元测试

为每个执行器创建测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    
    #[tokio::test]
    async fn test_basic_functionality() {
        // 测试基本功能
    }
    
    #[tokio::test]
    async fn test_empty_input() {
        // 测试空输入
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        // 测试错误处理
    }
}
```

### 集成测试

在 `tests/` 目录创建集成测试：

```rust
// tests/executor_integration_tests.rs
#[tokio::test]
async fn test_union_executor_pipeline() {
    // 测试 Union 执行器与其他执行器的组合
}
```

### 验证清单

每个新执行器应通过以下检查：

- [ ] 编译通过 (`cargo check`)
- [ ] 单元测试通过 (`cargo test`)
- [ ] 代码格式 (`cargo fmt`)
- [ ] Clippy 检查 (`cargo clippy`)
- [ ] 文档注释完整
- [ ] 在 `mod.rs` 正确导出
- [ ] 性能合理（无明显性能回归）

---

## 迁移进度追踪

### 完成度统计

| 阶段 | 执行器数量 | 完成数 | 进度 |
|---|---|---|---|
| 阶段 1：集合运算 | 4 | 0 | 0% |
| 阶段 2：JOIN | 3 | 0 | 0% |
| 阶段 3：数据转换 | 5 | 0 | 0% |
| 阶段 4：图遍历扩展 | 2 | 0 | 0% |
| 阶段 5：结果处理 | 2 | 0 | 0% |
| 阶段 6：数据访问 | 5 | 0 | 0% |
| **总计** | **21** | **0** | **0%** |

### 优先级任务

**立即开始**（阶段 1-2）:
1. UnionExecutor
2. InnerJoinExecutor  
3. LeftJoinExecutor
4. UnwindExecutor
5. AssignExecutor

**次优先**（阶段 3-4）:
6. IntersectExecutor
7. MinusExecutor
8. CartesianProductExecutor
9. AllPathsExecutor
10. AppendVerticesExecutor

---

## 参考文档和资源

### 相关文档

- `executor_refactoring_plan.md` - 详细的拆分方案
- `executor_mapping_table.md` - NebulaGraph 到新架构的映射
- `README.md` - 项目概述

### 代码参考

- `src/query/executor/base.rs` - 基础 trait 定义
- `src/query/executor/data_processing/filter.rs` - 现有实现参考
- `src/query/executor/result_processing/projection.rs` - 现有实现参考

### 相关工具

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy

# 运行测试
cargo test

# 生成文档
cargo doc --open
```

---

**文档版本**: v2.0  
**最后更新**: 2025-12-09  
**状态**: 持续更新中  
**下一步**: 开始实施阶段 1 的集合运算执行器
