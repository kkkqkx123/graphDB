# Optimizer Heuristic Visitor Panic Analysis

> Date: 2026-05-27
> Scope: `crates/graphdb-query/src/query/optimizer/heuristic/visitor.rs`

## Problem

Multiple DELETE operations with pipe syntax or MATCH pattern syntax cause a panic:

```
internal error: entered unreachable code: visit_default should not be called
  - all node types should have specific visit methods
```

## Affected Tests (10 tests)

```
dml::delete::test_match_delete_vertex_execution
dml::delete::test_match_delete_vertex_with_edge_execution
dml::delete::test_match_delete_with_limit
dml::delete::test_match_delete_with_pattern_execution
dml::delete::test_pipe_delete_edge_execution
dml::delete::test_pipe_delete_vertex_execution
dml::delete::test_pipe_delete_vertex_with_edge_execution
dml::delete::test_pipe_delete_with_where_clause
```

## Location

`crates/graphdb-query/src/query/optimizer/heuristic/visitor.rs:185`

```rust
fn visit_default(&mut self, _node: &PlanNodeRef) -> VisitResult {
    unreachable!("visit_default should not be called - all node types should have specific visit methods");
}
```

## Root Cause

The heuristic optimizer visitor uses a `define_opt_visitor!` macro or manual dispatch that
maps each `PlanNodeType` variant to a `visit_*` method. When a plan node type is added to the
plan (e.g., for new DML operations like MATCH DELETE or PIPE DELETE) but the corresponding
`visit_*` method is not added to the visitor, the dispatch falls through to `visit_default`.

The `visit_default` implementation calls `unreachable!()`, which panics at runtime.

## Fix

Add `visit_*` methods for the missing plan node types in the heuristic visitor.
The visitor dispatch needs to handle all `PlanNodeType` variants that DELETE operations
can produce (likely `DeleteNode` or similar).

## Temporary Workaround

Replace `unreachable!()` with a no-op pass-through in `visit_default`:

```rust
fn visit_default(&mut self, node: &PlanNodeRef) -> VisitResult {
    // TODO: Add specific visit methods for missing node types
    self.visit_children(node)
}
```

This prevents the panic and allows the query to run unoptimized (using the default plan)
but does not fix the underlying missing optimization logic.

## Risk

- Panic prevents any MATCH/PIPE DELETE query from executing
- No workaround at query level (cannot disable optimizer for specific queries)
- User-facing impact: DELETE with pipe syntax crashes the server
