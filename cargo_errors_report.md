# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 4
- **Total Warnings**: 68
- **Total Issues**: 72
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 41
- **Files with Issues**: 42

## Error Statistics

**Total Errors**: 4

### Error Type Breakdown

- **error[E0061]**: 4 errors

### Files with Errors (Top 10)

- `src\query\parser\ast\stmt_parser.rs`: 3 errors
- `src\query\parser\ast\expr_parser.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 68

### Warning Type Breakdown

- **warning**: 68 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 12 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 8 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\core\context\mod.rs`: 2 warnings
- `src\query\planner\plan\management\ddl\edge_ops.rs`: 2 warnings
- `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 warnings
- `src\expression\evaluator\traits.rs`: 2 warnings
- `src\query\planner\match_planning\clauses\unwind_planner.rs`: 2 warnings
- `src\query\executor\result_processing\aggregation.rs`: 2 warnings
- `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 warnings

## Detailed Error Categorization

### error[E0061]: this function takes 3 arguments but 2 arguments were supplied

**Total Occurrences**: 4  
**Unique Files**: 2

#### `src\query\parser\ast\stmt_parser.rs`: 3 occurrences

- Line 929: this function takes 3 arguments but 2 arguments were supplied
- Line 942: this function takes 3 arguments but 2 arguments were supplied
- Line 980: this function takes 3 arguments but 2 arguments were supplied

#### `src\query\parser\ast\expr_parser.rs`: 1 occurrences

- Line 319: this function takes 3 arguments but 2 arguments were supplied

## Detailed Warning Categorization

### warning: ambiguous glob re-exports: the name `SessionVariable` in the type namespace is first re-exported here

**Total Occurrences**: 68  
**Unique Files**: 40

#### `src\query\planner\plan\core\nodes\factory.rs`: 12 occurrences

- Line 35: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 31: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- Line 49: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- ... 9 more occurrences in this file

#### `src\expression\evaluator\expression_evaluator.rs`: 8 occurrences

- Line 242: variable does not need to be mutable
- Line 310: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 310: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- ... 5 more occurrences in this file

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 127: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\core\context\mod.rs`: 2 occurrences

- Line 20: ambiguous glob re-exports: the name `SessionVariable` in the type namespace is first re-exported here
- Line 22: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\expression\evaluator\traits.rs`: 2 occurrences

- Line 30: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 30: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\query\planner\ngql\go_planner.rs`: 2 occurrences

- Line 11: unused import: `std::sync::Arc`
- Line 59: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\planner\plan\management\ddl\edge_ops.rs`: 2 occurrences

- Line 5: unused import: `crate::query::context::validate::types::Variable`
- Line 8: unused import: `std::sync::Arc`

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 284: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`
- Line 284: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 occurrences

- Line 5: unused import: `crate::query::context::validate::types::Variable`
- Line 746: unused import: `super::plan_node_operations::*`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 319: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 319: variable does not need to be mutable

#### `src\query\planner\match_planning\clauses\unwind_planner.rs`: 2 occurrences

- Line 1: unused import: `std::sync::Arc`
- Line 53: unused import: `crate::query::planner::plan::factory::PlanNodeFactory`

#### `src\core\mod.rs`: 1 occurrences

- Line 45: ambiguous glob re-exports: the name `SymbolType` in the type namespace is first re-exported here

#### `src\query\parser\cypher\expression_evaluator.rs`: 1 occurrences

- Line 6: unused imports: `BinaryOperator` and `UnaryOperator`

#### `src\query\planner\plan\core\nodes\sort_node.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\admin\index_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\dml\data_constructors.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\dml\insert_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\expression\visitor.rs`: 1 occurrences

- Line 286: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\expression\cache\mod.rs`: 1 occurrences

- Line 12: unused import: `std::time::Duration`

#### `src\core\query_pipeline_manager.rs`: 1 occurrences

- Line 117: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\query\planner\plan\management\dml\delete_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\admin\host_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\ngql\fetch_edges_planner.rs`: 1 occurrences

- Line 4: unused import: `std::sync::Arc`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\cache\traits.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 51: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\planner\plan\management\ddl\tag_ops.rs`: 1 occurrences

- Line 8: unused import: `std::sync::Arc`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 12: unused import: `std::sync::Arc`

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 1 occurrences

- Line 5: unused import: `crate::core::ValueTypeDef`

#### `src\query\planner\plan\core\nodes\mod.rs`: 1 occurrences

- Line 32: unused import: `plan_node_operations::*`

#### `src\query\planner\plan\management\admin\config_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\admin\system_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 10: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\dml\update_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 862: unused import: `crate::query::planner::plan::algorithms::IndexScan`

#### `src\query\planner\ngql\fetch_vertices_planner.rs`: 1 occurrences

- Line 5: unused import: `Column`

#### `src\query\planner\plan\management\ddl\space_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 6: unused import: `ZeroInputNode`

#### `src\query\visitor\mod.rs`: 1 occurrences

- Line 5: unused import: `crate::core::visitor::VisitorConfig`

