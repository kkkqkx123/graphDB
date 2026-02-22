# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 9
- **Total Issues**: 9
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 7
- **Files with Issues**: 7

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 9

### Warning Type Breakdown

- **warning**: 9 warnings

### Files with Warnings (Top 10)

- `src\query\planner\statements\match_statement_planner.rs`: 2 warnings
- `src\query\optimizer\rules\index\index_covering_scan.rs`: 2 warnings
- `src\query\query_pipeline_manager.rs`: 2 warnings
- `src\query\optimizer\rules\scan\index_full_scan.rs`: 1 warnings
- `src\query\optimizer\rules\projection_pushdown\push_project_down.rs`: 1 warnings
- `src\query\planner\statements\set_operation_planner.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `std::sync::Arc`

**Total Occurrences**: 9  
**Unique Files**: 6

#### `src\query\query_pipeline_manager.rs`: 2 occurrences

- Line 16: unused import: `StaticConfigurablePlannerRegistry`
- Line 104: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\optimizer\rules\index\index_covering_scan.rs`: 2 occurrences

- Line 38: unused import: `std::sync::Arc`
- Line 159: unused import: `OptGroupNode`

#### `src\query\planner\statements\match_statement_planner.rs`: 2 occurrences

- Line 41: field `config` is never read
- Line 161: multiple methods are never used

#### `src\query\optimizer\rules\scan\index_full_scan.rs`: 1 occurrences

- Line 17: unused import: `std::sync::Arc`

#### `src\query\planner\statements\set_operation_planner.rs`: 1 occurrences

- Line 6: unused import: `SetOperationStmt`

#### `src\query\optimizer\rules\projection_pushdown\push_project_down.rs`: 1 occurrences

- Line 32: unused import: `std::sync::Arc`

