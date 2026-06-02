# Remaining Dead Code Analysis (70 warnings)

Base: 99 warnings after removal passes completed (Categories A-C, E partial).
Current: 70 warnings remain.

## 1. edge/csr_trait.rs — 4 warnings (Category B/D)

| Item | Type | Action |
|------|------|--------|
| `CsrBase::is_empty()` line 15 | Trait default method | **DELETE** — never called |
| `CsrBase::csr_type()` line 19 | Trait method | **DELETE** — never called |
| `MutableCsrTrait::degree()/has_edge()/compact()` line 72 | Trait methods | **DELETE** — never called |
| `CsrType` enum line 101 + `from_strategy()` line 108 | Enum + method | **DELETE** — all variants are dead after trait method removal |

> All CSR traits except `CsrBase`/`MutableCsrTrait` base definitions are kept per Category D. But these individual methods are dead.

## 2. edge/csr.rs — 6 methods (Category B)

| Method | Action |
|--------|--------|
| `vertex_capacity()`, `edge_count()`, `get_edge_by_id()` | **DELETE** — public API but never called |
| `offsets()`, `edges()` | **DELETE** — raw accessors never used externally |
| `from_raw()` | **DELETE** — only used to construct Csr, but no caller exists |

## 3. edge/mod.rs — 2 warnings (Category B)

| Method | Action |
|--------|--------|
| `EdgeSchema::from_edge_type_info()` | **DELETE** — unused constructor |
| `Nbr::is_deleted()` | **DELETE** — unused method |

## 4. edge/edge_table.rs — 11 warnings (Category B)

| Method | Action |
|--------|--------|
| `oe_offset`, `ie_offset` fields in `UpdateEdgePropertyByOffsetParams` | **DELETE** — never read |
| `delete_edge_by_id()`, `get_edge_by_offset()` | **DELETE** |
| `out_degree()`, `in_degree()` | **DELETE** |
| `revert_delete_edge()`, `vertex_capacity()` | **DELETE** |
| `get_properties()`, `compact()`, `clear()` | **DELETE** |

## 5. edge/mutable_csr.rs — 14 methods (Category B)

| Method | Action |
|--------|--------|
| `insert_edge_with_expand()`, `revert_delete()` | **DELETE** |
| `get_degrees()`, `get_adj_offsets()`, `get_nbr_list()` | **DELETE** |
| `has_edge()`, `get_edge_by_id()` | **DELETE** |
| `compact()` | **DELETE** |
| `nbr_slice()`, `nbr_slice_mut()` | **DELETE** |
| `degrees()`, `primary_capacities()`, `adj_offsets()`, `overflow_starts()` | **DELETE** |
| `degree()` line 1123 | **FIX BUG** — recurses infinitely (`MutableCsr::degree` calls itself). Either delete or delegate to inner CSR. |

## 6. edge/mutable_csr_variant.rs — 4 methods (Category B)

| Method | Action |
|--------|--------|
| `is_single()`, `is_multiple()`, `resize()`, `revert_delete()` | **DELETE** |

## 7. edge/property_table.rs — 21 warnings (Category B)

| Item | Action |
|------|--------|
| `OverflowPointer::overflow_id`, `original_size` fields | **DELETE** — never read |
| `OverflowStore::entry_count()` | **DELETE** |
| `RowGroup::row_count()`, `contains_row()` | **DELETE** |
| PropertyTable: `rename_property()`, `remove_property()`, `get_property()`, `get_property_by_id()`, `row_count()`, `property_count()`, `schema()`, `property_names()`, `clear()`, `get_schema()`, `get_schema_by_id()`, `get_property_type()`, `name_indexer()`, `row_group_count()`, `get_row_group()`, `get_row_group_for_row()`, `compact_row_group()`, `memory_size()`, `overflow_store()`, `overflow_count()` | **DELETE** |

## 8. edge/single_mutable_csr.rs — 8 methods (Category B)

| Method | Action |
|--------|--------|
| `is_empty()`, `update_edge()` | **DELETE** |
| `revert_delete()`, `revert_delete_by_id()` | **DELETE** |
| `degree()`, `has_edge()` | **DELETE** |
| `compact()`, `batch_put_edges()` | **DELETE** |

## 9. engine/graph_storage/mod.rs — 3 methods (Category B)

