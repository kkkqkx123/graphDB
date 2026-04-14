# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 115
- **Total Warnings**: 5
- **Total Issues**: 120
- **Unique Error Patterns**: 18
- **Unique Warning Patterns**: 2
- **Files with Issues**: 24

## Error Statistics

**Total Errors**: 115

### Error Type Breakdown

- **error[E0599]**: 89 errors
- **error[E0282]**: 8 errors
- **error[E0277]**: 6 errors
- **error[E0433]**: 4 errors
- **error[E0308]**: 3 errors
- **error[E0425]**: 2 errors
- **error[E0614]**: 2 errors
- **error[E0422]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 18 errors
- `src\query\executor\result_processing\sample.rs`: 13 errors
- `src\query\executor\result_processing\transformations\pattern_apply.rs`: 12 errors
- `src\query\executor\result_processing\topn.rs`: 8 errors
- `src\query\executor\result_processing\transformations\unwind.rs`: 8 errors
- `src\query\executor\result_processing\transformations\append_vertices.rs`: 8 errors
- `src\query\executor\result_processing\dedup.rs`: 8 errors
- `src\query\executor\result_processing\projection.rs`: 7 errors
- `src\query\executor\data_processing\join\left_join.rs`: 6 errors
- `src\query\executor\result_processing\transformations\assign.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 5

### Warning Type Breakdown

- **warning**: 5 warnings

### Files with Warnings (Top 10)

- `src\query\executor\data_processing\set_operations\intersect.rs`: 1 warnings
- `src\query\executor\data_processing\set_operations\union.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 warnings
- `src\query\executor\data_processing\set_operations\union_all.rs`: 1 warnings
- `src\query\executor\data_processing\set_operations\minus.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`

**Total Occurrences**: 89  
**Unique Files**: 18

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 12 occurrences

- Line 589: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 593: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 613: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 9 more occurrences in this file

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 12 occurrences

- Line 363: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 364: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 378: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 9 more occurrences in this file

#### `src\query\executor\result_processing\topn.rs`: 8 occurrences

- Line 1162: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 1166: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 1170: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 5 more occurrences in this file

#### `src\query\executor\result_processing\dedup.rs`: 7 occurrences

- Line 509: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 591: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 600: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 4 more occurrences in this file

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

#### `src\query\executor\data_processing\join\left_join.rs`: 6 occurrences

- Line 515: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 515: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`
- Line 520: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 3 more occurrences in this file

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 5 occurrences

- Line 132: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 157: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 180: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 2 more occurrences in this file

#### `src\query\executor\result_processing\transformations\unwind.rs`: 5 occurrences

- Line 143: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 177: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 206: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 2 more occurrences in this file

#### `src\query\executor\result_processing\limit.rs`: 4 occurrences

- Line 81: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 85: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 89: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 4 occurrences

- Line 240: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 240: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`
- Line 244: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\transformations\assign.rs`: 3 occurrences

- Line 75: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`
- Line 84: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 90: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`

#### `src\query\executor\result_processing\aggregation.rs`: 3 occurrences

- Line 277: no variant or associated item named `Vertices` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 282: no variant or associated item named `Edges` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 287: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`

#### `src\query\executor\data_processing\join\inner_join.rs`: 2 occurrences

- Line 737: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 737: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 2 occurrences

- Line 255: no variant or associated item named `Values` found for enum `query::executor::base::execution_result::ExecutionResult` in the current scope: variant or associated item not found in `query::executor::base::execution_result::ExecutionResult`
- Line 255: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`

#### `src\core\value\value_compare.rs`: 1 occurrences

- Line 268: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`

#### `src\query\optimizer\cost\calculator.rs`: 1 occurrences

- Line 715: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`

#### `src\query\executor\expression\functions\signature.rs`: 1 occurrences

- Line 74: no variant or associated item named `DataSet` found for enum `core::value::value_def::Value` in the current scope: variant or associated item not found in `core::value::value_def::Value`

### error[E0282]: type annotations needed: cannot infer type

**Total Occurrences**: 8  
**Unique Files**: 4

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 3 occurrences

- Line 135: type annotations needed: cannot infer type
- Line 159: type annotations needed: cannot infer type
- Line 182: type annotations needed: cannot infer type

#### `src\query\executor\result_processing\transformations\unwind.rs`: 3 occurrences

- Line 147: type annotations needed: cannot infer type
- Line 180: type annotations needed: cannot infer type
- Line 209: type annotations needed: cannot infer type

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 593: type annotations needed: cannot infer type

#### `src\query\executor\result_processing\transformations\assign.rs`: 1 occurrences

- Line 80: type annotations needed

### error[E0277]: a slice of type `[core::value::value_def::Value]` cannot be built since `[core::value::value_def::Value]` has no definite size: try explicitly collecting into a `Vec<core::value::value_def::Value>`

**Total Occurrences**: 6  
**Unique Files**: 1

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 6 occurrences

- Line 411: a slice of type `[core::value::value_def::Value]` cannot be built since `[core::value::value_def::Value]` has no definite size: try explicitly collecting into a `Vec<core::value::value_def::Value>`
- Line 410: the size for values of type `[core::value::value_def::Value]` cannot be known at compilation time: doesn't have a size known at compile-time
- Line 411: the size for values of type `[core::value::value_def::Value]` cannot be known at compilation time: doesn't have a size known at compile-time
- ... 3 more occurrences in this file

### error[E0433]: failed to resolve: use of undeclared type `DataSet`: use of undeclared type `DataSet`

**Total Occurrences**: 4  
**Unique Files**: 2

#### `src\query\executor\pipeline_executors.rs`: 2 occurrences

- Line 63: failed to resolve: use of undeclared type `DataSet`: use of undeclared type `DataSet`
- Line 358: failed to resolve: use of undeclared type `DataSet`: use of undeclared type `DataSet`

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 104: failed to resolve: use of undeclared type `Value`: use of undeclared type `Value`
- Line 113: failed to resolve: use of undeclared type `Value`: use of undeclared type `Value`

### error[E0308]: mismatched types: expected `Edge`, found `Box<Edge>`

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\query\executor\data_access\edge.rs`: 2 occurrences

- Line 43: mismatched types: expected `Edge`, found `Box<Edge>`
- Line 140: mismatched types: expected `Edge`, found `Box<Edge>`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 609: mismatched types: expected `&i64`, found `i64`

### error[E0425]: cannot find type `Value` in this scope: not found in this scope

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 102: cannot find type `Value` in this scope: not found in this scope
- Line 111: cannot find type `Value` in this scope: not found in this scope

### error[E0614]: type `i64` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 550: type `i64` cannot be dereferenced: can't be dereferenced
- Line 550: type `i64` cannot be dereferenced: can't be dereferenced

### error[E0422]: cannot find struct, variant or union type `DataSet` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\logic\loops.rs`: 1 occurrences

- Line 185: cannot find struct, variant or union type `DataSet` in this scope: not found in this scope

## Detailed Warning Categorization

### warning: unused import: `crate::core::Value`

**Total Occurrences**: 5  
**Unique Files**: 5

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Value`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 10: unused import: `crate::core::Value`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Value`

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 occurrences

- Line 5: unused import: `crate::core::value::list::List`

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 10: unused import: `crate::core::Value`

