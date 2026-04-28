# Code Analysis: MATCH Query Variable Binding Issue

## Problem Summary

MATCH queries fail with `UndefinedVariable` error when trying to reference variables defined in the MATCH pattern (e.g., `p` in `MATCH (p:person)`).

## Error Location

**Error Type**: `ExpressionErrorType::UndefinedVariable`
**Error Source**: `src/core/error/expression.rs`

## Code Flow Analysis

### 1. Query Planning Phase

**File**: `src/query/planning/statements/match_statement_planner.rs`

The MATCH statement planner creates a `ScanVerticesNode` for the first node in the pattern:

```rust
fn plan_pattern_node(
    &self,
    node: &crate::query::parser::ast::pattern::NodePattern,
    space_id: u64,
    space_name: &str,
) -> Result<SubPlan, PlannerError> {
    let mut scan_node = ScanVerticesNode::new(space_id, space_name);
    // Set the column name to the node variable name
    let var_name = node.variable.clone().unwrap_or_else(|| "n".to_string());
    scan_node.set_col_names(vec![var_name.clone()]);
    scan_node.set_output_var(var_name);
    let mut plan = SubPlan::from_root(scan_node.into_enum());
    Ok(plan)
}
```

**Issue**: The planner sets `col_names` to the variable name (e.g., `["p"]`), but the `ScanVerticesExecutor` returns rows with a single column containing the vertex value, not binding the variable name to the vertex.

### 2. Query Execution Phase

**File**: `src/query/executor/data_access/vertex.rs`

The `ScanVerticesExecutor.execute()` method returns:

```rust
fn execute(&mut self) -> DBResult<ExecutionResult> {
    // ...
    let rows: Vec<Vec<Value>> = vertices
        .into_iter()
        .map(|v| vec![Value::Vertex(Box::new(v))])
        .collect();
    let dataset = DataSet::from_rows(rows, self.col_names.clone());
    Ok(ExecutionResult::DataSet(dataset))
}
```

**Issue**: The executor creates rows with column names from `self.col_names`, but when `col_names` is `["p"]`, the data is a vertex value. However, subsequent operators (like Filter or Project) expect to access properties via `p.name`, which requires the variable `p` to be bound to the vertex.

### 3. Projection Phase

**File**: `src/query/executor/relational_algebra/projection.rs`

The `ProjectExecutor.project_row()` method sets up the expression context:

```rust
fn project_row(&self, row: &[Value], col_names: &[String]) -> DBResult<Vec<Value>> {
    let mut context = DefaultExpressionContext::new();

    // Set the value of the current row to the context variable
    for (i, col_name) in col_names.iter().enumerate() {
        if i < row.len() {
            context.set_variable(col_name.clone(), row[i].clone());
        }
    }
    // ...
}
```

**Root Cause**:

1. When `ScanVerticesNode` sets `col_names = ["p"]`, the `ScanVerticesExecutor` creates a dataset with column name "p" and value as the vertex.

2. However, when the `ProjectExecutor` processes this data, it sets `context.set_variable("p", vertex_value)`, which should work.

3. **The actual problem** appears to be in how the variable is propagated through the execution pipeline. The `ScanVerticesNode` sets `output_var`, but this may not be properly used by downstream operators.

## Key Code Locations

| File                         | Line    | Description                                         |
| ---------------------------- | ------- | --------------------------------------------------- |
| `match_statement_planner.rs` | 401-406 | `plan_pattern_node()` sets col_names and output_var |
| `vertex.rs`                  | 295-308 | `ScanVerticesExecutor.execute()` creates dataset    |
| `projection.rs`              | 208-225 | `project_row()` sets up expression context          |

## Potential Fix

The issue may be in how the execution context is passed between operators. The `ScanVerticesExecutor` should ensure that:

1. The variable name (e.g., "p") is properly bound to the vertex value in the execution context
2. This binding is accessible to downstream operators like Filter and Project

Currently, the data flows as:

```
ScanVerticesExecutor -> DataSet (col_names=["p"], rows=[[Vertex]])
  -> ProjectExecutor -> reads col_names and sets variables
```

But the variable binding may be lost when the `DataSet` is passed through intermediate operators or when the context is not properly shared.

## Related Files

- `src/query/planning/statements/match_statement_planner.rs` - MATCH query planning
- `src/query/executor/data_access/vertex.rs` - Vertex scan execution
- `src/query/executor/relational_algebra/projection.rs` - Projection execution
- `src/query/executor/base/execution_context.rs` - Execution context management
