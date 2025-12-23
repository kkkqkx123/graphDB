# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 241
- **Total Warnings**: 72
- **Total Issues**: 313
- **Unique Error Patterns**: 108
- **Unique Warning Patterns**: 55
- **Files with Issues**: 85

## Error Statistics

**Total Errors**: 241

### Error Type Breakdown

- **error[E0308]**: 157 errors
- **error[E0599]**: 35 errors
- **error[E0614]**: 18 errors
- **error[E0277]**: 9 errors
- **error[E0004]**: 8 errors
- **error[E0615]**: 7 errors
- **error[E0515]**: 4 errors
- **error[E0382]**: 2 errors
- **error[E0412]**: 1 errors

### Files with Errors (Top 10)

- `src\query\optimizer\predicate_pushdown.rs`: 18 errors
- `src\query\optimizer\limit_pushdown.rs`: 13 errors
- `src\query\optimizer\index_optimization.rs`: 13 errors
- `src\query\optimizer\operation_merge.rs`: 10 errors
- `src\query\planner\ngql\go_planner.rs`: 10 errors
- `src\query\planner\plan\management\ddl\space_ops.rs`: 8 errors
- `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 8 errors
- `src\query\planner\ngql\subgraph_planner.rs`: 8 errors
- `src\query\visitor\evaluable_expr_visitor.rs`: 7 errors
- `src\query\optimizer\elimination_rules.rs`: 7 errors

## Warning Statistics

**Total Warnings**: 72

### Warning Type Breakdown

- **warning**: 72 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 14 warnings
- `src\core\evaluator\expression_evaluator.rs`: 8 warnings
- `src\query\visitor\extract_filter_expr_visitor.rs`: 3 warnings
- `src\query\optimizer\elimination_rules.rs`: 3 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\core\context\runtime.rs`: 2 warnings
- `src\core\evaluator\traits.rs`: 2 warnings
- `src\core\context\query.rs`: 2 warnings
- `src\core\context\mod.rs`: 2 warnings
- `src\query\executor\result_processing\aggregation.rs`: 2 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `SubmitJob`, found `Arc<SubmitJob>`

**Total Occurrences**: 157  
**Unique Files**: 50

#### `src\query\optimizer\predicate_pushdown.rs`: 9 occurrences

- Line 88: mismatched types: expected `PlanNodeEnum`, found `Arc<FilterNode>`
- Line 144: mismatched types: expected `PlanNodeEnum`, found `Arc<FilterNode>`
- Line 205: mismatched types: expected `PlanNodeEnum`, found `Arc<FilterNode>`
- ... 6 more occurrences in this file

#### `src\query\planner\plan\management\ddl\space_ops.rs`: 8 occurrences

- Line 65: mismatched types: expected `CreateSpace`, found `Arc<CreateSpace>`
- Line 89: mismatched types: expected `DescSpace`, found `Arc<DescSpace>`
- Line 113: mismatched types: expected `ShowCreateSpace`, found `Arc<ShowCreateSpace>`
- ... 5 more occurrences in this file

#### `src\query\optimizer\elimination_rules.rs`: 7 occurrences

- Line 882: mismatched types: expected `PlanNodeEnum`, found `Box<PlanNodeEnum>`
- Line 894: mismatched types: expected `PlanNodeEnum`, found `Box<PlanNodeEnum>`
- Line 906: mismatched types: expected `PlanNodeEnum`, found `Box<PlanNodeEnum>`
- ... 4 more occurrences in this file

#### `src\query\planner\ngql\fetch_vertices_planner.rs`: 7 occurrences

- Line 77: mismatched types: expected `PlanNodeEnum`, found `Arc<ArgumentNode>`
- Line 95: mismatched types: expected `PlanNodeEnum`, found `Arc<GetVerticesNode>`
- Line 103: mismatched types: expected `PlanNodeEnum`, found `Arc<GetVerticesNode>`
- ... 4 more occurrences in this file

#### `src\query\planner\ngql\go_planner.rs`: 6 occurrences

- Line 93: arguments to this function are incorrect
- Line 117: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- Line 144: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- ... 3 more occurrences in this file

#### `src\query\planner\ngql\subgraph_planner.rs`: 6 occurrences

- Line 65: mismatched types: expected `PlanNodeEnum`, found `Arc<ExpandAllNode>`
- Line 78: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- Line 91: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- ... 3 more occurrences in this file

