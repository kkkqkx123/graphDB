# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 64
- **Total Warnings**: 6
- **Total Issues**: 70
- **Unique Error Patterns**: 13
- **Unique Warning Patterns**: 3
- **Files with Issues**: 29

## Error Statistics

**Total Errors**: 64

### Error Type Breakdown

- **error[E0061]**: 46 errors
- **error[E0308]**: 14 errors
- **error[E0689]**: 2 errors
- **error[E0107]**: 1 errors
- **error[E0425]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\data_processing\graph_traversal\tests.rs`: 10 errors
- `src\query\executor\admin\index\tests.rs`: 6 errors
- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 5 errors
- `src\query\executor\factory\builders\join_builder.rs`: 5 errors
- `src\query\executor\data_processing\join\inner_join.rs`: 5 errors
- `src\query\executor\factory\builders\data_access_builder.rs`: 4 errors
- `src\query\executor\result_processing\transformations\pattern_apply.rs`: 4 errors
- `src\query\executor\factory\builders\traversal_builder.rs`: 4 errors
- `src\query\executor\factory\builders\transformation_builder.rs`: 3 errors
- `src\query\executor\data_processing\join\left_join.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 6

### Warning Type Breakdown

- **warning**: 6 warnings

### Files with Warnings (Top 10)

- `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 2 warnings
- `src\query\validator\strategies\clause_strategy.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\algorithms\bfs_shortest.rs`: 1 warnings
- `src\query\executor\data_access\search.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 warnings

## Detailed Error Categorization

### error[E0061]: this function takes 4 arguments but 8 arguments were supplied

**Total Occurrences**: 46  
**Unique Files**: 18

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 10 occurrences

- Line 147: this function takes 5 arguments but 9 arguments were supplied
- Line 260: this function takes 5 arguments but 9 arguments were supplied
- Line 283: this function takes 5 arguments but 9 arguments were supplied
- ... 7 more occurrences in this file

#### `src\query\executor\admin\index\tests.rs`: 6 occurrences

- Line 25: this function takes 1 argument but 8 arguments were supplied
- Line 51: this function takes 1 argument but 8 arguments were supplied
- Line 166: this function takes 1 argument but 8 arguments were supplied
- ... 3 more occurrences in this file

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 5 occurrences

- Line 572: this function takes 4 arguments but 8 arguments were supplied
- Line 615: this function takes 4 arguments but 8 arguments were supplied
- Line 679: this function takes 4 arguments but 8 arguments were supplied
- ... 2 more occurrences in this file

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 4 occurrences

- Line 370: this function takes 4 arguments but 8 arguments were supplied
- Line 403: this function takes 4 arguments but 8 arguments were supplied
- Line 436: this function takes 4 arguments but 8 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\factory\builders\traversal_builder.rs`: 4 occurrences

- Line 104: this function takes 2 arguments but 8 arguments were supplied
- Line 130: this function takes 3 arguments but 9 arguments were supplied
- Line 155: this function takes 2 arguments but 11 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\factory\builders\transformation_builder.rs`: 3 occurrences

- Line 118: this function takes 2 arguments but 9 arguments were supplied
- Line 159: this function takes 2 arguments but 8 arguments were supplied
- Line 194: this function takes 2 arguments but 8 arguments were supplied

#### `src\query\executor\factory\builders\admin_builder.rs`: 2 occurrences

- Line 326: this function takes 1 argument but 8 arguments were supplied
- Line 423: this function takes 1 argument but 8 arguments were supplied

#### `src\query\executor\factory\builders\data_access_builder.rs`: 2 occurrences

- Line 128: this function takes 2 arguments but 12 arguments were supplied
- Line 178: this function takes 2 arguments but 12 arguments were supplied

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 43: this function takes 4 arguments but 8 arguments were supplied

#### `src\storage\index\edge_index_manager.rs`: 1 occurrences

- Line 330: this function takes 1 argument but 8 arguments were supplied

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 374: this function takes 4 arguments but 9 arguments were supplied

#### `src\query\executor\data_modification\index_ops.rs`: 1 occurrences

- Line 108: this function takes 1 argument but 8 arguments were supplied

#### `src\storage\index\index_updater.rs`: 1 occurrences

- Line 442: this function takes 1 argument but 8 arguments were supplied

#### `src\storage\index\vertex_index_manager.rs`: 1 occurrences

- Line 307: this function takes 1 argument but 8 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\algorithms\multi_shortest_path.rs`: 1 occurrences

- Line 664: this function takes 2 arguments but 8 arguments were supplied

#### `src\query\executor\admin\index\tag_index.rs`: 1 occurrences

- Line 39: this function takes 1 argument but 8 arguments were supplied

#### `src\storage\index\index_data_manager.rs`: 1 occurrences

- Line 341: this function takes 1 argument but 8 arguments were supplied

#### `src\query\executor\admin\index\edge_index.rs`: 1 occurrences

- Line 39: this function takes 1 argument but 8 arguments were supplied

### error[E0308]: arguments to this function are incorrect

**Total Occurrences**: 14  
**Unique Files**: 4

#### `src\query\executor\factory\builders\join_builder.rs`: 5 occurrences

- Line 60: arguments to this function are incorrect
- Line 84: arguments to this function are incorrect
- Line 108: arguments to this function are incorrect
- ... 2 more occurrences in this file

#### `src\query\executor\data_processing\join\inner_join.rs`: 5 occurrences

- Line 366: arguments to this function are incorrect
- Line 462: arguments to this function are incorrect
- Line 547: arguments to this function are incorrect
- ... 2 more occurrences in this file

#### `src\query\executor\data_processing\join\left_join.rs`: 3 occurrences

- Line 329: arguments to this function are incorrect
- Line 405: arguments to this function are incorrect
- Line 497: arguments to this function are incorrect

#### `src\query\executor\logic\loops.rs`: 1 occurrences

- Line 751: arguments to this function are incorrect

### error[E0689]: can't call method `wrapping_mul` on ambiguous numeric type `{integer}`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\factory\builders\data_access_builder.rs`: 2 occurrences

- Line 134: can't call method `wrapping_mul` on ambiguous numeric type `{integer}`
- Line 137: can't call method `wrapping_mul` on ambiguous numeric type `{integer}`

### error[E0107]: missing generics for trait `executor_base::Executor`: expected 1 generic argument

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\base\config.rs`: 1 occurrences

- Line 105: missing generics for trait `executor_base::Executor`: expected 1 generic argument

### error[E0425]: cannot find value `max_steps` in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\data_processing\graph_traversal\algorithms\multi_shortest_path.rs`: 1 occurrences

- Line 95: cannot find value `max_steps` in this scope

## Detailed Warning Categorization

### warning: unused import: `crate::query::validator::context::ExpressionAnalysisContext`

**Total Occurrences**: 6  
**Unique Files**: 5

#### `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 2 occurrences

- Line 27: unused import: `crate::query::validator::context::ExpressionAnalysisContext`
- Line 29: unused import: `parking_lot::Mutex`

#### `src\query\executor\data_processing\graph_traversal\algorithms\bfs_shortest.rs`: 1 occurrences

- Line 12: unused import: `crate::query::validator::context::ExpressionAnalysisContext`

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 13: unused import: `crate::query::validator::context::ExpressionAnalysisContext`

#### `src\query\validator\strategies\clause_strategy.rs`: 1 occurrences

- Line 467: variable does not need to be mutable

#### `src\query\executor\data_access\search.rs`: 1 occurrences

- Line 12: unused import: `crate::query::validator::context::ExpressionAnalysisContext`

