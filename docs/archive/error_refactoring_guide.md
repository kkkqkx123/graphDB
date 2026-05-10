# Error Handling System Refactoring Guide

## Overview

This document describes the error handling system refactoring for GraphDB. The main goal is to reduce the size of `DBError` from ~160 bytes to ~24 bytes by using boxed errors.

## Key Changes

### 1. DBError is now a struct, not an enum

**Old design:**

```rust
pub enum DBError {
    Storage(#[from] StorageError),
    Query(#[from] QueryError),
    // ... 22 variants
}
```

**New design:**

```rust
pub struct DBError {
    kind: ErrorKind,
    message: String,
    source: Option<BoxedError>,
    class: ErrorClass,
}
```

### 2. Migration Patterns

#### Pattern 1: Simple error creation

**Old:**

```rust
DBError::Query(QueryError::ExecutionError(msg))
DBError::Validation(msg)
DBError::Internal(msg)
DBError::Io(msg)
DBError::Transaction(msg)
DBError::Search(msg)
```

**New:**

```rust
DBError::query(msg)
DBError::validation(msg)
DBError::internal(msg)
DBError::io(msg)
DBError::transaction(msg)
DBError::search(msg)
```

#### Pattern 2: Error with sub-error type

**Old:**

```rust
DBError::Storage(storage_error)
DBError::Query(query_error)
DBError::Session(session_error)
```

**New:**

```rust
DBError::from(storage_error)  // or just .into()
DBError::from(query_error)
DBError::from(session_error)
```

#### Pattern 3: Pattern matching on DBError

**Old:**

```rust
match err {
    DBError::Query(qe) => ...,
    DBError::Storage(se) => ...,
    _ => ...
}
```

**New:**

```rust
match err.kind() {
    ErrorKind::Query => {
        if let Some(ref source) = err.source() {
            if let Some(qe) = source.downcast_ref::<QueryError>() {
                // use qe
            }
        }
    }
    ErrorKind::Storage => ...,
    _ => ...
}
```

### 3. New Features

#### Error Classification

```rust
if err.is_retryable() {
    // retry logic
} else if err.is_user_error() {
    // return to client
} else {
    // system error, log and handle
}
```

#### Error Kind

```rust
match err.kind() {
    ErrorKind::Query => ...,
    ErrorKind::Storage => ...,
    ErrorKind::Transaction => ...,
    // ...
}
```

## Files to Update

### Test Files

- [x] tests/common/query_helpers.rs
- [x] tests/common/mod.rs
- [x] tests/common/transaction_helpers.rs

### Core Files

- [x] src/core/error/mod.rs
- [x] src/core/error/query.rs
- [x] src/api/core/error.rs
- [x] src/api/mod.rs

### Query Executor Files (need update)

- src/query/executor/relational_algebra/join/cross_join.rs
- src/query/executor/relational_algebra/join/left_join.rs
- src/query/executor/relational_algebra/join/full_outer_join.rs
- src/query/executor/result_processing/transformations/unwind.rs
- src/query/executor/result_processing/transformations/pattern_apply.rs
- src/query/executor/result_processing/transformations/helpers.rs
- src/query/executor/result_processing/transformations/append_vertices.rs
- src/query/executor/result_processing/topn.rs
- src/query/executor/result_processing/sample.rs
- src/query/executor/result_processing/limit.rs
- src/query/executor/result_processing/dedup.rs
- src/query/executor/relational_algebra/selection/filter.rs
- src/query/executor/relational_algebra/projection.rs
- src/query/executor/relational_algebra/aggregation.rs
- src/query/executor/data_modification/update.rs
- src/query/executor/data_modification/delete.rs
- src/query/executor/data_access/vector_search.rs
- src/query/executor/data_access/match_fulltext.rs
- src/query/executor/data_access/fulltext_search.rs
- src/query/executor/control_flow/loops.rs
- src/query/executor/relational_algebra/set_operations/base.rs
- src/query/executor/base/result_processor.rs
- src/query/executor/factory/executor_factory.rs
- src/query/executor/result_processing/sort.rs
- src/query/executor/data_access/vertex.rs
- src/query/executor/result_processing/transformations/rollup_apply.rs
- src/query/executor/result_processing/transformations/assign.rs
- src/query/executor/result_processing/agg_function_manager.rs
- src/query/executor/graph_operations/materialize.rs
- src/query/executor/data_modification/remove.rs
- src/query/executor/admin/index/fulltext_index/describe_fulltext_index.rs
- src/query/executor/relational_algebra/join/hash_table.rs
- src/query/executor/graph_operations/graph_traversal/algorithms/multi_shortest_path.rs
- src/query/executor/graph_operations/graph_traversal/algorithms/subgraph_executor.rs
- src/query/executor/graph_operations/graph_traversal/all_paths.rs
- src/query/executor/graph_operations/graph_traversal/traversal_utils.rs
- src/query/executor/data_access/vector_index.rs
- src/query/executor/relational_algebra/set_operations/union_all.rs
- src/query/executor/relational_algebra/set_operations/union.rs
- src/query/executor/relational_algebra/set_operations/minus.rs
- src/query/executor/relational_algebra/set_operations/intersect.rs
- src/query/executor/utils/recursion_detector.rs
- src/query/executor/admin/user/revoke_role.rs
- src/query/executor/admin/user/drop_user.rs
- src/query/executor/admin/user/grant_role.rs
- src/query/executor/admin/user/change_password.rs
- src/query/executor/admin/user/create_user.rs
- src/query/executor/admin/user/alter_user.rs
- src/query/executor/admin/space/alter_space.rs
- src/query/executor/admin/index/fulltext_index/drop_fulltext_index.rs
- src/query/executor/admin/index/fulltext_index/alter_fulltext_index.rs
- src/query/executor/admin/index/fulltext_index/create_fulltext_index.rs

## Automated Migration Script

Use the following sed-style replacements:

```bash
# Simple replacements
DBError::Query(QueryError::ExecutionError(format!(...))) -> DBError::query(format!(...))
DBError::Validation(format!(...)) -> DBError::validation(format!(...))
DBError::Internal(format!(...)) -> DBError::internal(format!(...))
DBError::Io(format!(...)) -> DBError::io(format!(...))
DBError::Transaction(format!(...)) -> DBError::transaction(format!(...))
DBError::Search(format!(...)) -> DBError::search(format!(...))

# With sub-errors
DBError::Storage(e) -> DBError::from(e)
DBError::Query(e) -> DBError::from(e) or DBError::query(e.to_string())
DBError::Session(e) -> DBError::from(e)
```

## Benefits

1. **Smaller error size**: ~24 bytes vs ~160 bytes
2. **No more `result_large_err` warnings**
3. **Error classification**: Built-in retryable/user-error detection
4. **Better error chains**: Full source tracking
5. **Consistent API**: All errors use the same pattern