#### `src\query\planner\plan\management\security\role_ops.rs`: 5 occurrences

- Line 33: mismatched types: expected `CreateRole`, found `Arc<CreateRole>`
- Line 63: mismatched types: expected `DropRole`, found `Arc<DropRole>`
- Line 93: mismatched types: expected `GrantRole`, found `Arc<GrantRole>`
- ... 2 more occurrences in this file

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

#### `src\query\planner\plan\management\ddl\tag_ops.rs`: 5 occurrences

- Line 40: mismatched types: expected `CreateTag`, found `Arc<CreateTag>`
- Line 64: mismatched types: expected `DescTag`, found `Arc<DescTag>`
- Line 94: mismatched types: expected `DropTag`, found `Arc<DropTag>`
- ... 2 more occurrences in this file

#### `src\query\planner\plan\management\admin\system_ops.rs`: 4 occurrences

- Line 40: mismatched types: expected `SubmitJob`, found `Arc<SubmitJob>`
- Line 70: mismatched types: expected `CreateSnapshot`, found `Arc<CreateSnapshot>`
- Line 94: mismatched types: expected `DropSnapshot`, found `Arc<DropSnapshot>`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\management\admin\host_ops.rs`: 4 occurrences

- Line 35: mismatched types: expected `AddHosts`, found `Arc<AddHosts>`
- Line 57: mismatched types: expected `DropHosts`, found `Arc<DropHosts>`
- Line 73: mismatched types: expected `ShowHosts`, found `Arc<ShowHosts>`
- ... 1 more occurrences in this file

#### `src\query\optimizer\index_optimization.rs`: 4 occurrences

- Line 89: mismatched types: expected `PlanNodeEnum`, found `Arc<FilterNode>`
- Line 195: mismatched types: expected `PlanNodeEnum`, found `Arc<FilterNode>`
- Line 735: mismatched types: expected `PlanNodeEnum`, found `Arc<IndexScan>`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\management\dml\data_constructors.rs`: 4 occurrences

- Line 30: mismatched types: expected `NewVertex`, found `Arc<NewVertex>`
- Line 57: mismatched types: expected `NewTag`, found `Arc<NewTag>`
- Line 87: mismatched types: expected `NewProp`, found `Arc<NewProp>`
- ... 1 more occurrences in this file

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

#### `src\core\visitor.rs`: 4 occurrences

- Line 325: mismatched types: expected `&Option<Expression>`, found `&Option<Box<Expression>>`
- Line 402: mismatched types: expected `Expression`, found `Box<Expression>`
- Line 409: mismatched types: expected `Expression`, found `Box<Expression>`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\management\ddl\edge_ops.rs`: 4 occurrences

- Line 41: mismatched types: expected `CreateEdge`, found `Arc<CreateEdge>`
- Line 71: mismatched types: expected `DropEdge`, found `Arc<DropEdge>`
- Line 87: mismatched types: expected `ShowEdges`, found `Arc<ShowEdges>`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\management\admin\index_ops.rs`: 4 occurrences

- Line 50: mismatched types: expected `CreateIndex`, found `Arc<CreateIndex>`
- Line 80: mismatched types: expected `DropIndex`, found `Arc<DropIndex>`
- Line 102: mismatched types: expected `ShowIndexes`, found `Arc<ShowIndexes>`
- ... 1 more occurrences in this file

#### `src\query\optimizer\operation_merge.rs`: 3 occurrences

- Line 77: mismatched types: expected `Vec<usize>`, found `Vec<MatchedResult>`
- Line 126: mismatched types: expected `PlanNodeEnum`, found `Box<PlanNodeEnum>`
- Line 135: mismatched types: expected `PlanNodeEnum`, found `Arc<FilterNode>`

#### `src\query\planner\match_planning\clauses\unwind_planner.rs`: 3 occurrences

- Line 132: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- Line 132: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- Line 239: mismatched types: expected `Arc<dyn PlanNode>`, found `PlanNodeEnum`

#### `src\query\planner\plan\management\dml\delete_ops.rs`: 3 occurrences

- Line 25: mismatched types: expected `DeleteVertices`, found `Arc<DeleteVertices>`
- Line 52: mismatched types: expected `DeleteTags`, found `Arc<DeleteTags>`
- Line 74: mismatched types: expected `DeleteEdges`, found `Arc<DeleteEdges>`

