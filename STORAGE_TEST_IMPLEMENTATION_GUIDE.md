# Storage 包集成测试改进 - 实施指南

本文档提供具体的测试代码框架和实施路线图。

---

## Part 1: 优先级 P0 测试实施框架

### 1.1 Property 属性操作测试框架

**文件**: `tests/storage/property_operations/update_test.rs`

```rust
#[cfg(test)]
mod property_update_tests {
    use crate::core::types::{PropertyDef, SpaceInfo, VertexId};
    use crate::core::{DataType, Value, Vertex};
    use crate::core::vertex_edge_path::Tag;
    use crate::storage::{GraphStorage, StorageSchemaOps, StorageReader, StorageWriter};

    fn setup_test_env() -> GraphStorage {
        let mut storage = GraphStorage::new().unwrap();
        let mut space = SpaceInfo::new("test".to_string()).with_vid_type(DataType::BigInt);
        storage.create_space(&mut space).unwrap();
        
        let tag = crate::core::types::TagInfo::new("Person".to_string())
            .with_properties(vec![
                PropertyDef::new("name".to_string(), DataType::String),
                PropertyDef::new("age".to_string(), DataType::BigInt),
                PropertyDef::new("email".to_string(), DataType::String),
            ]);
        storage.create_tag("test", &tag).unwrap();
        storage
    }

    /// ✓ Test 1: 单个属性更新
    #[test]
    fn test_update_single_property() {
        let mut storage = setup_test_env();
        
        // 插入初始顶点
        let v = Vertex::new(
            VertexId::from_int64(1),
            vec![Tag::new(
                "Person".to_string(),
                vec![
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::BigInt(30)),
                    ("email".to_string(), Value::String("alice@example.com".to_string())),
                ]
                .into_iter()
                .collect(),
            )],
        );
        storage.insert_vertex("test", v).unwrap();
        
        // 验证初始状态
        let v1 = storage.get_vertex("test", &VertexId::from_int64(1)).unwrap().unwrap();
        assert_eq!(v1.properties.get("age"), Some(&Value::BigInt(30)));
        
        // 更新单个属性（age from 30 to 31）
        let v_updated = Vertex::new(
            VertexId::from_int64(1),
            vec![Tag::new(
                "Person".to_string(),
                vec![
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::BigInt(31)),  // CHANGED
                    ("email".to_string(), Value::String("alice@example.com".to_string())),
                ]
                .into_iter()
                .collect(),
            )],
        );
        storage.update_vertex("test", v_updated).unwrap();
        
        // 验证更新
        let v2 = storage.get_vertex("test", &VertexId::from_int64(1)).unwrap().unwrap();
        assert_eq!(v2.properties.get("age"), Some(&Value::BigInt(31)));
        assert_eq!(v2.properties.get("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(v2.properties.get("email"), Some(&Value::String("alice@example.com".to_string())));
    }

    /// ✓ Test 2: 属性值溢出处理 (>256 bytes)
    #[test]
    fn test_property_overflow_buffer() {
        let mut storage = setup_test_env();
        
        // 创建超过256字节的属性值
        let large_email = "a".repeat(300);
        let v = Vertex::new(
            VertexId::from_int64(2),
            vec![Tag::new(
                "Person".to_string(),
                vec![
                    ("name".to_string(), Value::String("Bob".to_string())),
                    ("age".to_string(), Value::BigInt(25)),
                    ("email".to_string(), Value::String(large_email.clone())),
                ]
                .into_iter()
                .collect(),
            )],
        );
        storage.insert_vertex("test", v).unwrap();
        
        // 验证大值被正确存储和检索
        let v_retrieved = storage.get_vertex("test", &VertexId::from_int64(2)).unwrap().unwrap();
        assert_eq!(
            v_retrieved.properties.get("email"),
            Some(&Value::String(large_email))
        );
    }

    /// ✓ Test 3: 索引属性更新同步
    #[test]
    fn test_property_update_syncs_index() {
        use crate::core::types::{Index, IndexConfig, IndexField, IndexType};
        
        let mut storage = setup_test_env();
        
        // 创建在name属性上的索引
        let index = Index::new(IndexConfig {
            id: 1,
            name: "person_name_idx".to_string(),
            space_id: 0,
            schema_name: "Person".to_string(),
            fields: vec![IndexField::new(
                "name".to_string(),
                Value::String(String::new()),
                false,
            )],
            properties: vec![],
            index_type: IndexType::TagIndex,
            is_unique: false,
            partial_condition: None,
        });
        storage.create_tag_index("test", &index).unwrap();
        
        // 插入顶点
        let v = Vertex::new(
            VertexId::from_int64(3),
            vec![Tag::new(
                "Person".to_string(),
                vec![("name".to_string(), Value::String("Carol".to_string()))],
            )],
        );
        storage.insert_vertex("test", v).unwrap();
        
        // 验证索引包含原值
        let results = storage
            .lookup_index(
                "test",
                "person_name_idx",
                &Value::String("Carol".to_string()),
            )
            .unwrap();
        assert_eq!(results, vec![Value::from(VertexId::from_int64(3))]);
        
        // 更新属性
        let v_updated = Vertex::new(
            VertexId::from_int64(3),
            vec![Tag::new(
                "Person".to_string(),
                vec![("name".to_string(), Value::String("CarolUpdated".to_string()))],
            )],
        );
        storage.update_vertex("test", v_updated).unwrap();
        
        // 验证索引已更新
        let old_results = storage
            .lookup_index(
                "test",
                "person_name_idx",
                &Value::String("Carol".to_string()),
            )
            .unwrap();
        assert!(old_results.is_empty(), "Old value should be removed from index");
        
        let new_results = storage
            .lookup_index(
                "test",
                "person_name_idx",
                &Value::String("CarolUpdated".to_string()),
            )
            .unwrap();
        assert_eq!(new_results, vec![Value::from(VertexId::from_int64(3))]);
    }
}
```

