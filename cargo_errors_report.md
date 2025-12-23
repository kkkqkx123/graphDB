# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 86
- **Total Warnings**: 64
- **Total Issues**: 150
- **Unique Error Patterns**: 40
- **Unique Warning Patterns**: 45
- **Files with Issues**: 62

## Error Statistics

**Total Errors**: 86

### Error Type Breakdown

- **error[E0308]**: 43 errors
- **error[E0614]**: 17 errors
- **error[E0599]**: 10 errors
- **error[E0433]**: 6 errors
- **error[E0512]**: 5 errors
- **error[E0282]**: 3 errors
- **error[E0277]**: 1 errors
- **error[E0432]**: 1 errors

### Files with Errors (Top 10)

- `src\query\visitor\find_visitor.rs`: 6 errors
- `src\query\visitor\extract_filter_expr_visitor.rs`: 6 errors
- `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 6 errors
- `src\query\visitor\evaluable_expr_visitor.rs`: 6 errors
- `src\query\planner\ngql\go_planner.rs`: 4 errors
- `src\query\planner\plan\algorithms\path_algorithms.rs`: 4 errors
- `src\query\planner\ngql\fetch_edges_planner.rs`: 4 errors
- `src\query\optimizer\index_optimization.rs`: 4 errors
- `src\query\executor\cypher\context.rs`: 3 errors
- `src\query\optimizer\operation_merge.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 64

### Warning Type Breakdown

- **warning**: 64 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 12 warnings
- `src\core\evaluator\expression_evaluator.rs`: 8 warnings
- `src\query\visitor\extract_filter_expr_visitor.rs`: 3 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\core\evaluator\traits.rs`: 2 warnings
- `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 warnings
- `src\core\context\mod.rs`: 2 warnings
- `src\query\planner\match_planning\clauses\unwind_planner.rs`: 2 warnings
- `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 warnings
- `src\query\executor\result_processing\aggregation.rs`: 2 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `PlanNodeEnum`, found `GetEdgesNode`

**Total Occurrences**: 43  
**Unique Files**: 27

#### `src\query\executor\cypher\context.rs`: 3 occurrences

- Line 255: mismatched types: expected `Option<&Vertex>`, found `Option<Vertex>`
- Line 265: mismatched types: expected `Option<&Edge>`, found `Option<Edge>`
- Line 331: mismatched types: expected `Option<&Value>`, found `Option<Value>`

#### `src\query\optimizer\operation_merge.rs`: 3 occurrences

- Line 57: mismatched types: expected `PlanNodeEnum`, found `Box<PlanNodeEnum>`
- Line 74: mismatched types: expected `Vec<usize>`, found `Vec<MatchedResult>`
- Line 123: mismatched types: expected `PlanNodeEnum`, found `Box<PlanNodeEnum>`

#### `src\query\planner\ngql\fetch_edges_planner.rs`: 2 occurrences

- Line 64: mismatched types: expected `PlanNodeEnum`, found `GetEdgesNode`
- Line 91: mismatched types: expected `PlanNodeEnum`, found `ArgumentNode`

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 411: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 497: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\planner\plan\management\security\role_ops.rs`: 2 occurrences

- Line 93: mismatched types: expected `GrantRole`, found `Arc<GrantRole>`
- Line 123: mismatched types: expected `RevokeRole`, found `Arc<RevokeRole>`

#### `src\query\executor\result_processing\filter.rs`: 2 occurrences

- Line 225: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 311: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\planner\plan\core\nodes\traversal_node.rs`: 2 occurrences

- Line 349: mismatched types: expected `Option<&String>`, found `Option<String>`
- Line 484: mismatched types: expected `Option<&String>`, found `Option<String>`

#### `src\query\executor\result_processing\topn.rs`: 2 occurrences

- Line 411: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`
- Line 427: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 396: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 488: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\optimizer\rule_traits.rs`: 2 occurrences

- Line 431: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`
- Line 440: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`

#### `src\query\executor\result_processing\limit.rs`: 2 occurrences

- Line 204: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 290: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\planner\match_planning\paths\match_path_planner.rs`: 2 occurrences

- Line 220: mismatched types: expected `Arc<dyn PlanNode>`, found `PlanNodeEnum`
- Line 274: mismatched types: expected `Arc<dyn PlanNode>`, found `PlanNodeEnum`

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 2 occurrences

- Line 58: mismatched types: expected `Arc<dyn PlanNode>`, found `PlanNodeEnum`
- Line 61: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`

#### `src\query\executor\result_processing\sort.rs`: 2 occurrences

- Line 200: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 286: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\planner\plan\management\admin\index_ops.rs`: 1 occurrences

- Line 102: mismatched types: expected `ShowIndexes`, found `Arc<ShowIndexes>`