#### `src\query\executor\cypher\context.rs`: 3 occurrences

- Line 255: mismatched types: expected `Option<&Vertex>`, found `Option<Vertex>`
- Line 265: mismatched types: expected `Option<&Edge>`, found `Option<Edge>`
- Line 331: mismatched types: expected `Option<&Value>`, found `Option<Value>`

#### `src\query\planner\ngql\maintain_planner.rs`: 3 occurrences

- Line 70: mismatched types: expected `PlanNodeEnum`, found `Arc<ArgumentNode>`
- Line 92: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`
- Line 93: mismatched types: expected `PlanNodeEnum`, found `Arc<ArgumentNode>`

#### `src\query\planner\ngql\fetch_edges_planner.rs`: 3 occurrences

- Line 65: mismatched types: expected `PlanNodeEnum`, found `Arc<GetEdgesNode>`
- Line 72: `match` arms have incompatible types: expected `Arc<FilterNode>`, found `Arc<GetEdgesNode>`
- Line 92: mismatched types: expected `PlanNodeEnum`, found `Arc<ArgumentNode>`

#### `src\query\planner\plan\management\admin\config_ops.rs`: 3 occurrences

- Line 51: mismatched types: expected `ShowConfigs`, found `Arc<ShowConfigs>`
- Line 87: mismatched types: expected `SetConfig`, found `Arc<SetConfig>`
- Line 117: mismatched types: expected `GetConfig`, found `Arc<GetConfig>`

#### `src\query\executor\result_processing\sort.rs`: 2 occurrences

- Line 200: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 286: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\executor\result_processing\topn.rs`: 2 occurrences

- Line 411: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`
- Line 427: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`

#### `src\query\planner\match_planning\paths\match_path_planner.rs`: 2 occurrences

- Line 220: mismatched types: expected `Arc<dyn PlanNode>`, found `PlanNodeEnum`
- Line 274: mismatched types: expected `Arc<dyn PlanNode>`, found `PlanNodeEnum`

#### `src\query\planner\plan\management\dml\insert_ops.rs`: 2 occurrences

- Line 25: mismatched types: expected `InsertVertices`, found `Arc<InsertVertices>`
- Line 47: mismatched types: expected `InsertEdges`, found `Arc<InsertEdges>`

#### `src\query\optimizer\rule_traits.rs`: 2 occurrences

- Line 427: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`
- Line 436: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`

#### `src\query\planner\plan\core\nodes\traversal_node.rs`: 2 occurrences

- Line 349: mismatched types: expected `Option<&String>`, found `Option<String>`
- Line 484: mismatched types: expected `Option<&String>`, found `Option<String>`

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 411: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 497: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\executor\result_processing\limit.rs`: 2 occurrences

- Line 204: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 290: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\planner\plan\management\dml\update_ops.rs`: 2 occurrences

- Line 33: mismatched types: expected `UpdateVertex`, found `Arc<UpdateVertex>`
- Line 63: mismatched types: expected `UpdateEdge`, found `Arc<UpdateEdge>`

#### `src\query\executor\result_processing\filter.rs`: 2 occurrences

- Line 217: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 303: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 2 occurrences

- Line 58: mismatched types: expected `Arc<dyn PlanNode>`, found `PlanNodeEnum`
- Line 61: mismatched types: expected `PlanNodeEnum`, found `Arc<dyn PlanNode>`

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 2 occurrences

- Line 283: mismatched types: expected `Vec<Vertex>`, found `Vec<Box<Vertex>>`
- Line 294: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 396: mismatched types: expected `Option<&ExecutionResult>`, found `Option<ExecutionResult>`
- Line 488: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 204: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 1 occurrences

- Line 868: mismatched types: expected `Option<&String>`, found `Option<String>`

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 349: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 799: mismatched types: expected `PlanNodeEnum`, found `Arc<ProjectNode>`

#### `src\query\visitor\evaluable_expr_visitor.rs`: 1 occurrences

- Line 73: mismatched types: expected `Option<&String>`, found `Option<String>`

#### `src\query\planner\match_planning\match_planner.rs`: 1 occurrences

- Line 138: mismatched types: expected `Option<&SubPlan>`, found `Option<SubPlan>`

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 88: mismatched types: expected `Option<&TypeDeductionError>`, found `Option<TypeDeductionError>`

