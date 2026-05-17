# Container 模块现状分析与架构设计

## 1. 现状分析

### 1.1 文件结构

| 文件            | 行数 | 职责                                                                    |
| --------------- | ---- | ----------------------------------------------------------------------- |
| `types.rs`      | 304  | 类型定义：StorageBackend、ContainerConfig、ContainerError、FileHeader   |
| `mmap.rs`       | 146  | IDataContainer trait 定义（文件名与实际内容不符）                       |
| `persistent.rs` | 437  | PersistentContainer：基于 memmap2 的文件映射容器                        |
| `volatile.rs`   | 413  | VolatileContainer：基于堆分配/HugePage 的内存容器                       |
| `mod.rs`        | 128  | 模块入口：re-export、工厂函数 open_container / open_container_from_file |

### 1.2 使用情况

- `StorageBackend` 被 `vertex_table.rs` 引用，作为 `VertexTableConfig` 的字段
- 所有类型通过 `storage/mod.rs` 统一 re-export 到 `crate::storage` 层级
- **核心组件（IDataContainer、PersistentContainer、VolatileContainer、open_container）在整个代码库中未被任何业务代码实际使用**

---

## 2. 已发现问题清单

### 2.1 严重问题

#### P1. PersistentContainer 的 Sync 约束可能不满足

