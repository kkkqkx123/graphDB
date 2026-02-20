# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 27
- **Total Warnings**: 2
- **Total Issues**: 29
- **Unique Error Patterns**: 10
- **Unique Warning Patterns**: 2
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 27

### Error Type Breakdown

- **error[E0412]**: 13 errors
- **error[E0405]**: 6 errors
- **error[E0433]**: 5 errors
- **error[E0422]**: 3 errors

### Files with Errors (Top 10)

- `src\query\validator\update_validator.rs`: 13 errors
- `src\query\validator\strategies\pagination_strategy.rs`: 5 errors
- `src\query\validator\strategies\clause_strategy.rs`: 5 errors
- `src\query\validator\strategies\expression_strategy.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 2

### Warning Type Breakdown

- **warning**: 2 warnings

### Files with Warnings (Top 10)

- `src\query\validator\go_validator.rs`: 1 warnings
- `src\query\validator\strategies\aggregate_strategy.rs`: 1 warnings

## Detailed Error Categorization

### error[E0412]: cannot find type `SpaceInfo` in module `crate::storage::metadata`: not found in `crate::storage::metadata`

**Total Occurrences**: 13  
**Unique Files**: 4

#### `src\query\validator\update_validator.rs`: 10 occurrences

- Line 667: cannot find type `SpaceInfo` in module `crate::storage::metadata`: not found in `crate::storage::metadata`
- Line 675: cannot find type `SpaceInfo` in module `crate::storage::metadata`: not found in `crate::storage::metadata`
- Line 689: cannot find type `SpaceInfo` in module `crate::storage::metadata`: not found in `crate::storage::metadata`
- ... 7 more occurrences in this file

#### `src\query\validator\strategies\clause_strategy.rs`: 1 occurrences

- Line 276: cannot find type `ValidationStrategyType` in this scope

#### `src\query\validator\strategies\pagination_strategy.rs`: 1 occurrences

- Line 182: cannot find type `ValidationStrategyType` in this scope

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 237: cannot find type `ValidationStrategyType` in this scope

### error[E0405]: cannot find trait `ValidationStrategy` in this scope: not found in this scope

**Total Occurrences**: 6  
**Unique Files**: 3

#### `src\query\validator\strategies\expression_strategy.rs`: 2 occurrences

- Line 204: cannot find trait `ValidationStrategy` in this scope: not found in this scope
- Line 205: cannot find trait `ValidationContext` in this scope: not found in this scope

#### `src\query\validator\strategies\clause_strategy.rs`: 2 occurrences

- Line 251: cannot find trait `ValidationStrategy` in this scope: not found in this scope
- Line 252: cannot find trait `ValidationContext` in this scope: not found in this scope

#### `src\query\validator\strategies\pagination_strategy.rs`: 2 occurrences

- Line 143: cannot find trait `ValidationStrategy` in this scope: not found in this scope
- Line 144: cannot find trait `ValidationContext` in this scope: not found in this scope

### error[E0433]: failed to resolve: use of undeclared type `ValidationStrategyType`: use of undeclared type `ValidationStrategyType`

**Total Occurrences**: 5  
**Unique Files**: 3

#### `src\query\validator\strategies\clause_strategy.rs`: 2 occurrences

- Line 277: failed to resolve: use of undeclared type `ValidationStrategyType`: use of undeclared type `ValidationStrategyType`
- Line 294: failed to resolve: use of undeclared type `ValidationStrategyType`: use of undeclared type `ValidationStrategyType`

#### `src\query\validator\strategies\pagination_strategy.rs`: 2 occurrences

- Line 183: failed to resolve: use of undeclared type `ValidationStrategyType`: use of undeclared type `ValidationStrategyType`
- Line 199: failed to resolve: use of undeclared type `ValidationStrategyType`: use of undeclared type `ValidationStrategyType`

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 238: failed to resolve: use of undeclared type `ValidationStrategyType`: use of undeclared type `ValidationStrategyType`

### error[E0422]: cannot find struct, variant or union type `SpaceInfo` in module `crate::storage::metadata`: not found in `crate::storage::metadata`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\validator\update_validator.rs`: 3 occurrences

- Line 677: cannot find struct, variant or union type `SpaceInfo` in module `crate::storage::metadata`: not found in `crate::storage::metadata`
- Line 703: cannot find struct, variant or union type `TagInfo` in module `crate::storage::metadata`: not found in `crate::storage::metadata`
- Line 734: cannot find struct, variant or union type `EdgeTypeInfo` in module `crate::storage::metadata`: not found in `crate::storage::metadata`

## Detailed Warning Categorization

### warning: unused imports: `AggregateFunction`, `BinaryOperator`, and `UnaryOperator`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 7: unused imports: `AggregateFunction`, `BinaryOperator`, and `UnaryOperator`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 4: unused import: `super::super::structs::*`

