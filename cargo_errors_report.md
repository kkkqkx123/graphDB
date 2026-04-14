# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 22
- **Total Issues**: 22
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 14
- **Files with Issues**: 15

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 22

### Warning Type Breakdown

- **warning**: 22 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\filter.rs`: 3 warnings
- `src\query\executor\result_processing\projection.rs`: 3 warnings
- `src\query\executor\result_processing\dedup.rs`: 2 warnings
- `src\query\executor\result_processing\sample.rs`: 2 warnings
- `src\query\executor\logic\loops.rs`: 2 warnings
- `src\query\executor\data_processing\set_operations\minus.rs`: 1 warnings
- `src\query\executor\data_processing\set_operations\intersect.rs`: 1 warnings
- `src\core\value\value_compare.rs`: 1 warnings
- `src\query\executor\data_processing\set_operations\union.rs`: 1 warnings
- `src\query\executor\result_processing\topn.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `crate::core::Value`

**Total Occurrences**: 22  
**Unique Files**: 15

#### `src\query\executor\result_processing\projection.rs`: 3 occurrences

- Line 27: function `extract_variable_names` is never used
- Line 144: constant `INTERNAL_VARIABLES` is never used
- Line 392: methods `project_vertices` and `project_edges` are never used

#### `src\query\executor\result_processing\filter.rs`: 3 occurrences

- Line 25: function `extract_variable_names` is never used
- Line 142: constant `INTERNAL_VARIABLES` is never used
- Line 348: methods `filter_values`, `filter_vertices`, and `filter_edges` are never used

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 99: multiple associated items are never used
- Line 612: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `b`

#### `src\query\executor\logic\loops.rs`: 2 occurrences

- Line 163: unreachable pattern: no value can reach this
- Line 180: variable does not need to be mutable

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 174: multiple methods are never used
- Line 530: manual `RangeInclusive::contains` implementation: help: use: `(1..=100).contains(&i)`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 10: unused import: `crate::core::Value`

#### `src\core\value\value_compare.rs`: 1 occurrences

- Line 518: associated functions `cmp_string_list` and `cmp_value_list` are never used

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 10: unused import: `crate::core::Value`

#### `tests\common\test_scenario.rs`: 1 occurrences

- Line 542: you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`

#### `tests\common\validation_helpers.rs`: 1 occurrences

- Line 166: you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 536: multiple methods are never used

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 288: methods `vertices_to_dataset` and `edges_to_dataset` are never used

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Value`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Value`

#### `tests\common\query_helpers.rs`: 1 occurrences

- Line 59: unreachable pattern: no value can reach this

