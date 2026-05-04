# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 1
- **Total Warnings**: 597
- **Total Issues**: 598
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 101
- **Files with Issues**: 133

## Error Statistics

**Total Errors**: 1

### Error Type Breakdown

- **error[E0609]**: 1 errors

### Files with Errors (Top 10)

- `tests\common\test_scenario.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 597

### Warning Type Breakdown

- **warning**: 597 warnings

### Files with Warnings (Top 10)

- `src\transaction\undo_log.rs`: 56 warnings
- `src\query\executor\factory\builders\admin_builder.rs`: 41 warnings
- `src\query\executor\result_processing\topn.rs`: 25 warnings
- `src\query\executor\data_access\vector_search.rs`: 20 warnings
- `src\query\executor\result_processing\sort.rs`: 17 warnings
- `src\query\query_pipeline_manager.rs`: 15 warnings
- `src\query\executor\result_processing\agg_function_manager.rs`: 14 warnings
- `src\transaction\update_transaction.rs`: 13 warnings
- `src\query\executor\factory\builders\data_modification_builder.rs`: 12 warnings
- `src\query\executor\relational_algebra\aggregation.rs`: 10 warnings

## Detailed Error Categorization

### error[E0609]: no field `schema_manager` on type `&graphdb::storage::StorageInner`: unknown field

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\common\test_scenario.rs`: 1 occurrences

- Line 40: no field `schema_manager` on type `&graphdb::storage::StorageInner`: unknown field

## Detailed Warning Categorization

### warning: the `Err`-variant returned from this function is very large

**Total Occurrences**: 597  
**Unique Files**: 132

#### `src\transaction\undo_log.rs`: 56 occurrences

- Line 622: unused variable: `label`: help: if this is intentional, prefix it with an underscore: `_label`
- Line 626: unused variable: `src_label`: help: if this is intentional, prefix it with an underscore: `_src_label`
- Line 626: unused variable: `dst_label`: help: if this is intentional, prefix it with an underscore: `_dst_label`
- ... 53 more occurrences in this file

#### `src\query\executor\factory\builders\admin_builder.rs`: 41 occurrences

- Line 62: the `Err`-variant returned from this function is very large
- Line 80: the `Err`-variant returned from this function is very large
- Line 95: the `Err`-variant returned from this function is very large
- ... 38 more occurrences in this file

#### `src\query\executor\result_processing\topn.rs`: 25 occurrences

- Line 248: the `Err`-variant returned from this function is very large
- Line 291: the `Err`-variant returned from this function is very large
- Line 306: the `Err`-variant returned from this function is very large
- ... 22 more occurrences in this file

#### `src\query\executor\data_access\vector_search.rs`: 20 occurrences

- Line 56: the `Err`-variant returned from this function is very large
- Line 85: the `Err`-variant returned from this closure is very large
- Line 132: the `Err`-variant returned from this function is very large
- ... 17 more occurrences in this file

#### `src\query\executor\result_processing\sort.rs`: 17 occurrences

- Line 106: the `Err`-variant returned from this function is very large
- Line 131: the `Err`-variant returned from this function is very large
- Line 157: the `Err`-variant returned from this function is very large
- ... 14 more occurrences in this file

#### `src\query\query_pipeline_manager.rs`: 15 occurrences

- Line 193: the `Err`-variant returned from this function is very large
- Line 201: the `Err`-variant returned from this function is very large
- Line 281: the `Err`-variant returned from this function is very large
- ... 12 more occurrences in this file

#### `src\query\executor\result_processing\agg_function_manager.rs`: 14 occurrences

- Line 44: the `Err`-variant returned from this closure is very large
- Line 65: the `Err`-variant returned from this closure is very large
- Line 92: the `Err`-variant returned from this closure is very large
- ... 11 more occurrences in this file

#### `src\transaction\update_transaction.rs`: 13 occurrences

