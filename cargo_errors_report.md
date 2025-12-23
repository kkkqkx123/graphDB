# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 146
- **Total Warnings**: 0
- **Total Issues**: 146
- **Unique Error Patterns**: 64
- **Unique Warning Patterns**: 0
- **Files with Issues**: 48

## Error Statistics

**Total Errors**: 146

### Error Type Breakdown

- **error[E0308]**: 106 errors
- **error[E0614]**: 18 errors
- **error[E0599]**: 9 errors
- **error[E0512]**: 5 errors
- **error[E0277]**: 4 errors
- **error[E0282]**: 3 errors
- **error[E0432]**: 1 errors

### Files with Errors (Top 10)

- `src\query\planner\ngql\subgraph_planner.rs`: 8 errors
- `src\query\visitor\evaluable_expr_visitor.rs`: 7 errors
- `src\query\planner\ngql\fetch_vertices_planner.rs`: 7 errors
- `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 6 errors
- `src\query\visitor\extract_filter_expr_visitor.rs`: 6 errors
- `src\query\visitor\find_visitor.rs`: 6 errors
- `src\query\planner\ngql\path_planner.rs`: 6 errors
- `src\query\planner\match_planning\utils\connection_strategy.rs`: 5 errors
- `src\query\planner\ngql\go_planner.rs`: 4 errors
- `src\query\validator\strategies\aggregate_strategy.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `ShowEdges`, found `Arc<ShowEdges>`

**Total Occurrences**: 106  
**Unique Files**: 41

#### `src\query\planner\ngql\fetch_vertices_planner.rs`: 7 occurrences

- Line 77: mismatched types: expected `PlanNodeEnum`, found `Arc<ArgumentNode>`
- Line 95: mismatched types: expected `PlanNodeEnum`, found `Arc<GetVerticesNode>`
- Line 103: mismatched types: expected `PlanNodeEnum`, found `Arc<GetVerticesNode>`
- ... 4 more occurrences in this file

#### `src\query\planner\ngql\subgraph_planner.rs`: 6 occurrences

- Line 65: mismatched types: expected `PlanNodeEnum`, found `Arc<ExpandAllNode>`
- Line 78: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- Line 91: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- ... 3 more occurrences in this file

#### `src\query\planner\ngql\path_planner.rs`: 5 occurrences

- Line 102: mismatched types: expected `PlanNodeEnum`, found `Arc<ExpandAllNode>`
- Line 120: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- Line 129: mismatched types: expected `PlanNodeEnum`, found `Arc<ProjectNode>`
- ... 2 more occurrences in this file

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 5 occurrences

- Line 177: arguments to this function are incorrect
- Line 233: arguments to this function are incorrect
- Line 235: arguments to this function are incorrect
- ... 2 more occurrences in this file

#### `src\query\executor\result_processing\aggregation.rs`: 4 occurrences

- Line 471: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 557: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`
- Line 739: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\aggregate_strategy.rs`: 4 occurrences

- Line 42: mismatched types: expected `&Expression`, found `Box<Expression>`
- Line 51: mismatched types: expected `&Expression`, found `Box<Expression>`
- Line 162: mismatched types: expected `&Expression`, found `Box<Expression>`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\management\security\role_ops.rs`: 4 occurrences

- Line 63: mismatched types: expected `DropRole`, found `Arc<DropRole>`
- Line 93: mismatched types: expected `GrantRole`, found `Arc<GrantRole>`
- Line 123: mismatched types: expected `RevokeRole`, found `Arc<RevokeRole>`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\management\ddl\space_ops.rs`: 4 occurrences

- Line 65: mismatched types: expected `CreateSpace`, found `Arc<CreateSpace>`
- Line 89: mismatched types: expected `DescSpace`, found `Arc<DescSpace>`
- Line 113: mismatched types: expected `ShowCreateSpace`, found `Arc<ShowCreateSpace>`
- ... 1 more occurrences in this file

#### `src\query\optimizer\index_optimization.rs`: 4 occurrences

- Line 77: mismatched types: expected `PlanNodeEnum`, found `Arc<IndexScan>`
- Line 88: mismatched types: expected `PlanNodeEnum`, found `Arc<FilterNode>`
- Line 183: mismatched types: expected `PlanNodeEnum`, found `Arc<IndexScan>`
- ... 1 more occurrences in this file

#### `src\query\planner\ngql\fetch_edges_planner.rs`: 3 occurrences

- Line 65: mismatched types: expected `PlanNodeEnum`, found `Arc<GetEdgesNode>`
- Line 72: `match` arms have incompatible types: expected `Arc<FilterNode>`, found `Arc<GetEdgesNode>`
- Line 92: mismatched types: expected `PlanNodeEnum`, found `Arc<ArgumentNode>`

#### `src\query\executor\cypher\context.rs`: 3 occurrences

