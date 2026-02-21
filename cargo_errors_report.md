# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 23
- **Total Warnings**: 1
- **Total Issues**: 24
- **Unique Error Patterns**: 9
- **Unique Warning Patterns**: 1
- **Files with Issues**: 14

## Error Statistics

**Total Errors**: 23

### Error Type Breakdown

- **error[E0433]**: 11 errors
- **error[E0599]**: 7 errors
- **error[E0308]**: 3 errors
- **error[E0432]**: 1 errors
- **error[E0422]**: 1 errors

### Files with Errors (Top 10)

- `src\query\context\execution\query_execution.rs`: 8 errors
- `src\query\executor\admin\space\alter_space.rs`: 3 errors
- `src\query\planner\plan\core\nodes\traversal_node.rs`: 2 errors
- `src\query\planner\statements\fetch_vertices_planner.rs`: 1 errors
- `src\query\planner\plan\core\nodes\control_flow_node.rs`: 1 errors
- `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 errors
- `src\query\planner\plan\algorithms\path_algorithms.rs`: 1 errors
- `src\query\visitor\deduce_type_visitor.rs`: 1 errors
- `src\api\service\query_processor.rs`: 1 errors
- `src\query\query_pipeline_manager.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\query\validator\with_validator.rs`: 1 warnings

## Detailed Error Categorization

### error[E0433]: failed to resolve: could not find `validate` in `context`: could not find `validate` in `context`

**Total Occurrences**: 11  
**Unique Files**: 10

#### `src\query\planner\plan\core\nodes\traversal_node.rs`: 2 occurrences

- Line 264: failed to resolve: could not find `validate` in `context`: could not find `validate` in `context`
- Line 318: failed to resolve: could not find `validate` in `context`: could not find `validate` in `context`

#### `src\query\planner\plan\core\nodes\data_processing_node.rs`: 1 occurrences

- Line 6: failed to resolve: could not find `validate` in `context`: could not find `validate` in `context`

#### `src\query\planner\plan\core\nodes\plan_node_traits.rs`: 1 occurrences

- Line 7: failed to resolve: could not find `validate` in `context`: could not find `validate` in `context`

#### `src\query\planner\plan\algorithms\path_algorithms.rs`: 1 occurrences

- Line 6: failed to resolve: could not find `validate` in `context`: could not find `validate` in `context`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 7: failed to resolve: could not find `validate` in `context`: could not find `validate` in `context`

#### `src\query\planner\statements\fetch_vertices_planner.rs`: 1 occurrences

- Line 5: failed to resolve: could not find `validate` in `context`: could not find `validate` in `context`

#### `src\query\planner\plan\algorithms\index_scan.rs`: 1 occurrences

- Line 4: failed to resolve: could not find `validate` in `context`: could not find `validate` in `context`

#### `src\api\service\query_processor.rs`: 1 occurrences

- Line 52: failed to resolve: could not find `validate` in `context`: could not find `validate` in `context`

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 231: failed to resolve: could not find `validate` in `context`: could not find `validate` in `context`

#### `src\query\planner\plan\core\nodes\control_flow_node.rs`: 1 occurrences

- Line 7: failed to resolve: could not find `validate` in `context`: could not find `validate` in `context`

### error[E0599]: no variant or associated item named `PartitionNum` found for enum `alter_space::SpaceAlterOption` in the current scope: variant or associated item not found in `SpaceAlterOption`

**Total Occurrences**: 7  
**Unique Files**: 2

#### `src\query\context\execution\query_execution.rs`: 4 occurrences

- Line 491: no method named `id` found for reference `&execution_plan::ExecutionPlan` in the current scope: field, not a method
- Line 492: no method named `is_profile_enabled` found for reference `&execution_plan::ExecutionPlan` in the current scope: method not found in `&ExecutionPlan`
- Line 497: no method named `enable_profile` found for mutable reference `&mut Box<execution_plan::ExecutionPlan>` in the current scope: method not found in `&mut Box<ExecutionPlan>`
- ... 1 more occurrences in this file

#### `src\query\executor\admin\space\alter_space.rs`: 3 occurrences

- Line 110: no variant or associated item named `PartitionNum` found for enum `alter_space::SpaceAlterOption` in the current scope: variant or associated item not found in `SpaceAlterOption`
- Line 111: no variant or associated item named `ReplicaFactor` found for enum `alter_space::SpaceAlterOption` in the current scope: variant or associated item not found in `SpaceAlterOption`
- Line 145: no variant or associated item named `PartitionNum` found for enum `alter_space::SpaceAlterOption` in the current scope: variant or associated item not found in `SpaceAlterOption`

### error[E0308]: mismatched types: expected `Option<PlanNodeEnum>`, found `i64`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\context\execution\query_execution.rs`: 3 occurrences

- Line 487: mismatched types: expected `Option<PlanNodeEnum>`, found `i64`
- Line 537: mismatched types: expected `Option<PlanNodeEnum>`, found `i64`
- Line 554: mismatched types: expected `Option<PlanNodeEnum>`, found `i64`

### error[E0422]: cannot find struct, variant or union type `PlanNode` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 646: cannot find struct, variant or union type `PlanNode` in this scope: not found in this scope

### error[E0432]: unresolved import `crate::query::context::validate`: could not find `validate` in `context`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 9: unresolved import `crate::query::context::validate`: could not find `validate` in `context`

## Detailed Warning Categorization

### warning: unused variable: `where_expr`: help: if this is intentional, prefix it with an underscore: `_where_expr`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\with_validator.rs`: 1 occurrences

- Line 364: unused variable: `where_expr`: help: if this is intentional, prefix it with an underscore: `_where_expr`

