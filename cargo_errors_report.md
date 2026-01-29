# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 300
- **Total Warnings**: 141
- **Total Issues**: 441
- **Unique Error Patterns**: 109
- **Unique Warning Patterns**: 72
- **Files with Issues**: 85

## Error Statistics

**Total Errors**: 300

### Error Type Breakdown

- **error[E0599]**: 138 errors
- **error[E0412]**: 44 errors
- **error[E0061]**: 38 errors
- **error[E0050]**: 30 errors
- **error[E0407]**: 26 errors
- **error[E0053]**: 12 errors
- **error[E0277]**: 4 errors
- **error[E0034]**: 3 errors
- **error[E0369]**: 3 errors
- **error[E0046]**: 2 errors

### Files with Errors (Top 10)

- `src\query\visitor\deduce_type_visitor.rs`: 73 errors
- `src\query\context\managers\impl\storage_client_impl.rs`: 43 errors
- `src\storage\test_mock.rs`: 41 errors
- `src\query\executor\executor_enum.rs`: 34 errors
- `src\index\storage.rs`: 15 errors
- `src\query\executor\data_access.rs`: 15 errors
- `src\query\executor\data_modification.rs`: 15 errors
- `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 14 errors
- `src\query\executor\data_processing\graph_traversal\tests.rs`: 7 errors
- `src\query\executor\search_executors.rs`: 6 errors

## Warning Statistics

**Total Warnings**: 141

### Warning Type Breakdown

- **warning**: 141 warnings

### Files with Warnings (Top 10)

- `src\storage\storage_client.rs`: 26 warnings
- `src\query\executor\result_processing\projection.rs`: 8 warnings
- `src\query\optimizer\elimination_rules.rs`: 7 warnings
- `src\query\context\runtime_context.rs`: 5 warnings
- `src\storage\iterator\composite.rs`: 4 warnings
- `src\query\planner\statements\match_planner.rs`: 3 warnings
- `src\query\planner\statements\paths\shortest_path_planner.rs`: 3 warnings
- `src\query\validator\insert_vertices_validator.rs`: 3 warnings
- `src\storage\plan\executors\mod.rs`: 3 warnings
- `src\storage\iterator\predicate.rs`: 3 warnings

## Detailed Error Categorization

### error[E0599]: no method named `get_node` found for reference `&std::sync::Arc<dyn storage::storage_client::StorageClient>` in the current scope

**Total Occurrences**: 138  
**Unique Files**: 19

#### `src\query\context\managers\impl\storage_client_impl.rs`: 43 occurrences

- Line 60: no method named `scan_all_vertices` found for struct `std::sync::RwLockReadGuard<'_, memory_storage::MemoryStorage>` in the current scope: method not found in `RwLockReadGuard<'_, MemoryStorage>`
- Line 86: no method named `scan_all_vertices` found for struct `std::sync::RwLockReadGuard<'_, memory_storage::MemoryStorage>` in the current scope: method not found in `RwLockReadGuard<'_, MemoryStorage>`
- Line 115: no method named `scan_all_vertices` found for struct `std::sync::RwLockReadGuard<'_, memory_storage::MemoryStorage>` in the current scope: method not found in `RwLockReadGuard<'_, MemoryStorage>`
- ... 40 more occurrences in this file

#### `src\query\executor\executor_enum.rs`: 33 occurrences

- Line 197: the method `name` exists for reference `&InsertVertexExecutor<S>`, but its trait bounds were not satisfied: method cannot be called on `&InsertVertexExecutor<S>` due to unsatisfied trait bounds
- Line 198: the method `name` exists for reference `&InsertEdgeExecutor<S>`, but its trait bounds were not satisfied: method cannot be called on `&InsertEdgeExecutor<S>` due to unsatisfied trait bounds
- Line 199: the method `name` exists for reference `&UpdateExecutor<S>`, but its trait bounds were not satisfied: method cannot be called on `&UpdateExecutor<S>` due to unsatisfied trait bounds
- ... 30 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 12 occurrences

- Line 282: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- Line 295: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- Line 352: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- ... 9 more occurrences in this file

#### `src\index\storage.rs`: 12 occurrences

- Line 465: no method named `get_node` found for struct `MutexGuard<'_, dyn StorageClient + Send + Sync>` in the current scope
- Line 508: no method named `get_node` found for struct `MutexGuard<'_, dyn StorageClient + Send + Sync>` in the current scope
- Line 558: no method named `get_node` found for struct `MutexGuard<'_, dyn StorageClient + Send + Sync>` in the current scope
- ... 9 more occurrences in this file

