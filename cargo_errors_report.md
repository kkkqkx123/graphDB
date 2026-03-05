# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 3
- **Total Warnings**: 0
- **Total Issues**: 3
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 3

### Error Type Breakdown

- **error[E0609]**: 2 errors
- **error[E0004]**: 1 errors

### Files with Errors (Top 10)

- `src\api\embedded\statement.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0609]: no field `items` on type `&GroupByStmt`: unknown field

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\api\embedded\statement.rs`: 2 occurrences

- Line 614: no field `items` on type `&GroupByStmt`: unknown field
- Line 617: no field `having` on type `&GroupByStmt`: unknown field

### error[E0004]: non-exhaustive patterns: `&stmt::CreateTarget::Path { .. }`, `&stmt::CreateTarget::Space { .. }` and `&stmt::CreateTarget::Index { .. }` not covered: patterns `&stmt::CreateTarget::Path { .. }`, `&stmt::CreateTarget::Space { .. }` and `&stmt::CreateTarget::Index { .. }` not covered

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\embedded\statement.rs`: 1 occurrences

- Line 561: non-exhaustive patterns: `&stmt::CreateTarget::Path { .. }`, `&stmt::CreateTarget::Space { .. }` and `&stmt::CreateTarget::Index { .. }` not covered: patterns `&stmt::CreateTarget::Path { .. }`, `&stmt::CreateTarget::Space { .. }` and `&stmt::CreateTarget::Index { .. }` not covered

