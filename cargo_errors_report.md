# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 23
- **Total Warnings**: 72
- **Total Issues**: 95
- **Unique Error Patterns**: 22
- **Unique Warning Patterns**: 44
- **Files with Issues**: 45

## Error Statistics

**Total Errors**: 23

### Error Type Breakdown

- **error[E0599]**: 17 errors
- **error[E0277]**: 6 errors

### Files with Errors (Top 10)

- `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 8 errors
- `src\query\planner\plan\algorithms\path_algorithms.rs`: 8 errors
- `src\query\planner\plan\algorithms\index_scan.rs`: 4 errors
- `src\query\optimizer\limit_pushdown.rs`: 2 errors
- `src\query\planner\ngql\fetch_vertices_planner.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 72

### Warning Type Breakdown

- **warning**: 72 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 12 warnings
- `src\core\evaluator\expression_evaluator.rs`: 8 warnings
- `src\query\visitor\extract_filter_expr_visitor.rs`: 3 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\query\planner\match_planning\clauses\unwind_planner.rs`: 2 warnings
- `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 warnings
- `src\query\executor\result_processing\aggregation.rs`: 2 warnings
- `src\query\planner\plan\management\ddl\edge_ops.rs`: 2 warnings
- `src\query\planner\ngql\go_planner.rs`: 2 warnings
- `src\core\evaluator\traits.rs`: 2 warnings

## Detailed Error Categorization

### error[E0599]: no method named `set_output_var` found for struct `IndexScan` in the current scope

**Total Occurrences**: 17  
**Unique Files**: 5

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 8 occurrences

- Line 516: no method named `dependencies` found for reference `&project_node::ProjectNode` in the current scope: method not found in `&ProjectNode`
- Line 526: no method named `dependencies` found for reference `&IndexScan` in the current scope: method not found in `&IndexScan`
- Line 527: no method named `dependencies` found for reference `&graph_scan_node::GetVerticesNode` in the current scope: method not found in `&GetVerticesNode`
- ... 5 more occurrences in this file

#### `src\query\planner\plan\algorithms\path_algorithms.rs`: 4 occurrences

- Line 99: no method named `visit_multi_shortest_path` found for mutable reference `&mut V` in the current scope: method not found in `&mut V`
- Line 194: no method named `visit_bfs_shortest` found for mutable reference `&mut V` in the current scope
- Line 292: no method named `visit_all_paths` found for mutable reference `&mut V` in the current scope
- ... 1 more occurrences in this file

#### `src\query\optimizer\limit_pushdown.rs`: 2 occurrences

- Line 130: no method named `set_output_var` found for struct `IndexScan` in the current scope
- Line 672: no method named `set_output_var` found for struct `IndexScan` in the current scope

#### `src\query\planner\plan\algorithms\index_scan.rs`: 2 occurrences

- Line 90: no method named `visit_index_scan` found for mutable reference `&mut V` in the current scope
- Line 155: no method named `visit_fulltext_index_scan` found for mutable reference `&mut V` in the current scope: method not found in `&mut V`

#### `src\query\planner\ngql\fetch_vertices_planner.rs`: 1 occurrences

- Line 68: no method named `add_dependency` found for struct `graph_scan_node::GetVerticesNode` in the current scope: method not found in `GetVerticesNode`

### error[E0277]: the trait bound `IndexScan: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `IndexScan`

**Total Occurrences**: 6  
**Unique Files**: 2

#### `src\query\planner\plan\algorithms\path_algorithms.rs`: 4 occurrences

- Line 103: the trait bound `MultiShortestPath: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `MultiShortestPath`
- Line 198: the trait bound `BFSShortest: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `BFSShortest`
- Line 296: the trait bound `AllPaths: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `AllPaths`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\algorithms\index_scan.rs`: 2 occurrences

- Line 94: the trait bound `IndexScan: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `IndexScan`
- Line 159: the trait bound `FulltextIndexScan: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `FulltextIndexScan`

## Detailed Warning Categorization

### warning: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`

**Total Occurrences**: 72  
**Unique Files**: 42

#### `src\query\planner\plan\core\nodes\factory.rs`: 12 occurrences

- Line 35: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 31: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- Line 49: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- ... 9 more occurrences in this file

#### `src\core\evaluator\expression_evaluator.rs`: 8 occurrences

- Line 243: variable does not need to be mutable
- Line 311: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 311: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- ... 5 more occurrences in this file

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 3 occurrences

- Line 4: unused import: `VisitorResult`
- Line 378: unused variable: `func`: help: if this is intentional, prefix it with an underscore: `_func`
- Line 514: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 127: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 318: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 318: variable does not need to be mutable

#### `src\core\context\mod.rs`: 2 occurrences

- Line 20: ambiguous glob re-exports: the name `SessionVariable` in the type namespace is first re-exported here
- Line 22: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\core\evaluator\traits.rs`: 2 occurrences

- Line 30: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 30: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 283: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`
- Line 283: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\planner\match_planning\clauses\unwind_planner.rs`: 2 occurrences

- Line 1: unused import: `std::sync::Arc`
- Line 53: unused import: `crate::query::planner::plan::factory::PlanNodeFactory`

#### `src\query\planner\ngql\go_planner.rs`: 2 occurrences

- Line 11: unused import: `std::sync::Arc`
- Line 59: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 occurrences

- Line 5: unused import: `crate::query::context::validate::types::Variable`
- Line 588: unused import: `super::plan_node_operations::*`

#### `src\query\planner\plan\management\ddl\edge_ops.rs`: 2 occurrences

- Line 5: unused import: `crate::query::context::validate::types::Variable`
- Line 7: unused import: `std::sync::Arc`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::core::expressions::ExpressionContext`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `VisitorResult`

#### `src\query\planner\plan\management\admin\host_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 1 occurrences

- Line 103: unused variable: `path`: help: if this is intentional, prefix it with an underscore: `_path`

#### `src\query\planner\plan\management\dml\delete_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\dml\insert_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 51: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\core\mod.rs`: 1 occurrences

- Line 46: ambiguous glob re-exports: the name `SymbolType` in the type namespace is first re-exported here

#### `src\query\planner\plan\core\nodes\sort_node.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\dml\update_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\ngql\fetch_vertices_planner.rs`: 1 occurrences

- Line 5: unused import: `Column`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\query\planner\plan\management\admin\system_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\plan\core\nodes\mod.rs`: 1 occurrences

- Line 28: unused import: `plan_node_operations::*`

#### `src\query\planner\plan\management\admin\config_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\dml\data_constructors.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\expression\cypher\expression_converter.rs`: 1 occurrences

- Line 7: unused imports: `BinaryOperator` and `UnaryOperator`

#### `src\core\query_pipeline_manager.rs`: 1 occurrences

- Line 117: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\query\planner\ngql\fetch_edges_planner.rs`: 1 occurrences

- Line 4: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\admin\index_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\security\role_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 10: unused import: `std::sync::Arc`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 862: unused import: `crate::query::planner::plan::algorithms::IndexScan`

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 1 occurrences

- Line 5: unused import: `crate::core::ValueTypeDef`

#### `src\query\planner\plan\management\ddl\space_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\core\visitor.rs`: 1 occurrences

- Line 382: unused variable: `target_type`: help: try ignoring the field: `target_type: _`

#### `src\query\visitor\mod.rs`: 1 occurrences

- Line 5: unused import: `crate::core::visitor::VisitorConfig`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 12: unused import: `std::sync::Arc`

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 400: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`

#### `src\query\planner\plan\management\ddl\tag_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

