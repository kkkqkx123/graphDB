# E2E Test Failures Analysis (2026-06-13)

## Summary

Two e2e tests remain failing after initial analysis. The edge count test was successfully fixed.

## Test Status

| Test | Status | Expected | Actual |
|------|--------|----------|--------|
| `test_social_network_edge_counts` | PASS | 30 | 30 |
| `test_social_network_vertex_counts` | PASS | 20 | 20 |
| `test_social_network_filter` | PASS | varies | correct |
| `test_social_network_go_traversal` | FAIL | non-empty | empty |
| `test_social_network_lookup_index` | FAIL | 1 | 0 |

## 1. Fixed: Edge Count Test

**Problem**: `MATCH ()-[f:friend]->() RETURN count(f)` returned 2 instead of 30.

**Root Cause**: `ExpandAllExecutor` was incorrectly filtering edges based on source vertices (`src_vids`) even when the pattern had no explicit source node.

**Fix Applied** (`crates/graphdb-query/src/query/executor/graph_operations/graph_traversal/expand_all.rs`):
- Added direct edge scanning optimization: when specific edge types are specified AND there are no input vertices from a previous step (i.e., this is a standalone edge pattern), use `storage.scan_edges_by_type()` directly.
- Guard: `has_specific_edge_types && !is_from_go_clause`

## 2. Failing: GO Traversal

**Test**: `GO 1 STEP FROM 'p1' OVER friend REVERSELY YIELD friend.name`

**Expected**: Based on `social_network_data.gql`:
- `p4 -> p1` (friend edge)
- `p2 -> p1` (friend edge)
- `p6 -> p1` (friend edge)
So p1 should have at least 3 reverse friends.

**Actual**: Empty result set.

**Data File Evidence** (`tests/e2e/data/social_network_data.gql`):
```gql
INSERT EDGE friend(degree, since, trust_level) VALUES "p4" -> "p1" @0: (0.95, "2019-06-26", 3)
INSERT EDGE friend(degree, since, trust_level) VALUES "p2" -> "p1" @0: (0.68, "2018-09-04", 5)
INSERT EDGE friend(degree, since, trust_level) VALUES "p6" -> "p1" @0: (0.93, "2018-06-19", 3)
```

## 3. Failing: LOOKUP Index Query

**Test**: `LOOKUP ON person WHERE person.name == 'Alice' YIELD person.name`

**Expected**: 1 result (p1 is Alice)

**Actual**: Empty result set.

**Data File Evidence**:
```gql
INSERT VERTEX person(name, age, ...) VALUES "p1": ("Alice", 34, ...)
CREATE TAG INDEX IF NOT EXISTS idx_person_name ON person(name)
```

## Analysis

### Storage Layer

The storage layer fix in `reader.rs` addresses potential issues with string ID handling:
- Added `vid_to_string()` helper that uses `as_str()` instead of `Display` (which adds quotes)
- Fixed `get_node_edges()` to use this helper in all three direction branches

However, the LOOKUP and GO issues likely originate from the query execution layer, not the storage layer.

### Query Execution Layer

Possible root causes:

1. **GO Query Execution**:
   - How does the query parser handle `FROM 'p1'`?
   - Is `p1` correctly resolved to a VertexId?
   - Does the ExpandAllExecutor receive the correct source vertices?

2. **LOOKUP Query Execution**:
   - Is the index being created properly?
   - Does the LOOKUP executor correctly query the index?
   - Is the WHERE condition being evaluated correctly?

### Investigation Required

1. Trace the execution of `GO 1 STEP FROM 'p1' OVER friend REVERSELY`:
   - Parser output
   - Query plan
   - Source vertex resolution
   - Edge expansion

2. Trace the execution of `LOOKUP ON person WHERE person.name == 'Alice'`:
   - Parser output
   - Index lookup implementation
   - Result filtering

## Files Modified During Investigation

### Completed Fixes

- `crates/graphdb-query/src/query/executor/graph_operations/graph_traversal/expand_all.rs` - Direct edge scan optimization
- `crates/graphdb-storage/src/storage/engine/graph_storage/reader.rs` - vid_to_string helper for correct string ID handling

### Files Needing Further Investigation

- `crates/graphdb-query/src/query/` - GO and LOOKUP query execution
- `crates/graphdb-storage/src/storage/engine/graph_storage/` - Index implementation

## Next Steps

1. Add debug output to understand GO query execution flow
2. Verify vertex `p1` exists and can be found by the system
3. Check index creation for `idx_person_name`
4. Trace LOOKUP query execution path

## Commands to Run Tests

```bash
# Run GO traversal test
cargo test --test integration_e2e -- test_social_network_go_traversal --nocapture

# Run LOOKUP test
cargo test --test integration_e2e -- test_social_network_lookup_index --nocapture

# Run edge count test (should pass now)
cargo test --test integration_e2e -- test_social_network_edge_counts --nocapture
```