#### `src\query\executor\data_access.rs`: 7 occurrences

- Line 53: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- Line 78: no method named `scan_all_vertices` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- Line 428: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- ... 4 more occurrences in this file

#### `src\query\executor\data_modification.rs`: 5 occurrences

- Line 62: no method named `insert_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- Line 221: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- Line 225: no method named `update_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope: method not found in `MutexGuard<'_, S>`
- ... 2 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 4 occurrences

- Line 138: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- Line 226: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- Line 231: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- ... 1 more occurrences in this file

#### `src\query\executor\search_executors.rs`: 4 occurrences

- Line 60: no method named `scan_all_vertices` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- Line 203: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- Line 230: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 4 occurrences

- Line 28: no method named `insert_node` found for struct `std::sync::MutexGuard<'_, test_mock::MockStorage>` in the current scope
- Line 31: no method named `insert_node` found for struct `std::sync::MutexGuard<'_, test_mock::MockStorage>` in the current scope
- Line 34: no method named `insert_node` found for struct `std::sync::MutexGuard<'_, test_mock::MockStorage>` in the current scope
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 4 occurrences

- Line 151: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- Line 281: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- Line 286: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope
- ... 1 more occurrences in this file

#### `src\query\planner\statements\seeks\vertex_seek.rs`: 2 occurrences

- Line 29: no method named `get_node` found for reference `&dyn storage::storage_client::StorageClient` in the current scope
- Line 37: no method named `get_node` found for reference `&dyn storage::storage_client::StorageClient` in the current scope

#### `src\query\context\managers\impl\index_manager_impl.rs`: 1 occurrences

- Line 1147: no method named `get_node` found for reference `&std::sync::Arc<dyn storage::storage_client::StorageClient>` in the current scope

#### `src\expression\storage\row_reader.rs`: 1 occurrences

- Line 392: no variant or associated item named `Int` found for enum `FieldType` in the current scope: variant or associated item not found in `FieldType`

#### `src\storage\iterator\composite.rs`: 1 occurrences

- Line 748: `composite::FilterIter<composite::CompositeIter<sequential_iter::SequentialIter>>` is not an iterator: `composite::FilterIter<composite::CompositeIter<sequential_iter::SequentialIter>>` is not an iterator

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 254: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope

#### `src\query\executor\data_processing\graph_traversal\expand.rs`: 1 occurrences

- Line 200: no method named `get_node` found for struct `std::sync::MutexGuard<'_, S>` in the current scope

#### `src\storage\transaction\lock.rs`: 1 occurrences

- Line 630: no method named `is_failure` found for enum `storage::transaction::lock::LockResult` in the current scope: method not found in `LockResult`

#### `src\storage\transaction\snapshot.rs`: 1 occurrences

- Line 432: no method named `uses_snapshot` found for struct `snapshot::Snapshot` in the current scope: method not found in `Snapshot`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 25: no method named `scan_all_vertices` found for reference `&dyn storage::storage_client::StorageClient` in the current scope

### error[E0412]: cannot find type `DBError` in this scope: not found in this scope

**Total Occurrences**: 44  
**Unique Files**: 1

#### `src\query\visitor\deduce_type_visitor.rs`: 44 occurrences

- Line 567: cannot find type `DBError` in this scope: not found in this scope
- Line 571: cannot find type `DBError` in this scope: not found in this scope
- Line 575: cannot find type `DBError` in this scope: not found in this scope
- ... 41 more occurrences in this file

### error[E0061]: this method takes 2 arguments but 1 argument was supplied

**Total Occurrences**: 38  
**Unique Files**: 13

#### `src\query\executor\data_modification.rs`: 10 occurrences

- Line 72: this method takes 2 arguments but 1 argument was supplied
- Line 265: this method takes 4 arguments but 3 arguments were supplied
- Line 270: this method takes 4 arguments but 3 arguments were supplied
- ... 7 more occurrences in this file

