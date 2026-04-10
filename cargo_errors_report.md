# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 61
- **Total Warnings**: 13
- **Total Issues**: 74
- **Unique Error Patterns**: 34
- **Unique Warning Patterns**: 12
- **Files with Issues**: 17

## Error Statistics

**Total Errors**: 61

### Error Type Breakdown

- **error[E0599]**: 14 errors
- **error[E0282]**: 13 errors
- **error[E0433]**: 12 errors
- **error[E0432]**: 7 errors
- **error[E0382]**: 3 errors
- **error[E0425]**: 2 errors
- **error[E0616]**: 2 errors
- **error[E0063]**: 2 errors
- **error[E0277]**: 2 errors
- **error[E0603]**: 1 errors
- **error[E0592]**: 1 errors
- **error[E0252]**: 1 errors
- **error**: 1 errors

### Files with Errors (Top 10)

- `src\sync\manager.rs`: 24 errors
- `src\transaction\index_buffer.rs`: 8 errors
- `src\transaction\sync_handle.rs`: 7 errors
- `src\sync\batch.rs`: 5 errors
- `src\transaction\manager.rs`: 5 errors
- `src\api\mod.rs`: 4 errors
- `src\transaction\context.rs`: 3 errors
- `src\sync\vector_sync.rs`: 3 errors
- `src\sync\task.rs`: 1 errors
- `src\query\validator\statements\insert_vertices_validator.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 13

### Warning Type Breakdown

- **warning**: 13 warnings

### Files with Warnings (Top 10)

- `crates\vector-client\src\embedding\service.rs`: 3 warnings
- `src\api\mod.rs`: 1 warnings
- `build.rs`: 1 warnings
- `crates\inversearch\src\config\validator.rs`: 1 warnings
- `src\query\planning\statements\dml\insert_planner.rs`: 1 warnings
- `src\transaction\context.rs`: 1 warnings
- `src\query\executor\expression\functions\builtin\aggregate.rs`: 1 warnings
- `src\storage\event_storage.rs`: 1 warnings
- `src\transaction\index_buffer.rs`: 1 warnings
- `src\query\executor\result_processing\agg_function_manager.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `try_take` found for struct `std::sync::Arc<AsyncQueue<SyncTask>>` in the current scope: method not found in `std::sync::Arc<AsyncQueue<SyncTask>>`

**Total Occurrences**: 14  
**Unique Files**: 5

#### `src\sync\manager.rs`: 8 occurrences

- Line 425: no method named `upsert_batch` found for reference `&std::sync::Arc<VectorSyncCoordinator>` in the current scope
- Line 447: no method named `delete_batch` found for reference `&std::sync::Arc<VectorSyncCoordinator>` in the current scope: method not found in `&std::sync::Arc<VectorSyncCoordinator>`
- Line 659: no method named `batch_sync` found for struct `std::sync::Arc<FulltextCoordinator>` in the current scope: method not found in `std::sync::Arc<FulltextCoordinator>`
- ... 5 more occurrences in this file

#### `src\transaction\sync_handle.rs`: 2 occurrences

- Line 133: the method `clone` exists for enum `std::option::Option<tokio::sync::oneshot::Receiver<std::result::Result<(), SyncError>>>`, but its trait bounds were not satisfied: method cannot be called due to unsatisfied trait bounds
- Line 123: the method `clone` exists for enum `std::option::Option<tokio::sync::oneshot::Receiver<std::result::Result<(), SyncError>>>`, but its trait bounds were not satisfied: method cannot be called due to unsatisfied trait bounds

#### `src\api\mod.rs`: 2 occurrences

- Line 94: no method named `expect` found for opaque type `impl futures::Future<Output = std::result::Result<VectorManager, VectorClientError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<VectorManager, VectorClientError>>`
- Line 117: no method named `expect` found for opaque type `impl futures::Future<Output = std::result::Result<VectorManager, VectorClientError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<VectorManager, VectorClientError>>`

#### `src\sync\batch.rs`: 1 occurrences

- Line 327: no method named `try_take` found for struct `std::sync::Arc<AsyncQueue<SyncTask>>` in the current scope: method not found in `std::sync::Arc<AsyncQueue<SyncTask>>`

