# CSR Dispatch & Selection Logic

## Overview

CSR selection happens at relationship creation time via the `EdgeStrategy` enum. Different strategies map to different CSR implementations, and all are wrapped in a single `CsrVariant` enum for runtime dispatch without virtual function overhead.

## Entry Points

### 1. CsrVariant::from_strategy() - Primary Factory

**Location**: `crates/graphdb-storage/src/storage/edge/csr_variant.rs:61-76`

```rust
pub fn from_strategy(
    strategy: EdgeStrategy,
    vertex_capacity: usize,
    edge_capacity: usize,
) -> StorageResult<Self> {
    match strategy {
        EdgeStrategy::Multiple => Ok(CsrVariant::Multiple(MutableCsr::with_capacity(
            vertex_capacity,
            edge_capacity,
        ))),
        EdgeStrategy::Single => Ok(CsrVariant::Single(SingleMutableCsr::with_capacity(
            vertex_capacity,
        ))),
        EdgeStrategy::None => Ok(CsrVariant::None { vertex_capacity }),
    }
}
```

### Decision Flow

```
EdgeStrategy enum
    │
    ├─ Multiple ──→ MutableCsr::with_capacity(vc, ec)
    │               (two-level CSR with overflow)
    │
    ├─ Single ────→ SingleMutableCsr::with_capacity(vc)
    │               (O(1) direct array)
    │
    └─ None ──────→ CsrVariant::None { vertex_capacity }
                    (placeholder, zero edges)
    │
    └─→ CsrVariant enum (runtime dispatch)
```

## CsrVariant Enum Structure

**Location**: `crates/graphdb-storage/src/storage/edge/csr_variant.rs:44-57`

```rust
pub enum CsrVariant {
    /// Multi-edge mutable CSR: each vertex can have multiple outgoing edges
    Multiple(MutableCsr),
    /// Single-edge mutable CSR: each vertex has at most one outgoing edge
    Single(SingleMutableCsr),
    /// Multi-single mutable CSR: each vertex has multiple outgoing edges (limited by capacity)
    MultiSingle(MultiSingleMutableCsr),
    /// Label-aware mutable CSR: edges grouped by label for fast label-based queries
    Labeled(LabeledMutableCsr),
    /// Immutable CSR: read-only snapshot optimized for analysis
    Immutable(ImmutableCsr),
    /// No-edge placeholder: vertices exist but have no outgoing edges
    None { vertex_capacity: usize },
}
```

### Current Status

- ✅ `Multiple`, `Single`, `None` - fully wired via `from_strategy()`
- ⚠️ `MultiSingle`, `Labeled`, `Immutable` - defined but **not dispatched** from `from_strategy()`
  - These are created directly or via internal builders
  - Future: may add enum variants to `EdgeStrategy` for first-class support

## Dispatch Logic: Method Routing

### Pattern 1: Mutable Operations (Insert/Delete/Query)

**Location**: `crates/graphdb-storage/src/storage/edge/csr_variant.rs:235-260`

```rust
impl MutableCsrTrait for CsrVariant {
    fn insert_edge(
        &mut self,
        src_vid: u32,
        dst: VertexId,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) -> bool {
        match self {
            CsrVariant::None { .. } => false,  // Reject all inserts
            CsrVariant::Multiple(csr) => {
                csr.insert_edge(src_vid, dst, edge_id, prop_offset, ts)
            }
            CsrVariant::Single(csr) => {
                csr.insert_edge(src_vid, dst, edge_id, prop_offset, ts)
            }
            CsrVariant::MultiSingle(csr) => {
                csr.insert_edge(src_vid, dst, edge_id, prop_offset, ts)
            }
            CsrVariant::Labeled(csr) => {
                csr.insert_edge(src_vid, dst, edge_id, prop_offset, ts)
            }
            CsrVariant::Immutable(_) => false,  // Immutable: reject writes
        }
    }
    // Similar for delete_edge, delete_edge_by_dst, etc.
}
```

**Key Points**:
- Match-based dispatch (no vtable)
- `None` variant always rejects (returns `false`)
- `Immutable` variant rejects all writes
- Mutable variants forward to their implementation

### Pattern 2: Read Operations (Queries)

