# Pattern 匹配功能补充计划

**文档版本**: 1.0  
**创建日期**: 2026 年 3 月 3 日  
**基于文档**: `pattern_matching_analysis.md`

---

## 1. 概述

本文档详细说明 graphDB 项目需要补充的通配符匹配和条件匹配功能，包含具体实现方案和代码示例。

---

## 2. 高优先级功能

### 2.1 通配符标签匹配 (`anyLabel`)

#### 2.1.1 问题描述

当前无法正确处理 `MATCH (n) RETURN n` 查询（无标签扫描）。

**当前代码问题** (`match_statement_planner.rs`):
```rust
// 问题：没有处理 labels.is_empty() 的情况
if !node.labels.is_empty() {
    let label_filter = Self::build_label_filter_expression(...);
    // ...
}
// 当 labels 为空时，直接跳过标签过滤，但没有设置全量扫描标志
```

#### 2.1.2 实现方案

**步骤 1**: 修改 `NodePattern` 结构

```rust
// query/parser/ast/pattern.rs
#[derive(Debug, Clone, PartialEq)]
pub struct NodePattern {
    pub span: Span,
    pub variable: Option<String>,
    pub labels: Vec<String>,
    pub properties: Option<ContextualExpression>,
    pub predicates: Vec<ContextualExpression>,
    // 新增：通配符标志
    pub is_wildcard: bool,  // true 表示匹配所有标签
}
```

**步骤 2**: 修改 `ScanSeek` 支持通配符

```rust
// query/planner/statements/seeks/scan_seek.rs
use crate::core::types::SpaceInfo;

#[derive(Debug, Clone)]
pub struct ScanSeek {
    any_label: bool,  // 新增字段
}

impl ScanSeek {
    pub fn new() -> Self {
        Self { any_label: false }
    }

    pub fn with_any_label(mut self, any_label: bool) -> Self {
        self.any_label = any_label;
        self
    }
}

impl SeekStrategy for ScanSeek {
    fn execute<S: StorageClient>(
        &self,
        storage: &S,
        context: &SeekStrategyContext,
    ) -> Result<SeekResult, StorageError> {
        if self.any_label {
            // 扫描所有标签的顶点
            self.scan_all_labels(storage, context)
        } else {
            // 当前逻辑
            self.scan_specific_labels(storage, context)
        }
    }

    fn supports(&self, _context: &SeekStrategyContext) -> bool {
        true  // ScanSeek 始终可用
    }
}

impl ScanSeek {
    fn scan_all_labels<S: StorageClient>(
        &self,
        storage: &S,
        context: &SeekStrategyContext,
    ) -> Result<SeekResult, StorageError> {
        // 获取空间中所有标签
        let all_tags = storage.list_tags("default")?;  // 需要添加此方法
        
        let mut vertex_ids = Vec::new();
        let mut rows_scanned = 0;

        // 扫描所有标签的顶点
        for tag in all_tags {
            let vertices = storage.scan_vertices_by_tag("default", &tag.name)?;
            for vertex in vertices {
                rows_scanned += 1;
                if self.vertex_matches_pattern(&vertex, &context.node_pattern, true) {
                    vertex_ids.push(vertex.vid().clone());
                }
            }
        }

        Ok(SeekResult {
            vertex_ids,
            strategy_used: SeekStrategyType::ScanSeek,
            rows_scanned,
        })
    }

    fn vertex_matches_pattern(
        &self, 
        vertex: &Vertex, 
        pattern: &NodePattern,
        any_label: bool,
    ) -> bool {
        if !pattern.labels.is_empty() {
            let has_all_labels = pattern
                .labels
                .iter()
                .all(|label| vertex.tags.iter().any(|tag| tag.name == *label));
            if !has_all_labels {
                return false;
            }
        } else if !any_label {
            // 非通配符模式下，空标签表示必须至少有一个标签
            if vertex.tags.is_empty() {
                return false;
            }
        }
        // 通配符模式下，any_label=true，不检查标签

        for (prop_name, prop_value) in &pattern.properties {
            let found = vertex
                .get_all_properties()
                .iter()
                .any(|(name, value)| name == prop_name && **value == *prop_value);
            if !found {
                return false;
            }
        }

        true
    }
}
```

**步骤 3**: 修改 `SeekStrategySelector`

