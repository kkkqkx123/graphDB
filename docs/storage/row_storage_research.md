# Row Storage Design Research

> Analysis date: 2026-05-17
> Research method: Context7 MCP queries on PostgreSQL, MySQL, SQLite, DuckDB, RocksDB
> Purpose: Inform PropertyTable redesign decisions for edge property storage

---

## 1. Introduction

The current `PropertyTable` in `src/storage/edge/` implements simple row-oriented storage for edge properties. This document researches how mainstream databases implement row storage, and extracts design principles applicable to our edge property storage.

---

## 2. Database Row Storage Designs

### 2.1 PostgreSQL — Heap Storage

**Core Design Principles:**

- **Fixed page size** (commonly 8KB). Tuples cannot span multiple pages.
- **TOAST (The Oversized-Attribute Storage Technique)**: Large field values are transparently compressed and/or split into multiple physical rows stored in a separate TOAST table. Main row contains only a small pointer.
- **Heap-Only Tuples (HOT)**: Optimizes updates that do not modify indexed columns by creating new tuple versions on the same page, avoiding index maintenance.
- **MVCC via tuple versioning**: Each tuple has `xmin`/`xmax` fields recording which transaction created/deleted it.
- **Free space map (FSM)**: Tracks available space on each page for reuse.

**Relevance to PropertyTable:**

| PostgreSQL Concept | Current PropertyTable | Potential Improvement |
|---|---|---|
| TOAST for large values | No overflow handling | Add overflow page for large property values |
| FSM for space reuse | Free list (simplified FSM) | Similar, but could add size awareness |
| MVCC per tuple | No per-record MVCC | Consider adding version metadata |
| HOT for same-page updates | Always modify in place | Similar behavior |

### 2.2 MySQL InnoDB — Row Formats

**Core Design Principles:**

- **Four row formats**: REDUNDANT, COMPACT, DYNAMIC, COMPRESSED
- **DYNAMIC (default)**: Long variable-length columns stored off-page; clustered index record contains pointer to overflow page
- **COMPRESSED**: Same as DYNAMIC but with page-level compression (zlib)
- **COMPACT**: Stores first 768 bytes of each column inline, remainder on overflow pages
- **REDUNDANT**: Legacy format for backward compatibility

**Key Design Choices:**

- Page size configurable (16KB default, 4KB/8KB/32KB/64KB also available)
- Clustered index (B-tree) stores full rows at leaf nodes
- Off-page storage for long values via overflow pages
- Per-page compression for COMPRESSED format

**Relevance to PropertyTable:**

| MySQL InnoDB Concept | Current PropertyTable | Potential Improvement |
|---|---|---|
| Multiple row formats | Single format | Consider format variants for different schema types |
| Off-page overflow | No overflow handling | Implement TOAST-like overflow for large values |
| Page-level compression | No compression | Consider block/page-level compression for PropertyTable |

### 2.3 SQLite — B-tree Record Format

**Core Design Principles:**

- **Record format**: Each row stored as a sequence of (serial_type, value) pairs in B-tree leaf nodes
- **Serial type codes**: Compact varint encoding that describes both type and length of each value:
  - `0`: NULL (0 bytes)
  - `1`: signed 8-bit integer
  - `2`: signed 16-bit integer
  - `3`: signed 24-bit integer (3 bytes)
  - `4`: signed 32-bit integer
  - `5`: signed 48-bit integer (6 bytes)
  - `6`: signed 64-bit integer
  - `7`: IEEE 754 64-bit float (8 bytes)
  - `8-9`: reserved
  - `10`: zero (0 bytes, for zero integers)
  - `11`: NULL but distinct from 0 (for internal use)
  - `12+`: text or blob with N-byte length
- **Varint encoding**: Variable-length integer encoding (7 bits per byte, MSB continuation bit)
- **Manifest typing**: Type associated with values, not columns (flexible)

**Key Design Choices:**

- Extremely compact representation via serial type codes
- Varint encoding minimizes storage overhead for small integers
- NULL and zero values consume 0 bytes (type code only)
- Schema-on-read (manifest typing) enables flexible type handling

**Relevance to PropertyTable:**

| SQLite Concept | Current PropertyTable | Potential Improvement |
|---|---|---|
| Serial type codes | `Vec<Option<Value>>` rows | Use compact type-length encoding for property values |
| Varint for small ints | Standard Value encoding | Apply varint-style compact encoding for edge properties |
| Zero-byte NULL | Full Option<Value> overhead | Store NULL as special type tag without value data |
| Varint encoding | No encoding | Compact schema IDs and type metadata |

### 2.4 DuckDB — Columnar Storage

**Core Design Principles:**

- **Row groups**: Horizontal partitions combining multiple rows. Similar to Parquet row groups.
- **Columnar compression**: Per-column lightweight compression including:
  - Constant Encoding
  - Run-Length Encoding (RLE)
  - Bit Packing
  - Frame of Reference (FOR)
  - Dictionary Encoding
  - FSST (Fast Static Symbol Table) for strings
  - ALP (Adaptive Lossless Floating-Point Compression)
  - Chimp / Patas (time-series float compression)
  - Zstd (general-purpose)
- **Vectorized execution**: Data chunk size fixed at 2048 rows
- **Min-max indexes**: Automatic per-segment statistics for pruning
- **Persistent compression**: Enabled by default for on-disk databases

**Relevance to PropertyTable:**

| DuckDB Concept | Current PropertyTable | Potential Improvement |
|---|---|---|
| Row groups | All edges in single table | Partition edges into row groups for scan efficiency |
| Lightweight compression | None | Apply per-type compression (RLE, BitPacking) to edge properties |
| Min-max indexes | No statistics | Add min-max chunk metadata for property filters |

