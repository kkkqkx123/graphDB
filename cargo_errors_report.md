# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 260
- **Total Warnings**: 63
- **Total Issues**: 323
- **Unique Error Patterns**: 24
- **Unique Warning Patterns**: 51
- **Files with Issues**: 60

## Error Statistics

**Total Errors**: 260

### Error Type Breakdown

- **error[E0412]**: 182 errors
- **error[E0433]**: 37 errors
- **error[E0308]**: 29 errors
- **error[E0599]**: 10 errors
- **error[E0382]**: 1 errors
- **error[E0432]**: 1 errors

### Files with Errors (Top 10)

- `src\query\optimizer\limit_pushdown.rs`: 16 errors
- `src\query\optimizer\predicate_pushdown.rs`: 14 errors
- `src\query\executor\result_processing\aggregation.rs`: 13 errors
- `src\query\executor\result_processing\filter.rs`: 13 errors
- `src\query\executor\data_processing\loops.rs`: 13 errors
- `src\query\executor\data_processing\join\left_join.rs`: 13 errors
- `src\query\executor\data_processing\join\cross_join.rs`: 13 errors
- `src\query\executor\data_processing\join\mod.rs`: 13 errors
- `src\query\executor\result_processing\topn.rs`: 13 errors
- `src\query\executor\result_processing\sample.rs`: 13 errors

## Warning Statistics

**Total Warnings**: 63

### Warning Type Breakdown

- **warning**: 63 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 15 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 6 warnings
- `src\query\planner\match_planning\utils\finder.rs`: 3 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\expression\evaluator\traits.rs`: 2 warnings
- `src\query\planner\match_planning\clauses\order_by_planner.rs`: 2 warnings
- `src\query\parser\cypher\expression_converter.rs`: 2 warnings
- `src\query\optimizer\limit_pushdown.rs`: 2 warnings
- `src\query\executor\result_processing\aggregation.rs`: 2 warnings
- `src\query\visitor\mod.rs`: 1 warnings

## Detailed Error Categorization

### error[E0412]: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`

**Total Occurrences**: 182  
**Unique Files**: 14

#### `src\query\executor\data_processing\join\inner_join.rs`: 13 occurrences

- Line 356: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 363: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 371: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\data_processing\join\mod.rs`: 13 occurrences

- Line 253: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 259: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 265: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\data_processing\loops.rs`: 13 occurrences

- Line 536: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 543: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 551: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\result_processing\filter.rs`: 13 occurrences

- Line 329: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 336: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 344: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\result_processing\topn.rs`: 13 occurrences

- Line 520: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 527: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 535: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\data_processing\join\left_join.rs`: 13 occurrences

- Line 363: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 370: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 378: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\factory.rs`: 13 occurrences

- Line 134: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 141: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 149: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 13 occurrences

- Line 353: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 360: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 367: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\data_processing\join\cross_join.rs`: 13 occurrences

- Line 383: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 390: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 398: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\result_processing\sort.rs`: 13 occurrences

- Line 303: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 310: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 318: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\result_processing\limit.rs`: 13 occurrences

- Line 306: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 313: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 321: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\result_processing\dedup.rs`: 13 occurrences

- Line 505: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 512: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 520: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\result_processing\aggregation.rs`: 13 occurrences

- Line 845: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 852: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 860: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

#### `src\query\executor\result_processing\sample.rs`: 13 occurrences

- Line 513: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 520: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- Line 528: cannot find type `StorageError` in module `crate::storage`: not found in `crate::storage`
- ... 10 more occurrences in this file

### error[E0433]: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`

**Total Occurrences**: 37  
**Unique Files**: 10

#### `src\query\optimizer\limit_pushdown.rs`: 16 occurrences

- Line 915: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 917: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 938: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- ... 13 more occurrences in this file

#### `src\query\validator\strategies\clause_strategy.rs`: 4 occurrences

- Line 306: failed to resolve: could not find `LiteralValue` in `expression`: could not find `LiteralValue` in `expression`
- Line 339: failed to resolve: could not find `LiteralValue` in `expression`: could not find `LiteralValue` in `expression`
- Line 372: failed to resolve: could not find `LiteralValue` in `expression`: could not find `LiteralValue` in `expression`
- ... 1 more occurrences in this file