```rust
// query/planner/statements/seeks/seek_strategy_base.rs
impl SeekStrategySelector {
    pub fn select_strategy<S: StorageClient + ?Sized>(
        &self,
        storage: &S,
        context: &SeekStrategyContext,
    ) -> SeekStrategyType {
        if context.has_explicit_vid() {
            SeekStrategyType::VertexSeek
        } else if context.has_property_predicates() && context.has_index_for_properties() {
            SeekStrategyType::PropIndexSeek
        } else if context.node_pattern.labels.is_empty() {
            // 空标签 = 通配符匹配
            SeekStrategyType::ScanSeek
        } else if let Some(_) = context.get_index_for_labels(&context.node_pattern.labels) {
            if context.estimated_rows < self.scan_threshold {
                SeekStrategyType::IndexSeek
            } else {
                SeekStrategyType::ScanSeek
            }
        } else {
            SeekStrategyType::ScanSeek
        }
    }
}
```

**步骤 4**: 修改 `MatchStatementPlanner`

```rust
// query/planner/statements/match_statement_planner.rs
impl MatchStatementPlanner {
    fn plan_pattern_node(
        &self,
        node: &crate::query::parser::ast::pattern::NodePattern,
        space_id: u64,
        qctx: &Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        // 判断是否为通配符匹配
        let is_wildcard = node.labels.is_empty();

        // 创建节点扫描
        let mut scan_node = ScanVerticesNode::new(space_id);
        
        // 设置通配符标志
        if is_wildcard {
            scan_node.set_any_label(true);  // 需要添加此方法
        }

        let mut plan = SubPlan::from_root(scan_node.into_enum());

        // 如果有标签过滤，添加过滤器
        if !node.labels.is_empty() {
            let label_filter = Self::build_label_filter_expression(&node.variable, &node.labels, qctx);
            let filter_node = FilterNode::new(
                plan.root.as_ref().expect("plan 的 root 应该存在").clone(),
                label_filter,
            )
            .map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
            plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
        }

        // ... 其余逻辑不变
        Ok(plan)
    }
}
```

#### 2.1.3 测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_label_match() {
        // MATCH (n) RETURN n
        let pattern = NodePattern {
            variable: Some("n".to_string()),
            labels: vec![],  // 空标签
            properties: None,
            predicates: vec![],
            span: Span::default(),
            is_wildcard: true,
        };

        let context = SeekStrategyContext::new(
            1,
            NodePattern {
                vid: None,
                labels: vec![],
                properties: vec![],
            },
            vec![],
        );

        let selector = SeekStrategySelector::new();
        let strategy = selector.select_strategy(&DummyStorage, &context);
        
        // 应该选择 ScanSeek
        assert_eq!(strategy, SeekStrategyType::ScanSeek);
    }
}
```

---

### 2.2 通配符边类型匹配

#### 2.2.1 问题描述

当前无法处理 `MATCH (a)-[]->(b) RETURN a,b`（任意边类型）。

#### 2.2.2 实现方案

**步骤 1**: 修改 `EdgePattern` 处理逻辑

```rust
// query/planner/statements/match_statement_planner.rs
impl MatchStatementPlanner {
    fn plan_pattern_edge(
        &self,
        edge: &crate::query::parser::ast::pattern::EdgePattern,
        space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        // 确定边方向
        let direction = match edge.direction {
            crate::query::parser::ast::types::EdgeDirection::Out => "out",
            crate::query::parser::ast::types::EdgeDirection::In => "in",
            crate::query::parser::ast::types::EdgeDirection::Both => "both",
        };

        // 处理边类型：None 表示所有边类型
        let edge_types = match &edge.edge_types {
            Some(types) if !types.is_empty() => types.clone(),
            _ => {
                // 获取空间中所有边类型
                self.get_all_edge_types(space_id)?
            }
        };

        // 创建边扩展节点
        let mut expand_node = ExpandAllNode::new(space_id, edge_types, direction);

        // 设置通配符标志
        if edge.edge_types.is_none() || edge.edge_types.as_ref().map(|v| v.is_empty()).unwrap_or(true) {
            expand_node.set_any_edge_type(true);  // 需要添加此方法
        }

        let mut plan = SubPlan::from_root(expand_node.into_enum());

        // ... 其余逻辑不变
        Ok(plan)
    }

    fn get_all_edge_types(&self, space_id: u64) -> Result<Vec<String>, PlannerError> {
        // 从存储层获取所有边类型
        // 这需要在 StorageClient trait 中添加相应方法
        Ok(vec![])  // 临时实现
    }
}
```

**步骤 2**: 修改 `ExpandAllNode`

```rust
// query/planner/plan/core/nodes/traversal_node.rs
define_plan_node! {
    pub struct ExpandAllNode {
        space_id: u64,
        edge_types: Vec<String>,
        direction: String,
        any_edge_type: bool,  // 新增：通配符标志
        // ... 其他字段
    }
}

impl ExpandAllNode {
    pub fn set_any_edge_type(&mut self, any: bool) {
        self.any_edge_type = any;
    }

