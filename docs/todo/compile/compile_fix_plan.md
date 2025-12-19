# GraphDB 编译错误修复计划

## 日期: 2025年12月19日

### 1. 修复导入与命名空间问题 (Import & Namespace Issues)

- **文件**: `src\query\planner\ngql\path_planner.rs`
  - **任务**: 移除第 76 行多余的 `use crate::graph::expression::Expression;` 语句。

- **文件**: `src\query\planner\match_planning\*.rs` (多个文件，如 `core\match_clause_planner.rs`, `clauses\projection_planner.rs`, `clauses\return_clause_planner.rs`, `clauses\unwind_planner.rs`, `clauses\yield_planner.rs`)
  - **任务**: 将所有 `crate::query::context::ast::AstContext` 的引用替换为 `crate::query::context::AstContext`。

### 2. 修复类型与 trait 对象问题 (Type & Trait Object Issues)

- **文件**: `src\query\query_pipeline_manager.rs`
  - **任务**: 在第 36 行 `Validator::new` 前添加 `dyn`，变为 `<dyn Validator>::new`。
  - **任务**: 在第 76 行 `crate::query::context::managers::SchemaManager::default()` 前添加 `dyn`，变为 `<dyn crate::query::context::managers::SchemaManager>::default()`。
  - **任务**: 在第 78 行 `crate::query::context::managers::StorageClient::default()` 前添加 `dyn`，变为 `<dyn crate::query::context::managers::StorageClient>::default()`。
  - **任务**: 在第 79 行 `crate::query::context::managers::StorageClient::default()` 前添加 `dyn`，变为 `<dyn crate::query::context::managers::StorageClient>::default()`。

- **文件**: `src\query\executor\result_processing\filter.rs`
  - **任务**: 修改 `execute` 方法的实现，使其返回 `Result<ExecutionResult, DBError>` 类型，而不是 `Result<(), DBError>`。

- **文件**: `src\query\planner\match_planning\clauses\projection_planner.rs`
  - **任务**: 将第 180 行的 `pagination.skip != 0` 修正为 `pagination.skip != Some(0)`。
  - **任务**: 将第 180 行的 `pagination.limit != i64::MAX` 修正为 `pagination.limit.map_or(true, |limit| limit as i64 != i64::MAX)`。

- **文件**: `src\query\planner\match_planning\match_planner.rs`
  - **任务**: 将第 45 行的 `AstContext::new("MATCH", "MATCH (n)")` 修改为 `AstContext::new("MATCH".to_string(), "MATCH (n)".to_string())`。

- **文件**: `src\query\executor\result_processing\aggregation.rs`
  - **任务**: 在第 439 行的 `impl<S: StorageEngine>` 后添加 `+ 'static`，变为 `impl<S: StorageEngine + 'static>`。

### 3. 修复字段与方法缺失/错误 (Missing or Incorrect Fields/Methods)

- **文件**: `src\query\planner\match_planning\clauses\*.rs` (包括 `return_clause_planner.rs`, `yield_planner.rs`, `with_clause_planner.rs`)
  - **任务**: 将所有对 `YieldClauseContext.yield_columns` 的引用替换为 `YieldClauseContext.columns`。
  - **任务**: 将所有对 `YieldClauseContext.proj_output_column_names` 的引用替换为 `YieldClauseContext.columns.map(|c| c.name.clone())` 或类似逻辑（需确认确切字段）。
  - **任务**: 将对 `ReturnClauseContext.order_by`, `pagination`, `distinct` 的引用调整为通过 `ReturnClauseContext.yield_clause` 访问相关字段。

- **文件**: `src\query\planner\match_planning\utils\finder.rs`
  - **任务**: 将对 `WhereClauseContext.aliases_available`, `paths`, `WithClauseContext.aliases_available`, `where_clause`, `ReturnClauseContext.aliases_available`, `UnwindClauseContext.aliases_available` 的引用调整为通过 `filter`, `yield_clause` 等当前字段访问。

- **文件**: `src\query\planner\match_planning\utils\finder.rs`
  - **任务**: 将对 `base_validator::NodeInfo.filter`, `props` 的引用替换为 `base_validator::NodeInfo.properties`。

- **文件**: `src\query\planner\match_planning\clauses\order_by_planner.rs`
  - **任务**: 将对 `OrderByClauseContext.indexed_order_factors` 的引用替换为 `OrderByClauseContext.columns`。

- **文件**: `src\query\planner\match_planning\*.rs`, `src\query\planner\ngql\*.rs`
  - **任务**: 检查 `CypherClauseContext`，并替换对 `.kind()` 的调用。这可能需要通过上下文传递的 `CypherClause` 枚举来判断类型，或调用其内部的类型字段。

- **文件**: `src\query\context\ast_context.rs` 和相关调用 `ast_ctx.statement_type()` 的文件
  - **任务**: 确保 `AstContext` 实现了 `Statement` trait，并且 `statement_type` 方法被正确实现和导出。

### 4. 修复格式化与显示问题 (Formatting & Display Issues)

- **文件**: `src\query\planner\match_planning\clauses\pagination_planner.rs`
  - **任务**: 将第 71, 72 行的 `format!("skip_{}", ...)` 和 `format!("limit_{}", ...)` 修改为 `format!("skip_{:?}", ...)` 和 `format!("limit_{:?}", ...)`，或先解包 `Option`。

### 5. 修复结构体字段不匹配 (Struct Field Mismatch)

- **文件**: `src\query\planner\ngql\fetch_vertices_planner.rs`
  - **任务**: 检查 `VariableInfo` 和 `Variable` 的定义，将创建 `VariableInfo` 的代码改为创建 `Variable`，或根据 `VariableInfo` 的字段构造 `Variable`。

### 6. 修复可变性与借用问题 (Mutability & Borrowing Issues)

- **文件**: `src\query\context\execution_context.rs`
  - **任务**: 将 `reset` 方法的签名从 `fn reset(&self)` 修改为 `fn reset(&mut self)`。

- **文件**: `src\query\executor\result_processing\aggregation.rs`
  - **任务**: 重构 `add_values_to_group` 方法内部的借用逻辑，例如将对 `self.sum` 的借用操作移到作用域之外，或使用临时变量。

- **文件**: `src\query\planner\match_planning\paths\match_path_planner.rs`
  - **任务**: 在访问 `edge.direction` 之前，先将其值克隆出来 (e.g., `let direction = edge.direction.clone();`)，然后在后续代码中使用这个克隆值。

- **文件**: `src\query\planner\match_planning\clauses\clause_planner.rs`
  - **任务**: 将第 94, 104 行的 `self.supported_kind` 访问改为 `self.supported_kind.clone()`。

### 7. 修复未使用的变量 (Unused Variables - Warnings)

- **文件**: `src\query\executor\result_processing\traits.rs`, `src\query\validator\base_validator.rs`, `src\query\validator\match_validator.rs`, `src\query\validator\create_validator.rs`
  - **任务**: 为所有标记为未使用的变量添加下划线前缀。

### 8. 修复其他问题

- **文件**: `src\query\planner\match_planning\clauses\projection_planner.rs`
  - **任务**: 检查 `create_sort_node` 和 `create_limit_node` 的函数签名，为函数调用补充缺失的 `input_plan` 参数。

- **文件**: `src\query\executor\result_processing\filter.rs`
  - **任务**: 在 `FilterExecutor` 的类型声明中补充泛型参数，如 `FilterExecutor<S>`。