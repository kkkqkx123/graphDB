# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 185
- **Total Warnings**: 3
- **Total Issues**: 188
- **Unique Error Patterns**: 42
- **Unique Warning Patterns**: 2
- **Files with Issues**: 43

## Error Statistics

**Total Errors**: 185

### Error Type Breakdown

- **error[E0308]**: 83 errors
- **error[E0614]**: 41 errors
- **error[E0382]**: 28 errors
- **error[E0507]**: 23 errors
- **error[E0277]**: 4 errors
- **error[E0508]**: 2 errors
- **error[E0369]**: 2 errors
- **error[E0599]**: 2 errors

### Files with Errors (Top 10)

- `src\storage\edge\edge_table.rs`: 25 errors
- `src\query\executor\graph_operations\graph_traversal\traversal_utils.rs`: 22 errors
- `src\storage\engine\graph_storage\writer.rs`: 13 errors
- `src\query\executor\graph_operations\graph_traversal\expand_all.rs`: 10 errors
- `src\storage\edge\cache_optimized_csr.rs`: 9 errors
- `src\transaction\rollback.rs`: 9 errors
- `src\transaction\insert_transaction.rs`: 8 errors
- `src\storage\index\secondary\index_updater.rs`: 6 errors
- `src\storage\engine\sync_wrapper.rs`: 6 errors
- `src\transaction\update_transaction.rs`: 6 errors

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\query\executor\factory\builders\traversal_builder.rs`: 1 warnings
- `src\query\executor\graph_operations\graph_traversal\algorithms\bidirectional_bfs.rs`: 1 warnings
- `src\query\executor\graph_operations\graph_traversal\algorithms\dijkstra.rs`: 1 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `VertexId`, found integer

**Total Occurrences**: 83  
**Unique Files**: 25

#### `src\storage\engine\graph_storage\writer.rs`: 13 occurrences

- Line 46: mismatched types: expected `&Value`, found `&VertexId`
- Line 85: mismatched types: expected `&Value`, found `&VertexId`
- Line 113: mismatched types: expected `&Value`, found `&VertexId`
- ... 10 more occurrences in this file

#### `src\transaction\rollback.rs`: 9 occurrences

- Line 234: mismatched types: expected `VertexId`, found `u64`
- Line 253: mismatched types: expected `VertexId`, found `u64`
- Line 254: mismatched types: expected `VertexId`, found `u64`
- ... 6 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\expand_all.rs`: 8 occurrences

- Line 190: mismatched types: expected `&Value`, found `&VertexId`
- Line 207: mismatched types: expected `VertexId`, found `Value`
- Line 217: mismatched types: expected `&VertexId`, found `&Value`
- ... 5 more occurrences in this file

#### `src\storage\engine\sync_wrapper.rs`: 6 occurrences

- Line 197: mismatched types: expected `&Value`, found `&VertexId`
- Line 238: mismatched types: expected `&Value`, found `&VertexId`
- Line 277: mismatched types: expected `&Value`, found `&VertexId`
- ... 3 more occurrences in this file

#### `src\transaction\update_transaction.rs`: 6 occurrences

- Line 537: mismatched types: expected `u64`, found `VertexId`
- Line 567: mismatched types: expected `u64`, found `VertexId`
- Line 569: mismatched types: expected `u64`, found `VertexId`
- ... 3 more occurrences in this file

#### `src\storage\engine\property_graph\transaction_targets\recovery.rs`: 5 occurrences

- Line 76: mismatched types: expected `VertexId`, found `u64`
- Line 78: mismatched types: expected `VertexId`, found `u64`
- Line 172: mismatched types: expected `VertexId`, found `u64`
- ... 2 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\shortest_path.rs`: 4 occurrences

- Line 81: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`
- Line 149: arguments to this method are incorrect
- Line 164: arguments to this method are incorrect
- ... 1 more occurrences in this file

#### `src\transaction\insert_transaction.rs`: 3 occurrences

- Line 194: mismatched types: expected `VertexId`, found integer
- Line 358: mismatched types: expected `VertexId`, found integer
- Line 362: mismatched types: expected `VertexId`, found integer

#### `src\query\executor\graph_operations\graph_traversal\algorithms\subgraph_executor.rs`: 3 occurrences

