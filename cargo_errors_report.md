# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 23
- **Total Issues**: 23
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 10
- **Files with Issues**: 13

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 23

### Warning Type Breakdown

- **warning**: 23 warnings

### Files with Warnings (Top 10)

- `src\query\executor\expression\functions\builtin\math.rs`: 4 warnings
- `src\query\executor\expression\functions\builtin\string.rs`: 3 warnings
- `src\core\npath.rs`: 3 warnings
- `src\api\embedded\session.rs`: 2 warnings
- `src\query\executor\expression\functions\builtin\geography.rs`: 2 warnings
- `src\query\executor\expression\functions\builtin\datetime.rs`: 2 warnings
- `src\query\executor\expression\evaluation_context\default_context.rs`: 1 warnings
- `src\query\executor\expression\evaluation_context\row_context.rs`: 1 warnings
- `src\query\executor\expression\functions\builtin\conversion.rs`: 1 warnings
- `src\query\executor\expression\evaluator\traits.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `crate::define_datetime_extractor`

**Total Occurrences**: 23  
**Unique Files**: 13

#### `src\query\executor\expression\functions\builtin\math.rs`: 4 occurrences

- Line 6: unused import: `crate::define_binary_numeric_fn`
- Line 7: unused import: `crate::define_function_enum`
- Line 8: unused import: `crate::define_unary_float_fn`
- ... 1 more occurrences in this file

#### `src\query\executor\expression\functions\builtin\string.rs`: 3 occurrences

- Line 6: unused import: `crate::define_binary_string_bool_fn`
- Line 7: unused import: `crate::define_function_enum`
- Line 8: unused import: `crate::define_unary_string_fn`

#### `src\core\npath.rs`: 3 occurrences

- Line 290: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here
- Line 295: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here
- Line 300: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

#### `src\query\executor\expression\functions\builtin\datetime.rs`: 2 occurrences

- Line 6: unused import: `crate::define_datetime_extractor`
- Line 7: unused import: `crate::define_function_enum`

#### `src\api\embedded\session.rs`: 2 occurrences

- Line 213: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here
- Line 252: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

#### `src\query\executor\expression\functions\builtin\geography.rs`: 2 occurrences

- Line 7: unused import: `crate::define_binary_geography_fn`
- Line 8: unused import: `crate::define_function_enum`

#### `src\query\executor\base\execution_context.rs`: 1 occurrences

- Line 84: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

#### `src\query\executor\expression\evaluator\traits.rs`: 1 occurrences

- Line 25: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

#### `src\query\executor\expression\evaluation_context\default_context.rs`: 1 occurrences

- Line 91: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

#### `src\query\planner\statements\maintain_planner.rs`: 1 occurrences

- Line 81: variable does not need to be mutable

#### `src\query\executor\expression\evaluation_context\row_context.rs`: 1 occurrences

- Line 76: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

#### `src\query\executor\expression\functions\builtin\conversion.rs`: 1 occurrences

- Line 6: unused import: `crate::define_function_enum`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 563: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

