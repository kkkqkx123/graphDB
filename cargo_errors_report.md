# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 36
- **Total Warnings**: 3
- **Total Issues**: 39
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 2
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 36

### Error Type Breakdown

- **error[E0433]**: 36 errors

### Files with Errors (Top 10)

- `src\query\parser\ast\stmt.rs`: 28 errors
- `src\query\parser\ast\pattern.rs`: 8 errors

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\query\parser\ast\stmt.rs`: 1 warnings
- `src\core\types\expression\utils.rs`: 1 warnings
- `src\query\parser\ast\pattern.rs`: 1 warnings

## Detailed Error Categorization

### error[E0433]: failed to resolve: use of undeclared type `CoreExprUtils`: use of undeclared type `CoreExprUtils`

**Total Occurrences**: 36  
**Unique Files**: 2

#### `src\query\parser\ast\stmt.rs`: 28 occurrences

- Line 906: failed to resolve: use of undeclared type `CoreExprUtils`: use of undeclared type `CoreExprUtils`
- Line 912: failed to resolve: use of undeclared type `CoreExprUtils`: use of undeclared type `CoreExprUtils`
- Line 921: failed to resolve: use of undeclared type `CoreExprUtils`: use of undeclared type `CoreExprUtils`
- ... 25 more occurrences in this file

#### `src\query\parser\ast\pattern.rs`: 8 occurrences

- Line 204: failed to resolve: use of undeclared type `CoreExprUtils`: use of undeclared type `CoreExprUtils`
- Line 207: failed to resolve: use of undeclared type `CoreExprUtils`: use of undeclared type `CoreExprUtils`
- Line 215: failed to resolve: use of undeclared type `CoreExprUtils`: use of undeclared type `CoreExprUtils`
- ... 5 more occurrences in this file

## Detailed Warning Categorization

### warning: unused import: `crate::core::types::expression::utils::collect_variables`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\parser\ast\stmt.rs`: 1 occurrences

- Line 9: unused import: `crate::core::types::expression::utils::collect_variables`

#### `src\core\types\expression\utils.rs`: 1 occurrences

- Line 9: unused imports: `BinaryOperator`, `DataType`, `UnaryOperator`, and `Value`

#### `src\query\parser\ast\pattern.rs`: 1 occurrences

- Line 7: unused import: `crate::core::types::expression::utils::collect_variables`