#### `src\transaction\index_buffer.rs`: 1 occurrences

- Line 45: no variant or associated item named `BufferFull` found for enum `BufferError` in the current scope: variant or associated item not found in `BufferError`

### error[E0282]: type annotations needed: cannot infer type

**Total Occurrences**: 13  
**Unique Files**: 4

#### `src\sync\manager.rs`: 6 occurrences

- Line 579: type annotations needed: cannot infer type
- Line 615: type annotations needed: cannot infer type
- Line 665: type annotations needed: cannot infer type
- ... 3 more occurrences in this file

#### `src\transaction\index_buffer.rs`: 5 occurrences

- Line 68: type annotations needed
- Line 79: type annotations needed
- Line 91: type annotations needed
- ... 2 more occurrences in this file

#### `src\transaction\sync_handle.rs`: 1 occurrences

- Line 135: type annotations needed: cannot infer type

#### `src\sync\batch.rs`: 1 occurrences

- Line 329: type annotations needed: cannot infer type

### error[E0433]: failed to resolve: could not find `SyncFailurePolicy` in `search`: could not find `SyncFailurePolicy` in `search`

**Total Occurrences**: 12  
**Unique Files**: 5

#### `src\transaction\manager.rs`: 4 occurrences

- Line 220: failed to resolve: could not find `SyncFailurePolicy` in `search`: could not find `SyncFailurePolicy` in `search`
- Line 230: failed to resolve: could not find `SyncFailurePolicy` in `search`: could not find `SyncFailurePolicy` in `search`
- Line 284: failed to resolve: could not find `SyncFailurePolicy` in `search`: could not find `SyncFailurePolicy` in `search`
- ... 1 more occurrences in this file

#### `src\sync\manager.rs`: 4 occurrences

- Line 565: failed to resolve: could not find `pending_update` in `sync`: could not find `pending_update` in `sync`
- Line 655: failed to resolve: could not find `FulltextBatchContext` in `coordinator`: could not find `FulltextBatchContext` in `coordinator`
- Line 719: failed to resolve: could not find `pending_update` in `sync`: could not find `pending_update` in `sync`
- ... 1 more occurrences in this file

#### `src\transaction\sync_handle.rs`: 2 occurrences

- Line 4: failed to resolve: use of unresolved module or unlinked crate `crossbeam`: use of unresolved module or unlinked crate `crossbeam`
- Line 156: failed to resolve: could not find `SyncMode` in `search`: could not find `SyncMode` in `search`

#### `src\sync\batch.rs`: 1 occurrences

- Line 33: failed to resolve: could not find `SyncFailurePolicy` in `search`: could not find `SyncFailurePolicy` in `search`

#### `src\api\mod.rs`: 1 occurrences

- Line 106: failed to resolve: use of undeclared type `FulltextConfig`: use of undeclared type `FulltextConfig`

### error[E0432]: unresolved import `crate::vector`: unresolved import, help: a similar path exists: `core::vector`

**Total Occurrences**: 7  
**Unique Files**: 6

#### `src\transaction\index_buffer.rs`: 2 occurrences

- Line 2: unresolved import `crate::sync::pending_update`: could not find `pending_update` in `sync`
- Line 3: unresolved import `crate::sync::sync_handle`: could not find `sync_handle` in `sync`

#### `src\sync\manager.rs`: 1 occurrences

- Line 13: unresolved import `crate::vector`: unresolved import, help: a similar path exists: `core::vector`

#### `src\transaction\sync_handle.rs`: 1 occurrences

- Line 2: unresolved import `crate::search::ChangeType`: no `ChangeType` in `search`

#### `src\api\mod.rs`: 1 occurrences

- Line 72: unresolved import `crate::sync::SyncConfig`: no `SyncConfig` in `sync`

#### `src\transaction\context.rs`: 1 occurrences

- Line 16: unresolved import `crate::sync::pending_update`: could not find `pending_update` in `sync`

#### `src\sync\task.rs`: 1 occurrences

- Line 5: unresolved import `crate::vector`: unresolved import, help: a similar path exists: `core::vector`

