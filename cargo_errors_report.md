# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 76
- **Total Warnings**: 7
- **Total Issues**: 83
- **Unique Error Patterns**: 34
- **Unique Warning Patterns**: 5
- **Files with Issues**: 16

## Error Statistics

**Total Errors**: 76

### Error Type Breakdown

- **error[E0599]**: 37 errors
- **error[E0061]**: 15 errors
- **error[E0432]**: 7 errors
- **error[E0560]**: 6 errors
- **error[E0609]**: 3 errors
- **error[E0277]**: 2 errors
- **error[E0412]**: 2 errors
- **error[E0422]**: 2 errors
- **error[E0308]**: 1 errors
- **error[E0282]**: 1 errors

### Files with Errors (Top 10)

- `src\transaction\context_test.rs`: 35 errors
- `src\transaction\manager_test.rs`: 7 errors
- `src\utils\error_convert.rs`: 7 errors
- `src\api\embedded\transaction.rs`: 7 errors
- `src\api\embedded\c_api\transaction.rs`: 5 errors
- `src\storage\operations\redb_writer.rs`: 4 errors
- `src\core\error\storage.rs`: 2 errors
- `src\storage\operations\rollback.rs`: 2 errors
- `src\api\embedded\session.rs`: 2 errors
- `src\storage\index\index_data_manager.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 7

### Warning Type Breakdown

- **warning**: 7 warnings

### Files with Warnings (Top 10)

- `src\storage\operations\rollback.rs`: 3 warnings
- `src\storage\metadata\redb_schema_manager.rs`: 2 warnings
- `src\storage\runtime_context.rs`: 1 warnings
- `src\transaction\context.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `savepoint_manager` found for reference `&'sess embedded::session::Session<S>` in the current scope

**Total Occurrences**: 37  
**Unique Files**: 6

#### `src\transaction\context_test.rs`: 18 occurrences

- Line 87: no variant or associated item named `Prepared` found for enum `transaction::types::TransactionState` in the current scope: variant or associated item not found in `TransactionState`
- Line 89: no variant or associated item named `Prepared` found for enum `transaction::types::TransactionState` in the current scope: variant or associated item not found in `TransactionState`
- Line 131: no variant or associated item named `Prepared` found for enum `transaction::types::TransactionState` in the current scope: variant or associated item not found in `TransactionState`
- ... 15 more occurrences in this file

#### `src\utils\error_convert.rs`: 7 occurrences

- Line 25: no variant or associated item named `TransactionNotPrepared` found for enum `transaction::types::TransactionError` in the current scope: variant or associated item not found in `TransactionError`
- Line 39: no variant or associated item named `SavepointFailed` found for enum `transaction::types::TransactionError` in the current scope: variant or associated item not found in `TransactionError`
- Line 42: no variant or associated item named `SavepointNotFound` found for enum `transaction::types::TransactionError` in the current scope: variant or associated item not found in `TransactionError`
- ... 4 more occurrences in this file

#### `src\api\embedded\transaction.rs`: 5 occurrences

- Line 274: no method named `savepoint_manager` found for reference `&'sess embedded::session::Session<S>` in the current scope
- Line 294: no method named `savepoint_manager` found for reference `&'sess embedded::session::Session<S>` in the current scope
- Line 313: no method named `savepoint_manager` found for reference `&'sess embedded::session::Session<S>` in the current scope
- ... 2 more occurrences in this file

#### `src\api\embedded\c_api\transaction.rs`: 3 occurrences

- Line 343: no method named `savepoint_manager` found for struct `embedded::session::Session` in the current scope
- Line 378: no method named `savepoint_manager` found for struct `embedded::session::Session` in the current scope
- Line 424: no method named `savepoint_manager` found for struct `embedded::session::Session` in the current scope

#### `src\storage\operations\redb_writer.rs`: 2 occurrences

- Line 63: no method named `add_operation_log` found for reference `&std::sync::Arc<TransactionContext>` in the current scope
- Line 69: no method named `record_table_modification` found for reference `&std::sync::Arc<TransactionContext>` in the current scope: method not found in `&Arc<TransactionContext>`

#### `src\transaction\manager_test.rs`: 2 occurrences

- Line 247: no variant or associated item named `WriteTransactionConflict` found for enum `transaction::types::TransactionError` in the current scope: variant or associated item not found in `TransactionError`
- Line 570: no method named `with_two_phase_commit` found for struct `transaction::types::TransactionOptions` in the current scope: method not found in `TransactionOptions`

### error[E0061]: this function takes 4 arguments but 5 arguments were supplied

**Total Occurrences**: 15  
**Unique Files**: 1

#### `src\transaction\context_test.rs`: 15 occurrences

- Line 35: this function takes 4 arguments but 5 arguments were supplied
- Line 77: this function takes 4 arguments but 5 arguments were supplied
- Line 114: this function takes 4 arguments but 5 arguments were supplied
- ... 12 more occurrences in this file

### error[E0432]: unresolved import `crate::transaction::SavepointId`: no `SavepointId` in `transaction`

**Total Occurrences**: 7  
**Unique Files**: 6

#### `src\api\embedded\c_api\transaction.rs`: 2 occurrences

- Line 377: unresolved import `crate::transaction::SavepointId`: no `SavepointId` in `transaction`
- Line 423: unresolved import `crate::transaction::SavepointId`: no `SavepointId` in `transaction`

