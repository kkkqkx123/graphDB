# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 291
- **Total Warnings**: 65
- **Total Issues**: 356
- **Unique Error Patterns**: 17
- **Unique Warning Patterns**: 8
- **Files with Issues**: 116

## Error Statistics

**Total Errors**: 291

### Error Type Breakdown

- **error[E0061]**: 108 errors
- **error[E0433]**: 78 errors
- **error[E0412]**: 69 errors
- **error[E0432]**: 21 errors
- **error[E0599]**: 13 errors
- **error[E0308]**: 2 errors

### Files with Errors (Top 10)

- `src\query\executor\graph_query_executor.rs`: 16 errors
- `src\query\executor\data_access.rs`: 11 errors
- `src\query\executor\factory.rs`: 10 errors
- `src\query\executor\special_executors.rs`: 9 errors
- `src\query\executor\data_modification.rs`: 9 errors
- `src\query\executor\result_processing\projection.rs`: 8 errors
- `src\query\planner\rewrite\merge\collapse_project.rs`: 8 errors
- `src\query\executor\admin\index\tag_index.rs`: 6 errors
- `src\query\executor\admin\index\edge_index.rs`: 6 errors
- `src\query\parser\parser\parse_context.rs`: 6 errors

## Warning Statistics

**Total Warnings**: 65

### Warning Type Breakdown

- **warning**: 65 warnings

### Files with Warnings (Top 10)

- `src\query\planner\rewrite\projection_pushdown\projection_pushdown.rs`: 3 warnings
- `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 3 warnings
- `src\query\optimizer\analysis\expression.rs`: 2 warnings
- `src\query\executor\result_processing\projection.rs`: 2 warnings
- `src\query\planner\rewrite\merge\collapse_project.rs`: 2 warnings
- `src\query\validator\helpers\variable_checker.rs`: 1 warnings
- `src\query\planner\plan\core\nodes\project_node.rs`: 1 warnings
- `src\query\validator\statements\delete_validator.rs`: 1 warnings
- `src\query\validator\statements\fetch_edges_validator.rs`: 1 warnings
- `src\query\executor\search_executors.rs`: 1 warnings

## Detailed Error Categorization

### error[E0061]: this function takes 6 arguments but 5 arguments were supplied

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

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 5 occurrences

- Line 563: this function takes 1 argument but 0 arguments were supplied
- Line 605: this function takes 1 argument but 0 arguments were supplied
- Line 665: this function takes 1 argument but 0 arguments were supplied
- ... 2 more occurrences in this file

#### `src\query\executor\data_modification.rs`: 5 occurrences

- Line 219: this function takes 4 arguments but 3 arguments were supplied
- Line 482: this function takes 4 arguments but 3 arguments were supplied
- Line 703: this function takes 4 arguments but 3 arguments were supplied
- ... 2 more occurrences in this file

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 4 occurrences

- Line 365: this function takes 1 argument but 0 arguments were supplied
- Line 397: this function takes 1 argument but 0 arguments were supplied
- Line 430: this function takes 1 argument but 0 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\logic\loops.rs`: 4 occurrences

- Line 56: this function takes 4 arguments but 3 arguments were supplied
- Line 574: this function takes 4 arguments but 3 arguments were supplied
- Line 694: this function takes 4 arguments but 3 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\factory.rs`: 3 occurrences

- Line 22: this function takes 6 arguments but 5 arguments were supplied
- Line 34: this function takes 7 arguments but 6 arguments were supplied
- Line 46: this function takes 7 arguments but 6 arguments were supplied

#### `src\query\executor\data_processing\set_operations\union.rs`: 3 occurrences

- Line 142: this function takes 5 arguments but 4 arguments were supplied
- Line 194: this function takes 5 arguments but 4 arguments were supplied
- Line 234: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\data_processing\join\cross_join.rs`: 3 occurrences

- Line 313: this function takes 5 arguments but 4 arguments were supplied
- Line 395: this function takes 5 arguments but 4 arguments were supplied
- Line 444: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\query_management\show_stats.rs`: 3 occurrences

- Line 46: this function takes 4 arguments but 3 arguments were supplied
- Line 53: this function takes 4 arguments but 3 arguments were supplied
- Line 60: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\analyze.rs`: 2 occurrences

