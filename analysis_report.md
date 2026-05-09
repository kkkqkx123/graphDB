# Type Check Report

## Type Issues Summary

- **Total**: 46
- **⚠️** warning: 46
- **Categories**: 40
- **Files Affected**: 25

## Breakdown by Category

- **field `sync_manager` is never read**: 2 occurrence(s)
- **multiple associated items are never**: 2 occurrence(s)
- **field `config` is never read**: 2 occurrence(s)
- **type `PendingWrite` is more private**: 2 occurrence(s)
- **methods `get_space_id`, `get_current_txn_id`, and `get_sync_manager`**: 2 occurrence(s)
- **constant `INVALID_TIMESTAMP` is never used**: 2 occurrence(s)
- **struct `DeltaCompressor` is never constructed**: 1 occurrence(s)
- **fields `last_cleanup_time` and `group_commit` are**: 1 occurrence(s)
- **field `start` is never read**: 1 occurrence(s)
- **methods `nbr_start`, `nbr_ptr`, and `nbr_ptr_mut`**: 1 occurrence(s)
- **field `background_thread` is never read**: 1 occurrence(s)
- **struct `IndexUpdateContext` is never constructed**: 1 occurrence(s)
- **enum `IndexUndoEntry` is never used**: 1 occurrence(s)
- **associated functions `insert_vertex_index`, `delete_vertex_index`, `insert_edge_index`,**: 1 occurrence(s)
- **field `footer` is never read**: 1 occurrence(s)
- **fields `deleted_vertex_properties` and `deleted_edge_properties` are**: 1 occurrence(s)
- **field `is_leader` is never read**: 1 occurrence(s)
- **method `dictionary_size` is never used**: 1 occurrence(s)
- **method `find_available_path` is never used**: 1 occurrence(s)
- **associated items `new`, `with_base`, `compress`,**: 1 occurrence(s)
- **field `page_writer` is never read**: 1 occurrence(s)
- **field `sample_rate` is never read**: 1 occurrence(s)
- **method `remaining` is never used**: 1 occurrence(s)
- **struct `IndexUndoLog` is never constructed**: 1 occurrence(s)
- **type alias `IndexKey` is never**: 1 occurrence(s)
- **function `get_system_huge_page_size` is never used**: 1 occurrence(s)
- **fields `page_directory` and `config` are**: 1 occurrence(s)
- **method `write_wal` is never used**: 1 occurrence(s)
- **method `is_empty` is never used**: 1 occurrence(s)
- **field `estimated_rows` is never read**: 1 occurrence(s)
- **associated function `notify_error` is never**: 1 occurrence(s)
- **field `return_planner` is never read**: 1 occurrence(s)
- **constant `DATA_FORMAT_VERSION` is never used**: 1 occurrence(s)
- **calls to `std::mem::drop` with a**: 1 occurrence(s)
- **method `prefix` is never used**: 1 occurrence(s)
- **fields `code` and `frequency` are**: 1 occurrence(s)
- **field `data` is never read**: 1 occurrence(s)
- **field `path` is never read**: 1 occurrence(s)
- **methods `reset` and `test` are**: 1 occurrence(s)
- **field `encoded_count` is never read**: 1 occurrence(s)

## Details by File

### `src\transaction\wal\writer.rs` (8 item(s))

- ⚠️ **warning** at line 98:5: type `PendingWrite` is more private than the item `GroupCommitManager::collect_batch`: method `GroupCommitManager::collect_batch` is reachable at visibility `pub`
- ⚠️ **warning** at line 117:5: type `PendingWrite` is more private than the item `GroupCommitManager::notify_results`: associated function `GroupCommitManager::notify_results` is reachable at visibility `pub`
- ⚠️ **warning** at line 35:5: field `data` is never read
- ⚠️ **warning** at line 45:5: field `is_leader` is never read
- ⚠️ **warning** at line 126:19: associated function `notify_error` is never used
- ⚠️ **warning** at line 169:5: fields `last_cleanup_time` and `group_commit` are never read
- ⚠️ **warning** at line 258:8: method `find_available_path` is never used
- ⚠️ **warning** at line 1021:9: calls to `std::mem::drop` with a reference instead of an owned value does nothing

### `src\storage\index\secondary\index_updater.rs` (6 item(s))

- ⚠️ **warning** at line 515:12: struct `IndexUpdateContext` is never constructed
- ⚠️ **warning** at line 525:12: multiple associated items are never used
- ⚠️ **warning** at line 616:10: enum `IndexUndoEntry` is never used
- ⚠️ **warning** at line 654:12: associated functions `insert_vertex_index`, `delete_vertex_index`, `insert_edge_index`, and `delete_edge_index` are never used
- ⚠️ **warning** at line 725:12: struct `IndexUndoLog` is never constructed
- ⚠️ **warning** at line 730:12: multiple associated items are never used

