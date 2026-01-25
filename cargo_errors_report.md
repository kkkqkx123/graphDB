# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 188
- **Total Warnings**: 22
- **Total Issues**: 210
- **Unique Error Patterns**: 40
- **Unique Warning Patterns**: 20
- **Files with Issues**: 41

## Error Statistics

**Total Errors**: 188

### Error Type Breakdown

- **error[E0433]**: 90 errors
- **error[E0599]**: 37 errors
- **error[E0412]**: 29 errors
- **error[E0432]**: 16 errors
- **error[E0560]**: 10 errors
- **error[E0061]**: 6 errors

### Files with Errors (Top 10)

- `src\query\executor\admin\index\edge_index.rs`: 25 errors
- `src\query\executor\admin\index\tag_index.rs`: 24 errors
- `src\query\executor\admin\data\insert.rs`: 13 errors
- `src\query\executor\admin\index\rebuild_index.rs`: 12 errors
- `src\query\executor\graph_query_executor.rs`: 8 errors
- `src\query\executor\admin\tag\desc_tag.rs`: 7 errors
- `src\query\executor\admin\space\desc_space.rs`: 7 errors
- `src\query\executor\admin\data\update.rs`: 7 errors
- `src\query\executor\admin\edge\desc_edge.rs`: 7 errors
- `src\query\executor\admin\edge\create_edge.rs`: 7 errors

## Warning Statistics

**Total Warnings**: 22

### Warning Type Breakdown

- **warning**: 22 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 warnings
- `src\query\planner\statements\match_planner.rs`: 2 warnings
- `src\query\context\runtime_context.rs`: 1 warnings
- `src\query\executor\admin\space\create_space.rs`: 1 warnings
- `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 warnings
- `src\query\context\managers\schema_traits.rs`: 1 warnings
- `src\query\executor\data_processing\join\base_join.rs`: 1 warnings
- `src\query\executor\admin\mod.rs`: 1 warnings
- `src\query\executor\base\executor_base.rs`: 1 warnings
- `src\query\scheduler\async_scheduler.rs`: 1 warnings

## Detailed Error Categorization

### error[E0433]: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

**Total Occurrences**: 90  
**Unique Files**: 21

#### `src\query\executor\admin\index\edge_index.rs`: 13 occurrences

- Line 61: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 64: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 66: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- ... 10 more occurrences in this file

#### `src\query\executor\admin\index\tag_index.rs`: 13 occurrences

- Line 93: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 96: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 98: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- ... 10 more occurrences in this file

#### `src\query\executor\admin\data\insert.rs`: 6 occurrences

- Line 62: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 63: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 64: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- ... 3 more occurrences in this file

#### `src\query\executor\admin\index\rebuild_index.rs`: 6 occurrences

- Line 43: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 44: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 45: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- ... 3 more occurrences in this file

#### `src\query\executor\admin\space\drop_space.rs`: 4 occurrences

- Line 54: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 57: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 59: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- ... 1 more occurrences in this file

#### `src\query\executor\admin\edge\drop_edge.rs`: 4 occurrences

- Line 57: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 60: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 62: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- ... 1 more occurrences in this file

#### `src\query\executor\admin\edge\create_edge.rs`: 4 occurrences

- Line 85: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 88: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 90: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- ... 1 more occurrences in this file

#### `src\query\executor\admin\tag\drop_tag.rs`: 4 occurrences

- Line 57: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 60: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 62: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- ... 1 more occurrences in this file

#### `src\query\executor\admin\tag\create_tag.rs`: 4 occurrences

- Line 85: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 88: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 90: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- ... 1 more occurrences in this file

#### `src\query\executor\admin\space\desc_space.rs`: 3 occurrences

- Line 83: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 85: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 86: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

#### `src\query\executor\admin\tag\desc_tag.rs`: 3 occurrences

- Line 86: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 88: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 90: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

#### `src\query\executor\admin\data\update.rs`: 3 occurrences

- Line 103: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 104: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 105: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

#### `src\query\executor\admin\edge\desc_edge.rs`: 3 occurrences

- Line 86: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 88: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 90: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

#### `src\query\executor\admin\data\delete.rs`: 3 occurrences

- Line 76: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 82: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 84: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

#### `src\query\executor\admin\user.rs`: 3 occurrences

- Line 49: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 50: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 51: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

#### `src\query\executor\admin\tag\alter_tag.rs`: 3 occurrences

- Line 108: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 109: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 111: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

#### `src\query\executor\admin\edge\alter_edge.rs`: 3 occurrences

- Line 108: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 109: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 111: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

#### `src\query\executor\admin\tag\show_tags.rs`: 2 occurrences

- Line 56: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 58: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

#### `src\query\executor\admin\space\show_spaces.rs`: 2 occurrences

- Line 70: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 72: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

#### `src\query\executor\admin\edge\show_edges.rs`: 2 occurrences

- Line 56: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 58: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

#### `src\query\executor\admin\space\create_space.rs`: 2 occurrences

- Line 79: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 80: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`