| Method | Action |
|--------|--------|
| `GraphStorage::index_gc_manager()`, `get_db()`, `persistence()` | **DELETE** — unused `pub(crate)` methods |

## 10. engine/graph_storage/type_utils.rs — 1 item (Category B)

| Item | Action |
|------|--------|
| `vertex_id_to_string()` | **DELETE** — unused utility function |

## 11. engine/persistence_coordinator.rs — 10 warnings (Category B/E)

| Item | Action |
|------|--------|
| `PersistenceState::FlushingData` variant | **DELETE** |
| `CheckpointInfo`: `checkpoint_id`, `timestamp`, `created_at`, `data_size`, `vertex_count`, `edge_count` fields (but keep `lsn`) | **DELETE** fields — never read |
| `FlushStats` struct + `PersistenceStats` struct | **DELETE** — never constructed |
| `PersistenceCoordinator`: `checkpoint_manager()`, `snapshot_manager()`, `current_state()`, `record_wal_entry()`, `recover()`, `cleanup_old_checkpoints()`, `get_stats()`, `register_transaction()`, `unregister_transaction()`, `trigger_flush()` | **DELETE** |

## 12. engine/snapshot_manager.rs — 6 warnings (Category B)

| Item | Action |
|------|--------|
| `SnapshotOptions::incremental` field | **DELETE** |
| `set_retention_policy()`, `list_snapshots()`, `current_snapshot_id()`, `snapshot_exists()`, `restore_snapshot()` | **DELETE** |

## 13. engine/transaction/ops.rs — 7 associated functions (Category B)

| Method | Action |
|--------|--------|
| `get_vertex_id()`, `get_vertex_oid()` | **DELETE** |
| `get_vertex_property_types()`, `get_edge_property_types()` | **DELETE** |
| `vertex_label_num()`, `lid_num()` | **DELETE** |
| `insert_vertex_undo()` | **DELETE** |

## 14. engine/wal_manager.rs — 2 methods (Category B)

| Method | Action |
|--------|--------|
| `append_entry()`, `close()` | **DELETE** |

## 15. index/edge_index_manager.rs — 22 methods (Category D)

| Method | Action |
|--------|--------|
| `with_compression()`, `is_compression_enabled()`, `train_compression()`, `compression_ratio()` | **DELETE** |
| All index CRUD methods (e.g. `update_edge_indexes`, `delete_edge_indexes`, `lookup_edge_index`, `scan_index_entries`, etc.) | **DELETE** — all native/native_mvcc variants are dead; only the core `update`/`delete`/`clear` methods called from `IndexDataManager` impl are live |

> **Decision**: Delete all dead index CRUD methods. Keep the `IndexDataManager` trait impls' method signatures only.

## 16. index/generic_index_manager.rs — 4 methods (Category D)

| Method | Action |
|--------|--------|
| `with_compression()`, `is_compression_enabled()`, `train_compression()`, `compression_ratio()` | **DELETE** — compression setup/training never called |

## 17. index/index_data_manager.rs — trait methods (Category D — keep traits)

| Trait | Methods | Action |
|-------|---------|--------|
| `VertexIndexOps` | `update_vertex_indexes()`, `delete_vertex_indexes()`, `delete_vertex_index_single()`, `delete_vertex_index_single_mvcc()`, `delete_tag_indexes()`, `clear_tag_index()`, `build_vertex_index_entry()`, `update_vertex_indexes_native()`, `update_vertex_indexes_native_mvcc()`, `delete_vertex_indexes_native()`, `delete_vertex_indexes_native_mvcc()`, `lookup_tag_index_native()`, `lookup_tag_index_native_mvcc()` | **KEEP trait definitions** per Category D, but **DELETE** individual never-used methods |
| `EdgeIndexOps` | `update_edge_indexes()`, `delete_edge_indexes()`, `delete_edge_index_single()`, `delete_edge_index_single_mvcc()`, `lookup_edge_index()`, `lookup_edge_index_mvcc()`, `clear_edge_index()`, `build_edge_index_entry()`, `update_edge_indexes_native()`, `update_edge_indexes_native_mvcc()`, `delete_edge_indexes_native()`, `delete_edge_indexes_native_mvcc()`, `lookup_edge_index_native()`, `lookup_edge_index_native_mvcc()` | Same — **DELETE** dead methods, **KEEP** trait defs with only live methods |

