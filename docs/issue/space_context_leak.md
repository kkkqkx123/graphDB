# Space Context Not Persisting After `USE <space>` via `Session::execute()`

> Analysis date: 2026-06-10
> Scope: `crates/graphdb-api/src/api/embedded/session.rs`, `crates/graphdb-api/src/api/core/query_api.rs`
> Discovered by: E2E test refactoring — 45/58 tests fail with `"No graph space selected, please execute USE <space> first"`

## Overview

E2E tests that issue `USE <space>` via `Session::execute()` fail because the space context is not persisted for subsequent queries. All 45 failing tests follow the same pattern: `CREATE SPACE` succeeds, `USE` succeeds, but the next statement fails with `"No graph space selected"`.

## Root Cause

`crates/graphdb-api/src/api/embedded/session.rs:168-188` — `Session::execute()` never captures the space context returned by `USE <space>`.

```rust
pub fn execute(&self, query: &str) -> CoreResult<QueryResult> {
    self.statistics.reset_last();
    let ctx = QueryRequest {
        space_id: self.space_id,          // None if use_space() was never called
        space_name: self.space_name.clone(),
        auto_commit: self.auto_commit,
        transaction_id: None,
        parameters: None,
    };
    let mut query_api = self.db.query_api.write();
    let result = query_api.execute(query, ctx)?;
    // ← NOTE: result may be a SpaceSwitched success, but neither
    //   self.space_id nor self.space_name is ever updated here.
    self.statistics.record_changes(result.metadata.rows_returned);
    Ok(QueryResult::from_core(result))
}
```

The `Session` reads `space_id`/`space_name` from its own fields, but after `QueryApi::execute()` returns the `SpaceSwitched` result, the session never extracts the new space info and stores it back.

## Detailed Flow

### How `USE <space>` reaches the executor

```
Session::execute("USE my_space")
  └─ QueryRequest { space_id: None, space_name: None, ... }
      └─ QueryApi::execute(query, ctx)                       [query_api.rs:136]
          └─ QueryPipelineManager::execute_query_with_request(query, rctx, space_info)  [pipeline_manager.rs:287]
              └─ Parser → Validator → Planner → Executor
                  └─ SwitchSpaceExecutor::execute()           [switch_space.rs:37]
                      └─ Returns ExecutionResult::SpaceSwitched(summary)
          └─ convert_to_query_result(SpaceSwitched(summary))  [query_api.rs:230]
              └─ Returns DataSet { columns: ["space_name", "space_id", "vid_type"], rows: [...] }
              └─ ← NOTE: Type information is lost — SpaceSwitched → DataSet
  ← Session receives QueryResult (opaque), space_id/space_name remain unchanged
```

### Where it works (server path)

`crates/graphdb-api/src/api/server/graph_service.rs:259-298` — `GraphService::execute()` works correctly:

```rust
pub async fn execute(&self, session_id: u64, stmt: &str) -> Result<QueryResult, ServerError> {
    let mut session = self.get_session(session_id).await?;
    let space_id = session.space();                                  // reads from session
    // ... executes query via QueryApi ...

    // After execution, detects USE and updates session
    if stmt.trim().to_uppercase().starts_with("USE ") {              // ← string matching
        if let Ok(ref exec_result) = result {
            if let Some(space_summary) = Self::extract_space_summary_from_result(exec_result) {
                session.set_space(space_summary);                    // ← persists space on session
            }
        }
    }
    // ...
}
```

This is a fragile workaround:
- Relies on `starts_with("USE ")` — not robust against all valid syntax (e.g., `USE` in lowercase before trim, trailing comments)
- Reverse-engineers `SpaceSummary` from `DataSet` columns — tightly coupled to the internal column names `space_name`, `space_id`, `vid_type`

## Design Flaws

### 1. `QueryApi::execute()` destroys type information (query_api.rs:230-254)

`ExecutionResult::SpaceSwitched(SpaceSummary)` is converted to a plain `DataSet`, making it impossible for the caller to distinguish a space-switch result from a regular query result. The semantic information about what happened is lost.

### 2. `Session` is unaware of `USE` results (session.rs:168-188)

