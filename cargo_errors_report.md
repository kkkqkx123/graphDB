# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 316
- **Total Warnings**: 103
- **Total Issues**: 419
- **Unique Error Patterns**: 157
- **Unique Warning Patterns**: 55
- **Files with Issues**: 102

## Error Statistics

**Total Errors**: 316

### Error Type Breakdown

- **error[E0599]**: 71 errors
- **error[E0308]**: 55 errors
- **error[E0046]**: 49 errors
- **error[E0432]**: 29 errors
- **error[E0277]**: 19 errors
- **error[E0609]**: 17 errors
- **error[E0560]**: 15 errors
- **error[E0412]**: 13 errors
- **error[E0034]**: 12 errors
- **error[E0063]**: 10 errors
- **error[E0053]**: 6 errors
- **error[E0422]**: 5 errors
- **error[E0433]**: 5 errors
- **error[E0382]**: 2 errors
- **error[E0195]**: 2 errors
- **error[E0407]**: 1 errors
- **error[E0271]**: 1 errors
- **error[E0592]**: 1 errors
- **error[E0502]**: 1 errors
- **error[E0061]**: 1 errors
- **error[E0614]**: 1 errors

### Files with Errors (Top 10)

- `src\query\context\managers\impl\storage_client_impl.rs`: 40 errors
- `src\query\context\managers\impl\meta_client_impl.rs`: 40 errors
- `src\query\optimizer\prune_properties_visitor.rs`: 26 errors
- `src\query\planner\statements\mod.rs`: 23 errors
- `src\query\planner\statements\clauses\where_clause_planner.rs`: 19 errors
- `src\query\optimizer\optimizer.rs`: 12 errors
- `src\query\parser\statements\create_impl.rs`: 10 errors
- `src\query\executor\graph_query_executor.rs`: 6 errors
- `src\query\context\managers\impl\index_manager_impl.rs`: 6 errors
- `src\query\context\runtime_context.rs`: 6 errors

## Warning Statistics

**Total Warnings**: 103

### Warning Type Breakdown

- **warning**: 103 warnings

### Files with Warnings (Top 10)

- `src\query\executor\graph_query_executor.rs`: 13 warnings
- `src\query\parser\mod.rs`: 4 warnings
- `src\core\result\memory_manager.rs`: 3 warnings
- `src\query\optimizer\plan_validator.rs`: 3 warnings
- `src\query\optimizer\prune_properties_visitor.rs`: 2 warnings
- `src\query\planner\statements\seeks\scan_seek.rs`: 2 warnings
- `src\query\context\request_context.rs`: 2 warnings
- `src\query\planner\statements\clauses\projection_planner.rs`: 2 warnings
- `src\query\validator\strategies\variable_validator.rs`: 2 warnings
- `src\query\planner\statements\seeks\vertex_seek.rs`: 2 warnings

## Detailed Error Categorization

### error[E0599]: no method named `validate_flow` found for reference `&WithClausePlanner` in the current scope: method not found in `&WithClausePlanner`

**Total Occurrences**: 71  
**Unique Files**: 16

#### `src\query\context\managers\impl\storage_client_impl.rs`: 40 occurrences

- Line 63: no method named `scan_all_vertices` found for struct `std::sync::RwLockReadGuard<'_, native_storage::NativeStorage>` in the current scope
- Line 89: no method named `scan_all_vertices` found for struct `std::sync::RwLockReadGuard<'_, native_storage::NativeStorage>` in the current scope
- Line 118: no method named `scan_all_vertices` found for struct `std::sync::RwLockReadGuard<'_, native_storage::NativeStorage>` in the current scope
- ... 37 more occurrences in this file

#### `src\query\parser\statements\create_impl.rs`: 8 occurrences

- Line 132: no variant named `Space` found for enum `stmt::CreateTarget`: variant not found in `stmt::CreateTarget`
- Line 179: no variant or associated item named `Int64` found for enum `core::types::expression::DataType` in the current scope: variant or associated item not found in `DataType`
- Line 183: no variant or associated item named `Int32` found for enum `core::types::expression::DataType` in the current scope: variant or associated item not found in `DataType`
- ... 5 more occurrences in this file

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 5 occurrences