- Line 220: mismatched types: expected `Vec<(VertexId, Edge)>`, found `Vec<(Value, Edge)>`
- Line 234: mismatched types: expected `&VertexId`, found `&Value`
- Line 244: mismatched types: expected `&VertexId`, found `&Value`

#### `src\query\executor\graph_operations\graph_traversal\traversal_utils.rs`: 2 occurrences

- Line 46: mismatched types: expected `&VertexId`, found `&Value`
- Line 129: mismatched types: expected `&VertexId`, found `&Value`

#### `src\storage\engine\property_graph\core_ops.rs`: 2 occurrences

- Line 50: mismatched types: expected `VertexId`, found `u64`
- Line 83: mismatched types: expected `VertexId`, found `u64`

#### `src\query\executor\factory\builders\traversal_builder.rs`: 2 occurrences

- Line 172: mismatched types: expected `Vec<VertexId>`, found `Vec<Value>`
- Line 222: mismatched types: expected `Vec<VertexId>`, found `Vec<Value>`

#### `src\query\planning\statements\paths\shortest_path_planner.rs`: 2 occurrences

- Line 214: mismatched types: expected `Value`, found `VertexId`
- Line 268: mismatched types: expected `&VertexId`, found `&Value`

#### `src\query\executor\graph_operations\graph_traversal\traverse.rs`: 2 occurrences

- Line 245: mismatched types: expected `&Value`, found `&VertexId`
- Line 251: mismatched types: expected `&VertexId`, found `&Value`

#### `src\query\planning\statements\seeks\vertex_seek.rs`: 2 occurrences

- Line 35: mismatched types: expected `&VertexId`, found `&Value`
- Line 43: mismatched types: expected `&VertexId`, found `&Value`

#### `src\storage\index\secondary\index_updater.rs`: 2 occurrences

- Line 234: arguments to this method are incorrect
- Line 279: arguments to this method are incorrect

#### `src\storage\engine\graph_storage\index_manager.rs`: 2 occurrences

- Line 94: mismatched types: expected `&Value`, found `&VertexId`
- Line 117: arguments to this method are incorrect

#### `src\query\planning\statements\seeks\scan_seek.rs`: 2 occurrences

- Line 72: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`
- Line 95: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`

#### `src\query\planning\statements\dql\path_planner.rs`: 2 occurrences

- Line 132: mismatched types: expected `Vec<VertexId>`, found `Vec<Value>`
- Line 133: mismatched types: expected `Vec<VertexId>`, found `Vec<Value>`

#### `src\query\planning\statements\seeks\prop_index_seek.rs`: 1 occurrences

- Line 312: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 184: mismatched types: expected `&VertexId`, found `&Value`

#### `src\storage\index\secondary\index_data_manager.rs`: 1 occurrences

- Line 614: arguments to this method are incorrect

#### `src\query\executor\graph_operations\graph_traversal\expand.rs`: 1 occurrences

- Line 218: mismatched types: expected `&VertexId`, found `&Value`

#### `src\query\planning\statements\seeks\variable_prop_index_seek.rs`: 1 occurrences

- Line 310: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`

#### `src\query\planning\statements\seeks\index_seek.rs`: 1 occurrences

- Line 49: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`

### error[E0614]: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 41  
**Unique Files**: 10

#### `src\query\executor\graph_operations\graph_traversal\traversal_utils.rs`: 20 occurrences

- Line 65: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 65: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 77: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- ... 17 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\all_paths.rs`: 4 occurrences

- Line 47: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 47: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 437: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- ... 1 more occurrences in this file

#### `src\storage\index\secondary\index_updater.rs`: 4 occurrences

- Line 659: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 660: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 710: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\topn.rs`: 4 occurrences

- Line 783: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 807: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 833: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- ... 1 more occurrences in this file

#### `src\query\planning\statements\seeks\edge_seek.rs`: 2 occurrences

- Line 78: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 85: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced

#### `src\storage\engine\graph_storage\transactional_writer.rs`: 2 occurrences

- Line 57: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 106: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced

#### `src\query\executor\graph_operations\graph_traversal\expand_all.rs`: 2 occurrences

