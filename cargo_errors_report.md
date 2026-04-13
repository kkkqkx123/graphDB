# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 47
- **Total Warnings**: 31
- **Total Issues**: 78
- **Unique Error Patterns**: 13
- **Unique Warning Patterns**: 24
- **Files with Issues**: 25

## Error Statistics

**Total Errors**: 47

### Error Type Breakdown

- **error[E0599]**: 25 errors
- **error[E0282]**: 10 errors
- **error[E0433]**: 9 errors
- **error[E0432]**: 3 errors

### Files with Errors (Top 10)

- `tests\common\sync_helpers.rs`: 20 errors
- `tests\common\test_scenario.rs`: 9 errors
- `src\query\executor\data_processing\set_operations\base.rs`: 7 errors
- `src\transaction\manager_test.rs`: 6 errors
- `tests\common\storage_helpers.rs`: 2 errors
- `src\transaction\manager.rs`: 2 errors
- `tests\common\mod.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 31

### Warning Type Breakdown

- **warning**: 31 warnings

### Files with Warnings (Top 10)

- `src\sync\coordinator\coordinator.rs`: 10 warnings
- `src\utils\output\manager.rs`: 3 warnings
- `src\query\planning\fulltext_planner.rs`: 2 warnings
- `src\query\validator\statements\insert_vertices_validator.rs`: 1 warnings
- `src\query\planning\vector_planner.rs`: 1 warnings
- `src\storage\extend\fulltext_storage.rs`: 1 warnings
- `src\utils\output\config.rs`: 1 warnings
- `src\sync\vector_transaction_buffer.rs`: 1 warnings
- `crates\vector-client\src\engine\qdrant\mod.rs`: 1 warnings
- `src\utils\output\json.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `create_space` found for struct `graphdb::storage::RedbStorage` in the current scope

**Total Occurrences**: 25  
**Unique Files**: 3

#### `tests\common\sync_helpers.rs`: 17 occurrences

- Line 101: no method named `create_space` found for struct `graphdb::storage::RedbStorage` in the current scope
- Line 120: no method named `create_tag` found for struct `graphdb::storage::RedbStorage` in the current scope
- Line 123: no method named `get_space_id` found for struct `graphdb::storage::RedbStorage` in the current scope
- ... 14 more occurrences in this file

#### `src\transaction\manager_test.rs`: 6 occurrences

- Line 136: no method named `abort_transaction` found for struct `transaction::manager::TransactionManager` in the current scope
- Line 194: no method named `abort_transaction` found for struct `transaction::manager::TransactionManager` in the current scope
- Line 231: no method named `abort_transaction` found for struct `transaction::manager::TransactionManager` in the current scope
- ... 3 more occurrences in this file

#### `src\transaction\manager.rs`: 2 occurrences

- Line 784: no method named `abort_transaction` found for struct `transaction::manager::TransactionManager` in the current scope
- Line 869: no method named `abort_transaction` found for struct `transaction::manager::TransactionManager` in the current scope

### error[E0282]: type annotations needed: cannot infer type

**Total Occurrences**: 10  
**Unique Files**: 2

#### `tests\common\test_scenario.rs`: 8 occurrences

- Line 368: type annotations needed: cannot infer type
- Line 390: type annotations needed: cannot infer type
- Line 452: type annotations needed
- ... 5 more occurrences in this file

#### `tests\common\sync_helpers.rs`: 2 occurrences

- Line 258: type annotations needed: cannot infer type
- Line 258: type annotations needed: cannot infer type

### error[E0433]: failed to resolve: could not find `redb_storage` in `storage`: could not find `redb_storage` in `storage`

**Total Occurrences**: 9  
**Unique Files**: 2

#### `src\query\executor\data_processing\set_operations\base.rs`: 7 occurrences

- Line 259: failed to resolve: could not find `redb_storage` in `storage`: could not find `redb_storage` in `storage`
- Line 260: failed to resolve: could not find `redb_storage` in `storage`: could not find `redb_storage` in `storage`
- Line 263: failed to resolve: could not find `redb_storage` in `storage`: could not find `redb_storage` in `storage`
- ... 4 more occurrences in this file

