# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 1
- **Total Warnings**: 1
- **Total Issues**: 2
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 1
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 1

### Error Type Breakdown

- **error[E0432]**: 1 errors

### Files with Errors (Top 10)

- `src\query\planner\plan\core\mod.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\plan_node_visitor.rs`: 1 warnings

## Detailed Error Categorization

### error[E0432]: unresolved import `nodes::plan_node_enum::PlanNodeVisitor`: no `PlanNodeVisitor` in `query::planner::plan::core::nodes::plan_node_enum`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\plan\core\mod.rs`: 1 occurrences

- Line 11: unresolved import `nodes::plan_node_enum::PlanNodeVisitor`: no `PlanNodeVisitor` in `query::planner::plan::core::nodes::plan_node_enum`

## Detailed Warning Categorization

### warning: unused imports: `InsertEdgesNode` and `InsertVerticesNode`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\plan_node_visitor.rs`: 1 occurrences

- Line 12: unused imports: `InsertEdgesNode` and `InsertVerticesNode`