#### `src\query\optimizer\join_optimization.rs`: 3 occurrences

- Line 134: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`
- Line 136: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`
- Line 145: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`

#### `src\cache\factory.rs`: 3 occurrences

- Line 452: failed to resolve: use of undeclared type `CachePolicy`: use of undeclared type `CachePolicy`
- Line 457: failed to resolve: use of undeclared type `CachePolicy`: use of undeclared type `CachePolicy`
- Line 467: failed to resolve: use of undeclared type `CachePolicy`: use of undeclared type `CachePolicy`

#### `src\cache\global_manager.rs`: 3 occurrences

- Line 174: failed to resolve: use of undeclared type `CacheStrategy`: use of undeclared type `CacheStrategy`
- Line 222: failed to resolve: use of undeclared type `CacheStrategy`: use of undeclared type `CacheStrategy`
- Line 225: failed to resolve: use of undeclared type `CacheStrategy`: use of undeclared type `CacheStrategy`

#### `src\query\planner\plan\core\nodes\sort_node.rs`: 3 occurrences

- Line 529: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`
- Line 543: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`
- Line 556: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`

#### `src\query\optimizer\scan_optimization.rs`: 2 occurrences

- Line 123: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`
- Line 141: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 384: failed to resolve: could not find `LiteralValue` in `expression`: could not find `LiteralValue` in `expression`

#### `src\cache\parser_cache.rs`: 1 occurrences

- Line 516: failed to resolve: use of undeclared type `Duration`: use of undeclared type `Duration`

#### `src\query\validator\strategies\alias_strategy.rs`: 1 occurrences

- Line 340: failed to resolve: could not find `LiteralValue` in `expression`: could not find `LiteralValue` in `expression`

### error[E0308]: arguments to this method are incorrect

**Total Occurrences**: 29  
**Unique Files**: 5

#### `src\query\optimizer\predicate_pushdown.rs`: 14 occurrences

- Line 1164: mismatched types: expected `PlanNodeEnum`, found `Arc<StartNode>`
- Line 1169: mismatched types: expected `PlanNodeEnum`, found `Arc<FilterNode>`
- Line 1186: mismatched types: expected `PlanNodeEnum`, found `Arc<StartNode>`
- ... 11 more occurrences in this file

#### `src\query\optimizer\operation_merge.rs`: 8 occurrences

- Line 494: mismatched types: expected `PlanNodeEnum`, found `Arc<StartNode>`
- Line 500: mismatched types: expected `PlanNodeEnum`, found `Arc<FilterNode>`
- Line 517: mismatched types: expected `PlanNodeEnum`, found `Arc<StartNode>`
- ... 5 more occurrences in this file

#### `src\query\planner\match_planning\clauses\projection_planner.rs`: 4 occurrences

- Line 324: arguments to this method are incorrect
- Line 340: arguments to this method are incorrect
- Line 390: arguments to this function are incorrect
- ... 1 more occurrences in this file

#### `src\query\optimizer\projection_pushdown.rs`: 2 occurrences

- Line 149: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`
- Line 171: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 118: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`

### error[E0599]: no function or associated item named `create_cache_by_policy` found for struct `cache::factory::CacheFactory` in the current scope: function or associated item not found in `CacheFactory`

**Total Occurrences**: 10  
**Unique Files**: 3

#### `src\query\planner\plan\core\nodes\control_flow_node.rs`: 4 occurrences

- Line 394: no method named `type_name` found for struct `control_flow_node::ArgumentNode` in the current scope
- Line 402: no method named `type_name` found for struct `control_flow_node::SelectNode` in the current scope
- Line 412: no method named `type_name` found for struct `control_flow_node::LoopNode` in the current scope
- ... 1 more occurrences in this file

#### `src\cache\factory.rs`: 3 occurrences

- Line 452: no function or associated item named `create_cache_by_policy` found for struct `cache::factory::CacheFactory` in the current scope: function or associated item not found in `CacheFactory`
- Line 456: no function or associated item named `create_cache_by_policy` found for struct `cache::factory::CacheFactory` in the current scope: function or associated item not found in `CacheFactory`
- Line 467: no function or associated item named `create_stats_cache_by_policy` found for struct `cache::factory::CacheFactory` in the current scope: function or associated item not found in `CacheFactory`

#### `src\cache\manager.rs`: 3 occurrences

- Line 371: no method named `hits` found for struct `Arc<StatsCacheWrapper<String, String, ..., ...>>` in the current scope: method not found in `Arc<StatsCacheWrapper<String, String, ..., ...>>`
- Line 414: no method named `hits` found for struct `Arc<StatsCacheWrapper<String, String, ..., ...>>` in the current scope: method not found in `Arc<StatsCacheWrapper<String, String, ..., ...>>`
- Line 422: no method named `hits` found for struct `Arc<StatsCacheWrapper<String, String, ..., ...>>` in the current scope: method not found in `Arc<StatsCacheWrapper<String, String, ..., ...>>`

### error[E0432]: unresolved import `crate::storage::StorageError`: no `StorageError` in `storage`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 16: unresolved import `crate::storage::StorageError`: no `StorageError` in `storage`

### error[E0382]: borrow of partially moved value: `runtime_ctx`: value borrowed here after partial move

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 448: borrow of partially moved value: `runtime_ctx`: value borrowed here after partial move

## Detailed Warning Categorization

### warning: ambiguous glob re-exports: the name `SymbolType` in the type namespace is first re-exported here

**Total Occurrences**: 63  
**Unique Files**: 35

#### `src\query\planner\plan\core\nodes\factory.rs`: 15 occurrences

- Line 333: unused import: `crate::core::Expression`
- Line 334: unused imports: `Expr` and `VariableExpr`
- Line 335: unused import: `crate::query::parser::ast::types::Span`
- ... 12 more occurrences in this file

#### `src\expression\evaluator\expression_evaluator.rs`: 6 occurrences

- Line 304: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 304: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- Line 889: unused variable: `distinct`: help: if this is intentional, prefix it with an underscore: `_distinct`
- ... 3 more occurrences in this file

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 127: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\query\planner\match_planning\utils\finder.rs`: 3 occurrences

