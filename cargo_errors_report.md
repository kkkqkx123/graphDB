# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 19
- **Total Warnings**: 67
- **Total Issues**: 86
- **Unique Error Patterns**: 17
- **Unique Warning Patterns**: 41
- **Files with Issues**: 43

## Error Statistics

**Total Errors**: 19

### Error Type Breakdown

- **error[E0599]**: 9 errors
- **error[E0277]**: 4 errors
- **error[E0433]**: 3 errors
- **error[E0308]**: 1 errors
- **error[E0609]**: 1 errors
- **error[E0603]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 6 errors
- `src\query\planner\plan\core\explain.rs`: 4 errors
- `src\query\parser\cypher\expression_evaluator.rs`: 3 errors
- `src\expression\context\evaluation.rs`: 2 errors
- `src\expression\context\basic_context.rs`: 2 errors
- `src\query\parser\cypher\cypher_processor.rs`: 1 errors
- `src\query\visitor\mod.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 67

### Warning Type Breakdown

- **warning**: 67 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 12 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 8 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\expression\evaluator\traits.rs`: 2 warnings
- `src\query\executor\result_processing\aggregation.rs`: 2 warnings
- `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 warnings
- `src\core\context\mod.rs`: 2 warnings
- `src\query\planner\ngql\go_planner.rs`: 2 warnings
- `src\query\planner\plan\management\ddl\edge_ops.rs`: 2 warnings
- `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 warnings

## Detailed Error Categorization

### error[E0599]: no method named `evaluate_cypher` found for struct `evaluator::expression_evaluator::ExpressionEvaluator` in the current scope

**Total Occurrences**: 9  
**Unique Files**: 3

#### `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 6 occurrences

- Line 41: no method named `evaluate_cypher` found for struct `evaluator::expression_evaluator::ExpressionEvaluator` in the current scope
- Line 59: no method named `evaluate_cypher_batch` found for struct `evaluator::expression_evaluator::ExpressionEvaluator` in the current scope
- Line 118: no method named `is_cypher_constant` found for struct `evaluator::expression_evaluator::ExpressionEvaluator` in the current scope: method not found in `ExpressionEvaluator`
- ... 3 more occurrences in this file

#### `src\query\parser\cypher\expression_evaluator.rs`: 2 occurrences

- Line 47: no function or associated item named `not_implemented` found for struct `expression::context::error::ExpressionError` in the current scope: function or associated item not found in `ExpressionError`
- Line 94: no function or associated item named `not_implemented` found for struct `expression::context::error::ExpressionError` in the current scope: function or associated item not found in `ExpressionError`

#### `src\query\visitor\mod.rs`: 1 occurrences

- Line 65: no function or associated item named `with_config` found for struct `DeducePropsVisitor` in the current scope: function or associated item not found in `DeducePropsVisitor`

### error[E0277]: the trait bound `control_flow_node::ArgumentNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `control_flow_node::ArgumentNode`

**Total Occurrences**: 4  
**Unique Files**: 1

#### `src\query\planner\plan\core\explain.rs`: 4 occurrences

- Line 298: the trait bound `control_flow_node::ArgumentNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `control_flow_node::ArgumentNode`
- Line 302: the trait bound `control_flow_node::LoopNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `control_flow_node::LoopNode`
- Line 306: the trait bound `control_flow_node::PassThroughNode: plan_node_traits::PlanNode` is not satisfied: the trait `plan_node_traits::PlanNode` is not implemented for `control_flow_node::PassThroughNode`
- ... 1 more occurrences in this file

### error[E0433]: failed to resolve: could not find `cache` in `super`: could not find `cache` in `super`

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\expression\context\evaluation.rs`: 2 occurrences

- Line 52: failed to resolve: could not find `cache` in `super`: could not find `cache` in `super`
- Line 101: failed to resolve: could not find `cache` in `super`: could not find `cache` in `super`

#### `src\query\parser\cypher\cypher_processor.rs`: 1 occurrences

- Line 43: failed to resolve: could not find `evaluator` in `core`: could not find `evaluator` in `core`

### error[E0609]: no field `props` on type `Box<vertex_edge_path::Vertex>`: unknown field

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\parser\cypher\expression_evaluator.rs`: 1 occurrences

