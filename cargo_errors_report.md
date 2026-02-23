# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 6
- **Total Issues**: 6
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 1
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 6

### Warning Type Breakdown

- **warning**: 6 warnings

### Files with Warnings (Top 10)

- `src\query\planner\rewrite\limit_pushdown\push_topn_down_index_scan.rs`: 1 warnings
- `src\query\planner\rewrite\limit_pushdown\push_limit_down_index_scan.rs`: 1 warnings
- `src\query\planner\rewrite\limit_pushdown\push_limit_down_get_edges.rs`: 1 warnings
- `src\query\planner\rewrite\limit_pushdown\push_limit_down_scan_vertices.rs`: 1 warnings
- `src\query\planner\rewrite\limit_pushdown\push_limit_down_scan_edges.rs`: 1 warnings
- `src\query\planner\rewrite\limit_pushdown\push_limit_down_get_vertices.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

**Total Occurrences**: 6  
**Unique Files**: 6

#### `src\query\planner\rewrite\limit_pushdown\push_limit_down_index_scan.rs`: 1 occurrences

- Line 7: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\planner\rewrite\limit_pushdown\push_topn_down_index_scan.rs`: 1 occurrences

- Line 7: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\planner\rewrite\limit_pushdown\push_limit_down_get_edges.rs`: 1 occurrences

- Line 7: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\planner\rewrite\limit_pushdown\push_limit_down_scan_vertices.rs`: 1 occurrences

- Line 7: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\planner\rewrite\limit_pushdown\push_limit_down_scan_edges.rs`: 1 occurrences

- Line 7: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\planner\rewrite\limit_pushdown\push_limit_down_get_vertices.rs`: 1 occurrences

- Line 7: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

