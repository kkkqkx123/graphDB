# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 16
- **Total Warnings**: 0
- **Total Issues**: 16
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 0
- **Files with Issues**: 7

## Error Statistics

**Total Errors**: 16

### Error Type Breakdown

- **error[E0061]**: 8 errors
- **error[E0599]**: 5 errors
- **error[E0609]**: 3 errors

### Files with Errors (Top 10)

- `src\core\symbol\symbol_table.rs`: 3 errors
- `src\query\planner\connector.rs`: 3 errors
- `src\query\validator\go_validator.rs`: 2 errors
- `src\query\validator\insert_edges_validator.rs`: 2 errors
- `src\query\validator\insert_vertices_validator.rs`: 2 errors
- `src\query\validator\limit_validator.rs`: 2 errors
- `src\query\validator\lookup_validator.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0061]: this function takes 1 argument but 0 arguments were supplied

**Total Occurrences**: 8  
**Unique Files**: 6

#### `src\query\planner\connector.rs`: 3 occurrences

- Line 139: this function takes 1 argument but 0 arguments were supplied
- Line 153: this function takes 1 argument but 0 arguments were supplied
- Line 192: this function takes 1 argument but 0 arguments were supplied

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 335: this function takes 1 argument but 0 arguments were supplied

#### `src\query\validator\lookup_validator.rs`: 1 occurrences

- Line 443: this function takes 1 argument but 0 arguments were supplied

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 515: this function takes 1 argument but 0 arguments were supplied

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 393: this function takes 1 argument but 0 arguments were supplied

#### `src\query\validator\limit_validator.rs`: 1 occurrences

- Line 279: this function takes 1 argument but 0 arguments were supplied

### error[E0599]: no method named `set_rctx` found for struct `query_context::QueryContext` in the current scope

**Total Occurrences**: 5  
**Unique Files**: 5

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 516: no method named `set_rctx` found for struct `query_context::QueryContext` in the current scope

#### `src\query\validator\limit_validator.rs`: 1 occurrences

- Line 280: no method named `set_rctx` found for struct `query_context::QueryContext` in the current scope

#### `src\query\validator\lookup_validator.rs`: 1 occurrences

- Line 444: no method named `set_rctx` found for struct `query_context::QueryContext` in the current scope

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 394: no method named `set_rctx` found for struct `query_context::QueryContext` in the current scope

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 336: no method named `set_rctx` found for struct `query_context::QueryContext` in the current scope

### error[E0609]: no field `name` on type `variable::VariableInfo`: unknown field

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\core\symbol\symbol_table.rs`: 3 occurrences

- Line 364: no field `name` on type `variable::VariableInfo`: unknown field
- Line 376: no field `name` on type `variable::VariableInfo`: unknown field
- Line 389: no field `name` on type `variable::VariableInfo`: unknown field

