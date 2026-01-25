# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 6
- **Total Warnings**: 32
- **Total Issues**: 38
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 30
- **Files with Issues**: 25

## Error Statistics

**Total Errors**: 6

### Error Type Breakdown

- **error[E0425]**: 5 errors
- **error[E0432]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\factory.rs`: 5 errors
- `src\query\planner\plan\core\nodes\admin_node.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 32

### Warning Type Breakdown

- **warning**: 32 warnings

### Files with Warnings (Top 10)

- `src\query\executor\factory.rs`: 4 warnings
- `src\query\planner\plan\core\nodes\admin_node.rs`: 3 warnings
- `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 warnings
- `src\query\planner\statements\match_planner.rs`: 2 warnings
- `src\query\executor\admin\space\create_space.rs`: 1 warnings
- `src\query\executor\base\executor_base.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 warnings
- `src\query\executor\admin\tag\alter_tag.rs`: 1 warnings
- `src\query\executor\admin\mod.rs`: 1 warnings
- `src\query\executor\admin\data\update.rs`: 1 warnings

## Detailed Error Categorization

### error[E0425]: cannot find value `node` in this scope: not found in this scope

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 5 occurrences

- Line 818: cannot find value `node` in this scope: not found in this scope
- Line 874: cannot find value `node` in this scope: not found in this scope
- Line 930: cannot find value `node` in this scope: not found in this scope
- ... 2 more occurrences in this file

### error[E0432]: unresolved import `crate::query::planner::PlanNodeID`: no `PlanNodeID` in `query::planner`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\admin_node.rs`: 1 occurrences

- Line 10: unresolved import `crate::query::planner::PlanNodeID`: no `PlanNodeID` in `query::planner`

## Detailed Warning Categorization

### warning: unused doc comment: rustdoc does not generate documentation for macro invocations

**Total Occurrences**: 32  
**Unique Files**: 25

#### `src\query\executor\factory.rs`: 4 occurrences

- Line 22: unused imports: `MultiShortestPathExecutor` and `ShortestPathExecutor`
- Line 45: unused imports: `EdgeAlterInfo`, `EdgeManageInfo`, `IndexManageInfo`, `SpaceManageInfo`, `TagAlterInfo`, and `TagManageInfo`
- Line 836: unused import: `AlterTagOp`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\core\nodes\admin_node.rs`: 3 occurrences

- Line 6: unused import: `SingleInputNode`
- Line 9: unused import: `crate::query::planner::plan::core::explain::PlanNodeDescription`
- Line 11: unused import: `std::fmt`

#### `src\query\planner\statements\match_planner.rs`: 2 occurrences

- Line 96: unused variable: `match_ctx`: help: if this is intentional, prefix it with an underscore: `_match_ctx`
- Line 157: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 207: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 207: variable does not need to be mutable

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 1: unused import: `crate::core::error::DBError`

#### `src\query\executor\admin\edge\alter_edge.rs`: 1 occurrences

- Line 9: unused import: `crate::core::types::graph_schema::PropertyType`

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 9: unused import: `ExecutionContext`

#### `src\query\parser\ast\stmt.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Value`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 55: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 1 occurrences

- Line 10: unused macro definition: `impl_graph_traversal_executor`

#### `src\query\executor\admin\space\create_space.rs`: 1 occurrences

- Line 8: unused import: `Value`

#### `src\query\executor\admin\tag\alter_tag.rs`: 1 occurrences

- Line 9: unused import: `crate::core::types::graph_schema::PropertyType`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\query\context\managers\schema_traits.rs`: 1 occurrences

- Line 247: unexpected `cfg` condition value: `schema-manager-default`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 275: unused imports: `AlterEdgeOp` and `AlterTagOp`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 42: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\executor\admin\mod.rs`: 1 occurrences

- Line 13: unused import: `crate::storage::StorageEngine`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 15: unused import: `std::collections::HashMap`

#### `src\query\validator\validation_interface.rs`: 1 occurrences

- Line 4: unused imports: `DBError`, `QueryError`, `ValidationError as CoreValidationError`, and `ValidationErrorType as CoreValidationErrorType`

#### `src\query\executor\base\executor_base.rs`: 1 occurrences

- Line 9: unused import: `crate::core::error::DBError`

#### `src\query\executor\admin\data\update.rs`: 1 occurrences

- Line 8: unused imports: `UpdateOp` and `UpdateTarget`

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 13: unused import: `crate::query::optimizer::rule_traits::BaseOptRule`

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 7: unused import: `Vertex`

