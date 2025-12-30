# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 1
- **Total Warnings**: 98
- **Total Issues**: 99
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 58
- **Files with Issues**: 74

## Error Statistics

**Total Errors**: 1

### Error Type Breakdown

- **error[E0046]**: 1 errors

### Files with Errors (Top 10)

- `src\query\visitor\deduce_type_visitor.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 98

### Warning Type Breakdown

- **warning**: 98 warnings

### Files with Warnings (Top 10)

- `src\query\context\ast\base.rs`: 4 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\query\executor\factory.rs`: 3 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 3 warnings
- `src\core\result\memory_manager.rs`: 3 warnings
- `src\query\executor\result_processing\sort.rs`: 2 warnings
- `src\query\executor\data_processing\join\full_outer_join.rs`: 2 warnings
- `src\query\parser\expressions\expression_converter.rs`: 2 warnings
- `src\query\executor\result_processing\topn.rs`: 2 warnings
- `src\query\executor\result_processing\aggregation.rs`: 2 warnings

## Detailed Error Categorization

### error[E0046]: not all trait items implemented, missing: `scan_all_edges`: missing `scan_all_edges` in implementation

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 989: not all trait items implemented, missing: `scan_all_edges`: missing `scan_all_edges` in implementation

## Detailed Warning Categorization

### warning: unused import: `SortNode`

**Total Occurrences**: 98  
**Unique Files**: 73

#### `src\query\context\ast\base.rs`: 4 occurrences

- Line 37: unused variable: `query_type`: help: if this is intentional, prefix it with an underscore: `_query_type`
- Line 37: unused variable: `query_text`: help: if this is intentional, prefix it with an underscore: `_query_text`
- Line 124: unused variable: `query_type`: help: if this is intentional, prefix it with an underscore: `_query_type`
- ... 1 more occurrences in this file

#### `src\core\result\memory_manager.rs`: 3 occurrences

- Line 444: unexpected `cfg` condition value: `system-monitor`: help: remove the condition
- Line 525: unexpected `cfg` condition value: `system-monitor`: help: remove the condition
- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\expression\evaluator\expression_evaluator.rs`: 3 occurrences

- Line 241: unused variable: `value`: help: if this is intentional, prefix it with an underscore: `_value`
- Line 242: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`
- Line 243: unused variable: `escape_char`: help: if this is intentional, prefix it with an underscore: `_escape_char`

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 10: unused import: `std::sync::Arc`
- Line 47: unused variable: `schema_id`: help: if this is intentional, prefix it with an underscore: `_schema_id`
- Line 49: unused variable: `return_cols`: help: if this is intentional, prefix it with an underscore: `_return_cols`

#### `src\query\executor\factory.rs`: 3 occurrences

