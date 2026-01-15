# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 99
- **Total Warnings**: 99
- **Total Issues**: 198
- **Unique Error Patterns**: 55
- **Unique Warning Patterns**: 55
- **Files with Issues**: 80

## Error Statistics

**Total Errors**: 99

### Error Type Breakdown

- **error[E0034]**: 12 errors
- **error[E0308]**: 11 errors
- **error[E0046]**: 10 errors
- **error[E0053]**: 8 errors
- **error[E0412]**: 8 errors
- **error[E0609]**: 8 errors
- **error[E0277]**: 7 errors
- **error[E0063]**: 7 errors
- **error[E0599]**: 5 errors
- **error[E0432]**: 4 errors
- **error[E0596]**: 3 errors
- **error[E0422]**: 3 errors
- **error[E0382]**: 2 errors
- **error[E0195]**: 2 errors
- **error[E0061]**: 1 errors
- **error[E0614]**: 1 errors
- **error[E0502]**: 1 errors
- **error[E0505]**: 1 errors
- **error[E0283]**: 1 errors
- **error[E0004]**: 1 errors
- **error[E0515]**: 1 errors
- **error[E0282]**: 1 errors
- **error[E0592]**: 1 errors

### Files with Errors (Top 10)

- `src\query\context\execution\query_execution.rs`: 9 errors
- `src\query\planner\statements\clauses\where_clause_planner.rs`: 8 errors
- `src\query\context\runtime_context.rs`: 7 errors
- `src\core\expression_visitor.rs`: 6 errors
- `src\query\optimizer\plan_validator.rs`: 6 errors
- `src\core\expression_utils.rs`: 5 errors
- `src\query\optimizer\elimination_rules.rs`: 5 errors
- `src\query\executor\graph_query_executor.rs`: 5 errors
- `src\query\context\request_context.rs`: 5 errors
- `src\query\optimizer\predicate_pushdown.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 99

### Warning Type Breakdown

- **warning**: 99 warnings

### Files with Warnings (Top 10)

- `src\query\executor\graph_query_executor.rs`: 13 warnings
- `src\query\parser\mod.rs`: 4 warnings
- `src\core\result\memory_manager.rs`: 3 warnings
- `src\query\optimizer\plan_validator.rs`: 3 warnings
- `src\query\validator\strategies\type_inference.rs`: 2 warnings
- `src\query\planner\statements\seeks\index_seek.rs`: 2 warnings
- `src\query\planner\statements\seeks\vertex_seek.rs`: 2 warnings
- `src\query\planner\statements\paths\match_path_planner.rs`: 2 warnings
- `src\query\validator\strategies\variable_validator.rs`: 2 warnings
- `src\query\planner\statements\seeks\scan_seek.rs`: 2 warnings

## Detailed Error Categorization

### error[E0034]: multiple applicable items in scope: multiple `children` found

**Total Occurrences**: 12  
**Unique Files**: 5

#### `src\core\expression_visitor.rs`: 6 occurrences

- Line 540: multiple applicable items in scope: multiple `children` found
- Line 718: multiple applicable items in scope: multiple `children` found
- Line 729: multiple applicable items in scope: multiple `children` found
- ... 3 more occurrences in this file

#### `src\core\types\expression.rs`: 2 occurrences

- Line 491: multiple applicable items in scope: multiple `children` found
- Line 512: multiple applicable items in scope: multiple `children` found

#### `src\expression\visitor.rs`: 2 occurrences

- Line 264: multiple applicable items in scope: multiple `children` found
- Line 282: multiple applicable items in scope: multiple `children` found

#### `src\query\optimizer\prune_properties_visitor.rs`: 1 occurrences

- Line 202: multiple applicable items in scope: multiple `children` found

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 80: multiple applicable items in scope: multiple `children` found

### error[E0308]: mismatched types: expected `&Expression`, found `&YieldColumn`

**Total Occurrences**: 11  
**Unique Files**: 6

#### `src\query\optimizer\plan_validator.rs`: 4 occurrences

- Line 197: mismatched types: expected `&Expression`, found `&YieldColumn`
- Line 209: mismatched types: expected `&Expression`, found `&&str`
- Line 212: mismatched types: expected `&Expression`, found `&&str`
- ... 1 more occurrences in this file

#### `src\core\expression_utils.rs`: 2 occurrences

- Line 749: mismatched types: expected `Vec<&Expression>`, found `Vec<&Box<Expression>>`
- Line 757: mismatched types: expected `Vec<&Expression>`, found `Vec<&Box<Expression>>`

#### `src\query\optimizer\optimizer.rs`: 2 occurrences

- Line 910: mismatched types: expected `&Expression`, found `&YieldColumn`
- Line 924: mismatched types: expected `&Expression`, found `&String`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 688: mismatched types: expected `&mut HashMap<String, CacheEntry>`, found `RwLockWriteGuard<'_, HashMap<..., ...>>`