#### `src\query\executor\data_access.rs`: 8 occurrences

- Line 208: this method takes 2 arguments but 1 argument was supplied
- Line 210: this method takes 1 argument but 0 arguments were supplied
- Line 296: this method takes 2 arguments but 1 argument was supplied
- ... 5 more occurrences in this file

#### `src\query\context\managers\impl\index_manager_impl.rs`: 3 occurrences

- Line 899: this method takes 2 arguments but 1 argument was supplied
- Line 919: this method takes 2 arguments but 1 argument was supplied
- Line 1160: this method takes 4 arguments but 3 arguments were supplied

#### `src\index\storage.rs`: 3 occurrences

- Line 470: this method takes 4 arguments but 3 arguments were supplied
- Line 513: this method takes 4 arguments but 3 arguments were supplied
- Line 563: this method takes 4 arguments but 3 arguments were supplied

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 3 occurrences

- Line 64: this method takes 2 arguments but 1 argument was supplied
- Line 67: this method takes 2 arguments but 1 argument was supplied
- Line 70: this method takes 2 arguments but 1 argument was supplied

#### `src\query\executor\data_processing\graph_traversal\traversal_utils.rs`: 2 occurrences

- Line 19: this method takes 3 arguments but 2 arguments were supplied
- Line 73: this method takes 3 arguments but 2 arguments were supplied

#### `src\query\executor\search_executors.rs`: 2 occurrences

- Line 222: this method takes 3 arguments but 2 arguments were supplied
- Line 364: this method takes 2 arguments but 1 argument was supplied

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 2 occurrences

- Line 206: this method takes 3 arguments but 2 arguments were supplied
- Line 1011: this method takes 3 arguments but 2 arguments were supplied

#### `src\query\planner\statements\seeks\index_seek.rs`: 1 occurrences

- Line 29: this method takes 2 arguments but 1 argument was supplied

#### `src\query\executor\admin\index\tag_index.rs`: 1 occurrences

- Line 85: this method takes 2 arguments but 1 argument was supplied

#### `src\query\executor\admin\edge\create_edge.rs`: 1 occurrences

- Line 104: this method takes 2 arguments but 1 argument was supplied

#### `src\query\executor\admin\tag\create_tag.rs`: 1 occurrences

- Line 104: this method takes 2 arguments but 1 argument was supplied

#### `src\query\executor\admin\index\edge_index.rs`: 1 occurrences

- Line 85: this method takes 2 arguments but 1 argument was supplied

### error[E0050]: method `scan_all_edges` has 1 parameter but the declaration in trait `storage::storage_client::StorageClient::scan_all_edges` has 2: expected 2 parameters, found 1

**Total Occurrences**: 30  
**Unique Files**: 2

#### `src\query\visitor\deduce_type_visitor.rs`: 15 occurrences

- Line 587: method `scan_all_edges` has 1 parameter but the declaration in trait `storage::storage_client::StorageClient::scan_all_edges` has 2: expected 2 parameters, found 1
- Line 591: method `scan_vertices_by_tag` has 2 parameters but the declaration in trait `storage::storage_client::StorageClient::scan_vertices_by_tag` has 3: expected 3 parameters, found 2
- Line 595: method `insert_edge` has 2 parameters but the declaration in trait `storage::storage_client::StorageClient::insert_edge` has 3: expected 3 parameters, found 2
- ... 12 more occurrences in this file

#### `src\storage\test_mock.rs`: 15 occurrences

- Line 42: method `insert_edge` has 2 parameters but the declaration in trait `storage::storage_client::StorageClient::insert_edge` has 3: expected 3 parameters, found 2
- Line 47: method `get_edge` has 4 parameters but the declaration in trait `storage::storage_client::StorageClient::get_edge` has 5: expected 5 parameters, found 4
- Line 56: method `get_node_edges` has 3 parameters but the declaration in trait `storage::storage_client::StorageClient::get_node_edges` has 4: expected 4 parameters, found 3
- ... 12 more occurrences in this file

### error[E0407]: method `insert_node` is not a member of trait `StorageClient`: not a member of trait `StorageClient`

**Total Occurrences**: 26  
**Unique Files**: 2

#### `src\storage\test_mock.rs`: 13 occurrences

