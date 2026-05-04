# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 2
- **Total Warnings**: 126
- **Total Issues**: 128
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 60
- **Files with Issues**: 34

## Error Statistics

**Total Errors**: 2

### Error Type Breakdown

- **error[E0061]**: 2 errors

### Files with Errors (Top 10)

- `src\storage\entity\edge_storage.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 126

### Warning Type Breakdown

- **warning**: 126 warnings

### Files with Warnings (Top 10)

- `src\transaction\undo_log.rs`: 53 warnings
- `src\transaction\update_transaction.rs`: 8 warnings
- `src\storage\metadata\inmemory_schema_manager.rs`: 8 warnings
- `src\storage\entity\edge_storage.rs`: 8 warnings
- `src\storage\entity\vertex_storage.rs`: 5 warnings
- `src\storage\vertex\vertex_table.rs`: 4 warnings
- `src\query\planning\plan\validation\schema_validation.rs`: 3 warnings
- `src\transaction\version_manager.rs`: 3 warnings
- `src\storage\property_graph.rs`: 3 warnings
- `src\transaction\read_transaction.rs`: 3 warnings

## Detailed Error Categorization

### error[E0061]: this method takes 2 arguments but 3 arguments were supplied

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\storage\entity\edge_storage.rs`: 2 occurrences

- Line 971: this method takes 2 arguments but 3 arguments were supplied
- Line 983: this method takes 5 arguments but 6 arguments were supplied

## Detailed Warning Categorization

### warning: unused import: `std::sync::RwLock`

**Total Occurrences**: 126  
**Unique Files**: 34

#### `src\transaction\undo_log.rs`: 53 occurrences

- Line 6: unused import: `std::collections::HashMap`
- Line 7: unused import: `std::sync::Arc`
- Line 625: unused variable: `label`: help: if this is intentional, prefix it with an underscore: `_label`
- ... 50 more occurrences in this file

#### `src\storage\metadata\inmemory_schema_manager.rs`: 8 occurrences

- Line 232: unused variable: `space_info`: help: if this is intentional, prefix it with an underscore: `_space_info`
- Line 250: unused variable: `space_info`: help: if this is intentional, prefix it with an underscore: `_space_info`
- Line 273: unused variable: `space_info`: help: if this is intentional, prefix it with an underscore: `_space_info`
- ... 5 more occurrences in this file

#### `src\storage\entity\edge_storage.rs`: 8 occurrences

- Line 13: unused import: `IndexMetadataManager`
- Line 108: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- Line 112: unused variable: `rank`: help: if this is intentional, prefix it with an underscore: `_rank`
- ... 5 more occurrences in this file

#### `src\transaction\update_transaction.rs`: 8 occurrences

- Line 9: unused import: `std::sync::Arc`
- Line 11: unused import: `decode_from_slice`
- Line 17: unused imports: `DeleteEdgeTypeUndo`, `DeleteVertexTypeUndo`, `InsertEdgeUndo`, and `InsertVertexUndo`
- ... 5 more occurrences in this file

#### `src\storage\entity\vertex_storage.rs`: 5 occurrences

- Line 12: unused import: `IndexMetadataManager`
- Line 64: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- Line 161: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- ... 2 more occurrences in this file

#### `src\storage\vertex\vertex_table.rs`: 4 occurrences

- Line 7: unused import: `std::sync::RwLock`
- Line 9: unused import: `INVALID_TIMESTAMP`
- Line 10: unused import: `DataType`
- ... 1 more occurrences in this file

#### `src\storage\property_graph.rs`: 3 occurrences

- Line 7: unused import: `std::sync::RwLock`
- Line 9: unused import: `DataType`
- Line 11: unused import: `EdgeDirection`

#### `src\query\planning\plan\validation\schema_validation.rs`: 3 occurrences

- Line 332: unused variable: `errors`: help: if this is intentional, prefix it with an underscore: `_errors`
- Line 334: unused variable: `node_id`: help: if this is intentional, prefix it with an underscore: `_node_id`
- Line 341: unused variable: `project_node`: help: if this is intentional, prefix it with an underscore: `_project_node`

#### `src\transaction\read_transaction.rs`: 3 occurrences

