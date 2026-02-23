# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 6
- **Total Issues**: 6
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 5
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 6

### Warning Type Breakdown

- **warning**: 6 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\core\analyze.rs`: 4 warnings
- `src\query\planner\rewrite\plan_rewriter.rs`: 2 warnings

## Detailed Warning Categorization

### warning: unused variable: `table_name`: help: if this is intentional, prefix it with an underscore: `_table_name`

**Total Occurrences**: 6  
**Unique Files**: 2

#### `src\query\optimizer\core\analyze.rs`: 4 occurrences

- Line 96: unused variable: `table_name`: help: if this is intentional, prefix it with an underscore: `_table_name`
- Line 142: unused variable: `table_name`: help: if this is intentional, prefix it with an underscore: `_table_name`
- Line 160: unused variable: `index_name`: help: if this is intentional, prefix it with an underscore: `_index_name`
- ... 1 more occurrences in this file

#### `src\query\planner\rewrite\plan_rewriter.rs`: 2 occurrences

- Line 13: unused import: `TransformResult`
- Line 14: unused import: `crate::query::optimizer::OptimizerError`