- Line 26: method `insert_node` is not a member of trait `StorageClient`: not a member of trait `StorageClient`
- Line 30: method `get_node` is not a member of trait `StorageClient`: not a member of trait `StorageClient`
- Line 34: method `update_node` is not a member of trait `StorageClient`: not a member of trait `StorageClient`
- ... 10 more occurrences in this file

#### `src\query\visitor\deduce_type_visitor.rs`: 13 occurrences

- Line 567: method `insert_node` is not a member of trait `StorageClient`: not a member of trait `StorageClient`
- Line 571: method `get_node` is not a member of trait `StorageClient`: not a member of trait `StorageClient`
- Line 575: method `update_node` is not a member of trait `StorageClient`: not a member of trait `StorageClient`
- ... 10 more occurrences in this file

### error[E0053]: method `create_space` has an incompatible type for trait: expected `core::error::StorageError`, found `core::error::DBError`

**Total Occurrences**: 12  
**Unique Files**: 1

#### `src\storage\test_mock.rs`: 12 occurrences

- Line 126: method `create_space` has an incompatible type for trait: expected `core::error::StorageError`, found `core::error::DBError`
- Line 130: method `drop_space` has an incompatible type for trait: expected `core::error::StorageError`, found `core::error::DBError`
- Line 134: method `get_space` has an incompatible type for trait: expected `core::error::StorageError`, found `core::error::DBError`
- ... 9 more occurrences in this file

### error[E0277]: `(dyn storage::storage_client::StorageClient + 'static)` doesn't implement `std::fmt::Debug`: `(dyn storage::storage_client::StorageClient + 'static)` cannot be formatted using `{:?}` because it doesn't implement `std::fmt::Debug`

**Total Occurrences**: 4  
**Unique Files**: 4

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 77: `(dyn storage::storage_client::StorageClient + 'static)` doesn't implement `std::fmt::Debug`: `(dyn storage::storage_client::StorageClient + 'static)` cannot be formatted using `{:?}` because it doesn't implement `std::fmt::Debug`

#### `src\storage\engine\memory_engine.rs`: 1 occurrences

- Line 320: `dyn storage::engine::StorageIterator` doesn't implement `std::fmt::Debug`: `dyn storage::engine::StorageIterator` cannot be formatted using `{:?}` because it doesn't implement `std::fmt::Debug`

#### `src\query\executor\executor_enum.rs`: 1 occurrences

- Line 118: the trait bound `S: storage_engine::StorageEngine` is not satisfied: the trait `storage_engine::StorageEngine` is not implemented for `S`

#### `src\storage\engine\redb_engine.rs`: 1 occurrences

- Line 385: `dyn storage::engine::StorageIterator` doesn't implement `std::fmt::Debug`: `dyn storage::engine::StorageIterator` cannot be formatted using `{:?}` because it doesn't implement `std::fmt::Debug`

### error[E0034]: multiple applicable items in scope: multiple `insert_edge` found

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\storage\memory_storage.rs`: 3 occurrences

- Line 1033: multiple applicable items in scope: multiple `insert_edge` found
- Line 1035: multiple applicable items in scope: multiple `get_edge` found
- Line 1055: multiple applicable items in scope: multiple `scan_vertices_by_tag` found

