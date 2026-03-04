# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 160
- **Total Warnings**: 20
- **Total Issues**: 180
- **Unique Error Patterns**: 23
- **Unique Warning Patterns**: 12
- **Files with Issues**: 61

## Error Statistics

**Total Errors**: 160

### Error Type Breakdown

- **error[E0061]**: 118 errors
- **error**: 22 errors
- **error[E0412]**: 11 errors
- **error[E0432]**: 5 errors
- **error[E0433]**: 1 errors
- **error[E0308]**: 1 errors
- **error[E0782]**: 1 errors
- **error[E0599]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\factory.rs`: 27 errors
- `src\query\executor\result_processing\transformations\pattern_apply.rs`: 9 errors
- `src\query\executor\admin\edge\tests.rs`: 8 errors
- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 8 errors
- `src\query\executor\admin\tag\tests.rs`: 8 errors
- `src\query\executor\admin\space\tests.rs`: 6 errors
- `src\query\validator\clauses\return_validator.rs`: 5 errors
- `src\query\validator\clauses\with_validator.rs`: 5 errors
- `src\query\executor\data_processing\graph_traversal\factory.rs`: 4 errors
- `src\query\executor\logic\loops.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 20

### Warning Type Breakdown

- **warning**: 20 warnings

### Files with Warnings (Top 10)

- `src\query\validator\strategies\aggregate_strategy.rs`: 4 warnings
- `src\query\executor\result_processing\projection.rs`: 2 warnings
- `src\query\planner\rewrite\visitor.rs`: 2 warnings
- `src\query\optimizer\strategy\materialization.rs`: 2 warnings
- `src\query\planner\plan\core\nodes\data_processing_node.rs`: 2 warnings
- `src\query\optimizer\analysis\expression.rs`: 2 warnings
- `src\query\executor\data_access.rs`: 1 warnings
- `src\query\planner\statements\clauses\order_by_planner.rs`: 1 warnings
- `src\query\planner\statements\clauses\where_clause_planner.rs`: 1 warnings
- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 1 warnings

## Detailed Error Categorization

### error[E0061]: this function takes 4 arguments but 3 arguments were supplied

**Total Occurrences**: 118  
**Unique Files**: 31

#### `src\query\executor\factory.rs`: 27 occurrences

- Line 605: this function takes 2 arguments but 1 argument was supplied
- Line 1267: this function takes 4 arguments but 3 arguments were supplied
- Line 1273: this function takes 4 arguments but 3 arguments were supplied
- ... 24 more occurrences in this file

#### `src\query\executor\admin\edge\tests.rs`: 8 occurrences

- Line 28: this function takes 4 arguments but 3 arguments were supplied
- Line 45: this function takes 4 arguments but 3 arguments were supplied
- Line 64: this function takes 4 arguments but 3 arguments were supplied
- ... 5 more occurrences in this file

#### `src\query\executor\admin\tag\tests.rs`: 8 occurrences

- Line 27: this function takes 4 arguments but 3 arguments were supplied
- Line 44: this function takes 4 arguments but 3 arguments were supplied
- Line 63: this function takes 4 arguments but 3 arguments were supplied
- ... 5 more occurrences in this file

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 7 occurrences

- Line 366: this function takes 1 argument but 0 arguments were supplied
- Line 400: this function takes 1 argument but 0 arguments were supplied
- Line 409: this function takes 9 arguments but 8 arguments were supplied
- ... 4 more occurrences in this file

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 7 occurrences

- Line 564: this function takes 1 argument but 0 arguments were supplied
- Line 608: this function takes 1 argument but 0 arguments were supplied
- Line 670: this function takes 1 argument but 0 arguments were supplied
- ... 4 more occurrences in this file

#### `src\query\executor\admin\space\tests.rs`: 6 occurrences

- Line 20: this function takes 4 arguments but 3 arguments were supplied
- Line 35: this function takes 4 arguments but 3 arguments were supplied
- Line 50: this function takes 4 arguments but 3 arguments were supplied
- ... 3 more occurrences in this file

#### `src\query\executor\logic\loops.rs`: 4 occurrences