### error[E0599]: no method named `get_storage` found for mutable reference `&mut CreateEdgeExecutor<S>` in the current scope: method not found in `&mut CreateEdgeExecutor<S>`

**Total Occurrences**: 37  
**Unique Files**: 21

#### `src\query\executor\admin\index\edge_index.rs`: 5 occurrences

- Line 51: no method named `get_storage` found for mutable reference `&mut CreateEdgeIndexExecutor<S>` in the current scope: method not found in `&mut CreateEdgeIndexExecutor<S>`
- Line 115: no method named `get_storage` found for mutable reference `&mut DropEdgeIndexExecutor<S>` in the current scope: method not found in `&mut DropEdgeIndexExecutor<S>`
- Line 168: no method named `get_storage` found for mutable reference `&mut DescEdgeIndexExecutor<S>` in the current scope: method not found in `&mut DescEdgeIndexExecutor<S>`
- ... 2 more occurrences in this file

#### `src\query\executor\admin\index\rebuild_index.rs`: 4 occurrences

- Line 35: no method named `get_storage` found for mutable reference `&mut RebuildTagIndexExecutor<S>` in the current scope: method not found in `&mut RebuildTagIndexExecutor<S>`
- Line 37: no variant or associated item named `StorageError` found for enum `core::error::DBError` in the current scope: variant or associated item not found in `DBError`
- Line 106: no method named `get_storage` found for mutable reference `&mut RebuildEdgeIndexExecutor<S>` in the current scope: method not found in `&mut RebuildEdgeIndexExecutor<S>`
- ... 1 more occurrences in this file

#### `src\query\executor\admin\index\tag_index.rs`: 4 occurrences

- Line 83: no method named `get_storage` found for mutable reference `&mut CreateTagIndexExecutor<S>` in the current scope: method not found in `&mut CreateTagIndexExecutor<S>`
- Line 147: no method named `get_storage` found for mutable reference `&mut DropTagIndexExecutor<S>` in the current scope: method not found in `&mut DropTagIndexExecutor<S>`
- Line 200: no method named `get_storage` found for mutable reference `&mut DescTagIndexExecutor<S>` in the current scope: method not found in `&mut DescTagIndexExecutor<S>`
- ... 1 more occurrences in this file

#### `src\query\executor\admin\data\insert.rs`: 4 occurrences

- Line 54: no method named `get_storage` found for mutable reference `&mut InsertVertexExecutor<S>` in the current scope: method not found in `&mut InsertVertexExecutor<S>`
- Line 56: no variant or associated item named `StorageError` found for enum `core::error::DBError` in the current scope: variant or associated item not found in `DBError`
- Line 123: no method named `get_storage` found for mutable reference `&mut InsertEdgeExecutor<S>` in the current scope: method not found in `&mut InsertEdgeExecutor<S>`
- ... 1 more occurrences in this file

#### `src\query\executor\admin\data\delete.rs`: 2 occurrences

- Line 61: no method named `get_storage` found for mutable reference `&mut delete::DeleteExecutor<S>` in the current scope: method not found in `&mut DeleteExecutor<S>`
- Line 63: no variant or associated item named `StorageError` found for enum `core::error::DBError` in the current scope: variant or associated item not found in `DBError`

#### `src\query\executor\admin\user.rs`: 2 occurrences

- Line 41: no method named `get_storage` found for mutable reference `&mut ChangePasswordExecutor<S>` in the current scope: method not found in `&mut ChangePasswordExecutor<S>`
- Line 43: no variant or associated item named `StorageError` found for enum `core::error::DBError` in the current scope: variant or associated item not found in `DBError`

#### `src\query\executor\admin\data\update.rs`: 2 occurrences

