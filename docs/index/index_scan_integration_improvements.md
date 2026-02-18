# 索引扫描集成改进方案

## 一、概述

本文档记录 GraphDB 项目中索引扫描（IndexScan）与查询执行流程集成的改进方案，包括已完成的工作和未来的改进方向。

## 二、已完成的工作

### 2.1 安全修复

**文件**: `src/storage/index/index_data_manager.rs`

修复了 `deserialize_value` 函数中的 `unwrap` 使用，改为显式模式匹配：

```rust
// 修改前
Value::Int(_) => i64::from_be_bytes(data.try_into().unwrap_or([0; 8])).into(),

// 修改后
Value::Int(_) => {
    let bytes: [u8; 8] = match data.try_into() {
        Ok(b) => b,
        Err(_) => return Value::Null(crate::core::NullType::Null),
    };
    i64::from_be_bytes(bytes).into()
}
```

### 2.2 LookupPlanner 改进

**文件**: `src/query/planner/statements/lookup_planner.rs`

将 `get_available_indexes` 方法从简化实现改进为从元数据服务获取真实索引：

```rust
/// 获取可用的索引列表
/// 从元数据服务获取真实的索引列表
fn get_available_indexes(
    &self,
    ast_ctx: &AstContext,
    space_id: u64,
    schema_id: i32,
    is_edge: bool,
) -> Result<Vec<Index>, PlannerError> {
    // 从查询上下文中获取索引元数据管理器
    let index_manager = ast_ctx.index_metadata_manager()
        .ok_or_else(|| PlannerError::PlanGenerationFailed(
            "Index metadata manager not available".to_string()
        ))?;

    // 获取schema名称
    let schema_name = if is_edge {
        ast_ctx.get_edge_type_name_by_id(space_id, schema_id)
            .ok_or_else(|| PlannerError::PlanGenerationFailed(
                format!("Edge type not found for ID: {}", schema_id)
            ))?
    } else {
        ast_ctx.get_tag_name_by_id(space_id, schema_id)
            .ok_or_else(|| PlannerError::PlanGenerationFailed(
                format!("Tag not found for ID: {}", schema_id)
            ))?
    };

    // 从元数据服务获取索引列表
    let indexes = if is_edge {
        index_manager.list_edge_indexes(space_id as i32)
            .map_err(|e| PlannerError::PlanGenerationFailed(
                format!("Failed to list edge indexes: {}", e)
            ))?
    } else {
        index_manager.list_tag_indexes(space_id as i32)
            .map_err(|e| PlannerError::PlanGenerationFailed(
                format!("Failed to list tag indexes: {}", e)
            ))?
    };

    // 过滤出与当前schema相关的索引
    let schema_indexes: Vec<Index> = indexes
        .into_iter()
        .filter(|idx| idx.schema_name == schema_name && idx.status == crate::index::IndexStatus::Active)
        .collect();

    if schema_indexes.is_empty() {
        return Err(PlannerError::PlanGenerationFailed(
            format!("No active indexes found for {}: {}", 
                if is_edge { "edge" } else { "tag" }, 
                schema_name
            )
        ));
    }

    Ok(schema_indexes)
}
```

### 2.3 AstContext 扩展

**文件**: `src/query/context/ast/base.rs`

添加了三个新方法支持索引查询：

```rust
/// 获取索引元数据管理器
pub fn index_metadata_manager(&self) -> Option<&std::sync::Arc<dyn crate::storage::metadata::IndexMetadataManager>> {
    self.qctx.as_ref().and_then(|qctx| qctx.index_metadata_manager())
}

/// 根据ID获取标签名称
pub fn get_tag_name_by_id(&self, space_id: u64, tag_id: i32) -> Option<String> {
    let qctx = self.qctx.as_ref()?;
    let schema_manager = qctx.schema_manager()?;
    let space = schema_manager.get_space_by_id(space_id as i32).ok()??;
    let tags = schema_manager.list_tags(&space.space_name).ok()?;
    tags.into_iter()
        .find(|tag| tag.tag_id == tag_id)
        .map(|tag| tag.tag_name)
}

/// 根据ID获取边类型名称
pub fn get_edge_type_name_by_id(&self, space_id: u64, edge_type_id: i32) -> Option<String> {
    let qctx = self.qctx.as_ref()?;
    let schema_manager = qctx.schema_manager()?;
    let space = schema_manager.get_space_by_id(space_id as i32).ok()??;
    let edge_types = schema_manager.list_edge_types(&space.space_name).ok()?;
    edge_types.into_iter()
        .find(|edge| edge.edge_type_id == edge_type_id)
        .map(|edge| edge.edge_type_name)
}
```

## 三、当前架构状态

### 3.1 组件清单

