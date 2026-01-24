# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 4
- **Total Warnings**: 12
- **Total Issues**: 16
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 12
- **Files with Issues**: 12

## Error Statistics

**Total Errors**: 4

### Error Type Breakdown

- **error[E0599]**: 2 errors
- **error[E0308]**: 1 errors
- **error[E0046]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\result_processing\transformations\assign.rs`: 2 errors
- `src\query\executor\aggregation.rs`: 1 errors
- `src\query\visitor\deduce_type_visitor.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 12

### Warning Type Breakdown

- **warning**: 12 warnings

### Files with Warnings (Top 10)

- `src\query\planner\statements\match_planner.rs`: 2 warnings
- `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 warnings
- `src\common\memory.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\impls.rs`: 1 warnings
- `src\query\executor\aggregation.rs`: 1 warnings
- `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 warnings
- `src\query\scheduler\async_scheduler.rs`: 1 warnings
- `src\query\executor\factory.rs`: 1 warnings
- `src\query\context\runtime_context.rs`: 1 warnings
- `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `get_variable` found for struct `query::executor::base::ExecutionContext` in the current scope: method not found in `ExecutionContext`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\result_processing\transformations\assign.rs`: 2 occurrences

- Line 183: no method named `get_variable` found for struct `query::executor::base::ExecutionContext` in the current scope: method not found in `ExecutionContext`
- Line 187: no method named `get_variable` found for struct `query::executor::base::ExecutionContext` in the current scope: method not found in `ExecutionContext`

### error[E0046]: not all trait items implemented, missing: `get_input`: missing `get_input` in implementation

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 548: not all trait items implemented, missing: `get_input`: missing `get_input` in implementation

### error[E0308]: mismatched types: expected `Value`, found floating-point number

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\aggregation.rs`: 1 occurrences

- Line 873: mismatched types: expected `Value`, found floating-point number

## Detailed Warning Categorization

### warning: unused import: `std::collections::HashMap`

**Total Occurrences**: 12  
**Unique Files**: 10

#### `src\query\planner\statements\match_planner.rs`: 2 occurrences

- Line 96: unused variable: `match_ctx`: help: if this is intentional, prefix it with an underscore: `_match_ctx`
- Line 157: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 207: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 207: variable does not need to be mutable

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 15: unused import: `std::collections::HashMap`

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 1 occurrences

- Line 10: unused macro definition: `impl_graph_traversal_executor`

#### `src\query\executor\aggregation.rs`: 1 occurrences

- Line 8: unused import: `HasInput`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 22: unused imports: `MultiShortestPathExecutor` and `ShortestPathExecutor`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 42: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 330: unnecessary parentheses around function argument

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 9: unused import: `ExecutionContext`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