### error[E0369]: binary operation `==` cannot be applied to type `std::option::Option<Box<dyn storage::engine::StorageIterator>>`: std::option::Option<Box<dyn storage::engine::StorageIterator>>, std::option::Option<Box<dyn storage::engine::StorageIterator>>

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\storage\engine\redb_engine.rs`: 1 occurrences

- Line 385: binary operation `==` cannot be applied to type `std::option::Option<Box<dyn storage::engine::StorageIterator>>`: std::option::Option<Box<dyn storage::engine::StorageIterator>>, std::option::Option<Box<dyn storage::engine::StorageIterator>>

#### `src\storage\transaction\mvcc.rs`: 1 occurrences

- Line 491: binary operation `==` cannot be applied to type `std::option::Option<&std::sync::Arc<mvcc::VersionRecord>>`: std::option::Option<&std::sync::Arc<mvcc::VersionRecord>>, std::option::Option<&std::sync::Arc<mvcc::VersionRecord>>

#### `src\storage\engine\memory_engine.rs`: 1 occurrences

- Line 320: binary operation `==` cannot be applied to type `std::option::Option<Box<dyn storage::engine::StorageIterator>>`: std::option::Option<Box<dyn storage::engine::StorageIterator>>, std::option::Option<Box<dyn storage::engine::StorageIterator>>

### error[E0046]: not all trait items implemented, missing: `get_vertex`, `scan_vertices`, `insert_vertex`, `update_vertex`, `delete_vertex`, `batch_insert_vertices`, `create_tag_index`, `drop_tag_index`, `get_tag_index`, `list_tag_indexes`, `rebuild_tag_index`, `create_edge_index`, `drop_edge_index`, `get_edge_index`, `list_edge_indexes`, `rebuild_edge_index`, `insert_vertex_data`, `insert_edge_data`, `delete_vertex_data`, `delete_edge_data`, `update_data`, `change_password`: missing `get_vertex`, `scan_vertices`, `insert_vertex`, `update_vertex`, `delete_vertex`, `batch_insert_vertices`, `create_tag_index`, `drop_tag_index`, `get_tag_index`, `list_tag_indexes`, `rebuild_tag_index`, `create_edge_index`, `drop_edge_index`, `get_edge_index`, `list_edge_indexes`, `rebuild_edge_index`, `insert_vertex_data`, `insert_edge_data`, `delete_vertex_data`, `delete_edge_data`, `update_data`, `change_password` in implementation

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 566: not all trait items implemented, missing: `get_vertex`, `scan_vertices`, `insert_vertex`, `update_vertex`, `delete_vertex`, `batch_insert_vertices`, `create_tag_index`, `drop_tag_index`, `get_tag_index`, `list_tag_indexes`, `rebuild_tag_index`, `create_edge_index`, `drop_edge_index`, `get_edge_index`, `list_edge_indexes`, `rebuild_edge_index`, `insert_vertex_data`, `insert_edge_data`, `delete_vertex_data`, `delete_edge_data`, `update_data`, `change_password`: missing `get_vertex`, `scan_vertices`, `insert_vertex`, `update_vertex`, `delete_vertex`, `batch_insert_vertices`, `create_tag_index`, `drop_tag_index`, `get_tag_index`, `list_tag_indexes`, `rebuild_tag_index`, `create_edge_index`, `drop_edge_index`, `get_edge_index`, `list_edge_indexes`, `rebuild_edge_index`, `insert_vertex_data`, `insert_edge_data`, `delete_vertex_data`, `delete_edge_data`, `update_data`, `change_password` in implementation

#### `src\storage\test_mock.rs`: 1 occurrences

- Line 25: not all trait items implemented, missing: `get_vertex`, `scan_vertices`, `insert_vertex`, `update_vertex`, `delete_vertex`, `batch_insert_vertices`, `create_tag_index`, `drop_tag_index`, `get_tag_index`, `list_tag_indexes`, `rebuild_tag_index`, `create_edge_index`, `drop_edge_index`, `get_edge_index`, `list_edge_indexes`, `rebuild_edge_index`, `insert_vertex_data`, `insert_edge_data`, `delete_vertex_data`, `delete_edge_data`, `update_data`, `change_password`, `get_vertex_with_schema`, `get_edge_with_schema`, `scan_vertices_with_schema`, `scan_edges_with_schema`: missing `get_vertex`, `scan_vertices`, `insert_vertex`, `update_vertex`, `delete_vertex`, `batch_insert_vertices`, `create_tag_index`, `drop_tag_index`, `get_tag_index`, `list_tag_indexes`, `rebuild_tag_index`, `create_edge_index`, `drop_edge_index`, `get_edge_index`, `list_edge_indexes`, `rebuild_edge_index`, `insert_vertex_data`, `insert_edge_data`, `delete_vertex_data`, `delete_edge_data`, `update_data`, `change_password`, `get_vertex_with_schema`, `get_edge_with_schema`, `scan_vertices_with_schema`, `scan_edges_with_schema` in implementation

## Detailed Warning Categorization

### warning: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

**Total Occurrences**: 141  
**Unique Files**: 64

#### `src\storage\storage_client.rs`: 26 occurrences

- Line 101: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- Line 105: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- Line 109: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- ... 23 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 8 occurrences

- Line 319: unused import: `DataSet`
- Line 321: unused import: `crate::query::executor::executor_enum::ExecutorEnum`
- Line 322: unused import: `crate::query::executor::base::BaseExecutor`
- ... 5 more occurrences in this file

#### `src\query\optimizer\elimination_rules.rs`: 7 occurrences

- Line 87: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- Line 171: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- Line 316: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- ... 4 more occurrences in this file

#### `src\query\context\runtime_context.rs`: 5 occurrences

- Line 9: unused import: `crate::core::Value`
- Line 10: unused import: `crate::core::EdgeDirection`
- Line 12: unused import: `crate::query::planner::plan::management::ddl::space_ops::Schema`
- ... 2 more occurrences in this file

#### `src\storage\iterator\composite.rs`: 4 occurrences

- Line 120: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`
- Line 141: unused variable: `row`: help: if this is intentional, prefix it with an underscore: `_row`
- Line 120: variable does not need to be mutable
- ... 1 more occurrences in this file