| 组件 | 状态 | 文件路径 |
|------|------|----------|
| IndexScan 计划节点 | 已实现 | `src/query/planner/plan/algorithms/index_scan.rs` |
| EdgeIndexScanNode 计划节点 | 已实现 | `src/query/planner/plan/core/nodes/graph_scan_node.rs` |
| IndexScanExecutor 执行器 | 已实现 | `src/query/executor/search_executors.rs` |
| LookupPlanner 规划器 | 已实现 | `src/query/planner/statements/lookup_planner.rs` |
| IndexFullScanRule 优化规则 | 已实现 | `src/query/optimizer/rules/scan/index_full_scan.rs` |
| PushLimitDownIndexScanRule | 已实现 | `src/query/optimizer/rules/limit_pushdown/push_limit_down_index_scan.rs` |
| OptimizeEdgeIndexScanByFilterRule | 已实现 | `src/query/optimizer/rules/index/optimize_edge_index_scan_by_filter.rs` |
| 执行器工厂集成 | 已实现 | `src/query/executor/factory.rs` |
| 存储层 lookup_index | 已实现 | `src/storage/storage_client.rs` |
| 查询流水线管理器 | 已注册 | `src/query/query_pipeline_manager.rs` |

### 3.2 查询执行流程

```
LOOKUP 语句
    ↓
Parser 解析
    ↓
Validator 验证
    ↓
LookupPlanner.transform() → 创建 IndexScan/EdgeIndexScan 计划节点
    ↓
Optimizer.apply_rules() → 应用优化规则
    - IndexFullScanRule: 选择最优索引
    - PushLimitDownIndexScanRule: 将 LIMIT 下推
    - OptimizeEdgeIndexScanByFilterRule: 基于过滤器优化
    ↓
ExecutorFactory.create_executor() → 创建 IndexScanExecutor
    ↓
IndexScanExecutor.execute()
    ├── lookup_by_index() → 调用 StorageClient.lookup_index()
    ├── fetch_entities() → 获取完整实体
    ├── apply_filter() → 应用过滤器
    └── project_columns() → 投影返回列
```

## 四、与 nebula-graph 的差异

| 方面 | nebula-graph | GraphDB |
|------|-------------|---------|
| **架构** | 分布式，需要 StorageClient 与存储服务通信 | 单节点，直接访问存储层 |
| **计划节点** | TagIndexFullScan / EdgeIndexFullScan | IndexScan（通过 is_edge_scan 区分）/ EdgeIndexScanNode |
| **优化器** | 基于 RBO + CBO 的复杂优化器 | 简化版规则优化器 |
| **执行方式** | 异步执行 (folly::Future) | 同步执行 |
| **索引选择** | 基于统计信息的代价模型 | 基于字段匹配的评分模型 |

## 五、未来改进方向

### 5.1 PushTopNDownIndexScanRule

**状态**: 待实现

**描述**: 将 TopN 操作下推到 IndexScan，避免扫描过多数据。

**模式**:
```
Before:
  TopN(count, sort_items)
      |
  IndexScan

After:
  IndexScan(limit=count, order_by=sort_items)
```

**实现位置**: `src/query/optimizer/rules/limit_pushdown/push_topn_down_index_scan.rs`

**参考实现**:
```rust
//! 将TopN下推到索引扫描的规则

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result as OptResult};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PushTopNDownIndexScanRule;

impl OptRule for PushTopNDownIndexScanRule {
    fn name(&self) -> &str {
        "PushTopNDownIndexScanRule"
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency("TopN", "IndexScan")
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = node.borrow();

        if !node_ref.plan_node.is_topn() {
            return Ok(None);
        }

        let topn_node = match node_ref.plan_node.as_topn() {
            Some(n) => n,
            None => return Ok(None),
        };

        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(n) => n,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();

        if !child_ref.plan_node.is_index_scan() {
            return Ok(None);
        }

        let index_scan = match child_ref.plan_node.as_index_scan() {
            Some(s) => s,
            None => return Ok(None),
        };

        // 创建新的 IndexScan，集成 TopN 的限制和排序
        let mut new_index_scan = index_scan.clone();
        new_index_scan.limit = Some(topn_node.limit() as i64);
        // 如果索引支持排序，可以设置 order_by

        let mut new_group_node = child_ref.clone();
        new_group_node.plan_node = PlanNodeEnum::IndexScan(new_index_scan);

        let mut result = TransformResult::new();
        result.add_new_group_node(Rc::new(RefCell::new(new_group_node)));
        result.erase_curr = true;

        Ok(Some(result))
    }
}

impl BaseOptRule for PushTopNDownIndexScanRule {}
```

### 5.2 基于统计信息的索引选择

**状态**: 待改进

**当前实现**: `src/query/optimizer/index_selector.rs`

基于简单的字段匹配评分：
```rust
pub enum IndexScore {
    NoMatch = 0,
    NotEqual = 1,
    Range = 2,
    Prefix = 3,
    FullMatch = 4,
}
```

