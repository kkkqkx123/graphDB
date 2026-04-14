# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 329
- **Total Warnings**: 5
- **Total Issues**: 334
- **Unique Error Patterns**: 14
- **Unique Warning Patterns**: 2
- **Files with Issues**: 26

## Error Statistics

**Total Errors**: 329

### Error Type Breakdown

- **error[E0599]**: 299 errors
- **error[E0282]**: 21 errors
- **error[E0308]**: 3 errors
- **error[E0425]**: 2 errors
- **error[E0433]**: 2 errors
- **error[E0614]**: 2 errors

### Files with Errors (Top 10)

- `src\query\executor\data_processing\set_operations\minus.rs`: 39 errors
- `src\query\executor\data_processing\join\cross_join.rs`: 35 errors
- `src\query\executor\data_processing\set_operations\intersect.rs`: 34 errors
- `src\query\executor\data_processing\set_operations\union_all.rs`: 29 errors
- `src\query\executor\data_processing\join\inner_join.rs`: 28 errors
- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 26 errors
- `src\query\executor\data_processing\join\left_join.rs`: 18 errors
- `src\query\executor\result_processing\transformations\pattern_apply.rs`: 18 errors
- `src\query\executor\result_processing\topn.rs`: 14 errors
- `src\query\executor\result_processing\transformations\unwind.rs`: 13 errors

## Warning Statistics

**Total Warnings**: 5

### Warning Type Breakdown

- **warning**: 5 warnings

### Files with Warnings (Top 10)

- `src\query\executor\data_processing\set_operations\union.rs`: 1 warnings
- `src\query\executor\data_processing\set_operations\intersect.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 warnings
- `src\query\executor\data_processing\set_operations\union_all.rs`: 1 warnings
- `src\query\executor\data_processing\set_operations\minus.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`

**Total Occurrences**: 299  
**Unique Files**: 23

#### `src\query\executor\data_processing\set_operations\minus.rs`: 39 occurrences

- Line 195: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 195: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`
- Line 199: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 36 more occurrences in this file

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 34 occurrences

- Line 186: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 186: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`
- Line 190: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 31 more occurrences in this file

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 29 occurrences

- Line 160: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 160: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`
- Line 164: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 26 more occurrences in this file

#### `src\query\executor\data_processing\join\cross_join.rs`: 29 occurrences

- Line 151: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 162: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 177: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 26 more occurrences in this file

#### `src\query\executor\data_processing\join\inner_join.rs`: 28 occurrences

- Line 408: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 408: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`
- Line 423: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 25 more occurrences in this file

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 22 occurrences

- Line 400: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 401: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 405: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 19 more occurrences in this file

#### `src\query\executor\data_processing\join\left_join.rs`: 18 occurrences

- Line 248: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 248: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`
- Line 264: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 15 more occurrences in this file

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 16 occurrences

- Line 24: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 25: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 29: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 13 more occurrences in this file

#### `src\query\executor\result_processing\topn.rs`: 14 occurrences

- Line 219: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 228: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 237: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 11 more occurrences in this file

#### `src\query\executor\logic\loops.rs`: 11 occurrences

- Line 159: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 163: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 167: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 8 more occurrences in this file

#### `src\query\executor\result_processing\dedup.rs`: 11 occurrences

- Line 89: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 97: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 106: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 8 more occurrences in this file

#### `src\query\executor\result_processing\transformations\unwind.rs`: 9 occurrences

- Line 107: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 141: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 170: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 6 more occurrences in this file

#### `src\query\executor\result_processing\sample.rs`: 7 occurrences

- Line 92: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 100: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 109: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 4 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 6 occurrences

- Line 567: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 571: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 575: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 3 more occurrences in this file

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 5 occurrences

- Line 132: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 157: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 180: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 2 more occurrences in this file

#### `src\query\executor\pipeline_executors.rs`: 4 occurrences

- Line 64: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 337: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 358: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\materialize.rs`: 4 occurrences

- Line 130: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 131: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 132: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\limit.rs`: 4 occurrences

- Line 81: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 85: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 89: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\aggregation.rs`: 3 occurrences

- Line 277: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 282: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 287: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`

#### `src\query\executor\result_processing\transformations\assign.rs`: 3 occurrences

- Line 75: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`
- Line 84: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 90: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`

#### `src\core\value\value_compare.rs`: 1 occurrences

- Line 268: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`

#### `src\query\optimizer\cost\calculator.rs`: 1 occurrences

- Line 715: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`

#### `src\query\executor\expression\functions\signature.rs`: 1 occurrences

- Line 74: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`

### error[E0282]: type annotations needed: cannot infer type

**Total Occurrences**: 21  
**Unique Files**: 7

#### `src\query\executor\data_processing\join\cross_join.rs`: 6 occurrences

- Line 155: type annotations needed
- Line 166: type annotations needed
- Line 275: type annotations needed
- ... 3 more occurrences in this file

#### `src\query\executor\result_processing\transformations\unwind.rs`: 4 occurrences

- Line 111: type annotations needed: cannot infer type
- Line 144: type annotations needed: cannot infer type
- Line 173: type annotations needed: cannot infer type
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 4 occurrences

- Line 403: type annotations needed
- Line 407: type annotations needed
- Line 422: type annotations needed
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 3 occurrences

- Line 135: type annotations needed: cannot infer type
- Line 159: type annotations needed: cannot infer type
- Line 182: type annotations needed: cannot infer type

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 2 occurrences

- Line 27: type annotations needed
- Line 29: type annotations needed

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 593: type annotations needed: cannot infer type

#### `src\query\executor\result_processing\transformations\assign.rs`: 1 occurrences

- Line 80: type annotations needed

### error[E0308]: mismatched types: expected `&i64`, found `i64`

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\query\executor\data_access\edge.rs`: 2 occurrences

- Line 43: mismatched types: expected `Edge`, found `Box<Edge>`
- Line 140: mismatched types: expected `Edge`, found `Box<Edge>`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 636: mismatched types: expected `&i64`, found `i64`

### error[E0425]: cannot find type `Value` in this scope: not found in this scope

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 102: cannot find type `Value` in this scope: not found in this scope
- Line 111: cannot find type `Value` in this scope: not found in this scope

### error[E0433]: failed to resolve: use of undeclared type `Value`: use of undeclared type `Value`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 104: failed to resolve: use of undeclared type `Value`: use of undeclared type `Value`
- Line 113: failed to resolve: use of undeclared type `Value`: use of undeclared type `Value`

### error[E0614]: type `i64` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 550: type `i64` cannot be dereferenced: can't be dereferenced
- Line 550: type `i64` cannot be dereferenced: can't be dereferenced

## Detailed Warning Categorization

### warning: unused import: `crate::core::value::list::List`

**Total Occurrences**: 5  
**Unique Files**: 5

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 occurrences

- Line 5: unused import: `crate::core::value::list::List`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Value`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 10: unused import: `crate::core::Value`

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 10: unused import: `crate::core::Value`

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Value`