    pub fn any_edge_type(&self) -> bool {
        self.any_edge_type
    }
}
```

---

### 2.3 路径收集 (RollUpApply)

#### 2.3.1 问题描述

当前无法处理路径变量返回：
```cypher
MATCH p = (a)-[:KNOWS]->(b)
RETURN p
```

#### 2.3.2 实现方案

**步骤 1**: 创建 `RollUpApply` 执行器

```rust
// query/executor/result_processing/transformations/rollup_apply.rs
use parking_lot::Mutex;
use std::sync::Arc;

use crate::core::error::{DBError, DBResult};
use crate::core::{DataSet, Edge, List, Value, Vertex};
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, ExecutionContext};
use crate::storage::StorageClient;

pub struct RollUpApplyExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    left_input_var: String,
    right_input_var: String,
    key_cols: Vec<crate::core::Expression>,
    collect_col: String,  // 收集列名
    col_names: Vec<String>,
}

impl<S: StorageClient + Send + 'static> RollUpApplyExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_input_var: String,
        right_input_var: String,
        key_cols: Vec<crate::core::Expression>,
        collect_col: String,
        col_names: Vec<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "RollUpApplyExecutor".to_string(), storage),
            left_input_var,
            right_input_var,
            key_cols,
            collect_col,
            col_names,
        }
    }

    fn execute_rollup_apply(&mut self) -> DBResult<DataSet> {
        // 获取左右输入
        let left_result = self.base.context.get_result(&self.left_input_var)
            .ok_or_else(|| DBError::Query(crate::core::error::QueryError::ExecutionError(
                format!("Left input variable '{}' not found", self.left_input_var)
            )))?;

        let right_result = self.base.context.get_result(&self.right_input_var)
            .ok_or_else(|| DBError::Query(crate::core::error::QueryError::ExecutionError(
                format!("Right input variable '{}' not found", self.right_input_var)
            )))?;

        // 按 key 分组收集右侧数据
        let mut grouped: std::collections::HashMap<Vec<Value>, Vec<Value>> = 
            std::collections::HashMap::new();

        // 处理右侧数据，按 key 分组
        // ...

        // 合并左右数据
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        // ...

        Ok(dataset)
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for RollUpApplyExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let dataset = self.execute_rollup_apply()?;
        // 转换为 ExecutionResult
        // ...
    }

    // ... 其他方法
}
```

**步骤 2**: 修改 `Pattern` AST 支持路径类型

```rust
// query/parser/ast/pattern.rs
#[derive(Debug, Clone, PartialEq)]
pub struct PathPattern {
    pub span: Span,
    pub elements: Vec<PathElement>,
    // 新增字段
    pub is_path_variable: bool,  // 是否为路径变量 (p = ...)
    pub path_variable_name: Option<String>,  // 路径变量名
}
```

**步骤 3**: 修改规划器判断

```rust
// query/planner/statements/match_statement_planner.rs
impl MatchStatementPlanner {
    fn plan_path_pattern(
        &self,
        pattern: &Pattern,
        space_id: u64,
        sym_table: &SymbolTable,
        validation_info: Option<&ValidationInfo>,
        qctx: &Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        match pattern {
            Pattern::Path(path) => {
                // 判断是否为路径谓词或路径变量
                if path.is_path_variable {
                    // 使用 RollUpApply
                    self.plan_rollup_apply(path, space_id, sym_table, validation_info, qctx)
                } else if path.is_pred {
                    // 使用 PatternApply (EXISTS/NOT EXISTS)
                    self.plan_pattern_apply(path, space_id, sym_table, validation_info, qctx)
                } else {
                    // 普通路径展开
                    self.plan_path_expand(path, space_id, sym_table, validation_info, qctx)
                }
            }
            // ...
        }
    }
}
```

---

## 3. 中优先级功能

### 3.1 索引选择优化

#### 3.1.1 问题描述

当前索引选择仅基于阈值，未考虑实际索引的选择性和字段数。

#### 3.1.2 实现方案

```rust
// query/planner/statements/seeks/seek_strategy_base.rs
#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub target_type: String,  // "tag" or "edge"
    pub target_name: String,
    pub properties: Vec<String>,
    // 新增字段
    pub selectivity: f32,      // 选择性估计 (0.0-1.0)
    pub field_count: usize,    // 索引字段数
}

