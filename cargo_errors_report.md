# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 21
- **Total Warnings**: 12
- **Total Issues**: 33
- **Unique Error Patterns**: 16
- **Unique Warning Patterns**: 8
- **Files with Issues**: 10

## Error Statistics

**Total Errors**: 21

### Error Type Breakdown

- **error[E0609]**: 7 errors
- **error[E0308]**: 5 errors
- **error[E0559]**: 2 errors
- **error[E0599]**: 2 errors
- **error[E0282]**: 2 errors
- **error[E0560]**: 2 errors
- **error[E0382]**: 1 errors

### Files with Errors (Top 10)

- `src\query\validator\match_validator.rs`: 6 errors
- `src\query\validator\go_validator.rs`: 4 errors
- `src\query\validator\lookup_validator.rs`: 3 errors
- `src\query\validator\limit_validator.rs`: 2 errors
- `src\query\validator\fetch_edges_validator.rs`: 2 errors
- `src\query\validator\delete_validator.rs`: 1 errors
- `src\query\validator\insert_edges_validator.rs`: 1 errors
- `src\query\validator\create_validator.rs`: 1 errors
- `src\query\validator\fetch_vertices_validator.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 12

### Warning Type Breakdown

- **warning**: 12 warnings

### Files with Warnings (Top 10)

- `src\query\validator\match_validator.rs`: 4 warnings
- `src\query\validator\fetch_edges_validator.rs`: 2 warnings
- `src\query\validator\fetch_vertices_validator.rs`: 2 warnings
- `src\query\validator\insert_vertices_validator.rs`: 1 warnings
- `src\query\validator\delete_validator.rs`: 1 warnings
- `src\query\validator\insert_edges_validator.rs`: 1 warnings
- `src\query\validator\go_validator.rs`: 1 warnings

## Detailed Error Categorization

### error[E0609]: no field `variable` on type `&query::parser::ast::pattern::PathPattern`: unknown field

**Total Occurrences**: 7  
**Unique Files**: 4

#### `src\query\validator\match_validator.rs`: 2 occurrences

- Line 197: no field `variable` on type `&query::parser::ast::pattern::PathPattern`: unknown field
- Line 229: no field `variable` on type `&query::parser::ast::pattern::PathPattern`: unknown field

#### `src\query\validator\limit_validator.rs`: 2 occurrences

- Line 206: no field `skip` on type `&stmt::QueryStmt`: unknown field
- Line 206: no field `limit` on type `&stmt::QueryStmt`: unknown field

#### `src\query\validator\lookup_validator.rs`: 2 occurrences

- Line 279: no field `label` on type `&stmt::LookupStmt`: unknown field
- Line 280: no field `label` on type `&stmt::LookupStmt`: unknown field

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 421: no field `expression` on type `stmt::FromClause`: unknown field

### error[E0308]: mismatched types: expected `u64`, found `i32`

**Total Occurrences**: 5  
**Unique Files**: 5

#### `src\query\validator\create_validator.rs`: 1 occurrences

- Line 214: mismatched types: expected `u64`, found `i32`

#### `src\query\validator\fetch_edges_validator.rs`: 1 occurrences

- Line 300: mismatched types: expected `i32`, found `u64`

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 221: mismatched types: expected `Value`, found enum constructor

#### `src\query\validator\lookup_validator.rs`: 1 occurrences

- Line 496: mismatched types: expected `AggregateFunction`, found enum constructor

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 437: mismatched types: expected `i32`, found `u64`

### error[E0560]: struct `stmt::FromClause` has no field named `expression`: `stmt::FromClause` does not have this field

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\validator\go_validator.rs`: 2 occurrences

- Line 510: struct `stmt::FromClause` has no field named `expression`: `stmt::FromClause` does not have this field
- Line 516: struct `stmt::OverClause` has no field named `is_reversible`: `stmt::OverClause` does not have this field

### error[E0559]: variant `core::types::expression::Expression::Property` has no field named `name`: `core::types::expression::Expression::Property` does not have this field

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\validator\fetch_vertices_validator.rs`: 1 occurrences

- Line 256: variant `core::types::expression::Expression::Property` has no field named `name`: `core::types::expression::Expression::Property` does not have this field

#### `src\query\validator\fetch_edges_validator.rs`: 1 occurrences

- Line 318: variant `core::types::expression::Expression::Property` has no field named `name`: `core::types::expression::Expression::Property` does not have this field

### error[E0282]: type annotations needed: cannot infer type for type parameter `T` declared on the enum `Option`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\validator\match_validator.rs`: 2 occurrences

- Line 198: type annotations needed: cannot infer type for type parameter `T` declared on the enum `Option`
- Line 230: type annotations needed: cannot infer type for type parameter `T` declared on the enum `Option`

### error[E0599]: no method named `requires_space` found for struct `match_validator::MatchValidator` in the current scope: method not found in `MatchValidator`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\validator\match_validator.rs`: 2 occurrences

- Line 916: no method named `requires_space` found for struct `match_validator::MatchValidator` in the current scope: method not found in `MatchValidator`
- Line 922: no method named `requires_write_permission` found for struct `match_validator::MatchValidator` in the current scope: method not found in `MatchValidator`

### error[E0382]: use of moved value: `source_type`: value used here after move

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 144: use of moved value: `source_type`: value used here after move

## Detailed Warning Categorization

### warning: unused variable: `name`: help: try ignoring the field: `name: _`

**Total Occurrences**: 12  
**Unique Files**: 7

#### `src\query\validator\match_validator.rs`: 4 occurrences

- Line 347: unused variable: `name`: help: try ignoring the field: `name: _`
- Line 459: unused variable: `skip_expression`: help: if this is intentional, prefix it with an underscore: `_skip_expression`
- Line 460: unused variable: `limit_expression`: help: if this is intentional, prefix it with an underscore: `_limit_expression`
- ... 1 more occurrences in this file

#### `src\query\validator\fetch_vertices_validator.rs`: 2 occurrences

- Line 21: unused import: `crate::core::types::DataType`
- Line 407: variable does not need to be mutable

#### `src\query\validator\fetch_edges_validator.rs`: 2 occurrences

- Line 21: unused import: `crate::core::types::DataType`
- Line 462: variable does not need to be mutable

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 7: unused imports: `AggregateFunction`, `BinaryOperator`, and `UnaryOperator`

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 643: variable does not need to be mutable

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 105: unused variable: `var_name`: help: if this is intentional, prefix it with an underscore: `_var_name`

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 7: unused import: `crate::core::types::DataType`

