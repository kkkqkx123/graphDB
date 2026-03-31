# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 3
- **Total Warnings**: 2
- **Total Issues**: 5
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 2
- **Files with Issues**: 4

## Error Statistics

**Total Errors**: 3

### Error Type Breakdown

- **error[E0061]**: 2 errors
- **error[E0599]**: 1 errors

### Files with Errors (Top 10)

- `src\query\planning\statements\dml\insert_planner.rs`: 2 errors
- `src\query\planning\statements\dml\merge_planner.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 2

### Warning Type Breakdown

- **warning**: 2 warnings

### Files with Warnings (Top 10)

- `src\query\executor\factory\builders\data_modification_builder.rs`: 1 warnings
- `src\query\planning\statements\dml\delete_planner.rs`: 1 warnings

## Detailed Error Categorization

### error[E0061]: this method takes 4 arguments but 3 arguments were supplied

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\planning\statements\dml\insert_planner.rs`: 2 occurrences

- Line 320: this method takes 4 arguments but 3 arguments were supplied
- Line 346: this method takes 5 arguments but 4 arguments were supplied

### error[E0599]: no method named `as_ref` found for enum `core::types::expr::def::Expression` in the current scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planning\statements\dml\merge_planner.rs`: 1 occurrences

- Line 111: no method named `as_ref` found for enum `core::types::expr::def::Expression` in the current scope

## Detailed Warning Categorization

### warning: unused import: `crate::core::types::ContextualExpression`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\planning\statements\dml\delete_planner.rs`: 1 occurrences

- Line 5: unused import: `crate::core::types::ContextualExpression`

#### `src\query\executor\factory\builders\data_modification_builder.rs`: 1 occurrences

- Line 12: unused imports: `EdgeUpdateInfo` and `VertexUpdateInfo`

