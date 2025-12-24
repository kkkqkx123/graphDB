# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 110
- **Total Issues**: 110
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 74
- **Files with Issues**: 54

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 110

### Warning Type Breakdown

- **warning**: 110 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 12 warnings
- `src\core\context\manager.rs`: 11 warnings
- `src\services\function.rs`: 10 warnings
- `src\core\evaluator\expression_evaluator.rs`: 9 warnings
- `src\query\visitor\extract_filter_expr_visitor.rs`: 3 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\core\context\runtime.rs`: 3 warnings
- `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 warnings
- `src\query\executor\result_processing\aggregation.rs`: 2 warnings
- `src\query\planner\match_planning\clauses\unwind_planner.rs`: 2 warnings

## Detailed Warning Categorization

### warning: unused import: `Column`

**Total Occurrences**: 110  
**Unique Files**: 54

#### `src\query\planner\plan\core\nodes\factory.rs`: 12 occurrences

- Line 35: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 31: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- Line 49: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- ... 9 more occurrences in this file

#### `src\core\context\manager.rs`: 11 occurrences

- Line 465: type `MockStorageEngine` is more private than the item `DefaultContextManager::create_runtime_context`: method `DefaultContextManager::create_runtime_context` is reachable at visibility `pub`
- Line 465: type `MockSchemaManager` is more private than the item `DefaultContextManager::create_runtime_context`: method `DefaultContextManager::create_runtime_context` is reachable at visibility `pub`
- Line 465: type `MockIndexManager` is more private than the item `DefaultContextManager::create_runtime_context`: method `DefaultContextManager::create_runtime_context` is reachable at visibility `pub`
- ... 8 more occurrences in this file

#### `src\services\function.rs`: 10 occurrences

- Line 118: unused `std::result::Result` that must be used
- Line 138: unused `std::result::Result` that must be used
- Line 158: unused `std::result::Result` that must be used
- ... 7 more occurrences in this file

#### `src\core\evaluator\expression_evaluator.rs`: 9 occurrences

- Line 243: variable does not need to be mutable
- Line 311: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 311: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- ... 6 more occurrences in this file

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 127: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\core\context\runtime.rs`: 3 occurrences

- Line 408: type `MockStorageEngine` is more private than the item `TestRuntimeContext`: type alias `TestRuntimeContext` is reachable at visibility `pub`
- Line 408: type `MockSchemaManager` is more private than the item `TestRuntimeContext`: type alias `TestRuntimeContext` is reachable at visibility `pub`
- Line 408: type `MockIndexManager` is more private than the item `TestRuntimeContext`: type alias `TestRuntimeContext` is reachable at visibility `pub`

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 3 occurrences

- Line 4: unused import: `VisitorResult`
- Line 378: unused variable: `func`: help: if this is intentional, prefix it with an underscore: `_func`
- Line 514: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 52: fields `storage` and `execution_context` are never read
- Line 70: methods `execute_executor` and `get_executable_executors` are never used

#### `src\query\planner\plan\management\ddl\edge_ops.rs`: 2 occurrences

- Line 5: unused import: `crate::query::context::validate::types::Variable`
- Line 8: unused import: `std::sync::Arc`

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 2 occurrences

- Line 5: unused import: `crate::core::ValueTypeDef`
- Line 19: field `match_clause_ctx` is never read

#### `src\core\context\mod.rs`: 2 occurrences

- Line 20: ambiguous glob re-exports: the name `SessionVariable` in the type namespace is first re-exported here
- Line 22: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\core\evaluator\traits.rs`: 2 occurrences

- Line 30: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 30: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\query\planner\ngql\go_planner.rs`: 2 occurrences

- Line 11: unused import: `std::sync::Arc`
- Line 59: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 318: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 318: variable does not need to be mutable

#### `src\query\planner\match_planning\clauses\unwind_planner.rs`: 2 occurrences

- Line 1: unused import: `std::sync::Arc`
- Line 53: unused import: `crate::query::planner::plan::factory::PlanNodeFactory`

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 283: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`
- Line 283: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\optimizer\optimizer.rs`: 2 occurrences

- Line 182: struct `DummyPlanNode` is never constructed
- Line 191: methods `id`, `type_name`, `dependencies`, `output_var`, `col_names`, and `cost` are never used

#### `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 occurrences

- Line 5: unused import: `crate::query::context::validate::types::Variable`
- Line 434: unused import: `super::plan_node_operations::*`

#### `src\core\query_pipeline_manager.rs`: 2 occurrences

- Line 117: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 20: fields `storage`, `parser`, `planner`, and `optimizer` are never read

#### `src\query\planner\ngql\fetch_vertices_planner.rs`: 1 occurrences

- Line 5: unused import: `Column`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\core\visitor.rs`: 1 occurrences

- Line 382: unused variable: `target_type`: help: try ignoring the field: `target_type: _`

#### `src\query\planner\plan\core\nodes\sort_node.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\admin\config_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 10: unused import: `std::sync::Arc`

#### `src\core\mod.rs`: 1 occurrences

- Line 46: ambiguous glob re-exports: the name `SymbolType` in the type namespace is first re-exported here

#### `src\query\visitor\mod.rs`: 1 occurrences

- Line 5: unused import: `crate::core::visitor::VisitorConfig`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::core::expressions::ExpressionContext`

#### `src\query\planner\plan\management\admin\index_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\core\nodes\mod.rs`: 1 occurrences

- Line 32: unused import: `plan_node_operations::*`

#### `src\query\planner\plan\management\dml\data_constructors.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\ddl\space_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\dml\insert_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 51: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\planner\plan\management\admin\host_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\expression\cypher\expression_converter.rs`: 1 occurrences

- Line 7: unused imports: `BinaryOperator` and `UnaryOperator`

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 400: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 6: unused import: `ZeroInputNode`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 12: unused import: `std::sync::Arc`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `VisitorResult`

#### `src\core\expressions\basic_context.rs`: 1 occurrences

- Line 338: method `args_to_hash` is never used

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 380: field `original_index` is never read

#### `src\query\planner\match_planning\match_planner.rs`: 1 occurrences

- Line 35: field `query_context` is never read

#### `src\query\planner\plan\management\dml\delete_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\ngql\fetch_edges_planner.rs`: 1 occurrences

- Line 4: unused import: `std::sync::Arc`

#### `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 1 occurrences

- Line 103: unused variable: `path`: help: if this is intentional, prefix it with an underscore: `_path`

#### `src\query\visitor\fold_constant_expr_visitor.rs`: 1 occurrences

- Line 11: field `parameters` is never read

#### `src\query\planner\plan\core\nodes\start_node.rs`: 1 occurrences

- Line 18: field `dependencies_vec` is never read

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 862: unused import: `crate::query::planner::plan::algorithms::IndexScan`

#### `src\cache\global_manager.rs`: 1 occurrences

- Line 137: creating a shared reference to mutable static: shared reference to mutable static

#### `src\cache\cache_impl\adaptive.rs`: 1 occurrences

- Line 11: variants `LFU` and `Hybrid` are never constructed

#### `src\query\planner\plan\management\admin\system_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\ddl\tag_ops.rs`: 1 occurrences

- Line 8: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\dml\update_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

