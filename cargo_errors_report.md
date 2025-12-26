# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 104
- **Total Warnings**: 51
- **Total Issues**: 155
- **Unique Error Patterns**: 62
- **Unique Warning Patterns**: 39
- **Files with Issues**: 47

## Error Statistics

**Total Errors**: 104

### Error Type Breakdown

- **error[E0609]**: 21 errors
- **error[E0308]**: 21 errors
- **error[E0616]**: 20 errors
- **error[E0034]**: 16 errors
- **error[E0599]**: 7 errors
- **error[E0277]**: 6 errors
- **error[E0061]**: 5 errors
- **error[E0560]**: 3 errors
- **error[E0053]**: 2 errors
- **error[E0119]**: 1 errors
- **error[E0499]**: 1 errors
- **error[E0382]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\factory.rs`: 47 errors
- `src\core\context\runtime.rs`: 9 errors
- `src\core\context\validation.rs`: 7 errors
- `src\query\executor\data_processing\join\full_outer_join.rs`: 7 errors
- `src\query\executor\data_processing\join\mod.rs`: 5 errors
- `src\query\executor\data_processing\join\right_join.rs`: 5 errors
- `src\query\executor\data_processing\join\left_join.rs`: 4 errors
- `src\query\executor\cypher\base.rs`: 4 errors
- `src\query\executor\result_processing\aggregation.rs`: 3 errors
- `src\query\executor\data_modification.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 51

### Warning Type Breakdown

- **warning**: 51 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 12 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 5 warnings
- `src\query\executor\data_processing\join\inner_join.rs`: 3 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\query\executor\data_processing\join\left_join.rs`: 2 warnings
- `src\query\planner\match_planning\utils\finder.rs`: 2 warnings
- `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 warnings
- `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 warnings
- `src\query\optimizer\scan_optimization.rs`: 1 warnings
- `src\core\context\mod.rs`: 1 warnings

## Detailed Error Categorization

### error[E0308]: arguments to this function are incorrect

**Total Occurrences**: 21  
**Unique Files**: 10

#### `src\query\executor\data_processing\join\mod.rs`: 5 occurrences

- Line 153: arguments to this function are incorrect
- Line 163: arguments to this function are incorrect
- Line 176: arguments to this function are incorrect
- ... 2 more occurrences in this file

#### `src\query\executor\cypher\base.rs`: 4 occurrences

- Line 43: mismatched types: expected `usize`, found `i64`
- Line 55: mismatched types: expected `usize`, found `i64`
- Line 72: mismatched types: expected `usize`, found `i64`
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\join\left_join.rs`: 4 occurrences

- Line 459: mismatched types: expected `Expression`, found `String`
- Line 460: mismatched types: expected `Expression`, found `String`
- Line 543: mismatched types: expected `Expression`, found `String`
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 236: mismatched types: expected `i64`, found `usize`
- Line 805: mismatched types: expected `usize`, found `i64`

#### `src\query\executor\data_processing\join\right_join.rs`: 1 occurrences

- Line 32: arguments to this function are incorrect

#### `src\query\executor\result_processing\traits.rs`: 1 occurrences

- Line 277: mismatched types: expected `usize`, found `i64`

#### `src\query\executor\cypher\clauses\match_executor.rs`: 1 occurrences

- Line 50: mismatched types: expected `usize`, found `i64`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 97: mismatched types: expected `usize`, found `i64`

#### `src\query\executor\cypher\factory.rs`: 1 occurrences

- Line 140: mismatched types: expected `usize`, found `i64`

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 32: arguments to this function are incorrect

### error[E0609]: no field `vertex_ids` on type `&graph_scan_node::GetVerticesNode`: unknown field

**Total Occurrences**: 21  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 21 occurrences

- Line 85: no field `vertex_ids` on type `&graph_scan_node::GetVerticesNode`: unknown field
- Line 86: no field `tag_filter` on type `&graph_scan_node::GetVerticesNode`: unknown field
- Line 87: no field `vertex_filter` on type `&graph_scan_node::GetVerticesNode`: unknown field
- ... 18 more occurrences in this file

### error[E0616]: field `id` of struct `start_node::StartNode` is private: private field

**Total Occurrences**: 20  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 20 occurrences

- Line 58: field `id` of struct `start_node::StartNode` is private: private field
- Line 65: field `id` of struct `graph_scan_node::ScanVerticesNode` is private: private field
- Line 68: field `tag_filter` of struct `graph_scan_node::ScanVerticesNode` is private: private field
- ... 17 more occurrences in this file

### error[E0034]: multiple applicable items in scope: multiple `touch` found

**Total Occurrences**: 16  
**Unique Files**: 2

#### `src\core\context\runtime.rs`: 9 occurrences

- Line 255: multiple applicable items in scope: multiple `touch` found
- Line 268: multiple applicable items in scope: multiple `touch` found
- Line 274: multiple applicable items in scope: multiple `touch` found
- ... 6 more occurrences in this file

#### `src\core\context\validation.rs`: 7 occurrences

- Line 184: multiple applicable items in scope: multiple `touch` found
- Line 190: multiple applicable items in scope: multiple `touch` found
- Line 259: multiple applicable items in scope: multiple `touch` found
- ... 4 more occurrences in this file

### error[E0599]: no method named `error_count` found for reference `&core::context::validation::ValidationContext` in the current scope: method not found in `&ValidationContext`

**Total Occurrences**: 7  
**Unique Files**: 4

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 3 occurrences

