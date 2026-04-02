# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 27
- **Total Issues**: 27
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 8
- **Files with Issues**: 10

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 27

### Warning Type Breakdown

- **warning**: 27 warnings

### Files with Warnings (Top 10)

- `tests\integration_dcl.rs`: 12 warnings
- `tests\integration_management.rs`: 6 warnings
- `src\query\executor\data_modification\tag_ops.rs`: 2 warnings
- `tests\integration_cypher_create.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 1 warnings
- `src\query\executor\factory\executors\plan_executor.rs`: 1 warnings
- `src\query\planning\plan\core\explain.rs`: 1 warnings
- `src\query\planning\statements\match_statement_planner.rs`: 1 warnings
- `src\query\executor\data_access\vertex.rs`: 1 warnings
- `src\query\executor\data_access\search.rs`: 1 warnings

## Detailed Warning Categorization

### warning: you seem to use `.enumerate()` and immediately discard the index

**Total Occurrences**: 27  
**Unique Files**: 10

#### `tests\integration_dcl.rs`: 12 occurrences

- Line 457: you seem to use `.enumerate()` and immediately discard the index
- Line 481: you seem to use `.enumerate()` and immediately discard the index
- Line 488: you seem to use `.enumerate()` and immediately discard the index
- ... 9 more occurrences in this file

#### `tests\integration_management.rs`: 6 occurrences

- Line 1353: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`
- Line 1374: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`
- Line 1613: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`
- ... 3 more occurrences in this file

#### `src\query\executor\data_modification\tag_ops.rs`: 2 occurrences

- Line 124: unused variable: `e`: help: if this is intentional, prefix it with an underscore: `_e`
- Line 136: unused variable: `e`: help: if this is intentional, prefix it with an underscore: `_e`

#### `src\query\executor\data_access\vertex.rs`: 1 occurrences

- Line 140: unused variable: `e`: help: if this is intentional, prefix it with an underscore: `_e`

#### `src\query\planning\plan\core\explain.rs`: 1 occurrences

- Line 284: method `get_dependency_ids` is never used

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 1 occurrences

- Line 410: unused variable: `e`: help: if this is intentional, prefix it with an underscore: `_e`

#### `tests\integration_cypher_create.rs`: 1 occurrences

- Line 386: unused variable: `ngql_result`: help: if this is intentional, prefix it with an underscore: `_ngql_result`

#### `src\query\executor\data_access\search.rs`: 1 occurrences

- Line 94: method `get_schema_name` is never used

#### `src\query\planning\statements\match_statement_planner.rs`: 1 occurrences

- Line 650: method `join_node_plans` is never used

#### `src\query\executor\factory\executors\plan_executor.rs`: 1 occurrences

- Line 6: unused import: `crate::core::Value`

