# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 14
- **Total Issues**: 14
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 11
- **Files with Issues**: 4

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 14

### Warning Type Breakdown

- **warning**: 14 warnings

### Files with Warnings (Top 10)

- `tests\integration_fulltext_edge_cases.rs`: 5 warnings
- `tests\integration_fulltext_concurrent.rs`: 3 warnings
- `tests\integration_fulltext_basic.rs`: 3 warnings
- `tests\integration_fulltext_sync.rs`: 3 warnings

## Detailed Warning Categorization

### warning: redundant closure: help: replace the closure with the tuple variant itself: `graphdb::core::Value::Int`

**Total Occurrences**: 14  
**Unique Files**: 4

#### `tests\integration_fulltext_edge_cases.rs`: 5 occurrences

- Line 347: function call inside of `expect`: help: try: `unwrap_or_else(|_| panic!("Failed to create index {}", i))`
- Line 363: function call inside of `expect`: help: try: `unwrap_or_else(|_| panic!("Failed to insert docs for index {}", i))`
- Line 374: function call inside of `expect`: help: try: `unwrap_or_else(|_| panic!("Search should succeed for index {}", i))`
- ... 2 more occurrences in this file

#### `tests\integration_fulltext_sync.rs`: 3 occurrences

- Line 329: redundant closure: help: replace the closure with the tuple variant itself: `graphdb::core::Value::Int`
- Line 333: the loop variable `i` is used to index `vertex_ids`
- Line 342: the loop variable `i` is used to index `vertex_ids`

#### `tests\integration_fulltext_basic.rs`: 3 occurrences

- Line 231: function call inside of `expect`: help: try: `unwrap_or_else(|_| panic!("Should contain doc_{}", i))`
- Line 371: function call inside of `expect`: help: try: `unwrap_or_else(|_| panic!("Should contain doc_{}", i))`
- Line 447: length comparison to zero: help: using `!is_empty` is clearer and more explicit: `!results.is_empty()`

#### `tests\integration_fulltext_concurrent.rs`: 3 occurrences

- Line 75: function call inside of `expect`: help: try: `unwrap_or_else(|_| panic!("Should find all {} documents", num_tasks))`
- Line 80: function call inside of `expect`: help: try: `unwrap_or_else(|_| panic!("Should contain doc_{}", i))`
- Line 139: function call inside of `expect`: help: try: `unwrap_or_else(|_| panic!("Should find all {} documents", num_docs))`

