# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 100
- **Total Warnings**: 3
- **Total Issues**: 103
- **Unique Error Patterns**: 21
- **Unique Warning Patterns**: 2
- **Files with Issues**: 31

## Error Statistics

**Total Errors**: 100

### Error Type Breakdown

- **error[E0308]**: 90 errors
- **error[E0277]**: 4 errors
- **error[E0433]**: 2 errors
- **error[E0507]**: 2 errors
- **error[E0282]**: 2 errors

### Files with Errors (Top 10)

- `src\storage\engine\graph_storage\writer.rs`: 13 errors
- `src\query\executor\graph_operations\graph_traversal\expand_all.rs`: 10 errors
- `src\transaction\rollback.rs`: 9 errors
- `src\storage\index\secondary\index_updater.rs`: 6 errors
- `src\transaction\update_transaction.rs`: 6 errors
- `src\storage\engine\sync_wrapper.rs`: 6 errors
- `src\query\executor\graph_operations\graph_traversal\algorithms\subgraph_executor.rs`: 5 errors
- `src\storage\engine\property_graph\transaction_targets\recovery.rs`: 5 errors
- `src\query\executor\graph_operations\graph_traversal\traversal_utils.rs`: 4 errors
- `src\query\executor\graph_operations\graph_traversal\shortest_path.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\query\executor\factory\builders\traversal_builder.rs`: 1 warnings
- `src\query\executor\graph_operations\graph_traversal\algorithms\bidirectional_bfs.rs`: 1 warnings
- `src\query\executor\graph_operations\graph_traversal\algorithms\dijkstra.rs`: 1 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `u64`, found `VertexId`

**Total Occurrences**: 90  
**Unique Files**: 24

#### `src\storage\engine\graph_storage\writer.rs`: 13 occurrences

- Line 46: mismatched types: expected `&Value`, found `&VertexId`
- Line 85: mismatched types: expected `&Value`, found `&VertexId`
- Line 113: mismatched types: expected `&Value`, found `&VertexId`
- ... 10 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\expand_all.rs`: 10 occurrences

- Line 190: mismatched types: expected `&Value`, found `&VertexId`
- Line 207: mismatched types: expected `VertexId`, found `Value`
- Line 217: mismatched types: expected `&VertexId`, found `&Value`
- ... 7 more occurrences in this file

#### `src\transaction\rollback.rs`: 9 occurrences

- Line 234: mismatched types: expected `VertexId`, found `u64`
- Line 253: mismatched types: expected `VertexId`, found `u64`
- Line 254: mismatched types: expected `VertexId`, found `u64`
- ... 6 more occurrences in this file

#### `src\transaction\update_transaction.rs`: 6 occurrences

- Line 537: mismatched types: expected `u64`, found `VertexId`
- Line 567: mismatched types: expected `u64`, found `VertexId`
- Line 569: mismatched types: expected `u64`, found `VertexId`
- ... 3 more occurrences in this file

#### `src\storage\engine\sync_wrapper.rs`: 6 occurrences

- Line 197: mismatched types: expected `&Value`, found `&VertexId`
- Line 238: mismatched types: expected `&Value`, found `&VertexId`
- Line 277: mismatched types: expected `&Value`, found `&VertexId`
- ... 3 more occurrences in this file

#### `src\storage\index\secondary\index_updater.rs`: 6 occurrences

- Line 234: arguments to this method are incorrect
- Line 279: arguments to this method are incorrect
- Line 659: mismatched types: expected `Value`, found `VertexId`
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

#### `src\query\executor\graph_operations\graph_traversal\algorithms\subgraph_executor.rs`: 3 occurrences

- Line 220: mismatched types: expected `Vec<(VertexId, Edge)>`, found `Vec<(Value, Edge)>`
- Line 234: mismatched types: expected `&VertexId`, found `&Value`
- Line 244: mismatched types: expected `&VertexId`, found `&Value`

#### `src\transaction\insert_transaction.rs`: 3 occurrences

- Line 194: mismatched types: expected `VertexId`, found integer
- Line 358: mismatched types: expected `VertexId`, found integer
- Line 362: mismatched types: expected `VertexId`, found integer

#### `src\query\executor\graph_operations\graph_traversal\expand.rs`: 3 occurrences

- Line 218: mismatched types: expected `&VertexId`, found `&Value`
- Line 270: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`
- Line 272: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`

#### `src\query\executor\graph_operations\graph_traversal\traverse.rs`: 3 occurrences

- Line 245: mismatched types: expected `&Value`, found `&VertexId`
- Line 251: mismatched types: expected `&VertexId`, found `&Value`
- Line 295: mismatched types: expected `Value`, found `VertexId`

#### `src\query\planning\statements\dql\path_planner.rs`: 2 occurrences

- Line 132: mismatched types: expected `Vec<VertexId>`, found `Vec<Value>`
- Line 133: mismatched types: expected `Vec<VertexId>`, found `Vec<Value>`

#### `src\query\planning\statements\paths\shortest_path_planner.rs`: 2 occurrences

- Line 214: mismatched types: expected `Value`, found `VertexId`
- Line 268: mismatched types: expected `&VertexId`, found `&Value`

#### `src\query\planning\statements\seeks\vertex_seek.rs`: 2 occurrences

- Line 35: mismatched types: expected `&VertexId`, found `&Value`
- Line 43: mismatched types: expected `&VertexId`, found `&Value`

#### `src\query\executor\factory\builders\traversal_builder.rs`: 2 occurrences

- Line 172: mismatched types: expected `Vec<VertexId>`, found `Vec<Value>`
- Line 222: mismatched types: expected `Vec<VertexId>`, found `Vec<Value>`

#### `src\storage\engine\graph_storage\index_manager.rs`: 2 occurrences

- Line 94: mismatched types: expected `&Value`, found `&VertexId`
- Line 117: arguments to this method are incorrect

#### `src\query\planning\statements\seeks\scan_seek.rs`: 2 occurrences

- Line 72: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`
- Line 95: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`

#### `src\storage\engine\property_graph\core_ops.rs`: 2 occurrences

- Line 50: mismatched types: expected `VertexId`, found `u64`
- Line 83: mismatched types: expected `VertexId`, found `u64`

#### `src\query\planning\statements\seeks\index_seek.rs`: 1 occurrences

- Line 49: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 184: mismatched types: expected `&VertexId`, found `&Value`

#### `src\query\planning\statements\seeks\prop_index_seek.rs`: 1 occurrences

- Line 312: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`

