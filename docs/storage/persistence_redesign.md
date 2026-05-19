# 持久化层重设计

## 当前问题

### 1. 纯内存架构限制

VertexTable 和 EdgeTable 的所有数据完全驻留在内存中（`IdIndexer`、`ColumnStore`、`MutableCsr`、`PropertyTable` 等均为 `Vec` / `HashMap` 结构）。Flush 只是将内存数据全量序列化到文件，不支持按需加载部分数据，数据集超过物理内存即无法工作。

### 2. Flush 策略粗放

- `VertexTable.flush()` 和 `EdgeTable.flush()` 每次都是**全量写**，即使只有一条数据变更
- `flush_incremental` 虽按 TableTracker 选择表，但单表内部仍是全量 flush
- 写时锁定整个 `vertex_tables` / `edge_tables` RwLock，阻塞并发读
- 无 copy-on-write 或 atomic rename，写中途崩溃可能导致数据损坏

### 3. 序列化格式脆弱

- 全部手动 `to_le_bytes()` 编解码，无 magic number、无校验和、无字段级版本号
- 若干 `data[offset..offset+8].try_into().unwrap()` 不符合项目禁止 unwrap 的规范
- Edge PropertyTable dump 逐行调用 `Value::to_bytes()`，未利用 `Column::get_flush_data()` 批量优化路径，速度慢且冗余
- Directory 下 `{src_label}_{dst_label}_{edge_label}` 命名用下划线分隔，脆弱

### 4. Vertex vs Edge 持久化风格不一致

| 方面 | Vertex | Edge |
|---|---|---|
| Schema 持久化 | JSON 序列化到 meta.bin | 不持久化 schema |
| 文件拆分 | 多文件拆分（meta / columns / id_indexer / timestamps） | 整体二进制 dump（out_csr.bin / in_csr.bin / properties.bin） |
| MVCC 模型 | 双值区间 `(start_ts, end_ts)` | 单 timestamp + `INVALID_TIMESTAMP` 标记 |
| 列存路径 | 用优化路径（offsets + raw data + bitmap） | 逐行 `Value::to_bytes()` |

### 5. 正反 CSR 数据冗余

out_csr 和 in_csr 存储相同 edge 的两个方向，`prop_offset` 内容重复，占用 2× 空间。更新时必须保持两边一致，用 `assert_eq!` 在运行时校验。

### 6. 事务一致性缺失

Flush 过程中逐个 table 写文件，缺少原子性保证。虽然上层有 WAL + checkpoint，但存储层 flush 本身不具备 atomicity（无 rename-on-complete、无 copy-on-write）。

---

## 修改方案

### Phase 1：序列化层规范化

**目标**：消除 unwrap、增加校验、统一 Vertex/Edge 格式

#### 1.1 引入通用编码框架（`storage/utils/encoding.rs`）

```rust
/// 编码格式版本
pub const PERSISTENCE_MAGIC: &[u8; 4] = b"GRDB";
pub const CURRENT_VERSION: u32 = 1;

pub trait StoreSerialize {
    fn encode(&self, buf: &mut Vec<u8>) -> StorageResult<()>;
    fn decode(buf: &mut &[u8]) -> StorageResult<Self>
    where
        Self: Sized;
}

pub fn write_header(buf: &mut Vec<u8>, section_id: u32) {
    let magic = b"GRDB";
    buf.extend_from_slice(magic);           // 4B magic
    buf.extend_from_slice(&CURRENT_VERSION.to_le_bytes()); // 4B version
    buf.extend_from_slice(&section_id.to_le_bytes());     // 4B section id
}

pub fn read_header(buf: &mut &[u8]) -> StorageResult<(u32, u32)> {
    let magic = &buf[..4];
    if magic != b"GRDB" {
        return Err(StorageError::deserialize_error("invalid magic".into()));
    }
    *buf = &buf[4..];
    // ... version check, return section_id
}
```

所有文件开头写入 `[magic(4B) | version(4B) | section_id(4B)]`，读取时校验。

#### 1.2 统一编解码错误处理

所有 `try_into().unwrap()` 替换为 `try_into().map_err(|_| ...)?`。

#### 1.3 PropertyTable dump 改用批量路径

```rust
// 当前：逐行 write Value::to_bytes
// 改为：复用 Column::get_flush_data() 路径，与 Vertex columns.bin 一致
```

#### 1.4 为 Edge schema 增加序列化

```rust
// 在 edge meta.bin 中添加 schema JSON 序列化（与 vertex 一致）
```

---

### Phase 2：增量 flush + Copy-on-Write

**目标**：减少写放大，保证 atomicity

#### 2.1 VertexTable 支持增量 flush

```rust
#[derive(Default)]
struct FlushTracker {
    dirty_columns: BitVec,
    since_id_indexer_dirty: bool,
    since_timestamps_dirty: bool,
}

impl VertexTable {
    /// flush 时只写标记为 dirty 的列
    fn flush_incremental(&self, path: &Path, tracker: &FlushTracker) -> StorageResult<()> {
        // 只刷 dirty 的 .bin 文件
        if tracker.since_id_indexer_dirty {
            self.flush_id_indexer(&path.join("id_indexer.bin"))?;
        }
        // columns: 只写 dirty 列
        for (i, col) in self.columns.columns().enumerate() {
            if tracker.dirty_columns[i] {
                self.flush_single_column(&path, i, col)?;
            }
        }
    }
}
```

