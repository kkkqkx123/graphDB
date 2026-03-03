# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 11
- **Total Warnings**: 2
- **Total Issues**: 13
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 2
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 11

### Error Type Breakdown

- **error[E0614]**: 9 errors
- **error[E0599]**: 2 errors

### Files with Errors (Top 10)

- `src\query\executor\tag_filter.rs`: 11 errors

## Warning Statistics

**Total Warnings**: 2

### Warning Type Breakdown

- **warning**: 2 warnings

### Files with Warnings (Top 10)

- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_get_nbrs.rs`: 1 warnings
- `src\query\validator\expression_analyzer.rs`: 1 warnings

## Detailed Error Categorization

### error[E0614]: type `operators::BinaryOperator` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 9  
**Unique Files**: 1

#### `src\query\executor\tag_filter.rs`: 9 occurrences

- Line 153: type `operators::BinaryOperator` cannot be dereferenced: can't be dereferenced
- Line 156: type `def::Expression` cannot be dereferenced: can't be dereferenced
- Line 158: type `operators::BinaryOperator` cannot be dereferenced: can't be dereferenced
- ... 6 more occurrences in this file

### error[E0599]: no function or associated item named `parse_simple_tag_list` found for struct `tag_filter::TagFilterProcessor` in the current scope: function or associated item not found in `TagFilterProcessor`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\tag_filter.rs`: 2 occurrences

- Line 144: no function or associated item named `parse_simple_tag_list` found for struct `tag_filter::TagFilterProcessor` in the current scope: function or associated item not found in `TagFilterProcessor`
- Line 193: no function or associated item named `parse_simple_tag_list` found for struct `tag_filter::TagFilterProcessor` in the current scope: function or associated item not found in `TagFilterProcessor`

## Detailed Warning Categorization

### warning: unused import: `std::sync::Arc`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_get_nbrs.rs`: 1 occurrences

- Line 16: unused import: `std::sync::Arc`

#### `src\query\validator\expression_analyzer.rs`: 1 occurrences

- Line 15: unused import: `ExpressionContext`

