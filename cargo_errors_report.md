# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 10
- **Total Issues**: 10
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 6
- **Files with Issues**: 8

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 10

### Warning Type Breakdown

- **warning**: 10 warnings

### Files with Warnings (Top 10)

- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_traverse.rs`: 3 warnings
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_cross_join.rs`: 1 warnings
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_get_nbrs.rs`: 1 warnings
- `src\query\planner\rewrite\predicate_pushdown\push_efilter_down.rs`: 1 warnings
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_all_paths.rs`: 1 warnings
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_left_join.rs`: 1 warnings
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_inner_join.rs`: 1 warnings
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `BinaryInputNode`

**Total Occurrences**: 10  
**Unique Files**: 8

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_traverse.rs`: 3 occurrences

- Line 7: unused import: `crate::query::planner::plan::core::nodes::traversal_node::TraverseNode`
- Line 196: unused import: `crate::query::planner::plan::core::nodes::start_node::StartNode`
- Line 197: unused import: `crate::query::planner::plan::core::nodes::filter_node::FilterNode`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_inner_join.rs`: 1 occurrences

- Line 15: unused import: `BinaryInputNode`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 1 occurrences

- Line 15: unused import: `BinaryInputNode`

#### `src\query\planner\rewrite\predicate_pushdown\push_efilter_down.rs`: 1 occurrences

- Line 8: unused import: `crate::query::planner::plan::core::nodes::traversal_node::TraverseNode`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_left_join.rs`: 1 occurrences

- Line 15: unused import: `BinaryInputNode`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_all_paths.rs`: 1 occurrences

- Line 7: unused import: `crate::query::planner::plan::algorithms::AllPaths`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_get_nbrs.rs`: 1 occurrences

- Line 6: unused import: `crate::core::Expression`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_cross_join.rs`: 1 occurrences

- Line 15: unused import: `BinaryInputNode`

