# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 180
- **Total Warnings**: 0
- **Total Issues**: 180
- **Unique Error Patterns**: 20
- **Unique Warning Patterns**: 0
- **Files with Issues**: 52

## Error Statistics

**Total Errors**: 180

### Error Type Breakdown

- **error[E0061]**: 115 errors
- **error**: 54 errors
- **error[E0412]**: 8 errors
- **error[E0308]**: 1 errors
- **error[E0599]**: 1 errors
- **error[E0782]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\factory.rs`: 27 errors
- `src\query\parser\parser\parser.rs`: 10 errors
- `src\query\planner\statements\clauses\yield_planner.rs`: 9 errors
- `src\query\executor\admin\tag\tests.rs`: 8 errors
- `src\query\executor\admin\edge\tests.rs`: 8 errors
- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 8 errors
- `src\query\executor\admin\space\tests.rs`: 6 errors
- `src\query\validator\clauses\yield_validator.rs`: 5 errors
- `src\query\validator\clauses\return_validator.rs`: 5 errors
- `src\query\validator\clauses\with_validator.rs`: 5 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0061]: this function takes 4 arguments but 3 arguments were supplied

**Total Occurrences**: 115  
**Unique Files**: 31

#### `src\query\executor\factory.rs`: 27 occurrences

- Line 605: this function takes 2 arguments but 1 argument was supplied
- Line 1267: this function takes 4 arguments but 3 arguments were supplied
- Line 1273: this function takes 4 arguments but 3 arguments were supplied
- ... 24 more occurrences in this file

#### `src\query\executor\admin\tag\tests.rs`: 8 occurrences

- Line 27: this function takes 4 arguments but 3 arguments were supplied
- Line 44: this function takes 4 arguments but 3 arguments were supplied
- Line 63: this function takes 4 arguments but 3 arguments were supplied
- ... 5 more occurrences in this file

#### `src\query\executor\admin\edge\tests.rs`: 8 occurrences

- Line 28: this function takes 4 arguments but 3 arguments were supplied
- Line 45: this function takes 4 arguments but 3 arguments were supplied
- Line 64: this function takes 4 arguments but 3 arguments were supplied
- ... 5 more occurrences in this file

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

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 4 occurrences

- Line 365: this function takes 1 argument but 0 arguments were supplied
- Line 397: this function takes 1 argument but 0 arguments were supplied
- Line 430: this function takes 1 argument but 0 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\admin\query_management\show_stats.rs`: 3 occurrences

- Line 46: this function takes 4 arguments but 3 arguments were supplied
- Line 53: this function takes 4 arguments but 3 arguments were supplied
- Line 60: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\change_password.rs`: 3 occurrences

- Line 112: this function takes 6 arguments but 5 arguments were supplied
- Line 133: this function takes 6 arguments but 5 arguments were supplied
- Line 153: this function takes 6 arguments but 5 arguments were supplied

#### `src\query\executor\admin\user\alter_user.rs`: 3 occurrences

- Line 97: this function takes 4 arguments but 3 arguments were supplied
- Line 113: this function takes 4 arguments but 3 arguments were supplied
- Line 128: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\switch_space.rs`: 3 occurrences

- Line 97: this function takes 4 arguments but 3 arguments were supplied
- Line 108: this function takes 4 arguments but 3 arguments were supplied
- Line 122: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\join\cross_join.rs`: 3 occurrences

- Line 313: this function takes 5 arguments but 4 arguments were supplied
- Line 395: this function takes 5 arguments but 4 arguments were supplied
- Line 444: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\user\revoke_role.rs`: 3 occurrences

- Line 104: this function takes 5 arguments but 4 arguments were supplied
- Line 120: this function takes 5 arguments but 4 arguments were supplied
- Line 139: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\space\alter_space.rs`: 3 occurrences

- Line 122: this function takes 5 arguments but 4 arguments were supplied
- Line 134: this function takes 5 arguments but 4 arguments were supplied
- Line 149: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\data_processing\set_operations\union.rs`: 3 occurrences

- Line 142: this function takes 5 arguments but 4 arguments were supplied
- Line 194: this function takes 5 arguments but 4 arguments were supplied
- Line 234: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\user\grant_role.rs`: 3 occurrences

- Line 114: this function takes 6 arguments but 5 arguments were supplied
- Line 131: this function takes 6 arguments but 5 arguments were supplied
- Line 151: this function takes 6 arguments but 5 arguments were supplied

#### `src\query\executor\admin\user\create_user.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 33: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\drop_user.rs`: 2 occurrences

- Line 24: this function takes 4 arguments but 3 arguments were supplied
- Line 32: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\analyze.rs`: 2 occurrences

