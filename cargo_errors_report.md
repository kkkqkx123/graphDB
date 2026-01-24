# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 5
- **Total Warnings**: 5
- **Total Issues**: 10
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 5
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 5

### Error Type Breakdown

- **error[E0061]**: 4 errors
- **error[E0609]**: 1 errors

### Files with Errors (Top 10)

- `src\query\planner\statements\core\match_clause_planner.rs`: 3 errors
- `src\query\planner\statements\match_planner.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 5

### Warning Type Breakdown

- **warning**: 5 warnings

### Files with Warnings (Top 10)

- `src\query\scheduler\async_scheduler.rs`: 1 warnings
- `src\query\validator\go_validator.rs`: 1 warnings
- `src\common\memory.rs`: 1 warnings
- `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 warnings
- `src\query\planner\statements\match_planner.rs`: 1 warnings

## Detailed Error Categorization

### error[E0061]: this function takes 0 arguments but 1 argument was supplied

**Total Occurrences**: 4  
**Unique Files**: 2

#### `src\query\planner\statements\core\match_clause_planner.rs`: 3 occurrences

- Line 163: this function takes 0 arguments but 1 argument was supplied
- Line 171: this function takes 0 arguments but 1 argument was supplied
- Line 210: this function takes 0 arguments but 1 argument was supplied

#### `src\query\planner\statements\match_planner.rs`: 1 occurrences

- Line 158: this function takes 0 arguments but 1 argument was supplied

### error[E0609]: no field `query_context` on type `match_planner::MatchPlanner`: unknown field

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\statements\match_planner.rs`: 1 occurrences

- Line 159: no field `query_context` on type `match_planner::MatchPlanner`: unknown field

## Detailed Warning Categorization

### warning: unreachable pattern: no value can reach this

**Total Occurrences**: 5  
**Unique Files**: 5

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 583: unreachable pattern: no value can reach this

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\planner\statements\match_planner.rs`: 1 occurrences

- Line 96: unused variable: `match_ctx`: help: if this is intentional, prefix it with an underscore: `_match_ctx`

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 9: unused import: `ExecutionContext`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 42: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

