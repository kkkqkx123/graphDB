# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 8
- **Total Issues**: 8
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 8
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 8

### Warning Type Breakdown

- **warning**: 8 warnings

### Files with Warnings (Top 10)

- `src\query\validator\validator_trait.rs`: 2 warnings
- `src\query\planner\statements\match_statement_planner.rs`: 2 warnings
- `src\query\planner\planner.rs`: 1 warnings
- `src\query\planner\rewrite\elimination\eliminate_sort.rs`: 1 warnings
- `src\query\optimizer\analysis\expression.rs`: 1 warnings
- `src\query\optimizer\analysis\fingerprint.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused variable: `stmt`: help: if this is intentional, prefix it with an underscore: `_stmt`

**Total Occurrences**: 8  
**Unique Files**: 6

#### `src\query\validator\validator_trait.rs`: 2 occurrences

- Line 377: unused variable: `stmt`: help: if this is intentional, prefix it with an underscore: `_stmt`
- Line 378: unused variable: `qctx`: help: if this is intentional, prefix it with an underscore: `_qctx`

#### `src\query\planner\statements\match_statement_planner.rs`: 2 occurrences

- Line 17: unused import: `crate::query::planner::plan::ExecutionPlan`
- Line 121: unused variable: `validation_info`: help: if this is intentional, prefix it with an underscore: `_validation_info`

#### `src\query\optimizer\analysis\fingerprint.rs`: 1 occurrences

- Line 394: unused variable: `calculator`: help: if this is intentional, prefix it with an underscore: `_calculator`

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 11: unused import: `ValidationInfo`

#### `src\query\planner\rewrite\elimination\eliminate_sort.rs`: 1 occurrences

- Line 212: function `create_test_sort_node` is never used

#### `src\query\optimizer\analysis\expression.rs`: 1 occurrences

- Line 466: unused variable: `analyzer`: help: if this is intentional, prefix it with an underscore: `_analyzer`

