# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 77
- **Total Warnings**: 47
- **Total Issues**: 124
- **Unique Error Patterns**: 56
- **Unique Warning Patterns**: 37
- **Files with Issues**: 35

## Error Statistics

**Total Errors**: 77

### Error Type Breakdown

- **error[E0609]**: 25 errors
- **error[E0616]**: 20 errors
- **error[E0034]**: 19 errors
- **error[E0061]**: 6 errors
- **error[E0560]**: 3 errors
- **error[E0599]**: 1 errors
- **error[E0499]**: 1 errors
- **error[E0277]**: 1 errors
- **error[E0382]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\factory.rs`: 52 errors
- `src\core\context\validation.rs`: 10 errors
- `src\core\context\runtime.rs`: 9 errors
- `src\core\expression_visitor.rs`: 2 errors
- `src\query\scheduler\async_scheduler.rs`: 2 errors
- `src\core\context\manager.rs`: 1 errors
- `src\core\context\traits.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 47

### Warning Type Breakdown

- **warning**: 47 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 12 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 5 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\query\planner\match_planning\utils\finder.rs`: 2 warnings
- `src\query\optimizer\elimination_rules.rs`: 1 warnings
- `src\query\planner\match_planning\utils\connection_builder.rs`: 1 warnings
- `src\core\context\session.rs`: 1 warnings
- `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 1 warnings
- `src\query\context\execution\query_execution.rs`: 1 warnings
- `src\query\planner\ngql\go_planner.rs`: 1 warnings

## Detailed Error Categorization

### error[E0609]: no field `limit` on type `&sort_node::LimitNode`: unknown field

**Total Occurrences**: 25  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 25 occurrences

- Line 53: no field `limit` on type `&sort_node::LimitNode`: unknown field
- Line 58: no field `max_iterations` on type `&control_flow_node::LoopNode`: unknown field
- Line 68: no field `max_depth` on type `&traversal_node::ExpandNode`: unknown field
- ... 22 more occurrences in this file

### error[E0616]: field `id` of struct `start_node::StartNode` is private: private field

**Total Occurrences**: 20  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 20 occurrences

- Line 95: field `id` of struct `start_node::StartNode` is private: private field
- Line 102: field `id` of struct `graph_scan_node::ScanVerticesNode` is private: private field
- Line 105: field `tag_filter` of struct `graph_scan_node::ScanVerticesNode` is private: private field
- ... 17 more occurrences in this file

### error[E0034]: multiple applicable items in scope: multiple `touch` found

**Total Occurrences**: 19  
**Unique Files**: 2

#### `src\core\context\validation.rs`: 10 occurrences

- Line 254: multiple applicable items in scope: multiple `touch` found
- Line 260: multiple applicable items in scope: multiple `touch` found
- Line 276: multiple applicable items in scope: multiple `touch` found
- ... 7 more occurrences in this file

#### `src\core\context\runtime.rs`: 9 occurrences

- Line 255: multiple applicable items in scope: multiple `touch` found
- Line 268: multiple applicable items in scope: multiple `touch` found
- Line 274: multiple applicable items in scope: multiple `touch` found
- ... 6 more occurrences in this file

### error[E0061]: this function takes 4 arguments but 5 arguments were supplied

**Total Occurrences**: 6  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 6 occurrences

- Line 166: this function takes 4 arguments but 5 arguments were supplied
- Line 187: this function takes 7 arguments but 4 arguments were supplied
- Line 196: this function takes 7 arguments but 4 arguments were supplied
- ... 3 more occurrences in this file

### error[E0560]: struct `DefaultContextManager` has no field named `_created_at`: unknown field

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 62: struct `AsyncMsgNotifyBasedScheduler<S>` has no field named `_storage`: unknown field
- Line 63: struct `AsyncMsgNotifyBasedScheduler<S>` has no field named `_execution_context`: unknown field

#### `src\core\context\manager.rs`: 1 occurrences

- Line 111: struct `DefaultContextManager` has no field named `_created_at`: unknown field

### error[E0277]: the size for values of type `Self` cannot be known at compilation time: doesn't have a size known at compile-time

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\context\traits.rs`: 1 occurrences

- Line 190: the size for values of type `Self` cannot be known at compilation time: doesn't have a size known at compile-time

### error[E0599]: no variant or associated item named `Assign` found for enum `nodes::plan_node_enum::PlanNodeEnum` in the current scope: variant or associated item not found in `PlanNodeEnum`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 235: no variant or associated item named `Assign` found for enum `nodes::plan_node_enum::PlanNodeEnum` in the current scope: variant or associated item not found in `PlanNodeEnum`

### error[E0382]: use of moved value: `expr_type`: value moved here, in previous iteration of loop

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\expression_visitor.rs`: 1 occurrences

- Line 647: use of moved value: `expr_type`: value moved here, in previous iteration of loop

### error[E0499]: cannot borrow `*self` as mutable more than once at a time: second mutable borrow occurs here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\expression_visitor.rs`: 1 occurrences

- Line 494: cannot borrow `*self` as mutable more than once at a time: second mutable borrow occurs here

## Detailed Warning Categorization

### warning: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

**Total Occurrences**: 47  
**Unique Files**: 29

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

#### `src\query\planner\match_planning\utils\finder.rs`: 2 occurrences

- Line 345: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`
- Line 352: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`

#### `src\core\context\mod.rs`: 1 occurrences

- Line 23: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 269: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 58: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\core\context\validation.rs`: 1 occurrences

- Line 9: unused import: `ContextExt`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 1 occurrences

- Line 318: variable does not need to be mutable

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 51: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `crate::core::types::expression::DataType`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\parser\cypher\parser.rs`: 1 occurrences

- Line 340: variable does not need to be mutable

#### `src\query\planner\match_planning\clauses\order_by_planner.rs`: 1 occurrences

- Line 195: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 561: variable does not need to be mutable

#### `src\expression\visitor.rs`: 1 occurrences

- Line 287: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 347: unused import: `UnaryOperator`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 887: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 564: unused import: `SortNode`

