# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 63
- **Total Warnings**: 30
- **Total Issues**: 93
- **Unique Error Patterns**: 22
- **Unique Warning Patterns**: 6
- **Files with Issues**: 18

## Error Statistics

**Total Errors**: 63

### Error Type Breakdown

- **error[E0061]**: 40 errors
- **error[E0560]**: 14 errors
- **error[E0308]**: 7 errors
- **error[E0599]**: 1 errors
- **error[E0603]**: 1 errors

### Files with Errors (Top 10)

- `src\query\validator\insert_edges_validator.rs`: 13 errors
- `src\query\validator\insert_vertices_validator.rs`: 12 errors
- `src\query\validator\find_path_validator.rs`: 11 errors
- `src\query\validator\get_subgraph_validator.rs`: 7 errors
- `src\query\validator\limit_validator.rs`: 7 errors
- `src\query\validator\update_validator.rs`: 4 errors
- `src\query\validator\go_validator.rs`: 4 errors
- `src\query\validator\lookup_validator.rs`: 3 errors
- `src\query\validator\use_validator.rs`: 1 errors
- `src\query\validator\fetch_edges_validator.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 30

### Warning Type Breakdown

- **warning**: 30 warnings

### Files with Warnings (Top 10)

- `src\query\validator\find_path_validator.rs`: 4 warnings
- `src\query\validator\get_subgraph_validator.rs`: 3 warnings
- `src\query\validator\fetch_edges_validator.rs`: 2 warnings
- `src\query\validator\sequential_validator.rs`: 2 warnings
- `src\query\validator\delete_validator.rs`: 2 warnings
- `src\query\validator\fetch_vertices_validator.rs`: 2 warnings
- `src\query\validator\pipe_validator.rs`: 2 warnings
- `src\query\validator\match_validator.rs`: 2 warnings
- `src\query\validator\order_by_validator.rs`: 2 warnings
- `src\query\validator\validator_enum.rs`: 2 warnings

## Detailed Error Categorization

### error[E0061]: this method takes 0 arguments but 1 argument was supplied

**Total Occurrences**: 40  
**Unique Files**: 6

#### `src\query\validator\insert_edges_validator.rs`: 13 occurrences

- Line 409: this method takes 1 argument but 2 arguments were supplied
- Line 430: this method takes 1 argument but 2 arguments were supplied
- Line 451: this method takes 1 argument but 2 arguments were supplied
- ... 10 more occurrences in this file

#### `src\query\validator\insert_vertices_validator.rs`: 12 occurrences

- Line 367: this method takes 1 argument but 2 arguments were supplied
- Line 388: this method takes 1 argument but 2 arguments were supplied
- Line 409: this method takes 1 argument but 2 arguments were supplied
- ... 9 more occurrences in this file

#### `src\query\validator\limit_validator.rs`: 7 occurrences

- Line 276: this method takes 1 argument but 2 arguments were supplied
- Line 290: this method takes 1 argument but 2 arguments were supplied
- Line 304: this method takes 1 argument but 2 arguments were supplied
- ... 4 more occurrences in this file

#### `src\query\validator\go_validator.rs`: 4 occurrences

- Line 541: this method takes 1 argument but 2 arguments were supplied
- Line 557: this method takes 1 argument but 2 arguments were supplied
- Line 590: this method takes 1 argument but 2 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\validator\lookup_validator.rs`: 3 occurrences

- Line 484: this method takes 1 argument but 2 arguments were supplied
- Line 495: this method takes 1 argument but 2 arguments were supplied
- Line 507: this method takes 1 argument but 2 arguments were supplied

#### `src\query\validator\use_validator.rs`: 1 occurrences

- Line 366: this method takes 0 arguments but 1 argument was supplied

### error[E0560]: struct `stmt::SubgraphStmt` has no field named `vertex_filters`: `stmt::SubgraphStmt` does not have this field

**Total Occurrences**: 14  
**Unique Files**: 2

#### `src\query\validator\find_path_validator.rs`: 8 occurrences

- Line 352: struct `stmt::FindPathStmt` has no field named `path_pattern`: `stmt::FindPathStmt` does not have this field
- Line 353: struct `stmt::FindPathStmt` has no field named `src_vertices`: `stmt::FindPathStmt` does not have this field
- Line 354: struct `stmt::FindPathStmt` has no field named `dst_vertices`: `stmt::FindPathStmt` does not have this field
- ... 5 more occurrences in this file

#### `src\query\validator\get_subgraph_validator.rs`: 6 occurrences

- Line 305: struct `stmt::SubgraphStmt` has no field named `vertex_filters`: `stmt::SubgraphStmt` does not have this field
- Line 306: struct `stmt::SubgraphStmt` has no field named `edge_filters`: `stmt::SubgraphStmt` does not have this field
- Line 307: struct `stmt::SubgraphStmt` has no field named `edge_types`: `stmt::SubgraphStmt` does not have this field
- ... 3 more occurrences in this file