#### `src\api\server\session\network_session.rs`: 1 occurrences

- Line 9: unresolved import `crate::transaction::SavepointId`: no `SavepointId` in `transaction`

#### `src\transaction\context_test.rs`: 1 occurrences

- Line 11: unresolved import `crate::transaction::types::OperationLog`: no `OperationLog` in `transaction::types`

#### `src\storage\operations\redb_writer.rs`: 1 occurrences

- Line 5: unresolved import `crate::transaction::OperationLog`: no `OperationLog` in `transaction`

#### `src\api\embedded\transaction.rs`: 1 occurrences

- Line 10: unresolved imports `crate::transaction::SavepointId`, `crate::transaction::SavepointInfo`: no `SavepointId` in `transaction`, no `SavepointInfo` in `transaction`

#### `src\storage\operations\rollback.rs`: 1 occurrences

- Line 8: unresolved import `crate::transaction::OperationLog`: no `OperationLog` in `transaction`

### error[E0560]: struct `transaction::types::TransactionManagerConfig` has no field named `auto_cleanup`: `transaction::types::TransactionManagerConfig` does not have this field

**Total Occurrences**: 6  
**Unique Files**: 4

#### `src\transaction\manager_test.rs`: 3 occurrences

- Line 24: struct `transaction::types::TransactionManagerConfig` has no field named `auto_cleanup`: `transaction::types::TransactionManagerConfig` does not have this field
- Line 427: struct `transaction::types::TransactionManagerConfig` has no field named `auto_cleanup`: `transaction::types::TransactionManagerConfig` does not have this field
- Line 541: struct `transaction::types::TransactionManagerConfig` has no field named `auto_cleanup`: `transaction::types::TransactionManagerConfig` does not have this field

#### `src\api\server\http\handlers\transaction.rs`: 1 occurrences

- Line 39: struct `transaction::types::TransactionOptions` has no field named `two_phase_commit`: `transaction::types::TransactionOptions` does not have this field

#### `src\api\embedded\transaction.rs`: 1 occurrences

- Line 95: struct `transaction::types::TransactionOptions` has no field named `two_phase_commit`: `transaction::types::TransactionOptions` does not have this field

#### `src\api\server\graph_service.rs`: 1 occurrences

- Line 445: struct `config::AuthConfig` has no field named `force_change_default_default_password`: unknown field

### error[E0609]: no field `two_phase_commit` on type `TransactionContext`: unknown field

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\transaction\manager_test.rs`: 2 occurrences

- Line 39: no field `auto_cleanup` on type `&transaction::types::TransactionManagerConfig`: unknown field
- Line 581: no field `two_phase_commit` on type `std::sync::Arc<TransactionContext>`: unknown field

#### `src\transaction\context_test.rs`: 1 occurrences

- Line 47: no field `two_phase_commit` on type `TransactionContext`: unknown field

### error[E0277]: the trait bound `core::error::storage::StorageError: Decode<()>` is not satisfied: the trait `Decode<()>` is not implemented for `core::error::storage::StorageError`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\storage\operations\rollback.rs`: 1 occurrences

- Line 104: the trait bound `core::error::storage::StorageError: Decode<()>` is not satisfied: the trait `Decode<()>` is not implemented for `core::error::storage::StorageError`

#### `src\storage\index\index_data_manager.rs`: 1 occurrences

- Line 105: the trait bound `core::error::storage::StorageError: Decode<()>` is not satisfied: the trait `Decode<()>` is not implemented for `core::error::storage::StorageError`

### error[E0412]: cannot find type `Error` in crate `bincode`: not found in `bincode`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\core\error\storage.rs`: 2 occurrences

- Line 83: cannot find type `Error` in crate `bincode`: not found in `bincode`
- Line 84: cannot find type `Error` in crate `bincode`: not found in `bincode`

### error[E0422]: cannot find struct, variant or union type `QueryRequest` in this scope

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\api\embedded\session.rs`: 2 occurrences

- Line 118: cannot find struct, variant or union type `QueryRequest` in this scope
- Line 144: cannot find struct, variant or union type `QueryRequest` in this scope

### error[E0308]: mismatched types: expected `Result<Vec<u8>, StorageError>`, found `Result<Vec<u8>, EncodeError>`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\index\index_data_manager.rs`: 1 occurrences

- Line 99: mismatched types: expected `Result<Vec<u8>, StorageError>`, found `Result<Vec<u8>, EncodeError>`

### error[E0282]: type annotations needed

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\operations\redb_writer.rs`: 1 occurrences

- Line 284: type annotations needed

## Detailed Warning Categorization

### warning: unused import: `std::sync::Arc`

**Total Occurrences**: 7  
**Unique Files**: 4

#### `src\storage\operations\rollback.rs`: 3 occurrences

- Line 27: function cannot return without recursing: cannot return without recursing
- Line 31: function cannot return without recursing: cannot return without recursing
- Line 43: function cannot return without recursing: cannot return without recursing

#### `src\storage\metadata\redb_schema_manager.rs`: 2 occurrences

- Line 5: unused imports: `EDGE_TYPE_NAME_INDEX_TABLE` and `TAG_NAME_INDEX_TABLE`
- Line 143: unused variable: `key`: help: if this is intentional, prefix it with an underscore: `_key`

#### `src\transaction\context.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\storage\runtime_context.rs`: 1 occurrences

- Line 10: unused import: `crate::storage::StorageClient`

