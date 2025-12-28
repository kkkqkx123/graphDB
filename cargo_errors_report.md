# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 18
- **Total Warnings**: 84
- **Total Issues**: 102
- **Unique Error Patterns**: 15
- **Unique Warning Patterns**: 55
- **Files with Issues**: 60

## Error Statistics

**Total Errors**: 18

### Error Type Breakdown

- **error[E0599]**: 9 errors
- **error[E0308]**: 3 errors
- **error[E0034]**: 2 errors
- **error[E0592]**: 1 errors
- **error[E0609]**: 1 errors
- **error[E0277]**: 1 errors
- **error[E0615]**: 1 errors

### Files with Errors (Top 10)

- `src\query\parser\cypher\ast\converters.rs`: 5 errors
- `src\core\value\types.rs`: 4 errors
- `src\query\visitor\deduce_type_visitor.rs`: 3 errors
- `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 errors
- `src\expression\aggregate_functions.rs`: 2 errors
- `src\services\function.rs`: 1 errors
- `src\expression\evaluator\operations.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 84

### Warning Type Breakdown

- **warning**: 84 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 12 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 6 warnings
- `src\query\executor\data_processing\join\inner_join.rs`: 3 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\core\value\mod.rs`: 3 warnings
- `src\query\executor\factory.rs`: 3 warnings
- `src\query\executor\result_processing\sort.rs`: 2 warnings
- `src\query\executor\data_processing\join\full_outer_join.rs`: 2 warnings
- `src\query\planner\match_planning\utils\finder.rs`: 2 warnings
- `src\query\parser\expressions\expression_converter.rs`: 2 warnings

## Detailed Error Categorization

### error[E0599]: no method named `hash` found for reference `&core::value::types::Value` in the current scope: method not found in `&Value`

**Total Occurrences**: 9  
**Unique Files**: 3

#### `src\query\parser\cypher\ast\converters.rs`: 5 occurrences

- Line 733: no method named `negate` found for enum `core::value::types::Value` in the current scope
- Line 774: no method named `abs` found for enum `core::value::types::Value` in the current scope: method not found in `Value`
- Line 780: no method named `length` found for enum `core::value::types::Value` in the current scope
- ... 2 more occurrences in this file

#### `src\query\visitor\deduce_type_visitor.rs`: 3 occurrences

- Line 945: no variant or associated item named `PathBuild` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`
- Line 965: no variant named `SubscriptRange` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 984: no variant named `TypeCasting` found for enum `core::types::expression::Expression`

#### `src\core\value\types.rs`: 1 occurrences

- Line 236: no method named `hash` found for reference `&core::value::types::Value` in the current scope: method not found in `&Value`

### error[E0308]: mismatched types: expected `String`, found `Result<String, String>`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\services\function.rs`: 1 occurrences

- Line 280: mismatched types: expected `String`, found `Result<String, String>`

#### `src\expression\aggregate_functions.rs`: 1 occurrences

- Line 207: mismatched types: expected `String`, found `Result<String, String>`

#### `src\expression\evaluator\operations.rs`: 1 occurrences

- Line 274: mismatched types: expected `&str`, found `&Result<String, String>`

### error[E0034]: multiple applicable items in scope: multiple `is_empty` found

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 282: multiple applicable items in scope: multiple `is_empty` found
- Line 242: multiple applicable items in scope: multiple `is_empty` found

### error[E0615]: attempted to take value of method `id` on type `&Box<vertex_edge_path::Vertex>`: method, not a field

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\value\types.rs`: 1 occurrences

- Line 368: attempted to take value of method `id` on type `&Box<vertex_edge_path::Vertex>`: method, not a field

### error[E0609]: no field `id` on type `&vertex_edge_path::Edge`: unknown field

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\value\types.rs`: 1 occurrences

- Line 369: no field `id` on type `&vertex_edge_path::Edge`: unknown field

### error[E0277]: the trait bound `std::string::String: Borrow<std::result::Result<std::string::String, std::string::String>>` is not satisfied: the trait `Borrow<std::result::Result<std::string::String, std::string::String>>` is not implemented for `std::string::String`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\expression\aggregate_functions.rs`: 1 occurrences

