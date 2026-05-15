# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 186
- **Total Warnings**: 222
- **Total Issues**: 408
- **Unique Error Patterns**: 8
- **Unique Warning Patterns**: 66
- **Files with Issues**: 53

## Error Statistics

**Total Errors**: 186

### Error Type Breakdown

- **error[E0308]**: 183 errors
- **error[E0614]**: 3 errors

### Files with Errors (Top 10)

- `src\query\executor\graph_operations\graph_traversal\tests.rs`: 32 errors
- `src\storage\index\primary\degree_index.rs`: 29 errors
- `src\transaction\undo_log.rs`: 21 errors
- `src\storage\index\primary\edge_id_index.rs`: 18 errors
- `src\storage\index\primary\primary_index_manager.rs`: 18 errors
- `src\query\executor\graph_operations\graph_traversal\algorithms\multi_shortest_path.rs`: 17 errors
- `src\storage\edge\edge_table.rs`: 12 errors
- `src\storage\edge\mutable_csr_variant.rs`: 8 errors
- `src\query\executor\graph_operations\graph_traversal\algorithms\subgraph_executor.rs`: 6 errors
- `src\query\executor\expression\functions\builtin\path.rs`: 6 errors

## Warning Statistics

**Total Warnings**: 222

### Warning Type Breakdown

- **warning**: 222 warnings

### Files with Warnings (Top 10)

- `src\query\executor\graph_operations\graph_traversal\algorithms\multi_shortest_path.rs`: 25 warnings
- `src\query\executor\graph_operations\graph_traversal\all_paths.rs`: 19 warnings
- `src\query\executor\graph_operations\graph_traversal\algorithms\a_star.rs`: 17 warnings
- `src\query\executor\graph_operations\graph_traversal\algorithms\bfs_shortest.rs`: 17 warnings
- `src\query\executor\graph_operations\graph_traversal\algorithms\bidirectional_bfs.rs`: 16 warnings
- `src\query\executor\graph_operations\graph_traversal\algorithms\dijkstra.rs`: 13 warnings
- `src\query\executor\graph_operations\graph_traversal\algorithms\subgraph_executor.rs`: 11 warnings
- `src\transaction\undo_log.rs`: 11 warnings
- `src\storage\engine\sync_wrapper.rs`: 10 warnings
- `src\storage\engine\graph_storage\writer.rs`: 9 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `VertexId`, found integer

**Total Occurrences**: 183  
**Unique Files**: 20

#### `src\query\executor\graph_operations\graph_traversal\tests.rs`: 32 occurrences

- Line 31: mismatched types: expected `VertexId`, found `Value`
- Line 32: mismatched types: expected `VertexId`, found `Value`
- Line 33: mismatched types: expected `VertexId`, found `Value`
- ... 29 more occurrences in this file

#### `src\storage\index\primary\degree_index.rs`: 29 occurrences

- Line 276: arguments to this method are incorrect
- Line 277: arguments to this method are incorrect
- Line 278: arguments to this method are incorrect
- ... 26 more occurrences in this file

#### `src\transaction\undo_log.rs`: 21 occurrences

- Line 853: mismatched types: expected `VertexId`, found integer
- Line 858: mismatched types: expected `VertexId`, found integer
- Line 859: mismatched types: expected `VertexId`, found integer
- ... 18 more occurrences in this file

#### `src\storage\index\primary\edge_id_index.rs`: 18 occurrences

- Line 161: arguments to this method are incorrect
- Line 162: arguments to this method are incorrect
- Line 163: arguments to this method are incorrect
- ... 15 more occurrences in this file

#### `src\storage\index\primary\primary_index_manager.rs`: 18 occurrences

- Line 224: arguments to this method are incorrect
- Line 225: arguments to this method are incorrect
- Line 226: arguments to this method are incorrect
- ... 15 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\algorithms\multi_shortest_path.rs`: 17 occurrences

- Line 539: arguments to this function are incorrect
- Line 542: mismatched types: expected `&VertexId`, found `&Value`
- Line 543: mismatched types: expected `&VertexId`, found `&Value`
- ... 14 more occurrences in this file

#### `src\storage\edge\edge_table.rs`: 12 occurrences

- Line 1016: arguments to this method are incorrect
- Line 1017: arguments to this method are incorrect
- Line 1018: arguments to this method are incorrect
- ... 9 more occurrences in this file

#### `src\storage\edge\mutable_csr_variant.rs`: 8 occurrences

- Line 334: arguments to this method are incorrect
- Line 335: arguments to this method are incorrect
- Line 348: arguments to this method are incorrect
- ... 5 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\algorithms\subgraph_executor.rs`: 6 occurrences

