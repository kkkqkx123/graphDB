# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 40
- **Total Warnings**: 56
- **Total Issues**: 96
- **Unique Error Patterns**: 13
- **Unique Warning Patterns**: 45
- **Files with Issues**: 43

## Error Statistics

**Total Errors**: 40

### Error Type Breakdown

- **error[E0282]**: 17 errors
- **error[E0599]**: 16 errors
- **error[E0308]**: 4 errors
- **error[E0407]**: 2 errors
- **error[E0061]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 10 errors
- `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 8 errors
- `src\query\executor\data_processing\graph_traversal\traverse.rs`: 8 errors
- `src\query\executor\data_modification.rs`: 6 errors
- `src\query\executor\data_processing\graph_traversal\expand.rs`: 4 errors
- `src\query\executor\cypher\base.rs`: 1 errors
- `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 errors
- `src\query\executor\data_processing\loops.rs`: 1 errors
- `src\query\executor\factory.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 56

### Warning Type Breakdown

- **warning**: 56 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 12 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 5 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\query\executor\data_processing\join\inner_join.rs`: 3 warnings
- `src\query\planner\match_planning\utils\finder.rs`: 2 warnings
- `src\query\executor\data_processing\join\left_join.rs`: 2 warnings
- `src\core\context\mod.rs`: 2 warnings
- `src\expression\context\basic_context.rs`: 1 warnings
- `src\core\context\session.rs`: 1 warnings
- `src\query\executor\data_processing\join\hash_table.rs`: 1 warnings

## Detailed Error Categorization

### error[E0282]: type annotations needed for `std::sync::MutexGuard<'_, _>`

**Total Occurrences**: 17  
**Unique Files**: 5

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 5 occurrences

- Line 95: type annotations needed for `std::sync::MutexGuard<'_, _>`
- Line 157: type annotations needed for `std::sync::MutexGuard<'_, _>`
- Line 186: type annotations needed for `std::sync::MutexGuard<'_, _>`
- ... 2 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 4 occurrences

- Line 70: type annotations needed for `std::sync::MutexGuard<'_, _>`
- Line 175: type annotations needed for `std::sync::MutexGuard<'_, _>`
- Line 262: type annotations needed for `std::sync::MutexGuard<'_, _>`
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 4 occurrences

- Line 96: type annotations needed for `std::sync::MutexGuard<'_, _>`
- Line 187: type annotations needed for `std::sync::MutexGuard<'_, _>`
- Line 317: type annotations needed for `std::sync::MutexGuard<'_, _>`
- ... 1 more occurrences in this file

#### `src\query\executor\data_modification.rs`: 2 occurrences

- Line 58: type annotations needed for `std::sync::MutexGuard<'_, _>`
- Line 68: type annotations needed for `std::sync::MutexGuard<'_, _>`

#### `src\query\executor\data_processing\graph_traversal\expand.rs`: 2 occurrences

- Line 67: type annotations needed for `std::sync::MutexGuard<'_, _>`
- Line 153: type annotations needed for `std::sync::MutexGuard<'_, _>`

### error[E0599]: no method named `get_storage` found for mutable reference `&mut append_vertices::AppendVerticesExecutor<S>` in the current scope

**Total Occurrences**: 16  
**Unique Files**: 5

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 5 occurrences

- Line 95: no method named `get_storage` found for reference `&ShortestPathExecutor<S>` in the current scope
- Line 157: no method named `get_storage` found for mutable reference `&mut ShortestPathExecutor<S>` in the current scope
- Line 186: no method named `get_storage` found for mutable reference `&mut ShortestPathExecutor<S>` in the current scope
- ... 2 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 4 occurrences

- Line 96: no method named `get_storage` found for reference `&TraverseExecutor<S>` in the current scope
- Line 187: no method named `get_storage` found for mutable reference `&mut TraverseExecutor<S>` in the current scope
- Line 317: no method named `get_storage` found for mutable reference `&mut TraverseExecutor<S>` in the current scope
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 4 occurrences

