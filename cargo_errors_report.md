# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 2
- **Files with Issues**: 10

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\rules\transformation\top_n.rs`: 1 warnings
- `src\query\executor\factory.rs`: 1 warnings
- `src\query\optimizer\rules\limit_pushdown\push_topn_down_index_scan.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `crate::core::types::graph_schema::OrderDirection`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\optimizer\rules\transformation\top_n.rs`: 1 occurrences

- Line 134: unused import: `crate::core::types::graph_schema::OrderDirection`

#### `src\query\optimizer\rules\limit_pushdown\push_topn_down_index_scan.rs`: 1 occurrences

- Line 130: unused import: `crate::core::types::graph_schema::OrderDirection`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 100: function `parse_sort_item` is never used

