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

- `src\query\validator\statements\insert_vertices_validator.rs`: 1 warnings
- `src\search\config.rs`: 1 warnings
- `src\query\executor\expression\functions\builtin\aggregate.rs`: 1 warnings

## Detailed Warning Categorization

### warning: this `if let` can be collapsed into the outer `if let`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\validator\statements\insert_vertices_validator.rs`: 1 occurrences

- Line 207: this `if let` can be collapsed into the outer `if let`

#### `src\query\executor\expression\functions\builtin\aggregate.rs`: 1 occurrences

- Line 378: you seem to use `.enumerate()` and immediately discard the index

#### `src\search\config.rs`: 1 occurrences

- Line 17: this `impl` can be derived

