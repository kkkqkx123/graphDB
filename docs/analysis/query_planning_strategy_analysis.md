# 查询规划和执行策略功能实现分析

**文档版本**: 1.0  
**创建日期**: 2026 年 3 月 3 日  
**分析对象**: graphDB 项目查询规划和执行策略  
**分析目的**: 评估查询规划和执行策略的功能实现完整性

---

## 1. 执行摘要

经过详细分析，查询规划和执行策略的实现情况如下：

| 功能 | 实现状态 | 完成度 | 需要补充 |
|------|----------|---------|----------|
| **通配符标签匹配** | ⚠️ 部分实现 | 60% | any_label 标志、全标签扫描 |
| **通配符边类型匹配** | ❌ 未实现 | 0% | 获取所有边类型、通配符处理 |
| **路径收集 (RollUpApply)** | ⚠️ 部分实现 | 40% | 路径构建逻辑、Path 类型支持 |
| **索引选择优化** | ⚠️ 基础实现 | 30% | 选择性估计、字段数优化 |
| **OR 条件索引嵌入** | ❌ 未实现 | 0% | OR 条件提取、IN 转换 |

**核心结论**:
- 🔴 **高优先级缺失**: 通配符边类型匹配、OR 条件索引嵌入
- 🟡 **中优先级优化**: 通配符标签匹配、索引选择优化
- 🟢 **已有基础**: 路径收集已有 RollUpApply 执行器，需要扩展

---

## 2. 通配符标签匹配分析

### 2.1 当前实现

**文件**: `src/query/planner/statements/seeks/scan_seek.rs`

```rust
impl ScanSeek {
    fn vertex_matches_pattern(&self, vertex: &Vertex, pattern: &NodePattern) -> bool {
        if !pattern.labels.is_empty() {
            let has_all_labels = pattern
                .labels
                .iter()
                .all(|label| vertex.tags.iter().any(|tag| tag.name == *label));
            if !has_all_labels {
                return false;
            }
        }
        // 当 labels 为空时，直接跳过标签检查
        // 问题：没有设置通配符标志，也没有扫描所有标签的逻辑

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

### 2.2 问题分析

#### 🔴 问题 1: 缺少通配符标志

**当前行为**: 当 `pattern.labels.is_empty()` 时，跳过标签检查

**期望行为**: 
- 应该设置 `any_label` 标志
- 扫描所有标签的顶点
- 使用 OR 逻辑而不是 AND 逻辑

**影响**: 无法正确处理 `MATCH (n) RETURN n` 查询

#### 🔴 问题 2: 缺少全标签扫描逻辑

**当前行为**: `scan_vertices` 只扫描指定标签的顶点

**期望行为**: 
- 当 `any_label` 为 true 时，扫描所有标签
- 获取空间中所有标签列表
- 遍历所有标签进行扫描

**影响**: 无法扫描无标签或有任意标签的顶点

### 2.3 需要补充的功能

#### 2.3.1 添加通配符标志

```rust
// src/query/planner/statements/seeks/scan_seek.rs

