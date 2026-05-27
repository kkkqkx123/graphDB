# DML Edge Persistence Failure Analysis

> Date: 2026-05-27
> Scope: INSERT EDGE, DELETE EDGE, UPDATE EDGE execution

## Problem

`INSERT EDGE` reports success but inserted edges are not retrievable. All subsequent operations
that depend on edge data (MATCH traversal, GO traversal, FETCH EDGE, DELETE EDGE, UPDATE EDGE)
return 0 results or fail.

## Evidence

### INSERT EDGE succeeds but edge not found

```
// Setup
INSERT VERTEX Person(name) VALUES 1:('Alice'), 2:('Bob')
INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')

// Read back — fails
FETCH PROP ON EDGE 1->2                → empty result
GO 1 STEPS FROM 1 OVER KNOWS           → 0 rows  
MATCH (v1)-[:KNOWS]->(v2) RETURN v1    → 0 rows
```

### Error messages

- "Expected edge 1 -> 2 with type KNOWS to exist" — from test assertion that calls storage directly
- "Failed to create edge type: [not_found] Source tag  not found" — when edge type has no valid source tag reference
- "Source tag  not found" — empty tag name in error

## Root Cause Analysis

The error `Source tag  not found` (with empty tag name) points to the edge type registration path:

### Hypothesis 1: Edge type not linked to source/destination vertex tags

When `INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')` executes:

1. Parser → `InsertEdgeStmt { edge_type: "KNOWS", src: 1, dst: 2, props: ["since"], values: ["2020-01-01"] }`
2. Validator checks edge type "KNOWS" exists in schema
3. Executor writes edge data to edge property table
4. Executor links edge to source vertex (1) and dest vertex (2)

If step 4 is skipped or the link uses wrong internal IDs, the edge becomes orphaned —
written to storage but not reachable from its endpoint vertices.

### Hypothesis 2: Internal vertex ID resolution fails

Edge insertion and traversal use `get_internal_id(external_id)` to resolve vertex IDs.
If this resolution returns wrong IDs or fails silently, the edge is stored under a
vertex that doesn't correspond to the external ID.

### Key Investigation Points

| File | Role |
|------|------|
| `src/query/executor/dml/insert_edge_executor.rs` | Edge insert execution |
| `src/storage/edge/edge_table.rs` | Edge property storage |
| `src/storage/vertex/vertex_table.rs` | Vertex ID resolution |
| `src/storage/edge/adjacency_list.rs` | Edge-to-vertex linking |
| `src/query/validator/statements/insert_edges_validator.rs` | Edge insert validation |

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
    .query("FETCH PROP ON EDGE 1->2")
    .assert_success()
    .assert_result_count(1); // fails: 0 rows
```

## Related Issues

- DDL `test_edge_default_value_execution` — same cause (edge not found after INSERT)
- DQL all edge traversal tests — same cause
- UPDATE EDGE tests — same cause
- DELETE EDGE tests — same cause
- Batch edge operations — same cause
