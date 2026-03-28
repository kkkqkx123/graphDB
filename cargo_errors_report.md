# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 178
- **Total Warnings**: 0
- **Total Issues**: 178
- **Unique Error Patterns**: 23
- **Unique Warning Patterns**: 0
- **Files with Issues**: 46

## Error Statistics

**Total Errors**: 178

### Error Type Breakdown

- **error[E0282]**: 80 errors
- **error[E0433]**: 55 errors
- **error[E0308]**: 12 errors
- **error[E0599]**: 10 errors
- **error[E0277]**: 8 errors
- **error[E0432]**: 8 errors
- **error[E0614]**: 5 errors

### Files with Errors (Top 10)

- `src\core\value\list.rs`: 32 errors
- `src\core\value\dataset.rs`: 26 errors
- `src\query\executor\result_processing\sort.rs`: 15 errors
- `src\query\executor\expression\evaluator\operations.rs`: 9 errors
- `src\core\vertex_edge_path.rs`: 7 errors
- `src\query\executor\data_processing\join\base_join.rs`: 6 errors
- `src\query\executor\expression\evaluator\functions.rs`: 5 errors
- `src\query\executor\result_processing\dedup.rs`: 5 errors
- `src\query\executor\data_processing\join\left_join.rs`: 5 errors
- `src\query\executor\result_processing\topn.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0282]: type annotations needed

**Total Occurrences**: 80  
**Unique Files**: 29

#### `src\query\executor\result_processing\sort.rs`: 14 occurrences

- Line 261: type annotations needed
- Line 421: type annotations needed: cannot infer type
- Line 431: type annotations needed: cannot infer type
- ... 11 more occurrences in this file

#### `src\core\value\dataset.rs`: 10 occurrences

- Line 69: type annotations needed
- Line 128: type annotations needed: cannot infer type
- Line 130: type annotations needed: cannot infer type
- ... 7 more occurrences in this file

#### `src\query\executor\expression\evaluator\operations.rs`: 9 occurrences

- Line 210: type annotations needed
- Line 226: type annotations needed
- Line 243: type annotations needed
- ... 6 more occurrences in this file

#### `src\query\executor\data_processing\join\base_join.rs`: 6 occurrences

- Line 181: type annotations needed: cannot infer type
- Line 205: type annotations needed: cannot infer type
- Line 229: type annotations needed: cannot infer type
- ... 3 more occurrences in this file

#### `src\query\executor\data_processing\join\cross_join.rs`: 3 occurrences

- Line 78: type annotations needed: cannot infer type
- Line 79: type annotations needed: cannot infer type
- Line 170: type annotations needed: cannot infer type

#### `src\query\executor\data_processing\join\left_join.rs`: 3 occurrences

- Line 143: type annotations needed: cannot infer type of the type parameter `Q` declared on the method `contains`
- Line 224: type annotations needed: cannot infer type of the type parameter `Q` declared on the method `contains`
- Line 256: type annotations needed: cannot infer type

#### `src\query\executor\result_processing\aggregation.rs`: 3 occurrences

- Line 341: type annotations needed: cannot infer type
- Line 471: type annotations needed: cannot infer type
- Line 943: type annotations needed: cannot infer type

#### `src\core\value\value_convert.rs`: 3 occurrences

- Line 183: type annotations needed
- Line 183: type annotations needed
- Line 185: type annotations needed

#### `src\query\executor\result_processing\filter.rs`: 3 occurrences

- Line 141: type annotations needed: cannot infer type
- Line 185: type annotations needed
- Line 188: type annotations needed

#### `src\api\core\query_api.rs`: 2 occurrences

- Line 97: type annotations needed: cannot infer type
- Line 98: type annotations needed: cannot infer type

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 155: type annotations needed: cannot infer type
- Line 174: type annotations needed: cannot infer type

#### `src\query\planning\statements\clauses\with_clause_planner.rs`: 2 occurrences

- Line 315: type annotations needed: cannot infer type
- Line 316: type annotations needed: cannot infer type

#### `src\query\executor\result_processing\projection.rs`: 2 occurrences

- Line 143: type annotations needed
- Line 146: type annotations needed

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 196: type annotations needed: cannot infer type
- Line 213: type annotations needed: cannot infer type

#### `src\query\executor\data_processing\join\hash_table.rs`: 2 occurrences

- Line 135: type annotations needed: cannot infer type
- Line 166: type annotations needed: cannot infer type

#### `src\api\server\http\handlers\query.rs`: 1 occurrences

- Line 109: type annotations needed

#### `src\api\server\http\handlers\stream.rs`: 1 occurrences

- Line 171: type annotations needed

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 88: type annotations needed

#### `src\query\executor\expression\evaluator\functions.rs`: 1 occurrences

- Line 128: type annotations needed

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 78: type annotations needed

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 300: type annotations needed

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 102: type annotations needed

#### `src\query\executor\result_processing\transformations\unwind.rs`: 1 occurrences

- Line 251: type annotations needed: cannot infer type

#### `src\core\value\memory.rs`: 1 occurrences

- Line 49: type annotations needed

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 110: type annotations needed

#### `src\query\executor\result_processing\transformations\assign.rs`: 1 occurrences

- Line 80: type annotations needed

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 490: type annotations needed

#### `src\query\executor\admin\query_management\show_stats.rs`: 1 occurrences

- Line 190: type annotations needed

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 297: type annotations needed

### error[E0433]: failed to resolve: could not find `utils` in `expr`: could not find `utils` in `expr`

**Total Occurrences**: 55  
**Unique Files**: 5

#### `src\core\value\list.rs`: 32 occurrences

- Line 12: failed to resolve: could not find `types` in `super`: could not find `types` in `super`
- Line 28: failed to resolve: could not find `types` in `super`: could not find `types` in `super`
- Line 32: failed to resolve: could not find `types` in `super`: could not find `types` in `super`
- ... 29 more occurrences in this file

#### `src\core\value\dataset.rs`: 16 occurrences

- Line 14: failed to resolve: could not find `types` in `super`: could not find `types` in `super`
- Line 40: failed to resolve: could not find `types` in `super`: could not find `types` in `super`
- Line 65: failed to resolve: could not find `types` in `super`: could not find `types` in `super`
- ... 13 more occurrences in this file

#### `src\core\types\expr\expression_utils.rs`: 4 occurrences

- Line 146: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- Line 159: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- Line 172: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- ... 1 more occurrences in this file

#### `src\query\planning\rewrite\expression_utils.rs`: 2 occurrences

- Line 460: failed to resolve: could not find `common_utils` in `expr`: could not find `common_utils` in `expr`
- Line 467: failed to resolve: could not find `common_utils` in `expr`: could not find `common_utils` in `expr`

#### `src\query\validator\strategies\expression_strategy_test.rs`: 1 occurrences

- Line 6: failed to resolve: could not find `utils` in `expr`: could not find `utils` in `expr`

### error[E0308]: mismatched types: expected `Vec<Value>`, found `&[Value]`

**Total Occurrences**: 12  
**Unique Files**: 6

#### `src\core\types\expr\analysis_utils.rs`: 3 occurrences

- Line 503: mismatched types: expected `AggregateFunction`, found enum constructor
- Line 518: mismatched types: expected `AggregateFunction`, found enum constructor
- Line 523: mismatched types: expected `AggregateFunction`, found enum constructor

#### `src\query\executor\result_processing\topn.rs`: 2 occurrences

- Line 423: mismatched types: expected `Vec<Value>`, found `&[Value]`
- Line 466: mismatched types: expected `Vec<Value>`, found `&[Value]`

#### `src\query\executor\data_processing\join\inner_join.rs`: 2 occurrences

- Line 135: mismatched types: expected `Vec<Value>`, found `&[Value]`
- Line 237: mismatched types: expected `Vec<Value>`, found `&[Value]`

#### `src\query\executor\data_processing\join\left_join.rs`: 2 occurrences

- Line 130: mismatched types: expected `Vec<Value>`, found `&[Value]`
- Line 211: mismatched types: expected `Vec<Value>`, found `&[Value]`

#### `src\core\types\expr\group_utils.rs`: 2 occurrences

- Line 273: mismatched types: expected `AggregateFunction`, found enum constructor
- Line 317: mismatched types: expected `AggregateFunction`, found enum constructor

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 460: mismatched types: expected `i64`, found `&{integer}`

### error[E0599]: no method named `extend_from_slice` found for reference `&[core::value::value::Value]` in the current scope

**Total Occurrences**: 10  
**Unique Files**: 3

#### `src\core\vertex_edge_path.rs`: 7 occurrences

- Line 46: no method named `estimated_size` found for reference `&core::value::value::Value` in the current scope: method not found in `&core::value::value::Value`
- Line 296: no method named `estimated_size` found for struct `std::boxed::Box<core::value::value::Value>` in the current scope: method not found in `std::boxed::Box<core::value::value::Value>`
- Line 311: no method named `estimated_size` found for reference `&core::value::value::Value` in the current scope: method not found in `&core::value::value::Value`
- ... 4 more occurrences in this file

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 2 occurrences

- Line 191: no method named `extend_from_slice` found for reference `&[core::value::value::Value]` in the current scope
- Line 203: no method named `extend_from_slice` found for reference `&[core::value::value::Value]` in the current scope

#### `src\core\value\value.rs`: 1 occurrences

- Line 163: no method named `hash` found for reference `&core::value::value::Value` in the current scope: method not found in `&core::value::value::Value`

### error[E0277]: the size for values of type `[core::value::value::Value]` cannot be known at compilation time: doesn't have a size known at compile-time

**Total Occurrences**: 8  
**Unique Files**: 4

#### `src\query\executor\result_processing\dedup.rs`: 3 occurrences

- Line 243: the trait bound `[std::vec::Vec<core::value::value::Value>]: std::default::Default` is not satisfied: the trait `std::default::Default` is not implemented for `[std::vec::Vec<core::value::value::Value>]`
- Line 243: the size for values of type `[std::vec::Vec<core::value::value::Value>]` cannot be known at compilation time: doesn't have a size known at compile-time
- Line 243: the size for values of type `[std::vec::Vec<core::value::value::Value>]` cannot be known at compilation time: doesn't have a size known at compile-time

#### `src\query\executor\result_processing\projection.rs`: 2 occurrences

- Line 129: the size for values of type `[core::value::value::Value]` cannot be known at compilation time: doesn't have a size known at compile-time
- Line 129: the size for values of type `[core::value::value::Value]` cannot be known at compilation time: doesn't have a size known at compile-time

#### `src\query\executor\result_processing\topn.rs`: 2 occurrences

- Line 315: the size for values of type `[core::value::value::Value]` cannot be known at compilation time: doesn't have a size known at compile-time
- Line 315: the size for values of type `[core::value::value::Value]` cannot be known at compilation time: doesn't have a size known at compile-time

#### `src\query\executor\result_processing\sort.rs`: 1 occurrences

- Line 622: the size for values of type `[core::value::value::Value]` cannot be known at compilation time: doesn't have a size known at compile-time

### error[E0432]: unresolved import `crate::core::types::expr::common_utils`: could not find `common_utils` in `expr`

**Total Occurrences**: 8  
**Unique Files**: 8

#### `src\query\planning\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 12: unresolved import `crate::core::types::expr::common_utils`: could not find `common_utils` in `expr`