#### `src\query\executor\data_processing\graph_traversal\expand_all.rs`: 1 occurrences

- Line 241: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 56: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`

#### `src\query\optimizer\optimizer.rs`: 1 occurrences

- Line 294: mismatched types: expected `&PlanNodeEnum`, found `PlanNodeEnum`

#### `src\query\executor\data_processing\graph_traversal\expand.rs`: 1 occurrences

- Line 172: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

### error[E0599]: no method named `plan_node` found for struct `OptGroupNode` in the current scope: field, not a method

**Total Occurrences**: 35  
**Unique Files**: 5

#### `src\query\optimizer\limit_pushdown.rs`: 12 occurrences

- Line 41: no method named `plan_node` found for struct `OptGroupNode` in the current scope: field, not a method
- Line 79: no method named `as_get_vertices` found for enum `plan_node_enum::PlanNodeEnum` in the current scope
- Line 102: no method named `as_get_edges` found for enum `plan_node_enum::PlanNodeEnum` in the current scope
- ... 9 more occurrences in this file

#### `src\query\optimizer\index_optimization.rs`: 9 occurrences

- Line 49: no method named `as_index_scan` found for enum `plan_node_enum::PlanNodeEnum` in the current scope
- Line 155: no method named `as_index_scan` found for enum `plan_node_enum::PlanNodeEnum` in the current scope
- Line 248: no method named `as_index_scan` found for enum `plan_node_enum::PlanNodeEnum` in the current scope
- ... 6 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 8 occurrences

- Line 55: no method named `as_any` found for reference `&plan_node_enum::PlanNodeEnum` in the current scope: method not found in `&PlanNodeEnum`
- Line 112: no method named `as_index_scan` found for reference `&plan_node_enum::PlanNodeEnum` in the current scope
- Line 170: no method named `as_any` found for reference `&plan_node_enum::PlanNodeEnum` in the current scope: method not found in `&PlanNodeEnum`
- ... 5 more occurrences in this file

#### `src\query\planner\plan\algorithms\path_algorithms.rs`: 4 occurrences

- Line 144: no method named `visit_multi_shortest_path` found for mutable reference `&mut V` in the current scope: method not found in `&mut V`
- Line 267: no method named `visit_bfs_shortest` found for mutable reference `&mut V` in the current scope
- Line 401: no method named `visit_all_paths` found for mutable reference `&mut V` in the current scope
- ... 1 more occurrences in this file

#### `src\query\planner\plan\algorithms\index_scan.rs`: 2 occurrences

- Line 135: no method named `visit_index_scan` found for mutable reference `&mut V` in the current scope
- Line 244: no method named `visit_fulltext_index_scan` found for mutable reference `&mut V` in the current scope: method not found in `&mut V`

### error[E0614]: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 18  
**Unique Files**: 3

#### `src\query\visitor\evaluable_expr_visitor.rs`: 6 occurrences

- Line 133: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 139: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 140: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- ... 3 more occurrences in this file

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

### error[E0277]: the trait bound `join_node::InnerJoinNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `join_node::InnerJoinNode`

**Total Occurrences**: 9  
**Unique Files**: 5

#### `src\query\planner\ngql\go_planner.rs`: 4 occurrences

- Line 112: the trait bound `join_node::InnerJoinNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `join_node::InnerJoinNode`
- Line 114: the trait bound `ExpandAllNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `ExpandAllNode`
- Line 138: the trait bound `join_node::InnerJoinNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `join_node::InnerJoinNode`
- ... 1 more occurrences in this file

#### `src\query\planner\ngql\subgraph_planner.rs`: 2 occurrences

- Line 69: the trait bound `ExpandAllNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `ExpandAllNode`
- Line 72: the trait bound `ExpandAllNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `ExpandAllNode`

#### `src\query\planner\ngql\path_planner.rs`: 1 occurrences

- Line 107: the trait bound `ExpandAllNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `ExpandAllNode`

#### `src\query\optimizer\optimizer.rs`: 1 occurrences

- Line 573: the trait bound `Box<dyn OptRule>: OptRule` is not satisfied: the trait `OptRule` is not implemented for `Box<dyn OptRule>`

#### `src\query\planner\match_planning\match_planner.rs`: 1 occurrences

