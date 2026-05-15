# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 21
- **Total Warnings**: 4
- **Total Issues**: 25
- **Unique Error Patterns**: 8
- **Unique Warning Patterns**: 3
- **Files with Issues**: 12

## Error Statistics

**Total Errors**: 21

### Error Type Breakdown

- **error[E0425]**: 13 errors
- **error[E0599]**: 2 errors
- **error[E0277]**: 2 errors
- **error[E0507]**: 2 errors
- **error[E0433]**: 1 errors
- **error[E0382]**: 1 errors

### Files with Errors (Top 10)

- `src\storage\test_mock.rs`: 13 errors
- `src\query\executor\data_modification\update.rs`: 2 errors
- `src\api\embedded\transaction.rs`: 1 errors
- `src\query\executor\graph_operations\graph_traversal\algorithms\a_star.rs`: 1 errors
- `src\api\server\http\handlers\transaction.rs`: 1 errors
- `src\query\planning\statements\paths\shortest_path_planner.rs`: 1 errors
- `src\storage\engine\graph_storage\writer.rs`: 1 errors
- `src\query\executor\graph_operations\graph_traversal\algorithms\bfs_shortest.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 4

### Warning Type Breakdown

- **warning**: 4 warnings

### Files with Warnings (Top 10)

- `src\query\executor\utils\tag_filter.rs`: 1 warnings
- `src\core\npath.rs`: 1 warnings
- `src\query\executor\expression\functions\builtin\path.rs`: 1 warnings
- `src\query\executor\graph_operations\graph_traversal\algorithms\multi_shortest_path.rs`: 1 warnings

## Detailed Error Categorization

### error[E0425]: cannot find type `VertexId` in this scope

**Total Occurrences**: 13  
**Unique Files**: 2

#### `src\storage\test_mock.rs`: 12 occurrences

- Line 54: cannot find type `VertexId` in this scope
- Line 79: cannot find type `VertexId` in this scope
- Line 80: cannot find type `VertexId` in this scope
- ... 9 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\algorithms\a_star.rs`: 1 occurrences

- Line 363: cannot find value `vertex_id` in this scope: not found in this scope

### error[E0599]: no variant or associated item named `Immediate` found for enum `core::types::transaction_config::DurabilityLevel` in the current scope: variant or associated item not found in `core::types::transaction_config::DurabilityLevel`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\api\embedded\transaction.rs`: 1 occurrences

- Line 60: no variant or associated item named `Immediate` found for enum `core::types::transaction_config::DurabilityLevel` in the current scope: variant or associated item not found in `core::types::transaction_config::DurabilityLevel`

#### `src\api\server\http\handlers\transaction.rs`: 1 occurrences

- Line 45: no variant or associated item named `Immediate` found for enum `core::types::transaction_config::DurabilityLevel` in the current scope: variant or associated item not found in `core::types::transaction_config::DurabilityLevel`

### error[E0277]: the trait bound `core::value::value_def::Value: std::convert::From<&core::types::storage_ids::VertexId>` is not satisfied: unsatisfied trait bound

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\storage\engine\graph_storage\writer.rs`: 1 occurrences

- Line 111: the trait bound `core::value::value_def::Value: std::convert::From<&core::types::storage_ids::VertexId>` is not satisfied: unsatisfied trait bound

#### `src\query\planning\statements\paths\shortest_path_planner.rs`: 1 occurrences

- Line 215: the trait bound `core::value::value_def::Value: std::convert::From<&core::types::storage_ids::VertexId>` is not satisfied: unsatisfied trait bound

### error[E0507]: cannot move out of `update.src` which is behind a shared reference: move occurs because `update.src` has type `core::value::value_def::Value`, which does not implement the `Copy` trait

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\data_modification\update.rs`: 2 occurrences

- Line 249: cannot move out of `update.src` which is behind a shared reference: move occurs because `update.src` has type `core::value::value_def::Value`, which does not implement the `Copy` trait
- Line 250: cannot move out of `update.dst` which is behind a shared reference: move occurs because `update.dst` has type `core::value::value_def::Value`, which does not implement the `Copy` trait

### error[E0433]: failed to resolve: use of undeclared type `VertexId`: use of undeclared type `VertexId`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\test_mock.rs`: 1 occurrences

- Line 122: failed to resolve: use of undeclared type `VertexId`: use of undeclared type `VertexId`

### error[E0382]: borrow of moved value: `new_vids`: value borrowed here after move

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\graph_operations\graph_traversal\algorithms\bfs_shortest.rs`: 1 occurrences

- Line 236: borrow of moved value: `new_vids`: value borrowed here after move

## Detailed Warning Categorization

### warning: unused import: `Value`

**Total Occurrences**: 4  
**Unique Files**: 4

#### `src\query\executor\graph_operations\graph_traversal\algorithms\multi_shortest_path.rs`: 1 occurrences

- Line 530: unused import: `Value`

#### `src\query\executor\expression\functions\builtin\path.rs`: 1 occurrences

- Line 6: unused import: `crate::core::types::VertexId`

#### `src\query\executor\utils\tag_filter.rs`: 1 occurrences

- Line 5: unused import: `crate::core::types::VertexId`

#### `src\core\npath.rs`: 1 occurrences

- Line 467: unused import: `crate::core::Value`