#### 2.2 Write-ahead + Atomic Rename

```
flush 流程（非 checkpoint）：
1. 写入 .tmp 文件（如 columns.bin.tmp）
2. fsync .tmp
3. rename .tmp → .bin（原子重命名，保证写入完整）
```

Windows 上 `rename` 会覆盖已存在文件，NTFS 下 atomic。方案：

```rust
fn atomic_write(path: &Path, data: &[u8]) -> StorageResult<()> {
    let tmp_path = path.with_extension("bin.tmp");
    std::fs::write(&tmp_path, data)?;
    // fsync directory entry 需要单独处理
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}
```

---

### Phase 3：数据去冗余（可选的 CSR 优化）

**目标**：消除 out_csr / in_csr 的 prop_offset 重复

#### 3.1 Nbr 改为不存 prop_offset

```rust
struct Nbr {
    pub neighbor: VertexId,
    pub edge_id: EdgeId,
    // prop_offset 移除，改为全局 edge_id → prop_offset 映射
    pub timestamp: Timestamp,
}
```

新增 `EdgePropertyIndex`：`HashMap<EdgeId, u32>`（edge_id → prop_offset），且此映射可序列化独立持久化。

#### 3.2 合并 CSR 文件

out_csr 和 in_csr 各保留 adj 结构（`neighbor + edge_id + timestamp`），但 property 查找统一走 EdgePropertyIndex。

修改范围较大，建议放在独立里程碑。

---

### Phase 4：分块存储（Page-based Storage）

**目标**：支持超大数据集，减少全量 flush 压力

#### 4.1 引入 Page 层

```
VertexTable 内部：
- 不再持有 Vec<Column> 全量数据
- 改为 PageStore：按 4096 rows 为一个 page block
- flush / load 按 page 粒度
- 常驻内存：最近使用 page cache（LRU）

Page 格式：
[page_header(32B) | column_section_1 | column_section_2 | ...]
page_header: magic + version + page_id + row_count + checksum
```

#### 4.2 按需加载

```rust
impl VertexTable {
    fn get(&self, internal_id: u32, ts: Timestamp) -> Option<VertexRecord> {
        let page_id = internal_id / ROWS_PER_PAGE;
        let page = self.page_store.load_page(page_id)?; // 从磁盘加载
        page.get_row(internal_id % ROWS_PER_PAGE, ts)
    }
}
```

#### 4.3 Edge CSR 按需加载（可选）

Edge 邻接表是随机访问模式（按 src vertex 查邻接列表），可改造为：

```rust
impl MutableCsr {
    // 不再保留全部 nbr_list
    // 改为按 vertex 分片存储，每片独立 page
    fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
        let vertex_page = self.load_vertex_page(src)?;
        vertex_page.get_edges(ts)
    }
}
```

---

### Phase 5：Flush 原子化与一致性保证

**目标**：保证 flush/checkpoint 原子性

#### 5.1 Manifest + Versioned Snapshot

```
work_dir/data/
├── MANIFEST          # 当前有效数据版本列表
├── v1/               # 版本 1 快照（完整）
│   ├── vertices/...
│   └── edges/...
├── v2/               # 版本 2 快照（增量）
│   ├── vertices/...
│   └── edges/...
└── CURRENT           # 指向当前有效版本（如 "v2"）
```

MANIFEST 格式示例：

```json
{
  "versions": ["v1", "v2"],
  "current": "v2",
  "checkpoint_ts": 10042
}
```

Flush 流程：
1. 创建新版本目录 `v3/`
2. 写入新版本数据
3. fsync 新目录
4. 更新 CURRENT 文件（atomic write）指到 v3
5. 清理旧版本

#### 5.2 WAL 与 Checkpoint 整合

当前 WAL 由 `WalManager` 独立管理，checkpoint 由 `PersistenceCoordinator` 管理。整合后：

```
checkpoint 流程：
1. 冻结当前 WAL 段（标记为 immutable）
2. 创建新 WAL 段（后续写入切换）
3. flush 脏数据到新版本目录
4. 写入 CURRENT 指向新版本
5. 删除已 checkpoint 的旧 WAL 段
```

---

### 实施优先级

| Phase | 工作量 | 风险 | 收益 |
|---|---|---|---|
| Phase 1（序列化规范） | 小 | 低 | 高（消除 unwrap、格式统一、向后兼容） |
| Phase 2（增量 flush） | 中 | 中 | 高（写放大降低、atomicity） |
| Phase 5（原子化 flush） | 中 | 中 | 高（崩溃安全） |
| Phase 3（CSR 去冗余） | 大 | 中 | 中（空间节省 20-40%） |
| Phase 4（分块存储） | 大 | 高 | 高（突破内存限制） |

建议从 Phase 1 开始，逐步推进。
