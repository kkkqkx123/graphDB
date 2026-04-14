# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 31
- **Total Warnings**: 19
- **Total Issues**: 50
- **Unique Error Patterns**: 7
- **Unique Warning Patterns**: 13
- **Files with Issues**: 15

## Error Statistics

**Total Errors**: 31

### Error Type Breakdown

- **error[E0282]**: 16 errors
- **error[E0599]**: 15 errors

### Files with Errors (Top 10)

- `tests\common\test_scenario.rs`: 18 errors
- `tests\common\validation_helpers.rs`: 8 errors
- `tests\common\query_helpers.rs`: 5 errors

## Warning Statistics

**Total Warnings**: 19

### Warning Type Breakdown

- **warning**: 19 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\filter.rs`: 3 warnings
- `src\query\executor\result_processing\projection.rs`: 3 warnings
- `src\query\executor\result_processing\dedup.rs`: 2 warnings
- `src\query\executor\logic\loops.rs`: 2 warnings
- `src\query\executor\result_processing\sample.rs`: 2 warnings
- `src\query\executor\data_processing\set_operations\minus.rs`: 1 warnings
- `src\query\executor\data_processing\set_operations\union_all.rs`: 1 warnings
- `src\query\executor\result_processing\aggregation.rs`: 1 warnings
- `src\query\executor\data_processing\set_operations\union.rs`: 1 warnings
- `src\query\executor\data_processing\set_operations\intersect.rs`: 1 warnings

## Detailed Error Categorization

### error[E0282]: type annotations needed: cannot infer type

**Total Occurrences**: 16  
**Unique Files**: 2

#### `tests\common\test_scenario.rs`: 11 occurrences

- Line 321: type annotations needed: cannot infer type
- Line 322: type annotations needed: cannot infer type
- Line 330: type annotations needed
- ... 8 more occurrences in this file

#### `tests\common\validation_helpers.rs`: 5 occurrences

- Line 57: type annotations needed: cannot infer type
- Line 57: type annotations needed: cannot infer type
- Line 130: type annotations needed: cannot infer type
- ... 2 more occurrences in this file

### error[E0599]: no variant or associated item named `Result` found for enum `graphdb::query::ExecutionResult` in the current scope: variant or associated item not found in `graphdb::query::ExecutionResult`

**Total Occurrences**: 15  
**Unique Files**: 3

#### `tests\common\test_scenario.rs`: 7 occurrences

- Line 254: no variant or associated item named `Result` found for enum `graphdb::query::ExecutionResult` in the current scope: variant or associated item not found in `graphdb::query::ExecutionResult`
- Line 292: no variant or associated item named `Result` found for enum `graphdb::query::ExecutionResult` in the current scope: variant or associated item not found in `graphdb::query::ExecutionResult`
- Line 313: no variant or associated item named `Result` found for enum `graphdb::query::ExecutionResult` in the current scope: variant or associated item not found in `graphdb::query::ExecutionResult`
- ... 4 more occurrences in this file

#### `tests\common\query_helpers.rs`: 5 occurrences

- Line 53: no variant or associated item named `Count` found for enum `graphdb::query::ExecutionResult` in the current scope: variant or associated item not found in `graphdb::query::ExecutionResult`
- Line 67: no variant or associated item named `Result` found for enum `graphdb::query::ExecutionResult` in the current scope: variant or associated item not found in `graphdb::query::ExecutionResult`
- Line 69: no variant or associated item named `Values` found for enum `graphdb::query::ExecutionResult` in the current scope: variant or associated item not found in `graphdb::query::ExecutionResult`
- ... 2 more occurrences in this file

#### `tests\common\validation_helpers.rs`: 3 occurrences

- Line 47: no variant or associated item named `Result` found for enum `graphdb::query::ExecutionResult` in the current scope: variant or associated item not found in `graphdb::query::ExecutionResult`
- Line 120: no variant or associated item named `Result` found for enum `graphdb::query::ExecutionResult` in the current scope: variant or associated item not found in `graphdb::query::ExecutionResult`
- Line 197: no variant or associated item named `Result` found for enum `graphdb::query::ExecutionResult` in the current scope: variant or associated item not found in `graphdb::query::ExecutionResult`

## Detailed Warning Categorization

### warning: associated functions `cmp_string_list` and `cmp_value_list` are never used

**Total Occurrences**: 19  
**Unique Files**: 12

#### `src\query\executor\result_processing\filter.rs`: 3 occurrences

- Line 25: function `extract_variable_names` is never used
- Line 142: constant `INTERNAL_VARIABLES` is never used
- Line 348: methods `filter_values`, `filter_vertices`, and `filter_edges` are never used

#### `src\query\executor\result_processing\projection.rs`: 3 occurrences

- Line 27: function `extract_variable_names` is never used
- Line 144: constant `INTERNAL_VARIABLES` is never used
- Line 392: methods `project_vertices` and `project_edges` are never used

#### `src\query\executor\logic\loops.rs`: 2 occurrences

- Line 163: unreachable pattern: no value can reach this
- Line 180: variable does not need to be mutable

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 174: multiple methods are never used
- Line 530: manual `RangeInclusive::contains` implementation: help: use: `(1..=100).contains(&i)`

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 99: multiple associated items are never used
- Line 612: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `b`

#### `src\core\value\value_compare.rs`: 1 occurrences

- Line 518: associated functions `cmp_string_list` and `cmp_value_list` are never used

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Value`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 536: multiple methods are never used

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 288: methods `vertices_to_dataset` and `edges_to_dataset` are never used

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 10: unused import: `crate::core::Value`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 10: unused import: `crate::core::Value`

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Value`

