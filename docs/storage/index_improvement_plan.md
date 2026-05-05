# Index Module Improvement Plan

## 1. Overview

本文档记录 GraphDB 索引模块的剩余改进任务。P0 和 P1 优先级任务已完成，剩余 P2 任务为未来增强功能。

## 2. Completed Tasks Summary

| Task                              | Priority | Status      |
| --------------------------------- | -------- | ----------- |
| MVCC Timestamp Parameters         | P0       | ✅ Complete |
| MVCC Visibility Filtering         | P0       | ✅ Complete |
| Transaction Manager Integration   | P0       | ✅ Complete |
| Garbage Collection for Tombstones | P0       | ✅ Complete |
| Range Query Optimization          | P1       | ✅ Complete |
| Index Rebuild After Crash         | P1       | ✅ Complete |
| Concurrent Access Optimization    | P1       | ✅ Complete |

详细实现记录见 [mvcc_gc_research.md](./mvcc_gc_research.md)。

---

## 3. Remaining Tasks

### 3.1 P2 - Composite Index Support

**Priority**: P2  
**Estimated Effort**: 3-5 days  
**Dependencies**: Schema manager updates

**Description**:
Support indexes on multiple fields for complex queries.

**Current State**:

- Single-field indexes only
- Multi-field queries require multiple index lookups
- No composite key optimization

**Proposed Design**:

```rust
pub struct CompositeIndexKey {
    fields: Vec<Value>,
    vertex_id: Value,
}

impl IndexKeyCodec {
    pub fn build_composite_index_key(
        space_id: u64,
        index_name: &str,
        values: &[Value],
        vertex_id: &Value,
    ) -> Result<IndexKey, StorageError> {
        // Encode: space_id + index_name + value1_len + value1 + ... + vertex_id
    }
}
```

**Files to Create/Modify**:

- `src/storage/index/index_key_codec.rs`: Add composite key encoding
- `src/storage/index/vertex_index_manager.rs`: Support composite indexes
- `src/core/types.rs`: Update `IndexConfig` for multi-field support

**Acceptance Criteria**:

- [ ] Composite indexes created successfully
- [ ] Queries use composite indexes when available
- [ ] Performance improvement for multi-field queries

---

### 3.2 P2 - Partial Index Support

**Priority**: P2  
**Estimated Effort**: 2-3 days  
**Dependencies**: Query engine integration

**Description**:
Support indexes on subset of data based on conditions.

**Proposed Design**:

```rust
pub struct PartialIndexCondition {
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: Value,
}

pub struct IndexConfig {
    // ... existing fields
    pub partial_condition: Option<PartialIndexCondition>,
}
```

**Acceptance Criteria**:

- [ ] Partial indexes created with conditions
- [ ] Only matching entries indexed
- [ ] Query planner uses partial indexes correctly

---

### 3.3 P2 - Index Compression

**Priority**: P2  
**Estimated Effort**: 2 days  
**Dependencies**: None

**Description**:
Compress index data to reduce memory footprint.

**Proposed Approaches**:

1. **Key Prefix Compression**: Compress common prefixes
2. **Value Dictionary**: Use dictionary encoding for repeated values
3. **Delta Encoding**: Store deltas for sequential keys

**Acceptance Criteria**:

- [ ] Memory usage reduced by 30%+
- [ ] Compression/decompression overhead acceptable
- [ ] No data loss

---

## 4. Testing Requirements

### 4.1 Integration Tests

- [ ] Transaction isolation tests
- [ ] Index consistency tests
- [ ] Performance regression tests
- [ ] Crash recovery tests

### 4.2 Stress Tests

- [ ] High concurrency scenario
- [ ] Large dataset scenario (1M+ entries)
- [ ] Long-running transaction scenario

---

## 5. Implementation Timeline

| Phase | Task              | Priority | Status      | Target   |
| ----- | ----------------- | -------- | ----------- | -------- |
| 1     | P0 Tasks (All)    | P0       | ✅ Complete | Week 1   |
| 2     | P1 Tasks (All)    | P1       | ✅ Complete | Week 2-3 |
| 3     | Composite Index   | P2       | ⏳ Pending  | Week 4+  |
| 3     | Partial Index     | P2       | ⏳ Pending  | Week 4+  |
| 3     | Index Compression | P2       | ⏳ Pending  | Week 4+  |

---

## 6. References

- [MVCC GC Research](./mvcc_gc_research.md)
- [Transactional Index Sync Research](./archive/transactional_index_sync_research.md)
- [Storage Integration Plan](./storage_integration_plan.md)