- Line 537: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced
- Line 602: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced

#### `src\query\executor\graph_operations\graph_traversal\traverse.rs`: 1 occurrences

- Line 296: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced

#### `src\query\planning\statements\paths\shortest_path_planner.rs`: 1 occurrences

- Line 271: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced

#### `src\query\executor\graph_operations\graph_traversal\expand.rs`: 1 occurrences

- Line 261: type `storage_ids::VertexId` cannot be dereferenced: can't be dereferenced

### error[E0382]: use of moved value: `vertex.vid`: value used here after move

**Total Occurrences**: 28  
**Unique Files**: 4

#### `src\storage\edge\edge_table.rs`: 22 occurrences

- Line 156: use of moved value: `src`: value used here after move
- Line 156: use of moved value: `dst`: value used here after move
- Line 160: borrow of moved value: `src`: value borrowed here after move
- ... 19 more occurrences in this file

#### `src\storage\index\primary\primary_index_manager.rs`: 3 occurrences

- Line 63: use of moved value: `src`: value used here after move
- Line 63: use of moved value: `dst`: value used here after move
- Line 73: use of partially moved value: `location`: value used here after partial move

#### `src\transaction\insert_transaction.rs`: 2 occurrences

- Line 233: use of moved value: `param.src_vid`: value used here after move
- Line 237: use of moved value: `param.dst_vid`: value used here after move

#### `src\storage\engine\property_graph\transaction_targets\undo.rs`: 1 occurrences

- Line 139: use of moved value: `vertex.vid`: value used here after move

### error[E0507]: cannot move out of `*src` which is behind a shared reference: move occurs because `*src` has type `storage_ids::VertexId`, which does not implement the `Copy` trait

**Total Occurrences**: 23  
**Unique Files**: 9

#### `src\storage\edge\cache_optimized_csr.rs`: 8 occurrences

- Line 265: cannot move out of index of `Vec<storage_ids::VertexId>`: move occurs because value has type `storage_ids::VertexId`, which does not implement the `Copy` trait
- Line 345: cannot move out of index of `Vec<storage_ids::VertexId>`: move occurs because value has type `storage_ids::VertexId`, which does not implement the `Copy` trait
- Line 360: cannot move out of index of `Vec<storage_ids::VertexId>`: move occurs because value has type `storage_ids::VertexId`, which does not implement the `Copy` trait
- ... 5 more occurrences in this file

#### `src\storage\index\primary\degree_index.rs`: 3 occurrences

- Line 221: cannot move out of a shared reference: move occurs because value has type `storage_ids::VertexId`, which does not implement the `Copy` trait
- Line 229: cannot move out of a shared reference: move occurs because value has type `storage_ids::VertexId`, which does not implement the `Copy` trait
- Line 237: cannot move out of a shared reference: move occurs because value has type `storage_ids::VertexId`, which does not implement the `Copy` trait

#### `src\storage\edge\edge_table.rs`: 3 occurrences

- Line 440: cannot move out of `src`, a captured variable in an `FnMut` closure: `src` is moved here
- Line 475: cannot move out of `dst`, a captured variable in an `FnMut` closure: `dst` is moved here
- Line 953: cannot move out of `self.src_vid` which is behind a mutable reference: `self.src_vid` is moved here

#### `src\storage\engine\batch.rs`: 2 occurrences

- Line 310: cannot move out of `*src` which is behind a shared reference: move occurs because `*src` has type `storage_ids::VertexId`, which does not implement the `Copy` trait
- Line 310: cannot move out of `*dst` which is behind a shared reference: move occurs because `*dst` has type `storage_ids::VertexId`, which does not implement the `Copy` trait

#### `src\storage\iterator\edge_iter.rs`: 2 occurrences

- Line 37: cannot move out of index of `Vec<storage_ids::VertexId>`: move occurs because value has type `storage_ids::VertexId`, which does not implement the `Copy` trait
- Line 78: cannot move out of index of `Vec<storage_ids::VertexId>`: move occurs because value has type `storage_ids::VertexId`, which does not implement the `Copy` trait

#### `src\storage\edge\mutable_csr.rs`: 2 occurrences

