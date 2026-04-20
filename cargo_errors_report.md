# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 60
- **Total Warnings**: 6
- **Total Issues**: 66
- **Unique Error Patterns**: 7
- **Unique Warning Patterns**: 4
- **Files with Issues**: 41

## Error Statistics

**Total Errors**: 60

### Error Type Breakdown

- **error[E0432]**: 39 errors
- **error[E0433]**: 18 errors
- **error[E0425]**: 3 errors

### Files with Errors (Top 10)

- `src\core\value\decimal128.rs`: 14 errors
- `src\core\types\span.rs`: 4 errors
- `src\core\error\storage.rs`: 4 errors
- `src\storage\operations\rollback.rs`: 2 errors
- `src\storage\entity\vertex_storage.rs`: 2 errors
- `src\storage\operations\redb\reader.rs`: 2 errors
- `src\storage\entity\edge_storage.rs`: 2 errors
- `src\core\vertex_edge_path.rs`: 1 errors
- `src\core\types\cluster.rs`: 1 errors
- `src\core\types\mod.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 6

### Warning Type Breakdown

- **warning**: 6 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\transformations\macros.rs`: 3 warnings
- `src\api\embedded\telemetry.rs`: 1 warnings
- `src\api\server\grpc\server.rs`: 1 warnings
- `src\query\cache\plan_cache.rs`: 1 warnings

## Detailed Error Categorization

### error[E0432]: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

**Total Occurrences**: 39  
**Unique Files**: 35

#### `src\storage\operations\rollback.rs`: 2 occurrences

- Line 8: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`
- Line 335: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\storage\entity\vertex_storage.rs`: 2 occurrences

- Line 603: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`
- Line 626: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\storage\entity\edge_storage.rs`: 2 occurrences

- Line 460: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`
- Line 491: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\value\decimal128.rs`: 2 occurrences

- Line 24: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`
- Line 31: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\import_export.rs`: 1 occurrences

- Line 3: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\value\null.rs`: 1 occurrences

- Line 5: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\tag.rs`: 1 occurrences

- Line 5: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\fulltext_query.rs`: 1 occurrences

- Line 7: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\storage\operations\redb\writer.rs`: 1 occurrences

- Line 7: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\metadata_version.rs`: 1 occurrences

- Line 4: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\storage\index\index_key_codec.rs`: 1 occurrences

- Line 8: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\query\parser\ast\fulltext.rs`: 1 occurrences

- Line 9: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\schema_change.rs`: 1 occurrences

- Line 4: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\user.rs`: 1 occurrences

- Line 3: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\value\value_def.rs`: 1 occurrences

- Line 4: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\data_modification.rs`: 1 occurrences

- Line 4: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\api\embedded\c_api\telemetry.rs`: 1 occurrences

- Line 7: unresolved import `crate::api::embedded::c_api::types::GRAPHDB_FREE_STRING`: no `GRAPHDB_FREE_STRING` in `api::embedded::c_api::types`

#### `src\core\vertex_edge_path.rs`: 1 occurrences

- Line 1: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\value\date_time.rs`: 1 occurrences

- Line 5: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\value\geography.rs`: 1 occurrences

- Line 5: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\property.rs`: 1 occurrences

- Line 5: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\index.rs`: 1 occurrences

- Line 7: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\storage\operations\redb\reader.rs`: 1 occurrences

- Line 5: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\query\parser\ast\vector.rs`: 1 occurrences

- Line 8: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\storage\metadata\redb_index_metadata_manager.rs`: 1 occurrences

- Line 5: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\cluster.rs`: 1 occurrences

- Line 3: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\mod.rs`: 1 occurrences

- Line 22: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\space.rs`: 1 occurrences

- Line 4: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\storage\metadata\redb_extended_schema.rs`: 1 occurrences

- Line 9: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\query\data_set.rs`: 1 occurrences

- Line 6: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\value\list.rs`: 1 occurrences

- Line 5: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\storage\metadata\redb_schema_manager.rs`: 1 occurrences

- Line 9: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\transaction\context.rs`: 1 occurrences

- Line 10: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\value\vector.rs`: 1 occurrences

- Line 6: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

#### `src\core\types\edge.rs`: 1 occurrences

- Line 5: unresolved import `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

### error[E0433]: failed to resolve: use of unresolved module or unlinked crate `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

**Total Occurrences**: 18  
**Unique Files**: 3

#### `src\core\value\decimal128.rs`: 12 occurrences

- Line 72: failed to resolve: use of unresolved module or unlinked crate `oxicoide`: use of unresolved module or unlinked crate `oxicoide`
- Line 75: failed to resolve: use of unresolved module or unlinked crate `oxicoide`: use of unresolved module or unlinked crate `oxicoide`
- Line 82: failed to resolve: use of unresolved module or unlinked crate `oxicoide`: use of unresolved module or unlinked crate `oxicoide`
- ... 9 more occurrences in this file

#### `src\core\types\span.rs`: 4 occurrences

- Line 92: failed to resolve: use of unresolved module or unlinked crate `oxicoide`: use of unresolved module or unlinked crate `oxicoide`
- Line 93: failed to resolve: use of unresolved module or unlinked crate `oxicoide`: use of unresolved module or unlinked crate `oxicoide`
- Line 23: failed to resolve: use of unresolved module or unlinked crate `oxicoide`: use of unresolved module or unlinked crate `oxicoide`
- ... 1 more occurrences in this file

#### `src\core\error\storage.rs`: 2 occurrences

- Line 84: failed to resolve: use of unresolved module or unlinked crate `oxicoide`: use of unresolved module or unlinked crate `oxicoide`
- Line 90: failed to resolve: use of unresolved module or unlinked crate `oxicoide`: use of unresolved module or unlinked crate `oxicoide`

### error[E0425]: cannot find type `EncodeError` in module `oxicode::error`: not found in `oxicode::error`

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\core\error\storage.rs`: 2 occurrences

- Line 83: cannot find type `EncodeError` in module `oxicode::error`: not found in `oxicode::error`
- Line 89: cannot find type `DecodeError` in module `oxicode::error`: not found in `oxicode::error`

#### `src\storage\operations\redb\reader.rs`: 1 occurrences

- Line 62: cannot find function `standard` in this scope: not found in this scope

## Detailed Warning Categorization

### warning: unused import: `std::sync::atomic::Ordering`

**Total Occurrences**: 6  
**Unique Files**: 4

#### `src\query\executor\result_processing\transformations\macros.rs`: 3 occurrences

- Line 7: `crate` references the macro call's crate: help: to reference the macro definition's crate, use: `$crate`
- Line 53: `crate` references the macro call's crate: help: to reference the macro definition's crate, use: `$crate`
- Line 98: `crate` references the macro call's crate: help: to reference the macro definition's crate, use: `$crate`

#### `src\query\cache\plan_cache.rs`: 1 occurrences

- Line 727: unused import: `std::sync::atomic::Ordering`

#### `src\api\embedded\telemetry.rs`: 1 occurrences

- Line 19: unused imports: `TelemetryRecorder` and `init_global_recorder`

#### `src\api\server\grpc\server.rs`: 1 occurrences

- Line 505: unused import: `super::*`

