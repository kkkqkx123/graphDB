# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 4
- **Total Warnings**: 1
- **Total Issues**: 5
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 1
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 4

### Error Type Breakdown

- **error[E0308]**: 2 errors
- **error[E0369]**: 1 errors
- **error[E0277]**: 1 errors

### Files with Errors (Top 10)

- `src\api\server\session\network_session.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\core\result\builder.rs`: 1 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `SavepointId`, found integer

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\api\server\session\network_session.rs`: 2 occurrences

- Line 525: mismatched types: expected `SavepointId`, found integer
- Line 526: mismatched types: expected `SavepointId`, found integer

### error[E0277]: can't compare `savepoint::SavepointId` with `{integer}`: no implementation for `savepoint::SavepointId == {integer}`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\server\session\network_session.rs`: 1 occurrences

- Line 528: can't compare `savepoint::SavepointId` with `{integer}`: no implementation for `savepoint::SavepointId == {integer}`

### error[E0369]: binary operation `==` cannot be applied to type `transaction::types::TransactionOptions`: transaction::types::TransactionOptions, transaction::types::TransactionOptions

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\server\session\network_session.rs`: 1 occurrences

- Line 505: binary operation `==` cannot be applied to type `transaction::types::TransactionOptions`: transaction::types::TransactionOptions, transaction::types::TransactionOptions

## Detailed Warning Categorization

### warning: unused import: `crate::core::result::iterator::DefaultIterator`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\result\builder.rs`: 1 occurrences

- Line 179: unused import: `crate::core::result::iterator::DefaultIterator`

