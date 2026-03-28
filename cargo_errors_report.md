# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 8
- **Total Warnings**: 8
- **Total Issues**: 16
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 5
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 8

### Error Type Breakdown

- **error[E0599]**: 7 errors
- **error**: 1 errors

### Files with Errors (Top 10)

- `tests\integration_transaction.rs`: 7 errors
- `src\query\optimizer\stats\feedback\collector.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 8

### Warning Type Breakdown

- **warning**: 8 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\strategy\expression_precomputation.rs`: 4 warnings
- `src\query\optimizer\stats\feedback\collector.rs`: 1 warnings
- `src\query\optimizer\analysis\batch.rs`: 1 warnings
- `src\query\optimizer\builder.rs`: 1 warnings
- `src\query\optimizer\context.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `set_rollback_executor_factory` found for struct `std::sync::Arc<graphdb::transaction::TransactionManager>` in the current scope: method not found in `std::sync::Arc<graphdb::transaction::TransactionManager>`

**Total Occurrences**: 7  
**Unique Files**: 1

#### `tests\integration_transaction.rs`: 7 occurrences

- Line 168: no method named `set_rollback_executor_factory` found for struct `std::sync::Arc<graphdb::transaction::TransactionManager>` in the current scope: method not found in `std::sync::Arc<graphdb::transaction::TransactionManager>`
- Line 247: no method named `set_rollback_executor_factory` found for struct `std::sync::Arc<graphdb::transaction::TransactionManager>` in the current scope: method not found in `std::sync::Arc<graphdb::transaction::TransactionManager>`
- Line 557: no method named `set_rollback_executor_factory` found for struct `std::sync::Arc<graphdb::transaction::TransactionManager>` in the current scope: method not found in `std::sync::Arc<graphdb::transaction::TransactionManager>`
- ... 4 more occurrences in this file

### error: this comparison involving the minimum or maximum element for this type contains a case that is always true or always false

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\stats\feedback\collector.rs`: 1 occurrences

- Line 295: this comparison involving the minimum or maximum element for this type contains a case that is always true or always false

## Detailed Warning Categorization

### warning: comparison is useless due to type limits

**Total Occurrences**: 8  
**Unique Files**: 5

#### `src\query\optimizer\strategy\expression_precomputation.rs`: 4 occurrences

- Line 305: this `map_or` can be simplified
- Line 308: this `map_or` can be simplified
- Line 326: this `map_or` can be simplified
- ... 1 more occurrences in this file

#### `src\query\optimizer\stats\feedback\collector.rs`: 1 occurrences

- Line 295: comparison is useless due to type limits

#### `src\query\optimizer\builder.rs`: 1 occurrences

- Line 114: using `clone` on type `CostModelConfig` which implements the `Copy` trait: help: try removing the `clone` call: `config`

#### `src\query\optimizer\context.rs`: 1 occurrences

- Line 131: using `clone` on type `CostModelConfig` which implements the `Copy` trait: help: try dereferencing it: `*engine.cost_config()`

#### `src\query\optimizer\analysis\batch.rs`: 1 occurrences

- Line 508: use of `default` to create a unit struct

