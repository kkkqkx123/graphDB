# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 78
- **Total Warnings**: 0
- **Total Issues**: 78
- **Unique Error Patterns**: 14
- **Unique Warning Patterns**: 0
- **Files with Issues**: 25

## Error Statistics

**Total Errors**: 78

### Error Type Breakdown

- **error[E0433]**: 31 errors
- **error[E0603]**: 21 errors
- **error[E0599]**: 10 errors
- **error[E0412]**: 9 errors
- **error[E0277]**: 2 errors
- **error[E0061]**: 2 errors
- **error[E0282]**: 2 errors
- **error[E0252]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\data_processing\graph_traversal\impls.rs`: 8 errors
- `src\query\parser\ast\pattern_parser.rs`: 7 errors
- `src\query\parser\ast\utils.rs`: 6 errors
- `src\query\executor\data_processing\graph_traversal\expand.rs`: 5 errors
- `src\query\parser\statements\go.rs`: 4 errors
- `src\query\parser\ast\pattern.rs`: 4 errors
- `src\query\executor\data_processing\graph_traversal\factory.rs`: 4 errors
- `src\query\parser\ast\stmt_parser.rs`: 4 errors
- `src\query\executor\data_processing\loops.rs`: 4 errors
- `src\query\parser\parser\pattern_parser.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0433]: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

**Total Occurrences**: 31  
**Unique Files**: 11

#### `src\query\parser\ast\pattern_parser.rs`: 7 occurrences

- Line 93: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 95: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 160: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- ... 4 more occurrences in this file

#### `src\query\parser\ast\stmt_parser.rs`: 4 occurrences

- Line 572: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 586: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 588: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- ... 1 more occurrences in this file

#### `src\query\parser\statements\go.rs`: 4 occurrences

- Line 85: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 89: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 93: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- ... 1 more occurrences in this file

#### `src\query\parser\parser\pattern_parser.rs`: 3 occurrences

- Line 105: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 109: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 113: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

#### `src\query\parser\statements\match_stmt.rs`: 3 occurrences

- Line 106: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 110: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 114: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

#### `src\query\parser\parser\statement_parser.rs`: 2 occurrences

- Line 313: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 316: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

#### `src\query\parser\ast\tests.rs`: 2 occurrences

- Line 278: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 303: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

#### `src\query\parser\ast\pattern.rs`: 2 occurrences

- Line 342: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 367: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

#### `src\query\parser\statements\create.rs`: 2 occurrences

- Line 88: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 92: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 62: failed to resolve: use of unresolved module or unlinked crate `recursion_detector`: use of unresolved module or unlinked crate `recursion_detector`

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 347: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

### error[E0603]: enum import `EdgeDirection` is private: private enum import

**Total Occurrences**: 21  
**Unique Files**: 9

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 8 occurrences