#### `src\storage\iterator\predicate.rs`: 3 occurrences

- Line 10: unused import: `std::any::Any`
- Line 11: unused import: `std::collections::HashMap`
- Line 488: unused variable: `pred2`: help: if this is intentional, prefix it with an underscore: `_pred2`

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 3 occurrences

- Line 23: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`
- Line 477: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 483: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`

#### `src\query\planner\statements\match_planner.rs`: 3 occurrences

- Line 75: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 296: unreachable pattern: no value can reach this
- Line 470: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\storage\transaction\log.rs`: 3 occurrences

- Line 15: unused import: `self`
- Line 460: unused variable: `flushed`: help: if this is intentional, prefix it with an underscore: `_flushed`
- Line 458: unused variable: `min_lsn`: help: if this is intentional, prefix it with an underscore: `_min_lsn`

#### `src\query\planner\statements\paths\match_path_planner.rs`: 3 occurrences

- Line 431: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 437: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`
- Line 459: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`

#### `src\storage\plan\executors\mod.rs`: 3 occurrences

- Line 1: unused import: `ColumnSchema`
- Line 2: unused imports: `StorageError` and `Vertex`
- Line 3: unused import: `ScanResult`

#### `src\query\validator\insert_vertices_validator.rs`: 3 occurrences

- Line 204: unused import: `crate::core::Value`
- Line 48: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 79: unused variable: `tag_name`: help: if this is intentional, prefix it with an underscore: `_tag_name`

#### `src\query\parser\ast\utils.rs`: 2 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`
- Line 55: unused variable: `match_expression`: help: if this is intentional, prefix it with an underscore: `_match_expression`

#### `src\storage\transaction\snapshot.rs`: 2 occurrences

- Line 9: unused import: `LockType`
- Line 290: unused variable: `key_lock`: help: if this is intentional, prefix it with an underscore: `_key_lock`

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 149: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 177: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\query\validator\update_validator.rs`: 2 occurrences

- Line 34: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 168: unused variable: `op`: help: try ignoring the field: `op: _`

#### `src\storage\iterator\storage_iter.rs`: 2 occurrences

- Line 10: unused imports: `Edge`, `Value`, and `Vertex`
- Line 11: unused import: `std::sync::Arc`

#### `src\expression\storage\row_reader.rs`: 2 occurrences

- Line 313: unreachable pattern: no value can reach this
- Line 326: unreachable pattern: no value can reach this

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 494: unused import: `crate::query::executor::base::BaseExecutor`
- Line 495: unused import: `crate::query::executor::executor_enum::ExecutorEnum`

#### `src\query\planner\statements\match_statement_planner.rs`: 2 occurrences

- Line 86: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 353: unreachable pattern: no value can reach this

#### `src\query\optimizer\optimizer_config.rs`: 2 occurrences

- Line 134: unused import: `std::io::Write`
- Line 135: unused import: `tempfile::NamedTempFile`

#### `src\query\validator\insert_edges_validator.rs`: 2 occurrences

- Line 53: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 84: unused variable: `edge_name`: help: if this is intentional, prefix it with an underscore: `_edge_name`

#### `src\expression\context\query_expression_context.rs`: 2 occurrences

- Line 444: function cannot return without recursing: cannot return without recursing
- Line 463: function cannot return without recursing: cannot return without recursing

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\expression\context\default_context.rs`: 2 occurrences

