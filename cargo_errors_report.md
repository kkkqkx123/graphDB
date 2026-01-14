# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 245
- **Total Warnings**: 63
- **Total Issues**: 308
- **Unique Error Patterns**: 95
- **Unique Warning Patterns**: 48
- **Files with Issues**: 80

## Error Statistics

**Total Errors**: 245

### Error Type Breakdown

- **error[E0599]**: 62 errors
- **error[E0308]**: 52 errors
- **error[E0046]**: 50 errors
- **error[E0609]**: 17 errors
- **error[E0034]**: 12 errors
- **error[E0560]**: 11 errors
- **error[E0063]**: 9 errors
- **error[E0053]**: 6 errors
- **error[E0412]**: 5 errors
- **error[E0369]**: 5 errors
- **error[E0614]**: 4 errors
- **error[E0432]**: 4 errors
- **error[E0382]**: 2 errors
- **error[E0407]**: 1 errors
- **error[E0592]**: 1 errors
- **error[E0061]**: 1 errors
- **error[E0433]**: 1 errors
- **error[E0502]**: 1 errors
- **error[E0271]**: 1 errors

### Files with Errors (Top 10)

- `src\query\context\managers\impl\storage_client_impl.rs`: 40 errors
- `src\query\context\managers\impl\meta_client_impl.rs`: 40 errors
- `src\query\optimizer\prune_properties_visitor.rs`: 26 errors
- `src\core\expression_utils.rs`: 22 errors
- `src\query\optimizer\optimizer.rs`: 14 errors
- `src\query\optimizer\plan_validator.rs`: 9 errors
- `src\core\expression_visitor.rs`: 6 errors
- `src\query\context\managers\impl\index_manager_impl.rs`: 6 errors
- `src\query\context\runtime_context.rs`: 6 errors
- `src\query\context\request_context.rs`: 6 errors

## Warning Statistics

**Total Warnings**: 63

### Warning Type Breakdown

- **warning**: 63 warnings

### Files with Warnings (Top 10)

- `src\core\result\memory_manager.rs`: 3 warnings
- `src\query\optimizer\plan_validator.rs`: 3 warnings
- `src\query\optimizer\optimizer.rs`: 2 warnings
- `src\query\planner\match_planning\utils\finder.rs`: 2 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 2 warnings
- `src\query\validator\strategies\type_inference.rs`: 2 warnings
- `src\core\expression_utils.rs`: 2 warnings
- `src\query\validator\strategies\variable_validator.rs`: 2 warnings
- `src\query\optimizer\prune_properties_visitor.rs`: 2 warnings
- `src\query\context\request_context.rs`: 2 warnings

## Detailed Error Categorization

### error[E0599]: no method named `get_stats_mut` found for struct `result_processing::traits::BaseResultProcessor` in the current scope: method not found in `BaseResultProcessor<S>`

**Total Occurrences**: 62  
**Unique Files**: 7

#### `src\query\context\managers\impl\storage_client_impl.rs`: 40 occurrences

- Line 63: no method named `scan_all_vertices` found for struct `std::sync::RwLockReadGuard<'_, native_storage::NativeStorage>` in the current scope
- Line 89: no method named `scan_all_vertices` found for struct `std::sync::RwLockReadGuard<'_, native_storage::NativeStorage>` in the current scope
- Line 118: no method named `scan_all_vertices` found for struct `std::sync::RwLockReadGuard<'_, native_storage::NativeStorage>` in the current scope
- ... 37 more occurrences in this file

#### `src\core\expression_utils.rs`: 8 occurrences

- Line 552: no variant or associated item named `Less` found for enum `operators::BinaryOperator` in the current scope: variant or associated item not found in `BinaryOperator`
- Line 553: no variant or associated item named `LessEqual` found for enum `operators::BinaryOperator` in the current scope: variant or associated item not found in `BinaryOperator`
- Line 554: no variant or associated item named `Greater` found for enum `operators::BinaryOperator` in the current scope: variant or associated item not found in `BinaryOperator`
- ... 5 more occurrences in this file

