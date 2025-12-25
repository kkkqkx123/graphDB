# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 91
- **Total Issues**: 91
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 75
- **Files with Issues**: 49

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 91

### Warning Type Breakdown

- **warning**: 91 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 12 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 7 warnings
- `src\services\stats.rs`: 5 warnings
- `src\query\executor\data_modification.rs`: 4 warnings
- `src\query\planner\plan\core\nodes\control_flow_node.rs`: 4 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\query\executor\data_access.rs`: 3 warnings
- `src\expression\evaluator\traits.rs`: 2 warnings
- `src\storage\native_storage.rs`: 2 warnings
- `src\query\executor\result_processing\aggregation.rs`: 2 warnings

## Detailed Warning Categorization

### warning: fields `storage` and `execution_context` are never read

**Total Occurrences**: 91  
**Unique Files**: 49

#### `src\query\planner\plan\core\nodes\factory.rs`: 12 occurrences

- Line 35: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 31: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- Line 49: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- ... 9 more occurrences in this file

#### `src\expression\evaluator\expression_evaluator.rs`: 7 occurrences

- Line 304: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 304: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- Line 889: unused variable: `distinct`: help: if this is intentional, prefix it with an underscore: `_distinct`
- ... 4 more occurrences in this file

#### `src\services\stats.rs`: 5 occurrences

- Line 15: field `created_at` is never read
- Line 53: field `created_at` is never read
- Line 88: field `created_at` is never read
- ... 2 more occurrences in this file

#### `src\query\executor\data_modification.rs`: 4 occurrences

- Line 122: field `condition` is never read
- Line 236: fields `condition` and `cascade` are never read
- Line 338: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read
- ... 1 more occurrences in this file

#### `src\query\planner\plan\core\nodes\control_flow_node.rs`: 4 occurrences

- Line 17: field `dependencies` is never read
- Line 107: field `dependencies` is never read
- Line 216: field `dependencies` is never read
- ... 1 more occurrences in this file

#### `src\query\executor\data_access.rs`: 3 occurrences

- Line 205: field `edge_type` is never read
- Line 270: fields `vertex_ids`, `edge_direction`, and `edge_types` are never read
- Line 347: fields `vertex_ids`, `edge_ids`, and `prop_names` are never read

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 127: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 52: fields `storage` and `execution_context` are never read
- Line 70: methods `execute_executor` and `get_executable_executors` are never used

#### `src\core\query_pipeline_manager.rs`: 2 occurrences

- Line 117: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 20: fields `storage`, `parser`, `planner`, and `optimizer` are never read

#### `src\common\thread.rs`: 2 occurrences

- Line 58: field `id` is never read
- Line 175: field `mutex` is never read

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 284: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`
- Line 284: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\visitor\fold_constant_expr_visitor.rs`: 2 occurrences

- Line 11: field `parameters` is never read
- Line 31: methods `evaluate_arithmetic`, `evaluate_logical`, `evaluate_relational`, `evaluate_unary`, `evaluate_function`, and `cast_value` are never used

#### `src\core\context\mod.rs`: 2 occurrences

- Line 20: ambiguous glob re-exports: the name `SessionVariable` in the type namespace is first re-exported here
- Line 22: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 319: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 30: field `props` is never read

#### `src\storage\native_storage.rs`: 2 occurrences

- Line 14: field `schema_tree` is never read
- Line 76: method `value_from_bytes` is never used

#### `src\core\context\manager.rs`: 2 occurrences

- Line 95: field `created_at` is never read
- Line 305: method `is_max_contexts_exceeded` is never used

#### `src\query\optimizer\optimizer.rs`: 2 occurrences

- Line 182: struct `DummyPlanNode` is never constructed
- Line 191: methods `id`, `type_name`, `dependencies`, `output_var`, `col_names`, and `cost` are never used

#### `src\expression\evaluator\traits.rs`: 2 occurrences

- Line 30: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 30: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 518: methods `visit_property`, `visit_set`, and `parse_type_def` are never used

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 381: field `original_index` is never read

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 334: method `args_to_hash` is never used

#### `src\query\planner\plan\core\nodes\start_node.rs`: 1 occurrences

- Line 18: field `dependencies_vec` is never read

#### `src\query\planner\match_planning\core\match_clause_planner.rs`: 1 occurrences

- Line 37: field `paths` is never read

#### `src\api\service\graph_service.rs`: 1 occurrences

- Line 11: field `config` is never read

#### `src\query\context\ast\base.rs`: 1 occurrences

- Line 8: field `query_text` is never read

#### `src\expression\visitor.rs`: 1 occurrences

- Line 287: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 661: function `extract_range_condition` is never used

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 48: field `data` is never read

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 1 occurrences

- Line 18: field `match_clause_ctx` is never read

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 198: method `create_null_right_row` is never used

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 32: field `exchange` is never read

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 51: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\cache\cache_impl\adaptive.rs`: 1 occurrences

- Line 12: variants `LFU` and `Hybrid` are never constructed

#### `src\core\mod.rs`: 1 occurrences

- Line 45: ambiguous glob re-exports: the name `SymbolType` in the type namespace is first re-exported here

#### `src\core\signal_handler.rs`: 1 occurrences

- Line 53: fields `signals` and `signal_info` are never read

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 245: field `planners` is never read

#### `src\query\planner\match_planning\match_planner.rs`: 1 occurrences

- Line 35: field `query_context` is never read

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 222: field `config` is never read

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 58: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\planner\match_planning\clauses\where_clause_planner.rs`: 1 occurrences

- Line 50: field `need_stable_filter` is never read

#### `src\storage\iterator\get_neighbors_iter.rs`: 1 occurrences

- Line 290: method `col_valid` is never used

#### `src\query\planner\match_planning\core\match_planner.rs`: 1 occurrences

- Line 14: field `tail_connected` is never read

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 84: method `execute_multi_way_cartesian_product` is never used

#### `src\cache\global_manager.rs`: 1 occurrences

- Line 137: creating a shared reference to mutable static: shared reference to mutable static

#### `src\core\result\result_core.rs`: 1 occurrences

- Line 206: method `update_iterator_and_value` is never used

#### `src\stats\graph_stats.rs`: 1 occurrences

- Line 40: field `enable_space_level_metrics` is never read

#### `src\query\planner\match_planning\clauses\return_clause_planner.rs`: 1 occurrences

- Line 215: function `get_yield_columns` is never used