- Line 7: unused import: `std::sync::Arc`
- Line 11: unused import: `super::undo_log::UndoTarget`
- Line 176: unused variable: `ts`: help: if this is intentional, prefix it with an underscore: `_ts`

#### `src\transaction\version_manager.rs`: 3 occurrences

- Line 6: unused import: `std::collections::HashSet`
- Line 285: variable `expected` is assigned to, but never used
- Line 291: value assigned to `expected` is never read

#### `src\transaction\manager.rs`: 2 occurrences

- Line 9: unused import: `std::time::Duration`
- Line 18: unused import: `super::wal::writer::WalWriter`

#### `src\transaction\insert_transaction.rs`: 2 occurrences

- Line 9: unused import: `std::sync::Arc`
- Line 16: unused imports: `CreateEdgeTypeRedo` and `CreateVertexTypeRedo`

#### `src\storage\container\mmap_container.rs`: 2 occurrences

- Line 7: unused import: `Read`
- Line 318: unnecessary `unsafe` block: unnecessary `unsafe` block

#### `src\query\planning\statements\seeks\multi_label_index_selector.rs`: 2 occurrences

- Line 50: unused variable: `covered_labels`: help: try ignoring the field: `covered_labels: _`
- Line 229: unused variable: `predicates`: help: if this is intentional, prefix it with an underscore: `_predicates`

#### `src\transaction\manager_test.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\cache\manager.rs`: 1 occurrences

- Line 24: unused imports: `CteCacheStatsSnapshot` and `PlanCacheStatsSnapshot`

#### `src\storage\edge\mod.rs`: 1 occurrences

- Line 22: unused imports: `AtomicU32`, `AtomicU64`, and `Ordering`

#### `src\transaction\context.rs`: 1 occurrences

- Line 7: unused import: `std::sync::Arc`

#### `src\storage\graph_storage.rs`: 1 occurrences

- Line 16: unused import: `IndexMetadataManager`

#### `src\storage\vertex\mod.rs`: 1 occurrences

- Line 22: unused imports: `AtomicU32` and `Ordering`

#### `src\query\planning\plan\core\nodes\base\plan_node_operations.rs`: 1 occurrences

- Line 151: unreachable pattern: no value can reach this

#### `src\query\metadata\schema_provider.rs`: 1 occurrences

- Line 7: unused imports: `EngineType` and `SpaceStatus`

#### `src\storage\edge\mutable_csr.rs`: 1 occurrences

- Line 9: unused import: `MAX_TIMESTAMP`

#### `src\transaction\compact_transaction.rs`: 1 occurrences

- Line 7: unused import: `std::sync::Arc`

#### `src\storage\vertex\column_store.rs`: 1 occurrences

- Line 144: unused variable: `start`: help: if this is intentional, prefix it with an underscore: `_start`

#### `src\storage\entity\event_storage.rs`: 1 occurrences

- Line 7: unused import: `crate::storage::metadata::inmemory_schema_manager::InMemorySchemaManager`

#### `src\storage\edge\property_table.rs`: 1 occurrences

- Line 270: unused variable: `offset2`: help: if this is intentional, prefix it with an underscore: `_offset2`

#### `src\transaction\wal\parser.rs`: 1 occurrences

- Line 9: unused import: `WalOpType`

#### `src\api\mod.rs`: 1 occurrences

- Line 34: unused import: `crate::storage::api::StorageClient`

#### `src\storage\test_mock.rs`: 1 occurrences

- Line 19: unused import: `PropertyGraphConfig`

#### `src\api\embedded\database.rs`: 1 occurrences

- Line 105: unused variable: `db`: help: if this is intentional, prefix it with an underscore: `_db`

#### `src\storage\edge\edge_table.rs`: 1 occurrences

- Line 79: unused variable: `edge_capacity`: help: if this is intentional, prefix it with an underscore: `_edge_capacity`

#### `src\storage\vertex\vertex_timestamp.rs`: 1 occurrences

- Line 50: unused variable: `ts`: help: if this is intentional, prefix it with an underscore: `_ts`

#### `src\query\context\query_context_builder.rs`: 1 occurrences

- Line 5: unused imports: `EngineType` and `SpaceStatus`