- Line 78: no method named `validate_flow` found for reference `&where_clause_planner::WhereClausePlanner` in the current scope: method not found in `&WhereClausePlanner`
- Line 88: no function or associated item named `create_filter_node` found for struct `nodes::factory::PlanNodeFactory` in the current scope: function or associated item not found in `PlanNodeFactory`
- Line 93: no method named `add_child` found for reference `&plan_node_enum::PlanNodeEnum` in the current scope: method not found in `&PlanNodeEnum`
- ... 2 more occurrences in this file

#### `src\query\context\managers\impl\index_manager_impl.rs`: 3 occurrences

- Line 907: no method named `get_space` found for reference `&std::sync::Arc<dyn storage_engine::StorageEngine>` in the current scope: method not found in `&Arc<dyn StorageEngine>`
- Line 911: no method named `scan_vertices` found for reference `&std::sync::Arc<dyn storage_engine::StorageEngine>` in the current scope
- Line 934: no method named `scan_edges` found for reference `&std::sync::Arc<dyn storage_engine::StorageEngine>` in the current scope

#### `src\query\executor\result_processing\filter.rs`: 3 occurrences

- Line 303: no method named `get_stats_mut` found for struct `result_processing::traits::BaseResultProcessor` in the current scope: method not found in `BaseResultProcessor<S>`
- Line 340: no method named `get_stats` found for struct `result_processing::traits::BaseResultProcessor` in the current scope: method not found in `BaseResultProcessor<S>`
- Line 344: no method named `get_stats_mut` found for struct `result_processing::traits::BaseResultProcessor` in the current scope: method not found in `BaseResultProcessor<S>`

#### `src\query\context\managers\impl\schema_manager_impl.rs`: 2 occurrences

- Line 1125: no method named `save_schema_changes_to_disk` found for reference `&schema_manager_impl::MemorySchemaManager` in the current scope
- Line 1146: no method named `save_schema_changes_to_disk` found for reference `&schema_manager_impl::MemorySchemaManager` in the current scope

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 32: no method named `validate_flow` found for reference `&WithClausePlanner` in the current scope: method not found in `&WithClausePlanner`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 48: no method named `validate_flow` found for reference `&ReturnClausePlanner` in the current scope: method not found in `&ReturnClausePlanner`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 44: no method named `validate_flow` found for reference `&PaginationPlanner` in the current scope: method not found in `&PaginationPlanner`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 1 occurrences

- Line 44: no method named `validate_flow` found for reference `&YieldClausePlanner` in the current scope: method not found in `&YieldClausePlanner`

#### `src\query\planner\statements\clauses\unwind_planner.rs`: 1 occurrences

- Line 48: no method named `validate_flow` found for reference `&UnwindClausePlanner` in the current scope: method not found in `&UnwindClausePlanner`

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 197: no method named `expect` found for reference `&std::sync::Mutex<S>` in the current scope: method not found in `&Mutex<S>`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 43: no method named `validate_flow` found for reference `&OrderByClausePlanner` in the current scope: method not found in `&OrderByClausePlanner`

#### `src\query\optimizer\optimizer.rs`: 1 occurrences

- Line 924: no method named `as_aggregate` found for enum `plan_node_enum::PlanNodeEnum` in the current scope: method not found in `PlanNodeEnum`

#### `src\query\optimizer\plan_validator.rs`: 1 occurrences

- Line 347: no method named `name` found for reference `&validate::types::Variable` in the current scope: field, not a method

#### `src\query\planner\statements\core\match_clause_planner.rs`: 1 occurrences

- Line 132: no function or associated item named `cartesian_product` found for struct `UnifiedConnector` in the current scope: function or associated item not found in `UnifiedConnector`

### error[E0308]: mismatched types: expected `&RequestParams`, found `&Arc<RwLock<RequestParams>>`

**Total Occurrences**: 55  
**Unique Files**: 11

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

#### `src\query\optimizer\plan_validator.rs`: 4 occurrences

- Line 197: mismatched types: expected `&Expression`, found `&YieldColumn`
- Line 209: mismatched types: expected `&Expression`, found `&&str`
- Line 212: mismatched types: expected `&Expression`, found `&&str`
- ... 1 more occurrences in this file

#### `src\query\context\runtime_context.rs`: 3 occurrences