#### `src\query\planner\plan\management\ddl\tag_ops.rs`: 1 occurrences

- Line 94: mismatched types: expected `DropTag`, found `Arc<DropTag>`

#### `src\query\planner\plan\management\admin\system_ops.rs`: 1 occurrences

- Line 43: mismatched types: expected `SubmitJob`, found `Arc<SubmitJob>`

#### `src\query\planner\ngql\fetch_vertices_planner.rs`: 1 occurrences

- Line 106: mismatched types: expected `PlanNodeEnum`, found `ArgumentNode`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 56: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`

#### `src\query\planner\plan\management\ddl\space_ops.rs`: 1 occurrences

- Line 64: mismatched types: expected `CreateSpace`, found `Arc<CreateSpace>`

#### `src\query\planner\plan\management\ddl\edge_ops.rs`: 1 occurrences

- Line 87: mismatched types: expected `ShowEdges`, found `Arc<ShowEdges>`

#### `src\query\planner\plan\management\dml\data_constructors.rs`: 1 occurrences

- Line 57: mismatched types: expected `NewTag`, found `Arc<NewTag>`

#### `src\query\planner\match_planning\match_planner.rs`: 1 occurrences

- Line 138: mismatched types: expected `Option<&SubPlan>`, found `Option<SubPlan>`

#### `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 1 occurrences

- Line 892: mismatched types: expected `Option<&String>`, found `Option<String>`

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 88: mismatched types: expected `Option<&TypeDeductionError>`, found `Option<TypeDeductionError>`

#### `src\query\visitor\evaluable_expr_visitor.rs`: 1 occurrences

- Line 79: mismatched types: expected `Option<&String>`, found `Option<String>`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 758: mismatched types: expected `PlanNodeEnum`, found `Arc<ProjectNode>`

### error[E0614]: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 17  
**Unique Files**: 3

#### `src\query\visitor\find_visitor.rs`: 6 occurrences

- Line 511: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 521: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 522: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- ... 3 more occurrences in this file

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 6 occurrences

- Line 201: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 211: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 212: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- ... 3 more occurrences in this file

#### `src\query\visitor\evaluable_expr_visitor.rs`: 5 occurrences

- Line 146: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 156: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 157: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- ... 2 more occurrences in this file

### error[E0599]: no method named `as_any` found for reference `&nodes::plan_node_enum::PlanNodeEnum` in the current scope: method not found in `&PlanNodeEnum`

**Total Occurrences**: 10  
**Unique Files**: 5

#### `src\query\planner\plan\algorithms\path_algorithms.rs`: 4 occurrences

- Line 144: no method named `visit_multi_shortest_path` found for mutable reference `&mut V` in the current scope: method not found in `&mut V`
- Line 267: no method named `visit_bfs_shortest` found for mutable reference `&mut V` in the current scope
- Line 401: no method named `visit_all_paths` found for mutable reference `&mut V` in the current scope
- ... 1 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 2 occurrences

- Line 53: no method named `as_any` found for reference `&nodes::plan_node_enum::PlanNodeEnum` in the current scope: method not found in `&PlanNodeEnum`
- Line 168: no method named `as_any` found for reference `&nodes::plan_node_enum::PlanNodeEnum` in the current scope: method not found in `&PlanNodeEnum`

#### `src\query\planner\plan\algorithms\index_scan.rs`: 2 occurrences

- Line 135: no method named `visit_index_scan` found for mutable reference `&mut V` in the current scope
- Line 244: no method named `visit_fulltext_index_scan` found for mutable reference `&mut V` in the current scope: method not found in `&mut V`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 515: no method named `dependencies` found for reference `&project_node::ProjectNode` in the current scope: method not found in `&ProjectNode`

#### `src\query\planner\match_planning\clauses\unwind_planner.rs`: 1 occurrences

- Line 231: `core::types::expression::Expression` doesn't implement `std::fmt::Display`: `core::types::expression::Expression` cannot be formatted with the default formatter

### error[E0433]: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`

**Total Occurrences**: 6  
**Unique Files**: 2

#### `src\query\optimizer\index_optimization.rs`: 4 occurrences

- Line 73: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`
- Line 84: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`
- Line 175: failed to resolve: use of undeclared type `PlanNodeEnum`: use of undeclared type `PlanNodeEnum`
- ... 1 more occurrences in this file

#### `src\query\planner\ngql\fetch_edges_planner.rs`: 2 occurrences

- Line 67: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 83: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`

### error[E0512]: cannot transmute between types of different sizes, or dependently-sized types

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 5 occurrences

- Line 116: cannot transmute between types of different sizes, or dependently-sized types
- Line 337: cannot transmute between types of different sizes, or dependently-sized types
- Line 530: cannot transmute between types of different sizes, or dependently-sized types
- ... 2 more occurrences in this file

### error[E0282]: type annotations needed: cannot infer type for type parameter `T` declared on the enum `Option`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\planner\ngql\go_planner.rs`: 3 occurrences

