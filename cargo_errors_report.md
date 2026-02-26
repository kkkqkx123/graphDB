# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 2
- **Total Warnings**: 0
- **Total Issues**: 2
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 2

### Error Type Breakdown

- **error[E0599]**: 2 errors

### Files with Errors (Top 10)

- `src\query\validator\lookup_validator.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0599]: no method named `get_edge_type` found for reference `&RedbSchemaManager` in the current scope

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\validator\lookup_validator.rs`: 2 occurrences

- Line 174: no method named `get_edge_type` found for reference `&RedbSchemaManager` in the current scope
- Line 190: no method named `get_tag` found for reference `&RedbSchemaManager` in the current scope