- Line 123: a value of type `Vec<&dyn CypherClausePlanner>` cannot be built from an iterator over elements of type `&Box<dyn CypherClausePlanner>`: value of type `Vec<&dyn CypherClausePlanner>` cannot be built from `std::iter::Iterator<Item=&Box<dyn CypherClausePlanner>>`

### error[E0004]: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::HashInnerJoin(_)`, `&plan_node_enum::PlanNodeEnum::HashLeftJoin(_)`, `&plan_node_enum::PlanNodeEnum::CartesianProduct(_)` and 1 more not covered: patterns `&plan_node_enum::PlanNodeEnum::HashInnerJoin(_)`, `&plan_node_enum::PlanNodeEnum::HashLeftJoin(_)`, `&plan_node_enum::PlanNodeEnum::CartesianProduct(_)` and 1 more not covered

**Total Occurrences**: 8  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 8 occurrences

- Line 11: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::HashInnerJoin(_)`, `&plan_node_enum::PlanNodeEnum::HashLeftJoin(_)`, `&plan_node_enum::PlanNodeEnum::CartesianProduct(_)` and 1 more not covered: patterns `&plan_node_enum::PlanNodeEnum::HashInnerJoin(_)`, `&plan_node_enum::PlanNodeEnum::HashLeftJoin(_)`, `&plan_node_enum::PlanNodeEnum::CartesianProduct(_)` and 1 more not covered
- Line 116: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::HashInnerJoin(_)`, `&plan_node_enum::PlanNodeEnum::HashLeftJoin(_)`, `&plan_node_enum::PlanNodeEnum::CartesianProduct(_)` and 1 more not covered: patterns `&plan_node_enum::PlanNodeEnum::HashInnerJoin(_)`, `&plan_node_enum::PlanNodeEnum::HashLeftJoin(_)`, `&plan_node_enum::PlanNodeEnum::CartesianProduct(_)` and 1 more not covered
- Line 209: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::HashInnerJoin(_)`, `&plan_node_enum::PlanNodeEnum::HashLeftJoin(_)`, `&plan_node_enum::PlanNodeEnum::CartesianProduct(_)` and 1 more not covered: patterns `&plan_node_enum::PlanNodeEnum::HashInnerJoin(_)`, `&plan_node_enum::PlanNodeEnum::HashLeftJoin(_)`, `&plan_node_enum::PlanNodeEnum::CartesianProduct(_)` and 1 more not covered
- ... 5 more occurrences in this file

### error[E0615]: attempted to take value of method `plan_node` on type `&MatchedResult`: method, not a field

**Total Occurrences**: 7  
**Unique Files**: 1

#### `src\query\optimizer\operation_merge.rs`: 7 occurrences

- Line 37: attempted to take value of method `plan_node` on type `&MatchedResult`: method, not a field
- Line 41: attempted to take value of method `plan_node` on type `&MatchedResult`: method, not a field
- Line 172: attempted to take value of method `plan_node` on type `&MatchedResult`: method, not a field
- ... 4 more occurrences in this file

### error[E0515]: cannot return value referencing local variable `deps`: returns a value referencing data owned by the current function

**Total Occurrences**: 4  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\plan_node_traits.rs`: 4 occurrences

- Line 42: cannot return value referencing local variable `deps`: returns a value referencing data owned by the current function
- Line 52: cannot return value referencing local variable `deps`: returns a value referencing data owned by the current function
- Line 62: cannot return value referencing local variable `deps`: returns a value referencing data owned by the current function
- ... 1 more occurrences in this file

### error[E0382]: borrow of moved value: `plan.root`: value borrowed here after move

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\planner\match_planning\clauses\yield_planner.rs`: 2 occurrences

- Line 86: borrow of moved value: `plan.root`: value borrowed here after move
- Line 117: borrow of moved value: `plan.root`: value borrowed here after move

### error[E0412]: cannot find type `ScanVertices` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\predicate_pushdown.rs`: 1 occurrences

- Line 56: cannot find type `ScanVertices` in this scope: not found in this scope

## Detailed Warning Categorization

### warning: unused import: `HierarchicalContext`

**Total Occurrences**: 72  
**Unique Files**: 39

#### `src\query\planner\plan\core\nodes\factory.rs`: 14 occurrences

- Line 10: unused import: `super::filter_node::FilterNode`
- Line 15: unused import: `super::project_node::ProjectNode`
- Line 37: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- ... 11 more occurrences in this file