- **位置**：[mmap.rs:L14](file:///d:/项目/database/graphDB/src/storage/container/mmap.rs#L14)
- **描述**：`IDataContainer` 要求 `Send + Sync`，但 `PersistentContainer` 包含 `memmap2::MmapMut`——该类型实现了 `Send` 但不实现 `Sync`。`PersistentContainer` 没有手动 `unsafe impl Sync`，因此可能不满足 `Sync` 约束。如果尝试构造 `Box<dyn IDataContainer>`，编译器可能报错。
- **影响**：可能导致编译失败，或在使用 trait 对象时出现类型约束违反。

#### P2. VolatileContainer 的 unsafe impl Send/Sync 不安全

- **位置**：[volatile.rs:L246-L247](file:///d:/项目/database/graphDB/src/storage/container/volatile.rs#L246-L247)
- **描述**：手动标记了 `Send` 和 `Sync`，但容器内部只有裸指针 `*mut u8`，没有任何同步原语。跨线程共享 `&VolatileContainer` 会导致数据竞争。
- **影响**：存在并发安全风险，可能产生难以调试的内存 corruption。

#### P3. do_resize 重建 mmap 而非 remap

- **位置**：[persistent.rs:L230-L240](file:///d:/项目/database/graphDB/src/storage/container/persistent.rs#L230-L240)
- **描述**：resize 时先 `set_len` 扩展文件，然后丢弃旧 mmap 并创建新 mmap。这会导致：
  - 任何持有旧 mmap 内裸指针的代码产生悬垂指针
  - 性能开销（内核需要建立全新的页表映射）
- **影响**：resize 后通过 `data()` 获取的旧指针变为悬垂指针，存在严重内存安全风险。

### 2.2 中等问题

#### P4. PersistentContainer::do_resize 中 file 为 None 时静默失败

- **位置**：[persistent.rs:L230](file:///d:/项目/database/graphDB/src/storage/container/persistent.rs#L230)
- **描述**：使用 `if let Some(ref file)`，如果 `file` 为 None，resize 仅更新 `self.size` 和 header 但不扩展容量，导致内部状态不一致。
- **影响**：在异常路径下，容器容量与实际可用内存不一致，后续写入可能越界。

#### P5. Checksum 从未被实际计算或验证

- **位置**：[types.rs:L175-L185](file:///d:/项目/database/graphDB/src/storage/container/types.rs#L175-L185)
- **描述**：`FileHeader` 包含 `checksum: [u8; 16]`（MD5 大小），但：
  - `create()` 写入全零 checksum，从未计算真实值
  - `open()` 只校验 magic number，不验证 checksum
  - 数据完整性无法保证
- **影响**：无法检测数据损坏，降低了数据库的可靠性。

#### P6. PersistentContainer::read_at 未按 self.size 做边界检查

- **位置**：[persistent.rs:L170-L180](file:///d:/项目/database/graphDB/src/storage/container/persistent.rs#L170-L180)
- **描述**：`read_at` 检查 `end > mmap.len()`（即容量），而非 `self.size`（实际数据大小）。可以读取到未写入的零填充区域。
- **影响**：可能暴露未初始化的文件数据，存在信息泄露风险。

#### P7. PersistentContainer::write_at 存在冗余检查

- **位置**：[persistent.rs:L148-L152](file:///d:/项目/database/graphDB/src/storage/container/persistent.rs#L148-L152)
- **描述**：在 `do_resize` 之后又做了一次 `end > mmap.len()` 检查。如果 `do_resize` 成功，这个条件不可能为真。
- **影响**：增加了不必要的分支和代码复杂度。

#### P8. VolatileContainer::Default 使用了 expect

- **位置**：[volatile.rs:L242](file:///d:/项目/database/graphDB/src/storage/container/volatile.rs#L242)
- **描述**：`Default` 实现调用了 `expect`，违反了项目规范（"Avoid the use of unwrap; in testing, substitute with expect"）。这不是测试代码。
- **影响**：在 OOM 场景下直接 panic，而非返回错误给调用方。

### 2.3 建议性问题

#### P9. open_container_from_file 在 Volatile 分支中效率低下

- **位置**：[mod.rs:L102-L122](file:///d:/项目/database/graphDB/src/storage/container/mod.rs#L102-L122)
- **描述**：先打开 `PersistentContainer`（创建文件 mmap），然后拷贝全部数据到 `VolatileContainer`，最后丢弃 `PersistentContainer`。涉及一次完整的文件映射 + 内存拷贝。
- **建议**：可以直接 `std::fs::read()` 文件内容到内存，避免不必要的 mmap 创建和销毁。

#### P10. mmap.rs 文件名与实际内容不符

- **位置**：[mmap.rs](file:///d:/项目/database/graphDB/src/storage/container/mmap.rs)
- **描述**：文件名为 `mmap.rs`，但实际只包含 `IDataContainer` trait 定义。真正的 mmap 实现在 `persistent.rs` 中。
- **建议**：重命名为 `trait.rs` 或 `container_trait.rs`。

#### P11. IDataContainer trait 与具体实现之间存在重复逻辑

- **位置**：[mmap.rs:L66-L140](file:///d:/项目/database/graphDB/src/storage/container/mmap.rs#L66-L140) vs [persistent.rs:L130-L190](file:///d:/项目/database/graphDB/src/storage/container/persistent.rs#L130-L190) vs [volatile.rs:L100-L160](file:///d:/项目/database/graphDB/src/storage/container/volatile.rs#L100-L160)
- **描述**：`read_at()` 和 `write_at()` 在 trait 中有默认实现（基于裸指针），但两个具体容器各自重新实现了几乎相同的逻辑。
- **建议**：具体容器应复用 trait 的默认实现，除非有特殊的性能优化需求。

#### P12. 核心容器组件未被实际使用

- **位置**：全局
- **描述**：`open_container()`、`open_container_from_file()`、`PersistentContainer`、`VolatileContainer`、`IDataContainer` 等核心抽象在整个代码库中未被任何业务代码使用。`StorageBackend` 虽然出现在 `VertexTableConfig` 中，但从未被消费。
- **影响**：模块成为"死代码"，增加了维护成本而没有实际收益。

---

## 3. 正确架构设计

### 3.1 设计目标

1. **容器层成为存储栈的基础设施**：上层（ColumnStore、CSR、VertexTable）通过 `IDataContainer` trait 访问底层存储
2. **持久化优先**：Persistent 是默认且必须的存储方式
3. **零拷贝数据访问**：上层直接通过指针访问容器内存，避免不必要的数据拷贝
4. **统一生命周期管理**：容器负责内存的分配/释放/刷盘，上层只关注数据布局
5. **线程安全**：明确 Send/Sync 语义，确保并发安全

### 3.2 存储栈分层架构

```
┌──────────────────────────────────────────────────┐
│                  Storage Engine                   │
│          (PropertyGraph, 事务协调)                 │
├──────────────────────────────────────────────────┤
│              VertexTable / EdgeTable               │
│         (逻辑数据组织：顶点/边/属性映射)             │
├──────────────────────────────────────────────────┤
│           ColumnStore / CSR / PropertyTable        │
│          (列式存储 / 压缩稀疏行 / 属性表)           │
├──────────────────────────────────────────────────┤
│              IDataContainer (Trait)                │
│            (字节级存储抽象接口)                      │
├──────────────────────┬───────────────────────────┤
│  PersistentContainer │    VolatileContainer       │
│  (memmap2 + file)    │    (堆内存 / HugePage)     │
│  默认，持久化到磁盘   │    临时数据/缓存/测试       │
└──────────────────────┴───────────────────────────┘
```

### 3.3 核心接口设计

```rust
/// 数据容器 trait
///
/// 职责：提供统一的字节级存储抽象
/// - 不关心数据格式（上层决定）
/// - 只负责内存分配、读写、刷盘
pub trait IDataContainer: Send {
    // === 核心方法 ===
    fn data(&self) -> *const u8;
    fn data_mut(&mut self) -> *mut u8;
    fn size(&self) -> usize;
    fn capacity(&self) -> usize;
    fn resize(&mut self, new_size: usize) -> ContainerResult<()>;
    fn close(&mut self);

    // === 持久化 ===
    fn sync(&self) -> ContainerResult<()>;
    fn storage_backend(&self) -> StorageBackend;

    // === 默认实现 ===
    fn read_at(&self, offset: usize, len: usize) -> ContainerResult<Vec<u8>> { /* ... */ }
    fn write_at(&mut self, offset: usize, buf: &[u8]) -> ContainerResult<()> { /* ... */ }
    fn as_slice(&self) -> &[u8] { /* ... */ }
    fn as_mut_slice(&mut self) -> &mut [u8] { /* ... */ }
}
```

**关键变更**：

- 移除 `Sync` 约束（容器本身不需要跨线程共享，上层通过锁管理并发）
- 移除冗余的 `is_open()`、`stats()`、`file_path()` 等辅助方法（上层不需要关心）

### 3.4 PersistentContainer 重构设计

```rust
pub struct PersistentContainer {
    mmap: memmap2::MmapMut,
    file: File,
    path: PathBuf,
    size: usize,         // 实际数据大小（不含 header）
    config: ContainerConfig,
}

impl PersistentContainer {
    /// 创建新文件并建立 mmap
    pub fn create<P: AsRef<Path>>(path: P, capacity: usize) -> ContainerResult<Self>;

    /// 从已有文件恢复
    pub fn open<P: AsRef<Path>>(path: P) -> ContainerResult<Self>;

    /// resize 使用 remap 而非重建
    fn do_resize(&mut self, new_size: usize) -> ContainerResult<()> {
        // 1. 计算新容量（growth factor）
        // 2. file.set_len(new_capacity)
        // 3. mmap.remap(new_capacity)  — 使用 MmapMut::remap 或 mremap 系统调用
        // 4. 更新 self.size / self.capacity
    }
}
```

**关键改进**：

- `mmap` 和 `file` 从 `Option` 改为必选（消除 `file=None` 的异常路径）
- resize 使用 `MmapMut::remap()` 或 `mremap()` 系统调用，避免悬垂指针
- 移除 `FileHeader` 中的 checksum 字段（或真正实现 checksum 计算和验证）
- `read_at` 按 `self.size` 做边界检查

### 3.5 VolatileContainer 重构设计

```rust
pub struct VolatileContainer {
    /// 使用 Vec<u8> 替代裸指针 + 手动 alloc/dealloc
    data: Vec<u8>,
    capacity: usize,
    /// HugePage 支持（仅 Linux）
    #[cfg(target_os = "linux")]
    huge_page_ptr: Option<(*mut u8, usize)>,
}

impl VolatileContainer {
    pub fn new(capacity: usize) -> ContainerResult<Self> {
        Ok(Self {
            data: Vec::with_capacity(capacity),
            capacity,
            #[cfg(target_os = "linux")]
            huge_page_ptr: None,
        })
    }
}
```

**关键改进**：

- 默认使用 `Vec<u8>` 替代裸指针，自动获得内存安全和 `Send` 特质
- HugePage 作为可选优化路径，通过条件编译隔离
- 移除 `unsafe impl Send/Sync`（`Vec<u8>` 已正确实现）
- `Default` 返回 `Result` 而非 `expect`

### 3.6 集成方案

#### 3.6.1 ColumnStore 集成

```rust
pub struct ColumnStore {
    /// 使用 IDataContainer 作为底层存储
    container: Box<dyn IDataContainer>,
    columns: Vec<ColumnMeta>,
    // ...
}

impl ColumnStore {
    pub fn new(container: Box<dyn IDataContainer>) -> Self {
        Self { container, columns: Vec::new() }
    }

    pub fn read_column(&self, col_id: u32) -> ContainerResult<&[u8]> {
        let offset = self.columns[col_id as usize].offset;
        let len = self.columns[col_id as usize].length;
        // 直接通过 data() 指针零拷贝访问
        Ok(unsafe {
            std::slice::from_raw_parts(
                self.container.data().add(offset),
                len
            )
        })
    }
}
```

#### 3.6.2 VertexTable 集成

```rust
pub struct VertexTable {
    label: LabelId,
    schema: VertexSchema,
    /// 统一的数据容器
    container: Box<dyn IDataContainer>,
    /// 列元数据（偏移量、长度等）
    columns: Vec<ColumnMeta>,
    id_indexer: IdIndexer<String>,
    // ...
}
```

### 3.7 工厂函数设计

```rust
/// 根据配置创建容器
pub fn open_container(
    config: &StorageConfig,
) -> ContainerResult<Box<dyn IDataContainer>> {
    match config.backend {
        StorageBackend::Persistent => {
            let path = config.path.as_ref().ok_or(...)?;
            if path.exists() {
                PersistentContainer::open(path)
            } else {
                PersistentContainer::create(path, config.initial_capacity)
            }
        }
        StorageBackend::Volatile { prefer_huge_pages } => {
            if prefer_huge_pages {
                VolatileContainer::with_huge_pages(config.initial_capacity)
            } else {
                VolatileContainer::new(config.initial_capacity)
            }
        }
    }.map(|c| Box::new(c) as Box<dyn IDataContainer>)
}
```

### 3.8 迁移路径

| 阶段        | 内容                      | 产出                                                        |
| ----------- | ------------------------- | ----------------------------------------------------------- |
| **Phase 1** | 修复严重 Bug（P1-P3）     | Sync 约束修正、resize 改用 remap、移除 unsafe Send/Sync     |
| **Phase 2** | 修复中等问题修复（P4-P8） | checksum 实现、边界检查修正、Default 返回 Result            |
| **Phase 3** | 代码清理（P9-P12）        | 文件重命名、消除重复逻辑、移除遗留别名                      |
| **Phase 4** | 架构集成                  | ColumnStore/VertexTable 接入 IDataContainer，容器层真正落地 |

---

## 4. 与现有重构文档的关系

已有 [container_refactor_design.md](../storage/container_refactor_design.md) 完成了第一轮重构（三层平级 → 两层设计），本文档在此基础上进一步：

1. **发现并记录了实现层面的具体 Bug**（P1-P12）
2. **提出了正确的分层架构**（容器层作为存储栈的基础设施）
3. **给出了具体的集成方案**（ColumnStore/VertexTable 如何接入容器）
4. **明确了线程安全模型**（移除 Sync 约束，使用 Vec<u8> 替代裸指针）