- Line 34: no field `props` on type `Box<vertex_edge_path::Vertex>`: unknown field

### error[E0603]: trait import `ExpressionContext` is private: private trait import

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 418: trait import `ExpressionContext` is private: private trait import

### error[E0308]: mismatched types: expected `core::error::ExpressionError`, found `ExpressionError`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 334: mismatched types: expected `core::error::ExpressionError`, found `ExpressionError`

## Detailed Warning Categorization

### warning: unused import: `std::sync::Arc`

**Total Occurrences**: 67  
**Unique Files**: 39

#### `src\query\planner\plan\core\nodes\factory.rs`: 12 occurrences

- Line 35: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 31: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- Line 49: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- ... 9 more occurrences in this file

#### `src\expression\evaluator\expression_evaluator.rs`: 8 occurrences

- Line 242: variable does not need to be mutable
- Line 310: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 310: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- ... 5 more occurrences in this file

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 127: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\core\context\mod.rs`: 2 occurrences

- Line 20: ambiguous glob re-exports: the name `SessionVariable` in the type namespace is first re-exported here
- Line 22: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 occurrences

- Line 5: unused import: `crate::query::context::validate::types::Variable`
- Line 746: unused import: `super::plan_node_operations::*`

#### `src\query\planner\plan\management\ddl\edge_ops.rs`: 2 occurrences

- Line 5: unused import: `crate::query::context::validate::types::Variable`
- Line 8: unused import: `std::sync::Arc`

#### `src\query\planner\ngql\go_planner.rs`: 2 occurrences

- Line 11: unused import: `std::sync::Arc`
- Line 59: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 319: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 319: variable does not need to be mutable

#### `src\query\planner\match_planning\clauses\unwind_planner.rs`: 2 occurrences

- Line 1: unused import: `std::sync::Arc`
- Line 53: unused import: `crate::query::planner::plan::factory::PlanNodeFactory`

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 284: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`
- Line 284: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\expression\evaluator\traits.rs`: 2 occurrences

- Line 30: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 30: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\query\planner\plan\management\dml\insert_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 862: unused import: `crate::query::planner::plan::algorithms::IndexScan`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 10: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\dml\data_constructors.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\query\planner\plan\management\admin\index_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 1 occurrences

- Line 5: unused import: `crate::core::ValueTypeDef`

#### `src\query\planner\plan\management\dml\update_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\admin\config_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 12: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\admin\system_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\management\ddl\tag_ops.rs`: 1 occurrences

- Line 8: unused import: `std::sync::Arc`

#### `src\core\mod.rs`: 1 occurrences

- Line 44: ambiguous glob re-exports: the name `SymbolType` in the type namespace is first re-exported here

#### `src\query\planner\ngql\fetch_edges_planner.rs`: 1 occurrences

- Line 4: unused import: `std::sync::Arc`

#### `src\query\visitor\mod.rs`: 1 occurrences

- Line 5: unused import: `crate::core::visitor::VisitorConfig`

#### `src\query\planner\plan\management\dml\delete_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\core\nodes\sort_node.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\ngql\fetch_vertices_planner.rs`: 1 occurrences

- Line 5: unused import: `Column`

#### `src\query\planner\plan\management\admin\host_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\expression\visitor.rs`: 1 occurrences

- Line 286: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 6: unused import: `ZeroInputNode`

#### `src\query\planner\plan\management\ddl\space_ops.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 1 occurrences

- Line 103: unused variable: `path`: help: if this is intentional, prefix it with an underscore: `_path`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 51: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\core\query_pipeline_manager.rs`: 1 occurrences

- Line 117: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\query\planner\plan\core\nodes\mod.rs`: 1 occurrences

- Line 32: unused import: `plan_node_operations::*`

#### `src\query\parser\cypher\expression_evaluator.rs`: 1 occurrences

- Line 6: unused imports: `BinaryOperator` and `UnaryOperator`

