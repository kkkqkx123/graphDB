# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 103
- **Total Warnings**: 21
- **Total Issues**: 124
- **Unique Error Patterns**: 8
- **Unique Warning Patterns**: 12
- **Files with Issues**: 36

## Error Statistics

**Total Errors**: 103

### Error Type Breakdown

- **error[E0061]**: 103 errors

### Files with Errors (Top 10)

- `src\query\executor\data_processing\graph_traversal\tests.rs`: 13 errors
- `src\query\executor\admin\tag\tests.rs`: 9 errors
- `src\query\executor\admin\edge\tests.rs`: 9 errors
- `src\query\executor\result_processing\transformations\pattern_apply.rs`: 7 errors
- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 7 errors
- `src\query\executor\admin\space\tests.rs`: 6 errors
- `src\query\executor\admin\index\show_edge_index_status.rs`: 4 errors
- `src\query\executor\admin\index\show_tag_index_status.rs`: 4 errors
- `src\query\executor\logic\loops.rs`: 4 errors
- `src\query\executor\admin\user\drop_user.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 21

### Warning Type Breakdown

- **warning**: 21 warnings

### Files with Warnings (Top 10)

- `src\query\validator\strategies\aggregate_strategy.rs`: 4 warnings
- `src\query\optimizer\strategy\materialization.rs`: 2 warnings
- `src\query\planner\rewrite\visitor.rs`: 2 warnings
- `src\query\planner\plan\core\nodes\data_processing_node.rs`: 2 warnings
- `src\query\optimizer\analysis\expression.rs`: 2 warnings
- `src\query\optimizer\strategy\subquery_unnesting.rs`: 1 warnings
- `src\query\executor\result_processing\projection.rs`: 1 warnings
- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 1 warnings
- `src\query\executor\result_processing\transformations\pattern_apply.rs`: 1 warnings
- `src\query\executor\data_access.rs`: 1 warnings

## Detailed Error Categorization

### error[E0061]: this function takes 1 argument but 0 arguments were supplied

**Total Occurrences**: 103  
**Unique Files**: 24

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 13 occurrences

- Line 82: this function takes 6 arguments but 5 arguments were supplied
- Line 101: this function takes 7 arguments but 6 arguments were supplied
- Line 120: this function takes 7 arguments but 6 arguments were supplied
- ... 10 more occurrences in this file

#### `src\query\executor\admin\edge\tests.rs`: 9 occurrences

- Line 28: this function takes 4 arguments but 3 arguments were supplied
- Line 45: this function takes 4 arguments but 3 arguments were supplied
- Line 64: this function takes 4 arguments but 3 arguments were supplied
- ... 6 more occurrences in this file

#### `src\query\executor\admin\tag\tests.rs`: 9 occurrences

- Line 27: this function takes 4 arguments but 3 arguments were supplied
- Line 44: this function takes 4 arguments but 3 arguments were supplied
- Line 63: this function takes 4 arguments but 3 arguments were supplied
- ... 6 more occurrences in this file

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 7 occurrences

- Line 564: this function takes 1 argument but 0 arguments were supplied
- Line 608: this function takes 1 argument but 0 arguments were supplied
- Line 670: this function takes 1 argument but 0 arguments were supplied
- ... 4 more occurrences in this file

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 7 occurrences

- Line 366: this function takes 1 argument but 0 arguments were supplied
- Line 400: this function takes 1 argument but 0 arguments were supplied
- Line 409: this function takes 9 arguments but 8 arguments were supplied
- ... 4 more occurrences in this file

#### `src\query\executor\admin\space\tests.rs`: 6 occurrences

- Line 20: this function takes 4 arguments but 3 arguments were supplied
- Line 35: this function takes 4 arguments but 3 arguments were supplied
- Line 50: this function takes 4 arguments but 3 arguments were supplied
- ... 3 more occurrences in this file

#### `src\query\executor\admin\index\show_edge_index_status.rs`: 4 occurrences

- Line 157: this function takes 4 arguments but 3 arguments were supplied
- Line 168: this function takes 5 arguments but 4 arguments were supplied
- Line 184: this function takes 4 arguments but 3 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\admin\user\drop_user.rs`: 4 occurrences

- Line 111: this function takes 4 arguments but 3 arguments were supplied
- Line 126: this function takes 4 arguments but 3 arguments were supplied
- Line 137: this function takes 4 arguments but 3 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\admin\index\show_tag_index_status.rs`: 4 occurrences

- Line 157: this function takes 4 arguments but 3 arguments were supplied
- Line 168: this function takes 5 arguments but 4 arguments were supplied
- Line 184: this function takes 4 arguments but 3 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\logic\loops.rs`: 4 occurrences

