# E2E Test Failures — Final Status (Resolved)

## Summary

All 6 originally-failing e2e tests now pass. The 68-test suite completes with 0 failures.

## What Was Fixed

### Phase 1: Metadata Provider Chain

**Problem**: `SchemaMetadataProvider` had `index_manager: None`, so fulltext and vector indexes could not be found during query planning.

**Fix**: Created dedicated metadata providers for fulltext and vector indexes, composed via `CompositeMetadataProvider`:

- `crates/graphdb-query/src/query/metadata/fulltext_provider.rs` — new `FulltextIndexMetadataProvider`
- `crates/graphdb-query/src/query/metadata/vector_provider.rs` — new `VectorIndexMetadataProvider`
- `crates/graphdb-api/src/api/core/query_api.rs:91-104` — wired into `CompositeMetadataProvider`

### Phase 2: space_id Propagation

**Problem**: `ExecutionContext::set_space_id()` was never called, so `context.current_space_id()` always returned 0. CREATE INDEX stored with `space_id=0`; SEARCH planned with `space_id=1`.

**Fix**: Added `space_id: u64` to plan nodes, populated from `QueryContext` in planners:

- `crates/graphdb-query/src/query/planning/plan/core/nodes/search/fulltext/management.rs` — `CreateFulltextIndexNode.space_id`
- `crates/graphdb-query/src/query/planning/plan/core/nodes/search/vector/management.rs` — `CreateVectorIndexNode.space_id`
- `crates/graphdb-query/src/query/planning/fulltext_planner.rs` — passes `qctx.space_id()`
- `crates/graphdb-query/src/query/planning/vector_planner.rs` — passes `qctx.space_id()`
- `crates/graphdb-query/src/query/executor/factory/builders/fulltext_search_builder.rs` — uses `node.space_id`
- `crates/graphdb-query/src/query/executor/data_access/vector_index.rs` — uses `self.node.space_id`

### Phase 3: Vector Index Registration in Disabled-Engine Mode

**Problem**: When the vector engine is disabled (e2e tests), `CreateVectorIndexExecutor` skipped `create_index_with_config()` entirely, so `logical_indexes` was empty and `list_indexes()` returned nothing.

**Fix**: Added `register_logical_index()` method and `index_name` field to `IndexMetadata`:

- `crates/vector-client/src/manager/index.rs` — added `index_name: Option<String>` field to `IndexMetadata`
- `crates/graphdb-sync/src/sync/vector_sync.rs` — added `register_logical_index()` method, `IndexMetadataWrapper.index_name`
- `crates/graphdb-query/src/query/executor/data_access/vector_index.rs` — calls `register_logical_index()` in disabled-engine branch
- `crates/graphdb-query/src/query/metadata/vector_provider.rs` — matches by `index_name`

### Phase 4: Vector Search Visitor Methods

**Problem**: `ChildRewriteVisitor` had `visit_vector_manage` but not `visit_vector_search`, `visit_vector_lookup`, or `visit_vector_match`. Optimizer panicked with `unreachable!()` on these node types.

**Fix**: Added three visitor methods:

- `crates/graphdb-query/src/query/optimizer/heuristic/visitor.rs` — added `visit_vector_search`, `visit_vector_lookup`, `visit_vector_match`

### Phase 5: Tokio Runtime in Disabled-Engine Mode

**Problem**: `VectorSearchExecutor::execute_search()` called `tokio::runtime::Handle::current()` which panics without a tokio runtime. E2E tests run synchronously.

**Fix**: Early return for disabled engine:

- `crates/graphdb-query/src/query/executor/data_access/vector_search.rs` — added `is_disabled_engine()` check before `tokio::runtime::Handle::current()`

### Phase 6: Vector Parser WHERE Clause

**Problem**: `parse_search_vector_statement` consumed `WHERE` at line 218, then called `parse_where_clause` which tried to consume `WHERE` again at line 324. The second consumption failed with "Expected keyword 'WHERE', found Identifier('price')".

**Fix**: Removed duplicate `WHERE` consumption:

- `crates/graphdb-query/src/query/parser/parsing/vector_parser.rs` — removed `ctx.consume_keyword("WHERE")?;` from `parse_where_clause()`

### Phase 7: Default Tantivy Tokenizer

**Problem**: Default `TokenizerKind` was `Jieba`, but the `jieba` Cargo feature had a version mismatch compile error. User's `analyzer='standard'` was also being ignored.

**Fix**:

- `crates/graphdb-config/src/config/common/fulltext.rs` — changed `#[default]` from `Jieba` to `Default`

## Test Results

```
test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Related Documents

- `docs/issue/lookup_scan_type_optimization.md` — LOOKUP ScanType optimization plan