- Line 95: no method named `get_storage` found for mutable reference `&mut update::UpdateExecutor<S>` in the current scope: method not found in `&mut UpdateExecutor<S>`
- Line 97: no variant or associated item named `StorageError` found for enum `core::error::DBError` in the current scope: variant or associated item not found in `DBError`

#### `src\query\executor\admin\edge\create_edge.rs`: 1 occurrences

- Line 75: no method named `get_storage` found for mutable reference `&mut CreateEdgeExecutor<S>` in the current scope: method not found in `&mut CreateEdgeExecutor<S>`

#### `src\query\executor\admin\tag\create_tag.rs`: 1 occurrences

- Line 75: no method named `get_storage` found for mutable reference `&mut CreateTagExecutor<S>` in the current scope: method not found in `&mut CreateTagExecutor<S>`

#### `src\query\executor\admin\tag\desc_tag.rs`: 1 occurrences

- Line 49: no method named `get_storage` found for mutable reference `&mut DescTagExecutor<S>` in the current scope: method not found in `&mut DescTagExecutor<S>`

#### `src\query\executor\admin\edge\drop_edge.rs`: 1 occurrences

- Line 47: no method named `get_storage` found for mutable reference `&mut DropEdgeExecutor<S>` in the current scope: method not found in `&mut DropEdgeExecutor<S>`

#### `src\query\executor\admin\space\desc_space.rs`: 1 occurrences

- Line 60: no method named `get_storage` found for mutable reference `&mut DescSpaceExecutor<S>` in the current scope: method not found in `&mut DescSpaceExecutor<S>`

#### `src\query\executor\admin\tag\drop_tag.rs`: 1 occurrences

- Line 47: no method named `get_storage` found for mutable reference `&mut DropTagExecutor<S>` in the current scope: method not found in `&mut DropTagExecutor<S>`

#### `src\query\executor\admin\tag\show_tags.rs`: 1 occurrences

- Line 34: no method named `get_storage` found for mutable reference `&mut ShowTagsExecutor<S>` in the current scope: method not found in `&mut ShowTagsExecutor<S>`

#### `src\query\executor\admin\edge\show_edges.rs`: 1 occurrences

- Line 34: no method named `get_storage` found for mutable reference `&mut ShowEdgesExecutor<S>` in the current scope: method not found in `&mut ShowEdgesExecutor<S>`

#### `src\query\executor\admin\space\show_spaces.rs`: 1 occurrences

- Line 32: no method named `get_storage` found for mutable reference `&mut ShowSpacesExecutor<S>` in the current scope: method not found in `&mut ShowSpacesExecutor<S>`

#### `src\query\executor\admin\space\drop_space.rs`: 1 occurrences

- Line 44: no method named `get_storage` found for mutable reference `&mut DropSpaceExecutor<S>` in the current scope: method not found in `&mut DropSpaceExecutor<S>`

#### `src\query\executor\admin\edge\desc_edge.rs`: 1 occurrences

- Line 49: no method named `get_storage` found for mutable reference `&mut DescEdgeExecutor<S>` in the current scope: method not found in `&mut DescEdgeExecutor<S>`

#### `src\query\executor\admin\space\create_space.rs`: 1 occurrences

- Line 69: no method named `get_storage` found for mutable reference `&mut CreateSpaceExecutor<S>` in the current scope: method not found in `&mut CreateSpaceExecutor<S>`

#### `src\query\executor\admin\edge\alter_edge.rs`: 1 occurrences

- Line 98: no method named `get_storage` found for mutable reference `&mut AlterEdgeExecutor<S>` in the current scope: method not found in `&mut AlterEdgeExecutor<S>`

#### `src\query\executor\admin\tag\alter_tag.rs`: 1 occurrences

- Line 98: no method named `get_storage` found for mutable reference `&mut AlterTagExecutor<S>` in the current scope: method not found in `&mut AlterTagExecutor<S>`

### error[E0412]: cannot find type `ExecutionResult` in this scope: not found in this scope

**Total Occurrences**: 29  
**Unique Files**: 21

#### `src\query\executor\admin\index\edge_index.rs`: 4 occurrences

- Line 50: cannot find type `ExecutionResult` in this scope: not found in this scope
- Line 114: cannot find type `ExecutionResult` in this scope: not found in this scope
- Line 167: cannot find type `ExecutionResult` in this scope: not found in this scope
- ... 1 more occurrences in this file