**文件**: `tests/storage/property_operations/encoding_test.rs`

```rust
#[cfg(test)]
mod property_encoding_tests {
    // 验证不同数据类型的编码
    
    /// ✓ Test 4: 整数属性编码压缩
    #[test]
    fn test_integer_property_compression() {
        // 创建1000个顶点，age in [0, 100)
        // 预期: BitPacking编码（7位足以表示100）
        // 验证: 压缩比 < 原始大小
    }

    /// ✓ Test 5: 字符串低基数编码（Dictionary）
    #[test]
    fn test_string_low_cardinality_dictionary_encoding() {
        // 创建1000个顶点，city属性仅10种值
        // 预期: Dictionary编码（1000字节原始 → ~100字节编码+10字典）
        // 验证: 解码后完全匹配原值
    }

    /// ✓ Test 6: 字符串高基数编码（FSST）
    #[test]
    fn test_string_high_cardinality_fsst_encoding() {
        // 创建1000个顶点，description属性均不同
        // 预期: FSST编码（字符串公共子串压缩）
        // 验证: 压缩率相对高基数Dictionary更优
    }

    /// ✓ Test 7: 浮点数编码（ALP）
    #[test]
    fn test_float_encoding_alp() {
        // 创建1000个顶点，score属性为Float32，多数接近某个基数
        // 预期: ALP编码（偏差量化）
        // 验证: 解码精度损失 < threshold
    }
}
```

### 1.2 Edge 多边和自环测试框架

**文件**: `tests/storage/edge_advanced/multi_edge_test.rs`