- Line 56: this function takes 4 arguments but 3 arguments were supplied
- Line 574: this function takes 4 arguments but 3 arguments were supplied
- Line 694: this function takes 4 arguments but 3 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\factory.rs`: 4 occurrences

- Line 22: this function takes 6 arguments but 5 arguments were supplied
- Line 34: this function takes 7 arguments but 6 arguments were supplied
- Line 46: this function takes 7 arguments but 6 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\admin\space\alter_space.rs`: 3 occurrences

- Line 122: this function takes 5 arguments but 4 arguments were supplied
- Line 134: this function takes 5 arguments but 4 arguments were supplied
- Line 149: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\user\grant_role.rs`: 3 occurrences

- Line 114: this function takes 6 arguments but 5 arguments were supplied
- Line 131: this function takes 6 arguments but 5 arguments were supplied
- Line 151: this function takes 6 arguments but 5 arguments were supplied

#### `src\query\executor\admin\user\alter_user.rs`: 3 occurrences

- Line 97: this function takes 4 arguments but 3 arguments were supplied
- Line 113: this function takes 4 arguments but 3 arguments were supplied
- Line 128: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\change_password.rs`: 3 occurrences

- Line 112: this function takes 6 arguments but 5 arguments were supplied
- Line 133: this function takes 6 arguments but 5 arguments were supplied
- Line 153: this function takes 6 arguments but 5 arguments were supplied

#### `src\query\executor\data_processing\join\cross_join.rs`: 3 occurrences

- Line 313: this function takes 5 arguments but 4 arguments were supplied
- Line 395: this function takes 5 arguments but 4 arguments were supplied
- Line 444: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\user\revoke_role.rs`: 3 occurrences

- Line 104: this function takes 5 arguments but 4 arguments were supplied
- Line 120: this function takes 5 arguments but 4 arguments were supplied
- Line 139: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\data_processing\set_operations\union.rs`: 3 occurrences

- Line 142: this function takes 5 arguments but 4 arguments were supplied
- Line 194: this function takes 5 arguments but 4 arguments were supplied
- Line 234: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\space\switch_space.rs`: 3 occurrences

- Line 97: this function takes 4 arguments but 3 arguments were supplied
- Line 108: this function takes 4 arguments but 3 arguments were supplied
- Line 122: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\query_management\show_stats.rs`: 3 occurrences

- Line 46: this function takes 4 arguments but 3 arguments were supplied
- Line 53: this function takes 4 arguments but 3 arguments were supplied
- Line 60: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\analyze.rs`: 2 occurrences

- Line 46: this function takes 4 arguments but 3 arguments were supplied
- Line 55: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\tests.rs`: 2 occurrences

- Line 305: this function takes 4 arguments but 3 arguments were supplied
- Line 329: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\show_tag_index_status.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 38: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\show_edge_index_status.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 38: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\drop_user.rs`: 2 occurrences

- Line 24: this function takes 4 arguments but 3 arguments were supplied
- Line 32: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\create_user.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 33: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\drop_space.rs`: 1 occurrences

- Line 35: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\edge\drop_edge.rs`: 1 occurrences

- Line 42: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\tag\drop_tag.rs`: 1 occurrences

- Line 42: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\result_processing\transformations\assign.rs`: 1 occurrences

- Line 178: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\algorithms\multi_shortest_path.rs`: 1 occurrences

- Line 88: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\algorithms\subgraph_executor.rs`: 1 occurrences

- Line 465: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 943: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\clear_space.rs`: 1 occurrences

- Line 23: this function takes 4 arguments but 3 arguments were supplied

### error: unknown start of token: \

**Total Occurrences**: 22  
**Unique Files**: 6

#### `src\query\validator\clauses\with_validator.rs`: 5 occurrences

- Line 9: unknown start of token: \
- Line 9: unknown start of token: \
- Line 10: unknown start of token: \
- ... 2 more occurrences in this file

#### `src\query\validator\clauses\return_validator.rs`: 5 occurrences

- Line 8: unknown start of token: \
- Line 8: unknown start of token: \
- Line 9: unknown start of token: \
- ... 2 more occurrences in this file

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 4 occurrences

- Line 36: unknown start of token: \
- Line 36: unknown start of token: \
- Line 36: unknown start of token: \
- ... 1 more occurrences in this file

#### `src\query\planner\rewrite\projection_pushdown\push_project_down.rs`: 4 occurrences

