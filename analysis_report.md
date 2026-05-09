# Type Check Report

## Type Issues Summary

- **Total**: 13
- **⚠️** warning: 13
- **Categories**: 12
- **Files Affected**: 13

## Breakdown by Category

- **field `config` is never read**: 2 occurrence(s)
- **type alias `IndexKey` is never**: 1 occurrence(s)
- **function `get_system_huge_page_size` is never used**: 1 occurrence(s)
- **methods `reset` and `test` are**: 1 occurrence(s)
- **fields `page_directory` and `config` are**: 1 occurrence(s)
- **methods `nbr_start`, `nbr_ptr`, and `nbr_ptr_mut`**: 1 occurrence(s)
- **field `start` is never read**: 1 occurrence(s)
- **field `footer` is never read**: 1 occurrence(s)
- **field `page_writer` is never read**: 1 occurrence(s)
- **method `is_empty` is never used**: 1 occurrence(s)
- **field `path` is never read**: 1 occurrence(s)
- **method `remaining` is never used**: 1 occurrence(s)

## Details by File

### `src\storage\edge\edge_table.rs` (1 item(s))

- ⚠️ **warning** at line 40:5: field `config` is never read

### `src\storage\persistence\sstable.rs` (1 item(s))

- ⚠️ **warning** at line 349:5: field `footer` is never read

### `src\storage\index\secondary\key_codec\key_types.rs` (1 item(s))

- ⚠️ **warning** at line 32:10: type alias `IndexKey` is never used

### `src\storage\persistence\page_writer.rs` (1 item(s))

- ⚠️ **warning** at line 441:5: field `page_writer` is never read

### `src\storage\edge\csr_persistence.rs` (1 item(s))

- ⚠️ **warning** at line 81:5: field `path` is never read

### `src\transaction\version_manager.rs` (1 item(s))

- ⚠️ **warning** at line 67:8: methods `reset` and `test` are never used

### `src\transaction\wal\parser.rs` (1 item(s))

- ⚠️ **warning** at line 523:8: method `is_empty` is never used

### `src\storage\container\arena_allocator.rs` (1 item(s))

- ⚠️ **warning** at line 61:8: method `remaining` is never used

### `src\storage\page\page_manager.rs` (1 item(s))

- ⚠️ **warning** at line 61:5: fields `page_directory` and `config` are never read

### `src\storage\memory\huge_pages.rs` (1 item(s))

- ⚠️ **warning** at line 310:8: function `get_system_huge_page_size` is never used

### `src\storage\vertex\vertex_table.rs` (1 item(s))

- ⚠️ **warning** at line 44:5: field `config` is never read

### `src\storage\edge\mutable_csr.rs` (1 item(s))

- ⚠️ **warning** at line 193:8: methods `nbr_start`, `nbr_ptr`, and `nbr_ptr_mut` are never used

### `src\storage\page\flat_csr.rs` (1 item(s))

- ⚠️ **warning** at line 370:5: field `start` is never read

