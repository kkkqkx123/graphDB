# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 130
- **Total Warnings**: 0
- **Total Issues**: 130
- **Unique Error Patterns**: 11
- **Unique Warning Patterns**: 0
- **Files with Issues**: 53

## Error Statistics

**Total Errors**: 130

### Error Type Breakdown

- **error[E0061]**: 108 errors
- **error[E0599]**: 14 errors
- **error[E0433]**: 6 errors
- **error[E0308]**: 2 errors

### Files with Errors (Top 10)

- `src\query\executor\graph_query_executor.rs`: 13 errors
- `src\query\executor\factory.rs`: 10 errors
- `src\query\executor\result_processing\projection.rs`: 7 errors
- `src\query\executor\result_processing\filter.rs`: 6 errors
- `src\query\executor\special_executors.rs`: 6 errors
- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 5 errors
- `src\query\executor\data_modification.rs`: 5 errors
- `src\query\executor\logic\loops.rs`: 4 errors
- `src\query\executor\result_processing\transformations\append_vertices.rs`: 4 errors
- `src\query\executor\result_processing\transformations\pattern_apply.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0061]: this function takes 4 arguments but 3 arguments were supplied

**Total Occurrences**: 108  
**Unique Files**: 48

#### `src\query\executor\graph_query_executor.rs`: 13 occurrences

- Line 278: this function takes 4 arguments but 3 arguments were supplied
- Line 284: this function takes 4 arguments but 3 arguments were supplied
- Line 301: this function takes 4 arguments but 3 arguments were supplied
- ... 10 more occurrences in this file

#### `src\query\executor\factory.rs`: 10 occurrences

- Line 1412: this function takes 4 arguments but 3 arguments were supplied
- Line 1417: this function takes 5 arguments but 4 arguments were supplied
- Line 1427: this function takes 5 arguments but 4 arguments were supplied
- ... 7 more occurrences in this file

#### `src\query\executor\special_executors.rs`: 6 occurrences

- Line 295: this function takes 4 arguments but 3 arguments were supplied
- Line 304: this function takes 4 arguments but 3 arguments were supplied
- Line 329: this function takes 4 arguments but 3 arguments were supplied
- ... 3 more occurrences in this file

#### `src\query\executor\data_modification.rs`: 5 occurrences

- Line 219: this function takes 4 arguments but 3 arguments were supplied
- Line 482: this function takes 4 arguments but 3 arguments were supplied
- Line 703: this function takes 4 arguments but 3 arguments were supplied
- ... 2 more occurrences in this file

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 5 occurrences

- Line 563: this function takes 1 argument but 0 arguments were supplied
- Line 605: this function takes 1 argument but 0 arguments were supplied
- Line 665: this function takes 1 argument but 0 arguments were supplied
- ... 2 more occurrences in this file

#### `src\query\executor\logic\loops.rs`: 4 occurrences

- Line 56: this function takes 4 arguments but 3 arguments were supplied
- Line 574: this function takes 4 arguments but 3 arguments were supplied
- Line 694: this function takes 4 arguments but 3 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 4 occurrences

- Line 365: this function takes 1 argument but 0 arguments were supplied
- Line 397: this function takes 1 argument but 0 arguments were supplied
- Line 430: this function takes 1 argument but 0 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\factory.rs`: 3 occurrences

- Line 22: this function takes 6 arguments but 5 arguments were supplied
- Line 34: this function takes 7 arguments but 6 arguments were supplied
- Line 46: this function takes 7 arguments but 6 arguments were supplied

#### `src\query\executor\data_processing\join\cross_join.rs`: 3 occurrences

- Line 313: this function takes 5 arguments but 4 arguments were supplied
- Line 395: this function takes 5 arguments but 4 arguments were supplied
- Line 444: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\query_management\show_stats.rs`: 3 occurrences

- Line 46: this function takes 4 arguments but 3 arguments were supplied
- Line 53: this function takes 4 arguments but 3 arguments were supplied
- Line 60: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\set_operations\union.rs`: 3 occurrences

- Line 142: this function takes 5 arguments but 4 arguments were supplied
- Line 194: this function takes 5 arguments but 4 arguments were supplied
- Line 234: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\edge\drop_edge.rs`: 2 occurrences

- Line 26: this function takes 4 arguments but 3 arguments were supplied
- Line 41: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\drop_user.rs`: 2 occurrences

