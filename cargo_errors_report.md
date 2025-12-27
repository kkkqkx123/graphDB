# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 115
- **Total Warnings**: 22
- **Total Issues**: 137
- **Unique Error Patterns**: 18
- **Unique Warning Patterns**: 22
- **Files with Issues**: 66

## Error Statistics

**Total Errors**: 115

### Error Type Breakdown

- **error[E0432]**: 35 errors
- **error[E0433]**: 31 errors
- **error[E0603]**: 21 errors
- **error[E0405]**: 14 errors
- **error[E0412]**: 9 errors
- **error[E0046]**: 5 errors

### Files with Errors (Top 10)

- `src\query\executor\traits.rs`: 9 errors
- `src\query\executor\data_processing\graph_traversal\impls.rs`: 8 errors
- `src\query\parser\ast\pattern_parser.rs`: 7 errors
- `src\query\parser\ast\utils.rs`: 6 errors
- `src\query\executor\data_processing\loops.rs`: 5 errors
- `src\query\executor\result_processing\projection.rs`: 4 errors
- `src\query\parser\ast\pattern.rs`: 4 errors
- `src\query\parser\statements\go.rs`: 4 errors
- `src\query\parser\ast\stmt_parser.rs`: 4 errors
- `src\query\executor\data_processing\graph_traversal\factory.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 22

### Warning Type Breakdown

- **warning**: 22 warnings

### Files with Warnings (Top 10)

- `src\query\parser\expressions\expression_converter.rs`: 2 warnings
- `src\core\context\mod.rs`: 2 warnings
- `src\query\validator\strategies\aggregate_strategy.rs`: 1 warnings
- `src\query\executor\data_processing\join\cross_join.rs`: 1 warnings
- `src\query\executor\data_processing\join\left_join.rs`: 1 warnings
- `src\query\optimizer\limit_pushdown.rs`: 1 warnings
- `src\query\visitor\find_visitor.rs`: 1 warnings
- `src\query\optimizer\projection_pushdown.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 warnings
- `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 warnings

## Detailed Error Categorization

### error[E0432]: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

**Total Occurrences**: 35  
**Unique Files**: 34

#### `src\query\executor\result_processing\projection.rs`: 2 occurrences

- Line 15: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`
- Line 346: unresolved import `crate::query::executor::traits::ExecutorCore`: no `ExecutorCore` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 12: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 8: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 10: unresolved import `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorMetadata` in `query::executor::traits`

#### `src\query\executor\mod.rs`: 1 occurrences

- Line 18: unresolved imports `traits::ExecutorCore`, `traits::ExecutorLifecycle`, `traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 occurrences

- Line 9: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\result_processing\limit.rs`: 1 occurrences

- Line 15: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 23: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 1 occurrences

- Line 9: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 18: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\base.rs`: 1 occurrences

- Line 8: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 10: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 9: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 11: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 20: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\cypher\factory.rs`: 1 occurrences

- Line 153: unresolved imports `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 16: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 15: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 20: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\cypher\base.rs`: 1 occurrences

- Line 9: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\join\right_join.rs`: 1 occurrences

- Line 11: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 11: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 15: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 18: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\result_processing\sort.rs`: 1 occurrences

- Line 18: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 16: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 11: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 16: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 10: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\join\inner_join.rs`: 1 occurrences

- Line 17: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 8: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\graph_traversal\expand.rs`: 1 occurrences

- Line 9: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 16: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 16: unresolved imports `crate::query::executor::traits::ExecutorCore`, `crate::query::executor::traits::ExecutorLifecycle`, `crate::query::executor::traits::ExecutorMetadata`: no `ExecutorCore` in `query::executor::traits`, no `ExecutorLifecycle` in `query::executor::traits`, no `ExecutorMetadata` in `query::executor::traits`, help: a similar name exists in the module: `Executor`

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

#### `src\query\parser\statements\match_stmt.rs`: 3 occurrences

- Line 106: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 110: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 114: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

#### `src\query\parser\parser\pattern_parser.rs`: 3 occurrences

- Line 105: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 109: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 113: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

#### `src\query\parser\ast\pattern.rs`: 2 occurrences

- Line 342: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 367: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

#### `src\query\parser\parser\statement_parser.rs`: 2 occurrences

- Line 313: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 316: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

#### `src\query\parser\statements\create.rs`: 2 occurrences

- Line 88: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 92: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

#### `src\query\parser\ast\tests.rs`: 2 occurrences