#### `src\query\executor\result_processing\filter.rs`: 3 occurrences

- Line 303: no method named `get_stats_mut` found for struct `result_processing::traits::BaseResultProcessor` in the current scope: method not found in `BaseResultProcessor<S>`
- Line 340: no method named `get_stats` found for struct `result_processing::traits::BaseResultProcessor` in the current scope: method not found in `BaseResultProcessor<S>`
- Line 344: no method named `get_stats_mut` found for struct `result_processing::traits::BaseResultProcessor` in the current scope: method not found in `BaseResultProcessor<S>`

#### `src\query\optimizer\plan_validator.rs`: 3 occurrences

- Line 110: no method named `name` found for reference `&validate::types::Variable` in the current scope: field, not a method
- Line 204: no method named `aggregate_items` found for reference `&aggregate_node::AggregateNode` in the current scope: method not found in `&AggregateNode`
- Line 347: no method named `name` found for reference `&validate::types::Variable` in the current scope: field, not a method

#### `src\query\optimizer\optimizer.rs`: 3 occurrences

- Line 924: no method named `as_aggregate` found for enum `nodes::plan_node_enum::PlanNodeEnum` in the current scope: method not found in `PlanNodeEnum`
- Line 1013: no method named `as_scan_vertices_mut` found for enum `nodes::plan_node_enum::PlanNodeEnum` in the current scope
- Line 1023: no method named `as_index_scan_mut` found for enum `nodes::plan_node_enum::PlanNodeEnum` in the current scope

#### `src\query\context\managers\impl\index_manager_impl.rs`: 3 occurrences

- Line 907: no method named `get_space` found for reference `&std::sync::Arc<dyn storage_engine::StorageEngine>` in the current scope: method not found in `&Arc<dyn StorageEngine>`
- Line 911: no method named `scan_vertices` found for reference `&std::sync::Arc<dyn storage_engine::StorageEngine>` in the current scope
- Line 934: no method named `scan_edges` found for reference `&std::sync::Arc<dyn storage_engine::StorageEngine>` in the current scope

#### `src\query\context\managers\impl\schema_manager_impl.rs`: 2 occurrences

- Line 1125: no method named `save_schema_changes_to_disk` found for reference `&schema_manager_impl::MemorySchemaManager` in the current scope
- Line 1146: no method named `save_schema_changes_to_disk` found for reference `&schema_manager_impl::MemorySchemaManager` in the current scope

### error[E0308]: mismatched types: expected `&Expression`, found `&Box<Expr>`

**Total Occurrences**: 52  
**Unique Files**: 8

#### `src\query\optimizer\prune_properties_visitor.rs`: 20 occurrences

- Line 435: mismatched types: expected `&Expression`, found `&Box<Expr>`
- Line 436: mismatched types: expected `&Expression`, found `&Box<Expr>`
- Line 440: mismatched types: expected `&Expression`, found `&Box<Expr>`
- ... 17 more occurrences in this file

#### `src\query\context\managers\impl\meta_client_impl.rs`: 15 occurrences

- Line 438: mismatched types: expected `meta_client::TagDef`, found `schema_manager::TagDef`
- Line 490: mismatched types: expected `schema_manager::TagDef`, found `meta_client::TagDef`
- Line 514: mismatched types: expected `schema_manager::TagDef`, found `meta_client::TagDef`
- ... 12 more occurrences in this file

#### `src\query\optimizer\optimizer.rs`: 5 occurrences

- Line 133: mismatched types: expected `[String]`, found `Option<_>`
- Line 142: mismatched types: expected `[String]`, found `Option<_>`
- Line 170: mismatched types: expected `[String]`, found `Option<_>`
- ... 2 more occurrences in this file

#### `src\query\optimizer\plan_validator.rs`: 5 occurrences

