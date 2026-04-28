# Code Analysis: GO Traversal Variable Binding Issue

## Problem Summary

GO traversal queries fail with `UndefinedVariable` error when trying to reference edge properties in the YIELD clause (e.g., `friend.name` in `GO 1 STEP FROM "p1" OVER friend YIELD friend.name`).

## Error Location

**Error Type**: `ExpressionErrorType::UndefinedVariable`
**Error Message**: `Undefined variable: friend`
**Error Source**: `src/core/error/expression.rs`

## Code Flow Analysis

### 1. Query Planning Phase

**File**: `src/query/planning/statements/dql/go_planner.rs`

The GO planner creates an `ExpandAllNode` for traversal:

```rust
// Create ExpandAllNode to traverse edges
let mut expand_all_node = ExpandAllNode::new(1, edge_types.clone(), direction_str);

// Set column names to match ExpandAll's output format: [src, edge, dst]
let mut col_names = vec![
    "src".to_string(),
    "edge".to_string(),
    "dst".to_string(),
];
// Add edge type as alias for "edge" column to support friend.name syntax
if edge_types.len() == 1 {
    col_names.push(edge_types[0].clone());
}
expand_all_node.set_col_names(col_names);
```

**Key Observation**: The planner attempts to support `friend.name` syntax by adding the edge type name as an alias column. However, this may not be sufficient for proper variable binding.

### 2. Projection Phase

**File**: `src/query/executor/relational_algebra/projection.rs`

The `ProjectExecutor.project_row()` method has special handling for GO queries:

```rust
// Map GO query special variables: $$ -> dst, $^ -> src, target -> dst, edge -> edge
if let Some(edge_idx) = col_names.iter().position(|c| c == "edge") {
    if edge_idx < row.len() {
        context.set_variable("edge".to_string(), row[edge_idx].clone());
        // Map edge type name to the edge value for GO queries like YIELD friend.name
        if let Value::Edge(ref edge_val) = row[edge_idx] {
            context.set_variable(edge_val.edge_type().to_string(), row[edge_idx].clone());
        }
    }
}
```

**Issue**: The code attempts to bind the edge type name (e.g., "friend") to the edge value, but this depends on:
1. The "edge" column being present in `col_names`
2. The row containing an `Edge` value at the edge index
3. The edge value having the correct `edge_type()`

### 3. YIELD Column Building

**File**: `src/query/planning/statements/dql/go_planner.rs` (lines 208-250)

```rust
fn build_yield_columns(
    go_stmt: &GoStmt,
    expr_context: &Arc<ExpressionAnalysisContext>,
) -> Result<Vec<crate::core::YieldColumn>, PlannerError> {
    let mut columns = Vec::new();

    if let Some(ref yield_clause) = go_stmt.yield_clause {
        for item in &yield_clause.items {
            columns.push(crate::core::YieldColumn {
                expression: item.expression.clone(),
                alias: item.alias.clone().unwrap_or_default(),
                is_matched: false,
            }));
        }
    }
    // ...
}
```

**Root Cause Analysis**:

The problem is a mismatch between:
1. **What the user writes**: `YIELD friend.name` (expecting `friend` to be the edge variable)
2. **What the system provides**: The edge is stored in a column named "edge" or the edge type name

The current implementation tries to map the edge type name to the edge value in `project_row()`, but this may fail because:

1. The `col_names` passed to `project_row()` may not include "edge" or the edge type name
2. The row data structure may not match the expected format
3. The variable binding happens too late (in projection rather than in the traversal executor)

## Key Code Locations

| File | Line | Description |
|------|------|-------------|
| `go_planner.rs` | 100-115 | Sets col_names including edge type alias |
| `go_planner.rs` | 208-250 | `build_yield_columns()` creates YIELD columns |
| `projection.rs` | 220-227 | Maps edge type name to edge value |

## Potential Fix

### Option 1: Fix Variable Binding in ExpandAll Executor

Ensure the `ExpandAllExecutor` (or equivalent traversal executor) properly binds the edge type name as a variable in the execution context before passing data to downstream operators.

### Option 2: Fix Column Name Propagation

Ensure that when `col_names` includes the edge type name (e.g., "friend"), the corresponding row data contains the edge value at that position.

### Option 3: Update Projection Logic

In `project_row()`, ensure the edge type name is correctly mapped even when the column structure differs:

```rust
// Current logic (may fail):
if let Value::Edge(ref edge_val) = row[edge_idx] {
    context.set_variable(edge_val.edge_type().to_string(), row[edge_idx].clone());
}

// Potential fix: Check if edge type name is in expected yield columns
for col_name in &self.columns {
    // Check if col_name refers to an edge type and bind accordingly
}
```

## Related Files

- `src/query/planning/statements/dql/go_planner.rs` - GO query planning
- `src/query/executor/relational_algebra/projection.rs` - Projection execution with GO special handling
- `src/query/executor/graph_operations/graph_traversal/expand_all.rs` - Traversal execution (if exists)
