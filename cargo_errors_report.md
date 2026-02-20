# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 11
- **Total Issues**: 11
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 6
- **Files with Issues**: 17

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 11

### Warning Type Breakdown

- **warning**: 11 warnings

### Files with Warnings (Top 10)

- `src\query\validator\fetch_vertices_validator.rs`: 2 warnings
- `src\query\validator\sequential_validator.rs`: 1 warnings
- `src\query\validator\unwind_validator.rs`: 1 warnings
- `src\query\validator\match_validator.rs`: 1 warnings
- `src\query\validator\pipe_validator.rs`: 1 warnings
- `src\query\validator\use_validator.rs`: 1 warnings
- `src\query\validator\order_by_validator.rs`: 1 warnings
- `src\query\validator\yield_validator.rs`: 1 warnings
- `src\query\validator\set_validator.rs`: 1 warnings
- `src\query\validator\fetch_edges_validator.rs`: 1 warnings

## Detailed Warning Categorization

### warning: methods `add_error` and `has_errors` are never used

**Total Occurrences**: 11  
**Unique Files**: 10

#### `src\query\validator\fetch_vertices_validator.rs`: 2 occurrences

- Line 178: method `get_tag_id` is never used
- Line 324: function `create_fetch_vertices_stmt` is never used

#### `src\query\validator\pipe_validator.rs`: 1 occurrences

- Line 134: methods `add_error` and `has_errors` are never used

#### `src\query\validator\sequential_validator.rs`: 1 occurrences

- Line 130: methods `add_error` and `has_errors` are never used

#### `src\query\validator\yield_validator.rs`: 1 occurrences

- Line 260: method `to_plan` is never used

#### `src\query\validator\use_validator.rs`: 1 occurrences

- Line 200: method `to_plan` is never used

#### `src\query\validator\set_validator.rs`: 1 occurrences

- Line 393: method `to_plan` is never used

#### `src\query\validator\match_validator.rs`: 1 occurrences

- Line 47: field `query_parts` is never read

#### `src\query\validator\unwind_validator.rs`: 1 occurrences

- Line 299: method `to_plan` is never used

#### `src\query\validator\order_by_validator.rs`: 1 occurrences

- Line 102: methods `add_error` and `has_errors` are never used

#### `src\query\validator\fetch_edges_validator.rs`: 1 occurrences

- Line 382: function `create_fetch_edges_stmt` is never used