### `src\storage\index\secondary\key_codec\compression.rs` (4 item(s))

- ⚠️ **warning** at line 133:12: method `prefix` is never used
- ⚠️ **warning** at line 239:12: method `dictionary_size` is never used
- ⚠️ **warning** at line 251:12: struct `DeltaCompressor` is never constructed
- ⚠️ **warning** at line 256:12: associated items `new`, `with_base`, `compress`, and `decompress` are never used

### `src\storage\entity\vertex_storage.rs` (3 item(s))

- ⚠️ **warning** at line 17:7: constant `INVALID_TIMESTAMP` is never used
- ⚠️ **warning** at line 25:5: field `sync_manager` is never read
- ⚠️ **warning** at line 51:8: methods `get_space_id`, `get_current_txn_id`, and `get_sync_manager` are never used

### `src\storage\entity\edge_storage.rs` (3 item(s))

- ⚠️ **warning** at line 19:7: constant `INVALID_TIMESTAMP` is never used
- ⚠️ **warning** at line 27:5: field `sync_manager` is never read
- ⚠️ **warning** at line 85:8: methods `get_space_id`, `get_current_txn_id`, and `get_sync_manager` are never used

### `src\storage\engine\property_graph.rs` (2 item(s))

- ⚠️ **warning** at line 48:7: constant `DATA_FORMAT_VERSION` is never used
- ⚠️ **warning** at line 218:8: method `write_wal` is never used

### `src\storage\vertex\encoding\fsst.rs` (2 item(s))

- ⚠️ **warning** at line 22:5: fields `code` and `frequency` are never read
- ⚠️ **warning** at line 99:5: field `encoded_count` is never read

### `src\transaction\wal\parser.rs` (1 item(s))

- ⚠️ **warning** at line 523:8: method `is_empty` is never used

### `src\storage\edge\mutable_csr.rs` (1 item(s))

- ⚠️ **warning** at line 193:8: methods `nbr_start`, `nbr_ptr`, and `nbr_ptr_mut` are never used

### `src\storage\index\secondary\key_codec\key_types.rs` (1 item(s))

- ⚠️ **warning** at line 32:10: type alias `IndexKey` is never used

### `src\storage\persistence\flush_manager.rs` (1 item(s))

- ⚠️ **warning** at line 49:5: field `background_thread` is never read

### `src\storage\edge\csr_persistence.rs` (1 item(s))

- ⚠️ **warning** at line 81:5: field `path` is never read

### `src\storage\stats\column_stats.rs` (1 item(s))

- ⚠️ **warning** at line 191:5: field `sample_rate` is never read

### `src\storage\vertex\vertex_table.rs` (1 item(s))

- ⚠️ **warning** at line 44:5: field `config` is never read

### `src\storage\persistence\page_writer.rs` (1 item(s))

- ⚠️ **warning** at line 441:5: field `page_writer` is never read

### `src\storage\memory\huge_pages.rs` (1 item(s))

- ⚠️ **warning** at line 310:8: function `get_system_huge_page_size` is never used

### `src\query\planning\statements\seeks\seek_strategy_base.rs` (1 item(s))

- ⚠️ **warning** at line 374:5: field `estimated_rows` is never read

### `src\storage\persistence\sstable.rs` (1 item(s))

- ⚠️ **warning** at line 349:5: field `footer` is never read

### `src\query\planning\statements\match_statement_planner.rs` (1 item(s))

- ⚠️ **warning** at line 55:5: field `return_planner` is never read

### `src\storage\container\arena_allocator.rs` (1 item(s))

- ⚠️ **warning** at line 61:8: method `remaining` is never used

### `src\storage\edge\edge_table.rs` (1 item(s))

- ⚠️ **warning** at line 40:5: field `config` is never read

### `src\transaction\update_transaction.rs` (1 item(s))

- ⚠️ **warning** at line 159:5: fields `deleted_vertex_properties` and `deleted_edge_properties` are never read

### `src\storage\page\page_manager.rs` (1 item(s))

- ⚠️ **warning** at line 61:5: fields `page_directory` and `config` are never read

### `src\transaction\version_manager.rs` (1 item(s))

- ⚠️ **warning** at line 67:8: methods `reset` and `test` are never used

### `src\storage\page\flat_csr.rs` (1 item(s))

- ⚠️ **warning** at line 370:5: field `start` is never read

