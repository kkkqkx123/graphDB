# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 2
- **Total Warnings**: 1
- **Total Issues**: 3
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 1
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 2

### Error Type Breakdown

- **error[E0599]**: 1 errors
- **error[E0433]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\factory.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 warnings

## Detailed Error Categorization

### error[E0433]: failed to resolve: could not find `MultiShortestPathExecutor` in `graph_traversal`: could not find `MultiShortestPathExecutor` in `graph_traversal`, help: a struct with a similar name exists: `ShortestPathExecutor`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 826: failed to resolve: could not find `MultiShortestPathExecutor` in `graph_traversal`: could not find `MultiShortestPathExecutor` in `graph_traversal`, help: a struct with a similar name exists: `ShortestPathExecutor`

### error[E0599]: no variant or associated item named `MultiShortestPath` found for enum `executor_enum::ExecutorEnum` in the current scope: variant or associated item not found in `ExecutorEnum<_>`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 835: no variant or associated item named `MultiShortestPath` found for enum `executor_enum::ExecutorEnum` in the current scope: variant or associated item not found in `ExecutorEnum<_>`

## Detailed Warning Categorization

### warning: unused import: `DBError`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 10: unused import: `DBError`