- Line 197: mismatched types: expected `&Expression`, found `&YieldColumn`
- Line 202: mismatched types: expected `&Expression`, found `&String`
- Line 209: mismatched types: expected `&Expression`, found `&&str`
- ... 2 more occurrences in this file

#### `src\query\context\runtime_context.rs`: 3 occurrences

- Line 630: mismatched types: expected `Option<_>`, found `RwLockReadGuard<'_, Option<SystemTime>>`
- Line 633: mismatched types: expected `SystemTime`, found `RwLockReadGuard<'_, SystemTime>`
- Line 688: mismatched types: expected `&mut HashMap<String, CacheEntry>`, found `RwLockWriteGuard<'_, HashMap<..., ...>>`

#### `src\core\expression_utils.rs`: 2 occurrences

- Line 789: mismatched types: expected `Vec<&Expression>`, found `Vec<&Box<Expression>>`
- Line 797: mismatched types: expected `Vec<&Expression>`, found `Vec<&Box<Expression>>`

#### `src\query\context\managers\impl\index_manager_impl.rs`: 1 occurrences

- Line 1162: mismatched types: expected `&Value`, found `&i64`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 405: mismatched types: expected `&RequestParams`, found `&Arc<RwLock<RequestParams>>`

### error[E0046]: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

**Total Occurrences**: 50  
**Unique Files**: 33

#### `src\query\executor\data_modification.rs`: 5 occurrences

- Line 52: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 156: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 336: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- ... 2 more occurrences in this file

#### `src\query\executor\data_processing\loops.rs`: 4 occurrences

- Line 263: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 385: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 466: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- ... 1 more occurrences in this file

#### `src\query\executor\data_access.rs`: 4 occurrences

- Line 41: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 194: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 277: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- ... 1 more occurrences in this file

#### `src\query\context\execution\query_execution.rs`: 3 occurrences

- Line 462: not all trait items implemented, missing: `add_tag_field`, `drop_tag_field`, `alter_tag_field`, `add_edge_type_field`, `drop_edge_type_field`, `alter_edge_type_field`, `record_schema_change`, `get_schema_changes`, `clear_schema_changes`, `export_schema`, `import_schema`, `validate_schema_compatibility`: missing `add_tag_field`, `drop_tag_field`, `alter_tag_field`, `add_edge_type_field`, `drop_edge_type_field`, `alter_edge_type_field`, `record_schema_change`, `get_schema_changes`, `clear_schema_changes`, `export_schema`, `import_schema`, `validate_schema_compatibility` in implementation
- Line 613: not all trait items implemented, missing: `rebuild_index`, `rebuild_all_indexes`, `get_index_stats`, `get_all_index_stats`, `analyze_index`, `analyze_all_indexes`, `check_index_consistency`, `repair_index`, `cleanup_index`, `batch_insert_vertices`, `batch_delete_vertices`, `batch_insert_edges`, `batch_delete_edges`: missing `rebuild_index`, `rebuild_all_indexes`, `get_index_stats`, `get_all_index_stats`, `analyze_index`, `analyze_all_indexes`, `check_index_consistency`, `repair_index`, `cleanup_index`, `batch_insert_vertices`, `batch_delete_vertices`, `batch_insert_edges`, `batch_delete_edges` in implementation
- Line 951: not all trait items implemented, missing: `create_tag`, `drop_tag`, `get_tag`, `list_tags`, `create_edge_type`, `drop_edge_type`, `get_edge_type`, `list_edge_types`, `get_metadata_version`, `update_metadata_version`: missing `create_tag`, `drop_tag`, `get_tag`, `list_tags`, `create_edge_type`, `drop_edge_type`, `get_edge_type`, `list_edge_types`, `get_metadata_version`, `update_metadata_version` in implementation

#### `src\query\executor\result_processing\aggregation.rs`: 3 occurrences

