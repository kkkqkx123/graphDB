# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 1
- **Total Warnings**: 1
- **Total Issues**: 2
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 1
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 1

### Error Type Breakdown

- **error[E0308]**: 1 errors

### Files with Errors (Top 10)

- `src\api\embedded\session.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\core\value\comparison.rs`: 1 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `u64`, found `usize`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\embedded\session.rs`: 1 occurrences

- Line 161: mismatched types: expected `u64`, found `usize`

## Detailed Warning Categorization

### warning: unreachable pattern: no value can reach this

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\value\comparison.rs`: 1 occurrences

- Line 167: unreachable pattern: no value can reach this