#### `src\query\planner\statements\match_planner.rs`: 1 occurrences

- Line 106: mismatched types: expected `Option<Expr>`, found `bool`

#### `src\query\context\managers\impl\index_manager_impl.rs`: 1 occurrences

- Line 1156: mismatched types: expected `&Value`, found `&i64`

### error[E0046]: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

**Total Occurrences**: 10  
**Unique Files**: 7

#### `src\query\context\execution\query_execution.rs`: 3 occurrences

- Line 462: not all trait items implemented, missing: `add_tag_field`, `drop_tag_field`, `alter_tag_field`, `add_edge_type_field`, `drop_edge_type_field`, `alter_edge_type_field`, `record_schema_change`, `get_schema_changes`, `clear_schema_changes`, `export_schema`, `import_schema`, `validate_schema_compatibility`: missing `add_tag_field`, `drop_tag_field`, `alter_tag_field`, `add_edge_type_field`, `drop_edge_type_field`, `alter_edge_type_field`, `record_schema_change`, `get_schema_changes`, `clear_schema_changes`, `export_schema`, `import_schema`, `validate_schema_compatibility` in implementation
- Line 613: not all trait items implemented, missing: `rebuild_index`, `rebuild_all_indexes`, `get_index_stats`, `get_all_index_stats`, `analyze_index`, `analyze_all_indexes`, `check_index_consistency`, `repair_index`, `cleanup_index`, `batch_insert_vertices`, `batch_delete_vertices`, `batch_insert_edges`, `batch_delete_edges`: missing `rebuild_index`, `rebuild_all_indexes`, `get_index_stats`, `get_all_index_stats`, `analyze_index`, `analyze_all_indexes`, `check_index_consistency`, `repair_index`, `cleanup_index`, `batch_insert_vertices`, `batch_delete_vertices`, `batch_insert_edges`, `batch_delete_edges` in implementation
- Line 951: not all trait items implemented, missing: `create_tag`, `drop_tag`, `get_tag`, `list_tags`, `create_edge_type`, `drop_edge_type`, `get_edge_type`, `list_edge_types`, `get_metadata_version`, `update_metadata_version`: missing `create_tag`, `drop_tag`, `get_tag`, `list_tags`, `create_edge_type`, `drop_edge_type`, `get_edge_type`, `list_edge_types`, `get_metadata_version`, `update_metadata_version` in implementation

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 744: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation
- Line 893: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 335: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 572: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 514: not all trait items implemented, missing: `stats`, `stats_mut`: missing `stats`, `stats_mut` in implementation

#### `src\query\optimizer\prune_properties_visitor.rs`: 1 occurrences

- Line 147: not all trait items implemented, missing: `visit_range`, `state`, `state_mut`: missing `visit_range`, `state`, `state_mut` in implementation

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 898: not all trait items implemented, missing: `add_tag_field`, `drop_tag_field`, `alter_tag_field`, `add_edge_type_field`, `drop_edge_type_field`, `alter_edge_type_field`, `record_schema_change`, `get_schema_changes`, `clear_schema_changes`, `export_schema`, `import_schema`, `validate_schema_compatibility`: missing `add_tag_field`, `drop_tag_field`, `alter_tag_field`, `add_edge_type_field`, `drop_edge_type_field`, `alter_edge_type_field`, `record_schema_change`, `get_schema_changes`, `clear_schema_changes`, `export_schema`, `import_schema`, `validate_schema_compatibility` in implementation