- Line 28: unknown start of token: \
- Line 29: unknown start of token: \
- Line 30: unknown start of token: \
- ... 1 more occurrences in this file

#### `src\query\validator\clauses\order_by_validator.rs`: 2 occurrences

- Line 19: unknown start of token: \
- Line 19: expected one of `::`, `;`, or `as`, found `validator_trait`: expected one of `::`, `;`, or `as`

#### `src\query\validator\strategies\helpers\variable_checker.rs`: 2 occurrences

- Line 7: unknown start of token: \
- Line 7: expected one of `::`, `;`, or `as`, found `structs`: expected one of `::`, `;`, or `as`

### error[E0412]: cannot find type `ExpressionContext` in this scope: not found in this scope

**Total Occurrences**: 11  
**Unique Files**: 10

#### `src\query\parser\parser\parser.rs`: 2 occurrences

- Line 81: cannot find type `ExpressionContext` in this scope: not found in this scope
- Line 86: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\admin\index\tag_index.rs`: 1 occurrences

- Line 332: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 161: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\admin\index\edge_index.rs`: 1 occurrences

- Line 332: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 76: cannot find type `ExpressionAnalysisContext` in this scope

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 153: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 42: cannot find type `ExpressionContextStruct` in this scope: help: a trait with a similar name exists: `ExpressionContext`

#### `src\query\executor\base\result_processor.rs`: 1 occurrences

- Line 254: cannot find type `ExpressionAnalysisContext` in this scope: not found in this scope

#### `src\query\executor\admin\index\tests.rs`: 1 occurrences

- Line 16: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 129: cannot find type `ExpressionContext` in this scope: not found in this scope

### error[E0432]: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

**Total Occurrences**: 5  
**Unique Files**: 4

#### `src\query\validator\clauses\yield_validator.rs`: 2 occurrences

- Line 355: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`
- Line 455: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\validator\strategies\pagination_strategy.rs`: 1 occurrences

- Line 175: unresolved import `crate::core::types::expression::ExpressionContext`: no `ExpressionContext` in `core::types::expression`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 1 occurrences

- Line 268: unresolved import `crate::core::types::ExpressionContext`: no `ExpressionContext` in `core::types`

#### `src\query\planner\rewrite\pattern.rs`: 1 occurrences

- Line 303: unresolved import `crate::core::types::ExpressionContext`: no `ExpressionContext` in `core::types`

### error[E0782]: expected a type, found a trait

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 53: expected a type, found a trait

### error[E0308]: arguments to this function are incorrect

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 322: arguments to this function are incorrect

### error[E0433]: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 375: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

### error[E0599]: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 863: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

## Detailed Warning Categorization

### warning: unused import: `ReferenceCountAnalysis`

**Total Occurrences**: 20  
**Unique Files**: 12

#### `src\query\validator\strategies\aggregate_strategy.rs`: 4 occurrences

- Line 5: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`
- Line 7: unused imports: `ExpressionMeta` and `Expression`
- Line 8: unused import: `BinaryOperator`
- ... 1 more occurrences in this file

#### `src\query\optimizer\strategy\materialization.rs`: 2 occurrences

- Line 33: unused import: `ReferenceCountAnalysis`
- Line 475: unused variable: `optimizer`: help: if this is intentional, prefix it with an underscore: `_optimizer`

#### `src\query\planner\plan\core\nodes\data_processing_node.rs`: 2 occurrences

- Line 550: unused doc comment
- Line 5: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\result_processing\projection.rs`: 2 occurrences

- Line 12: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`
- Line 411: unused import: `Expression`

#### `src\query\planner\rewrite\visitor.rs`: 2 occurrences

- Line 19: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`
- Line 63: unused imports: `InsertEdgesNode` and `InsertVerticesNode`

#### `src\query\optimizer\analysis\expression.rs`: 2 occurrences

- Line 296: unused variable: `func`: help: if this is intentional, prefix it with an underscore: `_func`
- Line 392: unused variable: `func`: help: if this is intentional, prefix it with an underscore: `_func`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 5: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\core\types\expression\utils.rs`: 1 occurrences

- Line 7: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 5: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 64: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 7: unused import: `crate::expression::DefaultExpressionContext`