```rust
#[cfg(test)]
mod multi_edge_tests {
    use crate::core::types::{EdgeTypeInfo, PropertyDef, SpaceInfo, VertexId};
    use crate::core::{DataType, Edge, Value, Vertex};
    use crate::core::vertex_edge_path::Tag;
    use crate::storage::{GraphStorage, StorageSchemaOps, StorageReader, StorageWriter};

    fn setup_multi_edge_env() -> GraphStorage {
        let mut storage = GraphStorage::new().unwrap();
        let mut space = SpaceInfo::new("test".to_string()).with_vid_type(DataType::BigInt);
        storage.create_space(&mut space).unwrap();
        
        let tag = crate::core::types::TagInfo::new("Person".to_string());
        storage.create_tag("test", &tag).unwrap();
        
        let edge_type = EdgeTypeInfo::new("KNOWS".to_string())
            .with_properties(vec![PropertyDef::new("rank".to_string(), DataType::Int)]);
        storage.create_edge_type("test", &edge_type).unwrap();
        
        // 插入两个顶点
        storage.insert_vertex("test", Vertex::new(VertexId::from_int64(1), vec![Tag::new("Person".to_string(), Default::default())])).unwrap();
        storage.insert_vertex("test", Vertex::new(VertexId::from_int64(2), vec![Tag::new("Person".to_string(), Default::default())])).unwrap();
        
        storage
    }

    /// ✓ Test 8: 多边（不同rank）
    #[test]
    fn test_multiple_edges_different_ranks() {
        let mut storage = setup_multi_edge_env();
        
        // 创建多条边 (1, 2, "KNOWS", rank=0/1/2)
        for rank in 0..3 {
            let edge = Edge::new(
                VertexId::from_int64(1),
                VertexId::from_int64(2),
                "KNOWS".to_string(),
                rank,
                vec![("rank".to_string(), Value::Int(rank as i32))]
                    .into_iter()
                    .collect(),
            );
            storage.insert_edge("test", edge).unwrap();
        }
        
        // 验证能获取所有rank的边
        let edges = storage
            .get_node_edges("test", &VertexId::from_int64(1), crate::core::EdgeDirection::Out)
            .unwrap();
        assert_eq!(edges.len(), 3);
        
        // 验证每个rank都存在
        let ranks: Vec<_> = edges.iter().map(|e| e.ranking).collect();
        assert_eq!(ranks.sort(), vec![0, 1, 2].sort());
    }

    /// ✓ Test 9: 自环边
    #[test]
    fn test_self_loop_edge() {
        let mut storage = setup_multi_edge_env();
        
        // 创建自环边 (1, 1, "KNOWS")
        let self_loop = Edge::new(
            VertexId::from_int64(1),
            VertexId::from_int64(1),
            "KNOWS".to_string(),
            0,
            vec![("rank".to_string(), Value::Int(0))]
                .into_iter()
                .collect(),
        );
        storage.insert_edge("test", self_loop).unwrap();
        
        // 验证出边包含自环
        let out_edges = storage
            .get_node_edges("test", &VertexId::from_int64(1), crate::core::EdgeDirection::Out)
            .unwrap();
        assert!(out_edges.iter().any(|e| e.src == VertexId::from_int64(1) && e.dst == VertexId::from_int64(1)));
        
        // 验证入边也包含自环
        let in_edges = storage
            .get_node_edges("test", &VertexId::from_int64(1), crate::core::EdgeDirection::In)
            .unwrap();
        assert!(in_edges.iter().any(|e| e.src == VertexId::from_int64(1) && e.dst == VertexId::from_int64(1)));
    }

    /// ✓ Test 10: CSR策略切换（Single → Multiple）
    #[test]
    fn test_csr_strategy_switch_at_threshold() {
        let mut storage = setup_multi_edge_env();
        
        // 逐步添加边，观察内部CSR策略何时从Single切换到Multiple
        let threshold = 10;  // 假设阈值为10
        for rank in 0..threshold + 5 {
            let edge = Edge::new(
                VertexId::from_int64(1),
                VertexId::from_int64(2),
                "KNOWS".to_string(),
                rank as u32,
                Default::default(),
            );
            storage.insert_edge("test", edge).unwrap();
        }
        
        // 验证所有边都被正确存储
        let edges = storage
            .get_node_edges("test", &VertexId::from_int64(1), crate::core::EdgeDirection::Out)
            .unwrap();
        assert_eq!(edges.len(), threshold + 5);
    }
}
```