> Note: If ALL methods in a trait are dead, consider whether the trait itself should be retained. Currently `VertexIndexOps` impls exist in `VertexIndexManager` and `EdgeIndexOps` in `EdgeIndexManager`.

## 18. index/vertex_index_manager.rs — 28 methods (Category D)

| Method | Action |
|--------|--------|
| `with_compression()`, `is_compression_enabled()`, `train_compression()`, `compression_ratio()` | **DELETE** |
| `update_vertex_indexes()`, `delete_vertex_indexes()`, `delete_vertex_index_single()`, `delete_tag_indexes()`, `clear_tag_index()` | **DELETE** — unused (but these are the trait impls for `VertexIndexOps`) |
| `lookup_tag_index()`, `lookup_tag_index_range()`, `lookup_tag_index_range_mvcc()` | **DELETE** |
| `scan_index_entries()`, `scan_index_entries_mvcc()` | **DELETE** |
| `update_composite_vertex_indexes()`, `update_composite_vertex_indexes_mvcc()` | **DELETE** |
| `lookup_composite_tag_index()`, `lookup_composite_tag_index_mvcc()`, `lookup_composite_tag_index_prefix()` | **DELETE** |
| `update_vertex_indexes_native()`, `update_vertex_indexes_native_mvcc()` | **DELETE** |
| `delete_vertex_indexes_native()`, `delete_vertex_indexes_native_mvcc()` | **DELETE** |
| `lookup_tag_index_native()`, `lookup_tag_index_native_mvcc()` | **DELETE** |
| `lookup_tag_index_range_native()`, `lookup_tag_index_range_native_mvcc()` | **DELETE** |

> Note: `update_vertex_indexes`, `delete_vertex_indexes`, `delete_vertex_index_single`, `delete_tag_indexes`, `clear_tag_index` are implementations of `VertexIndexOps` trait. Keep the trait definitions, remove these impl methods.

## 19. index/key_codec/compression.rs — 5 warnings (Category D/E)

Already partially cleaned. Remaining:

| Item | Action |
|------|--------|
| `CompressionConfig::compression_type` field | **DELETE** — never read after variant pruning |
| `CompressionConfig::new()`, `with_compression_type()` | **DELETE** — `default()` is used instead |
| `PrefixCompressor::train()`, `prefix()` | **DELETE** — training is done via `IndexCompressor::train_keys` but that's also unused |
| `IndexCompressor::config` field | **DELETE** — never read |
| `train_keys()`, `compression_ratio()`, `config()` | **DELETE** |

> After removing these, `PrefixCompressor` and `IndexCompressor` become simpler wrappers. Consider inlining or keeping as-is.

## 20. index/key_codec/key_builder.rs — 7 methods (Category D)

Already partially cleaned. Remaining dead:
- `build_vertex_index_key_native()`, `build_vertex_reverse_key_native()`, `build_vertex_reverse_prefix_native()`
- `build_edge_index_key_native()`, `build_edge_reverse_key_native()`, `build_edge_reverse_prefix_native_with_dst()`
- `build_composite_vertex_index_key()`

→ **DELETE** — all are dead.

## 21. index/key_codec/key_parser.rs — 5 methods (Category D)

Already partially cleaned. Remaining dead:
- `parse_vertex_id_from_key_native()`, `parse_vertex_reverse_key_native()`
- `parse_edge_ids_from_key_native()`, `parse_edge_reverse_key_native()`
- `parse_composite_vertex_index_key()`

→ **DELETE** — all are dead. Note: if the associated `build_*` functions in KeyBuilder are deleted, the corresponding `parse_*` functions must also go.

## 22. index/index_gc_manager.rs — 1 warning (Category D)

| Method | Action |
|--------|--------|
| `with_defaults()` | **DELETE** — `new()` is used instead |

> Check that `new()` doesn't call `with_defaults()` internally before deleting.

## 23. vertex/column_store.rs — 41 warnings (Category B)

This file has the most remaining warnings. Break down by section:

