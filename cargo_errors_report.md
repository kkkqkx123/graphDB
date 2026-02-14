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

- **error[E0308]**: 2 errors

### Files with Errors (Top 10)

- `src\common\thread.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\api\service\stats_manager.rs`: 1 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `&mut MutexGuard<'_, RawMutex, _>`, found `MutexGuard<'_, RawMutex, VecDeque<...>>`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\common\thread.rs`: 2 occurrences

- Line 215: mismatched types: expected `&mut MutexGuard<'_, RawMutex, _>`, found `MutexGuard<'_, RawMutex, VecDeque<...>>`
- Line 215: mismatched types: expected `MutexGuard<'_, RawMutex, VecDeque<...>>`, found `()`

## Detailed Warning Categorization

### warning: unused variable: `now`: help: if this is intentional, prefix it with an underscore: `_now`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\service\stats_manager.rs`: 1 occurrences

- Line 116: unused variable: `now`: help: if this is intentional, prefix it with an underscore: `_now`