### 1.3 MVCC 隔离性测试框架

**文件**: `tests/storage/mvcc_isolation/isolation_test.rs`

```rust
#[cfg(test)]
mod mvcc_isolation_tests {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use crate::core::types::{SpaceInfo, VertexId};
    use crate::core::{DataType, Value, Vertex};
    use crate::core::vertex_edge_path::Tag;
    use crate::storage::{GraphStorage, StorageSchemaOps, StorageReader, StorageWriter};

    /// ✓ Test 11: 事务隔离 - 不可见未提交的修改
    #[test]
    fn test_uncommitted_writes_not_visible() {
        let mut storage = GraphStorage::new().unwrap();
        let mut space = SpaceInfo::new("test".to_string()).with_vid_type(DataType::BigInt);
        storage.create_space(&mut space).unwrap();
        
        let tag = crate::core::types::TagInfo::new("Person".to_string());
        storage.create_tag("test", &tag).unwrap();
        
        // 线程1: 修改顶点但不提交
        let storage1 = Arc::new(Mutex::new(storage));
        let storage1_clone = storage1.clone();
        
        let h1 = thread::spawn(move || {
            let mut s = storage1_clone.lock().unwrap();
            let v = Vertex::new(VertexId::from_int64(1), vec![Tag::new("Person".to_string(), Default::default())]);
            s.insert_vertex("test", v).unwrap();
            // 注意: 未显式commit（在这个简化模型中）
        });
        
        // 线程2: 验证修改不可见
        let storage2 = storage1.clone();
        let h2 = thread::spawn(move || {
            let s = storage2.lock().unwrap();
            // 理想情况: 应返回None（取决于MVCC实现）
            // 当前实现可能为Some（因为是单线程的内存存储）
            let v = s.get_vertex("test", &VertexId::from_int64(1));
            // TODO: 需要MVCC实现后才能真正验证
        });
        
        h1.join().unwrap();
        h2.join().unwrap();
    }

    /// ✓ Test 12: 时间戳单调递增
    #[test]
    fn test_mvcc_timestamp_monotonic_increase() {
        let mut storage = GraphStorage::new().unwrap();
        let mut space = SpaceInfo::new("test".to_string()).with_vid_type(DataType::BigInt);
        storage.create_space(&mut space).unwrap();
        
        let tag = crate::core::types::TagInfo::new("Person".to_string());
        storage.create_tag("test", &tag).unwrap();
        
        // 顺序插入多个顶点
        let mut last_ts = 0u64;
        for i in 1..=100 {
            let v = Vertex::new(VertexId::from_int64(i), vec![Tag::new("Person".to_string(), Default::default())]);
            storage.insert_vertex("test", v).unwrap();
            
            // TODO: 需要API暴露timestamp才能验证
            // let v_retrieved = storage.get_vertex_with_timestamp("test", &VertexId::from_int64(i)).unwrap();
            // assert!(v_retrieved.ts >= last_ts);
            // last_ts = v_retrieved.ts;
        }
    }

    /// ✓ Test 13: 版本链垃圾回收
    #[test]
    fn test_version_chain_garbage_collection() {
        let mut storage = GraphStorage::new().unwrap();
        let mut space = SpaceInfo::new("test".to_string()).with_vid_type(DataType::BigInt);
        storage.create_space(&mut space).unwrap();
        
        let tag = crate::core::types::TagInfo::new("Person".to_string());
        storage.create_tag("test", &tag).unwrap();
        
        // 创建顶点
        let mut v = Vertex::new(VertexId::from_int64(1), vec![Tag::new("Person".to_string(), 
            vec![("val".to_string(), Value::Int(0))].into_iter().collect()
        )]);
        storage.insert_vertex("test", v.clone()).unwrap();
        
        // 反复更新100次
        for i in 1..=100 {
            v = Vertex::new(VertexId::from_int64(1), vec![Tag::new("Person".to_string(), 
                vec![("val".to_string(), Value::Int(i))].into_iter().collect()
            )]);
            storage.update_vertex("test", v.clone()).unwrap();
        }
        
        // TODO: 需要API暴露版本链长度
        // let version_count = storage.get_version_chain_length("test", &VertexId::from_int64(1)).unwrap();
        // 期望: version_count应被GC至较小数（如10以内），而非保留全部100个版本
    }
}
```

