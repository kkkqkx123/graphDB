# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 4
- **Total Issues**: 4
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 1
- **Files with Issues**: 4

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 4

### Warning Type Breakdown

- **warning**: 4 warnings

### Files with Warnings (Top 10)

- `src\query\executor\expression\evaluator\traits.rs`: 1 warnings
- `src\query\executor\expression\evaluation_context\default_context.rs`: 1 warnings
- `src\query\executor\expression\evaluation_context\row_context.rs`: 1 warnings
- `src\query\executor\base\execution_context.rs`: 1 warnings

## Detailed Warning Categorization

### warning: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

**Total Occurrences**: 4  
**Unique Files**: 4

#### `src\query\executor\expression\evaluator\traits.rs`: 1 occurrences

- Line 25: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

#### `src\query\executor\base\execution_context.rs`: 1 occurrences

- Line 83: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

#### `src\query\executor\expression\evaluation_context\default_context.rs`: 1 occurrences

- Line 91: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

#### `src\query\executor\expression\evaluation_context\row_context.rs`: 1 occurrences

- Line 76: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

