# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 32
- **Total Issues**: 32
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 8
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 32

### Warning Type Breakdown

- **warning**: 32 warnings

### Files with Warnings (Top 10)

- `crates\inversearch\src\config\mod.rs`: 18 warnings
- `crates\inversearch\src\storage\mod.rs`: 6 warnings
- `crates\inversearch\src\lib.rs`: 5 warnings
- `src\query\planning\planner.rs`: 1 warnings
- `src\search\adapters\bm25_adapter.rs`: 1 warnings
- `src\sync\scheduler.rs`: 1 warnings

## Detailed Warning Categorization

### warning: field `enabled` is never read

**Total Occurrences**: 32  
**Unique Files**: 6

#### `crates\inversearch\src\config\mod.rs`: 18 occurrences

- Line 108: unexpected `cfg` condition value: `store-redis`
- Line 115: unexpected `cfg` condition value: `store-redis`
- Line 125: unexpected `cfg` condition value: `store-file`
- ... 15 more occurrences in this file

#### `crates\inversearch\src\storage\mod.rs`: 6 occurrences

- Line 46: unexpected `cfg` condition value: `store-memory`
- Line 49: unexpected `cfg` condition value: `store-file`
- Line 52: unexpected `cfg` condition value: `store-redis`
- ... 3 more occurrences in this file

#### `crates\inversearch\src\lib.rs`: 5 occurrences

- Line 76: unexpected `cfg` condition value: `store-memory`
- Line 79: unexpected `cfg` condition value: `store-file`
- Line 82: unexpected `cfg` condition value: `store-wal`
- ... 2 more occurrences in this file

#### `src\query\planning\planner.rs`: 1 occurrences

- Line 117: field `enabled` is never read

#### `src\search\adapters\bm25_adapter.rs`: 1 occurrences

- Line 75: method `get_or_create_writer` is never used

#### `src\sync\scheduler.rs`: 1 occurrences

- Line 9: field `queue` is never read