- Line 340: unused variable: `node`: help: if this is intentional, prefix it with an underscore: `_node`
- Line 143: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- Line 234: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`

#### `src\query\executor\result_processing\filter.rs`: 2 occurrences

- Line 20: unused import: `HasStorage`
- Line 299: unused import: `crate::core::value::NullType`

#### `src\query\planner\match_planning\utils\finder.rs`: 2 occurrences

- Line 345: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`
- Line 352: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 18: unused import: `HasStorage`
- Line 491: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\sort.rs`: 2 occurrences

- Line 204: unused variable: `estimated_memory`: help: if this is intentional, prefix it with an underscore: `_estimated_memory`
- Line 745: variable does not need to be mutable

#### `src\query\context\managers\retry.rs`: 2 occurrences

- Line 7: unused import: `ErrorCategory`
- Line 149: variable does not need to be mutable

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 2 occurrences

- Line 7: unused import: `crate::core::error::DBResult`
- Line 8: unused import: `crate::query::executor::traits::ExecutionResult`

#### `src\query\planner\ngql\go_planner.rs`: 2 occurrences

- Line 5: unused import: `crate::query::parser::ast::expr::Expr`
- Line 62: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 2 occurrences

- Line 11: unused import: `HasStorage`
- Line 107: unused variable: `right_col_map`: help: if this is intentional, prefix it with an underscore: `_right_col_map`

#### `src\query\executor\result_processing\topn.rs`: 2 occurrences

- Line 20: unused import: `HasStorage`
- Line 494: unused import: `crate::core::value::NullType`

#### `src\query\executor\data_processing\graph_traversal\traits.rs`: 2 occurrences

- Line 3: unused import: `crate::core::error::DBResult`
- Line 4: unused imports: `ExecutionResult` and `Executor`

#### `src\query\executor\result_processing\limit.rs`: 2 occurrences

- Line 15: unused import: `HasStorage`
- Line 284: unused import: `crate::core::value::NullType`

#### `src\query\executor\data_processing\join\inner_join.rs`: 2 occurrences

- Line 10: unused import: `crate::expression::evaluator::expression_evaluator::ExpressionEvaluator`
- Line 11: unused import: `crate::expression::evaluator::traits::ExpressionContext`

#### `src\query\parser\expressions\expression_converter.rs`: 2 occurrences

- Line 6: unused import: `NullType`
- Line 457: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 23: unused import: `HasStorage`
- Line 944: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 564: unused import: `SortNode`

#### `src\query\parser\cypher\parser.rs`: 1 occurrences

- Line 257: variable does not need to be mutable

#### `src\expression\evaluator\operations.rs`: 1 occurrences

- Line 7: unused import: `ExpressionErrorType`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 340: unused import: `UnaryOperator`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `crate::core::types::expression::DataType`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 12: unused import: `HasStorage`

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 11: unused import: `HasStorage`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 242: unused imports: `Direction` and `Value`

#### `src\query\context\managers\index_manager.rs`: 1 occurrences

- Line 5: unused import: `ManagerError`

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 18: unused import: `HasStorage`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 182: unused variable: `id_str`: help: if this is intentional, prefix it with an underscore: `_id_str`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\context\ast\common.rs`: 1 occurrences

- Line 4: unused import: `crate::query::parser::ast::expr::Expr`

#### `src\query\planner\match_planning\clauses\order_by_planner.rs`: 1 occurrences

- Line 195: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `src\core\result\result_core.rs`: 1 occurrences

- Line 186: variable does not need to be mutable

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\context\managers\schema_manager.rs`: 1 occurrences

- Line 5: unused import: `ManagerError`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 52: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\context\managers\meta_client.rs`: 1 occurrences

- Line 4: unused import: `ManagerError`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 480: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 11: unused import: `HasStorage`

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 269: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 8: unused import: `ManagerError`

#### `src\query\executor\recursion_detector.rs`: 1 occurrences

- Line 3: unused import: `HashMap`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 241: variable does not need to be mutable

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 16: unused import: `HasStorage`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 259: unused import: `std::fs`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 316: unused import: `crate::storage::StorageEngine`

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 10: unused import: `HasStorage`

#### `src\core\context\request.rs`: 1 occurrences

- Line 14: unused import: `SessionStatus`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\context\ast\cypher_ast_context.rs`: 1 occurrences

- Line 212: unused variable: `label`: help: if this is intentional, prefix it with an underscore: `_label`

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 16: unused import: `HasStorage`

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 7: unused import: `ManagerError`

#### `src\query\context\managers\storage_client.rs`: 1 occurrences

- Line 5: unused import: `ManagerError`

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 15: unused import: `HasStorage`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 10: unused import: `HasStorage`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 530: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 887: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 264: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 792: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`

#### `src\expression\visitor.rs`: 1 occurrences

- Line 278: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\query\executor\base.rs`: 1 occurrences

- Line 7: unused import: `HasInput`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 9: unused import: `SessionStatus`

#### `src\core\context\mod.rs`: 1 occurrences

- Line 5: unused import: `crate::core::Value`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