#### `src\query\executor\admin\index\tag_index.rs`: 4 occurrences

- Line 82: cannot find type `ExecutionResult` in this scope: not found in this scope
- Line 146: cannot find type `ExecutionResult` in this scope: not found in this scope
- Line 199: cannot find type `ExecutionResult` in this scope: not found in this scope
- ... 1 more occurrences in this file

#### `src\query\executor\admin\data\insert.rs`: 2 occurrences

- Line 53: cannot find type `ExecutionResult` in this scope: not found in this scope
- Line 122: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\index\rebuild_index.rs`: 2 occurrences

- Line 34: cannot find type `ExecutionResult` in this scope: not found in this scope
- Line 105: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\space\create_space.rs`: 1 occurrences

- Line 68: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\tag\show_tags.rs`: 1 occurrences

- Line 33: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\tag\create_tag.rs`: 1 occurrences

- Line 74: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\edge\show_edges.rs`: 1 occurrences

- Line 33: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\edge\alter_edge.rs`: 1 occurrences

- Line 97: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\data\update.rs`: 1 occurrences

- Line 94: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\tag\drop_tag.rs`: 1 occurrences

- Line 46: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\edge\desc_edge.rs`: 1 occurrences

- Line 48: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\user.rs`: 1 occurrences

- Line 40: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\data\delete.rs`: 1 occurrences

- Line 60: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\space\drop_space.rs`: 1 occurrences

- Line 43: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\edge\drop_edge.rs`: 1 occurrences

- Line 46: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\tag\alter_tag.rs`: 1 occurrences

- Line 97: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\space\desc_space.rs`: 1 occurrences

- Line 59: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\edge\create_edge.rs`: 1 occurrences