- Line 646: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 737: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 887: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\projection.rs`: 2 occurrences

- Line 203: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 327: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 416: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 506: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\join\left_join.rs`: 2 occurrences

- Line 210: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 306: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 431: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 77: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 358: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 226: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\join\inner_join.rs`: 1 occurrences

- Line 282: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 898: not all trait items implemented, missing: `add_tag_field`, `drop_tag_field`, `alter_tag_field`, `add_edge_type_field`, `drop_edge_type_field`, `alter_edge_type_field`, `record_schema_change`, `get_schema_changes`, `clear_schema_changes`, `export_schema`, `import_schema`, `validate_schema_compatibility`: missing `add_tag_field`, `drop_tag_field`, `alter_tag_field`, `add_edge_type_field`, `drop_edge_type_field`, `alter_edge_type_field`, `record_schema_change`, `get_schema_changes`, `clear_schema_changes`, `export_schema`, `import_schema`, `validate_schema_compatibility` in implementation

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 390: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\aggregation.rs`: 1 occurrences

- Line 398: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 768: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\cypher\base.rs`: 1 occurrences

- Line 233: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 99: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\optimizer\prune_properties_visitor.rs`: 1 occurrences

- Line 147: not all trait items implemented, missing: `visit_type_casting`, `visit_path_build`, `visit_subscript_range`, `visit_constant_expr`, `state`, `state_mut`: missing `visit_type_casting`, `visit_path_build`, `visit_subscript_range`, `visit_constant_expr`, `state`, `state_mut` in implementation

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 306: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\sort.rs`: 1 occurrences

- Line 553: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 1 occurrences

- Line 244: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 67: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\set_operations\base.rs`: 1 occurrences

- Line 200: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 246: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 occurrences

- Line 299: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 459: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 91: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 369: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 107: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\join\right_join.rs`: 1 occurrences

- Line 162: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\limit.rs`: 1 occurrences

- Line 224: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

### error[E0609]: no field `query` on type `std::sync::Arc<std::sync::RwLock<request_context::RequestParams>>`: unknown field

**Total Occurrences**: 17  
**Unique Files**: 4

#### `src\query\context\request_context.rs`: 5 occurrences

- Line 334: no field `query` on type `std::sync::Arc<std::sync::RwLock<request_context::RequestParams>>`: unknown field
- Line 352: no field `query` on type `std::sync::Arc<std::sync::RwLock<request_context::RequestParams>>`: unknown field
- Line 370: no field `query` on type `std::sync::Arc<std::sync::RwLock<request_context::RequestParams>>`: unknown field
- ... 2 more occurrences in this file

#### `src\query\context\managers\impl\meta_client_impl.rs`: 5 occurrences

- Line 126: no field `properties` on type `&schema_manager::TagDef`: unknown field
- Line 144: no field `edge_name` on type `&schema_manager::EdgeTypeDef`: unknown field
- Line 149: no field `properties` on type `&schema_manager::EdgeTypeDef`: unknown field
- ... 2 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 4 occurrences

- Line 46: no field `id` on type `&MatchedResult`: unknown field
- Line 115: no field `id` on type `&MatchedResult`: unknown field
- Line 185: no field `id` on type `&MatchedResult`: unknown field
- ... 1 more occurrences in this file

#### `src\query\optimizer\prune_properties_visitor.rs`: 3 occurrences

- Line 460: no field `items` on type `&expr::ListExpr`: unknown field
- Line 472: no field `cases` on type `&expr::CaseExpr`: unknown field
- Line 557: no field `items` on type `&expr::PathExpr`: unknown field

### error[E0034]: multiple applicable items in scope: multiple `children` found

**Total Occurrences**: 12  
**Unique Files**: 5

#### `src\core\expression_visitor.rs`: 6 occurrences

- Line 540: multiple applicable items in scope: multiple `children` found
- Line 718: multiple applicable items in scope: multiple `children` found
- Line 729: multiple applicable items in scope: multiple `children` found
- ... 3 more occurrences in this file