- Line 160: the trait bound `std::string::String: Borrow<std::result::Result<std::string::String, std::string::String>>` is not satisfied: the trait `Borrow<std::result::Result<std::string::String, std::string::String>>` is not implemented for `std::string::String`

### error[E0592]: duplicate definitions with name `is_empty`: duplicate definitions for `is_empty`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\value\types.rs`: 1 occurrences

- Line 219: duplicate definitions with name `is_empty`: duplicate definitions for `is_empty`

## Detailed Warning Categorization

### warning: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

**Total Occurrences**: 84  
**Unique Files**: 54

#### `src\query\planner\plan\core\nodes\factory.rs`: 12 occurrences

- Line 36: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 32: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- Line 50: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- ... 9 more occurrences in this file

#### `src\expression\evaluator\expression_evaluator.rs`: 6 occurrences

- Line 6: unused import: `BinaryOperator`
- Line 284: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 284: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- ... 3 more occurrences in this file

#### `src\core\value\mod.rs`: 3 occurrences

- Line 16: unused import: `comparison::*`
- Line 17: unused import: `operations::*`
- Line 18: unused import: `conversion::*`

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 127: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\query\executor\factory.rs`: 3 occurrences

- Line 341: unused variable: `node`: help: if this is intentional, prefix it with an underscore: `_node`
- Line 144: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- Line 235: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`

#### `src\query\executor\data_processing\join\inner_join.rs`: 3 occurrences

- Line 11: unused import: `crate::expression::evaluator::traits::ExpressionContext`
- Line 73: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`
- Line 145: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`

#### `src\query\executor\result_processing\sort.rs`: 2 occurrences

- Line 204: unused variable: `estimated_memory`: help: if this is intentional, prefix it with an underscore: `_estimated_memory`
- Line 772: variable does not need to be mutable

#### `src\query\parser\expressions\expression_converter.rs`: 2 occurrences

- Line 6: unused import: `NullType`
- Line 457: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\planner\match_planning\utils\finder.rs`: 2 occurrences

- Line 345: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`
- Line 352: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 2 occurrences

- Line 11: unused import: `HasStorage`
- Line 107: unused variable: `right_col_map`: help: if this is intentional, prefix it with an underscore: `_right_col_map`

#### `src\query\executor\data_processing\join\left_join.rs`: 2 occurrences

- Line 10: unused import: `crate::expression::evaluator::expression_evaluator::ExpressionEvaluator`
- Line 11: unused import: `crate::expression::evaluator::traits::ExpressionContext`

#### `src\core\context\mod.rs`: 2 occurrences

- Line 5: unused import: `crate::core::Value`
- Line 46: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 20: unused import: `HasStorage`

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 269: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 561: variable does not need to be mutable

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 339: variable does not need to be mutable

#### `src\query\parser\cypher\parser.rs`: 1 occurrences

- Line 257: variable does not need to be mutable

#### `src\expression\visitor.rs`: 1 occurrences

- Line 278: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 11: unused import: `HasStorage`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 52: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 16: unused import: `HasStorage`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 887: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `crate::core::types::expression::DataType`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 11: unused import: `HasStorage`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 18: unused import: `HasStorage`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 15: unused import: `HasStorage`

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 9: unused import: `Evaluator`

#### `src\query\planner\match_planning\clauses\order_by_planner.rs`: 1 occurrences

- Line 195: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 1 occurrences

- Line 318: variable does not need to be mutable

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 18: unused import: `HasStorage`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 10: unused import: `HasStorage`

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 60: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 10: unused import: `HasStorage`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 20: unused import: `HasStorage`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\expression\evaluator\operations.rs`: 1 occurrences

- Line 7: unused import: `ExpressionErrorType`

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 16: unused import: `HasStorage`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 23: unused import: `HasStorage`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 340: unused import: `UnaryOperator`

#### `src\query\executor\result_processing\limit.rs`: 1 occurrences

- Line 15: unused import: `HasStorage`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 564: unused import: `SortNode`

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 12: unused import: `HasStorage`

#### `src\query\executor\base.rs`: 1 occurrences

- Line 7: unused import: `HasInput`

#### `src\query\executor\recursion_detector.rs`: 1 occurrences

- Line 3: unused import: `HashMap`

