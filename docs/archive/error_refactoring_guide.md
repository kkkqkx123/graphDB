# Error Handling System Refactoring Guide

## Overview

This document describes the error handling system refactoring for GraphDB. The main goal is to reduce the size of error types by using boxed errors.

## Completed Work

### Phase 1: DBError Refactoring (Done)

`DBError` has been refactored from enum to struct with boxed source error:

```rust
pub struct DBError {
    kind: ErrorKind,
    message: String,
    source: Option<BoxedError>,
    class: ErrorClass,
}
```

**Size**: ~24 bytes (down from ~160 bytes)

### Phase 2: QueryError Refactoring (Done)

`QueryError` has been refactored from enum to struct with boxed source error:

```rust
pub struct QueryError {
    kind: QueryErrorKind,
    message: String,
    source: Option<BoxedError>,
}
```

**Size**: ~24 bytes (down from ~160 bytes)

## Migration Patterns

### Pattern 1: Simple error creation

**Old:**

```rust
QueryError::ExecutionError(msg)
QueryError::InvalidQuery(msg)
QueryError::PlanningError(msg)
```

**New:**

```rust
QueryError::execution(msg)
QueryError::invalid_query(msg)
QueryError::planning(msg)
```

### Pattern 2: Error with source

**Old:**

```rust
QueryError::StorageError(storage_error)
QueryError::ExpressionError(expression_error)
```

**New:**

```rust
QueryError::from(storage_error)  // or just .into()
QueryError::from(expression_error)
```

### Pattern 3: Accessing error details

**Old:**

```rust
match err {
    QueryError::ParseError(e) => ...,
    QueryError::ExecutionError(msg) => ...,
    _ => ...
}
```

**New:**

```rust
match err.kind() {
    QueryErrorKind::Parse => {
        if let Some(ref source) = err.source() {
            if let Some(pe) = source.downcast_ref::<StructuredParseError>() {
                // use pe
            }
        }
    }
    QueryErrorKind::Execution => ...,
    _ => ...
}
```

## Remaining Clippy Warnings (39 total)

After the refactoring, the following warnings remain:

### 1. `too_many_arguments` (30 warnings)

Functions with more than 7 arguments. These require parameter grouping into structs.

**Affected files:**

- `src/transaction/insert_transaction.rs`
- `src/transaction/undo_log.rs`
- `src/transaction/update_transaction.rs`
- `src/query/executor/data_access/fulltext_search.rs`
- `src/query/executor/graph_operations/materialize.rs`

**Recommended fix:**

```rust
// Before
fn add_edge(&mut self, src_label: LabelId, src_vid: VertexId,
            dst_label: LabelId, dst_vid: VertexId, edge_type: LabelId,
            props: HashMap<String, Value>, ts: Timestamp) -> Result<EdgeId>;

// After
struct EdgeParams {
    src_label: LabelId,
    src_vid: VertexId,
    dst_label: LabelId,
    dst_vid: VertexId,
    edge_type: LabelId,
    props: HashMap<String, Value>,
}
fn add_edge(&mut self, params: EdgeParams, ts: Timestamp) -> Result<EdgeId>;
```

### 2. `type_complexity` (5 warnings)

Complex type definitions that should use type aliases.

**Affected files:**

- `src/transaction/update_transaction.rs:222`

**Recommended fix:**

```rust
// Before
) -> UpdateTransactionResult<Vec<(LabelId, LabelId, LabelId, Vec<RelatedEdgeInfo>)>>;

// After
type EdgeUpdateResult = Vec<(LabelId, LabelId, LabelId, Vec<RelatedEdgeInfo>)>;
) -> UpdateTransactionResult<EdgeUpdateResult>;
```

### 3. `large_enum_variant` (3 warnings)

Enums with significantly different variant sizes.

**Affected files:**

- `src/query/executor/base/manage_executor_enums.rs`

**Recommended fix:**

```rust
// Before
pub enum FulltextManageExecutor {
    Create(CreateFulltextIndexExecutor<S>, ...),
    Drop(DropFulltextIndexExecutor<S>, ...),
}

// After
pub enum FulltextManageExecutor {
    Create(Box<CreateFulltextIndexExecutor<S>>, ...),
    Drop(Box<DropFulltextIndexExecutor<S>>, ...),
}
```

### 4. `module_inception` (1 warning)

Module has the same name as its containing module.

**Affected files:**

- `src/storage/page/mod.rs`

**Recommended fix:**
Rename the inner module or restructure the module hierarchy.

## Benefits Achieved

1. **Smaller error size**: ~24 bytes vs ~160 bytes
2. **No more `result_large_err` warnings** (175 warnings eliminated)
3. **Error classification**: Built-in retryable/user-error detection via `ErrorClass`
4. **Better error chains**: Full source tracking
5. **Consistent API**: All errors use the same pattern

## Summary

| Metric                        | Before | After |
| ----------------------------- | ------ | ----- |
| Total warnings                | 229    | 39    |
| `result_large_err` warnings   | 175    | 0     |
| `too_many_arguments` warnings | 30     | 30    |
| `type_complexity` warnings    | 5      | 5     |
| `large_enum_variant` warnings | 3      | 3     |
| `module_inception` warnings   | 1      | 1     |

The remaining 39 warnings require larger architectural changes and are deferred for future iterations.