- Line 758: cannot move out of a shared reference
- Line 992: cannot move out of index of `Vec<Nbr>`: move occurs because value has type `Nbr`, which does not implement the `Copy` trait

#### `src\storage\index\primary\edge_id_index.rs`: 1 occurrences

- Line 70: cannot move out of dereference of `dashmap::mapref::one::Ref<'_, u64, edge_id_index::EdgeLocation>`: move occurs because value has type `edge_id_index::EdgeLocation`, which does not implement the `Copy` trait

#### `src\transaction\insert_transaction.rs`: 1 occurrences

- Line 162: cannot move out of a shared reference

#### `src\storage\iterator\vertex_iter.rs`: 1 occurrences

- Line 88: cannot move out of index of `Vec<storage_ids::VertexId>`: move occurs because value has type `storage_ids::VertexId`, which does not implement the `Copy` trait

### error[E0277]: the trait bound `std::string::String: From<&storage_ids::VertexId>` is not satisfied: the trait `From<&storage_ids::VertexId>` is not implemented for `std::string::String`

**Total Occurrences**: 4  
**Unique Files**: 3

#### `src\query\executor\graph_operations\graph_traversal\algorithms\subgraph_executor.rs`: 2 occurrences

- Line 218: a value of type `Vec<(value_def::Value, Edge)>` cannot be built from an iterator over elements of type `(storage_ids::VertexId, Edge)`: value of type `Vec<(value_def::Value, Edge)>` cannot be built from `std::iter::Iterator<Item=(storage_ids::VertexId, Edge)>`
- Line 230: a value of type `Vec<value_def::Value>` cannot be built from an iterator over elements of type `storage_ids::VertexId`: value of type `Vec<value_def::Value>` cannot be built from `std::iter::Iterator<Item=storage_ids::VertexId>`

#### `src\storage\engine\graph_storage\type_utils.rs`: 1 occurrences

- Line 38: the trait bound `std::string::String: From<&storage_ids::VertexId>` is not satisfied: the trait `From<&storage_ids::VertexId>` is not implemented for `std::string::String`

#### `src\storage\edge\csr.rs`: 1 occurrences

- Line 125: the trait bound `ImmutableNbr: std::marker::Copy` is not satisfied: unsatisfied trait bound

### error[E0508]: cannot move out of type `[storage_ids::VertexId]`, a non-copy slice: cannot move out of here, move occurs because `dst_list[_]` has type `storage_ids::VertexId`, which does not implement the `Copy` trait

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\storage\edge\cache_optimized_csr.rs`: 1 occurrences

- Line 815: cannot move out of type `[storage_ids::VertexId]`, a non-copy slice: cannot move out of here, move occurs because `dst_list[_]` has type `storage_ids::VertexId`, which does not implement the `Copy` trait

#### `src\storage\edge\mutable_csr.rs`: 1 occurrences

- Line 683: cannot move out of type `[storage_ids::VertexId]`, a non-copy slice: cannot move out of here, move occurs because `dst_list[_]` has type `storage_ids::VertexId`, which does not implement the `Copy` trait

### error[E0369]: cannot add `u64` to `storage_ids::VertexId`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\transaction\insert_transaction.rs`: 2 occurrences

- Line 153: cannot add `u64` to `storage_ids::VertexId`
- Line 202: cannot add `u64` to `storage_ids::VertexId`

### error[E0599]: no method named `as_ref` found for struct `storage_ids::VertexId` in the current scope

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\planning\statements\paths\shortest_path_planner.rs`: 2 occurrences

- Line 272: no method named `as_ref` found for struct `storage_ids::VertexId` in the current scope
- Line 274: no method named `as_ref` found for struct `storage_ids::VertexId` in the current scope

## Detailed Warning Categorization

### warning: unused import: `Value`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\executor\graph_operations\graph_traversal\algorithms\dijkstra.rs`: 1 occurrences

- Line 9: unused import: `Value`

#### `src\query\executor\graph_operations\graph_traversal\algorithms\bidirectional_bfs.rs`: 1 occurrences

- Line 8: unused import: `Value`

#### `src\query\executor\factory\builders\traversal_builder.rs`: 1 occurrences

- Line 189: unused import: `crate::core::Value`