### error[E0412]: cannot find type `SkipClause` in this scope

**Total Occurrences**: 8  
**Unique Files**: 4

#### `src\query\executor\graph_query_executor.rs`: 3 occurrences

- Line 155: cannot find type `DBResult` in this scope
- Line 171: cannot find type `DBResult` in this scope
- Line 176: cannot find type `DBResult` in this scope

#### `src\query\parser\clauses\skip_limit_impl.rs`: 2 occurrences

- Line 8: cannot find type `SkipClause` in this scope
- Line 19: cannot find type `LimitClause` in this scope: not found in this scope

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 2 occurrences

- Line 13: cannot find type `OrderByItem` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 17: cannot find type `OrderByItem` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`

#### `src\query\parser\clauses\where_clause_impl.rs`: 1 occurrences

- Line 8: cannot find type `WhereClause` in this scope

### error[E0609]: no field `query` on type `std::sync::Arc<std::sync::RwLock<request_context::RequestParams>>`: unknown field

**Total Occurrences**: 8  
**Unique Files**: 2

#### `src\query\context\request_context.rs`: 4 occurrences

- Line 334: no field `query` on type `std::sync::Arc<std::sync::RwLock<request_context::RequestParams>>`: unknown field
- Line 352: no field `query` on type `std::sync::Arc<std::sync::RwLock<request_context::RequestParams>>`: unknown field
- Line 370: no field `query` on type `std::sync::Arc<std::sync::RwLock<request_context::RequestParams>>`: unknown field
- ... 1 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 4 occurrences

- Line 46: no field `id` on type `&MatchedResult`: unknown field
- Line 115: no field `id` on type `&MatchedResult`: unknown field
- Line 185: no field `id` on type `&MatchedResult`: unknown field
- ... 1 more occurrences in this file

### error[E0053]: method `get_tag` has an incompatible type for trait: expected `TagDefWithId`, found `managers::types::TagDef`

**Total Occurrences**: 8  
**Unique Files**: 2

#### `src\query\context\execution\query_execution.rs`: 4 occurrences

- Line 492: method `get_tag` has an incompatible type for trait: expected `TagDefWithId`, found `managers::types::TagDef`
- Line 499: method `list_tags` has an incompatible type for trait: expected `TagDefWithId`, found `managers::types::TagDef`
- Line 524: method `get_edge_type` has an incompatible type for trait: expected `EdgeTypeDefWithId`, found `managers::types::EdgeTypeDef`
- ... 1 more occurrences in this file

#### `src\query\context\runtime_context.rs`: 4 occurrences

- Line 928: method `get_tag` has an incompatible type for trait: expected `TagDefWithId`, found `managers::types::TagDef`
- Line 935: method `list_tags` has an incompatible type for trait: expected `TagDefWithId`, found `managers::types::TagDef`
- Line 960: method `get_edge_type` has an incompatible type for trait: expected `EdgeTypeDefWithId`, found `managers::types::EdgeTypeDef`
- ... 1 more occurrences in this file

### error[E0277]: the trait bound `BaseClausePlanner: cypher_clause_planner::CypherClausePlanner` is not satisfied: the trait `cypher_clause_planner::CypherClausePlanner` is not implemented for `BaseClausePlanner`

**Total Occurrences**: 7  
**Unique Files**: 2

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 6 occurrences

- Line 55: the trait bound `where_clause_planner::WhereClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `where_clause_planner::WhereClausePlanner`
- Line 65: the trait bound `where_clause_planner::WhereClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `where_clause_planner::WhereClausePlanner`
- Line 111: the trait bound `where_clause_planner::WhereClausePlanner: cypher_clause_planner::DataFlowNode` is not satisfied: the trait `cypher_clause_planner::DataFlowNode` is not implemented for `where_clause_planner::WhereClausePlanner`
- ... 3 more occurrences in this file

#### `src\query\planner\statements\clauses\clause_planner.rs`: 1 occurrences

- Line 100: the trait bound `BaseClausePlanner: cypher_clause_planner::CypherClausePlanner` is not satisfied: the trait `cypher_clause_planner::CypherClausePlanner` is not implemented for `BaseClausePlanner`

### error[E0063]: missing field `bodies` in initializer of `query::optimizer::optimizer::OptGroupNode`: missing `bodies`

**Total Occurrences**: 7  
**Unique Files**: 2

#### `src\query\optimizer\elimination_rules.rs`: 5 occurrences

- Line 74: missing field `bodies` in initializer of `query::optimizer::optimizer::OptGroupNode`: missing `bodies`
- Line 177: missing field `bodies` in initializer of `query::optimizer::optimizer::OptGroupNode`: missing `bodies`
- Line 365: missing field `bodies` in initializer of `query::optimizer::optimizer::OptGroupNode`: missing `bodies`
- ... 2 more occurrences in this file

#### `src\query\context\execution\query_execution.rs`: 2 occurrences

- Line 953: missing field `version` in initializer of `ClusterInfo`: missing `version`
- Line 964: missing fields `edge_types`, `tags` and `version` in initializer of `managers::types::SpaceInfo`: missing `edge_types`, `tags` and `version`

### error[E0599]: no variant or associated item named `Timestamp` found for enum `core::types::expression::DataType` in the current scope: variant or associated item not found in `DataType`

**Total Occurrences**: 5  
**Unique Files**: 3

#### `src\query\parser\statements\create_impl.rs`: 2 occurrences

- Line 203: no variant or associated item named `Timestamp` found for enum `core::types::expression::DataType` in the current scope: variant or associated item not found in `DataType`
- Line 215: no variant or associated item named `Datetime` found for enum `core::types::expression::DataType` in the current scope: variant or associated item not found in `DataType`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 2 occurrences

- Line 76: no method named `validate_flow` found for reference `&where_clause_planner::WhereClausePlanner` in the current scope: method not found in `&WhereClausePlanner`
- Line 118: no method named `requires_input` found for struct `where_clause_planner::WhereClausePlanner` in the current scope: method not found in `WhereClausePlanner`

#### `src\query\optimizer\plan_validator.rs`: 1 occurrences

- Line 347: no method named `name` found for reference `&validate::types::Variable` in the current scope: field, not a method

### error[E0432]: unresolved import `parser`: help: a similar path exists: `super::parser`

**Total Occurrences**: 4  
**Unique Files**: 4

#### `src\query\parser\mod.rs`: 1 occurrences

- Line 21: unresolved import `parser`: help: a similar path exists: `super::parser`

#### `src\query\context\managers\impl\index_manager_impl.rs`: 1 occurrences

- Line 7: unresolved imports `super::super::IndexStats`, `super::super::IndexOptimization`: no `IndexStats` in `query::context::managers`, no `IndexOptimization` in `query::context::managers`, help: a similar name exists in the module: `IndexStatus`

#### `src\query\optimizer\plan_validator.rs`: 1 occurrences

- Line 7: unresolved import `crate::query::optimizer::OptimizerError`: no `OptimizerError` in `query::optimizer`

#### `src\core\query_pipeline_manager.rs`: 1 occurrences

- Line 6: unresolved import `crate::query::parser::parser`: could not find `parser` in `parser`

### error[E0422]: cannot find struct, variant or union type `WhereClause` in this scope

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\query\parser\clauses\skip_limit_impl.rs`: 2 occurrences

