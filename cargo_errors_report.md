# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 7
- **Total Warnings**: 20
- **Total Issues**: 27
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 9
- **Files with Issues**: 11

## Error Statistics

**Total Errors**: 7

### Error Type Breakdown

- **error[E0433]**: 7 errors

### Files with Errors (Top 10)

- `src\query\executor\expression\functions\builtin\path.rs`: 5 errors
- `src\query\executor\utils\tag_filter.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 20

### Warning Type Breakdown

- **warning**: 20 warnings

### Files with Warnings (Top 10)

- `src\storage\engine\sync_wrapper.rs`: 6 warnings
- `src\storage\index\secondary\index_updater.rs`: 4 warnings
- `src\storage\engine\graph_storage\writer.rs`: 2 warnings
- `src\query\executor\graph_operations\graph_traversal\algorithms\bfs_shortest.rs`: 2 warnings
- `src\query\executor\result_processing\topn.rs`: 2 warnings
- `src\storage\edge\mutable_csr.rs`: 1 warnings
- `src\query\executor\graph_operations\graph_traversal\algorithms\a_star.rs`: 1 warnings
- `src\core\npath.rs`: 1 warnings
- `src\core\vertex_edge_path.rs`: 1 warnings

## Detailed Error Categorization

### error[E0433]: failed to resolve: use of undeclared type `VertexId`: use of undeclared type `VertexId`

**Total Occurrences**: 7  
**Unique Files**: 2

#### `src\query\executor\expression\functions\builtin\path.rs`: 5 occurrences

- Line 101: failed to resolve: use of undeclared type `VertexId`: use of undeclared type `VertexId`
- Line 112: failed to resolve: use of undeclared type `VertexId`: use of undeclared type `VertexId`
- Line 113: failed to resolve: use of undeclared type `VertexId`: use of undeclared type `VertexId`
- ... 2 more occurrences in this file

#### `src\query\executor\utils\tag_filter.rs`: 2 occurrences

- Line 101: failed to resolve: use of undeclared type `VertexId`: use of undeclared type `VertexId`
- Line 121: failed to resolve: use of undeclared type `VertexId`: use of undeclared type `VertexId`

## Detailed Warning Categorization

### warning: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `last_step.edge.src`

**Total Occurrences**: 20  
**Unique Files**: 9

#### `src\storage\engine\sync_wrapper.rs`: 6 occurrences

- Line 262: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*id`
- Line 273: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*id`
- Line 298: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*id`
- ... 3 more occurrences in this file

#### `src\storage\index\secondary\index_updater.rs`: 4 occurrences

- Line 663: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- Line 664: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`
- Line 714: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- ... 1 more occurrences in this file

#### `src\query\executor\graph_operations\graph_traversal\algorithms\bfs_shortest.rs`: 2 occurrences

- Line 348: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*meet_vid`
- Line 351: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*meet_vid`

#### `src\query\executor\result_processing\topn.rs`: 2 occurrences

- Line 860: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.src`
- Line 861: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `edge.dst`

#### `src\storage\engine\graph_storage\writer.rs`: 2 occurrences

- Line 264: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*src`
- Line 265: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*dst`

#### `src\core\vertex_edge_path.rs`: 1 occurrences

- Line 659: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `last_step.edge.src`

#### `src\core\npath.rs`: 1 occurrences

- Line 271: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `v.vid`

#### `src\storage\edge\mutable_csr.rs`: 1 occurrences

- Line 811: using `clone` on type `VertexId` which implements the `Copy` trait: help: try dereferencing it: `*src`

#### `src\query\executor\graph_operations\graph_traversal\algorithms\a_star.rs`: 1 occurrences

- Line 365: using `clone` on type `VertexId` which implements the `Copy` trait: help: try removing the `clone` call: `current.vertex_id`