- Line 255: mismatched types: expected `Option<&Vertex>`, found `Option<Vertex>`
- Line 265: mismatched types: expected `Option<&Edge>`, found `Option<Edge>`
- Line 331: mismatched types: expected `Option<&Value>`, found `Option<Value>`

#### `src\query\planner\ngql\maintain_planner.rs`: 3 occurrences

- Line 70: mismatched types: expected `PlanNodeEnum`, found `Arc<ArgumentNode>`
- Line 92: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- Line 93: mismatched types: expected `PlanNodeEnum`, found `Arc<ArgumentNode>`

#### `src\query\optimizer\operation_merge.rs`: 3 occurrences

- Line 60: mismatched types: expected `PlanNodeEnum`, found `Box<PlanNodeEnum>`
- Line 77: mismatched types: expected `Vec<usize>`, found `Vec<MatchedResult>`
- Line 126: mismatched types: expected `PlanNodeEnum`, found `Box<PlanNodeEnum>`

#### `src\query\planner\plan\management\ddl\tag_ops.rs`: 3 occurrences

- Line 64: mismatched types: expected `DescTag`, found `Arc<DescTag>`
- Line 94: mismatched types: expected `DropTag`, found `Arc<DropTag>`
- Line 110: mismatched types: expected `ShowTags`, found `Arc<ShowTags>`

#### `src\query\optimizer\elimination_rules.rs`: 3 occurrences

- Line 927: mismatched types: expected `PlanNodeEnum`, found `Box<PlanNodeEnum>`
- Line 978: arguments to this function are incorrect
- Line 993: arguments to this function are incorrect

#### `src\query\planner\match_planning\clauses\unwind_planner.rs`: 3 occurrences

- Line 132: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- Line 132: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- Line 239: mismatched types: expected `Arc<dyn PlanNode>`, found `PlanNodeEnum`

#### `src\query\planner\plan\management\admin\index_ops.rs`: 3 occurrences

- Line 80: mismatched types: expected `DropIndex`, found `Arc<DropIndex>`
- Line 102: mismatched types: expected `ShowIndexes`, found `Arc<ShowIndexes>`
- Line 126: mismatched types: expected `DescIndex`, found `Arc<DescIndex>`

#### `src\query\planner\plan\management\dml\data_constructors.rs`: 3 occurrences

- Line 30: mismatched types: expected `NewVertex`, found `Arc<NewVertex>`
- Line 57: mismatched types: expected `NewTag`, found `Arc<NewTag>`
- Line 87: mismatched types: expected `NewProp`, found `Arc<NewProp>`

#### `src\query\planner\plan\management\ddl\edge_ops.rs`: 2 occurrences

- Line 87: mismatched types: expected `ShowEdges`, found `Arc<ShowEdges>`
- Line 111: mismatched types: expected `ShowCreateEdge`, found `Arc<ShowCreateEdge>`

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 2 occurrences

- Line 58: mismatched types: expected `Arc<dyn PlanNode>`, found `PlanNodeEnum`
- Line 61: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`

#### `src\query\planner\plan\core\nodes\traversal_node.rs`: 2 occurrences

- Line 349: mismatched types: expected `Option<&String>`, found `Option<String>`
- Line 484: mismatched types: expected `Option<&String>`, found `Option<String>`

#### `src\query\executor\result_processing\filter.rs`: 2 occurrences

- Line 217: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 303: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 396: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 488: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\executor\result_processing\sort.rs`: 2 occurrences

- Line 200: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 286: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\planner\plan\management\admin\host_ops.rs`: 2 occurrences

- Line 57: mismatched types: expected `DropHosts`, found `Arc<DropHosts>`
- Line 73: mismatched types: expected `ShowHosts`, found `Arc<ShowHosts>`

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 411: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 497: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\optimizer\rule_traits.rs`: 2 occurrences

- Line 427: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`
- Line 436: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`

#### `src\query\executor\result_processing\topn.rs`: 2 occurrences

- Line 411: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`
- Line 427: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`

#### `src\query\planner\plan\management\admin\system_ops.rs`: 2 occurrences

- Line 40: mismatched types: expected `SubmitJob`, found `Arc<SubmitJob>`
- Line 94: mismatched types: expected `DropSnapshot`, found `Arc<DropSnapshot>`

#### `src\query\executor\result_processing\limit.rs`: 2 occurrences

