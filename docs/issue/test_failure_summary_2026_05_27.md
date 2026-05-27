# Integration Test Failure Summary

> Date: 2026-05-27
> Scope: `tests/integration_ddl.rs`, `tests/integration_dml.rs`, `tests/integration_dql.rs`
> Build: `cargo test --test integration_ddl --test integration_dml --test integration_dql`

## Totals

| Suite | Total | Passed | Failed | Pass Rate |
|-------|-------|--------|--------|-----------|
| DDL | 98 | 93 | 5 | 94.9% |
| DML | 85 | 40 | 45 | 47.1% |
| DQL | 127 | 101 | 26 | 79.5% |
| **Total** | **310** | **234** | **76** | **75.5%** |

---

## 1. DDL Constraints — 5 failures

### Source: `tests/ddl/constraints.rs`

All 5 failures are in DEFAULT/NOT NULL constraint execution tests. Parser-level tests pass.

| Test | Symptom | Root Cause |
|------|---------|------------|
| `test_default_value_execution_insert` | INSERT without specifying a column: default value (18) not applied, property is `None` | DML executor or storage layer doesn't resolve column defaults during INSERT |
| `test_default_value_string_execution` | Parse error at line 1, col 22: `Expected identifier, found RParen` | Parser rejects `DEFAULT ""` syntax (empty string) |
| `test_default_with_not_null_constraint` | Property is `None` after INSERT, expected default value `"unknown"` | Same as no.1 — default not applied |
| `test_edge_default_value_execution` | Edge 1→2 with type KNOWS doesn't exist after INSERT EDGE | Edge insert with default constraint doesn't persist |
| `test_not_null_constraint_reject_null` | INSERT with NULL on NOT NULL column succeeds instead of error | Validator or executor doesn't enforce NOT NULL |

**Impact**: Schema-level constraints (DEFAULT, NOT NULL) defined in `CREATE TAG/EDGE` are parsed and stored in metadata but not enforced during DML execution.

---

## 2. DML Edge Operations — 35 failures

### Core symptom: Edge insertion does not persist

Inserted edges are not retrievable. `INSERT EDGE` reports success but subsequent reads find no edges.

### Test Files Affected

| File | Failures | Pattern |
|------|----------|---------|
| `tests/dml/insert_edge.rs` | 8 | All edge insert execution tests fail |
| `tests/dml/delete.rs` | 15 | DELETE EDGE, DELETE VERTEX with edge, MATCH/PIPE DELETE |
| `tests/dml/update.rs` | 4 | UPDATE EDGE fails, UPDATE VERTEX with SET condition fails |
| `tests/dml/upsert.rs` | 10 | UPSERT VERTEX/EDGE parse error (ON DUPLICATE keyword rejected) |
| `tests/dml/batch_operations.rs` | 5 | Batch edge operations, social network data flow |

### Specific Sub-Issues

#### 2a. INSERT EDGE does not persist (8 tests)
- INSERT EDGE returns success but `vertex_has_edge()` returns false
- INSERT EDGE with IF NOT EXISTS: parser rejects `IF` keyword
- INSERT EDGE with rank: `Rank must be an integer constant or variable`
- INSERT EDGE nonexistent edge type/vertices: succeeds instead of error

#### 2b. DELETE operations fail (15 tests)
- DELETE EDGE/VERTEX with pipe syntax: panics in `heuristic/visitor.rs:185` — `visit_default should not be called` (unreachable code reached)
- MATCH DELETE with pattern: same `visit_default` panic
- DELETE with WHERE clause: same panic
- DELETE EDGE parse: parser wants identifier but finds integer literal

#### 2c. UPDATE fails for edges (4 tests)
- UPDATE EDGE syntax: parser expects `OF` keyword after `UPDATE`, finds integer
- UPDATE VERTEX with condition: YIELD returns wrong column (`v` instead of id), `UndefinedVariable: v` in ORDER BY

#### 2d. UPSERT syntax not supported (10 tests)
- UPSERT VERTEX/EDGE: parser rejects `ON` keyword after `INSERT` — `Unexpected token in expression: On`
- MERGE VERTEX (without ON DUPLICATE) works correctly

#### 2e. Batch operations fail (5 tests)
- All batch operations involving edges fail
- Vertex batch operations pass