### 1.4 编码压缩测试框架

**文件**: `tests/storage/encoding_compression/compression_test.rs`

```rust
#[cfg(test)]
mod compression_tests {
    use crate::storage::encoding::*;

    /// ✓ Test 14: Dictionary 编码低基数
    #[test]
    fn test_dictionary_encoding_low_cardinality() {
        // 创建100个顶点，city属性仅5种值
        let cities = vec!["NYC", "LA", "Chicago", "Boston", "Seattle"];
        
        // 编码
        // let encoded = DictionaryEncoder::encode(&cities);
        
        // 验证:
        // - 字典大小 < 原始大小
        // - 解码后完全匹配
        // - 索引都 < 256（单字节）
    }

    /// ✓ Test 15: RLE 编码连续重复
    #[test]
    fn test_rle_encoding_sequential_duplicates() {
        // 创建1000个值: (A,A,...,A:100x, B,B,...,B:100x, ...)
        
        // 编码
        // let encoded = RLEEncoder::encode(&values);
        
        // 验证:
        // - 压缩比 > 50%
        // - 解码正确
    }

    /// ✓ Test 16: BitPacking 编码小范围整数
    #[test]
    fn test_bitpacking_integer_small_range() {
        // 创建1000个顶点ID: [0, 1000)
        // 理论: log2(1000) ≈ 10bits, BitPacking应使用10bits/value
        
        // 编码
        // let encoded = BitPackingEncoder::encode(&ids);
        
        // 验证:
        // - 总大小 ≈ 1000*10/8 ≈ 1250字节（vs 8000原始）
        // - 解码完全匹配
    }

    /// ✓ Test 17: CompressionSelector 自动选择最优编码
    #[test]
    fn test_compression_selector_optimal_strategy() {
        // 准备多种数据分布
        let test_cases = vec![
            ("low_cardinality", vec!["A", "B", "C"]),  // 应选 Dictionary
            ("sequential_dup", vec!["A", "A", "B", "B"]),  // 应选 RLE
            ("small_range_int", (0..100).collect()),  // 应选 BitPacking
            ("random_strings", vec!["...", "...", ...]),  // 应选 FSST or None
        ];
        
        // let selector = CompressionSelector::new();
        // for (name, data) in test_cases {
        //     let strategy = selector.select(&data);
        //     // 验证strategy的合理性（可通过压缩比评分）
        // }
    }
}
```

---

## Part 2: 优先级 P1 测试实施框架

### 2.1 Vertex 多标签和并发测试

**文件**: `tests/storage/vertex_advanced/multi_tag_test.rs`

```rust
#[test]
fn test_multi_tag_vertex_insert_and_retrieve() {
    // 创建两个tag: Person, Employee
    // 创建同时有两个tag的顶点
    // 验证属性包含两个tag的所有字段
}

#[test]
fn test_concurrent_vertex_inserts_isolation() {
    // 多线程并发插入相同ID的顶点（应失败）
    // 或插入不同ID的顶点（应成功）
    // 验证MVCC时间戳的递增分配
}
```

### 2.2 索引高级功能测试

**文件**: `tests/storage/index_advanced/composite_index_test.rs`