- Line 700: this function takes 4 arguments but 3 arguments were supplied
- Line 706: this function takes 6 arguments but 5 arguments were supplied
- Line 724: this function takes 4 arguments but 3 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\admin\user\create_user.rs`: 4 occurrences

- Line 114: this function takes 4 arguments but 3 arguments were supplied
- Line 131: this function takes 4 arguments but 3 arguments were supplied
- Line 144: this function takes 4 arguments but 3 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\executor\admin\space\switch_space.rs`: 3 occurrences

- Line 97: this function takes 4 arguments but 3 arguments were supplied
- Line 108: this function takes 4 arguments but 3 arguments were supplied
- Line 122: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\user\revoke_role.rs`: 3 occurrences

- Line 104: this function takes 5 arguments but 4 arguments were supplied
- Line 120: this function takes 5 arguments but 4 arguments were supplied
- Line 139: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\user\alter_user.rs`: 3 occurrences

- Line 97: this function takes 4 arguments but 3 arguments were supplied
- Line 113: this function takes 4 arguments but 3 arguments were supplied
- Line 128: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\join\cross_join.rs`: 3 occurrences

- Line 313: this function takes 5 arguments but 4 arguments were supplied
- Line 395: this function takes 5 arguments but 4 arguments were supplied
- Line 444: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\user\grant_role.rs`: 3 occurrences

- Line 114: this function takes 6 arguments but 5 arguments were supplied
- Line 131: this function takes 6 arguments but 5 arguments were supplied
- Line 151: this function takes 6 arguments but 5 arguments were supplied

#### `src\query\executor\data_processing\set_operations\union.rs`: 3 occurrences

- Line 142: this function takes 5 arguments but 4 arguments were supplied
- Line 194: this function takes 5 arguments but 4 arguments were supplied
- Line 234: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\user\change_password.rs`: 3 occurrences

- Line 112: this function takes 6 arguments but 5 arguments were supplied
- Line 133: this function takes 6 arguments but 5 arguments were supplied
- Line 153: this function takes 6 arguments but 5 arguments were supplied

#### `src\query\executor\admin\space\alter_space.rs`: 3 occurrences

- Line 122: this function takes 5 arguments but 4 arguments were supplied
- Line 134: this function takes 5 arguments but 4 arguments were supplied
- Line 149: this function takes 5 arguments but 4 arguments were supplied

#### `src\query\executor\admin\space\clear_space.rs`: 3 occurrences

- Line 96: this function takes 4 arguments but 3 arguments were supplied
- Line 107: this function takes 4 arguments but 3 arguments were supplied
- Line 121: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\admin\index\tests.rs`: 2 occurrences

- Line 305: this function takes 4 arguments but 3 arguments were supplied
- Line 329: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\algorithms\multi_shortest_path.rs`: 1 occurrences

- Line 664: this function takes 8 arguments but 7 arguments were supplied

#### `src\query\executor\result_processing\transformations\assign.rs`: 1 occurrences

- Line 178: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\algorithms\subgraph_executor.rs`: 1 occurrences

- Line 465: this function takes 5 arguments but 4 arguments were supplied

## Detailed Warning Categorization

### warning: unused doc comment

**Total Occurrences**: 21  
**Unique Files**: 14

#### `src\query\validator\strategies\aggregate_strategy.rs`: 4 occurrences

- Line 5: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`
- Line 7: unused imports: `ExpressionMeta` and `Expression`
- Line 8: unused import: `BinaryOperator`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\core\nodes\data_processing_node.rs`: 2 occurrences

- Line 550: unused doc comment
- Line 5: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\planner\rewrite\visitor.rs`: 2 occurrences

- Line 19: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`
- Line 63: unused imports: `InsertEdgesNode` and `InsertVerticesNode`

#### `src\query\optimizer\analysis\expression.rs`: 2 occurrences

- Line 296: unused variable: `func`: help: if this is intentional, prefix it with an underscore: `_func`
- Line 392: unused variable: `func`: help: if this is intentional, prefix it with an underscore: `_func`

#### `src\query\optimizer\strategy\materialization.rs`: 2 occurrences

- Line 33: unused import: `ReferenceCountAnalysis`
- Line 475: unused variable: `optimizer`: help: if this is intentional, prefix it with an underscore: `_optimizer`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 5: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 5: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 75: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 1 occurrences

- Line 582: unused variable: `optimizer`: help: if this is intentional, prefix it with an underscore: `_optimizer`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 411: unused import: `Expression`

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 7: unused import: `crate::query::executor::expression::DefaultExpressionContext`

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 64: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\core\types\expression\utils.rs`: 1 occurrences

- Line 7: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::core::types::expression::context::ExpressionAnalysisContext`

