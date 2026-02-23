# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 13
- **Total Warnings**: 0
- **Total Issues**: 13
- **Unique Error Patterns**: 9
- **Unique Warning Patterns**: 0
- **Files with Issues**: 11

## Error Statistics

**Total Errors**: 13

### Error Type Breakdown

- **error[E0433]**: 13 errors

### Files with Errors (Top 10)

- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_expand_all.rs`: 2 errors
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_get_nbrs.rs`: 2 errors
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_left_join.rs`: 1 errors
- `src\query\optimizer\rules\join\join_optimization.rs`: 1 errors
- `src\query\optimizer\rule_traits.rs`: 1 errors
- `src\query\optimizer\rules\scan\scan_with_filter_optimization.rs`: 1 errors
- `src\query\optimizer\rules\transformation\top_n.rs`: 1 errors
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_cross_join.rs`: 1 errors
- `src\query\optimizer\rules\transformation\optimize_set_operation_input_order.rs`: 1 errors
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0433]: failed to resolve: use of undeclared type `HashInnerJoinNode`: use of undeclared type `HashInnerJoinNode`

**Total Occurrences**: 13  
**Unique Files**: 11

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_expand_all.rs`: 2 occurrences

- Line 140: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- Line 147: failed to resolve: use of undeclared type `ExpandAllNode`: use of undeclared type `ExpandAllNode`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_get_nbrs.rs`: 2 occurrences

- Line 159: failed to resolve: use of undeclared type `FilterNode`: use of undeclared type `FilterNode`
- Line 162: failed to resolve: use of undeclared type `GetNeighborsNode`: use of undeclared type `GetNeighborsNode`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 1 occurrences

- Line 214: failed to resolve: use of undeclared type `HashInnerJoinNode`: use of undeclared type `HashInnerJoinNode`

#### `src\query\optimizer\rules\join\join_optimization.rs`: 1 occurrences

- Line 346: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`

#### `src\query\optimizer\rules\scan\scan_with_filter_optimization.rs`: 1 occurrences

- Line 60: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_cross_join.rs`: 1 occurrences

- Line 214: failed to resolve: use of undeclared type `CrossJoinNode`: use of undeclared type `CrossJoinNode`

#### `src\query\optimizer\rules\transformation\top_n.rs`: 1 occurrences

- Line 135: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`

#### `src\query\optimizer\rule_traits.rs`: 1 occurrences

- Line 620: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_inner_join.rs`: 1 occurrences

- Line 220: failed to resolve: use of undeclared type `InnerJoinNode`: use of undeclared type `InnerJoinNode`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_left_join.rs`: 1 occurrences

- Line 214: failed to resolve: use of undeclared type `HashLeftJoinNode`: use of undeclared type `HashLeftJoinNode`

#### `src\query\optimizer\rules\transformation\optimize_set_operation_input_order.rs`: 1 occurrences

- Line 251: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`