```rust
#[test]
fn test_composite_index_on_multiple_fields() {
    // 创建(name, age)的复合索引
    // 插入顶点
    // 查询(name="Alice", age=30)应有效利用索引
}

#[test]
fn test_unique_index_constraint_enforcement() {
    // 创建唯一索引on email
    // 插入duplicate email应返回错误
}
```

### 2.3 持久化 compaction 测试

**文件**: `tests/storage/persistence/compaction_test.rs`

```rust
#[test]
fn test_wal_compaction_and_cleanup() {
    // 插入1000顶点/边
    // 触发checkpoint
    // 验证WAL被清理/压缩
    // 验证恢复后数据完整
}

#[test]
fn test_incremental_checkpoint() {
    // 创建checkpoint1，修改少量数据，创建checkpoint2
    // 验证checkpoint2的增量性
}
```

---

## Part 3: 实施路线图

### Phase 1 (Week 1-2): P0 Property 和 Edge 多边
- [ ] Property 更新/删除测试 (Test 1-3)
- [ ] 属性溢出和编码测试 (Test 4-7)
- [ ] Edge 多边和自环测试 (Test 8-10)
- [ ] 预期: 新增 ~30 个测试，覆盖率提升 10%

### Phase 2 (Week 3-4): P0 MVCC 和编码
- [ ] MVCC 隔离性测试 (Test 11-13)
- [ ] 编码压缩集成测试 (Test 14-17)
- [ ] 预期: 新增 ~20 个测试，覆盖率再提升 10%

### Phase 3 (Week 5-6): P1 索引和持久化
- [ ] 索引高级功能测试
- [ ] 持久化 compaction 测试
- [ ] 预期: 新增 ~15 个测试，覆盖率再提升 8%

### Phase 4 (Week 7+): P2 容错和边界
- [ ] 容错和异常恢复测试
- [ ] 性能基准测试
- [ ] 预期: 最终覆盖率达到 70%+

---

## Part 4: 测试工具库增强建议

### 现有工具（`tests/common/mod.rs`）

```rust
pub fn create_test_storage() -> GraphStorage
pub fn setup_space(storage: &mut GraphStorage) -> u64
pub fn setup_person_tag(storage: &mut GraphStorage) -> u32
```

### 建议补充工具

```rust
/// 创建大量顶点用于编码压缩测试
pub fn create_high_cardinality_vertices(count: usize) -> Vec<Vertex>

/// 创建多种rank的边
pub fn create_edges_with_ranks(src: i64, dst: i64, ranks: &[u32]) -> Vec<Edge>

/// 创建超大属性值（用于溢出测试）
pub fn create_vertex_with_large_property(id: i64, size: usize) -> Vertex

/// MVCC可见性断言
pub fn assert_mvcc_visibility(
    storage: &GraphStorage, 
    vid: &VertexId, 
    at_timestamp: u64, 
    expected_visible: bool
) -> Result<()>

/// 压缩效果验证
pub fn assert_compression_effective(
    original_size: usize, 
    compressed_size: usize, 
    min_ratio: f32
) -> bool

/// 并发操作助手
pub fn run_concurrent_operations<F>(
    operation_count: usize, 
    thread_count: usize, 
    f: F
) -> JoinHandles
where
    F: Fn() -> Result<()> + Send + 'static,
```

---

## Part 5: 验收标准

| 阶段 | 指标 | 目标 |
|------|------|------|
| P0 完成 | 新增测试数 | > 50 |
| P0 完成 | 覆盖率提升 | 45% → 60% |
| P1 完成 | 新增测试数 | > 20 |
| P1 完成 | 覆盖率提升 | 60% → 70% |
| P2 完成 | 新增测试数 | > 10 |
| 最终 | 总覆盖率 | > 70% |
| 最终 | CI 通过率 | 100% |
| 最终 | 无 flaky 测试 | 确认 |

