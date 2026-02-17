# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 3
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\query\visitor\ast_traverser.rs`: 1 warnings
- `src\query\executor\graph_query_executor.rs`: 1 warnings
- `src\query\parser\parser\stmt_parser.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused variable: `result`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 769: unused variable: `result`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 351: unused import: `ShowConfigsStmt`

#### `src\query\visitor\ast_traverser.rs`: 1 occurrences

- Line 12: unused imports: `KillQueryStmt`, `ShowConfigsStmt`, `ShowQueriesStmt`, `ShowSessionsStmt`, and `UpdateConfigsStmt`