#### `src\core\types\expression.rs`: 2 occurrences

- Line 486: multiple applicable items in scope: multiple `children` found
- Line 507: multiple applicable items in scope: multiple `children` found

#### `src\expression\visitor.rs`: 2 occurrences

- Line 264: multiple applicable items in scope: multiple `children` found
- Line 282: multiple applicable items in scope: multiple `children` found

#### `src\query\optimizer\prune_properties_visitor.rs`: 1 occurrences

- Line 202: multiple applicable items in scope: multiple `children` found

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 80: multiple applicable items in scope: multiple `children` found

### error[E0560]: struct `schema_manager::TagDef` has no field named `properties`: `schema_manager::TagDef` does not have this field

**Total Occurrences**: 11  
**Unique Files**: 1

#### `src\query\context\managers\impl\meta_client_impl.rs`: 11 occurrences

- Line 1037: struct `schema_manager::TagDef` has no field named `properties`: `schema_manager::TagDef` does not have this field
- Line 1079: struct `schema_manager::TagDef` has no field named `properties`: `schema_manager::TagDef` does not have this field
- Line 1097: struct `schema_manager::TagDef` has no field named `properties`: `schema_manager::TagDef` does not have this field
- ... 8 more occurrences in this file

### error[E0063]: missing field `version` in initializer of `meta_client::ClusterInfo`: missing `version`

**Total Occurrences**: 9  
**Unique Files**: 3

#### `src\query\optimizer\elimination_rules.rs`: 5 occurrences

- Line 74: missing field `bodies` in initializer of `query::optimizer::optimizer::OptGroupNode`: missing `bodies`
- Line 177: missing field `bodies` in initializer of `query::optimizer::optimizer::OptGroupNode`: missing `bodies`
- Line 365: missing field `bodies` in initializer of `query::optimizer::optimizer::OptGroupNode`: missing `bodies`
- ... 2 more occurrences in this file

#### `src\query\context\execution\query_execution.rs`: 2 occurrences

- Line 953: missing field `version` in initializer of `meta_client::ClusterInfo`: missing `version`
- Line 964: missing fields `edge_types`, `tags` and `version` in initializer of `meta_client::SpaceInfo`: missing `edge_types`, `tags` and `version`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 2 occurrences

- Line 812: missing field `version` in initializer of `meta_client::ClusterInfo`: missing `version`
- Line 965: missing field `version` in initializer of `meta_client::ClusterInfo`: missing `version`

### error[E0053]: method `create_tag` has an incompatible type for trait: expected `meta_client::TagDef`, found `schema_manager::TagDef`

**Total Occurrences**: 6  
**Unique Files**: 1

#### `src\query\context\managers\impl\meta_client_impl.rs`: 6 occurrences

- Line 413: method `create_tag` has an incompatible type for trait: expected `meta_client::TagDef`, found `schema_manager::TagDef`
- Line 474: method `get_tag` has an incompatible type for trait: expected `meta_client::TagDef`, found `schema_manager::TagDef`
- Line 498: method `list_tags` has an incompatible type for trait: expected `meta_client::TagDef`, found `schema_manager::TagDef`
- ... 3 more occurrences in this file

### error[E0369]: cannot add `core::value::types::Value` to `core::value::types::Value`

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\core\expression_utils.rs`: 5 occurrences

- Line 660: cannot add `core::value::types::Value` to `core::value::types::Value`
- Line 661: cannot subtract `core::value::types::Value` from `core::value::types::Value`
- Line 662: cannot multiply `core::value::types::Value` by `core::value::types::Value`
- ... 2 more occurrences in this file

### error[E0412]: cannot find type `PropertyTracker` in module `super`: not found in `super`

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\optimizer\optimizer.rs`: 5 occurrences