#### `src\core\evaluator\expression_evaluator.rs`: 8 occurrences

- Line 243: variable does not need to be mutable
- Line 311: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 311: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- ... 5 more occurrences in this file

#### `src\query\optimizer\elimination_rules.rs`: 3 occurrences

- Line 4: unused import: `std::sync::Arc`
- Line 11: unused imports: `AppendVerticesNode`, `DedupNode`, `FilterNode`, `GetEdgesNode`, `GetVerticesNode`, `InnerJoinNode`, `LeftJoinNode`, `LimitNode`, `ScanEdgesNode`, `ScanVerticesNode`, `SortNode`, and `StartNode`
- Line 869: unused import: `crate::query::planner::plan::algorithms::IndexScan`

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 83: value assigned to `index_scan_node` is never read
- Line 119: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 3 occurrences

- Line 4: unused import: `VisitorResult`
- Line 343: unused variable: `func`: help: if this is intentional, prefix it with an underscore: `_func`
- Line 467: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 314: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 314: variable does not need to be mutable

#### `src\core\context\runtime.rs`: 2 occurrences

- Line 17: unused imports: `MemoryIndexManager` and `MemorySchemaManager`
- Line 18: unused import: `crate::storage::native_storage::NativeStorage`

#### `src\core\evaluator\traits.rs`: 2 occurrences

- Line 30: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 30: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\core\context\query.rs`: 2 occurrences

- Line 7: unused import: `QueryResult`
- Line 8: unused import: `crate::core::Value`

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 283: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`
- Line 283: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 occurrences

- Line 5: unused import: `crate::query::context::validate::types::Variable`
- Line 525: unused import: `super::plan_node_operations::*`

#### `src\core\context\mod.rs`: 2 occurrences

- Line 20: ambiguous glob re-exports: the name `SessionVariable` in the type namespace is first re-exported here
- Line 22: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\core\context\validation.rs`: 1 occurrences

- Line 9: unused import: `HierarchicalContext`

#### `src\core\query_pipeline_manager.rs`: 1 occurrences

- Line 117: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\query\planner\plan\core\nodes\mod.rs`: 1 occurrences

- Line 33: unused import: `plan_node_operations::*`

#### `src\query\visitor\mod.rs`: 1 occurrences

- Line 5: unused import: `crate::core::visitor::VisitorConfig`

#### `src\cache\global_manager.rs`: 1 occurrences

- Line 5: unused import: `CacheStrategy`

#### `src\query\planner\plan\management\ddl\edge_ops.rs`: 1 occurrences

- Line 5: unused import: `crate::query::context::validate::types::Variable`

#### `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 1 occurrences

- Line 103: unused variable: `path`: help: if this is intentional, prefix it with an underscore: `_path`

#### `src\core\context\request.rs`: 1 occurrences

- Line 10: unused import: `HierarchicalContext`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 9: unused import: `crate::query::planner::plan::core::nodes::FilterNode`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 8: unused import: `IndexScan`

#### `src\core\mod.rs`: 1 occurrences

- Line 47: ambiguous glob re-exports: the name `SymbolType` in the type namespace is first re-exported here

#### `src\core\context\base.rs`: 1 occurrences

- Line 102: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\expression\cypher\expression_converter.rs`: 1 occurrences

- Line 3: unused imports: `BinaryOperator` and `UnaryOperator`

#### `src\query\planner\plan\core\nodes\traversal_node.rs`: 1 occurrences

- Line 8: unused import: `Arc`

#### `src\core\context\session.rs`: 1 occurrences

- Line 8: unused import: `crate::core::Value`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 8: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 8: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\planner\plan\core\nodes\sort_node.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\core\context\execution.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Value`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `VisitorResult`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::core::expressions::ExpressionContext`

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 400: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`

#### `src\cache\parser_cache.rs`: 1 occurrences

- Line 13: unused import: `std::time::Duration`

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 1 occurrences

- Line 5: unused import: `crate::core::ValueTypeDef`

#### `src\query\optimizer\optimizer.rs`: 1 occurrences

- Line 7: unused import: `std::sync::Arc`

#### `src\cache\manager.rs`: 1 occurrences

- Line 12: unused import: `super::traits::StatsCache`

#### `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 1 occurrences

- Line 9: unused import: `Arc`