#### `tests\common\storage_helpers.rs`: 2 occurrences

- Line 54: failed to resolve: could not find `redb_storage` in `storage`: could not find `redb_storage` in `storage`
- Line 55: failed to resolve: could not find `redb_storage` in `storage`: could not find `redb_storage` in `storage`

### error[E0432]: unresolved import `graphdb::storage::redb_storage`: could not find `redb_storage` in `storage`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `tests\common\mod.rs`: 1 occurrences

- Line 22: unresolved import `graphdb::storage::redb_storage`: could not find `redb_storage` in `storage`

#### `tests\common\sync_helpers.rs`: 1 occurrences

- Line 9: unresolved import `graphdb::storage::storage_client`: could not find `storage_client` in `storage`

#### `tests\common\test_scenario.rs`: 1 occurrences

- Line 9: unresolved import `graphdb::storage::redb_storage`: could not find `redb_storage` in `storage`

## Detailed Warning Categorization

### warning: unused variable: `id`: help: try ignoring the field: `id: _`

**Total Occurrences**: 31  
**Unique Files**: 19

#### `src\sync\coordinator\coordinator.rs`: 10 occurrences

- Line 567: unused variable: `id`: help: try ignoring the field: `id: _`
- Line 569: unused variable: `payload`: help: try ignoring the field: `payload: _`
- Line 573: unused variable: `text`: help: if this is intentional, prefix it with an underscore: `_text`
- ... 7 more occurrences in this file

#### `src\utils\output\manager.rs`: 3 occurrences

- Line 3: unused import: `self`
- Line 20: this `impl` can be derived
- Line 127: this `repeat().take()` can be written more concisely: help: consider using `repeat_n()` instead: `std::iter::repeat_n(char, length)`

#### `src\query\planning\fulltext_planner.rs`: 2 occurrences

- Line 533: unused imports: `FulltextMatchCondition` and `FulltextYieldClause`
- Line 33: field `metadata_context` is never read

#### `src\query\planning\vector_planner.rs`: 1 occurrences

- Line 590: unused imports: `VectorQueryExpr` and `VectorQueryType`

#### `src\utils\output\config.rs`: 1 occurrences

- Line 14: this `impl` can be derived

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 670: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\utils\output\stream.rs`: 1 occurrences

- Line 38: field `file` is never read

#### `src\query\validator\statements\insert_vertices_validator.rs`: 1 occurrences

- Line 212: this `if let` can be collapsed into the outer `if let`

#### `src\utils\output\json.rs`: 1 occurrences

- Line 7: unused import: `OutputError`

#### `src\storage\extend\fulltext_storage.rs`: 1 occurrences

- Line 163: this `if let` can be collapsed into the outer `if let`

#### `src\utils\output\table.rs`: 1 occurrences

- Line 165: function `print_table_slices` is never used

#### `crates\vector-client\src\engine\qdrant\mod.rs`: 1 occurrences

- Line 24: unused import: `convert_index_type`

#### `src\query\executor\factory\builders\fulltext_search_builder.rs`: 1 occurrences

- Line 27: unused import: `crate::search::manager::FulltextIndexManager`

#### `src\sync\manager.rs`: 1 occurrences

- Line 311: unused variable: `vector_coord`: help: if this is intentional, prefix it with an underscore: `_vector_coord`

#### `tests\integration_transaction.rs`: 1 occurrences

- Line 281: variable does not need to be mutable

#### `tests\common\sync_helpers.rs`: 1 occurrences

- Line 14: unused imports: `TransactionManagerConfig`, `TransactionManager`, and `TransactionOptions`

#### `crates\vector-client\src\engine\qdrant\config.rs`: 1 occurrences

- Line 18: function `convert_index_type` is never used

#### `src\transaction\index_buffer.rs`: 1 occurrences

- Line 100: unused import: `crate::sync::coordinator::ChangeType`

#### `src\sync\vector_transaction_buffer.rs`: 1 occurrences

- Line 154: unused variable: `location`: help: if this is intentional, prefix it with an underscore: `_location`