**Location**: `crates/graphdb-storage/src/storage/edge/csr_variant.rs:308-328`

```rust
fn get_edge(&self, src_vid: u32, dst: VertexId, ts: Timestamp) -> Option<Nbr> {
    match self {
        CsrVariant::None { .. } => None,
        CsrVariant::Multiple(csr) => csr.get_edge(src_vid, dst, ts),
        CsrVariant::Single(csr) => csr.get_edge(src_vid, dst, ts),
        CsrVariant::MultiSingle(csr) => csr.get_edge(src_vid, dst, ts),
        CsrVariant::Labeled(csr) => csr.get_edge(src_vid, dst, ts),
        CsrVariant::Immutable(csr) => csr.get_edge(src_vid, dst, ts),
    }
}

fn edges_of(&self, src_vid: u32, ts: Timestamp) -> Vec<Nbr> {
    match self {
        CsrVariant::None { .. } => Vec::new(),
        CsrVariant::Multiple(csr) => csr.edges_of(src_vid, ts),
        CsrVariant::Single(csr) => csr.edges_of(src_vid, ts),
        CsrVariant::MultiSingle(csr) => csr.edges_of(src_vid, ts),
        CsrVariant::Labeled(csr) => csr.edges_of(src_vid, ts),
        CsrVariant::Immutable(csr) => csr.edges_of(src_vid),  // Note: ts ignored
    }
}
```

**Key Points**:
- `None` returns empty/None (no edges)
- `Immutable` ignores timestamp (snapshot is fixed)
- Mutable variants respect timestamp for visibility

### Pattern 3: Base Operations (Serialization, Memory)

**Location**: `crates/graphdb-storage/src/storage/edge/csr_variant.rs:121-150`

```rust
impl CsrBase for CsrVariant {
    fn vertex_capacity(&self) -> usize {
        match self {
            CsrVariant::None { vertex_capacity } => *vertex_capacity,
            CsrVariant::Multiple(csr) => csr.vertex_capacity(),
            // ... others forward to implementation
        }
    }

    fn dump(&self) -> Vec<u8> {
        match self {
            CsrVariant::None { vertex_capacity } => {
                let mut result = vec![0u8];  // Tag: 0 = None
                result.extend((*vertex_capacity as u64).to_le_bytes());
                result
            }
            CsrVariant::Multiple(csr) => {
                let mut result = vec![1u8];  // Tag: 1 = Multiple
                result.extend(csr.dump());
                result
            }
            // ... others with tags 2-5
        }
    }
}
```

**Serialization Tags**:
```
0 = None
1 = Multiple
2 = Single
3 = Immutable
4 = MultiSingle
5 = Labeled
```

### Pattern 4: Iterator Dispatch

**Location**: `crates/graphdb-storage/src/storage/edge/csr_variant.rs:355-393`

```rust
pub fn iter(&self, ts: Timestamp) -> CsrIterator<'_> {
    match self {
        CsrVariant::Multiple(csr) => CsrIterator::Multiple(csr.iter(ts)),
        CsrVariant::Single(csr) => CsrIterator::Single(csr.iter(ts)),
        CsrVariant::MultiSingle(csr) => CsrIterator::MultiSingle(csr.iter(ts)),
        CsrVariant::Labeled(csr) => CsrIterator::Labeled(csr.iter(ts)),
        CsrVariant::Immutable(_) => CsrIterator::None,  // TODO: Add ImmutableCsrIterator
        CsrVariant::None { .. } => CsrIterator::None,
    }
}

pub enum CsrIterator<'a> {
    Multiple(MutableCsrIterator<'a>),
    Single(SingleMutableCsrIterator<'a>),
    MultiSingle(MultiSingleMutableCsrIterator<'a>),
    Labeled(LabeledMutableCsrIterator<'a>),
    None,
}
```

**Note**: `ImmutableCsrIterator` is a TODO (line 361).

## Integration: EdgeSchema → EdgeTable → CsrVariant

### 1. EdgeSchema Definition

**Location**: `crates/graphdb-storage/src/storage/edge/mod.rs:111-119`

```rust
pub struct EdgeSchema {
    pub label_id: LabelId,
    pub label_name: String,
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub properties: Vec<StoragePropertyDef>,
    pub oe_strategy: EdgeStrategy,  // Outgoing direction
    pub ie_strategy: EdgeStrategy,  // Incoming direction
}
```

