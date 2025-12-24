# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 31
- **Total Warnings**: 0
- **Total Issues**: 31
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 0
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 31

### Error Type Breakdown

- **error[E0061]**: 26 errors
- **error[E0412]**: 3 errors
- **error[E0609]**: 2 errors

### Files with Errors (Top 10)

- `src\query\parser\ast\expr_parser.rs`: 10 errors
- `src\query\parser\ast\stmt_parser.rs`: 6 errors
- `src\query\parser\parser\expr_parser.rs`: 5 errors
- `src\query\parser\parser\utils.rs`: 4 errors
- `src\query\planner\plan\core\nodes\management_node_traits.rs`: 3 errors
- `src\query\parser\parser\statement_parser.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0061]: this function takes 3 arguments but 2 arguments were supplied

**Total Occurrences**: 26  
**Unique Files**: 5

#### `src\query\parser\ast\expr_parser.rs`: 10 occurrences

- Line 200: this function takes 3 arguments but 2 arguments were supplied
- Line 311: this function takes 3 arguments but 2 arguments were supplied
- Line 325: this function takes 3 arguments but 2 arguments were supplied
- ... 7 more occurrences in this file

#### `src\query\parser\ast\stmt_parser.rs`: 6 occurrences

- Line 286: this function takes 3 arguments but 2 arguments were supplied
- Line 326: this function takes 3 arguments but 2 arguments were supplied
- Line 909: this function takes 3 arguments but 2 arguments were supplied
- ... 3 more occurrences in this file

#### `src\query\parser\parser\expr_parser.rs`: 5 occurrences

- Line 215: this function takes 3 arguments but 2 arguments were supplied
- Line 355: this function takes 3 arguments but 2 arguments were supplied
- Line 369: this function takes 3 arguments but 2 arguments were supplied
- ... 2 more occurrences in this file

#### `src\query\parser\parser\statement_parser.rs`: 3 occurrences

- Line 228: this function takes 3 arguments but 2 arguments were supplied
- Line 318: this function takes 3 arguments but 2 arguments were supplied
- Line 578: this function takes 3 arguments but 2 arguments were supplied

#### `src\query\parser\parser\utils.rs`: 2 occurrences

- Line 35: this function takes 3 arguments but 2 arguments were supplied
- Line 55: this function takes 3 arguments but 2 arguments were supplied

### error[E0412]: cannot find type `PlanNodeVisitError` in this scope: not found in this scope

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\management_node_traits.rs`: 3 occurrences

- Line 43: cannot find type `PlanNodeVisitError` in this scope: not found in this scope
- Line 49: cannot find type `PlanNodeVisitError` in this scope: not found in this scope
- Line 54: cannot find type `PlanNodeVisitError` in this scope: not found in this scope

### error[E0609]: no field `position` on type `token::Token`: unknown field

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\parser\parser\utils.rs`: 2 occurrences

- Line 93: no field `position` on type `token::Token`: unknown field
- Line 94: no field `position` on type `token::Token`: unknown field