- Line 294: unused imports: `ReturnClauseContext`, `UnwindClauseContext`, `WhereClauseContext`, `WithClauseContext`, and `YieldClauseContext`
- Line 347: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`
- Line 354: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`

#### `src\query\planner\match_planning\clauses\order_by_planner.rs`: 2 occurrences

- Line 148: unused import: `std::collections::HashMap`
- Line 196: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `src\query\optimizer\limit_pushdown.rs`: 2 occurrences

- Line 888: unused import: `crate::query::planner::plan::algorithms::IndexScan`
- Line 890: unused imports: `GetEdgesNode`, `GetNeighborsNode`, `GetVerticesNode`, `ProjectNode`, `ScanEdgesNode`, and `ScanVerticesNode`

#### `src\query\parser\cypher\expression_converter.rs`: 2 occurrences

- Line 268: unused imports: `CaseAlternative`, `CaseExpression`, `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`
- Line 272: unused import: `UnaryOperator`

#### `src\expression\evaluator\traits.rs`: 2 occurrences

- Line 30: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 30: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 284: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`
- Line 284: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\core\mod.rs`: 1 occurrences

- Line 46: ambiguous glob re-exports: the name `SymbolType` in the type namespace is first re-exported here

#### `src\query\optimizer\predicate_pushdown.rs`: 1 occurrences

- Line 1138: unused imports: `ExpandNode`, `ScanVerticesNode`, and `TraverseNode`

#### `src\query\planner\match_planning\core\match_planner.rs`: 1 occurrences

- Line 90: unused imports: `AliasType` and `CypherClauseContext`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 347: unused import: `UnaryOperator`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\core\query_pipeline_manager.rs`: 1 occurrences

- Line 117: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 51: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 58: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 393: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\visitor\mod.rs`: 1 occurrences

- Line 147: variable does not need to be mutable

#### `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 1 occurrences

- Line 318: variable does not need to be mutable

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 100: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 560: variable does not need to be mutable

#### `src\core\context\mod.rs`: 1 occurrences

- Line 22: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\query\executor\cypher\factory.rs`: 1 occurrences

- Line 152: unused import: `CypherExecutorTrait`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 319: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 110: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 555: unused import: `SortNode`

#### `src\expression\visitor.rs`: 1 occurrences

- Line 287: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\parser\cypher\parser.rs`: 1 occurrences

- Line 340: variable does not need to be mutable