- Line 13: cannot find struct, variant or union type `SkipClause` in this scope
- Line 24: cannot find struct, variant or union type `LimitClause` in this scope: not found in this scope

#### `src\query\parser\clauses\where_clause_impl.rs`: 1 occurrences

- Line 13: cannot find struct, variant or union type `WhereClause` in this scope

### error[E0596]: cannot borrow data in dereference of `std::sync::RwLockReadGuard<'_, native_storage::NativeStorage>` as mutable: cannot borrow as mutable

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\context\managers\impl\storage_client_impl.rs`: 3 occurrences

- Line 298: cannot borrow data in dereference of `std::sync::RwLockReadGuard<'_, native_storage::NativeStorage>` as mutable: cannot borrow as mutable
- Line 322: cannot borrow data in dereference of `std::sync::RwLockReadGuard<'_, native_storage::NativeStorage>` as mutable: cannot borrow as mutable
- Line 325: cannot borrow data in dereference of `std::sync::RwLockReadGuard<'_, native_storage::NativeStorage>` as mutable: cannot borrow as mutable

### error[E0382]: use of partially moved value: `object`: value used here after partial move

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\core\expression_utils.rs`: 2 occurrences

- Line 137: use of partially moved value: `object`: value used here after partial move
- Line 181: use of partially moved value: `object`: value used here after partial move

### error[E0195]: lifetime parameters or bounds on method `open` do not match the trait declaration: lifetimes do not match method in trait

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 171: lifetime parameters or bounds on method `open` do not match the trait declaration: lifetimes do not match method in trait
- Line 176: lifetime parameters or bounds on method `close` do not match the trait declaration: lifetimes do not match method in trait

### error[E0061]: this method takes 3 arguments but 1 argument was supplied

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\managers\impl\index_manager_impl.rs`: 1 occurrences