- Line 46: this function takes 4 arguments but 3 arguments were supplied
- Line 55: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\create_space.rs`: 2 occurrences

- Line 73: this function takes 4 arguments but 3 arguments were supplied
- Line 86: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\tag\drop_tag.rs`: 2 occurrences

- Line 26: this function takes 4 arguments but 3 arguments were supplied
- Line 41: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\edge\drop_edge.rs`: 2 occurrences

- Line 26: this function takes 4 arguments but 3 arguments were supplied
- Line 41: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\drop_space.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 34: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\show_tag_index_status.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 38: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\tag\create_tag.rs`: 2 occurrences

- Line 81: this function takes 4 arguments but 3 arguments were supplied
- Line 90: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\tests.rs`: 2 occurrences

- Line 305: this function takes 4 arguments but 3 arguments were supplied
- Line 329: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\edge\create_edge.rs`: 2 occurrences

- Line 81: this function takes 4 arguments but 3 arguments were supplied
- Line 94: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\create_user.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 33: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\show_edge_index_status.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 38: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\drop_user.rs`: 2 occurrences

- Line 24: this function takes 4 arguments but 3 arguments were supplied
- Line 32: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\edge\alter_edge.rs`: 1 occurrences

- Line 89: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\show_spaces.rs`: 1 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\edge\desc_edge.rs`: 1 occurrences

- Line 42: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\clear_space.rs`: 1 occurrences

- Line 23: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\tag\show_tags.rs`: 1 occurrences

- Line 26: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\desc_space.rs`: 1 occurrences

- Line 48: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\switch_space.rs`: 1 occurrences

- Line 23: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\change_password.rs`: 1 occurrences

- Line 32: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\base\executor_base.rs`: 1 occurrences

- Line 261: this function takes 3 arguments but 2 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\algorithms\multi_shortest_path.rs`: 1 occurrences

- Line 88: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\algorithms\subgraph_executor.rs`: 1 occurrences

- Line 184: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 77: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\grant_role.rs`: 1 occurrences

- Line 32: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\result_processing\transformations\unwind.rs`: 1 occurrences

- Line 381: this function takes 1 argument but 0 arguments were supplied

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 76: this function takes 4 arguments but 5 arguments were supplied

#### `src\query\executor\admin\space\alter_space.rs`: 1 occurrences

- Line 35: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\tag\desc_tag.rs`: 1 occurrences

- Line 41: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 447: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\tag\alter_tag.rs`: 1 occurrences

- Line 89: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 399: this function takes 1 argument but 0 arguments were supplied

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 321: this function takes 8 arguments but 7 arguments were supplied

#### `src\query\executor\admin\user\alter_user.rs`: 1 occurrences

- Line 24: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\edge\show_edges.rs`: 1 occurrences

- Line 26: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\revoke_role.rs`: 1 occurrences

- Line 24: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\base\result_processor.rs`: 1 occurrences

- Line 255: this function takes 4 arguments but 3 arguments were supplied

### error[E0433]: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

**Total Occurrences**: 78  
**Unique Files**: 40

#### `src\query\planner\rewrite\merge\collapse_project.rs`: 8 occurrences

- Line 86: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 96: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 102: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- ... 5 more occurrences in this file

#### `src\query\validator\helpers\variable_checker.rs`: 5 occurrences

- Line 332: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 340: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 356: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- ... 2 more occurrences in this file

#### `src\query\validator\helpers\expression_checker.rs`: 4 occurrences

- Line 574: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 591: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 604: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\filter.rs`: 4 occurrences

- Line 189: failed to resolve: use of undeclared type `EvalContext`: use of undeclared type `EvalContext`
- Line 215: failed to resolve: use of undeclared type `EvalContext`: use of undeclared type `EvalContext`
- Line 244: failed to resolve: use of undeclared type `EvalContext`: use of undeclared type `EvalContext`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 4 occurrences

- Line 574: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 591: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 604: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 3 occurrences

- Line 33: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 100: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 240: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\executor\graph_query_executor.rs`: 3 occurrences

- Line 714: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 727: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 750: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\strategies\helpers\type_checker.rs`: 3 occurrences

- Line 619: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 631: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 643: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\planner\rewrite\projection_pushdown\projection_pushdown.rs`: 3 occurrences

- Line 234: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 293: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 325: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\planner\rewrite\expression_utils.rs`: 3 occurrences