#### `src\query\parser\ast\stmt.rs`: 1 occurrences

- Line 10: unresolved import `crate::core::types::expr::utils`: could not find `utils` in `expr`

#### `src\query\parser\ast\pattern.rs`: 1 occurrences

- Line 7: unresolved import `crate::core::types::expr::utils`: could not find `utils` in `expr`

#### `src\query\executor\expression\evaluator\expression_evaluator.rs`: 1 occurrences

- Line 6: unresolved import `crate::core::types::expr::utils`: could not find `utils` in `expr`

#### `src\query\planning\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 5: unresolved import `crate::core::types::expr::common_utils`: could not find `common_utils` in `expr`

#### `src\query\validator\strategies\pagination_strategy.rs`: 1 occurrences

- Line 6: unresolved import `crate::core::types::expr::utils`: could not find `utils` in `expr`

#### `src\query\planning\statements\dql\fetch_edges_planner.rs`: 1 occurrences

- Line 4: unresolved import `crate::core::types::expr::common_utils`: could not find `common_utils` in `expr`

#### `src\core\value\memory.rs`: 1 occurrences

- Line 93: unresolved import `super::list`: could not find `list` in `super`

### error[E0614]: type `i64` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 5  
**Unique Files**: 2

#### `src\query\executor\expression\evaluator\functions.rs`: 4 occurrences

- Line 117: type `i64` cannot be dereferenced: can't be dereferenced
- Line 118: type `f64` cannot be dereferenced: can't be dereferenced
- Line 161: type `i64` cannot be dereferenced: can't be dereferenced
- ... 1 more occurrences in this file

#### `src\query\executor\admin\query_management\show_stats.rs`: 1 occurrences

- Line 197: type `i64` cannot be dereferenced: can't be dereferenced