Unlike `GraphService`, the embedded `Session::execute()` does not check or extract space context from the result. It treats all successful results uniformly.

### 3. Inconsistency between embedded and server API

| Path | `USE` detection | Space persistence | Status |
|------|----------------|-------------------|--------|
| `Session::execute()` | ❌ | `use_space()` method only | Broken |
| `GraphService::execute()` | `starts_with("USE ")` | On `ClientSession::SpaceContext` | Works (fragile) |

### 4. `Session::use_space()` updates space but `Session::execute("USE x")` doesn't (session.rs:130 vs 168)

```rust
pub fn use_space(&mut self, space_name: &str) -> CoreResult<()> {
    let space_id = self.db.schema_api.use_space(space_name)?;    // ← directly reads from storage
    self.space_id = Some(space_id);
    self.space_name = Some(space_name.to_string());
    Ok(())
}
```

This is a direct storage call that bypasses the query pipeline entirely. It works, but it means the embedded API has two inconsistent paths for space switching: one with full pipeline semantics (via `execute`) and one without (via `use_space`).

## Location Summary

| File | Lines | Role |
|------|-------|------|
| `crates/graphdb-api/src/api/embedded/session.rs` | 51-52 | `space_id`/`space_name` fields (never updated by `execute()`) |
| `crates/graphdb-api/src/api/embedded/session.rs` | 130-135 | `Session::use_space()` — works but bypasses query pipeline |
| `crates/graphdb-api/src/api/embedded/session.rs` | 168-188 | `Session::execute()` — does not capture space from `USE` result |
| `crates/graphdb-api/src/api/core/query_api.rs` | 136-161 | `QueryApi::execute()` — converts `SpaceSwitched` to `DataSet` |
| `crates/graphdb-api/src/api/core/query_api.rs` | 230-254 | `convert_to_query_result()` — loses `SpaceSwitched` variant |
| `crates/graphdb-api/src/api/server/graph_service.rs` | 289-298 | Fragile string-match fix for server path |
| `crates/graphdb-query/src/query/executor/admin/space/switch_space.rs` | 37-73 | Produces `SpaceSwitched` correctly |
| `crates/graphdb-query/src/query/executor/base/execution_result.rs` | 21 | `SpaceSwitched(SpaceSummary)` variant definition |

## Recommended Fixes

### Fix A (Targeted — Session level)

In `Session::execute()`, after calling `QueryApi::execute()`, inspect the result for space switch data and update `self.space_id`/`self.space_name`.

**Approach**: Check if the returned `DataSet` has `space_name`/`space_id` columns (like `GraphService::extract_space_summary_from_result()` does). This keeps the fix local to the embedded path.

**Trade-off**: Duplicates the fragile logic from `GraphService`, relies on column naming convention.

### Fix B (Clean — API level)

Make `QueryApi::execute()` return `ExecutionResult`-aware results so the caller can distinguish `SpaceSwitched`.

**Approach**: Add a variant or additional metadata to `QueryResult` that signals a space switch. `Session::execute()` and `GraphService::execute()` both use this metadata to update their session state.

**Trade-off**: Requires changing the `QueryResult` type and all callers, but eliminates the fragile string-matching approach.

### Fix C (Full — Unify session state management)

Introduce `SessionContext` as a shared state object that both `Session` (embedded) and `QueryApi` (core) reference. The `USE` executor writes the new space context directly into the shared `SessionContext`, making it available for the next query without extraction logic.

**Trade-off**: Larger refactoring, requires changing the query pipeline's context model to accept writable session state.

## Impact on E2E Tests

The E2E tests cannot use `session.execute("USE <space>")` until this is fixed. The workaround is to use `session.use_space("<space>")` instead, which bypasses the query pipeline entirely. This means the `USE` statement path is never tested via the embedded API.

Once Fix A, B, or C is applied, the following test pattern will work correctly:

```rust
let db = GraphDatabase::open(path)?;
let mut session = db.session()?;
session.execute("CREATE SPACE s (vid_type=STRING)")?;
session.execute("USE s")?;
session.execute("CREATE TAG person(name STRING)")?;  // should work, currently fails
```
