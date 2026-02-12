# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 6
- **Total Warnings**: 0
- **Total Issues**: 6
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 0
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 6

### Error Type Breakdown

- **error[E0599]**: 6 errors

### Files with Errors (Top 10)

- `src\query\planner\statements\match_statement_planner.rs`: 3 errors
- `src\query\planner\statements\clauses\return_clause_planner.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0599]: no function or associated item named `is_aggregated_expression` found for struct `return_clause_planner::ReturnClausePlanner` in the current scope: function or associated item not found in `ReturnClausePlanner`

**Total Occurrences**: 6  
**Unique Files**: 2

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 3 occurrences

- Line 161: no function or associated item named `is_aggregated_expression` found for struct `return_clause_planner::ReturnClausePlanner` in the current scope: function or associated item not found in `ReturnClausePlanner`
- Line 162: no function or associated item named `is_aggregated_expression` found for struct `return_clause_planner::ReturnClausePlanner` in the current scope: function or associated item not found in `ReturnClausePlanner`
- Line 163: no function or associated item named `is_aggregated_expression` found for struct `return_clause_planner::ReturnClausePlanner` in the current scope: function or associated item not found in `ReturnClausePlanner`

#### `src\query\planner\statements\match_statement_planner.rs`: 3 occurrences

- Line 489: no function or associated item named `parse_tag_from_pattern` found for struct `match_statement_planner::MatchStatementPlanner` in the current scope: function or associated item not found in `MatchStatementPlanner`
- Line 490: no function or associated item named `parse_tag_from_pattern` found for struct `match_statement_planner::MatchStatementPlanner` in the current scope: function or associated item not found in `MatchStatementPlanner`
- Line 491: no function or associated item named `parse_tag_from_pattern` found for struct `match_statement_planner::MatchStatementPlanner` in the current scope: function or associated item not found in `MatchStatementPlanner`

