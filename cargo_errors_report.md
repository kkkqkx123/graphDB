# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 22
- **Total Warnings**: 38
- **Total Issues**: 60
- **Unique Error Patterns**: 10
- **Unique Warning Patterns**: 8
- **Files with Issues**: 37

## Error Statistics

**Total Errors**: 22

### Error Type Breakdown

- **error[E0599]**: 18 errors
- **error[E0223]**: 2 errors
- **error[E0277]**: 1 errors
- **error[E0164]**: 1 errors

### Files with Errors (Top 10)

- `tests\transaction\error_scenarios.rs`: 19 errors
- `src\transaction\manager_test.rs`: 2 errors
- `tests\transaction\concurrent.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 38

### Warning Type Breakdown

- **warning**: 38 warnings

### Files with Warnings (Top 10)

- `src\storage\engine\transaction.rs`: 5 warnings
- `src\storage\engine\edge.rs`: 5 warnings
- `src\transaction\update_transaction.rs`: 3 warnings
- `src\query\executor\graph_operations\graph_traversal\expand_all.rs`: 2 warnings
- `src\transaction\undo_log.rs`: 2 warnings
- `src\storage\engine\batch.rs`: 2 warnings
- `src\storage\engine\property_graph.rs`: 2 warnings
- `src\query\planning\statements\dql\composite_index_analyzer.rs`: 1 warnings
- `src\query\planning\plan\core\nodes\traversal\traversal_node.rs`: 1 warnings
- `src\core\error\storage.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no associated item named `TransactionNotFound` found for struct `graphdb::transaction::TransactionError` in the current scope: associated item not found in `graphdb::transaction::TransactionError`

**Total Occurrences**: 18  
**Unique Files**: 2

#### `tests\transaction\error_scenarios.rs`: 17 occurrences

- Line 28: no associated item named `TransactionNotFound` found for struct `graphdb::transaction::TransactionError` in the current scope: associated item not found in `graphdb::transaction::TransactionError`
- Line 34: no associated item named `TransactionNotFound` found for struct `graphdb::transaction::TransactionError` in the current scope: associated item not found in `graphdb::transaction::TransactionError`
- Line 40: no associated item named `TransactionNotFound` found for struct `graphdb::transaction::TransactionError` in the current scope: associated item not found in `graphdb::transaction::TransactionError`
- ... 14 more occurrences in this file

#### `tests\transaction\concurrent.rs`: 1 occurrences

- Line 101: no associated item named `WriteTransactionConflict` found for struct `graphdb::transaction::TransactionError` in the current scope: associated item not found in `graphdb::transaction::TransactionError`

### error[E0223]: ambiguous associated type

**Total Occurrences**: 2  
**Unique Files**: 1

#### `tests\transaction\error_scenarios.rs`: 2 occurrences

- Line 64: ambiguous associated type
- Line 80: ambiguous associated type

### error[E0277]: `transaction::context::TransactionContext` doesn't implement `std::fmt::Debug`: unsatisfied trait bound

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\transaction\manager_test.rs`: 1 occurrences

- Line 158: `transaction::context::TransactionContext` doesn't implement `std::fmt::Debug`: unsatisfied trait bound

### error[E0164]: expected tuple struct or tuple variant, found associated function `TransactionError::internal`: `fn` calls are not allowed in patterns

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\transaction\manager_test.rs`: 1 occurrences

- Line 506: expected tuple struct or tuple variant, found associated function `TransactionError::internal`: `fn` calls are not allowed in patterns

## Detailed Warning Categorization

### warning: this function has too many arguments (9/7)

**Total Occurrences**: 38  
**Unique Files**: 24

#### `src\storage\engine\edge.rs`: 5 occurrences

- Line 33: this function has too many arguments (8/7)
- Line 139: this function has too many arguments (9/7)
- Line 178: this function has too many arguments (8/7)
- ... 2 more occurrences in this file

#### `src\storage\engine\transaction.rs`: 5 occurrences

- Line 41: this function has too many arguments (9/7)
- Line 210: this function has too many arguments (9/7)
- Line 234: this function has too many arguments (9/7)
- ... 2 more occurrences in this file

#### `src\transaction\update_transaction.rs`: 3 occurrences

- Line 205: this function has too many arguments (9/7)
- Line 222: very complex type used. Consider factoring parts into `type` definitions
- Line 517: this function has too many arguments (9/7)

#### `src\storage\engine\batch.rs`: 2 occurrences

- Line 192: very complex type used. Consider factoring parts into `type` definitions
- Line 285: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\executor\graph_operations\graph_traversal\expand_all.rs`: 2 occurrences

- Line 71: this function has too many arguments (9/7)
- Line 104: this function has too many arguments (9/7)

#### `src\storage\engine\property_graph.rs`: 2 occurrences

- Line 410: this function has too many arguments (8/7)
- Line 481: this function has too many arguments (9/7)

#### `src\transaction\undo_log.rs`: 2 occurrences

- Line 645: this function has too many arguments (8/7)
- Line 681: this function has too many arguments (10/7)

#### `src\query\executor\graph_operations\graph_traversal\factory.rs`: 1 occurrences

- Line 36: this function has too many arguments (9/7)

#### `src\query\validator\helpers\schema_validator.rs`: 1 occurrences

- Line 671: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\executor\data_access\fulltext_search.rs`: 1 occurrences

- Line 78: this function has too many arguments (10/7)

#### `src\query\executor\base\manage_executor_enums.rs`: 1 occurrences

- Line 50: large size difference between variants

#### `src\query\planning\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 194: very complex type used. Consider factoring parts into `type` definitions

#### `src\storage\edge\edge_table.rs`: 1 occurrences

- Line 503: this function has too many arguments (8/7)

#### `src\query\planning\plan\core\nodes\management\manage_node_enums.rs`: 1 occurrences

- Line 55: large size difference between variants

#### `src\query\planning\statements\dql\composite_index_analyzer.rs`: 1 occurrences

- Line 180: large size difference between variants

#### `src\storage\edge\mutable_csr.rs`: 1 occurrences

- Line 775: this function has too many arguments (8/7)

#### `src\query\planning\plan\core\nodes\traversal\traversal_node.rs`: 1 occurrences

- Line 919: this function has too many arguments (11/7)

#### `src\storage\entity\edge_storage.rs`: 1 occurrences

- Line 907: this function has too many arguments (9/7)

#### `src\core\error\storage.rs`: 1 occurrences

- Line 152: constructor `storage_error` has the same name as the type

#### `src\storage\entity\vertex_storage.rs`: 1 occurrences

- Line 525: this function has too many arguments (8/7)

#### `src\storage\operations\rollback.rs`: 1 occurrences

- Line 294: this function has too many arguments (9/7)

#### `src\transaction\insert_transaction.rs`: 1 occurrences

- Line 89: this function has too many arguments (8/7)

#### `src\storage\params.rs`: 1 occurrences

- Line 159: this function has too many arguments (8/7)

#### `src\storage\page\mod.rs`: 1 occurrences

- Line 8: module has the same name as its containing module