- Line 46: this function takes 4 arguments but 3 arguments were supplied
- Line 55: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\show_edge_index_status.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 38: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\show_tag_index_status.rs`: 2 occurrences

- Line 25: this function takes 4 arguments but 3 arguments were supplied
- Line 38: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\tests.rs`: 2 occurrences

- Line 305: this function takes 4 arguments but 3 arguments were supplied
- Line 329: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\edge\drop_edge.rs`: 1 occurrences

- Line 42: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 943: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\space\drop_space.rs`: 1 occurrences

- Line 35: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\algorithms\multi_shortest_path.rs`: 1 occurrences

- Line 88: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\tag\drop_tag.rs`: 1 occurrences

- Line 42: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\algorithms\subgraph_executor.rs`: 1 occurrences

- Line 465: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\space\clear_space.rs`: 1 occurrences

- Line 23: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\result_processing\transformations\assign.rs`: 1 occurrences

- Line 178: this function takes 4 arguments but 3 arguments were supplied

### error: unknown start of token: \

**Total Occurrences**: 54  
**Unique Files**: 12

#### `src\query\parser\parser\parser.rs`: 10 occurrences

- Line 6: unknown start of token: \
- Line 6: unknown start of token: \
- Line 6: unknown start of token: \
- ... 7 more occurrences in this file

#### `src\query\planner\statements\clauses\yield_planner.rs`: 9 occurrences

- Line 9: unknown start of token: \
- Line 9: unknown start of token: \
- Line 9: unknown start of token: \
- ... 6 more occurrences in this file

#### `src\query\validator\clauses\yield_validator.rs`: 5 occurrences

- Line 17: unknown start of token: \
- Line 17: unknown start of token: \
- Line 18: unknown start of token: \
- ... 2 more occurrences in this file

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

#### `src\query\validator\strategies\clause_strategy.rs`: 4 occurrences

- Line 10: unknown start of token: \
- Line 10: unknown start of token: \
- Line 11: unknown start of token: \
- ... 1 more occurrences in this file

#### `src\query\planner\rewrite\projection_pushdown\push_project_down.rs`: 4 occurrences

- Line 28: unknown start of token: \
- Line 29: unknown start of token: \
- Line 30: unknown start of token: \
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\pagination_strategy.rs`: 2 occurrences

- Line 9: unknown start of token: \
- Line 9: expected one of `::`, `;`, or `as`, found `structs`: expected one of `::`, `;`, or `as`

#### `src\query\validator\clauses\order_by_validator.rs`: 2 occurrences

- Line 19: unknown start of token: \
- Line 19: expected one of `::`, `;`, or `as`, found `validator_trait`: expected one of `::`, `;`, or `as`

#### `src\query\planner\rewrite\pattern.rs`: 2 occurrences

- Line 8: unknown start of token: \
- Line 8: expected one of `::`, `;`, or `as`, found `plan`: expected one of `::`, `;`, or `as`

#### `src\query\validator\strategies\helpers\variable_checker.rs`: 2 occurrences

- Line 7: unknown start of token: \
- Line 7: expected one of `::`, `;`, or `as`, found `structs`: expected one of `::`, `;`, or `as`

### error[E0412]: cannot find type `ExpressionContext` in this scope: not found in this scope

**Total Occurrences**: 8  
**Unique Files**: 8

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 161: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 42: cannot find type `ExpressionContextStruct` in this scope: help: a trait with a similar name exists: `ExpressionContext`

#### `src\query\executor\admin\index\edge_index.rs`: 1 occurrences

- Line 332: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 129: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 153: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\base\result_processor.rs`: 1 occurrences

- Line 254: cannot find type `ExpressionAnalysisContext` in this scope: not found in this scope

#### `src\query\executor\admin\index\tag_index.rs`: 1 occurrences

- Line 332: cannot find type `ExpressionContext` in this scope: not found in this scope

#### `src\query\executor\admin\index\tests.rs`: 1 occurrences

- Line 16: cannot find type `ExpressionContext` in this scope: not found in this scope

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

### error[E0599]: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 863: no method named `set_variable` found for struct `DefaultExpressionContext` in the current scope