- Line 875: cannot find type `PropertyTracker` in module `super`: not found in `super`
- Line 904: cannot find type `PropertyTracker` in module `super`: not found in `super`
- Line 943: cannot find type `PropertyTracker` in module `super`: not found in `super`
- ... 2 more occurrences in this file

### error[E0432]: unresolved imports `super::super::IndexStats`, `super::super::IndexOptimization`: no `IndexStats` in `query::context::managers`, no `IndexOptimization` in `query::context::managers`, help: a similar name exists in the module: `IndexStatus`

**Total Occurrences**: 4  
**Unique Files**: 4

#### `src\query\context\managers\impl\index_manager_impl.rs`: 1 occurrences

- Line 7: unresolved imports `super::super::IndexStats`, `super::super::IndexOptimization`: no `IndexStats` in `query::context::managers`, no `IndexOptimization` in `query::context::managers`, help: a similar name exists in the module: `IndexStatus`

#### `src\query\optimizer\plan_validator.rs`: 1 occurrences

- Line 7: unresolved import `crate::query::optimizer::OptimizerError`: no `OptimizerError` in `query::optimizer`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 4: unresolved imports `super::super::MetadataVersion`, `super::super::PropertyDef`, `super::super::PropertyType`: no `MetadataVersion` in `query::context::managers`, no `PropertyDef` in `query::context::managers`, no `PropertyType` in `query::context::managers`

#### `src\query\context\managers\impl\schema_manager_impl.rs`: 1 occurrences

- Line 4: unresolved imports `super::super::SchemaChange`, `super::super::SchemaChangeType`, `super::super::SchemaExportConfig`, `super::super::SchemaImportResult`: no `SchemaChange` in `query::context::managers`, no `SchemaChangeType` in `query::context::managers`, no `SchemaExportConfig` in `query::context::managers`, no `SchemaImportResult` in `query::context::managers`, help: a similar name exists in the module: `SchemaManager`

### error[E0614]: type `bool` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 4  
**Unique Files**: 1

#### `src\core\expression_utils.rs`: 4 occurrences

- Line 673: type `bool` cannot be dereferenced: can't be dereferenced
- Line 673: type `bool` cannot be dereferenced: can't be dereferenced
- Line 680: type `bool` cannot be dereferenced: can't be dereferenced
- ... 1 more occurrences in this file

### error[E0382]: use of partially moved value: `filter`: value used here after partial move

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\core\expression_utils.rs`: 2 occurrences

- Line 138: use of partially moved value: `filter`: value used here after partial move
- Line 182: use of partially moved value: `filter`: value used here after partial move

### error[E0433]: failed to resolve: could not find `PropertyTracker` in `super`: could not find `PropertyTracker` in `super`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\optimizer.rs`: 1 occurrences

- Line 859: failed to resolve: could not find `PropertyTracker` in `super`: could not find `PropertyTracker` in `super`

### error[E0271]: expected `now` to return `RwLockReadGuard<'_, SystemTime>`, but it returns `SystemTime`: expected `RwLockReadGuard<'_, SystemTime>`, found `SystemTime`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 631: expected `now` to return `RwLockReadGuard<'_, SystemTime>`, but it returns `SystemTime`: expected `RwLockReadGuard<'_, SystemTime>`, found `SystemTime`

### error[E0502]: cannot borrow `*cache` as mutable because it is also borrowed as immutable: mutable borrow occurs here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 756: cannot borrow `*cache` as mutable because it is also borrowed as immutable: mutable borrow occurs here

### error[E0592]: duplicate definitions with name `children`: duplicate definitions for `children`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 761: duplicate definitions with name `children`: duplicate definitions for `children`

### error[E0061]: this method takes 3 arguments but 1 argument was supplied

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\managers\impl\index_manager_impl.rs`: 1 occurrences

- Line 1175: this method takes 3 arguments but 1 argument was supplied

### error[E0407]: method `visit_constant` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\prune_properties_visitor.rs`: 1 occurrences