- Line 93: type annotations needed: cannot infer type for type parameter `T` declared on the enum `Option`
- Line 115: type annotations needed: cannot infer type for type parameter `T` declared on the enum `Option`
- Line 117: type annotations needed: cannot infer type for type parameter `T` declared on the enum `Option`

### error[E0432]: unresolved import `crate::query::planner::plan::core::plan_node_enum`: could not find `plan_node_enum` in `core`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 5: unresolved import `crate::query::planner::plan::core::plan_node_enum`: could not find `plan_node_enum` in `core`

### error[E0277]: a value of type `Vec<&dyn CypherClausePlanner>` cannot be built from an iterator over elements of type `&Box<dyn CypherClausePlanner>`: value of type `Vec<&dyn CypherClausePlanner>` cannot be built from `std::iter::Iterator<Item=&Box<dyn CypherClausePlanner>>`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\match_planning\match_planner.rs`: 1 occurrences

- Line 123: a value of type `Vec<&dyn CypherClausePlanner>` cannot be built from an iterator over elements of type `&Box<dyn CypherClausePlanner>`: value of type `Vec<&dyn CypherClausePlanner>` cannot be built from `std::iter::Iterator<Item=&Box<dyn CypherClausePlanner>>`

## Detailed Warning Categorization

### warning: unused import: `std::sync::Arc`

**Total Occurrences**: 64  
**Unique Files**: 36

#### `src\query\planner\plan\core\nodes\factory.rs`: 12 occurrences

- Line 35: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 31: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- Line 49: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- ... 9 more occurrences in this file

#### `src\core\evaluator\expression_evaluator.rs`: 8 occurrences

- Line 243: variable does not need to be mutable
- Line 311: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 311: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- ... 5 more occurrences in this file

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 127: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 3 occurrences

- Line 4: unused import: `VisitorResult`
- Line 377: unused variable: `func`: help: if this is intentional, prefix it with an underscore: `_func`
- Line 513: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\core\evaluator\traits.rs`: 2 occurrences

- Line 30: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 30: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 283: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`
- Line 283: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\planner\match_planning\clauses\unwind_planner.rs`: 2 occurrences

- Line 1: unused import: `std::sync::Arc`
- Line 53: unused import: `crate::query::planner::plan::factory::PlanNodeFactory`

#### `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 occurrences

- Line 5: unused import: `crate::query::context::validate::types::Variable`
- Line 588: unused import: `super::plan_node_operations::*`

#### `src\core\context\mod.rs`: 2 occurrences

- Line 20: ambiguous glob re-exports: the name `SessionVariable` in the type namespace is first re-exported here
- Line 22: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 318: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 318: variable does not need to be mutable

#### `src\query\planner\plan\core\nodes\sort_node.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\admin\host_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\plan\core\nodes\traversal_node.rs`: 1 occurrences

- Line 8: unused import: `Arc`

#### `src\core\query_pipeline_manager.rs`: 1 occurrences

- Line 117: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\query\planner\plan\management\admin\config_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 1 occurrences

- Line 103: unused variable: `path`: help: if this is intentional, prefix it with an underscore: `_path`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 862: unused import: `crate::query::planner::plan::algorithms::IndexScan`

#### `src\query\planner\plan\management\dml\insert_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 1 occurrences

- Line 9: unused import: `Arc`

#### `src\expression\cypher\expression_converter.rs`: 1 occurrences

- Line 7: unused imports: `BinaryOperator` and `UnaryOperator`

#### `src\query\planner\plan\core\nodes\mod.rs`: 1 occurrences

- Line 28: unused import: `plan_node_operations::*`

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 11: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\ddl\edge_ops.rs`: 1 occurrences

- Line 5: unused import: `crate::query::context::validate::types::Variable`

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 1 occurrences

- Line 5: unused import: `crate::core::ValueTypeDef`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 51: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\planner\plan\management\dml\delete_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\visitor\mod.rs`: 1 occurrences

- Line 5: unused import: `crate::core::visitor::VisitorConfig`

#### `src\core\mod.rs`: 1 occurrences

- Line 46: ambiguous glob re-exports: the name `SymbolType` in the type namespace is first re-exported here

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 11: unused import: `std::sync::Arc`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::core::expressions::ExpressionContext`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\query\planner\ngql\fetch_vertices_planner.rs`: 1 occurrences

- Line 5: unused import: `Column`

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 400: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `VisitorResult`

#### `src\core\visitor.rs`: 1 occurrences

- Line 385: unused variable: `target_type`: help: try ignoring the field: `target_type: _`

#### `src\query\planner\plan\management\dml\update_ops.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