impl SeekStrategySelector {
    pub fn select_best_index(&self, indexes: &[IndexInfo], predicates: &[Expression]) -> Option<&IndexInfo> {
        if indexes.is_empty() {
            return None;
        }

        // 过滤出能匹配谓词的索引
        let candidate_indexes: Vec<&IndexInfo> = indexes
            .iter()
            .filter(|idx| {
                // 检查索引属性是否覆盖谓词属性
                predicates.iter().any(|pred| {
                    if let Expression::Binary { left, right, .. } = pred {
                        // 提取属性名并检查
                        // ...
                        true
                    } else {
                        false
                    }
                })
            })
            .collect();

        if candidate_indexes.is_empty() {
            return indexes.iter().min_by_key(|idx| idx.field_count);
        }

        // 选择最优索引：字段数少且选择性高
        candidate_indexes
            .into_iter()
            .min_by(|a, b| {
                // 先比较字段数
                let field_cmp = a.field_count.cmp(&b.field_count);
                if field_cmp != std::cmp::Ordering::Equal {
                    return field_cmp;
                }
                // 再比较选择性 (越高越好)
                b.selectivity.partial_cmp(&a.selectivity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}
```

---

### 3.2 OR 条件索引嵌入

#### 3.2.1 问题描述

无法利用索引处理 `WHERE n.age = 10 OR n.age = 20`。

#### 3.2.2 实现方案

```rust
// query/planner/statements/seeks/prop_index_seek.rs
impl PropIndexSeek {
    pub fn extract_predicates(predicates: &[Expression]) -> Vec<PropertyPredicate> {
        let mut result = Vec::new();

        for pred in predicates {
            match pred {
                Expression::Binary { operator, left, right } => {
                    if matches!(operator, BinaryOperator::Eq | BinaryOperator::Lt | ...) {
                        // 提取单个条件
                        result.push(PropertyPredicate {
                            property: Self::extract_property_name(left)?,
                            operator: operator.clone(),
                            value: Self::extract_value(right)?,
                        });
                    }
                }
                Expression::Logical { operator, operands } => {
                    if *operator == LogicalOperator::Or {
                        // 处理 OR 条件
                        if let Some(or_predicates) = Self::extract_or_predicates(operands) {
                            result.extend(or_predicates);
                        }
                    } else if *operator == LogicalOperator::And {
                        // 处理 AND 条件
                        result.extend(Self::extract_predicates(operands));
                    }
                }
                _ => {}
            }
        }

        result
    }

    fn extract_or_predicates(operands: &[Expression]) -> Option<Vec<PropertyPredicate>> {
        // 检查所有 OR 条件是否为同一属性的等值比较
        let mut property_name: Option<String> = None;
        let mut values = Vec::new();

        for operand in operands {
            if let Expression::Binary { operator, left, right } = operand {
                if *operator != BinaryOperator::Eq {
                    return None;  // 只支持等值 OR
                }

                let prop = Self::extract_property_name(left)?;
                let val = Self::extract_value(right)?;

                if let Some(ref existing_prop) = property_name {
                    if *existing_prop != prop {
                        return None;  // 必须是同一属性
                    }
                } else {
                    property_name = Some(prop);
                }
                values.push(val);
            } else {
                return None;
            }
        }

        // 转换为 IN 谓词
        if let Some(prop) = property_name {
            Some(vec![PropertyPredicate {
                property: prop,
                operator: BinaryOperator::In,
                value: Value::List(List::from(values)),
            }])
        } else {
            None
        }
    }
}
```

---

## 4. 实现时间表

| 阶段 | 功能 | 预计工时 | 依赖 |
|------|------|----------|------|
| **阶段 1** | `anyLabel` 通配符 | 3 天 | 无 |
| **阶段 2** | 边类型通配符 | 2 天 | 阶段 1 |
| **阶段 3** | RollUpApply | 5 天 | 无 |
| **阶段 4** | 索引选择优化 | 3 天 | 无 |
| **阶段 5** | OR 条件嵌入 | 4 天 | 阶段 4 |

---

## 5. 测试计划

### 5.1 通配符标签测试

```cypher
-- 测试 1: 无标签扫描
MATCH (n) RETURN n

-- 测试 2: 单标签扫描
MATCH (n:Person) RETURN n

-- 测试 3: 多标签扫描
MATCH (n:Person:Actor) RETURN n
```

### 5.2 通配符边测试

```cypher
-- 测试 1: 任意边类型
MATCH (a)-[]->(b) RETURN a, b

-- 测试 2: 指定边类型
MATCH (a)-[:KNOWS]->(b) RETURN a, b

-- 测试 3: 多边形类型
MATCH (a)-[:KNOWS|WORKS_WITH]->(b) RETURN a, b
```

### 5.3 路径收集测试

```cypher
-- 测试 1: 路径变量返回
MATCH p = (a)-[:KNOWS]->(b) RETURN p

-- 测试 2: 多跳路径
MATCH p = (a)-[:KNOWS*1..3]->(b) RETURN p
```

---

## 6. 参考文档

- `docs/plan/pattern_matching_analysis.md` - NebulaGraph Pattern 系统分析
- `nebula-3.8.0/src/graph/planner/match/` - NebulaGraph 匹配规划器实现
- `nebula-3.8.0/src/graph/executor/query/PatternApplyExecutor.*` - PatternApply 执行器