- Line 630: mismatched types: expected `Option<_>`, found `RwLockReadGuard<'_, Option<SystemTime>>`
- Line 633: mismatched types: expected `SystemTime`, found `RwLockReadGuard<'_, SystemTime>`
- Line 688: mismatched types: expected `&mut HashMap<String, CacheEntry>`, found `RwLockWriteGuard<'_, HashMap<..., ...>>`

#### `src\core\expression_utils.rs`: 2 occurrences

- Line 749: mismatched types: expected `Vec<&Expression>`, found `Vec<&Box<Expression>>`
- Line 757: mismatched types: expected `Vec<&Expression>`, found `Vec<&Box<Expression>>`

#### `src\query\parser\clauses\set_clause_impl.rs`: 2 occurrences

- Line 39: mismatched types: expected `&str`, found `String`
- Line 47: mismatched types: expected `&str`, found `String`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 405: mismatched types: expected `&RequestParams`, found `&Arc<RwLock<RequestParams>>`

#### `src\query\planner\statements\match_planner.rs`: 1 occurrences

- Line 106: mismatched types: expected `Option<Expr>`, found `bool`

#### `src\query\parser\statements\create_impl.rs`: 1 occurrences

- Line 122: mismatched types: expected `Option<Expr>`, found `Vec<PropertyDef>`

#### `src\query\context\managers\impl\index_manager_impl.rs`: 1 occurrences

- Line 1162: mismatched types: expected `&Value`, found `&i64`

### error[E0046]: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

**Total Occurrences**: 49  
**Unique Files**: 32

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

#### `src\query\executor\data_processing\join\left_join.rs`: 2 occurrences

- Line 210: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 306: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\projection.rs`: 2 occurrences

- Line 203: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 327: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 416: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 506: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 246: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 77: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 occurrences

- Line 299: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 358: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 431: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\optimizer\prune_properties_visitor.rs`: 1 occurrences

- Line 147: not all trait items implemented, missing: `visit_type_casting`, `visit_path_build`, `visit_subscript_range`, `visit_constant_expr`, `state`, `state_mut`: missing `visit_type_casting`, `visit_path_build`, `visit_subscript_range`, `visit_constant_expr`, `state`, `state_mut` in implementation

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 91: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 1 occurrences

- Line 244: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 226: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 768: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 898: not all trait items implemented, missing: `add_tag_field`, `drop_tag_field`, `alter_tag_field`, `add_edge_type_field`, `drop_edge_type_field`, `alter_edge_type_field`, `record_schema_change`, `get_schema_changes`, `clear_schema_changes`, `export_schema`, `import_schema`, `validate_schema_compatibility`: missing `add_tag_field`, `drop_tag_field`, `alter_tag_field`, `add_edge_type_field`, `drop_edge_type_field`, `alter_edge_type_field`, `record_schema_change`, `get_schema_changes`, `clear_schema_changes`, `export_schema`, `import_schema`, `validate_schema_compatibility` in implementation

#### `src\query\executor\data_processing\join\right_join.rs`: 1 occurrences

- Line 162: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 99: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 306: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 390: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\limit.rs`: 1 occurrences

- Line 224: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 369: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 67: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 107: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\sort.rs`: 1 occurrences

- Line 553: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\join\inner_join.rs`: 1 occurrences

- Line 282: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 459: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\set_operations\base.rs`: 1 occurrences