- Line 524: function cannot return without recursing: cannot return without recursing
- Line 543: function cannot return without recursing: cannot return without recursing

#### `src\expression\context\row_context.rs`: 2 occurrences

- Line 249: function cannot return without recursing: cannot return without recursing
- Line 268: function cannot return without recursing: cannot return without recursing

#### `src\storage\transaction\traits.rs`: 2 occurrences

- Line 10: unused import: `Value`
- Line 11: unused imports: `LockManager`, `LogRecord`, `TransactionLog`, and `VersionVec`

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`
- Line 152: variable does not need to be mutable

#### `src\storage\memory_storage.rs`: 2 occurrences

- Line 1: unused import: `SchemaManager`
- Line 9: unused import: `RowReaderWrapper`

#### `src\expression\context\basic_context.rs`: 2 occurrences

- Line 592: function cannot return without recursing: cannot return without recursing
- Line 611: function cannot return without recursing: cannot return without recursing

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 50: unused import: `SpaceManageInfo`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 45: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\context\managers\schema_traits.rs`: 1 occurrences

- Line 247: unexpected `cfg` condition value: `schema-manager-default`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\query\visitor\ast_transformer.rs`: 1 occurrences

- Line 8: unused imports: `AlterStmt`, `Assignment`, `ChangePasswordStmt`, `CreateStmt`, `DeleteStmt`, `DescStmt`, `DropStmt`, `ExplainStmt`, `FetchStmt`, `FindPathStmt`, `GoStmt`, `InsertStmt`, `LookupStmt`, `MatchStmt`, `MergeStmt`, `PipeStmt`, `QueryStmt`, `RemoveStmt`, `ReturnStmt`, `SetStmt`, `ShowStmt`, `Stmt`, `SubgraphStmt`, `UnwindStmt`, `UpdateStmt`, `UseStmt`, and `WithStmt`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 305: unused variable: `tag_name`: help: if this is intentional, prefix it with an underscore: `_tag_name`

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 32: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\storage\plan\nodes\mod.rs`: 1 occurrences

- Line 2: unused imports: `Edge` and `Vertex`

#### `src\core\result\iterator.rs`: 1 occurrences

- Line 2: unused import: `crate::core::DBResult`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\storage\metadata\schema_manager.rs`: 1 occurrences

- Line 6: unused import: `IndexInfo`

#### `src\core\result\builder.rs`: 1 occurrences

- Line 2: unused import: `ResultMeta`

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 191: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 18: unused import: `crate::storage::StorageError`

#### `src\storage\storage_engine.rs`: 1 occurrences

- Line 7: unused import: `RowReaderWrapper`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 100: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\optimizer\predicate_pushdown.rs`: 1 occurrences

- Line 180: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\expression\context\traits.rs`: 1 occurrences

- Line 5: unused import: `crate::core::error::ExpressionError`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 55: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 450: unused variable: `test_expr`: help: if this is intentional, prefix it with an underscore: `_test_expr`

#### `src\storage\transaction\mvcc.rs`: 1 occurrences

- Line 11: unused import: `TransactionState`

#### `src\expression\evaluator\expression_evaluator.rs`: 1 occurrences

- Line 437: unreachable pattern: no value can reach this

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 36: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 184: value assigned to `last_changes` is never read

#### `src\query\planner\plan\execution_plan.rs`: 1 occurrences

- Line 68: unused variable: `n`: help: if this is intentional, prefix it with an underscore: `_n`

#### `src\storage\transaction\lock.rs`: 1 occurrences

- Line 11: unused import: `crate::core::StorageError`

#### `src\storage\test_mock.rs`: 1 occurrences

- Line 15: unused import: `StorageError`

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 13: unused import: `crate::expression::evaluator::traits::ExpressionContext`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 43: unused variable: `input_id`: help: if this is intentional, prefix it with an underscore: `_input_id`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\scheduler\execution_plan_analyzer.rs`: 1 occurrences

- Line 110: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode`

#### `src\query\optimizer\loop_unrolling.rs`: 1 occurrences

- Line 71: variable does not need to be mutable

