# Migration Guide: bincode to OxiCode

## Overview

This document describes the migration from `bincode` to `oxicode` for binary serialization in the GraphDB project.

**Date**: 2026-04-20
**Library Version**: OxiCode 0.2.1

## Why Migrate?

OxiCode is the spiritual successor to bincode, offering:
- 100% binary compatibility with bincode format
- Modern Rust practices and patterns
- No `unwrap()` policy with comprehensive error handling
- SIMD-optimized encoding/decoding
- Built-in compression and checksum support
- Schema versioning and evolution support

## API Mapping

### Import Changes

| bincode | oxicoide |
|---------|----------|
| `use bincode::{encode_to_vec, decode_from_slice};` | `use oxicoide::{encode_to_vec, decode_from_slice};` |
| `use bincode::{Decode, Encode};` | `use oxicoide::{Decode, Encode};` |
| `use bincode::config::standard;` | Not needed (OxiCode uses standard config by default) |

### Function Changes

| bincode | oxicoide |
|---------|----------|
| `encode_to_vec(value, standard())` | `encode_to_vec(&value)` |
| `decode_from_slice(bytes, standard())` | `decode_from_slice(bytes)` |
| Returns `(value, bytes_consumed)` | Returns `Result<value, Error>` |

### Key Differences

1. **No Config Required**: OxiCode uses standard configuration by default. No need to pass `standard()` config.
2. **Error Handling**: OxiCode returns `Result` types instead of tuples. Decoding returns `Result<T, Error>` instead of `(T, usize)`.
3. **Serde Integration**: OxiCode requires explicit `features = ["serde"]` in Cargo.toml.

## Affected Files

### Main Project (src/)

| File | Changes |
|------|---------|
| `src/transaction/context.rs` | Import and function changes |
| `src/storage/operations/redb/reader.rs` | Import and function changes |
| `src/storage/operations/redb/writer.rs` | Import and function changes |
| `src/storage/operations/rollback.rs` | Import and function changes |
| `src/storage/metadata/redb_index_metadata_manager.rs` | Import and function changes |
| `src/storage/metadata/redb_schema_manager.rs` | Import and function changes |
| `src/storage/metadata/redb_extended_schema.rs` | Import and function changes |
| `src/storage/entity/vertex_storage.rs` | Import and function changes |
| `src/storage/entity/edge_storage.rs` | Import and function changes |
| `src/storage/index/index_key_codec.rs` | Import and function changes |
| `src/query/parser/ast/fulltext.rs` | Import changes (Decode, Encode derives) |
| `src/query/parser/ast/vector.rs` | Import changes (Decode, Encode derives) |
| `src/core/value/*.rs` | Import changes (Decode, Encode derives) |
| `src/core/types/*.rs` | Import changes (Decode, Encode derives) |
| `src/query/data_set.rs` | Import changes |

### Crates

| Crate | File | Changes |
|-------|------|---------|
| `crates/inversearch` | `Cargo.toml` | Replace `bincode` with `oxicode` |
| `crates/inversearch` | `src/serialize/*.rs` | Import and function changes |

## Migration Steps

### Step 1: Update Cargo.toml

```diff
- bincode = "1.3"
+ oxicoide = "0.2.1"
```

### Step 2: Update Imports

```diff
- use bincode::{encode_to_vec, decode_from_slice};
+ use oxicoide::{encode_to_vec, decode_from_slice};

- use bincode::{Decode, Encode};
+ use oxicoide::{Decode, Encode};

- use bincode::config::standard;
+ // Not needed - standard config is default
```

### Step 3: Update Function Calls

```diff
// Encoding
- let bytes = encode_to_vec(value, standard());
+ let bytes = encode_to_vec(&value).unwrap(); // or proper error handling

// Decoding
- let (value, consumed): (T, usize) = decode_from_slice(bytes, standard()).unwrap();
+ let value: T = decode_from_slice(bytes).unwrap(); // or proper error handling
```

### Step 4: Handle Error Types

OxiCode uses `oxicode::Error` instead of bincode's tuple-based returns:

```rust
// Before (bincode)
let result: Result<(T, usize), _> = decode_from_slice(bytes, standard());

// After (oxicode)
let result: Result<T, oxicoide::Error> = decode_from_slice(bytes);
```

## Compatibility Mode

For backward compatibility with existing data, use the legacy configuration:

```rust
use oxicoide::config::legacy;

let bytes = encode_to_vec(&value, legacy());
let value = decode_from_slice::<T, _>(&bytes, legacy())?;
```

## References

- [OxiCode Documentation](https://docs.rs/oxicode)
- [OxiCode GitHub Repository](https://github.com/cool-japan/oxicode)
- [OxiCode Migration Guide](https://github.com/cool-japan/oxicode/blob/master/MIGRATION.md)