- Line 200: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\aggregation.rs`: 1 occurrences

- Line 398: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

### error[E0432]: unresolved import `paths::MatchPathPlanner`: no `MatchPathPlanner` in `query::planner::statements::paths`

**Total Occurrences**: 29  
**Unique Files**: 7

#### `src\query\planner\statements\mod.rs`: 23 occurrences

- Line 34: unresolved import `paths::MatchPathPlanner`: no `MatchPathPlanner` in `query::planner::statements::paths`
- Line 35: unresolved import `paths::ShortestPathPlanner`: no `ShortestPathPlanner` in `query::planner::statements::paths`
- Line 38: unresolved import `seeks::IndexScanMetadata`: no `IndexScanMetadata` in `query::planner::statements::seeks`
- ... 20 more occurrences in this file

#### `src\query\context\managers\impl\schema_manager_impl.rs`: 1 occurrences

- Line 4: unresolved imports `super::super::SchemaChange`, `super::super::SchemaChangeType`, `super::super::SchemaExportConfig`, `super::super::SchemaImportResult`: no `SchemaChange` in `query::context::managers`, no `SchemaChangeType` in `query::context::managers`, no `SchemaExportConfig` in `query::context::managers`, no `SchemaImportResult` in `query::context::managers`, help: a similar name exists in the module: `SchemaManager`

#### `src\core\query_pipeline_manager.rs`: 1 occurrences

- Line 6: unresolved import `crate::query::parser::parser`: could not find `parser` in `parser`

#### `src\query\parser\mod.rs`: 1 occurrences

- Line 21: unresolved import `parser`: help: a similar path exists: `super::parser`

#### `src\query\optimizer\plan_validator.rs`: 1 occurrences

- Line 7: unresolved import `crate::query::optimizer::OptimizerError`: no `OptimizerError` in `query::optimizer`

#### `src\query\context\managers\impl\index_manager_impl.rs`: 1 occurrences

- Line 7: unresolved imports `super::super::IndexStats`, `super::super::IndexOptimization`: no `IndexStats` in `query::context::managers`, no `IndexOptimization` in `query::context::managers`, help: a similar name exists in the module: `IndexStatus`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 4: unresolved imports `super::super::MetadataVersion`, `super::super::PropertyDef`, `super::super::PropertyType`: no `MetadataVersion` in `query::context::managers`, no `PropertyDef` in `query::context::managers`, no `PropertyType` in `query::context::managers`

### error[E0277]: the trait bound `YieldClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `YieldClausePlanner`

**Total Occurrences**: 19  
**Unique Files**: 9

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 6 occurrences

