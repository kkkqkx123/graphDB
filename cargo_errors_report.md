# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 10
- **Total Warnings**: 13
- **Total Issues**: 23
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 10
- **Files with Issues**: 13

## Error Statistics

**Total Errors**: 10

### Error Type Breakdown

- **error[E0061]**: 10 errors

### Files with Errors (Top 10)

- `src\storage\engine\property_graph_tests.rs`: 10 errors

## Warning Statistics

**Total Warnings**: 13

### Warning Type Breakdown

- **warning**: 13 warnings

### Files with Warnings (Top 10)

- `src\transaction\update_transaction.rs`: 2 warnings
- `src\query\planning\plan\core\nodes\management\manage_node_enums.rs`: 1 warnings
- `src\core\error\storage.rs`: 1 warnings
- `src\query\executor\graph_operations\graph_traversal\factory.rs`: 1 warnings
- `src\transaction\insert_transaction.rs`: 1 warnings
- `src\api\server\client\transaction_context.rs`: 1 warnings
- `src\query\executor\base\manage_executor_enums.rs`: 1 warnings
- `src\storage\engine\graph_storage.rs`: 1 warnings
- `src\query\validator\helpers\schema_validator.rs`: 1 warnings
- `src\query\planning\statements\clauses\return_clause_planner.rs`: 1 warnings

## Detailed Error Categorization

### error[E0061]: this method takes 1 argument but 7 arguments were supplied

**Total Occurrences**: 10  
**Unique Files**: 1

#### `src\storage\engine\property_graph_tests.rs`: 10 occurrences

- Line 81: this method takes 1 argument but 7 arguments were supplied
- Line 216: this method takes 1 argument but 7 arguments were supplied
- Line 227: this method takes 1 argument but 7 arguments were supplied
- ... 7 more occurrences in this file

## Detailed Warning Categorization

### warning: very complex type used. Consider factoring parts into `type` definitions

**Total Occurrences**: 13  
**Unique Files**: 12

#### `src\transaction\update_transaction.rs`: 2 occurrences

- Line 241: very complex type used. Consider factoring parts into `type` definitions
- Line 533: this function has too many arguments (9/7)

#### `src\query\planning\plan\core\nodes\management\manage_node_enums.rs`: 1 occurrences

- Line 55: large size difference between variants: the entire enum is at least 312 bytes

#### `src\storage\page\mod.rs`: 1 occurrences

- Line 8: module has the same name as its containing module

#### `src\query\planning\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 194: very complex type used. Consider factoring parts into `type` definitions

#### `src\storage\engine\graph_storage.rs`: 1 occurrences

- Line 27: unused import: `PropertyGraphUpdateEdgePropertyParams`

#### `src\api\server\client\transaction_context.rs`: 1 occurrences

- Line 15: you should consider adding a `Default` implementation for `TransactionContext`

#### `src\core\error\storage.rs`: 1 occurrences

- Line 152: constructor `storage_error` has the same name as the type

#### `src\query\executor\graph_operations\graph_traversal\factory.rs`: 1 occurrences

- Line 36: this function has too many arguments (9/7)

#### `src\query\planning\statements\dql\composite_index_analyzer.rs`: 1 occurrences

- Line 180: large size difference between variants: the entire enum is at least 448 bytes

#### `src\query\validator\helpers\schema_validator.rs`: 1 occurrences

- Line 671: very complex type used. Consider factoring parts into `type` definitions

#### `src\transaction\insert_transaction.rs`: 1 occurrences

- Line 89: this function has too many arguments (8/7)

#### `src\query\executor\base\manage_executor_enums.rs`: 1 occurrences

- Line 50: large size difference between variants: the entire enum is at least 544 bytes