### ColumnStorage trait (line 31)
| Method | Action |
|--------|--------|
| `data()`, `data_size()`, `null_bitmap_raw()`, `load_data()` | KEEP trait definitions (Category D principle), but these specific methods are only used inside `Column` impl. **DELETE** from trait if all callers removed. |

### Column constructors
| Method | Action |
|--------|--------|
| `FixedWidthColumn::with_capacity()` | **DELETE** |
| `VariableWidthColumn::with_capacity()` | **DELETE** |

### Column impl (line 810) — ~20 methods
| Method | Action |
|--------|--------|
| `with_capacity()` | **DELETE** |
| `element_size()` | **DELETE** |
| `data_size()`, `data()`, `null_bitmap_raw()` | **DELETE** — debug/inspection methods |
| `reset_encoding()`, `recompress()` | **DELETE** |
| `load_data()` | **DELETE** |
| `collect_stats()`, `update_int_stats()` | **DELETE** — unused |
| `encoding()` | **DELETE** |
| `decode_fsst_value()`, `get_encoded_fsst()`, `fsst_encoder()`, `fsst_column()`, `fsst_symbol_table_bytes()`, `load_fsst_from_data()`, `append_fsst_value()`, `can_append_fsst()`, `fsst_row_count()`, `fsst_compression_ratio()`, `auto_compress()` | **DELETE** — all FSST helper methods are dead |
| `get_f32()`, `is_empty()`, `is_null()`, `compression_ratio()`, `clear()`, `encoder()` on AlpColumn | Not in column_store.rs — these are in alp.rs. See section 24. |

### ColumnStore impl (line 1663)
| Method | Action |
|--------|--------|
| `get_property()` | **DELETE** |
| `reset_encodings()`, `recompress_all()` | **DELETE** |
| `column_names()`, `load_column()`, `iter_columns()` | **DELETE** |
| `apply_fsst_to_string_columns()` | **DELETE** |

## 24. vertex/encoding/alp.rs — 12 methods (Category B/E)

### AlpEncoder (line 64)
| Method | Action |
|--------|--------|
| `analyze_f32()` | Already removed in earlier pass — check if residual warning |
| `compress_f32()`, `decompress_f32()` | **DELETE** |
| `factor()`, `float_type()`, `bit_width()` | **DELETE** — accessors for internal state never used externally |
| `compression_ratio()` | **DELETE** |

### AlpColumn (line 265)
| Method | Action |
|--------|--------|
| `get_f32()` | **DELETE** |
| `is_empty()`, `is_null()` | **DELETE** |
| `compression_ratio()`, `clear()`, `encoder()` | **DELETE** |

## 25. vertex/encoding/bitpacking.rs — 9 methods (Category B/E)

### BitPackedColumn (line 30)
| Method | Action |
|--------|--------|
| `with_capacity()` | **DELETE** |
| `is_empty()`, `bit_width()`, `min_value()`, `max_value()` | **DELETE** |
| `compression_ratio()`, `clear()` | **DELETE** |
| `to_values()` | **DELETE** |

### BitPackedIntColumn (line 285)
| Method | Action |
|--------|--------|
| `new()`, `append()` | **DELETE** |
| `is_empty()`, `is_null()` | **DELETE** |
| `compression_ratio()`, `clear()` | **DELETE** |

## 26. vertex/encoding/dictionary.rs — 9 methods (Category B)

### StringDictionary (line 42)
| Method | Action |
|--------|--------|
| `get_index()`, `len()`, `is_empty()` | **DELETE** |
| `clear()`, `iter()` | **DELETE** |

### DictionaryEncoder (line 125)
| Method | Action |
|--------|--------|
| `is_empty()`, `dictionary_size()` | **DELETE** |
| `clear()`, `dictionary()`, `indices()` | **DELETE** |

### DictionaryColumn (line 214)
| Method | Action |
|--------|--------|
| `is_null()`, `is_empty()`, `clear()` | **DELETE** |

## 27. vertex/encoding/fsst.rs — 8 methods (Category B/E)

### FsstSymbolTable (line 62)
| Method | Action |
|--------|--------|
| `len()`, `is_empty()` | **DELETE** |

### FsstEncoder (line 210)
| Method | Action |
|--------|--------|
| `compression_ratio()`, `symbol_count()` | **DELETE** |
| `with_table()` | **DELETE** |