- Line 9: unused import: `std::sync::Arc`
- Line 11: unused import: `decode_from_slice`
- Line 17: unused imports: `DeleteEdgeTypeUndo`, `DeleteVertexTypeUndo`, `InsertEdgeUndo`, and `InsertVertexUndo`
- ... 10 more occurrences in this file

#### `src\query\executor\factory\builders\data_modification_builder.rs`: 12 occurrences

- Line 39: the `Err`-variant returned from this function is very large
- Line 126: the `Err`-variant returned from this function is very large
- Line 209: the `Err`-variant returned from this function is very large
- ... 9 more occurrences in this file

#### `src\query\executor\relational_algebra\aggregation.rs`: 10 occurrences

- Line 152: the `Err`-variant returned from this function is very large
- Line 165: the `Err`-variant returned from this function is very large
- Line 275: the `Err`-variant returned from this function is very large
- ... 7 more occurrences in this file

#### `src\storage\entity\edge_storage.rs`: 9 occurrences

- Line 13: unused import: `IndexMetadataManager`
- Line 108: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- Line 112: unused variable: `rank`: help: if this is intentional, prefix it with an underscore: `_rank`
- ... 6 more occurrences in this file

#### `src\query\executor\factory\builders\traversal_builder.rs`: 9 occurrences

- Line 46: the `Err`-variant returned from this function is very large
- Line 65: the `Err`-variant returned from this function is very large
- Line 121: the `Err`-variant returned from this function is very large
- ... 6 more occurrences in this file

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 9 occurrences

- Line 83: the `Err`-variant returned from this function is very large
- Line 133: the `Err`-variant returned from this function is very large
- Line 166: the `Err`-variant returned from this function is very large
- ... 6 more occurrences in this file

#### `src\storage\metadata\inmemory_schema_manager.rs`: 9 occurrences

- Line 232: unused variable: `space_info`: help: if this is intentional, prefix it with an underscore: `_space_info`
- Line 250: unused variable: `space_info`: help: if this is intentional, prefix it with an underscore: `_space_info`
- Line 273: unused variable: `space_info`: help: if this is intentional, prefix it with an underscore: `_space_info`
- ... 6 more occurrences in this file

#### `src\query\executor\factory\builders\control_flow_builder.rs`: 9 occurrences

- Line 45: the `Err`-variant returned from this function is very large
- Line 75: the `Err`-variant returned from this function is very large
- Line 91: the `Err`-variant returned from this closure is very large
- ... 6 more occurrences in this file

#### `src\query\executor\factory\builders\fulltext_search_builder.rs`: 8 occurrences

- Line 50: the `Err`-variant returned from this function is very large
- Line 81: the `Err`-variant returned from this function is very large
- Line 106: the `Err`-variant returned from this function is very large
- ... 5 more occurrences in this file

#### `src\query\executor\factory\builders\join_builder.rs`: 8 occurrences

- Line 55: the `Err`-variant returned from this function is very large
- Line 79: the `Err`-variant returned from this function is very large
- Line 102: the `Err`-variant returned from this function is very large
- ... 5 more occurrences in this file

#### `src\storage\entity\vertex_storage.rs`: 8 occurrences

- Line 12: unused import: `IndexMetadataManager`
- Line 64: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- Line 161: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- ... 5 more occurrences in this file

#### `src\query\executor\factory\builders\data_processing_builder.rs`: 8 occurrences

- Line 39: the `Err`-variant returned from this function is very large
- Line 52: the `Err`-variant returned from this function is very large
- Line 75: the `Err`-variant returned from this function is very large
- ... 5 more occurrences in this file

#### `src\query\executor\relational_algebra\join\base_join.rs`: 7 occurrences

- Line 105: the `Err`-variant returned from this function is very large
- Line 157: the `Err`-variant returned from this function is very large
- Line 180: the `Err`-variant returned from this function is very large
- ... 4 more occurrences in this file