### error[E0382]: borrow of moved value: `points`: value borrowed here after move

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\sync\vector_sync.rs`: 3 occurrences

- Line 297: borrow of moved value: `points`: value borrowed here after move
- Line 363: borrow of moved value: `points`: value borrowed here after move
- Line 504: borrow of moved value: `points`: value borrowed here after move

### error[E0425]: cannot find type `SyncMode` in module `crate::search`: not found in `crate::search`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\transaction\sync_handle.rs`: 1 occurrences

- Line 148: cannot find type `SyncMode` in module `crate::search`: not found in `crate::search`

#### `src\sync\batch.rs`: 1 occurrences

- Line 23: cannot find type `SyncFailurePolicy` in module `crate::search`: not found in `crate::search`

### error[E0277]: the size for values of type `str` cannot be known at compilation time: doesn't have a size known at compile-time

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\sync\manager.rs`: 2 occurrences

- Line 742: the size for values of type `str` cannot be known at compilation time: doesn't have a size known at compile-time
- Line 761: the size for values of type `str` cannot be known at compilation time: doesn't have a size known at compile-time

### error[E0616]: field `completion_tx` of struct `SyncHandle` is private: private field

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\sync\manager.rs`: 2 occurrences

- Line 549: field `completion_tx` of struct `SyncHandle` is private: private field
- Line 554: field `completion_tx` of struct `SyncHandle` is private: private field

### error[E0063]: missing field `two_phase_commit` in initializer of `transaction::types::TransactionConfig`: missing `two_phase_commit`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\transaction\manager.rs`: 1 occurrences

- Line 102: missing field `two_phase_commit` in initializer of `transaction::types::TransactionConfig`: missing `two_phase_commit`

#### `src\transaction\context.rs`: 1 occurrences

- Line 162: missing fields `pending_index_updates`, `sync_handle` and `two_phase_enabled` in initializer of `transaction::context::TransactionContext`: missing `pending_index_updates`, `sync_handle` and `two_phase_enabled`

### error[E0603]: struct import `VectorPoint` is private: private struct import

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\sync\manager.rs`: 1 occurrences

- Line 12: struct import `VectorPoint` is private: private struct import

### error: 7 positional arguments in format string, but there are 6 arguments

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\statements\insert_vertices_validator.rs`: 1 occurrences

- Line 213: 7 positional arguments in format string, but there are 6 arguments

### error[E0252]: the name `Arc` is defined multiple times: `Arc` reimported here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\transaction\context.rs`: 1 occurrences

- Line 19: the name `Arc` is defined multiple times: `Arc` reimported here

### error[E0592]: duplicate definitions with name `config`: duplicate definitions for `config`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\sync\batch.rs`: 1 occurrences

- Line 383: duplicate definitions with name `config`: duplicate definitions for `config`

## Detailed Warning Categorization

### warning: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`

**Total Occurrences**: 13  
**Unique Files**: 11

#### `crates\vector-client\src\embedding\service.rs`: 3 occurrences

- Line 40: field `usage` is never read
- Line 51: fields `prompt_tokens` and `total_tokens` are never read
- Line 118: method `add_auth` is never used

#### `src\query\executor\result_processing\agg_function_manager.rs`: 1 occurrences

- Line 469: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`

#### `src\api\mod.rs`: 1 occurrences

- Line 71: unused import: `crate::sync::batch::BatchConfig`

#### `src\transaction\index_buffer.rs`: 1 occurrences

- Line 8: unused import: `tokio::sync::RwLock`

#### `crates\inversearch\src\config\validator.rs`: 1 occurrences

- Line 16: unused import: `std::fmt`

#### `src\query\planning\statements\dml\insert_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::metadata::MetadataContext`

#### `src\storage\event_storage.rs`: 1 occurrences

- Line 165: unused variable: `old_vertex`: help: if this is intentional, prefix it with an underscore: `_old_vertex`

#### `src\sync\vector_sync.rs`: 1 occurrences

- Line 256: unused variable: `mode`: help: if this is intentional, prefix it with an underscore: `_mode`

#### `src\query\executor\expression\functions\builtin\aggregate.rs`: 1 occurrences

- Line 379: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`

#### `src\transaction\context.rs`: 1 occurrences

- Line 19: unused import: `std::sync::Arc`

#### `build.rs`: 1 occurrences

- Line 6: unused import: `std::path::PathBuf`