**改进方案**: 利用已有的统计信息基础设施 (`src/query/optimizer/core/cost.rs`)

```rust
pub struct CostBasedIndexSelector;

impl CostBasedIndexSelector {
    /// 基于统计信息计算索引扫描代价
    fn calculate_index_cost(
        index: &Index,
        constraints: &HashMap<String, ColumnConstraint>,
        table_stats: &TableStats,
    ) -> Cost {
        let mut cost = Cost::default();
        
        // 1. 计算选择率
        let selectivity = Self::calculate_selectivity(index, constraints, table_stats);
        
        // 2. 估算扫描行数
        let estimated_rows = (table_stats.row_count as f64 * selectivity) as u64;
        
        // 3. 计算IO代价（索引页访问）
        cost.io_cost = Self::estimate_io_cost(index, estimated_rows);
        
        // 4. 计算CPU代价（比较操作）
        cost.cpu_cost = Self::estimate_cpu_cost(estimated_rows);
        
        // 5. 考虑索引类型（唯一索引有额外优势）
        if index.is_unique && selectivity == 1.0 {
            cost.io_cost *= 0.1;
        }
        
        cost
    }
    
    /// 计算选择率
    fn calculate_selectivity(
        index: &Index,
        constraints: &HashMap<String, ColumnConstraint>,
        table_stats: &TableStats,
    ) -> f64 {
        let mut selectivity = 1.0;
        
        for field in &index.fields {
            if let Some(constraint) = constraints.get(&field.name) {
                if let Some(col_stats) = table_stats.column_stats.get(&field.name) {
                    match constraint {
                        ColumnConstraint::Equal(_) => {
                            selectivity *= 1.0 / col_stats.distinct_count.max(1) as f64;
                        }
                        ColumnConstraint::Range { start, end, .. } => {
                            selectivity *= Self::estimate_range_selectivity(
                                start, end, col_stats
                            );
                        }
                    }
                } else {
                    selectivity *= 0.1;
                }
            } else {
                break;
            }
        }
        
        selectivity
    }
}
```

**修改点**:
1. 修改 `IndexSelector::select_best_index` 方法，添加 `table_stats` 参数
2. 在 `OptContext` 中获取统计信息缓存
3. 使用代价模型替代简单评分

### 5.3 索引覆盖扫描优化

**状态**: 待实现

**描述**: 如果查询的所有列都在索引中，直接返回索引数据，避免回表查询。

**实现位置**: `src/query/optimizer/rules/index/index_covering_scan.rs`

**条件**:
- 查询的返回列都是索引字段
- 索引状态为 Active

### 5.4 索引合并优化

**状态**: 待实现

**描述**: 对于多个索引条件的查询，使用索引合并（Index Merge）技术。

**场景**:
```sql
LOOKUP ON tag WHERE field1 == 'a' OR field2 == 'b'
```

**实现**:
- 分别扫描两个索引
- 使用 Union 或 Intersect 合并结果

## 六、实施优先级

| 改进项 | 优先级 | 难度 | 预期收益 |
|--------|--------|------|----------|
| PushTopNDownIndexScanRule | 高 | 中 | 减少数据扫描量 |
| 基于统计信息的索引选择 | 高 | 高 | 更优的索引选择 |
| 索引覆盖扫描优化 | 中 | 中 | 避免回表查询 |
| 索引合并优化 | 低 | 高 | 支持复杂条件查询 |

## 七、参考文档

- [nebula-graph 优化器文档](https://github.com/vesoft-inc/nebula-graph)
- [MySQL 索引选择优化](https://dev.mysql.com/doc/refman/8.0/en/index-hints.html)
- [PostgreSQL 查询优化](https://www.postgresql.org/docs/current/planner-optimizer.html)

## 八、相关文件

### 核心文件
- `src/query/planner/plan/algorithms/index_scan.rs` - IndexScan 计划节点
- `src/query/executor/search_executors.rs` - IndexScanExecutor 执行器
- `src/query/planner/statements/lookup_planner.rs` - LOOKUP 规划器
- `src/query/optimizer/index_selector.rs` - 索引选择器

### 优化规则
- `src/query/optimizer/rules/scan/index_full_scan.rs`
- `src/query/optimizer/rules/limit_pushdown/push_limit_down_index_scan.rs`
- `src/query/optimizer/rules/index/optimize_edge_index_scan_by_filter.rs`

### 存储层
- `src/storage/storage_client.rs` - 存储客户端接口
- `src/storage/index/index_data_manager.rs` - 索引数据管理
- `src/storage/metadata/index_metadata_manager.rs` - 索引元数据管理

### 上下文
- `src/query/context/ast/base.rs` - AstContext
- `src/query/context/execution/query_execution.rs` - QueryContext
- `src/query/optimizer/plan/context.rs` - OptContext
