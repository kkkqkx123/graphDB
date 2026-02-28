# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 97
- **Total Warnings**: 0
- **Total Issues**: 97
- **Unique Error Patterns**: 12
- **Unique Warning Patterns**: 0
- **Files with Issues**: 30

## Error Statistics

**Total Errors**: 97

### Error Type Breakdown

- **error[E0308]**: 92 errors
- **error[E0277]**: 4 errors
- **error[E0599]**: 1 errors

### Files with Errors (Top 10)

- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_node.rs`: 10 errors
- `src\query\planner\statements\match_statement_planner.rs`: 7 errors
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 6 errors
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_inner_join.rs`: 6 errors
- `src\query\planner\rewrite\aggregate\push_filter_down_aggregate.rs`: 6 errors
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_cross_join.rs`: 6 errors
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_left_join.rs`: 6 errors
- `src\query\planner\rewrite\projection_pushdown\push_project_down.rs`: 5 errors
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_traverse.rs`: 5 errors
- `src\query\planner\plan\core\nodes\control_flow_node.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `ContextualExpression`, found `Expression`

**Total Occurrences**: 92  
**Unique Files**: 29

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_node.rs`: 9 occurrences

- Line 97: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 114: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 116: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 6 more occurrences in this file

#### `src\query\planner\statements\match_statement_planner.rs`: 7 occurrences

- Line 304: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 313: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 323: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 4 more occurrences in this file

#### `src\query\planner\rewrite\aggregate\push_filter_down_aggregate.rs`: 6 occurrences

- Line 178: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 186: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 189: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_cross_join.rs`: 6 occurrences

- Line 106: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 107: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 122: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 6 occurrences

- Line 106: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 107: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 122: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_left_join.rs`: 6 occurrences

- Line 106: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 107: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 122: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_inner_join.rs`: 6 occurrences

- Line 106: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 107: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 123: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\projection_pushdown\push_project_down.rs`: 5 occurrences

- Line 380: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 520: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 542: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 2 more occurrences in this file

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_traverse.rs`: 5 occurrences

- Line 104: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 120: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 122: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 2 more occurrences in this file

#### `src\query\executor\factory.rs`: 4 occurrences

- Line 647: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 1016: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 1022: mismatched types: expected `Expression`, found `ContextualExpression`
- ... 1 more occurrences in this file

#### `src\query\planner\rewrite\merge\combine_filter.rs`: 4 occurrences

- Line 96: arguments to this method are incorrect
- Line 102: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 161: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\core\nodes\factory.rs`: 3 occurrences

- Line 36: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 211: mismatched types: expected `ContextualExpression`, found `&str`
- Line 219: mismatched types: expected `ContextualExpression`, found `&str`

#### `src\query\planner\rewrite\pattern.rs`: 3 occurrences

- Line 306: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 347: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 355: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\optimizer\cost\node_estimators\data_processing.rs`: 2 occurrences

- Line 91: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 93: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\planner\rewrite\predicate_pushdown\push_efilter_down.rs`: 2 occurrences

- Line 81: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 87: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\rewrite\elimination\eliminate_filter.rs`: 2 occurrences

- Line 115: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 132: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\planner\plan\core\nodes\control_flow_node.rs`: 2 occurrences

- Line 395: mismatched types: expected `ContextualExpression`, found `&str`
- Line 405: mismatched types: expected `ContextualExpression`, found `&str`

#### `src\query\planner\rewrite\predicate_pushdown\push_vfilter_down_scan_vertices.rs`: 2 occurrences

- Line 81: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 87: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_expand_all.rs`: 2 occurrences

- Line 89: mismatched types: expected `ContextualExpression`, found `String`
- Line 145: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\go_planner.rs`: 1 occurrences

- Line 74: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 1 occurrences

- Line 72: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 1 occurrences

- Line 91: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\fetch_edges_planner.rs`: 1 occurrences

- Line 73: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\subgraph_planner.rs`: 1 occurrences

- Line 163: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 115: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\optimizer\analysis\fingerprint.rs`: 1 occurrences

- Line 277: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_get_nbrs.rs`: 1 occurrences

- Line 161: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\group_by_planner.rs`: 1 occurrences

- Line 202: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\lookup_planner.rs`: 1 occurrences

- Line 91: mismatched types: expected `ContextualExpression`, found `Expression`

### error[E0277]: `contextual::ContextualExpression` doesn't implement `std::fmt::Display`: `contextual::ContextualExpression` cannot be formatted with the default formatter

**Total Occurrences**: 4  
**Unique Files**: 3

#### `src\query\planner\plan\core\nodes\control_flow_node.rs`: 2 occurrences

- Line 398: can't compare `contextual::ContextualExpression` with `str`: no implementation for `contextual::ContextualExpression == str`
- Line 408: can't compare `contextual::ContextualExpression` with `str`: no implementation for `contextual::ContextualExpression == str`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_node.rs`: 1 occurrences

- Line 172: `contextual::ContextualExpression` doesn't implement `std::fmt::Display`: `contextual::ContextualExpression` cannot be formatted with the default formatter

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_get_nbrs.rs`: 1 occurrences

- Line 88: the trait bound `contextual::ContextualExpression: serde::Serialize` is not satisfied: the trait `Serialize` is not implemented for `contextual::ContextualExpression`

### error[E0599]: no method named `trim` found for reference `&contextual::ContextualExpression` in the current scope: method not found in `&ContextualExpression`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\cost\node_estimators\control_flow.rs`: 1 occurrences

- Line 40: no method named `trim` found for reference `&contextual::ContextualExpression` in the current scope: method not found in `&ContextualExpression`