#### `src\query\executor\factory\builders\data_access_builder.rs`: 7 occurrences

- Line 40: the `Err`-variant returned from this function is very large
- Line 70: the `Err`-variant returned from this function is very large
- Line 87: the `Err`-variant returned from this function is very large
- ... 4 more occurrences in this file

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 7 occurrences

- Line 24: the `Err`-variant returned from this function is very large
- Line 88: the `Err`-variant returned from this function is very large
- Line 114: the `Err`-variant returned from this function is very large
- ... 4 more occurrences in this file

#### `src\storage\edge\mutable_csr.rs`: 7 occurrences

- Line 9: unused import: `MAX_TIMESTAMP`
- Line 11: constant `DEFAULT_CAPACITY` is never used
- Line 12: constant `GROWTH_FACTOR` is never used
- ... 4 more occurrences in this file

#### `src\query\executor\factory\builders\transformation_builder.rs`: 7 occurrences

- Line 41: the `Err`-variant returned from this function is very large
- Line 75: the `Err`-variant returned from this function is very large
- Line 101: the `Err`-variant returned from this function is very large
- ... 4 more occurrences in this file

#### `src\query\executor\result_processing\dedup.rs`: 7 occurrences

- Line 87: the `Err`-variant returned from this function is very large
- Line 106: the `Err`-variant returned from this function is very large
- Line 120: the `Err`-variant returned from this function is very large
- ... 4 more occurrences in this file

#### `src\query\executor\factory\executor_factory.rs`: 6 occurrences

- Line 85: the `Err`-variant returned from this function is very large
- Line 97: the `Err`-variant returned from this function is very large
- Line 121: the `Err`-variant returned from this function is very large
- ... 3 more occurrences in this file

#### `src\storage\property_graph.rs`: 6 occurrences

- Line 7: unused import: `std::sync::RwLock`
- Line 9: unused import: `DataType`
- Line 11: unused import: `EdgeDirection`
- ... 3 more occurrences in this file

#### `src\query\executor\utils\recursion_detector.rs`: 6 occurrences

- Line 27: the `Err`-variant returned from this function is very large
- Line 93: the `Err`-variant returned from this function is very large
- Line 210: the `Err`-variant returned from this function is very large
- ... 3 more occurrences in this file

#### `src\query\executor\result_processing\sample.rs`: 6 occurrences

- Line 70: the `Err`-variant returned from this function is very large
- Line 86: the `Err`-variant returned from this function is very large
- Line 100: the `Err`-variant returned from this function is very large
- ... 3 more occurrences in this file

#### `src\transaction\version_manager.rs`: 6 occurrences

- Line 6: unused import: `std::collections::HashSet`
- Line 285: variable `expected` is assigned to, but never used
- Line 291: value assigned to `expected` is never read
- ... 3 more occurrences in this file

#### `src\query\metadata\provider.rs`: 6 occurrences

- Line 37: the `Err`-variant returned from this function is very large
- Line 44: the `Err`-variant returned from this function is very large
- Line 51: the `Err`-variant returned from this function is very large
- ... 3 more occurrences in this file

#### `src\storage\vertex\vertex_table.rs`: 6 occurrences

- Line 7: unused import: `std::sync::RwLock`
- Line 9: unused import: `INVALID_TIMESTAMP`
- Line 10: unused import: `DataType`
- ... 3 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\algorithms\multi_shortest_path.rs`: 5 occurrences

- Line 155: the `Err`-variant returned from this function is very large
- Line 233: the `Err`-variant returned from this function is very large
- Line 320: the `Err`-variant returned from this function is very large
- ... 2 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\all_paths.rs`: 5 occurrences

- Line 221: the `Err`-variant returned from this function is very large
- Line 281: the `Err`-variant returned from this function is very large
- Line 340: the `Err`-variant returned from this function is very large
- ... 2 more occurrences in this file