- Line 442: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 504: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 539: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\helpers\type_checker.rs`: 3 occurrences

- Line 619: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 631: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 643: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\strategies\alias_strategy.rs`: 2 occurrences

- Line 338: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 349: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\optimizer\strategy\aggregate_strategy.rs`: 2 occurrences

- Line 167: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 179: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\planner\rewrite\elimination\remove_append_vertices_below_join.rs`: 2 occurrences

- Line 178: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 187: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\executor\result_processing\projection.rs`: 2 occurrences

- Line 240: failed to resolve: use of undeclared type `EvalContext`: use of undeclared type `EvalContext`
- Line 324: failed to resolve: use of undeclared type `EvalContext`: use of undeclared type `EvalContext`

#### `src\query\parser\parser\parse_context.rs`: 2 occurrences

- Line 27: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 44: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\executor\base\execution_context.rs`: 2 occurrences

- Line 40: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`
- Line 76: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 162: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 1 occurrences

- Line 188: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\statements\insert_vertices_validator.rs`: 1 occurrences

- Line 379: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\statements\delete_validator.rs`: 1 occurrences

- Line 607: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 1 occurrences

- Line 158: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\query_context.rs`: 1 occurrences

- Line 81: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\statements\match_validator.rs`: 1 occurrences

- Line 838: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\statements\fetch_edges_validator.rs`: 1 occurrences

- Line 485: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\utility\update_config_validator.rs`: 1 occurrences

- Line 263: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\statements\insert_edges_validator.rs`: 1 occurrences

- Line 417: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\statements\update_validator.rs`: 1 occurrences

- Line 839: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\statements\unwind_validator.rs`: 1 occurrences

- Line 461: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\optimizer\engine.rs`: 1 occurrences

- Line 82: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\statements\set_validator.rs`: 1 occurrences

- Line 527: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 154: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 1 occurrences

- Line 163: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 130: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\executor\admin\index\tests.rs`: 1 occurrences

- Line 17: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\planner\rewrite\context.rs`: 1 occurrences

- Line 58: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\statements\go_validator.rs`: 1 occurrences

- Line 526: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 863: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\statements\remove_validator.rs`: 1 occurrences

- Line 264: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

#### `src\query\validator\statements\fetch_vertices_validator.rs`: 1 occurrences

- Line 394: failed to resolve: use of undeclared type `ExpressionContext`: use of undeclared type `ExpressionContext`

### error[E0412]: cannot find type `ExpressionContext` in this scope: not found in this scope

**Total Occurrences**: 69  
**Unique Files**: 27

#### `src\query\executor\data_access.rs`: 8 occurrences

- Line 27: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 215: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 302: cannot find type `ExpressionContext` in this scope: not found in this scope
- ... 5 more occurrences in this file

#### `src\query\executor\admin\index\tag_index.rs`: 6 occurrences

- Line 61: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 69: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 152: cannot find type `ExpressionContext` in this scope: not found in this scope
- ... 3 more occurrences in this file

#### `src\query\executor\admin\index\edge_index.rs`: 6 occurrences

- Line 61: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 69: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 152: cannot find type `ExpressionContext` in this scope: not found in this scope
- ... 3 more occurrences in this file

#### `src\query\query_context.rs`: 4 occurrences

- Line 65: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 88: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 242: cannot find type `ExpressionContext` in this scope: not found in this scope
- ... 1 more occurrences in this file

#### `src\query\executor\data_modification.rs`: 4 occurrences

- Line 39: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 48: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 62: cannot find type `ExpressionContext` in this scope: not found in this scope
- ... 1 more occurrences in this file

#### `src\query\parser\parser\parse_context.rs`: 4 occurrences

- Line 20: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 58: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 62: cannot find type `ExpressionContext` in this scope: not found in this scope
- ... 1 more occurrences in this file

#### `src\query\executor\special_executors.rs`: 3 occurrences

- Line 21: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 130: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 209: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\optimizer\engine.rs`: 3 occurrences

- Line 52: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 93: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 195: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\base\execution_context.rs`: 3 occurrences

- Line 21: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 26: cannot find type `ExpressionContext` in this scope
- Line 65: cannot find type `ExpressionContext` in this scope

#### `src\query\planner\rewrite\context.rs`: 3 occurrences