- Line 278: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`
- Line 303: failed to resolve: use of undeclared type `EdgeDirection`: use of undeclared type `EdgeDirection`

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

#### `src\query\executor\data_processing\graph_traversal\traits.rs`: 2 occurrences

- Line 4: enum import `EdgeDirection` is private: private enum import
- Line 13: enum import `EdgeDirection` is private: private enum import

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 278: enum import `EdgeDirection` is private: private enum import
- Line 288: enum import `EdgeDirection` is private: private enum import

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 occurrences

- Line 7: enum import `EdgeDirection` is private: private enum import

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 5: enum import `EdgeDirection` is private: private enum import

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 7: enum import `EdgeDirection` is private: private enum import

#### `src\query\executor\data_processing\graph_traversal\expand.rs`: 1 occurrences

- Line 7: enum import `EdgeDirection` is private: private enum import

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 1 occurrences

- Line 7: enum import `EdgeDirection` is private: private enum import

### error[E0405]: cannot find trait `ExecutorLifecycle` in module `crate::query::executor::traits`: not found in `crate::query::executor::traits`

**Total Occurrences**: 14  
**Unique Files**: 3

#### `src\query\executor\traits.rs`: 9 occurrences

- Line 196: cannot find trait `StorageAccess` in this scope: not found in this scope
- Line 206: cannot find trait `InputAccess` in this scope: not found in this scope
- Line 224: cannot find trait `ExecutorWithStorage` in this scope: not found in this scope
- ... 6 more occurrences in this file

#### `src\query\executor\data_processing\set_operations\base.rs`: 3 occurrences

- Line 200: cannot find trait `ExecutorCore` in module `crate::query::executor::traits`: help: a trait with a similar name exists: `Executor`
- Line 216: cannot find trait `ExecutorLifecycle` in module `crate::query::executor::traits`: not found in `crate::query::executor::traits`
- Line 230: cannot find trait `ExecutorMetadata` in module `crate::query::executor::traits`: not found in `crate::query::executor::traits`

#### `src\query\executor\result_processing\projection.rs`: 2 occurrences

- Line 458: cannot find trait `ExecutorLifecycle` in module `crate::query::executor::traits`: not found in `crate::query::executor::traits`
- Line 472: cannot find trait `ExecutorMetadata` in module `crate::query::executor::traits`: not found in `crate::query::executor::traits`

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

### error[E0046]: not all trait items implemented, missing: `execute`, `open`, `close`, `is_open`, `id`, `name`, `description`: missing `execute`, `open`, `close`, `is_open`, `id`, `name`, `description` in implementation

**Total Occurrences**: 5  
**Unique Files**: 3

#### `src\query\executor\data_processing\loops.rs`: 3 occurrences

- Line 394: not all trait items implemented, missing: `execute`, `open`, `close`, `is_open`, `id`, `name`, `description`: missing `execute`, `open`, `close`, `is_open`, `id`, `name`, `description` in implementation
- Line 462: not all trait items implemented, missing: `execute`, `open`, `close`, `is_open`, `id`, `name`, `description`: missing `execute`, `open`, `close`, `is_open`, `id`, `name`, `description` in implementation
- Line 568: not all trait items implemented, missing: `execute`, `open`, `close`, `is_open`, `id`, `name`, `description`: missing `execute`, `open`, `close`, `is_open`, `id`, `name`, `description` in implementation

#### `src\query\executor\base.rs`: 1 occurrences

- Line 240: not all trait items implemented, missing: `execute`, `open`, `close`, `is_open`, `id`, `name`, `description`: missing `execute`, `open`, `close`, `is_open`, `id`, `name`, `description` in implementation

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 449: not all trait items implemented, missing: `execute`, `open`, `close`, `is_open`, `id`, `name`, `description`: missing `execute`, `open`, `close`, `is_open`, `id`, `name`, `description` in implementation

## Detailed Warning Categorization

### warning: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

**Total Occurrences**: 22  
**Unique Files**: 20

#### `src\query\parser\expressions\expression_converter.rs`: 2 occurrences

- Line 6: unused import: `NullType`
- Line 457: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\core\context\mod.rs`: 2 occurrences

- Line 5: unused import: `crate::core::Value`
- Line 46: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 269: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 12: unused import: `HasStorage`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 10: unused import: `crate::expression::evaluator::expression_evaluator::ExpressionEvaluator`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 347: unused import: `UnaryOperator`

#### `src\query\executor\recursion_detector.rs`: 1 occurrences

- Line 3: unused import: `HashMap`

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 564: unused import: `SortNode`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `crate::core::types::expression::DataType`

#### `src\query\parser\ast\types.rs`: 1 occurrences

- Line 4: unused import: `crate::core::types::EdgeDirection`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 887: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\executor\base.rs`: 1 occurrences

- Line 8: unused import: `HasInput`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