#### `src\storage\container\arena_allocator.rs`: 5 occurrences

- Line 61: method `remaining` is never used
- Line 377: the loop variable `i` is used to index `slice`
- Line 380: the loop variable `i` is used to index `slice`
- ... 2 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\algorithms\a_star.rs`: 5 occurrences

- Line 116: the `Err`-variant returned from this function is very large
- Line 146: the `Err`-variant returned from this function is very large
- Line 159: the `Err`-variant returned from this function is very large
- ... 2 more occurrences in this file

#### `src\query\executor\factory\builders\vector_search_builder.rs`: 5 occurrences

- Line 49: the `Err`-variant returned from this function is very large
- Line 74: the `Err`-variant returned from this function is very large
- Line 99: the `Err`-variant returned from this function is very large
- ... 2 more occurrences in this file

#### `src\query\executor\relational_algebra\selection\filter.rs`: 5 occurrences

- Line 204: the `Err`-variant returned from this function is very large
- Line 220: the `Err`-variant returned from this function is very large
- Line 235: the `Err`-variant returned from this function is very large
- ... 2 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\algorithms\subgraph_executor.rs`: 4 occurrences

- Line 201: the `Err`-variant returned from this function is very large
- Line 242: the `Err`-variant returned from this function is very large
- Line 292: the `Err`-variant returned from this function is very large
- ... 1 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\traverse.rs`: 4 occurrences

- Line 106: the `Err`-variant returned from this function is very large
- Line 210: the `Err`-variant returned from this function is very large
- Line 225: the `Err`-variant returned from this function is very large
- ... 1 more occurrences in this file

#### `src\query\executor\base\executor_base.rs`: 4 occurrences

- Line 22: the `Err`-variant returned from this function is very large
- Line 25: the `Err`-variant returned from this function is very large
- Line 28: the `Err`-variant returned from this function is very large
- ... 1 more occurrences in this file

#### `src\query\planning\statements\dql\composite_index_analyzer.rs`: 4 occurrences

- Line 178: large size difference between variants: the entire enum is at least 448 bytes
- Line 393: this `map_or` can be simplified
- Line 532: called `Iterator::last` on a `DoubleEndedIterator`; this will needlessly iterate the entire iterator
- ... 1 more occurrences in this file

#### `src\query\planning\plan\validation\schema_validation.rs`: 4 occurrences

- Line 332: unused variable: `errors`: help: if this is intentional, prefix it with an underscore: `_errors`
- Line 334: unused variable: `node_id`: help: if this is intentional, prefix it with an underscore: `_node_id`
- Line 341: unused variable: `project_node`: help: if this is intentional, prefix it with an underscore: `_project_node`
- ... 1 more occurrences in this file

#### `src\query\executor\data_modification\delete.rs`: 4 occurrences

- Line 142: the `Err`-variant returned from this function is very large
- Line 468: the `Err`-variant returned from this function is very large
- Line 565: the `Err`-variant returned from this function is very large
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 4 occurrences

- Line 82: the `Err`-variant returned from this function is very large
- Line 143: the `Err`-variant returned from this function is very large
- Line 175: the `Err`-variant returned from this function is very large
- ... 1 more occurrences in this file

#### `src\transaction\insert_transaction.rs`: 4 occurrences

- Line 9: unused import: `std::sync::Arc`
- Line 16: unused imports: `CreateEdgeTypeRedo` and `CreateVertexTypeRedo`
- Line 91: this function has too many arguments (8/7)
- ... 1 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\expand.rs`: 4 occurrences

- Line 97: the `Err`-variant returned from this function is very large
- Line 137: the `Err`-variant returned from this function is very large
- Line 158: the `Err`-variant returned from this function is very large
- ... 1 more occurrences in this file

#### `src\query\executor\data_modification\update.rs`: 4 occurrences