- Line 28: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 63: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 115: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\admin\index\rebuild_index.rs`: 2 occurrences

- Line 24: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 104: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\data_processing\set_operations\minus.rs`: 2 occurrences

- Line 32: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 161: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\optimizer\strategy\aggregate_strategy.rs`: 2 occurrences

- Line 97: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 187: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\base\executor_base.rs`: 2 occurrences

- Line 109: cannot find type `ExpressionContext` in this scope
- Line 122: cannot find type `ExpressionContext` in this scope

#### `src\query\executor\search_executors.rs`: 2 occurrences

- Line 55: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 544: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 2 occurrences

- Line 32: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 129: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\planner\rewrite\expression_utils.rs`: 2 occurrences

- Line 52: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 78: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 2 occurrences

- Line 32: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 153: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 1 occurrences

- Line 142: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\admin\index\tests.rs`: 1 occurrences

- Line 16: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\data_processing\set_operations\base.rs`: 1 occurrences

- Line 35: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\result_processing\transformations\unwind.rs`: 1 occurrences

- Line 40: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 32: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 1 occurrences

- Line 58: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 50: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\data_processing\graph_traversal\expand.rs`: 1 occurrences

- Line 59: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 52: cannot find type `ExpressionContext` in this scope: not found in this scope

### error[E0432]: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

**Total Occurrences**: 21  
**Unique Files**: 19

#### `src\query\validator\clauses\yield_validator.rs`: 2 occurrences

- Line 354: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`
- Line 454: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\validator\clauses\order_by_validator.rs`: 2 occurrences

- Line 612: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`
- Line 641: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\validator\strategies\pagination_strategy.rs`: 1 occurrences

- Line 174: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 270: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\validator\expression_analyzer.rs`: 1 occurrences

- Line 723: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\core\types\expression\serializable.rs`: 1 occurrences

- Line 8: unresolved import `super::context::ExpressionContext`: no `ExpressionContext` in `core::types::expression::context`

#### `src\query\planner\plan\core\nodes\data_processing_node.rs`: 1 occurrences

- Line 616: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\core\types\mod.rs`: 1 occurrences

- Line 65: unresolved import `self::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 669: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\validator\strategies\helpers\variable_checker.rs`: 1 occurrences

- Line 299: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 9: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\planner\plan\core\nodes\insert_nodes.rs`: 1 occurrences

- Line 138: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\core\types\expression\utils.rs`: 1 occurrences

- Line 567: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\validator\clauses\return_validator.rs`: 1 occurrences

- Line 375: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 7: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\validator\strategies\clause_strategy.rs`: 1 occurrences

- Line 6: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\validator\clauses\with_validator.rs`: 1 occurrences

- Line 408: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 1 occurrences

- Line 32: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\parser\parser\parser.rs`: 1 occurrences

- Line 3: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

### error[E0599]: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

**Total Occurrences**: 13  
**Unique Files**: 4

#### `src\query\executor\result_processing\projection.rs`: 5 occurrences

- Line 76: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 146: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 190: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- ... 2 more occurrences in this file

#### `src\query\executor\data_access.rs`: 3 occurrences

- Line 164: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 377: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 1015: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 3 occurrences

- Line 123: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 148: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 171: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

#### `src\query\executor\result_processing\filter.rs`: 2 occurrences

- Line 142: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope
- Line 159: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

### error[E0308]: mismatched types: expected `&ContextualExpression`, found `&Expression`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\optimizer\strategy\materialization.rs`: 2 occurrences

- Line 483: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 491: mismatched types: expected `&ContextualExpression`, found `&Expression`

## Detailed Warning Categorization

### warning: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

**Total Occurrences**: 65  
**Unique Files**: 58

#### `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 3 occurrences

- Line 28: unused import: `crate::core::types::expression::ExpressionAnalysisContext`
- Line 95: unused import: `crate::core::types::expression::ExpressionAnalysisContext`
- Line 235: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\planner\rewrite\projection_pushdown\projection_pushdown.rs`: 3 occurrences

- Line 222: unused import: `crate::core::types::expression::ExpressionAnalysisContext`
- Line 281: unused import: `crate::core::types::expression::ExpressionAnalysisContext`
- Line 318: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\planner\rewrite\merge\collapse_project.rs`: 2 occurrences

