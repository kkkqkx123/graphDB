# Vector Search API Documentation

## Overview
This document describes the vector search functionality and its API design.

## VectorSearchParams

The `VectorSearchParams` struct encapsulates all parameters needed for creating a vector search node. This refactoring was done to address the "too many arguments" issue in the original function.

### Fields
- `index_name`: Name of the index to search
- `space_id`: ID of the space containing the data
- `tag_name`: Name of the tag to search within
- `field_name`: Name of the field containing vector data
- `query`: The vector query expression
- `threshold`: Optional similarity threshold for filtering results
- `filter`: Optional vector filter for payload filtering (e.g., WHERE clause conditions)
- `limit`: Maximum number of results to return
- `offset`: Offset for pagination
- `output_fields`: List of output fields to include in results
- `metadata_version`: Version of metadata for validation (0 if not tracked)

### Usage
```rust
let params = VectorSearchParams::new(
    "my_index".to_string(),
    123,
    "person".to_string(),
    "embedding".to_string(),
    query_expr,
    Some(0.8),
    None,
    10,
    0,
    vec![OutputField {
        name: "name".to_string(),
        alias: None,
    }],
);

let node = VectorSearchNode::with_metadata_version(params);
```

## SearchOptions

The `SearchOptions` struct provides a builder pattern for configuring vector searches with various options.

### Methods
- `new()`: Create basic search options with required parameters
- `with_threshold()`: Add a similarity threshold filter
- `with_filter()`: Add a payload filter

### Usage
```rust
let options = SearchOptions::new(123, "person", "embedding", query_vector, 10)
    .with_threshold(0.8)
    .with_filter(filter);

let results = coordinator.search_with_options(options).await?;
```

## Migration Guide

### Before
```rust
let node = VectorSearchNode::with_metadata_version(
    "my_index".to_string(),
    123,
    "person".to_string(),
    "embedding".to_string(),
    query_expr,
    Some(0.8),
    None,
    10,
    0,
    output_fields,
    1,
);
```

### After
```rust
let params = VectorSearchParams::new(
    "my_index".to_string(),
    123,
    "person".to_string(),
    "embedding".to_string(),
    query_expr,
    Some(0.8),
    None,
    10,
    0,
    output_fields,
);
let node = VectorSearchNode::with_metadata_version(params);
```

The old `new()` method is preserved for backward compatibility and now uses the new parameter structure internally.