- Line 166: the `Err`-variant returned from this function is very large
- Line 308: the `Err`-variant returned from this function is very large
- Line 350: the `Err`-variant returned from this function is very large
- ... 1 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\expand_all.rs`: 4 occurrences

- Line 71: this function has too many arguments (9/7)
- Line 104: this function has too many arguments (9/7)
- Line 162: the `Err`-variant returned from this function is very large
- ... 1 more occurrences in this file

#### `src\query\executor\data_modification\remove.rs`: 4 occurrences

- Line 133: the `Err`-variant returned from this function is very large
- Line 168: the `Err`-variant returned from this function is very large
- Line 179: the `Err`-variant returned from this function is very large
- ... 1 more occurrences in this file

#### `src\query\executor\data_access\fulltext_search.rs`: 3 occurrences

- Line 78: this function has too many arguments (10/7)
- Line 109: the `Err`-variant returned from this function is very large
- Line 400: the `Err`-variant returned from this function is very large

#### `src\transaction\read_transaction.rs`: 3 occurrences

- Line 7: unused import: `std::sync::Arc`
- Line 11: unused import: `super::undo_log::UndoTarget`
- Line 176: unused variable: `ts`: help: if this is intentional, prefix it with an underscore: `_ts`

#### `src\query\executor\base\result_processor.rs`: 3 occurrences

- Line 48: the `Err`-variant returned from this function is very large
- Line 69: the `Err`-variant returned from this function is very large
- Line 158: the `Err`-variant returned from this function is very large

#### `src\query\executor\factory\builders\set_operation_builder.rs`: 3 occurrences

- Line 34: the `Err`-variant returned from this function is very large
- Line 58: the `Err`-variant returned from this function is very large
- Line 81: the `Err`-variant returned from this function is very large

#### `src\query\planning\statements\seeks\multi_label_index_selector.rs`: 3 occurrences

- Line 50: unused variable: `covered_labels`: help: try ignoring the field: `covered_labels: _`
- Line 229: unused variable: `predicates`: help: if this is intentional, prefix it with an underscore: `_predicates`
- Line 281: this `map_or` can be simplified

#### `src\query\executor\control_flow\loops.rs`: 3 occurrences

- Line 76: the `Err`-variant returned from this function is very large
- Line 87: the `Err`-variant returned from this function is very large
- Line 132: the `Err`-variant returned from this function is very large

#### `src\query\executor\result_processing\transformations\helpers.rs`: 3 occurrences

- Line 10: the `Err`-variant returned from this function is very large
- Line 22: the `Err`-variant returned from this function is very large
- Line 43: the `Err`-variant returned from this function is very large

#### `src\transaction\wal\parser.rs`: 3 occurrences

- Line 9: unused import: `WalOpType`
- Line 68: this `map_or` can be simplified
- Line 143: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

#### `src\query\executor\data_access\search.rs`: 3 occurrences

- Line 86: the `Err`-variant returned from this function is very large
- Line 98: the `Err`-variant returned from this function is very large
- Line 326: the `Err`-variant returned from this function is very large

#### `src\query\executor\explain\explain_executor.rs`: 3 occurrences

- Line 62: the `Err`-variant returned from this function is very large
- Line 83: the `Err`-variant returned from this function is very large
- Line 157: the `Err`-variant returned from this function is very large

#### `src\storage\edge\csr.rs`: 3 occurrences

- Line 89: the loop variable `i` is used to index `new_offsets`
- Line 156: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here
- Line 160: hiding a lifetime that's elided elsewhere is confusing: the lifetime is elided here, the same lifetime is hidden here

#### `src\query\executor\result_processing\limit.rs`: 3 occurrences

- Line 56: the `Err`-variant returned from this function is very large
- Line 74: the `Err`-variant returned from this function is very large
- Line 88: the `Err`-variant returned from this function is very large

#### `src\query\executor\graph_operations\graph_traversal\algorithms\dijkstra.rs`: 3 occurrences

- Line 71: the `Err`-variant returned from this function is very large
- Line 128: the `Err`-variant returned from this function is very large
- Line 141: the `Err`-variant returned from this function is very large

#### `src\query\executor\relational_algebra\set_operations\base.rs`: 3 occurrences

- Line 57: the `Err`-variant returned from this function is very large
- Line 75: the `Err`-variant returned from this function is very large
- Line 99: the `Err`-variant returned from this function is very large

#### `src\storage\edge\edge_table.rs`: 3 occurrences

- Line 79: unused variable: `edge_capacity`: help: if this is intentional, prefix it with an underscore: `_edge_capacity`
- Line 37: field `config` is never read
- Line 107: this `if` statement can be collapsed

#### `src\query\executor\graph_operations\graph_traversal\algorithms\traits.rs`: 3 occurrences

- Line 33: the `Err`-variant returned from this function is very large
- Line 62: the `Err`-variant returned from this function is very large
- Line 86: the `Err`-variant returned from this function is very large

#### `src\query\executor\explain\profile_executor.rs`: 2 occurrences

- Line 47: the `Err`-variant returned from this function is very large
- Line 68: the `Err`-variant returned from this function is very large

#### `src\query\planning\statements\seeks\seek_strategy_base.rs`: 2 occurrences

- Line 378: field `estimated_rows` is never read
- Line 264: use of `filter_map` with an identity function: help: try: `flatten()`

#### `src\query\executor\data_modification\index_ops.rs`: 2 occurrences

- Line 100: the `Err`-variant returned from this function is very large
- Line 213: the `Err`-variant returned from this function is very large

#### `src\query\executor\relational_algebra\join\inner_join.rs`: 2 occurrences

- Line 137: the `Err`-variant returned from this function is very large
- Line 301: the `Err`-variant returned from this function is very large

#### `src\query\executor\graph_operations\graph_traversal\traversal_utils.rs`: 2 occurrences

- Line 42: the `Err`-variant returned from this function is very large
- Line 125: the `Err`-variant returned from this function is very large

#### `src\query\executor\factory\engine.rs`: 2 occurrences

- Line 80: the `Err`-variant returned from this function is very large
- Line 179: the `Err`-variant returned from this function is very large

#### `src\query\executor\relational_algebra\projection.rs`: 2 occurrences

- Line 204: the `Err`-variant returned from this function is very large
- Line 288: the `Err`-variant returned from this function is very large

#### `src\query\executor\relational_algebra\join\hash_table.rs`: 2 occurrences

- Line 93: the `Err`-variant returned from this function is very large
- Line 130: the `Err`-variant returned from this function is very large

#### `src\query\executor\graph_operations\graph_traversal\algorithms\bidirectional_bfs.rs`: 2 occurrences

- Line 44: the `Err`-variant returned from this function is very large
- Line 100: the `Err`-variant returned from this function is very large

#### `src\transaction\manager.rs`: 2 occurrences

- Line 9: unused import: `std::time::Duration`
- Line 18: unused import: `super::wal::writer::WalWriter`

#### `src\query\executor\data_access\vertex.rs`: 2 occurrences

- Line 132: the `Err`-variant returned from this function is very large
- Line 354: the `Err`-variant returned from this function is very large

#### `src\query\metadata\schema_provider.rs`: 2 occurrences

- Line 7: unused imports: `EngineType` and `SpaceStatus`
- Line 28: the `Err`-variant returned from this function is very large

#### `src\query\executor\data_access\edge.rs`: 2 occurrences

- Line 90: the `Err`-variant returned from this function is very large
- Line 185: the `Err`-variant returned from this function is very large

#### `src\api\server\client\query_context.rs`: 2 occurrences

- Line 60: the `Err`-variant returned from this function is very large
- Line 90: the `Err`-variant returned from this closure is very large

#### `src\query\executor\relational_algebra\join\cross_join.rs`: 2 occurrences

- Line 91: the `Err`-variant returned from this function is very large
- Line 130: the `Err`-variant returned from this function is very large

#### `src\transaction\compact_transaction.rs`: 2 occurrences

- Line 7: unused import: `std::sync::Arc`
- Line 94: slow zero-filling initialization: help: consider replacing this with: `vec![0; WalHeader::SIZE]`

#### `src\query\executor\relational_algebra\join\left_join.rs`: 2 occurrences

- Line 81: the `Err`-variant returned from this function is very large
- Line 162: the `Err`-variant returned from this function is very large

#### `src\query\planning\plan\core\nodes\graph_operations\graph_operations_node.rs`: 2 occurrences

- Line 1249: creating a new box: help: replace existing content with inner value instead: `*self.left_input = input`
- Line 1256: creating a new box: help: replace existing content with inner value instead: `*self.right_input = input`

#### `src\query\executor\graph_operations\graph_traversal\algorithms\bfs_shortest.rs`: 2 occurrences

- Line 142: the `Err`-variant returned from this function is very large
- Line 243: the `Err`-variant returned from this function is very large

#### `src\storage\container\mmap_container.rs`: 2 occurrences

- Line 7: unused import: `Read`
- Line 318: unnecessary `unsafe` block: unnecessary `unsafe` block

#### `src\transaction\wal\writer.rs`: 2 occurrences

- Line 42: field `version` is never read
- Line 156: methods `rotate_if_needed` and `rotate` are never used

#### `src\query\executor\relational_algebra\set_operations\intersect.rs`: 1 occurrences

- Line 52: the `Err`-variant returned from this function is very large

#### `src\query\executor\result_processing\transformations\unwind.rs`: 1 occurrences

- Line 77: the `Err`-variant returned from this function is very large

#### `src\query\planning\plan\core\nodes\management\manage_node_enums.rs`: 1 occurrences

- Line 55: large size difference between variants: the entire enum is at least 312 bytes

#### `src\query\executor\result_processing\transformations\assign.rs`: 1 occurrences

- Line 60: the `Err`-variant returned from this function is very large

#### `src\query\executor\data_modification\insert.rs`: 1 occurrences

- Line 170: the `Err`-variant returned from this function is very large

#### `src\query\executor\data_access\index.rs`: 1 occurrences

- Line 97: the `Err`-variant returned from this function is very large

#### `src\query\context\query_context_builder.rs`: 1 occurrences

- Line 5: unused imports: `EngineType` and `SpaceStatus`

#### `src\transaction\manager_test.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\storage\edge\property_table.rs`: 1 occurrences

- Line 270: unused variable: `offset2`: help: if this is intentional, prefix it with an underscore: `_offset2`

#### `src\query\planning\statements\match_statement_planner.rs`: 1 occurrences

- Line 53: field `return_planner` is never read

#### `src\query\executor\relational_algebra\set_operations\minus.rs`: 1 occurrences

- Line 52: the `Err`-variant returned from this function is very large

#### `src\storage\test_mock.rs`: 1 occurrences

- Line 19: unused import: `PropertyGraphConfig`

#### `src\query\planning\plan\core\nodes\control_flow\control_flow_node.rs`: 1 occurrences

- Line 522: this `impl` can be derived

#### `src\query\executor\admin\analyze.rs`: 1 occurrences

- Line 183: the `Err`-variant returned from this function is very large

#### `src\api\embedded\database.rs`: 1 occurrences

- Line 105: unused variable: `db`: help: if this is intentional, prefix it with an underscore: `_db`

#### `src\query\executor\base\manage_executor_enums.rs`: 1 occurrences

- Line 52: large size difference between variants: the entire enum is at least 544 bytes

#### `src\query\executor\graph_operations\graph_traversal\factory.rs`: 1 occurrences

- Line 36: this function has too many arguments (9/7)

#### `src\storage\vertex\column_store.rs`: 1 occurrences

- Line 144: unused variable: `start`: help: if this is intentional, prefix it with an underscore: `_start`

#### `src\query\executor\data_access\neighbor.rs`: 1 occurrences

- Line 100: the `Err`-variant returned from this function is very large

#### `src\query\executor\relational_algebra\join\full_outer_join.rs`: 1 occurrences

- Line 66: the `Err`-variant returned from this function is very large

#### `src\storage\vertex\mod.rs`: 1 occurrences

- Line 22: unused imports: `AtomicU32` and `Ordering`

#### `src\query\executor\data_access\property.rs`: 1 occurrences

- Line 96: the `Err`-variant returned from this function is very large

#### `src\storage\graph_storage.rs`: 1 occurrences

- Line 16: unused import: `IndexMetadataManager`

#### `src\storage\entity\event_storage.rs`: 1 occurrences

- Line 7: unused import: `crate::storage::metadata::inmemory_schema_manager::InMemorySchemaManager`

#### `src\query\executor\data_modification\tag_ops.rs`: 1 occurrences

- Line 114: the `Err`-variant returned from this function is very large

#### `src\query\cache\manager.rs`: 1 occurrences

- Line 24: unused imports: `CteCacheStatsSnapshot` and `PlanCacheStatsSnapshot`

#### `src\query\planning\statements\dml\delete_planner.rs`: 1 occurrences

- Line 116: you are needlessly cloning iterator elements: help: remove the `map` call

#### `src\query\executor\data_access\match_fulltext.rs`: 1 occurrences

- Line 72: the `Err`-variant returned from this function is very large

#### `src\query\planning\statements\dql\pipe_variable_resolver.rs`: 1 occurrences

- Line 291: this `if let` can be collapsed into the outer `if let`

#### `src\api\server\client\client_session.rs`: 1 occurrences

- Line 139: the `Err`-variant returned from this function is very large

#### `src\query\executor\graph_operations\materialize.rs`: 1 occurrences

- Line 94: the `Err`-variant returned from this function is very large

#### `src\query\executor\relational_algebra\set_operations\union_all.rs`: 1 occurrences

- Line 51: the `Err`-variant returned from this function is very large

#### `src\query\planning\plan\validation\cycle_detection.rs`: 1 occurrences

- Line 188: this `map_or` can be simplified

#### `src\storage\edge\mod.rs`: 1 occurrences

- Line 22: unused imports: `AtomicU32`, `AtomicU64`, and `Ordering`

#### `src\transaction\context.rs`: 1 occurrences

- Line 7: unused import: `std::sync::Arc`

#### `src\api\mod.rs`: 1 occurrences

- Line 34: unused import: `crate::storage::api::StorageClient`

#### `src\query\planning\plan\core\nodes\base\plan_node_operations.rs`: 1 occurrences

- Line 151: unreachable pattern: no value can reach this

#### `src\query\executor\relational_algebra\set_operations\union.rs`: 1 occurrences

- Line 52: the `Err`-variant returned from this function is very large

#### `src\query\validator\statements\update_validator.rs`: 1 occurrences

- Line 230: the `Err`-variant returned from this function is very large

#### `src\query\planning\plan\core\nodes\traversal\traversal_node.rs`: 1 occurrences

- Line 919: this function has too many arguments (11/7)

#### `src\api\server\graph_service.rs`: 1 occurrences

- Line 336: casting to the same type is unnecessary (`u64` -> `u64`): help: try: `s.id`

#### `src\storage\vertex\vertex_timestamp.rs`: 1 occurrences

- Line 50: unused variable: `ts`: help: if this is intentional, prefix it with an underscore: `_ts`

#### `src\query\planning\statements\paths\variable_length_path_planner.rs`: 1 occurrences

- Line 108: this `map_or` can be simplified

#### `src\query\planning\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 191: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\executor\graph_operations\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 141: the `Err`-variant returned from this function is very large