- Line 56: the trait bound `where_clause_planner::WhereClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `where_clause_planner::WhereClausePlanner`
- Line 66: the trait bound `where_clause_planner::WhereClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `where_clause_planner::WhereClausePlanner`
- Line 123: the trait bound `where_clause_planner::WhereClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `where_clause_planner::WhereClausePlanner`
- ... 3 more occurrences in this file

#### `src\query\planner\statements\clauses\yield_planner.rs`: 2 occurrences

- Line 23: the trait bound `YieldClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `YieldClausePlanner`
- Line 33: the trait bound `YieldClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `YieldClausePlanner`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 2 occurrences

- Line 23: the trait bound `PaginationPlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `PaginationPlanner`
- Line 33: the trait bound `PaginationPlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `PaginationPlanner`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 2 occurrences

- Line 22: the trait bound `OrderByClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `OrderByClausePlanner`
- Line 32: the trait bound `OrderByClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `OrderByClausePlanner`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 2 occurrences

- Line 27: the trait bound `ReturnClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `ReturnClausePlanner`
- Line 37: the trait bound `ReturnClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `ReturnClausePlanner`

#### `src\query\planner\statements\clauses\unwind_planner.rs`: 2 occurrences

- Line 27: the trait bound `UnwindClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `UnwindClausePlanner`
- Line 37: the trait bound `UnwindClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `UnwindClausePlanner`

#### `src\query\planner\statements\clauses\clause_planner.rs`: 1 occurrences

- Line 100: the trait bound `BaseClausePlanner: cypher_clause_planner::CypherClausePlanner` is not satisfied: the trait `cypher_clause_planner::CypherClausePlanner` is not implemented for `BaseClausePlanner`

#### `src\query\planner\statements\clauses\projection_planner.rs`: 1 occurrences

- Line 20: the trait bound `ProjectionPlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `ProjectionPlanner`

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 21: the trait bound `WithClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `WithClausePlanner`

### error[E0609]: no field `id` on type `&MatchedResult`: unknown field

**Total Occurrences**: 17  
**Unique Files**: 4

#### `src\query\context\managers\impl\meta_client_impl.rs`: 5 occurrences

- Line 126: no field `properties` on type `&schema_manager::TagDef`: unknown field
- Line 144: no field `edge_name` on type `&schema_manager::EdgeTypeDef`: unknown field
- Line 149: no field `properties` on type `&schema_manager::EdgeTypeDef`: unknown field
- ... 2 more occurrences in this file

#### `src\query\context\request_context.rs`: 5 occurrences

- Line 334: no field `query` on type `std::sync::Arc<std::sync::RwLock<request_context::RequestParams>>`: unknown field
- Line 352: no field `query` on type `std::sync::Arc<std::sync::RwLock<request_context::RequestParams>>`: unknown field
- Line 370: no field `query` on type `std::sync::Arc<std::sync::RwLock<request_context::RequestParams>>`: unknown field
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

### error[E0560]: struct `schema_manager::TagDef` has no field named `properties`: `schema_manager::TagDef` does not have this field

**Total Occurrences**: 15  
**Unique Files**: 2

#### `src\query\context\managers\impl\meta_client_impl.rs`: 11 occurrences

- Line 1037: struct `schema_manager::TagDef` has no field named `properties`: `schema_manager::TagDef` does not have this field
- Line 1079: struct `schema_manager::TagDef` has no field named `properties`: `schema_manager::TagDef` does not have this field
- Line 1097: struct `schema_manager::TagDef` has no field named `properties`: `schema_manager::TagDef` does not have this field
- ... 8 more occurrences in this file

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 4 occurrences

- Line 138: struct `clause_structs::WhereClauseContext` has no field named `is_optional`: `clause_structs::WhereClauseContext` does not have this field
- Line 149: struct `clause_structs::ReturnClauseContext` has no field named `return_items`: `clause_structs::ReturnClauseContext` does not have this field
- Line 163: struct `clause_structs::WhereClauseContext` has no field named `is_optional`: `clause_structs::WhereClauseContext` does not have this field
- ... 1 more occurrences in this file

### error[E0412]: cannot find type `PropertyTracker` in this scope: not found in this scope

**Total Occurrences**: 13  
**Unique Files**: 5

#### `src\query\optimizer\optimizer.rs`: 5 occurrences

- Line 875: cannot find type `PropertyTracker` in this scope: not found in this scope
- Line 904: cannot find type `PropertyTracker` in this scope: not found in this scope
- Line 943: cannot find type `PropertyTracker` in this scope: not found in this scope
- ... 2 more occurrences in this file

#### `src\query\executor\graph_query_executor.rs`: 3 occurrences

- Line 155: cannot find type `DBResult` in this scope
- Line 171: cannot find type `DBResult` in this scope
- Line 176: cannot find type `DBResult` in this scope

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 2 occurrences

- Line 13: cannot find type `OrderByItem` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 17: cannot find type `OrderByItem` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`

#### `src\query\parser\clauses\skip_limit_impl.rs`: 2 occurrences

- Line 8: cannot find type `SkipClause` in this scope
- Line 19: cannot find type `LimitClause` in this scope: not found in this scope

#### `src\query\parser\clauses\where_clause_impl.rs`: 1 occurrences

- Line 8: cannot find type `WhereClause` in this scope

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

### error[E0063]: missing field `version` in initializer of `meta_client::ClusterInfo`: missing `version`

**Total Occurrences**: 10  
**Unique Files**: 4

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

#### `src\query\parser\statements\create_impl.rs`: 1 occurrences

- Line 120: missing fields `direction`, `dst`, `src` and 1 other field in initializer of `stmt::CreateTarget`: missing `direction`, `dst`, `src` and 1 other field

### error[E0053]: method `create_tag` has an incompatible type for trait: expected `meta_client::TagDef`, found `schema_manager::TagDef`

**Total Occurrences**: 6  
**Unique Files**: 1

#### `src\query\context\managers\impl\meta_client_impl.rs`: 6 occurrences

- Line 413: method `create_tag` has an incompatible type for trait: expected `meta_client::TagDef`, found `schema_manager::TagDef`
- Line 474: method `get_tag` has an incompatible type for trait: expected `meta_client::TagDef`, found `schema_manager::TagDef`
- Line 498: method `list_tags` has an incompatible type for trait: expected `meta_client::TagDef`, found `schema_manager::TagDef`
- ... 3 more occurrences in this file

### error[E0422]: cannot find struct, variant or union type `QueryInfo` in this scope: not found in this scope

**Total Occurrences**: 5  
**Unique Files**: 3

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 2 occurrences