### FsstColumn (line 245)
| Method | Action |
|--------|--------|
| `train_and_build()`, `append()` | **DELETE** |
| `is_empty()`, `is_null()` | **DELETE** |
| `compression_ratio()`, `clear()`, `encoder()` | **DELETE** |

## 28. vertex/encoding/mod.rs — 3 warnings (Category D/E)

| Item | Action |
|------|--------|
| `EncodedColumn` trait | **DELETE** — never used (no implementations) |
| `ColumnEncoding::is_empty()`, `is_null()`, `compression_ratio()`, `clear()` | **DELETE** |
| Note: `EncodedColumn` might already be removed. Check. |

## 29. vertex/encoding/rle.rs — 6 methods (Category B)

### RleEncoder (line 28)
| Method | Action |
|--------|--------|
| `with_capacity()`, `is_empty()`, `run_count()`, `runs()`, `clear()` | **DELETE** |

### RleIntColumn (line 146)
| Method | Action |
|--------|--------|
| `is_null()`, `is_empty()`, `run_count()`, `clear()` | **DELETE** |

### RleBoolColumn (line 220)
| Method | Action |
|--------|--------|
| `is_null()`, `is_empty()`, `run_count()`, `clear()` | **DELETE** |

## 30. vertex/encoding/selector.rs — 4 warnings (Category E)

Already partially cleaned. Remaining:

| Item | Action |
|------|--------|
| `ColumnStats::access_count` field | **DELETE** — never read |
| `ColumnStats::null_ratio()`, `is_cold()` | **DELETE** — never called |
| `CompressionConfig::hot_access_threshold`, `cold_access_threshold` fields | **DELETE** — never read |
| `CompressionSelector::config()` | **DELETE** — never called |

## 31. vertex/id_indexer.rs — 9 methods (Category B)

| Method | Action |
|--------|--------|
| `IdIndexerConfig::with_growth_factor()`, `with_max_capacity()`, `with_free_list()` | **DELETE** |
| `IdIndexer::is_empty()`, `capacity()`, `free_count()`, `total_slots()` | **DELETE** |
| `IdIndexer::reserve()`, `shrink_to_fit()` | **DELETE** |
| `IdIndexer::keys()` | **DELETE** |

## 32. vertex/vertex_table.rs — 4 methods (Category B)

| Method | Action |
|--------|--------|
| `ensure_capacity()` | **DELETE** |
| `get_by_key()` (private) | **DELETE** — only called internally but nothing calls it |
| `get_property()` | **DELETE** |
| `compact_with_ts()` | **DELETE** |

## 33. vertex/vertex_timestamp.rs — 1 method (Category B)

| Method | Action |
|--------|--------|
| `reserve()` | **DELETE** |

## 34. utils/name_indexer.rs — 7 methods (Category B)

| Method | Action |
|--------|--------|
| `get_name()` | **DELETE** |
| `len()`, `is_empty()` | **DELETE** |
| `names()`, `ids()` | **DELETE** |
| `next_id()`, `remove()` | **DELETE** |
| `memory_size()` | **DELETE** |

## Summary

| File | Count | Priority |
|------|-------|----------|
| vertex/column_store.rs | 41 | High — biggest chunk |
| vertex/encoding/alp.rs | 12 | Medium |
| index/vertex_index_manager.rs | 28 | High |
| index/edge_index_manager.rs | 22 | High |
| edge/property_table.rs | 21 | High |
| engine/persistence_coordinator.rs | 10 | Medium |
| vertex/encoding/fsst.rs | 8 | Low |
| edge/single_mutable_csr.rs | 8 | Low |
| vertex/id_indexer.rs | 9 | Low |
| vertex/encoding/dictionary.rs | 9 | Low |
| vertex/encoding/bitpacking.rs | 9 | Low |
| vertex/encoding/rle.rs | 6 | Low |
| utils/name_indexer.rs | 7 | Low |
| engine/snapshot_manager.rs | 6 | Low |
| index/index_data_manager.rs | 13 (trait) | Low — keep traits |
| All others | 1-4 each | Low |

Total unique items in this analysis: ~210 warning instances grouped into ~70 compiler warnings.

Script-assisted bulk editing recommended for high-count files (column_store.rs, vertex_index_manager.rs, edge_index_manager.rs, property_table.rs).