- Line 115: no method named `parse` found for enum `core::types::expression::Expression` in the current scope: method not found in `Expression`
- Line 151: no method named `parse` found for enum `core::types::expression::Expression` in the current scope: method not found in `Expression`
- Line 191: no method named `parse` found for enum `core::types::expression::Expression` in the current scope: method not found in `Expression`

#### `src\query\executor\data_processing\join\right_join.rs`: 2 occurrences

- Line 115: no method named `parse` found for enum `core::types::expression::Expression` in the current scope: method not found in `Expression`
- Line 155: no method named `parse` found for enum `core::types::expression::Expression` in the current scope: method not found in `Expression`

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 570: no method named `error_count` found for reference `&core::context::validation::ValidationContext` in the current scope: method not found in `&ValidationContext`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 199: no variant or associated item named `Assign` found for enum `nodes::plan_node_enum::PlanNodeEnum` in the current scope: variant or associated item not found in `PlanNodeEnum`

### error[E0277]: can't compare `std::string::String` with `core::types::expression::Expression`: no implementation for `std::string::String == core::types::expression::Expression`

**Total Occurrences**: 6  
**Unique Files**: 3

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 3 occurrences

- Line 108: can't compare `std::string::String` with `core::types::expression::Expression`: no implementation for `std::string::String == core::types::expression::Expression`
- Line 144: can't compare `std::string::String` with `core::types::expression::Expression`: no implementation for `std::string::String == core::types::expression::Expression`
- Line 184: can't compare `std::string::String` with `core::types::expression::Expression`: no implementation for `std::string::String == core::types::expression::Expression`

#### `src\query\executor\data_processing\join\right_join.rs`: 2 occurrences

- Line 108: can't compare `std::string::String` with `core::types::expression::Expression`: no implementation for `std::string::String == core::types::expression::Expression`
- Line 148: can't compare `std::string::String` with `core::types::expression::Expression`: no implementation for `std::string::String == core::types::expression::Expression`

#### `src\core\context\traits.rs`: 1 occurrences

- Line 190: the size for values of type `Self` cannot be known at compilation time: doesn't have a size known at compile-time

### error[E0061]: this function takes 4 arguments but 5 arguments were supplied

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 5 occurrences

- Line 129: this function takes 4 arguments but 5 arguments were supplied
- Line 150: this function takes 7 arguments but 4 arguments were supplied
- Line 159: this function takes 7 arguments but 4 arguments were supplied
- ... 2 more occurrences in this file

### error[E0560]: struct `DefaultContextManager` has no field named `_created_at`: unknown field

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 62: struct `AsyncMsgNotifyBasedScheduler<S>` has no field named `_storage`: unknown field
- Line 63: struct `AsyncMsgNotifyBasedScheduler<S>` has no field named `_execution_context`: unknown field

#### `src\core\context\manager.rs`: 1 occurrences

- Line 111: struct `DefaultContextManager` has no field named `_created_at`: unknown field

### error[E0053]: method `id` has an incompatible type for trait: expected `i64`, found `usize`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 804: method `id` has an incompatible type for trait: expected `i64`, found `usize`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 96: method `id` has an incompatible type for trait: expected `i64`, found `usize`

### error[E0499]: cannot borrow `*self` as mutable more than once at a time: second mutable borrow occurs here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\expression_visitor.rs`: 1 occurrences

- Line 494: cannot borrow `*self` as mutable more than once at a time: second mutable borrow occurs here

### error[E0382]: use of moved value: `expr_type`: value moved here, in previous iteration of loop

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\expression_visitor.rs`: 1 occurrences

- Line 647: use of moved value: `expr_type`: value moved here, in previous iteration of loop

### error[E0119]: conflicting implementations of trait `From<CypherExecutorError>` for type `core::error::DBError`: conflicting implementation for `core::error::DBError`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\cypher\mod.rs`: 1 occurrences

- Line 67: conflicting implementations of trait `From<CypherExecutorError>` for type `core::error::DBError`: conflicting implementation for `core::error::DBError`

## Detailed Warning Categorization

### warning: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

**Total Occurrences**: 51  
**Unique Files**: 30

#### `src\query\planner\plan\core\nodes\factory.rs`: 12 occurrences

- Line 35: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 31: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- Line 49: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- ... 9 more occurrences in this file

#### `src\expression\evaluator\expression_evaluator.rs`: 5 occurrences

- Line 304: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 304: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- Line 1009: unused variable: `regex_pattern`: help: if this is intentional, prefix it with an underscore: `_regex_pattern`
- ... 2 more occurrences in this file

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 127: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\query\executor\data_processing\join\inner_join.rs`: 3 occurrences

- Line 11: unused import: `crate::expression::evaluator::traits::ExpressionContext`
- Line 73: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`
- Line 145: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`

#### `src\query\planner\match_planning\utils\finder.rs`: 2 occurrences

- Line 345: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`
- Line 352: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`

#### `src\query\executor\data_processing\join\left_join.rs`: 2 occurrences

- Line 11: unused import: `crate::expression::evaluator::traits::ExpressionContext`
- Line 126: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`

#### `src\expression\visitor.rs`: 1 occurrences

- Line 287: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 564: unused import: `SortNode`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\core\context\validation.rs`: 1 occurrences

- Line 8: unused import: `ContextExt`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\planner\match_planning\clauses\order_by_planner.rs`: 1 occurrences

- Line 195: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 269: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\core\context\mod.rs`: 1 occurrences

- Line 23: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 887: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\parser\cypher\parser.rs`: 1 occurrences

- Line 257: variable does not need to be mutable

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 347: unused import: `UnaryOperator`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `crate::core::types::expression::DataType`

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 58: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 1 occurrences

- Line 318: variable does not need to be mutable

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 51: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

