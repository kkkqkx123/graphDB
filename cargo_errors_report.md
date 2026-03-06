# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 3
- **Total Warnings**: 1
- **Total Issues**: 4
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 1
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 3

### Error Type Breakdown

- **error[E0599]**: 2 errors
- **error[E0560]**: 1 errors

### Files with Errors (Top 10)

- `src\api\embedded\c_api\batch.rs`: 2 errors
- `src\api\embedded\c_api\transaction.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\api\embedded\c_api\error.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `set_tag` found for struct `vertex_edge_path::Vertex` in the current scope

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\api\embedded\c_api\batch.rs`: 2 occurrences

- Line 108: no method named `set_tag` found for struct `vertex_edge_path::Vertex` in the current scope
- Line 110: no method named `set_property` found for struct `vertex_edge_path::Vertex` in the current scope

### error[E0560]: struct `GraphDbResultHandle` has no field named `current_row`: `GraphDbResultHandle` does not have this field

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\embedded\c_api\transaction.rs`: 1 occurrences

- Line 175: struct `GraphDbResultHandle` has no field named `current_row`: `GraphDbResultHandle` does not have this field

## Detailed Warning Categorization

### warning: unused doc comment: rustdoc does not generate documentation for macro invocations

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\embedded\c_api\error.rs`: 1 occurrences

- Line 9: unused doc comment: rustdoc does not generate documentation for macro invocations

