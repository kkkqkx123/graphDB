# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 47
- **Total Warnings**: 4
- **Total Issues**: 51
- **Unique Error Patterns**: 8
- **Unique Warning Patterns**: 3
- **Files with Issues**: 12

## Error Statistics

**Total Errors**: 47

### Error Type Breakdown

- **error[E0599]**: 28 errors
- **error[E0308]**: 15 errors
- **error[E0282]**: 2 errors
- **error[E0277]**: 1 errors
- **error[E0061]**: 1 errors

### Files with Errors (Top 10)

- `src\transaction\manager_test.rs`: 19 errors
- `src\transaction\manager.rs`: 9 errors
- `src\sync\manager.rs`: 4 errors
- `src\api\server\graph_service.rs`: 3 errors
- `src\api\embedded\transaction.rs`: 2 errors
- `src\api\embedded\database.rs`: 2 errors
- `src\api\mod.rs`: 2 errors
- `src\api\server\http\handlers\transaction.rs`: 2 errors
- `src\api\embedded\session.rs`: 2 errors
- `src\transaction\mod.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 4

### Warning Type Breakdown

- **warning**: 4 warnings

### Files with Warnings (Top 10)

- `src\sync\batch\processor.rs`: 2 warnings
- `src\sync\manager.rs`: 1 warnings
- `src\sync\coordinator\coordinator.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `map_err` found for opaque type `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>`

**Total Occurrences**: 28  
**Unique Files**: 5

#### `src\transaction\manager_test.rs`: 16 occurrences

- Line 109: no method named `expect` found for opaque type `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>`
- Line 160: no method named `expect` found for opaque type `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>`
- Line 208: no method named `expect` found for opaque type `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>`
- ... 13 more occurrences in this file

#### `src\transaction\manager.rs`: 8 occurrences

- Line 683: no method named `expect` found for opaque type `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>`
- Line 730: no method named `expect` found for opaque type `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>`
- Line 755: no method named `expect` found for opaque type `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>`
- ... 5 more occurrences in this file

#### `src\transaction\mod.rs`: 2 occurrences

- Line 100: no method named `expect` found for opaque type `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>`
- Line 120: no method named `expect` found for opaque type `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>`

#### `src\api\embedded\transaction.rs`: 1 occurrences

- Line 244: no method named `map_err` found for opaque type `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>`

#### `src\api\embedded\session.rs`: 1 occurrences

- Line 458: no method named `map_err` found for opaque type `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>` in the current scope: method not found in `impl futures::Future<Output = std::result::Result<(), transaction::types::TransactionError>>`

### error[E0308]: mismatched types: expected future, found `Result<_, _>`

**Total Occurrences**: 15  
**Unique Files**: 7

#### `src\transaction\manager_test.rs`: 3 occurrences

- Line 183: mismatched types: expected future, found `Result<_, _>`
- Line 214: mismatched types: expected future, found `Result<_, _>`
- Line 361: mismatched types: expected future, found `Result<_, _>`

#### `src\api\server\graph_service.rs`: 3 occurrences

- Line 184: mismatched types: expected future, found `Result<_, _>`
- Line 448: mismatched types: expected future, found `Result<_, _>`
- Line 454: mismatched types: expected future, found `Result<_, _>`

#### `src\sync\manager.rs`: 2 occurrences

- Line 164: mismatched types: expected `sync::coordinator::types::ChangeType`, found `coordinator::fulltext::ChangeType`
- Line 183: mismatched types: expected `sync::coordinator::types::ChangeType`, found `coordinator::fulltext::ChangeType`

#### `src\api\server\http\handlers\transaction.rs`: 2 occurrences

- Line 92: mismatched types: expected future, found `Result<_, _>`
- Line 96: mismatched types: expected future, found `Result<_, _>`

#### `src\api\mod.rs`: 2 occurrences

- Line 92: mismatched types: expected `Arc<SyncCoordinator>`, found `Arc<FulltextCoordinator>`
- Line 116: mismatched types: expected `Arc<SyncCoordinator>`, found `Arc<FulltextCoordinator>`

#### `src\api\embedded\database.rs`: 2 occurrences

- Line 126: mismatched types: expected `Arc<SyncCoordinator>`, found `Arc<FulltextCoordinator>`
- Line 163: mismatched types: expected `Arc<SyncCoordinator>`, found `Arc<FulltextCoordinator>`

#### `src\transaction\manager.rs`: 1 occurrences

- Line 761: mismatched types: expected future, found `Result<_, _>`

### error[E0282]: type annotations needed

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\api\embedded\session.rs`: 1 occurrences

- Line 458: type annotations needed

#### `src\api\embedded\transaction.rs`: 1 occurrences

- Line 244: type annotations needed

### error[E0061]: this method takes 2 arguments but 1 argument was supplied

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\sync\manager.rs`: 1 occurrences

- Line 269: this method takes 2 arguments but 1 argument was supplied

### error[E0277]: `?` couldn't convert the error to `sync::manager::SyncError`: the trait `std::convert::From<sync::recovery::RecoveryError>` is not implemented for `sync::manager::SyncError`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\sync\manager.rs`: 1 occurrences

- Line 127: `?` couldn't convert the error to `sync::manager::SyncError`: the trait `std::convert::From<sync::recovery::RecoveryError>` is not implemented for `sync::manager::SyncError`

## Detailed Warning Categorization

### warning: unused import: `ExternalIndexClient`

**Total Occurrences**: 4  
**Unique Files**: 3

#### `src\sync\batch\processor.rs`: 2 occurrences

- Line 215: use of deprecated field `sync::batch::processor::TransactionBatchBuffer::processor`
- Line 228: use of deprecated field `sync::batch::processor::TransactionBatchBuffer::processor`

#### `src\sync\coordinator\coordinator.rs`: 1 occurrences

- Line 10: unused import: `ExternalIndexClient`

#### `src\sync\manager.rs`: 1 occurrences

- Line 96: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

