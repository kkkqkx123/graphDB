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

- **error[E0277]**: 2 errors

### Files with Errors (Top 10)

- `src\storage\operations\rollback.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0277]: `RefCell<Vec<std::string::String>>` cannot be shared between threads safely: `RefCell<Vec<std::string::String>>` cannot be shared between threads safely

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\storage\operations\rollback.rs`: 2 occurrences

- Line 399: `RefCell<Vec<std::string::String>>` cannot be shared between threads safely: `RefCell<Vec<std::string::String>>` cannot be shared between threads safely
- Line 444: `RefCell<Vec<std::string::String>>` cannot be shared between threads safely: `RefCell<Vec<std::string::String>>` cannot be shared between threads safely

