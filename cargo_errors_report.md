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

- `src\query\optimizer\cost\node_estimators\scan.rs`: 1 warnings
- `src\query\optimizer\cost\assigner.rs`: 1 warnings
- `src\query\optimizer\cost\node_estimators\graph_traversal.rs`: 1 warnings

## Detailed Warning Categorization

### warning: method `get_avg_in_degree` is never used

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\optimizer\cost\node_estimators\graph_traversal.rs`: 1 occurrences

- Line 38: method `get_avg_in_degree` is never used

#### `src\query\optimizer\cost\assigner.rs`: 1 occurrences

- Line 28: unused import: `CostError`

#### `src\query\optimizer\cost\node_estimators\scan.rs`: 1 occurrences

- Line 14: unused import: `get_input_rows`