- Line 74: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\space\show_spaces.rs`: 1 occurrences

- Line 31: cannot find type `ExecutionResult` in this scope: not found in this scope

#### `src\query\executor\admin\tag\desc_tag.rs`: 1 occurrences

- Line 48: cannot find type `ExecutionResult` in this scope: not found in this scope

### error[E0432]: unresolved import `crate::core::Row`: no `Row` in `core`

**Total Occurrences**: 16  
**Unique Files**: 15

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 275: unresolved imports `admin_executor::AlterEdgeInfo`, `admin_executor::AlterTagInfo`, `admin_executor::AlterTagItem`, `admin_executor::AlterEdgeItem`, `admin_executor::AlterTagOp`, `admin_executor::AlterEdgeOp`: no `AlterEdgeInfo` in `query::executor::admin`, no `AlterTagInfo` in `query::executor::admin`, no `AlterTagItem` in `query::executor::admin`, no `AlterEdgeItem` in `query::executor::admin`, no `AlterTagOp` in `query::executor::admin`, no `AlterEdgeOp` in `query::executor::admin`
- Line 303: unresolved import `admin_executor::PasswordInfo`: no `PasswordInfo` in `query::executor::admin`

#### `src\query\executor\admin\edge\show_edges.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::Row`: no `Row` in `core`

#### `src\query\executor\admin\tag\create_tag.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::PropertyDef`: no `PropertyDef` in `core`

#### `src\query\executor\admin\edge\desc_edge.rs`: 1 occurrences

- Line 8: unresolved imports `crate::core::PropertyType`, `crate::core::Row`: no `PropertyType` in `core`, no `Row` in `core`

#### `src\query\executor\admin\data\update.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::PropertyValue`: no `PropertyValue` in `core`

#### `src\query\executor\admin\tag\show_tags.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::Row`: no `Row` in `core`

#### `src\query\executor\admin\space\show_spaces.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::Row`: no `Row` in `core`

#### `src\query\executor\admin\data\insert.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::PropertyValue`: no `PropertyValue` in `core`

#### `src\query\executor\admin\tag\desc_tag.rs`: 1 occurrences

- Line 8: unresolved imports `crate::core::PropertyType`, `crate::core::Row`: no `PropertyType` in `core`, no `Row` in `core`

#### `src\query\executor\admin\edge\create_edge.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::PropertyDef`: no `PropertyDef` in `core`

#### `src\query\executor\admin\edge\alter_edge.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::PropertyDef`: no `PropertyDef` in `core`

#### `src\query\executor\admin\space\desc_space.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::Row`: no `Row` in `core`

#### `src\query\executor\admin\index\tag_index.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::Row`: no `Row` in `core`

#### `src\query\executor\admin\index\edge_index.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::Row`: no `Row` in `core`

#### `src\query\executor\admin\tag\alter_tag.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::PropertyDef`: no `PropertyDef` in `core`

### error[E0560]: struct `core::value::types::DataSet` has no field named `columns`: `core::value::types::DataSet` does not have this field

**Total Occurrences**: 10  
**Unique Files**: 8

#### `src\query\executor\admin\index\tag_index.rs`: 2 occurrences

- Line 219: struct `core::value::types::DataSet` has no field named `columns`: `core::value::types::DataSet` does not have this field
- Line 281: struct `core::value::types::DataSet` has no field named `columns`: `core::value::types::DataSet` does not have this field

#### `src\query\executor\admin\index\edge_index.rs`: 2 occurrences

- Line 187: struct `core::value::types::DataSet` has no field named `columns`: `core::value::types::DataSet` does not have this field
- Line 247: struct `core::value::types::DataSet` has no field named `columns`: `core::value::types::DataSet` does not have this field

#### `src\query\executor\admin\space\show_spaces.rs`: 1 occurrences

- Line 59: struct `core::value::types::DataSet` has no field named `columns`: `core::value::types::DataSet` does not have this field

#### `src\query\executor\admin\tag\show_tags.rs`: 1 occurrences

- Line 53: struct `core::value::types::DataSet` has no field named `columns`: `core::value::types::DataSet` does not have this field

#### `src\query\executor\admin\edge\show_edges.rs`: 1 occurrences

- Line 53: struct `core::value::types::DataSet` has no field named `columns`: `core::value::types::DataSet` does not have this field

#### `src\query\executor\admin\tag\desc_tag.rs`: 1 occurrences

- Line 77: struct `core::value::types::DataSet` has no field named `columns`: `core::value::types::DataSet` does not have this field

#### `src\query\executor\admin\space\desc_space.rs`: 1 occurrences

- Line 72: struct `core::value::types::DataSet` has no field named `columns`: `core::value::types::DataSet` does not have this field

#### `src\query\executor\admin\edge\desc_edge.rs`: 1 occurrences

- Line 77: struct `core::value::types::DataSet` has no field named `columns`: `core::value::types::DataSet` does not have this field

### error[E0061]: this function takes 4 arguments but 3 arguments were supplied

**Total Occurrences**: 6  
**Unique Files**: 1

#### `src\query\executor\graph_query_executor.rs`: 6 occurrences

- Line 228: this function takes 4 arguments but 3 arguments were supplied
- Line 233: this function takes 4 arguments but 3 arguments were supplied
- Line 238: this function takes 4 arguments but 3 arguments were supplied
- ... 3 more occurrences in this file

## Detailed Warning Categorization

### warning: unused import: `crate::core::Value`

**Total Occurrences**: 22  
**Unique Files**: 20

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 207: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 207: variable does not need to be mutable

#### `src\query\planner\statements\match_planner.rs`: 2 occurrences

- Line 96: unused variable: `match_ctx`: help: if this is intentional, prefix it with an underscore: `_match_ctx`
- Line 157: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\parser\ast\stmt.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Value`

#### `src\query\validator\validation_interface.rs`: 1 occurrences

- Line 4: unused imports: `DBError`, `QueryError`, `ValidationError as CoreValidationError`, and `ValidationErrorType as CoreValidationErrorType`

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 1: unused import: `crate::core::error::DBError`

#### `src\query\executor\base\executor_base.rs`: 1 occurrences

- Line 9: unused import: `crate::core::error::DBError`

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 9: unused import: `ExecutionContext`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 7: unused import: `Vertex`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 330: unnecessary parentheses around function argument

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 55: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\executor\admin\space\create_space.rs`: 1 occurrences

- Line 8: unused import: `crate::core::Value`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 15: unused import: `std::collections::HashMap`

#### `src\query\executor\admin\mod.rs`: 1 occurrences

- Line 13: unused import: `crate::storage::StorageEngine`

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 1 occurrences

- Line 10: unused macro definition: `impl_graph_traversal_executor`

#### `src\query\context\managers\schema_traits.rs`: 1 occurrences

- Line 247: unexpected `cfg` condition value: `schema-manager-default`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 22: unused imports: `MultiShortestPathExecutor` and `ShortestPathExecutor`

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 13: unused import: `crate::query::optimizer::rule_traits::BaseOptRule`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 42: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