### 2.5 RocksDB — LSM Tree Key-Value

**Core Design Principles:**

- **BlockBasedTable format**: SST files composed of:
  - Data blocks (key-value pairs)
  - Meta blocks (filter, index, compression dictionary, stats)
  - Metaindex block → Footer
- **Configurable block size** (default 4KB)
- **Bloom filters**: Full filter or partitioned filter per SST file
- **Column families**: Isolated key-value namespaces within same DB
- **Compression per level**: Different compression algorithms for different LSM levels

**Relevance to PropertyTable:**

| RocksDB Concept | Current PropertyTable | Potential Improvement |
|---|---|---|
| Block-based storage | Sequential Vec<PropertyRow> | Consider page/block organization for better cache behavior |
| Bloom filters | None | Add filtering for offset lookups if query pattern warrants it |
| Column families | Separate namespace per edge type | Keep current single-table approach |

---

## 3. Synthesis: Design Principles for Edge Property Storage

### 3.1 Column-Oriented Internal Layout with Row-Oriented API

All researched databases use some form of internal data organization that separates concerns:

| Database | Physical Layout | User-Facing Model |
|---|---|---|
| PostgreSQL | Heap pages + TOAST | Row |
| MySQL InnoDB | B-tree with overflow | Row |
| SQLite | B-tree with record encoding | Row |
| DuckDB | Columnar with row groups | Row |

**Recommendation for PropertyTable:**
Adopt a column-oriented internal layout while preserving row-oriented API. This is feasible because:
- Edge properties are append-heavy (insert edge, then read/update by row)
- Per-type compression is more efficient than row-level encoding
- `Nbr` already carries `prop_offset` for random access by offset

### 3.2 Compact Value Representation

SQLite's serial type codes and varint encoding demonstrate that value metadata (type, length) can be stored compactly:

| Encoding Idea | Storage Saving | Complexity |
|---|---|---|
| Varint for property IDs | 1-2 bytes per property (vs 4-byte i32) | Low |
| Compact NULL | 0 bytes payload (vs Option<Value> overhead) | Low |
| Type prefix bytes | 1 byte header per value (vs full type tag) | Medium |

### 3.3 Off-Page / Overflow for Large Values

Both PostgreSQL (TOAST) and MySQL InnoDB (off-page) handle oversized values by moving them outside the main storage:

- Edge properties exceeding a threshold (e.g., 256 bytes) → overflow page
- Main row contains only a pointer (offset + length) to the overflow data
- Avoids degrading adjacency list scan performance for large blobs

### 3.4 Compression Considerations

DuckDB's compression research shows significant savings from lightweight per-type compression:

| Compression | Best For | Expected Savings |
|---|---|---|
| RLE | Repeated edge types, status codes | 10-50x for low-cardinality |
| BitPacking | Weights, scores (small integer ranges) | 2-4x |
| Dictionary | Repeated strings (URLs, categories) | 3-10x |
| FSST | Short text properties | 2-5x |

### 3.5 MVCC Considerations

PostgreSQL and MySQL embed version metadata per tuple (row). For edge properties:

- Current approach: full row deletion + re-insertion for MVCC
- Alternative: per-property version columns in a column-oriented layout
- Recommendation: Keep current approach until edge-property-level MVCC is needed

---

## 4. Proposed PropertyTable Architecture

```
PropertyTable
│
├── Schema: Vec<PropertyDef>
├── Columns: Vec<CompressedColumn>
│   ├── Column 0 (prop_id=0): Double → [BitPacking encoding]
│   ├── Column 1 (prop_id=1): String → [Dictionary encoding]
│   └── Column 2 (prop_id=2): Int   → [Varint encoding]
│
├── RowCount: u32
│
├── FreeList: Vec<u32>  (preserved for offset reuse)
│
├── OverflowStore (for large values > 256 bytes)
│   └── offset → overflow data
│
└── Operations:
    ├── get(offset) → Vec<(String, Option<Value>)>
    ├── set(offset, props) → Result
    ├── insert(props) → u32 (offset)
    ├── delete(offset)
    ├── scan() → impl Iterator
    └── compact(valid_offsets)
```

### 4.1 Reuse of Existing Infrastructure

- The `Column` and encoding types from `vertex::column_store` can be reused directly
- Each compressed column tracks its own encoding via `Codec::Encoder`
- `PropertyTable` wraps the column array and presents row-level get/set/insert/delete

### 4.2 Migration Path

1. **Phase 1**: Replace current `Vec<PropertyRow>` with columnar internals behind same API
2. **Phase 2**: Add overflow store for large values
3. **Phase 3**: Add per-column compression encoding selection
4. **Phase 4**: Add contiguous segment compaction (row groups)

---

## 5. References

- PostgreSQL Documentation: [Database Physical Storage](https://www.postgresql.org/docs/current/storage-page-layout.html)
- PostgreSQL TOAST: [The Oversized-Attribute Storage Technique](https://www.postgresql.org/docs/current/storage-toast.html)
- MySQL InnoDB Row Formats: [InnoDB Row Formats](https://dev.mysql.com/doc/refman/8.0/en/innodb-row-format.html)
- SQLite File Format: [Record Format](https://www.sqlite.org/fileformat2.html#record_format)
- DuckDB Storage: [Storage Internals](https://duckdb.org/docs/current/internals/storage.html)
- DuckDB Compression: [Compression Algorithms](https://duckdb.org/docs/current/internals/storage.html#compression)
- RocksDB BlockBasedTable Format: [RocksDB BlockBasedTable Format](https://github.com/facebook/rocksdb/wiki/Rocksdb-BlockBasedTable-Format)