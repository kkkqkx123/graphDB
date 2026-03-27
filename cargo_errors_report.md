# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 4
- **Total Issues**: 4
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 4
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 4

### Warning Type Breakdown

- **warning**: 4 warnings

### Files with Warnings (Top 10)

- `src\query\executor\object_pool.rs`: 2 warnings
- `src\query\cache\global_manager.rs`: 1 warnings
- `src\query\optimizer\strategy\expression_precomputation.rs`: 1 warnings

## Detailed Warning Categorization

### warning: mutable key type

**Total Occurrences**: 4  
**Unique Files**: 3

#### `src\query\executor\object_pool.rs`: 2 occurrences

- Line 113: field assignment outside of initializer for an instance created with Default::default()
- Line 404: this `if` statement can be collapsed

#### `src\query\optimizer\strategy\expression_precomputation.rs`: 1 occurrences

- Line 411: mutable key type

#### `src\query\cache\global_manager.rs`: 1 occurrences

- Line 388: struct update has no effect, all the fields in the struct have already been specified