**Impact**: Edge-oriented DML is effectively unusable. The edge storage layer writes data but reads fail to find it, suggesting a metadata/index inconsistency (e.g., edge type not linked to source/destination vertex after insertion).

---

## 3. DQL Execution — 26 failures

### 3a. FIND PATH returns 0 results (6 failures)
- `FIND SHORTEST PATH FROM 1 TO 2 OVER KNOWS` returns 0 rows even when edge 1→2 exists
- `FIND ALL PATH FROM 1 TO 4 OVER KNOWS` returns 0 rows in diamond topology
- Parser tests all pass; execution always returns empty

### 3b. MATCH with edge traversal returns 0 results (7 failures)
- `MATCH (v1)-[:KNOWS]->(v2) RETURN v1.name, v2.name` returns 0 rows
- Single-tag MATCH (`MATCH (p:Person)`) works correctly
- Multi-hop, multi-edge-type, self-loop, complex social network patterns all fail
- ORDER BY in MATCH: `UndefinedVariable: v` — variable name collision

### 3c. GO traversal returns 0 results (5 failures)
- `GO 1 STEPS FROM 1 OVER KNOWS` returns 0 rows
- Same issue as MATCH with edge — no traversal results

### 3d. Aggregation results mismatch (3 failures)
- `SUM(price)` returns `String("30.0")` instead of `Double(30.0)` or `BigInt`
- `MIN(age)` and `MAX(age)` return wrong types (String instead of Int)
- `SUM(age)` with `GROUP BY name` returns String instead of Int

### 3e. FETCH EDGE (3 failures)
- `FETCH PROP ON EDGE 1->2` returns no data

### 3f. Optimizer join tests (2 failures)
- `test_join_001_join_algorithm_selection`: `Failed to create edge type: Source tag not found`
- `test_optimizer_complex_join`: same error

**Impact**: No graph traversal query works at execution level. Only single-table lookup operations (LOOKUP, simple MATCH) produce results. Path, edge-traversal, and aggregation queries are all broken.

---

## 4. Root Cause Patterns

### Pattern A: Edge Storage Write-Read Gap
The most consistent failure across DDL (constraint edge default), DML (insert edge), and DQL (edge traversal) is that **edges are not retrievable after insertion**. The write path succeeds (no error reported) but the read path finds nothing. This suggests:
- Edge data is written to the edge table
- But the edge index (linking edges to source/destination vertices) is not updated
- Or the edge type metadata is inconsistent

### Pattern B: Optimizer Visitor Unreachable Code
Multiple DELETE operations with pipe/MATCH syntax hit `visit_default should not be called` in `optimizer/heuristic/visitor.rs:185`. The `visit_default` method is a catch-all that should never be reached — the optimizer's plan visitor doesn't handle certain plan node types that DELETE can produce.

### Pattern C: Parser Syntax Incompatibility
Three syntax-related failures:
- UPSERT `ON DUPLICATE` keyword rejected (parser doesn't support the UPSERT syntax variant)
- DELETE EDGE `DELETE EDGE 1->2` rejects integer (expects identifier first? expects `DELETE EDGE <edge_type> <src> -> <dst>`)
- UPDATE EDGE expects `OF` keyword (syntax is `UPDATE EDGE ON <src> -> <dst>` but parser expects `UPDATE <edge_type> OF ...`)

### Pattern D: Aggregation Type Coercion
SUM/MIN/MAX return values as `Value::String` instead of numeric types. The aggregation functions convert results to string during computation or serialization.

---

## 5. Recommended Fix Order

| Priority | Issue | Effort | Impact |
|----------|-------|--------|--------|
| P0 | Edge storage write-read inconsistency | Medium | Unblocks 35+ DML tests + all edge traversal |
| P1 | Optimizer visitor `visit_default` panic | Small | Prevents crashes on MATCH/PIPE DELETE |
| P1 | FIND PATH executor returns 0 results | Medium | Path queries completely broken |
| P2 | UPSERT parser `ON` keyword | Small | Syntax gap |
| P2 | DELETE EDGE parser integer | Small | Syntax gap |
| P2 | UPDATE EDGE parser `OF` keyword | Small | Syntax gap |
| P3 | DEFAULT constraint enforcement | Medium | Schema-level feature |
| P3 | NOT NULL constraint enforcement | Small | Schema-level feature |
| P3 | Aggregation type coercion | Small | Wrong result types |