- Line 204: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 290: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\planner\match_planning\paths\match_path_planner.rs`: 2 occurrences

- Line 220: mismatched types: expected `Arc<dyn PlanNode>`, found `PlanNodeEnum`
- Line 274: mismatched types: expected `Arc<dyn PlanNode>`, found `PlanNodeEnum`

#### `src\query\planner\match_planning\match_planner.rs`: 1 occurrences

- Line 138: mismatched types: expected `Option<&SubPlan>`, found `Option<SubPlan>`

#### `src\query\planner\plan\management\dml\insert_ops.rs`: 1 occurrences

- Line 25: mismatched types: expected `InsertVertices`, found `Arc<InsertVertices>`

#### `src\query\planner\plan\management\dml\delete_ops.rs`: 1 occurrences

- Line 52: mismatched types: expected `DeleteTags`, found `Arc<DeleteTags>`

#### `src\query\planner\plan\management\dml\update_ops.rs`: 1 occurrences

- Line 33: mismatched types: expected `UpdateVertex`, found `Arc<UpdateVertex>`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 799: mismatched types: expected `PlanNodeEnum`, found `Arc<ProjectNode>`

#### `src\query\planner\plan\management\admin\config_ops.rs`: 1 occurrences

- Line 51: mismatched types: expected `ShowConfigs`, found `Arc<ShowConfigs>`

#### `src\query\visitor\evaluable_expr_visitor.rs`: 1 occurrences

- Line 73: mismatched types: expected `Option<&String>`, found `Option<String>`

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 88: mismatched types: expected `Option<&TypeDeductionError>`, found `Option<TypeDeductionError>`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 56: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`

#### `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 1 occurrences

- Line 884: mismatched types: expected `Option<&String>`, found `Option<String>`

### error[E0614]: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 18  
**Unique Files**: 3

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 6 occurrences

- Line 191: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 197: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 198: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- ... 3 more occurrences in this file

#### `src\query\visitor\find_visitor.rs`: 6 occurrences

- Line 511: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 521: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 522: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- ... 3 more occurrences in this file

#### `src\query\visitor\evaluable_expr_visitor.rs`: 6 occurrences

- Line 133: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 139: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 140: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- ... 3 more occurrences in this file

### error[E0599]: no method named `visit_index_scan` found for mutable reference `&mut V` in the current scope

**Total Occurrences**: 9  
**Unique Files**: 4

#### `src\query\planner\plan\algorithms\path_algorithms.rs`: 4 occurrences

- Line 144: no method named `visit_multi_shortest_path` found for mutable reference `&mut V` in the current scope: method not found in `&mut V`
- Line 267: no method named `visit_bfs_shortest` found for mutable reference `&mut V` in the current scope
- Line 401: no method named `visit_all_paths` found for mutable reference `&mut V` in the current scope
- ... 1 more occurrences in this file

#### `src\query\planner\plan\algorithms\index_scan.rs`: 2 occurrences

- Line 135: no method named `visit_index_scan` found for mutable reference `&mut V` in the current scope
- Line 244: no method named `visit_fulltext_index_scan` found for mutable reference `&mut V` in the current scope: method not found in `&mut V`

#### `src\query\optimizer\predicate_pushdown.rs`: 2 occurrences

- Line 55: no method named `as_any` found for reference `&plan_node_enum::PlanNodeEnum` in the current scope: method not found in `&PlanNodeEnum`
- Line 170: no method named `as_any` found for reference `&plan_node_enum::PlanNodeEnum` in the current scope: method not found in `&PlanNodeEnum`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 515: no method named `dependencies` found for reference `&project_node::ProjectNode` in the current scope: method not found in `&ProjectNode`

### error[E0512]: cannot transmute between types of different sizes, or dependently-sized types

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 5 occurrences

- Line 115: cannot transmute between types of different sizes, or dependently-sized types
- Line 334: cannot transmute between types of different sizes, or dependently-sized types
- Line 525: cannot transmute between types of different sizes, or dependently-sized types
- ... 2 more occurrences in this file

### error[E0277]: the trait bound `ExpandAllNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `ExpandAllNode`

**Total Occurrences**: 4  
**Unique Files**: 3

#### `src\query\planner\ngql\subgraph_planner.rs`: 2 occurrences

- Line 69: the trait bound `ExpandAllNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `ExpandAllNode`
- Line 72: the trait bound `ExpandAllNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `ExpandAllNode`

#### `src\query\planner\ngql\path_planner.rs`: 1 occurrences

- Line 107: the trait bound `ExpandAllNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `ExpandAllNode`

#### `src\query\planner\match_planning\match_planner.rs`: 1 occurrences

- Line 123: a value of type `Vec<&dyn CypherClausePlanner>` cannot be built from an iterator over elements of type `&Box<dyn CypherClausePlanner>`: value of type `Vec<&dyn CypherClausePlanner>` cannot be built from `std::iter::Iterator<Item=&Box<dyn CypherClausePlanner>>`

### error[E0282]: type annotations needed: cannot infer type for type parameter `T` declared on the enum `Option`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\planner\ngql\go_planner.rs`: 3 occurrences

- Line 94: type annotations needed: cannot infer type for type parameter `T` declared on the enum `Option`
- Line 114: type annotations needed: cannot infer type for type parameter `T` declared on the enum `Option`
- Line 116: type annotations needed: cannot infer type for type parameter `T` declared on the enum `Option`

### error[E0432]: unresolved import `crate::query::planner::plan::core::plan_node_enum`: could not find `plan_node_enum` in `core`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 9: unresolved import `crate::query::planner::plan::core::plan_node_enum`: could not find `plan_node_enum` in `core`

