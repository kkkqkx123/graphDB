# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 118
- **Total Warnings**: 0
- **Total Issues**: 118
- **Unique Error Patterns**: 17
- **Unique Warning Patterns**: 0
- **Files with Issues**: 25

## Error Statistics

**Total Errors**: 118

### Error Type Breakdown

- **error[E0425]**: 71 errors
- **error[E0433]**: 18 errors
- **error[E0432]**: 17 errors
- **error[E0223]**: 12 errors

### Files with Errors (Top 10)

- `src\query\parser\ast\stmt.rs`: 48 errors
- `src\core\query_result\iterator.rs`: 15 errors
- `src\query\parser\ast\pattern.rs`: 9 errors
- `src\query\parser\parsing\traversal_parser.rs`: 7 errors
- `src\core\types\expr\common_utils.rs`: 4 errors
- `src\query\executor\data_modification\delete.rs`: 3 errors
- `src\query\executor\data_modification\update.rs`: 3 errors
- `src\query\parser\parsing\clause_parser.rs`: 3 errors
- `src\query\parser\parsing\util_stmt_parser.rs`: 3 errors
- `src\query\parser\core\error.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0425]: cannot find type `contextualExpression` in module `crate::core::types::expr`

**Total Occurrences**: 71  
**Unique Files**: 12

#### `src\query\parser\ast\stmt.rs`: 47 occurrences

- Line 496: cannot find type `ContextualExpression` in this scope: not found in this scope
- Line 502: cannot find type `ContextualExpression` in this scope: not found in this scope
- Line 503: cannot find type `ContextualExpression` in this scope: not found in this scope
- ... 44 more occurrences in this file

#### `src\query\parser\ast\pattern.rs`: 8 occurrences

- Line 36: cannot find type `ContextualExpression` in this scope: not found in this scope
- Line 37: cannot find type `ContextualExpression` in this scope: not found in this scope
- Line 44: cannot find type `ContextualExpression` in this scope: not found in this scope
- ... 5 more occurrences in this file

#### `src\query\parser\parsing\traversal_parser.rs`: 4 occurrences

- Line 512: cannot find type `ContextualExpression` in this scope: not found in this scope
- Line 529: cannot find type `ContextualExpression` in this scope: not found in this scope
- Line 571: cannot find type `ContextualExpression` in this scope: not found in this scope
- ... 1 more occurrences in this file

#### `src\query\executor\data_modification\delete.rs`: 2 occurrences

- Line 26: cannot find type `ContextualExpression` in this scope: not found in this scope
- Line 37: cannot find type `ContextualExpression` in this scope: not found in this scope

#### `src\query\executor\data_modification\update.rs`: 2 occurrences

- Line 34: cannot find type `ContextualExpression` in this scope: not found in this scope
- Line 76: cannot find type `ContextualExpression` in this scope: not found in this scope

#### `src\query\parser\parsing\util_stmt_parser.rs`: 2 occurrences

- Line 392: cannot find type `ContextualExpression` in this scope: not found in this scope
- Line 409: cannot find type `ContextualExpression` in this scope: not found in this scope

#### `src\query\optimizer\strategy\traversal_start.rs`: 1 occurrences

- Line 331: cannot find type `contextualExpression` in module `crate::core::types::expr`

#### `src\core\types\expr\common_utils.rs`: 1 occurrences

- Line 25: cannot find type `PlannerError` in this scope: not found in this scope

#### `src\query\planning\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 105: cannot find type `contextualExpression` in module `crate::core::types::expr`

#### `src\query\executor\data_modification\remove.rs`: 1 occurrences

- Line 26: cannot find type `ContextualExpression` in this scope: not found in this scope

#### `src\query\parser\parsing\clause_parser.rs`: 1 occurrences

- Line 297: cannot find type `ContextualExpression` in this scope: not found in this scope

#### `src\query\parser\parsing\parser.rs`: 1 occurrences

- Line 75: cannot find type `ContextualExpression` in this scope: not found in this scope

### error[E0433]: failed to resolve: could not find `statement` in `embedded`: could not find `statement` in `embedded`

**Total Occurrences**: 18  
**Unique Files**: 10

#### `src\core\query_result\iterator.rs`: 3 occurrences

- Line 154: failed to resolve: could not find `result` in `core`: could not find `result` in `core`
- Line 191: failed to resolve: could not find `result` in `core`: could not find `result` in `core`
- Line 241: failed to resolve: could not find `result` in `core`: could not find `result` in `core`

#### `src\api\embedded\precompiled\statement.rs`: 2 occurrences

- Line 7: failed to resolve: could not find `statement` in `embedded`: could not find `statement` in `embedded`
- Line 8: failed to resolve: could not find `statement` in `embedded`: could not find `statement` in `embedded`

#### `src\core\types\expr\common_utils.rs`: 2 occurrences

- Line 37: failed to resolve: use of undeclared type `PlannerError`: use of undeclared type `PlannerError`
- Line 45: failed to resolve: use of undeclared type `PlannerError`: use of undeclared type `PlannerError`