#[derive(Debug, Clone)]
pub struct ScanSeek {
    any_label: bool,  // 新增：通配符标志
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

#### 2.3.2 修改 SeekStrategySelector

```rust
// src/query/planner/statements/seeks/seek_strategy_base.rs

impl SeekStrategySelector {
    pub fn select_strategy<S: StorageClient + ?Sized>(
        &self,
        _storage: &S,
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

#### 2.3.3 修改 MatchStatementPlanner

```rust
// src/query/planner/statements/match_statement_planner.rs

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

### 2.4 实现优先级

🔴 **高优先级** - 影响基础查询功能

---

## 3. 通配符边类型匹配分析

### 3.1 当前实现

**文件**: `src/query/planner/statements/match_statement_planner.rs`

```rust
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

        // 创建边扩展节点
        let expand_node = ExpandAllNode::new(space_id, edge.edge_types.clone(), direction);

        let mut plan = SubPlan::from_root(expand_node.into_enum());

        // ... 其余逻辑
        Ok(plan)
    }
}
```

### 3.2 问题分析

#### 🔴 问题 1: 未处理空边类型

**当前行为**: 当 `edge.edge_types` 为 `None` 或空时，直接传递空列表

**期望行为**: 
- 获取空间中所有边类型
- 使用所有边类型进行扩展

**影响**: 无法正确处理 `MATCH (a)-[]->(b) RETURN a,b` 查询

#### 🔴 问题 2: 缺少通配符标志

**当前行为**: `ExpandAllNode` 没有通配符标志

**期望行为**: 
- 添加 `any_edge_type` 标志
- 在执行器中根据标志决定是否扫描所有边类型

**影响**: 无法区分指定边类型和任意边类型

### 3.3 需要补充的功能

#### 3.3.1 添加获取所有边类型的方法

```rust
// src/query/planner/statements/match_statement_planner.rs

impl MatchStatementPlanner {
    fn get_all_edge_types(&self, space_id: u64) -> Result<Vec<String>, PlannerError> {
        // 从存储层获取所有边类型
        // 这需要在 StorageClient trait 中添加相应方法
        Ok(vec![])  // 临时实现
    }

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
}
```

#### 3.3.2 修改 ExpandAllNode

```rust
// src/query/planner/plan/core/nodes/traversal_node.rs

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

### 3.4 实现优先级

🔴 **高优先级** - 影响基础查询功能

---

## 4. 路径收集 (RollUpApply) 分析

### 4.1 当前实现

**文件**: `src/query/executor/result_processing/transformations/rollup_apply.rs`

```rust
pub struct RollUpApplyExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    left_input_var: String,
    right_input_var: String,
    compare_cols: Vec<Expression>,
    collect_col: Expression,
    col_names: Vec<String>,
    movable: bool,
}
```

### 4.2 问题分析

#### 🟡 问题 1: 缺少路径构建逻辑

**当前行为**: 通用的聚合执行器，可以收集值到列表

**期望行为**: 
- 支持路径变量（如 `p = (a)-[:KNOWS]->(b)`）
- 构建路径对象（包含顶点和边）
- 返回路径类型的结果

**影响**: 无法处理 `MATCH p = (a)-[:KNOWS]->(b) RETURN p` 查询

#### 🟡 问题 2: 缺少 Path 类型支持

**当前行为**: 只支持值列表的收集

**期望行为**: 
- 支持 `Path` 类型
- 支持路径的序列化和反序列化
- 支持路径的显示和比较

**影响**: 无法返回路径类型的结果

### 4.3 需要补充的功能

#### 4.3.1 添加 Path 类型

```rust
// src/core/types/path.rs

use crate::core::{Edge, Vertex};

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
}

impl Path {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn with_vertices(mut self, vertices: Vec<Vertex>) -> Self {
        self.vertices = vertices;
        self
    }

    pub fn with_edges(mut self, edges: Vec<Edge>) -> Self {
        self.edges = edges;
        self
    }

    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.vertices.push(vertex);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn length(&self) -> usize {
        self.edges.len()
    }

    pub fn start_node(&self) -> Option<&Vertex> {
        self.vertices.first()
    }

    pub fn end_node(&self) -> Option<&Vertex> {
        self.vertices.last()
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}
```

#### 4.3.2 扩展 RollUpApplyExecutor

```rust
// src/query/executor/result_processing/transformations/rollup_apply.rs

pub struct RollUpApplyExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    left_input_var: String,
    right_input_var: String,
    compare_cols: Vec<Expression>,
    collect_col: Expression,
    col_names: Vec<String>,
    movable: bool,
    // 新增：路径收集模式
    path_mode: bool,
}

