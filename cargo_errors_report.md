# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 2
- **Total Warnings**: 0
- **Total Issues**: 2
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 2

### Error Type Breakdown

- **error[E0382]**: 2 errors

### Files with Errors (Top 10)

- `src\api\embedded\c_api\database.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0382]: borrow of moved value: `e`: value borrowed here after move

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\api\embedded\c_api\database.rs`: 2 occurrences

- Line 262: borrow of moved value: `e`: value borrowed here after move
- Line 303: borrow of moved value: `e`: value borrowed here after move