- Line 166: cannot find struct, variant or union type `QueryInfo` in this scope: not found in this scope
- Line 183: cannot find struct, variant or union type `QueryInfo` in this scope: not found in this scope

#### `src\query\parser\clauses\skip_limit_impl.rs`: 2 occurrences

- Line 13: cannot find struct, variant or union type `SkipClause` in this scope
- Line 24: cannot find struct, variant or union type `LimitClause` in this scope: not found in this scope

#### `src\query\parser\clauses\where_clause_impl.rs`: 1 occurrences

- Line 13: cannot find struct, variant or union type `WhereClause` in this scope

### error[E0433]: failed to resolve: could not find `Literal` in `core`: could not find `Literal` in `core`

**Total Occurrences**: 5  
**Unique Files**: 3

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 2 occurrences

- Line 115: failed to resolve: could not find `Literal` in `core`: could not find `Literal` in `core`
- Line 129: failed to resolve: use of undeclared type `FlowDirection`: use of undeclared type `FlowDirection`

#### `src\query\planner\statements\core\match_clause_planner.rs`: 2 occurrences

- Line 222: failed to resolve: could not find `match_planning` in `planner`: could not find `match_planning` in `planner`
- Line 261: failed to resolve: could not find `match_planning` in `planner`: could not find `match_planning` in `planner`

#### `src\query\optimizer\optimizer.rs`: 1 occurrences

- Line 859: failed to resolve: use of undeclared type `PropertyTracker`: use of undeclared type `PropertyTracker`

### error[E0195]: lifetime parameters or bounds on method `open` do not match the trait declaration: lifetimes do not match method in trait

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 171: lifetime parameters or bounds on method `open` do not match the trait declaration: lifetimes do not match method in trait
- Line 176: lifetime parameters or bounds on method `close` do not match the trait declaration: lifetimes do not match method in trait

### error[E0382]: use of partially moved value: `object`: value used here after partial move

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\core\expression_utils.rs`: 2 occurrences

- Line 137: use of partially moved value: `object`: value used here after partial move
- Line 181: use of partially moved value: `object`: value used here after partial move

### error[E0271]: expected `now` to return `RwLockReadGuard<'_, SystemTime>`, but it returns `SystemTime`: expected `RwLockReadGuard<'_, SystemTime>`, found `SystemTime`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 631: expected `now` to return `RwLockReadGuard<'_, SystemTime>`, but it returns `SystemTime`: expected `RwLockReadGuard<'_, SystemTime>`, found `SystemTime`

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

### error[E0502]: cannot borrow `*cache` as mutable because it is also borrowed as immutable: mutable borrow occurs here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 756: cannot borrow `*cache` as mutable because it is also borrowed as immutable: mutable borrow occurs here

### error[E0592]: duplicate definitions with name `children`: duplicate definitions for `children`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 721: duplicate definitions with name `children`: duplicate definitions for `children`

### error[E0614]: type `i64` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\parser\statements\go_impl.rs`: 1 occurrences

- Line 65: type `i64` cannot be dereferenced: can't be dereferenced

## Detailed Warning Categorization

### warning: unused import: `crate::expression::ExpressionContext`

**Total Occurrences**: 103  
**Unique Files**: 66

#### `src\query\executor\graph_query_executor.rs`: 13 occurrences

- Line 100: unused variable: `clause`: help: if this is intentional, prefix it with an underscore: `_clause`
- Line 104: unused variable: `clause`: help: if this is intentional, prefix it with an underscore: `_clause`
- Line 108: unused variable: `clause`: help: if this is intentional, prefix it with an underscore: `_clause`
- ... 10 more occurrences in this file

#### `src\query\parser\mod.rs`: 4 occurrences

- Line 17: ambiguous glob re-exports: the name `SetParser` in the type namespace is first re-exported here
- Line 17: ambiguous glob re-exports: the name `ReturnParser` in the type namespace is first re-exported here
- Line 17: ambiguous glob re-exports: the name `WithParser` in the type namespace is first re-exported here
- ... 1 more occurrences in this file

#### `src\query\optimizer\plan_validator.rs`: 3 occurrences

- Line 387: unused import: `OptGroup`
- Line 373: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 373: unused variable: `group_id`: help: if this is intentional, prefix it with an underscore: `_group_id`