- Line 1169: this method takes 3 arguments but 1 argument was supplied

### error[E0592]: duplicate definitions with name `children`: duplicate definitions for `children`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 721: duplicate definitions with name `children`: duplicate definitions for `children`

### error[E0282]: type annotations needed: cannot infer type

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 628: type annotations needed: cannot infer type

### error[E0283]: type annotations needed: cannot infer type

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 628: type annotations needed: cannot infer type

### error[E0614]: type `i64` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\parser\statements\go_impl.rs`: 1 occurrences

- Line 65: type `i64` cannot be dereferenced: can't be dereferenced

### error[E0502]: cannot borrow `*cache` as mutable because it is also borrowed as immutable: mutable borrow occurs here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 756: cannot borrow `*cache` as mutable because it is also borrowed as immutable: mutable borrow occurs here

### error[E0505]: cannot move out of `indexes` because it is borrowed: move out of `indexes` occurs here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\managers\impl\index_manager_impl.rs`: 1 occurrences

- Line 890: cannot move out of `indexes` because it is borrowed: move out of `indexes` occurs here

### error[E0004]: non-exhaustive patterns: `&core::types::expression::DataType::Int8`, `&core::types::expression::DataType::Int16`, `&core::types::expression::DataType::Int32` and 2 more not covered: patterns `&core::types::expression::DataType::Int8`, `&core::types::expression::DataType::Int16`, `&core::types::expression::DataType::Int32` and 2 more not covered

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 456: non-exhaustive patterns: `&core::types::expression::DataType::Int8`, `&core::types::expression::DataType::Int16`, `&core::types::expression::DataType::Int32` and 2 more not covered: patterns `&core::types::expression::DataType::Int8`, `&core::types::expression::DataType::Int16`, `&core::types::expression::DataType::Int32` and 2 more not covered

### error[E0515]: cannot return value referencing function parameter `p`: returns a value referencing data owned by the current function

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 401: cannot return value referencing function parameter `p`: returns a value referencing data owned by the current function

## Detailed Warning Categorization

### warning: unused import: `crate::query::context::validate::types::Variable`

**Total Occurrences**: 99  
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

#### `src\core\result\memory_manager.rs`: 3 occurrences