- Line 426: method `visit_constant` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`

## Detailed Warning Categorization

### warning: unused import: `crate::core::value::NullType`

**Total Occurrences**: 63  
**Unique Files**: 50

#### `src\query\optimizer\plan_validator.rs`: 3 occurrences

- Line 387: unused import: `OptGroup`
- Line 373: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 373: unused variable: `group_id`: help: if this is intentional, prefix it with an underscore: `_group_id`

#### `src\core\result\memory_manager.rs`: 3 occurrences

- Line 444: unexpected `cfg` condition value: `system_monitor`: help: remove the condition
- Line 520: unexpected `cfg` condition value: `system_monitor`: help: remove the condition
- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\core\expression_utils.rs`: 2 occurrences

- Line 5: unused imports: `ExpressionVisitorState` and `ExpressionVisitor`
- Line 8: unused import: `std::collections::HashSet`

#### `src\query\optimizer\prune_properties_visitor.rs`: 2 occurrences

- Line 8: unused import: `crate::query::context::validate::types::Variable`
- Line 562: unused variable: `e`: help: if this is intentional, prefix it with an underscore: `_e`

#### `src\query\optimizer\optimizer.rs`: 2 occurrences

- Line 265: unused variable: `rule`: help: if this is intentional, prefix it with an underscore: `_rule`
- Line 405: unused variable: `rule`: help: if this is intentional, prefix it with an underscore: `_rule`

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 247: unused import: `std::collections::HashMap`
- Line 251: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\planner\ngql\lookup_planner.rs`: 2 occurrences

- Line 119: unused variable: `score_expr`: help: if this is intentional, prefix it with an underscore: `_score_expr`
- Line 284: unused variable: `is_edge`: help: if this is intentional, prefix it with an underscore: `_is_edge`

#### `src\query\context\request_context.rs`: 2 occurrences

- Line 203: unused variable: `now`: help: if this is intentional, prefix it with an underscore: `_now`
- Line 1066: variable does not need to be mutable

#### `src\query\executor\aggregation.rs`: 2 occurrences

- Line 528: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- Line 557: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`

#### `src\query\planner\match_planning\utils\finder.rs`: 2 occurrences

- Line 349: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`
- Line 356: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`

#### `src\query\validator\strategies\type_inference.rs`: 2 occurrences

- Line 77: unused variable: `arg`: help: try ignoring the field: `arg: _`
- Line 568: unused variable: `type_inference`: help: if this is intentional, prefix it with an underscore: `_type_inference`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 945: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 478: unused import: `crate::core::value::NullType`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 525: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 567: unused import: `SortNode`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 488: unused import: `std::sync::Arc`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 351: unused import: `UnaryOperator`

#### `src\query\context\ast\cypher_ast_context.rs`: 1 occurrences

- Line 226: unused variable: `label`: help: if this is intentional, prefix it with an underscore: `_label`

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 537: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\parser\cypher\parser.rs`: 1 occurrences

- Line 257: variable does not need to be mutable

#### `src\query\planner\match_planning\clauses\order_by_planner.rs`: 1 occurrences

- Line 195: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 458: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 193: unused variable: `id_str`: help: if this is intentional, prefix it with an underscore: `_id_str`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 671: unused import: `std::fs`

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 299: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 374: unused variable: `var_name`: help: if this is intentional, prefix it with an underscore: `_var_name`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 247: variable does not need to be mutable

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 568: variable does not need to be mutable

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 802: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 313: unused import: `crate::storage::StorageEngine`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 10: unused import: `crate::expression::ExpressionContext`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 252: unused imports: `Direction` and `Value`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 468: unused import: `DedupNode as Dedup`

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 107: unused variable: `right_col_map`: help: if this is intentional, prefix it with an underscore: `_right_col_map`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 361: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 489: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\sort.rs`: 1 occurrences

- Line 815: variable does not need to be mutable

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 61: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 52: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 9: unused import: `AtomicU64`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