- Line 446: mismatched types: expected `VertexId`, found `Value`
- Line 460: mismatched types: expected `VertexId`, found `Value`
- Line 460: mismatched types: expected `VertexId`, found `Value`
- ... 3 more occurrences in this file

#### `src\query\planning\statements\seeks\edge_seek.rs`: 4 occurrences

- Line 219: arguments to this function are incorrect
- Line 224: arguments to this function are incorrect
- Line 239: arguments to this function are incorrect
- ... 1 more occurrences in this file

#### `src\storage\iterator\edge_iter.rs`: 4 occurrences

- Line 144: arguments to this method are incorrect
- Line 145: arguments to this method are incorrect
- Line 146: arguments to this method are incorrect
- ... 1 more occurrences in this file

#### `src\query\executor\expression\functions\builtin\path.rs`: 3 occurrences

- Line 101: mismatched types: expected `VertexId`, found `Value`
- Line 111: arguments to this function are incorrect
- Line 118: arguments to this function are incorrect

#### `src\storage\engine\batch.rs`: 2 occurrences

- Line 414: arguments to this method are incorrect
- Line 445: arguments to this method are incorrect

#### `src\query\executor\utils\tag_filter.rs`: 2 occurrences

- Line 101: mismatched types: expected `VertexId`, found `Value`
- Line 121: mismatched types: expected `VertexId`, found `Value`

#### `src\query\executor\expression\functions\builtin\graph.rs`: 2 occurrences

- Line 273: mismatched types: expected `VertexId`, found `Value`
- Line 277: arguments to this function are incorrect

#### `src\storage\iterator\vertex_iter.rs`: 1 occurrences

- Line 192: mismatched types: expected `VertexId`, found integer

#### `src\query\executor\graph_operations\graph_traversal\all_paths.rs`: 1 occurrences

- Line 639: arguments to this function are incorrect

#### `src\storage\edge\cache_optimized_csr.rs`: 1 occurrences

- Line 1046: mismatched types: expected `u64`, found `i64`

#### `src\transaction\insert_transaction.rs`: 1 occurrences

- Line 420: mismatched types: expected `VertexId`, found integer

#### `src\query\optimizer\cost\calculator.rs`: 1 occurrences

- Line 953: mismatched types: expected `VertexId`, found `Value`

### error[E0614]: type `core::types::storage_ids::VertexId` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\executor\expression\functions\builtin\path.rs`: 3 occurrences

- Line 148: type `core::types::storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 153: type `core::types::storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 158: type `core::types::storage_ids::VertexId` cannot be dereferenced: can't be dereferenced

## Detailed Warning Categorization

### warning: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `vertex.vid`

**Total Occurrences**: 222  
**Unique Files**: 42

#### `src\query\executor\graph_operations\graph_traversal\algorithms\multi_shortest_path.rs`: 25 occurrences

- Line 123: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*src`
- Line 125: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*src`
- Line 126: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*src`
- ... 22 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\all_paths.rs`: 19 occurrences

- Line 88: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `v.vid`
- Line 250: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- Line 257: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`
- ... 16 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\algorithms\a_star.rs`: 17 occurrences

- Line 122: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*current_id`
- Line 128: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*end_id`
- Line 185: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- ... 14 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\algorithms\bfs_shortest.rs`: 17 occurrences

- Line 181: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- Line 183: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`
- Line 215: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `dst`
- ... 14 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\algorithms\bidirectional_bfs.rs`: 16 occurrences

- Line 71: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- Line 78: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`
- Line 85: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`
- ... 13 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\algorithms\dijkstra.rs`: 13 occurrences

- Line 96: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- Line 103: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`
- Line 110: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`
- ... 10 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\algorithms\subgraph_executor.rs`: 11 occurrences

- Line 113: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`
- Line 210: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`
- Line 213: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- ... 8 more occurrences in this file

#### `src\transaction\undo_log.rs`: 11 occurrences

- Line 156: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `self.vid`
- Line 180: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `self.src_vid`
- Line 182: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `self.dst_vid`
- ... 8 more occurrences in this file

#### `src\storage\engine\sync_wrapper.rs`: 10 occurrences

- Line 192: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `vertex.vid`
- Line 220: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `vertex.vid`
- Line 234: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `vertex.vid`
- ... 7 more occurrences in this file

#### `src\storage\engine\graph_storage\writer.rs`: 9 occurrences

- Line 42: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `vertex.vid`
- Line 63: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `vertex.vid`
- Line 82: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `vertex.vid`
- ... 6 more occurrences in this file

#### `src\storage\index\secondary\index_updater.rs`: 8 occurrences

- Line 234: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- Line 235: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`
- Line 281: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- ... 5 more occurrences in this file

#### `src\core\npath.rs`: 5 occurrences