#### `src\query\planning\statements\seeks\variable_prop_index_seek.rs`: 1 occurrences

- Line 310: mismatched types: expected `Vec<Value>`, found `Vec<VertexId>`

#### `src\storage\index\secondary\index_data_manager.rs`: 1 occurrences

- Line 614: arguments to this method are incorrect

### error[E0277]: the trait bound `std::string::String: From<&storage_ids::VertexId>` is not satisfied: the trait `From<&storage_ids::VertexId>` is not implemented for `std::string::String`

**Total Occurrences**: 4  
**Unique Files**: 3

#### `src\query\executor\graph_operations\graph_traversal\algorithms\subgraph_executor.rs`: 2 occurrences

- Line 218: a value of type `Vec<(value_def::Value, Edge)>` cannot be built from an iterator over elements of type `(storage_ids::VertexId, Edge)`: value of type `Vec<(value_def::Value, Edge)>` cannot be built from `std::iter::Iterator<Item=(storage_ids::VertexId, Edge)>`
- Line 230: a value of type `Vec<value_def::Value>` cannot be built from an iterator over elements of type `storage_ids::VertexId`: value of type `Vec<value_def::Value>` cannot be built from `std::iter::Iterator<Item=storage_ids::VertexId>`

#### `src\storage\engine\graph_storage\type_utils.rs`: 1 occurrences

- Line 38: the trait bound `std::string::String: From<&storage_ids::VertexId>` is not satisfied: the trait `From<&storage_ids::VertexId>` is not implemented for `std::string::String`

#### `src\storage\edge\csr.rs`: 1 occurrences

- Line 125: the trait bound `ImmutableNbr: std::marker::Copy` is not satisfied: the trait `std::marker::Copy` is not implemented for `ImmutableNbr`

### error[E0282]: type annotations needed

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\graph_operations\graph_traversal\traversal_utils.rs`: 2 occurrences

- Line 44: type annotations needed
- Line 127: type annotations needed

### error[E0507]: cannot move out of index of `Vec<Nbr>`: move occurs because value has type `Nbr`, which does not implement the `Copy` trait

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\storage\edge\mutable_csr.rs`: 1 occurrences

- Line 992: cannot move out of index of `Vec<Nbr>`: move occurs because value has type `Nbr`, which does not implement the `Copy` trait

#### `src\storage\index\primary\edge_id_index.rs`: 1 occurrences

- Line 70: cannot move out of dereference of `dashmap::mapref::one::Ref<'_, u64, edge_id_index::EdgeLocation>`: move occurs because value has type `edge_id_index::EdgeLocation`, which does not implement the `Copy` trait

### error[E0433]: failed to resolve: use of undeclared type `VertexId`: use of undeclared type `VertexId`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\graph_operations\graph_traversal\traversal_utils.rs`: 2 occurrences

- Line 43: failed to resolve: use of undeclared type `VertexId`: use of undeclared type `VertexId`
- Line 126: failed to resolve: use of undeclared type `VertexId`: use of undeclared type `VertexId`

## Detailed Warning Categorization

### warning: unused import: `Value`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\executor\graph_operations\graph_traversal\algorithms\bidirectional_bfs.rs`: 1 occurrences

- Line 8: unused import: `Value`

#### `src\query\executor\factory\builders\traversal_builder.rs`: 1 occurrences

- Line 189: unused import: `crate::core::Value`

#### `src\query\executor\graph_operations\graph_traversal\algorithms\dijkstra.rs`: 1 occurrences

- Line 9: unused import: `Value`

