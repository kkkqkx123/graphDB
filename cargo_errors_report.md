# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 12
- **Total Issues**: 12
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 10
- **Files with Issues**: 9

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 12

### Warning Type Breakdown

- **warning**: 12 warnings

### Files with Warnings (Top 10)

- `tests\common\test_scenario.rs`: 2 warnings
- `tests\integration_dql.rs`: 2 warnings
- `tests\common\query_helpers.rs`: 2 warnings
- `src\query\validator\validator_enum.rs`: 1 warnings
- `src\query\executor\data_access\vertex.rs`: 1 warnings
- `src\query\planning\statements\dml\insert_planner.rs`: 1 warnings
- `src\query\executor\explain\instrumented_executor.rs`: 1 warnings
- `tests\common\validation_helpers.rs`: 1 warnings
- `src\query\executor\explain\explain_executor.rs`: 1 warnings

## Detailed Warning Categorization

### warning: this function has too many arguments (8/7)

**Total Occurrences**: 12  
**Unique Files**: 9

#### `tests\integration_dql.rs`: 2 occurrences

- Line 17: unused import: `graphdb::core::Value`
- Line 21: unused import: `std::collections::HashMap`

#### `tests\common\test_scenario.rs`: 2 occurrences

- Line 488: accessing first element with `ds.rows.get(0)`: help: try: `ds.rows.first()`
- Line 498: accessing first element with `vertices.get(0)`: help: try: `vertices.first()`

#### `tests\common\query_helpers.rs`: 2 occurrences

- Line 12: unused import: `crate::common::TestStorage`
- Line 87: trait `common::query_helpers::FromValue` is more private than the item `common::query_helpers::QueryHelper::<S>::query_scalar`: method `common::query_helpers::QueryHelper::<S>::query_scalar` is reachable at visibility `pub(crate)`

#### `src\query\executor\data_access\vertex.rs`: 1 occurrences

- Line 22: this function has too many arguments (8/7)

#### `src\query\executor\explain\explain_executor.rs`: 1 occurrences

- Line 46: field `plan_description` is never read

#### `src\query\planning\statements\dml\insert_planner.rs`: 1 occurrences

- Line 101: method `create_yield_columns` is never used

#### `tests\common\validation_helpers.rs`: 1 occurrences

- Line 13: unused import: `crate::common::TestStorage`

#### `src\query\executor\explain\instrumented_executor.rs`: 1 occurrences

- Line 68: you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`

#### `src\query\validator\validator_enum.rs`: 1 occurrences

- Line 550: you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`