- Line 5: unused import: `crate::core::types::expression::ExpressionAnalysisContext`
- Line 282: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\optimizer\analysis\expression.rs`: 2 occurrences

- Line 296: unused variable: `func`: help: if this is intentional, prefix it with an underscore: `_func`
- Line 392: unused variable: `func`: help: if this is intentional, prefix it with an underscore: `_func`

#### `src\query\executor\result_processing\projection.rs`: 2 occurrences

- Line 12: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`
- Line 410: unused import: `Expression`

#### `src\query\optimizer\strategy\aggregate_strategy.rs`: 1 occurrences

- Line 23: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\optimizer\strategy\materialization.rs`: 1 occurrences

- Line 33: unused import: `ReferenceCountAnalysis`

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 1 occurrences

- Line 180: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 1 occurrences

- Line 22: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\validator\statements\insert_edges_validator.rs`: 1 occurrences

- Line 409: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 6: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\admin\index\rebuild_index.rs`: 1 occurrences

- Line 8: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 9: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\validator\statements\fetch_edges_validator.rs`: 1 occurrences

- Line 478: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\validator\utility\update_config_validator.rs`: 1 occurrences

- Line 215: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\validator\helpers\expression_checker.rs`: 1 occurrences

- Line 6: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\executor\base\executor_base.rs`: 1 occurrences

- Line 8: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\planner\plan\core\nodes\data_processing_node.rs`: 1 occurrences

- Line 549: unused doc comment

#### `src\query\planner\rewrite\expression_utils.rs`: 1 occurrences

- Line 19: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\validator\strategies\helpers\type_checker.rs`: 1 occurrences

- Line 611: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\validator\statements\set_validator.rs`: 1 occurrences

- Line 521: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\validator\helpers\variable_checker.rs`: 1 occurrences

- Line 6: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\planner\plan\core\nodes\project_node.rs`: 1 occurrences

- Line 80: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 8: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 9: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\query_context.rs`: 1 occurrences

- Line 8: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\executor\result_processing\transformations\unwind.rs`: 1 occurrences

- Line 9: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 1 occurrences

- Line 5: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 8: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\planner\rewrite\context.rs`: 1 occurrences

- Line 11: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\validator\statements\remove_validator.rs`: 1 occurrences

- Line 259: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\validator\statements\match_validator.rs`: 1 occurrences

- Line 832: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 5: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\admin\index\tests.rs`: 1 occurrences

- Line 3: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 1 occurrences

- Line 155: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\optimizer\engine.rs`: 1 occurrences

- Line 38: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\special_executors.rs`: 1 occurrences

- Line 5: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\planner\rewrite\elimination\remove_append_vertices_below_join.rs`: 1 occurrences

- Line 36: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\executor\admin\index\edge_index.rs`: 1 occurrences

- Line 9: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\validator\strategies\alias_strategy.rs`: 1 occurrences

- Line 6: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\validator\statements\delete_validator.rs`: 1 occurrences

- Line 600: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\validator\statements\update_validator.rs`: 1 occurrences

- Line 832: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 1 occurrences

- Line 6: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\executor\base\execution_context.rs`: 1 occurrences

- Line 8: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 1 occurrences

- Line 150: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\executor\data_processing\set_operations\base.rs`: 1 occurrences

- Line 10: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\planner\rewrite\visitor.rs`: 1 occurrences

- Line 62: unused imports: `InsertEdgesNode` and `InsertVerticesNode`

#### `src\query\validator\statements\fetch_vertices_validator.rs`: 1 occurrences

- Line 386: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\validator\statements\insert_vertices_validator.rs`: 1 occurrences

- Line 369: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\admin\index\tag_index.rs`: 1 occurrences

- Line 9: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 9: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 8: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\validator\statements\go_validator.rs`: 1 occurrences

- Line 516: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\parser\parser\parse_context.rs`: 1 occurrences

- Line 3: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\validator\helpers\type_checker.rs`: 1 occurrences

- Line 611: unused import: `crate::core::types::expression::ExpressionAnalysisContext`

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 1 occurrences

- Line 582: unused variable: `optimizer`: help: if this is intentional, prefix it with an underscore: `_optimizer`

#### `src\query\executor\data_processing\graph_traversal\expand.rs`: 1 occurrences

- Line 6: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\validator\statements\unwind_validator.rs`: 1 occurrences

- Line 455: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