### error[E0308]: mismatched types: types differ in mutability

**Total Occurrences**: 7  
**Unique Files**: 3

#### `src\query\validator\update_validator.rs`: 4 occurrences

- Line 808: mismatched types: types differ in mutability
- Line 823: mismatched types: types differ in mutability
- Line 838: mismatched types: types differ in mutability
- ... 1 more occurrences in this file

#### `src\query\validator\find_path_validator.rs`: 2 occurrences

- Line 431: mismatched types: expected `&Option<YieldClause>`, found `&[_; 0]`
- Line 444: mismatched types: expected `&Option<YieldClause>`, found `&Vec<(Expression, Option<String>)>`

#### `src\query\validator\get_subgraph_validator.rs`: 1 occurrences

- Line 304: mismatched types: expected `Steps`, found `Option<({integer}, Option<{integer}>)>`

### error[E0599]: no method named `validate_yield_clause` found for struct `fetch_edges_validator::FetchEdgesValidator` in the current scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\fetch_edges_validator.rs`: 1 occurrences

- Line 432: no method named `validate_yield_clause` found for struct `fetch_edges_validator::FetchEdgesValidator` in the current scope

### error[E0603]: struct import `PathPattern` is private: private struct import

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\find_path_validator.rs`: 1 occurrences

- Line 342: struct import `PathPattern` is private: private struct import

## Detailed Warning Categorization

### warning: unused import: `crate::query::parser::ast::Stmt`

**Total Occurrences**: 30  
**Unique Files**: 17

#### `src\query\validator\find_path_validator.rs`: 4 occurrences

- Line 17: unused import: `std::collections::HashMap`
- Line 23: unused import: `crate::query::context::ast::AstContext`
- Line 24: unused import: `crate::query::context::execution::QueryContext`
- ... 1 more occurrences in this file

#### `src\query\validator\get_subgraph_validator.rs`: 3 occurrences

- Line 23: unused import: `crate::query::context::ast::AstContext`
- Line 24: unused import: `crate::query::context::execution::QueryContext`
- Line 395: variable does not need to be mutable

#### `src\query\validator\match_validator.rs`: 2 occurrences

- Line 13: unused import: `crate::query::context::ast::AstContext`
- Line 14: unused import: `crate::query::context::execution::QueryContext`

#### `src\query\validator\pipe_validator.rs`: 2 occurrences

- Line 16: unused import: `crate::query::context::ast::AstContext`
- Line 17: unused import: `crate::query::context::execution::QueryContext`

#### `src\query\validator\validator_enum.rs`: 2 occurrences

- Line 11: unused import: `crate::query::context::ast::AstContext`
- Line 12: unused import: `crate::query::context::execution::QueryContext`

#### `src\query\validator\fetch_edges_validator.rs`: 2 occurrences

- Line 21: unused import: `crate::query::context::ast::AstContext`
- Line 22: unused import: `crate::query::context::execution::QueryContext`

#### `src\query\validator\sequential_validator.rs`: 2 occurrences

- Line 16: unused import: `crate::query::context::ast::AstContext`
- Line 17: unused import: `crate::query::context::execution::QueryContext`

#### `src\query\validator\fetch_vertices_validator.rs`: 2 occurrences

- Line 21: unused import: `crate::query::context::ast::AstContext`
- Line 22: unused import: `crate::query::context::execution::QueryContext`

#### `src\query\validator\order_by_validator.rs`: 2 occurrences

- Line 17: unused import: `crate::query::context::ast::AstContext`
- Line 18: unused import: `crate::query::context::execution::QueryContext`

#### `src\query\validator\delete_validator.rs`: 2 occurrences

- Line 21: unused import: `crate::query::context::ast::AstContext`
- Line 22: unused import: `crate::query::context::execution::QueryContext`

#### `src\query\validator\validator_trait.rs`: 1 occurrences

- Line 13: unused import: `crate::query::parser::ast::Stmt`

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 8: unused import: `crate::query::context::execution::QueryContext`

#### `src\query\validator\limit_validator.rs`: 1 occurrences

- Line 8: unused import: `crate::query::context::execution::QueryContext`

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 11: unused import: `crate::query::context::execution::QueryContext`

#### `src\query\validator\lookup_validator.rs`: 1 occurrences

- Line 8: unused import: `crate::query::context::execution::QueryContext`

#### `src\query\validator\update_validator.rs`: 1 occurrences

- Line 644: unused variable: `validated`: help: if this is intentional, prefix it with an underscore: `_validated`

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 8: unused import: `crate::query::context::execution::QueryContext`