#### `src\core\result\memory_manager.rs`: 3 occurrences

- Line 444: unexpected `cfg` condition value: `system_monitor`: help: remove the condition
- Line 520: unexpected `cfg` condition value: `system_monitor`: help: remove the condition
- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\query\validator\strategies\type_inference.rs`: 2 occurrences

- Line 77: unused variable: `arg`: help: try ignoring the field: `arg: _`
- Line 568: unused variable: `type_inference`: help: if this is intentional, prefix it with an underscore: `_type_inference`

#### `src\query\planner\statements\seeks\index_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\executor\aggregation.rs`: 2 occurrences

- Line 528: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- Line 557: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`

#### `src\query\planner\statements\paths\match_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\vertex_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\utils\connection_strategy.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\seek_strategy.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\optimizer\prune_properties_visitor.rs`: 2 occurrences

- Line 8: unused import: `crate::query::context::validate::types::Variable`
- Line 562: unused variable: `e`: help: if this is intentional, prefix it with an underscore: `_e`

#### `src\query\planner\statements\utils\finder.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\context\request_context.rs`: 2 occurrences

- Line 203: unused variable: `now`: help: if this is intentional, prefix it with an underscore: `_now`
- Line 1066: variable does not need to be mutable

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 247: unused import: `std::collections::HashMap`
- Line 251: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 2 occurrences

- Line 23: unused import: `crate::query::planner::statements::paths::match_path_planner::MatchPathPlanner`
- Line 24: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\query\planner\statements\utils\connection_builder.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\lookup_planner.rs`: 2 occurrences

- Line 119: unused variable: `score_expr`: help: if this is intentional, prefix it with an underscore: `_score_expr`
- Line 284: unused variable: `is_edge`: help: if this is intentional, prefix it with an underscore: `_is_edge`

#### `src\query\planner\statements\clauses\projection_planner.rs`: 2 occurrences

- Line 3: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`
- Line 7: unused import: `crate::query::validator::structs::CypherClauseKind`

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\optimizer\optimizer.rs`: 2 occurrences

- Line 265: unused variable: `rule`: help: if this is intentional, prefix it with an underscore: `_rule`
- Line 405: unused variable: `rule`: help: if this is intentional, prefix it with an underscore: `_rule`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 10: unused import: `crate::expression::ExpressionContext`

#### `src\query\planner\statements\subgraph_planner.rs`: 1 occurrences

- Line 52: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\parser\clauses\skip_limit_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 9: unused import: `AtomicU64`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 107: unused variable: `right_col_map`: help: if this is intentional, prefix it with an underscore: `_right_col_map`

#### `src\query\planner\statements\go_planner.rs`: 1 occurrences

- Line 61: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 567: unused import: `SortNode`

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 537: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 313: unused import: `crate::storage::StorageEngine`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 4: unused import: `QueryInfo`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 8: unused import: `crate::query::validator::structs::CypherClauseKind`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 489: unused import: `crate::core::value::NullType`

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 7: unused import: `std::collections::HashSet`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 247: variable does not need to be mutable

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 351: unused import: `UnaryOperator`

#### `src\query\executor\result_processing\sort.rs`: 1 occurrences

- Line 815: variable does not need to be mutable

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 458: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 671: unused import: `std::fs`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 478: unused import: `crate::core::value::NullType`

#### `src\query\planner\statements\clauses\unwind_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 945: unused import: `crate::core::value::NullType`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 568: variable does not need to be mutable

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 4: unused import: `QueryInfo`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 252: unused imports: `Direction` and `Value`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 193: unused variable: `id_str`: help: if this is intentional, prefix it with an underscore: `_id_str`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\planner\statements\clauses\clause_planner.rs`: 1 occurrences

- Line 5: unused import: `DataFlowNode`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 525: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 468: unused import: `DedupNode as Dedup`

#### `src\query\context\ast\cypher_ast_context.rs`: 1 occurrences

- Line 226: unused variable: `label`: help: if this is intentional, prefix it with an underscore: `_label`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 374: unused variable: `var_name`: help: if this is intentional, prefix it with an underscore: `_var_name`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 361: unused import: `crate::core::value::NullType`

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 802: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`

#### `src\query\parser\clauses\where_clause_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