impl<S: StorageClient + Send + 'static> RollUpApplyExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_input_var: String,
        right_input_var: String,
        compare_cols: Vec<Expression>,
        collect_col: Expression,
        col_names: Vec<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "RollUpApplyExecutor".to_string(), storage),
            left_input_var,
            right_input_var,
            compare_cols,
            collect_col,
            col_names,
            movable: false,
            path_mode: false,
        }
    }

    pub fn with_path_mode(mut self, path_mode: bool) -> Self {
        self.path_mode = path_mode;
        self
    }

    fn execute_path_rollup_apply(&mut self) -> DBResult<DataSet> {
        // 获取左右输入
        let left_result = self
            .base
            .context
            .get_result(&self.left_input_var)
            .expect("Context should have left result");
        let right_result = self
            .base
            .context
            .get_result(&self.right_input_var)
            .expect("Context should have right result");

        let left_values = match left_result {
            ExecutionResult::Values(values) => values.clone(),
            _ => {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Invalid left input result type".to_string(),
                    ),
                ))
            }
        };

        let right_values = match right_result {
            ExecutionResult::Values(values) => values.clone(),
            _ => {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Invalid right input result type".to_string(),
                    ),
                ))
            }
        };

        // 按 key 分组收集右侧数据
        let mut grouped: std::collections::HashMap<Vec<Value>, Vec<Value>> = 
            std::collections::HashMap::new();

        let mut expr_context = DefaultExpressionContext::new();

        for value in &right_values {
            expr_context.set_variable("_".to_string(), value.clone());

            let mut key_list = Vec::new();
            for col in &self.compare_cols {
                let val = ExpressionEvaluator::evaluate(col, &expr_context).map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(
                        e.to_string(),
                    ))
                })?;
                key_list.push(val);
            }

            let collect_val =
                ExpressionEvaluator::evaluate(&self.collect_col, &expr_context).map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(
                        e.to_string(),
                    ))
                })?;

            let entry = grouped.entry(key_list).or_insert_with(Vec::new);
            entry.push(collect_val);
        }

        // 合并左右数据，构建路径
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        for value in &left_values {
            expr_context.set_variable("_".to_string(), value.clone());

            let mut key_list = Vec::new();
            for col in &self.compare_cols {
                let val = ExpressionEvaluator::evaluate(col, &expr_context).map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(
                        e.to_string(),
                    ))
                })?;
                key_list.push(val);
            }

            let collected = grouped.get(&key_list).cloned().unwrap_or_default();

            let mut row = Vec::new();
            if self.movable {
                row.push(value.clone());
            }

            if self.path_mode {
                // 构建路径对象
                let path = self.build_path(&collected)?;
                row.push(Value::Path(Box::new(path)));
            } else {
                // 收集为列表
                row.push(Value::List(List::from(collected)));
            }

            dataset.rows.push(row);
        }

        Ok(dataset)
    }

    fn build_path(&self, values: &[Value]) -> DBResult<Path> {
        let mut path = Path::new();

        for value in values {
            match value {
                Value::Vertex(v) => path.add_vertex(v.as_ref().clone()),
                Value::Edge(e) => path.add_edge(e.clone()),
                _ => {
                    return Err(DBError::Query(
                        crate::core::error::QueryError::ExecutionError(
                            format!("Invalid path element: {:?}", value),
                        ),
                    ))
                }
            }
        }

        Ok(path)
    }
}
```

### 4.4 实现优先级

🟡 **中优先级** - 已有基础实现，需要扩展

---

## 5. 索引选择优化分析

### 5.1 当前实现

**文件**: `src/query/planner/statements/seeks/seek_strategy_base.rs`

```rust
impl SeekStrategySelector {
    pub fn select_strategy<S: StorageClient + ?Sized>(
        &self,
        _storage: &S,
        context: &SeekStrategyContext,
    ) -> SeekStrategyType {
        if context.has_explicit_vid() {
            SeekStrategyType::VertexSeek
        } else if context.has_property_predicates() && context.has_index_for_properties() {
            SeekStrategyType::PropIndexSeek
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

### 5.2 问题分析

#### 🟡 问题 1: 索引选择过于简化

**当前行为**: 只基于阈值选择索引

**期望行为**: 
- 考虑索引的选择性
- 考虑索引的字段数
- 选择最优索引

**影响**: 可能选择次优索引，影响查询性能

#### 🟡 问题 2: 缺少选择性估计

**当前行为**: 没有选择性估计

**期望行为**: 
- 基于统计信息估计选择性
- 选择高选择性的索引
- 优化查询计划

**影响**: 无法选择最优索引

### 5.3 需要补充的功能

#### 5.3.1 添加选择性估计

```rust
// src/query/planner/statements/seeks/seek_strategy_base.rs

#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub name: String,
    pub target_type: String,
    pub target_name: String,
    pub properties: Vec<String>,
    // 新增字段
    pub selectivity: f32,      // 选择性估计 (0.0-1.0)
    pub field_count: usize,    // 索引字段数
}

impl IndexInfo {
    pub fn new(
        name: String,
        target_type: String,
        target_name: String,
        properties: Vec<String>,
    ) -> Self {
        let field_count = properties.len();
        Self {
            name,
            target_type,
            target_name,
            properties,
            selectivity: 0.5,  // 默认选择性
            field_count,
        }
    }

    pub fn with_selectivity(mut self, selectivity: f32) -> Self {
        self.selectivity = selectivity;
        self
    }
}
```

#### 5.3.2 优化索引选择

```rust
// src/query/planner/statements/seeks/seek_strategy_base.rs

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
                        // 这里可以使用 Visitor 来简化代码
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

### 5.4 实现优先级

🟡 **中优先级** - 影响查询性能，但不影响功能

---

## 6. OR 条件索引嵌入分析

### 6.1 当前实现

**文件**: `src/query/planner/statements/seeks/prop_index_seek.rs`

```rust
impl PropIndexSeek {
    /// 从表达式列表提取属性谓词
    pub fn extract_predicates(expressions: &[crate::core::Expression]) -> Vec<PropertyPredicate> {
        let mut predicates = Vec::new();

        for expr in expressions {
            if let Some(pred) = Self::extract_predicate(expr) {
                predicates.push(pred);
            }
        }

        predicates
    }

    /// 从单个表达式提取属性谓词
    fn extract_predicate(expr: &crate::core::Expression) -> Option<PropertyPredicate> {
        use crate::core::types::operators::BinaryOperator;

        match expr {
            crate::core::Expression::Binary { op, left, right } => {
                // 只处理简单的二元表达式
                // 没有处理 OR 条件的逻辑
                // ...
            }
            _ => None,
        }
    }
}
```

### 6.2 问题分析

#### 🔴 问题 1: 未处理 OR 条件

**当前行为**: 只处理简单的二元表达式

**期望行为**: 
- 识别 OR 条件
- 检查是否可以转换为 IN 条件
- 转换 `WHERE n.age = 10 OR n.age = 20` 为 `WHERE n.age IN [10, 20]`

**影响**: 无法利用索引处理 OR 条件

#### 🔴 问题 2: 缺少 OR 条件提取逻辑

**当前行为**: 没有专门的 OR 条件提取

**期望行为**: 
- 提取 OR 条件中的所有子条件
- 检查是否为同一属性的等值比较
- 转换为 IN 谓词

**影响**: 无法优化 OR 条件查询

### 6.3 需要补充的功能

#### 6.3.1 添加 OR 条件提取

```rust
// src/query/planner/statements/seeks/prop_index_seek.rs

impl PropIndexSeek {
    /// 从表达式列表提取属性谓词
    pub fn extract_predicates(expressions: &[crate::core::Expression]) -> Vec<PropertyPredicate> {
        let mut predicates = Vec::new();

        for expr in expressions {
            // 检查是否为 OR 条件
            if let Some(or_predicates) = Self::extract_or_predicates(expr) {
                predicates.extend(or_predicates);
            } else if let Some(pred) = Self::extract_predicate(expr) {
                predicates.push(pred);
            }
        }

        predicates
    }

    /// 提取 OR 条件
    fn extract_or_predicates(expr: &crate::core::Expression) -> Option<Vec<PropertyPredicate>> {
        use crate::core::types::operators::BinaryOperator;
        use crate::core::types::operators::LogicalOperator;

        match expr {
            crate::core::Expression::Logical { operator, operands } => {
                if *operator == LogicalOperator::Or {
                    // 处理 OR 条件
                    Self::extract_or_conditions(operands)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// 提取 OR 条件中的所有子条件
    fn extract_or_conditions(operands: &[crate::core::Expression]) -> Option<Vec<PropertyPredicate>> {
        let mut property_name: Option<String> = None;
        let mut values = Vec::new();

        for operand in operands {
            if let Some(pred) = Self::extract_predicate(operand) {
                // 检查是否为等值操作
                if !pred.op.is_equality() {
                    return None;  // 只支持等值 OR
                }

                if let Some(ref existing_prop) = property_name {
                    if *existing_prop != pred.property {
                        return None;  // 必须是同一属性
                    }
                } else {
                    property_name = Some(pred.property.clone());
                }
                values.push(pred.value);
            } else {
                return None;
            }
        }

        // 转换为 IN 谓词
        if let Some(prop) = property_name {
            Some(vec![PropertyPredicate {
                property: prop,
                op: PredicateOp::In,
                value: crate::core::Value::List(crate::core::value::dataset::List::from(values)),
            }])
        } else {
            None
        }
    }
}
```

### 6.4 实现优先级

🔴 **高优先级** - 影响查询性能和功能

---

## 7. 实现建议

### 7.1 优先级排序

| 优先级 | 功能 | 预计工时 | 依赖 |
|--------|------|----------|------|
| 🔴 P0 | 通配符边类型匹配 | 2 天 | 无 |
| 🔴 P0 | OR 条件索引嵌入 | 3 天 | 无 |
| 🟡 P1 | 通配符标签匹配 | 2 天 | 无 |
| 🟡 P1 | 索引选择优化 | 2 天 | 无 |
| 🟢 P2 | 路径收集扩展 | 3 天 | 无 |

### 7.2 实现顺序

**阶段 1**: P0 功能（5 天）
1. 通配符边类型匹配
2. OR 条件索引嵌入

**阶段 2**: P1 功能（4 天）
1. 通配符标签匹配
2. 索引选择优化

**阶段 3**: P2 功能（3 天）
1. 路径收集扩展

### 7.3 与 Visitor 体系的集成

根据 [pattern_visitor_integration_analysis.md](file:///d:/项目/database/graphDB/docs/analysis/pattern_visitor_integration_analysis.md) 的分析：

**应该集成**:
- ✅ OR 条件提取 - 创建 `OrConditionCollector`
- ✅ 属性谓词提取 - 创建 `PropertyPredicateCollector`

**不应集成**:
- ❌ 通配符标签匹配 - 查询规划策略
- ❌ 通配符边类型匹配 - 查询规划策略
- ❌ 路径收集 - 执行器功能

**部分集成**:
- ⚠️ 索引选择优化 - 使用现有 `PropertyContainsChecker` 辅助

---

## 8. 总结

### 8.1 功能完成度

| 类别 | 数量 | 完成度 |
|------|------|--------|
| **查询规划策略** | 3 项 | ⚠️ 40% |
| **执行器功能** | 1 项 | ⚠️ 40% |
| **索引优化** | 2 项 | ⚠️ 15% |

### 8.2 整体评价

- **基础功能存在**: 已有基本的查询规划和执行框架
- **通配符支持不足**: 缺少通配符标签和边类型匹配
- **优化空间较大**: 索引选择和 OR 条件处理需要增强

### 8.3 建议行动

1. **立即补充**: 通配符边类型匹配、OR 条件索引嵌入
2. **近期补充**: 通配符标签匹配、索引选择优化
3. **中期扩展**: 路径收集功能

---

## 附录A: 需要修改的文件清单

### 查询规划层

- `src/query/planner/statements/seeks/scan_seek.rs` - 添加通配符标签支持
- `src/query/planner/statements/seeks/seek_strategy_base.rs` - 优化索引选择
- `src/query/planner/statements/seeks/prop_index_seek.rs` - 添加 OR 条件提取
- `src/query/planner/statements/match_statement_planner.rs` - 添加通配符边类型支持

### 执行器层

- `src/query/executor/result_processing/transformations/rollup_apply.rs` - 扩展路径收集功能

### 核心类型

- `src/core/types/path.rs` - 添加 Path 类型（新建）
- `src/core/types/mod.rs` - 导出 Path 类型

### 存储层

- `src/storage/storage_client.rs` - 添加 `list_tags` 和 `list_edge_types` 方法