- Line 467: unused import: `crate::core::Value`
- Line 140: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `new_path.vertex.vid`
- Line 249: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- ... 2 more occurrences in this file

#### `src\storage\edge\edge_table.rs`: 5 occurrences

- Line 353: redundant field names in struct initialization: help: replace it with: `src_vid`
- Line 533: redundant field names in struct initialization: help: replace it with: `src_vid`
- Line 275: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `nbr.neighbor`
- ... 2 more occurrences in this file

#### `src\query\executor\result_processing\topn.rs`: 4 occurrences

- Line 782: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `vertex.vid`
- Line 805: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `vertex.vid`
- Line 860: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- ... 1 more occurrences in this file

#### `src\core\vertex_edge_path.rs`: 4 occurrences

- Line 216: this `impl` can be derived
- Line 645: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `step.edge.src`
- Line 646: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `step.edge.dst`
- ... 1 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\traverse.rs`: 4 occurrences

- Line 247: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*current_node`
- Line 299: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `vertex.vid`
- Line 334: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `path.src.vid`
- ... 1 more occurrences in this file

#### `src\storage\edge\mutable_csr.rs`: 4 occurrences

- Line 749: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `src_list[i]`
- Line 751: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `dst_list[i]`
- Line 768: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `src`
- ... 1 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\expand_all.rs`: 3 occurrences

- Line 207: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `neighbor_id`
- Line 226: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `neighbor_id`
- Line 238: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `neighbor_id`

#### `src\query\executor\data_modification\update.rs`: 3 occurrences

- Line 223: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `vertex_vid`
- Line 290: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge_src`
- Line 291: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge_dst`

#### `src\storage\engine\graph_storage\index_manager.rs`: 3 occurrences

- Line 94: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `vertex.vid`
- Line 118: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- Line 119: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`

#### `src\storage\edge\single_mutable_csr.rs`: 3 occurrences

- Line 350: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `dst_list[i]`
- Line 454: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `self.src`
- Line 488: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `src`

#### `src\sync\vector_sync.rs`: 2 occurrences

- Line 384: the borrowed expression implements the required traits: help: change this to: `vertex.vid`
- Line 442: the borrowed expression implements the required traits: help: change this to: `vertex.vid`

#### `src\query\executor\admin\analyze.rs`: 2 occurrences

- Line 159: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*edge.src()`
- Line 160: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*edge.dst()`

#### `src\query\executor\graph_operations\graph_traversal\algorithms\types.rs`: 2 occurrences

- Line 47: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*dst`
- Line 48: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*src`

#### `src\storage\edge\single_immutable_csr.rs`: 2 occurrences

- Line 109: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `dst_list[i]`
- Line 254: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `src`

#### `src\query\planning\statements\seeks\scan_seek.rs`: 2 occurrences

- Line 66: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*vertex.vid()`
- Line 90: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*vertex.vid()`

#### `src\storage\index\secondary\index_data_manager.rs`: 2 occurrences

- Line 612: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- Line 613: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`

#### `src\query\executor\expression\functions\builtin\graph.rs`: 2 occurrences

- Line 227: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `e.src`
- Line 245: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `e.dst`

#### `src\storage\test_mock.rs`: 1 occurrences

- Line 13: unused import: `NullType`

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 101: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `(*edge.dst())`

#### `src\storage\edge\csr.rs`: 1 occurrences

- Line 136: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `dst_list[i]`

#### `src\query\planning\statements\seeks\index_seek.rs`: 1 occurrences

- Line 39: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*vertex.vid()`

#### `src\api\core\batch.rs`: 1 occurrences

- Line 309: unused import: `crate::core::Value`

#### `src\storage\engine\batch.rs`: 1 occurrences

- Line 180: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `self.current_src`

#### `src\storage\engine\graph_storage\type_utils.rs`: 1 occurrences

- Line 38: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `record.vid`

#### `src\query\planning\statements\paths\shortest_path_planner.rs`: 1 occurrences

- Line 215: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*vertex.vid()`

#### `src\query\planning\statements\seeks\variable_prop_index_seek.rs`: 1 occurrences

- Line 304: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*vertex.vid()`

#### `src\storage\index\primary\degree_index.rs`: 1 occurrences

- Line 244: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*entry.key()`

#### `src\query\executor\data_access\path.rs`: 1 occurrences

- Line 70: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`

#### `src\query\planning\statements\seeks\prop_index_seek.rs`: 1 occurrences

- Line 306: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*vertex.vid()`

#### `src\query\planning\statements\seeks\vertex_seek.rs`: 1 occurrences

- Line 41: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*vertex.vid()`

#### `src\storage\edge\cache_optimized_csr.rs`: 1 occurrences

- Line 825: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `dst`

