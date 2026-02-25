# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 20
- **Total Warnings**: 4
- **Total Issues**: 24
- **Unique Error Patterns**: 15
- **Unique Warning Patterns**: 4
- **Files with Issues**: 10

## Error Statistics

**Total Errors**: 20

### Error Type Breakdown

- **error[E0599]**: 11 errors
- **error[E0308]**: 5 errors
- **error[E0433]**: 3 errors
- **error[E0432]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\admin\query_management\show_stats.rs`: 6 errors
- `src\api\server\permission\permission_checker.rs`: 5 errors
- `src\query\query_context.rs`: 3 errors
- `src\query\planner\statements\create_planner.rs`: 3 errors
- `src\query\planner\statements\lookup_planner.rs`: 1 errors
- `src\api\server\graph_service.rs`: 1 errors
- `src\query\planner\statements\match_statement_planner.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 4

### Warning Type Breakdown

- **warning**: 4 warnings

### Files with Warnings (Top 10)

- `src\api\core\transaction_api.rs`: 2 warnings
- `src\api\core\schema_api.rs`: 1 warnings
- `src\query\query_manager.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `space_id` found for reference `&RequestContext` in the current scope: method not found in `&RequestContext`

**Total Occurrences**: 11  
**Unique Files**: 5

#### `src\query\executor\admin\query_management\show_stats.rs`: 4 occurrences

- Line 147: no method named `get_query_stats` found for reference `&std::sync::Arc<QueryManager>` in the current scope
- Line 203: no method named `get_query_stats` found for reference `&std::sync::Arc<QueryManager>` in the current scope
- Line 248: no variant or associated item named `Success` found for enum `query::query_manager::QueryStatus` in the current scope: variant or associated item not found in `QueryStatus`
- ... 1 more occurrences in this file

#### `src\query\query_context.rs`: 3 occurrences

- Line 214: no method named `set_response_error` found for struct `std::sync::Arc<RequestContext>` in the current scope
- Line 231: no method named `get_parameter` found for struct `std::sync::Arc<RequestContext>` in the current scope
- Line 263: no function or associated item named `default` found for struct `RequestContext` in the current scope: function or associated item not found in `RequestContext`

#### `src\api\server\permission\permission_checker.rs`: 2 occurrences

- Line 104: no method named `can_write_schema` found for struct `permission_manager::PermissionManager` in the current scope
- Line 173: no method named `can_write_role` found for struct `permission_manager::PermissionManager` in the current scope

#### `src\query\planner\statements\match_statement_planner.rs`: 1 occurrences

- Line 76: no method named `space_id` found for reference `&RequestContext` in the current scope: method not found in `&RequestContext`

#### `src\query\planner\statements\lookup_planner.rs`: 1 occurrences

- Line 58: no method named `space_id` found for reference `&RequestContext` in the current scope: method not found in `&RequestContext`

### error[E0308]: mismatched types: expected `core::stats::QueryStatus`, found `query::query_manager::QueryStatus`

**Total Occurrences**: 5  
**Unique Files**: 2

#### `src\query\planner\statements\create_planner.rs`: 3 occurrences

- Line 133: mismatched types: expected `&str`, found `String`
- Line 152: mismatched types: expected `String`, found `&str`
- Line 171: mismatched types: expected `String`, found `&str`

#### `src\query\executor\admin\query_management\show_stats.rs`: 2 occurrences

- Line 249: mismatched types: expected `core::stats::QueryStatus`, found `query::query_manager::QueryStatus`
- Line 384: mismatched types: expected `core::stats::QueryStatus`, found `query::query_manager::QueryStatus`

### error[E0433]: failed to resolve: could not find `session` in `api`: could not find `session` in `api`

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\api\server\permission\permission_checker.rs`: 2 occurrences

- Line 269: failed to resolve: could not find `session` in `api`: could not find `session` in `api`
- Line 305: failed to resolve: could not find `service` in `api`: could not find `service` in `api`

#### `src\api\server\graph_service.rs`: 1 occurrences

- Line 58: failed to resolve: use of undeclared type `CoreStatsManager`: use of undeclared type `CoreStatsManager`, help: a struct with a similar name exists: `StatsManager`

### error[E0432]: unresolved import `crate::api::session`: could not find `session` in `api`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\server\permission\permission_checker.rs`: 1 occurrences

- Line 268: unresolved import `crate::api::session`: could not find `session` in `api`

## Detailed Warning Categorization

### warning: unused import: `Duration`

**Total Occurrences**: 4  
**Unique Files**: 3

#### `src\api\core\transaction_api.rs`: 2 occurrences

- Line 5: unused import: `TransactionId`
- Line 6: unused import: `SavepointId`

#### `src\query\query_manager.rs`: 1 occurrences

- Line 9: unused import: `Duration`

#### `src\api\core\schema_api.rs`: 1 occurrences

- Line 6: unused import: `CoreError`