#### `src\core\error\mod.rs`: 2 occurrences

- Line 160: failed to resolve: could not find `lexer` in `parser`: could not find `lexer` in `parser`
- Line 161: failed to resolve: could not find `lexer` in `parser`: could not find `lexer` in `parser`

#### `src\query\parser\parsing\traversal_parser.rs`: 2 occurrences

- Line 561: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 575: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

#### `src\core\query_result\iterator_enum.rs`: 2 occurrences

- Line 6: failed to resolve: could not find `result` in `core`: could not find `result` in `core`
- Line 7: failed to resolve: could not find `result` in `core`: could not find `result` in `core`

#### `src\query\parser\core\error.rs`: 2 occurrences

- Line 170: failed to resolve: could not find `lexer` in `super`: could not find `lexer` in `super`
- Line 171: failed to resolve: could not find `lexer` in `super`: could not find `lexer` in `super`

#### `src\query\parser\parsing\clause_parser.rs`: 1 occurrences

- Line 39: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

#### `src\core\query_result\result.rs`: 1 occurrences

- Line 1: failed to resolve: could not find `result` in `core`: could not find `result` in `core`

#### `src\query\parser\parsing\stmt_parser.rs`: 1 occurrences

- Line 199: failed to resolve: could not find `contextualExpression` in `expr`: could not find `contextualExpression` in `expr`

### error[E0432]: unresolved import `crate::core::types::expr::contextualExpression`: no `contextualExpression` in `core::types::expr`

**Total Occurrences**: 17  
**Unique Files**: 16

#### `src\query\parser\parsing\parse_context.rs`: 2 occurrences

- Line 5: unresolved import `crate::query::parser::lexer`: could not find `lexer` in `parser`
- Line 6: unresolved import `crate::query::parser::lexer`: could not find `lexer` in `parser`

#### `src\query\parser\ast\stmt.rs`: 1 occurrences

- Line 10: unresolved import `crate::core::types::expr::contextualExpression`: no `contextualExpression` in `core::types::expr`

#### `src\query\parser\parsing\traversal_parser.rs`: 1 occurrences

- Line 5: unresolved import `crate::core::types::expr::contextualExpression`: no `contextualExpression` in `core::types::expr`

#### `src\api\embedded\session.rs`: 1 occurrences

- Line 8: unresolved import `crate::api::embedded::statement`: could not find `statement` in `embedded`

#### `src\query\parser\lexing\lexer.rs`: 1 occurrences

- Line 6: unresolved import `crate::query::parser::lexer`: could not find `lexer` in `parser`

#### `src\core\types\expr\common_utils.rs`: 1 occurrences

- Line 12: unresolved import `crate::query::planning::plannerError`: no `plannerError` in `query::planning`

#### `src\query\executor\data_modification\remove.rs`: 1 occurrences

- Line 9: unresolved import `crate::core::types::expr::contextualExpression`: no `contextualExpression` in `core::types::expr`

#### `src\core\mod.rs`: 1 occurrences

- Line 34: unresolved import `types::expression`: could not find `expression` in `types`

#### `src\query\parser\ast\pattern.rs`: 1 occurrences

- Line 7: unresolved import `crate::core::types::expr::contextualExpression`: no `contextualExpression` in `core::types::expr`

#### `src\query\parser\parsing\clause_parser.rs`: 1 occurrences

- Line 5: unresolved import `crate::core::types::expr::contextualExpression`: no `contextualExpression` in `core::types::expr`

#### `src\query\parser\parsing\parser.rs`: 1 occurrences

- Line 3: unresolved import `crate::core::types::expr::contextualExpression`: no `contextualExpression` in `core::types::expr`

#### `src\query\parser\parsing\util_stmt_parser.rs`: 1 occurrences

- Line 5: unresolved import `crate::core::types::expr::contextualExpression`: no `contextualExpression` in `core::types::expr`

#### `src\api\embedded\c_api\statement.rs`: 1 occurrences

- Line 11: unresolved import `crate::api::embedded::statement`: could not find `statement` in `embedded`

#### `src\query\executor\base\execution_result.rs`: 1 occurrences

- Line 6: unresolved import `crate::core::result`: could not find `result` in `core`

#### `src\query\executor\data_modification\update.rs`: 1 occurrences

- Line 16: unresolved import `crate::core::types::expr::contextualExpression`: no `contextualExpression` in `core::types::expr`

#### `src\query\executor\data_modification\delete.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::types::expr::contextualExpression`: no `contextualExpression` in `core::types::expr`

### error[E0223]: ambiguous associated type

**Total Occurrences**: 12  
**Unique Files**: 1

#### `src\core\query_result\iterator.rs`: 12 occurrences

- Line 157: ambiguous associated type
- Line 167: ambiguous associated type
- Line 176: ambiguous associated type
- ... 9 more occurrences in this file