- Line 70: no method named `get_storage` found for reference `&ExpandAllExecutor<S>` in the current scope
- Line 175: no method named `get_storage` found for mutable reference `&'a mut ExpandAllExecutor<S>` in the current scope
- Line 262: no method named `get_storage` found for mutable reference `&mut ExpandAllExecutor<S>` in the current scope
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\expand.rs`: 2 occurrences

- Line 67: no method named `get_storage` found for reference `&ExpandExecutor<S>` in the current scope
- Line 153: no method named `get_storage` found for reference `&ExpandExecutor<S>` in the current scope

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 275: no method named `get_storage` found for mutable reference `&mut append_vertices::AppendVerticesExecutor<S>` in the current scope

### error[E0308]: mismatched types: expected `&Mutex<_>`, found `&Option<Arc<Mutex<S>>>`

**Total Occurrences**: 4  
**Unique Files**: 1

#### `src\query\executor\data_modification.rs`: 4 occurrences

- Line 58: mismatched types: expected `&Mutex<_>`, found `&Option<Arc<Mutex<S>>>`
- Line 68: mismatched types: expected `&Mutex<_>`, found `&Option<Arc<Mutex<S>>>`
- Line 274: mismatched types: expected `&Mutex<_>`, found `&Option<Arc<Mutex<S>>>`
- ... 1 more occurrences in this file

### error[E0407]: method `storage` is not a member of trait `Executor`: not a member of trait `Executor`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\executor\cypher\base.rs`: 1 occurrences

- Line 285: method `storage` is not a member of trait `Executor`: not a member of trait `Executor`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 693: method `storage` is not a member of trait `Executor`: not a member of trait `Executor`

### error[E0061]: this function takes 1 argument but 2 arguments were supplied

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 114: this function takes 1 argument but 2 arguments were supplied

## Detailed Warning Categorization

### warning: unused import: `SortNode`

**Total Occurrences**: 56  
**Unique Files**: 34

#### `src\query\planner\plan\core\nodes\factory.rs`: 12 occurrences

- Line 36: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 32: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- Line 50: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- ... 9 more occurrences in this file

#### `src\expression\evaluator\expression_evaluator.rs`: 5 occurrences

- Line 304: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 304: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- Line 1009: unused variable: `regex_pattern`: help: if this is intentional, prefix it with an underscore: `_regex_pattern`
- ... 2 more occurrences in this file

#### `src\query\executor\data_processing\join\inner_join.rs`: 3 occurrences

- Line 11: unused import: `crate::expression::evaluator::traits::ExpressionContext`
- Line 73: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`
- Line 145: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 127: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\core\context\mod.rs`: 2 occurrences

- Line 5: unused import: `crate::core::Value`
- Line 46: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\query\planner\match_planning\utils\finder.rs`: 2 occurrences

- Line 345: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`
- Line 352: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`

#### `src\query\executor\data_processing\join\left_join.rs`: 2 occurrences

- Line 10: unused import: `crate::expression::evaluator::expression_evaluator::ExpressionEvaluator`
- Line 11: unused import: `crate::expression::evaluator::traits::ExpressionContext`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 564: unused import: `SortNode`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 52: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `crate::core::types::expression::DataType`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 107: unused variable: `right_col_map`: help: if this is intentional, prefix it with an underscore: `_right_col_map`

#### `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 1 occurrences

- Line 318: variable does not need to be mutable

#### `src\query\parser\cypher\parser.rs`: 1 occurrences

- Line 257: variable does not need to be mutable

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 175: unused variable: `col_names`: help: if this is intentional, prefix it with an underscore: `_col_names`

#### `src\expression\visitor.rs`: 1 occurrences

- Line 287: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 887: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\planner\match_planning\clauses\order_by_planner.rs`: 1 occurrences

- Line 195: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 60: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 347: unused import: `UnaryOperator`

#### `src\query\executor\base.rs`: 1 occurrences

- Line 7: unused import: `HasInput`

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 12: unused imports: `Executor` and `HasStorage`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 561: variable does not need to be mutable

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 269: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