- Line 24: this function takes 4 arguments but 3 arguments were supplied
- Line 32: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\tests.rs`: 2 occurrences

- Line 305: this function takes 4 arguments but 3 arguments were supplied
- Line 329: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\show_edge_index_status.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 38: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\edge\create_edge.rs`: 2 occurrences

- Line 81: this function takes 4 arguments but 3 arguments were supplied
- Line 94: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\create_user.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 33: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\tag\drop_tag.rs`: 2 occurrences

- Line 26: this function takes 4 arguments but 3 arguments were supplied
- Line 41: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\tag\create_tag.rs`: 2 occurrences

- Line 81: this function takes 4 arguments but 3 arguments were supplied
- Line 90: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\create_space.rs`: 2 occurrences

- Line 73: this function takes 4 arguments but 3 arguments were supplied
- Line 86: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\drop_space.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 34: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\show_tag_index_status.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 38: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\analyze.rs`: 2 occurrences

- Line 46: this function takes 4 arguments but 3 arguments were supplied
- Line 55: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 76: this function takes 4 arguments but 5 arguments were supplied

#### `src\query\executor\admin\user\revoke_role.rs`: 1 occurrences

- Line 24: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 77: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\base\executor_base.rs`: 1 occurrences

- Line 261: this function takes 3 arguments but 2 arguments were supplied

#### `src\query\executor\admin\space\show_spaces.rs`: 1 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\tag\desc_tag.rs`: 1 occurrences

- Line 41: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\grant_role.rs`: 1 occurrences

- Line 32: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\algorithms\multi_shortest_path.rs`: 1 occurrences

- Line 88: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 399: this function takes 1 argument but 0 arguments were supplied

#### `src\query\executor\admin\tag\show_tags.rs`: 1 occurrences

- Line 26: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 447: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\space\clear_space.rs`: 1 occurrences

- Line 23: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\edge\show_edges.rs`: 1 occurrences

- Line 26: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\base\result_processor.rs`: 1 occurrences

- Line 255: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\result_processing\transformations\unwind.rs`: 1 occurrences

- Line 381: this function takes 1 argument but 0 arguments were supplied

#### `src\query\executor\admin\space\desc_space.rs`: 1 occurrences

- Line 48: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\algorithms\subgraph_executor.rs`: 1 occurrences

- Line 184: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\switch_space.rs`: 1 occurrences

- Line 23: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\edge\alter_edge.rs`: 1 occurrences

- Line 89: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 321: this function takes 8 arguments but 7 arguments were supplied

#### `src\query\executor\admin\user\change_password.rs`: 1 occurrences

- Line 32: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\alter_space.rs`: 1 occurrences

- Line 35: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\tag\alter_tag.rs`: 1 occurrences

- Line 89: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\edge\desc_edge.rs`: 1 occurrences

- Line 42: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\alter_user.rs`: 1 occurrences

- Line 24: this function takes 4 arguments but 3 arguments were supplied

### error[E0599]: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

**Total Occurrences**: 14  
**Unique Files**: 5

#### `src\query\executor\result_processing\projection.rs`: 5 occurrences

- Line 76: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 146: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 190: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- ... 2 more occurrences in this file

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 3 occurrences

- Line 123: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 148: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 171: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

#### `src\query\executor\data_access.rs`: 3 occurrences

- Line 164: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 377: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 1015: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

#### `src\query\executor\result_processing\filter.rs`: 2 occurrences

- Line 142: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 159: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 864: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

### error[E0433]: failed to resolve: use of undeclared type `EvalContext`: use of undeclared type `EvalContext`

**Total Occurrences**: 6  
**Unique Files**: 2

#### `src\query\executor\result_processing\filter.rs`: 4 occurrences

- Line 189: failed to resolve: use of undeclared type `EvalContext`: use of undeclared type `EvalContext`
- Line 215: failed to resolve: use of undeclared type `EvalContext`: use of undeclared type `EvalContext`
- Line 244: failed to resolve: use of undeclared type `EvalContext`: use of undeclared type `EvalContext`
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 2 occurrences

- Line 240: failed to resolve: use of undeclared type `EvalContext`: use of undeclared type `EvalContext`
- Line 324: failed to resolve: use of undeclared type `EvalContext`: use of undeclared type `EvalContext`

### error[E0308]: mismatched types: expected `&ContextualExpression`, found `&Expression`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\optimizer\strategy\materialization.rs`: 2 occurrences

- Line 483: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 491: mismatched types: expected `&ContextualExpression`, found `&Expression`

