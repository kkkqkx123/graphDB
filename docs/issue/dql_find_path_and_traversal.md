# DQL Graph Traversal Execution Failure Analysis

> Date: 2026-05-27
> Scope: FIND PATH, MATCH (edge), GO traversal

## Problem

All graph traversal queries return 0 results. Only single-tag `MATCH (p:Person)` (vertex scan)
and `LOOKUP` (index scan) produce results.

## Symptoms

### FIND PATH — 6 failing tests

All `FIND SHORTEST PATH` and `FIND ALL PATH` queries return 0 rows
even when the edge topology is set up correctly:

```
FIND SHORTEST PATH FROM 1 TO 2 OVER KNOWS   → 0 rows (edge 1→2 exists)
FIND ALL PATH FROM 1 TO 4 OVER KNOWS         → 0 rows (diamond topology)
FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS UPTO 2 STEPS → ok (empty expected)
```

Parser passes for all FIND PATH syntax variants.

### MATCH with edge — 7 failing tests

```
MATCH (v1:Person)-[:KNOWS]->(v2:Person) RETURN v1.name   → 0 rows
MATCH (v1)-[:KNOWS]->(v2)-[:KNOWS]->(v3) RETURN v1.name  → 0 rows
MATCH (v1:Person)-[:KNOWS|:FRIEND]->(v2) RETURN v1.name   → 0 rows
MATCH (v1)-[:KNOWS]->(v1) RETURN v1.name                  → 0 rows (self-loop)
```

Single-tag `MATCH (p:Person) RETURN p.name` works correctly (returns all vertices).

### GO traversal — 5 failing tests

```
GO 1 STEPS FROM 1 OVER KNOWS              → 0 rows
GO 1 STEPS FROM 1 OVER KNOWS REVERSELY    → 0 rows
GO 1 STEPS FROM 1 OVER KNOWS BIDIRECT     → 0 rows
GO 1 STEPS FROM 1 OVER KNOWS YIELD ...    → 0 rows
GO 2 STEPS FROM 1 OVER KNOWS              → ok (multi-step possibly uses different path)
```

## Root Cause Analysis

### Pre-requisite: Edge persistence (see `dml_edge_persistence.md`)

All traversal tests depend on edges being retrievable. Since `INSERT EDGE` does not persist
edges correctly, traversal naturally returns 0 results. **Fixing edge persistence is a prerequisite**
for any traversal fix.

### Secondary: Executor chain not including traversal operators

Even if edges were persisted, the existing `current_issues_and_pending_tasks.md` documents
a separate bug where `FilterExecutor` is excluded from the executor chain during optimization.
A similar issue may affect `ExpandExecutor` (edge traversal) — the optimizer may be dropping
or bypassing traversal plan nodes.

**Evidence**: `visit_default should not be called` panic in `optimizer/heuristic/visitor.rs:185`
when executing MATCH DELETE patterns. This indicates the optimizer visitor encounters
unexpected plan node types, suggesting the plan tree structure is incorrect for traversal operations.

### Investigation Points

| Component | File | Question |
|-----------|------|----------|
| Edge expand executor | `src/query/executor/relational_algebra/expand/` | Is expand executed? Does it find edges? |
| Plan building for MATCH | `src/query/planning/planner/match/` | Does the planner generate correct ExpandNode? |
| Optimizer rewrite | `src/query/optimizer/heuristic/visitor.rs` | Does optimizer drop or reorder expand nodes? |
| Executor factory | `src/query/executor/factory/engine.rs` | Are expand executors included in the chain? |

## How to Debug

1. Fix edge persistence first (see `dml_edge_persistence.md`)
2. Add `log::info!` in `ExpandExecutor::execute()` to confirm it's called
3. Run `EXPLAIN MATCH (v1)-[:KNOWS]->(v2) RETURN v1.name` to see the plan
4. Check if `ExpandNode` appears in the optimized plan

## Reproducer

```rust
let mut scenario = TestScenario::new()
    .expect("Failed")
    .setup_space("test")
    .exec_ddl("CREATE TAG Person(name STRING)")
    .exec_ddl("CREATE EDGE KNOWS(since DATE)")
    .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')")
    .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')")
    .assert_success()
    .query("MATCH (v1:Person)-[:KNOWS]->(v2:Person) RETURN v1.name, v2.name")
    .assert_success()
    .assert_result_count(1); // fails: 0 rows
```

## Related

- `dml_edge_persistence.md` — prerequisite (edges not retrievable)
- `docs/issue/logic_defects.md` — VertexTable get_external_id missing timestamp check (may be related)
- `docs/issue/code_analysis_go_traversal_variable_binding.md` — GO variable binding analysis