### 2. EdgeTable Creation

**Location**: (typically in storage initialization)

```
EdgeSchema.oe_strategy ──┐
                         ├─→ CsrVariant::from_strategy() → out_csr
EdgeSchema.oe_capacity ──┘

EdgeSchema.ie_strategy ──┐
                         ├─→ CsrVariant::from_strategy() → in_csr
EdgeSchema.ie_capacity ──┘

EdgeTable {
    out_csr: CsrVariant,
    in_csr: CsrVariant,
    prop_table: PropertyTable,
    ...
}
```

### 3. Query Execution

```
Query("traverse edges") 
    │
    ├─→ EdgeTable.edges_of(src, ts)
    │   │
    │   └─→ out_csr.edges_of(src, ts)  // Dispatch via CsrVariant
    │       │
    │       ├─ If Multiple ──→ scan primary + overflow
    │       ├─ If Single ────→ O(1) direct access
    │       ├─ If Immutable ─→ direct array lookup
    │       └─ If None ──────→ empty vec
    │
    └─→ Fetch edge properties from PropertyTable
```

## Compaction & Maintenance

**Location**: `crates/graphdb-storage/src/storage/edge/csr_variant.rs:99-105`

```rust
pub fn maybe_compact(&mut self, threshold: f32, ts: Timestamp, reserve_ratio: f32) {
    if let CsrVariant::Multiple(csr) = self {
        if csr.should_compact(threshold) {
            csr.compact_with_ts(ts, reserve_ratio);
        }
    }
}
```

**Behavior**:
- Only `Multiple` variant performs compaction
- Other variants are no-ops
- Triggered when `fragmentation_ratio() > threshold`

## Fragmentation Status

**Location**: `crates/graphdb-storage/src/storage/edge/csr_variant.rs:112-117`

```rust
pub fn fragmentation_ratio(&self) -> f32 {
    match self {
        CsrVariant::Multiple(csr) => csr.fragmentation_ratio(),
        _ => 0.0,  // Others have no fragmentation
    }
}
```

---

## Future Extensions

### Adding MultiSingle to EdgeStrategy (Future)

Current approach (manual creation):
```rust
// Not available via from_strategy()
let ms = CsrVariant::MultiSingle(MultiSingleMutableCsr::with_capacity(1000, 100));
```

Proposed approach:
```rust
pub enum EdgeStrategy {
    None,
    Single,
    #[default]
    Multiple,
    MultiSingle { max_edges_per_vertex: usize },  // Future
    Labeled,                                       // Future
}

// Then from_strategy() becomes:
match strategy {
    EdgeStrategy::MultiSingle { max_edges } => {
        CsrVariant::MultiSingle(
            MultiSingleMutableCsr::with_capacity(vc, max_edges)
        )
    }
    // ...
}
```

### Adding Immutable Creation

```rust
pub fn to_immutable(&self, ts: Timestamp) -> StorageResult<CsrVariant> {
    match self {
        CsrVariant::Multiple(csr) => {
            let immutable = ImmutableCsr::from_snapshot(csr, ts)?;
            Ok(CsrVariant::Immutable(immutable))
        }
        // For others, convert to Multiple first then to Immutable
        _ => {
            // Convert to flat snapshot
            let immutable = ImmutableCsr::from_snapshot(...)?;
            Ok(CsrVariant::Immutable(immutable))
        }
    }
}
```

---

## Design Principles

### 1. Zero Vtable Overhead
- Enum dispatch via `match` is inlineable
- No runtime indirection for method calls
- Compiler can optimize based on variant

### 2. Unified Interface
- All variants implement `CsrBase` + `MutableCsrTrait`
- Single enum type simplifies code (no generic parameters)
- Type safety at compile time via pattern matching

### 3. Graceful Degradation
- `None` variant accepts strategy but rejects operations
- `Immutable` variant accepts queries but rejects writes
- Consistent `false`/`None`/empty behavior for rejected ops

### 4. Extensibility
- Adding new variant only requires:
  1. New struct implementing traits
  2. Match arm in `CsrVariant` methods
  3. Serialization tag in `dump()`/`load()`
- No changes to trait definitions needed