- Line 444: unexpected `cfg` condition value: `system_monitor`: help: remove the condition
- Line 520: unexpected `cfg` condition value: `system_monitor`: help: remove the condition
- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\query\optimizer\plan_validator.rs`: 3 occurrences

- Line 387: unused import: `OptGroup`
- Line 373: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 373: unused variable: `group_id`: help: if this is intentional, prefix it with an underscore: `_group_id`

#### `src\query\executor\aggregation.rs`: 2 occurrences

- Line 536: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- Line 565: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`

#### `src\query\context\request_context.rs`: 2 occurrences

- Line 203: unused variable: `now`: help: if this is intentional, prefix it with an underscore: `_now`
- Line 1068: variable does not need to be mutable

#### `src\query\planner\statements\seeks\vertex_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\utils\connection_builder.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\lookup_planner.rs`: 2 occurrences

- Line 119: unused variable: `score_expr`: help: if this is intentional, prefix it with an underscore: `_score_expr`
- Line 284: unused variable: `is_edge`: help: if this is intentional, prefix it with an underscore: `_is_edge`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 2 occurrences

- Line 23: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`
- Line 25: unused import: `crate::query::planner::plan::factory::PlanNodeFactory`

#### `src\query\planner\statements\paths\match_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\utils\finder.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\validator\strategies\type_inference.rs`: 2 occurrences

- Line 77: unused variable: `arg`: help: try ignoring the field: `arg: _`
- Line 568: unused variable: `type_inference`: help: if this is intentional, prefix it with an underscore: `_type_inference`

#### `src\query\planner\statements\seeks\index_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\seek_strategy.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 247: unused import: `std::collections::HashMap`
- Line 251: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\optimizer\prune_properties_visitor.rs`: 1 occurrences

- Line 8: unused import: `crate::query::context::validate::types::Variable`

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 7: unused import: `std::collections::HashSet`

#### `src\query\planner\statements\clauses\clause_planner.rs`: 1 occurrences

- Line 5: unused import: `DataFlowNode`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 549: unused import: `crate::core::value::NullType`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 4: unused import: `QueryInfo`

#### `src\query\context\managers\meta_client.rs`: 1 occurrences

- Line 5: unused imports: `PropertyDef` and `PropertyType`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 10: unused import: `crate::expression::ExpressionContext`

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 8: unused import: `crate::query::validator::structs::CypherClauseKind`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\query\context\managers\schema_manager.rs`: 1 occurrences

- Line 5: unused imports: `CharsetInfo` and `SchemaChangeType`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 568: variable does not need to be mutable

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 496: unused import: `crate::core::value::NullType`

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 537: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 951: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\parser\clauses\where_clause_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 374: unused variable: `var_name`: help: if this is intentional, prefix it with an underscore: `_var_name`

#### `src\query\planner\statements\clauses\unwind_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 458: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 317: unused import: `std::path::Path`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 201: unused variable: `id_str`: help: if this is intentional, prefix it with an underscore: `_id_str`

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 107: unused variable: `right_col_map`: help: if this is intentional, prefix it with an underscore: `_right_col_map`

#### `src\query\parser\clauses\skip_limit_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 9: unused import: `AtomicU64`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 247: variable does not need to be mutable

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 567: unused import: `SortNode`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 4: unused import: `QueryInfo`

#### `src\query\planner\statements\clauses\projection_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::validator::structs::CypherClauseKind`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 351: unused import: `UnaryOperator`

#### `src\query\planner\statements\subgraph_planner.rs`: 1 occurrences

- Line 52: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 360: unused import: `crate::core::value::NullType`

#### `src\query\planner\statements\go_planner.rs`: 1 occurrences

- Line 61: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\context\ast\cypher_ast_context.rs`: 1 occurrences

- Line 226: unused variable: `label`: help: if this is intentional, prefix it with an underscore: `_label`

#### `src\query\executor\result_processing\sort.rs`: 1 occurrences

- Line 822: variable does not need to be mutable

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 802: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 252: unused imports: `Direction` and `Value`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 321: unused import: `crate::storage::StorageEngine`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 468: unused import: `DedupNode as Dedup`

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 486: unused import: `crate::core::value::NullType`