- Line 9: enum import `EdgeDirection` is private: private enum import
- Line 21: enum import `EdgeDirection` is private: private enum import
- Line 35: enum import `EdgeDirection` is private: private enum import
- ... 5 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\factory.rs`: 4 occurrences

- Line 16: enum import `EdgeDirection` is private: private enum import
- Line 27: enum import `EdgeDirection` is private: private enum import
- Line 38: enum import `EdgeDirection` is private: private enum import
- ... 1 more occurrences in this file

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 246: enum import `EdgeDirection` is private: private enum import
- Line 256: enum import `EdgeDirection` is private: private enum import

#### `src\query\executor\data_processing\graph_traversal\traits.rs`: 2 occurrences

- Line 4: enum import `EdgeDirection` is private: private enum import
- Line 13: enum import `EdgeDirection` is private: private enum import

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 occurrences

- Line 7: enum import `EdgeDirection` is private: private enum import

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 7: enum import `EdgeDirection` is private: private enum import

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 5: enum import `EdgeDirection` is private: private enum import

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 1 occurrences

- Line 7: enum import `EdgeDirection` is private: private enum import

#### `src\query\executor\data_processing\graph_traversal\expand.rs`: 1 occurrences

- Line 7: enum import `EdgeDirection` is private: private enum import

### error[E0599]: no variant or associated item named `Out` found for enum `core::types::graph::EdgeDirection` in the current scope: variant or associated item not found in `EdgeDirection`

**Total Occurrences**: 10  
**Unique Files**: 5

#### `src\query\planner\plan\core\nodes\traversal_node.rs`: 2 occurrences

- Line 799: no variant or associated item named `Out` found for enum `core::types::graph::EdgeDirection` in the current scope: variant or associated item not found in `EdgeDirection`
- Line 802: no variant or associated item named `Out` found for enum `core::types::graph::EdgeDirection` in the current scope: variant or associated item not found in `EdgeDirection`

#### `src\query\executor\data_modification.rs`: 2 occurrences

- Line 243: no method named `get_storage` found for mutable reference `&mut DeleteExecutor<S>` in the current scope: method not found in `&mut DeleteExecutor<S>`
- Line 251: no method named `get_storage` found for mutable reference `&mut DeleteExecutor<S>` in the current scope: method not found in `&mut DeleteExecutor<S>`

#### `src\query\executor\data_processing\graph_traversal\expand.rs`: 2 occurrences

- Line 67: no method named `get_storage` found for reference `&ExpandExecutor<S>` in the current scope: method not found in `&ExpandExecutor<S>`
- Line 153: no method named `get_storage` found for reference `&ExpandExecutor<S>` in the current scope: method not found in `&ExpandExecutor<S>`

#### `src\query\planner\ngql\path_planner.rs`: 2 occurrences

- Line 64: no variant or associated item named `In` found for enum `core::types::graph::EdgeDirection` in the current scope: variant or associated item not found in `EdgeDirection`
- Line 66: no variant or associated item named `Out` found for enum `core::types::graph::EdgeDirection` in the current scope: variant or associated item not found in `EdgeDirection`

#### `src\query\executor\data_processing\loops.rs`: 2 occurrences

- Line 421: no method named `get_storage` found for struct `LoopExecutor` in the current scope: method not found in `LoopExecutor<S>`
- Line 520: no method named `get_storage` found for struct `LoopExecutor` in the current scope: method not found in `LoopExecutor<S>`

### error[E0412]: cannot find type `EdgeDirection` in this scope

**Total Occurrences**: 9  
**Unique Files**: 3

#### `src\query\parser\ast\utils.rs`: 5 occurrences

- Line 122: cannot find type `EdgeDirection` in this scope
- Line 301: cannot find type `EdgeDirection` in this scope
- Line 329: cannot find type `EdgeDirection` in this scope
- ... 2 more occurrences in this file

#### `src\query\parser\ast\stmt.rs`: 2 occurrences

- Line 83: cannot find type `EdgeDirection` in this scope
- Line 235: cannot find type `EdgeDirection` in this scope

#### `src\query\parser\ast\pattern.rs`: 2 occurrences

- Line 65: cannot find type `EdgeDirection` in this scope
- Line 75: cannot find type `EdgeDirection` in this scope

### error[E0277]: `recursion_detector::ExecutorSafetyValidator` doesn't implement `std::fmt::Debug`: `recursion_detector::ExecutorSafetyValidator` cannot be formatted using `{:?}`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 2 occurrences

- Line 35: `recursion_detector::ExecutorSafetyValidator` doesn't implement `std::fmt::Debug`: `recursion_detector::ExecutorSafetyValidator` cannot be formatted using `{:?}`
- Line 294: `?` couldn't convert the error to `core::error::QueryError`: the trait `From<core::error::DBError>` is not implemented for `core::error::QueryError`

### error[E0282]: type annotations needed for `std::sync::MutexGuard<'_, _>`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\data_processing\graph_traversal\expand.rs`: 2 occurrences

- Line 67: type annotations needed for `std::sync::MutexGuard<'_, _>`
- Line 153: type annotations needed for `std::sync::MutexGuard<'_, _>`

### error[E0061]: this method takes 1 argument but 2 arguments were supplied

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 273: this method takes 1 argument but 2 arguments were supplied

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 291: this method takes 1 argument but 2 arguments were supplied

### error[E0252]: the name `HasStorage` is defined multiple times: `HasStorage` reimported here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 8: the name `HasStorage` is defined multiple times: `HasStorage` reimported here

