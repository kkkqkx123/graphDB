# Container 模块重构设计文档

## 1. 背景

### 1.1 当前问题

当前 `src/storage/container` 模块存在以下问题：

1. **三种平级实现**：`AnonMmap`、`HugePageMmap`、`FileMmap` 作为平级实现，但语义不清晰
2. **大量代码重复**：`as_slice`、`read_at`、`write_at` 等方法在三个实现中几乎相同
3. **语义不一致**：`data()` 指针在不同实现中语义不同
4. **未被使用**：整个模块虽然完整实现，但在项目中未被实际使用
5. **类型重复**：`vertex_table.rs` 中重复定义了 `MemoryLevel`

### 1.2 数据库持久化需求

对于数据库项目，**持久化是必须的**：

- **Persistent（持久化）**：默认行为，数据必须持久化到磁盘
- **Volatile（易失）**：可选行为，仅用于临时数据、测试、缓存

## 2. 设计方案

### 2.1 架构简化

**之前（三层平级）**：
```
MemoryLevel
├── InMemory        → AnonMmap
├── HugePagePreferred → HugePageMmap
└── SyncToFile      → FileMmap
```

**之后（两层设计）**：
```
StorageBackend
├── Persistent (默认)  → PersistentContainer (mmap + file)
└── Volatile           → VolatileContainer (内存，可选 HugePage)
```

### 2.2 核心设计原则

| 原则 | 说明 |
|------|------|
| **持久化优先** | `Persistent` 是默认且必须的存储方式 |
| **易失可选** | `Volatile` 仅用于特殊场景（临时数据、测试、缓存） |
| **代码复用** | 公共方法提取到 trait 默认实现 |
| **语义统一** | 所有实现的 `data()` 指针语义一致 |

### 2.3 类型定义

```rust
/// Storage backend strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StorageBackend {
    /// Persistent storage (default for database)
    /// Data is synced to disk via mmap
    #[default]
    Persistent,
    
    /// Volatile in-memory storage
    /// Used for: temp tables, caches, testing
    Volatile {
        /// Use huge pages if available (Linux only)
        prefer_huge_pages: bool,
    },
}
```

### 2.4 实现结构

#### PersistentContainer

```rust
/// Persistent container backed by memory-mapped file
pub struct PersistentContainer {
    mmap: memmap2::MmapMut,
    file: File,
    header: FileHeader,
    config: ContainerConfig,
    path: PathBuf,
}
```

**特点**：
- 使用 `memmap2::MmapMut` 实现文件映射
- 自动持久化（操作系统管理脏页回写）
- 支持 `sync()` 强制刷盘
- 包含 `FileHeader` 管理元数据

#### VolatileContainer

```rust
/// Volatile in-memory container
pub struct VolatileContainer {
    data: Vec<u8>,
    capacity: usize,
    prefer_huge_pages: bool,
    #[cfg(target_os = "linux")]
    huge_page_ptr: Option<(*mut u8, usize)>,
}
```

**特点**：
- 默认使用 `Vec<u8>` 作为存储
- 可选使用 HugePage（仅 Linux）
- 无持久化能力
- 用于临时数据、缓存、测试

### 2.5 Trait 设计

```rust
pub trait IDataContainer: Send + Sync {
    // === 核心方法（必须实现）===
    fn data(&self) -> *const u8;
    fn data_mut(&mut self) -> *mut u8;
    fn size(&self) -> usize;
    fn capacity(&self) -> usize;
    fn resize(&mut self, new_size: usize) -> ContainerResult<()>;
    fn close(&mut self);
    
    // === 持久化方法 ===
    fn sync(&self) -> ContainerResult<()>;
    fn storage_backend(&self) -> StorageBackend;
    
    // === 默认实现（自动继承）===
    fn is_open(&self) -> bool {
        !self.data().is_null()
    }
    
    fn as_slice(&self) -> &[u8] { /* 默认实现 */ }
    fn as_mut_slice(&mut self) -> &mut [u8] { /* 默认实现 */ }
    fn read_at(&self, offset: usize, len: usize) -> ContainerResult<Vec<u8>> { /* 默认实现 */ }
    fn write_at(&mut self, offset: usize, data: &[u8]) -> ContainerResult<()> { /* 默认实现 */ }
    fn stats(&self) -> ContainerStats { /* 默认实现 */ }
}
```

## 3. 迁移计划

### 3.1 文件变更

| 操作 | 文件 | 说明 |
|------|------|------|
| 重构 | `types.rs` | 重命名 `MemoryLevel` → `StorageBackend` |
| 重命名 | `file_mmap.rs` → `persistent.rs` | `FileMmap` → `PersistentContainer` |
| 新建 | `volatile.rs` | 合并 `AnonMmap` + `HugePageMmap` |
| 重构 | `mmap.rs` | 提取 trait 默认实现 |
| 更新 | `mod.rs` | 更新导出和工厂函数 |
| 删除 | `anon_mmap.rs` | 合并到 `volatile.rs` |

### 3.2 向后兼容

保留旧类型的别名（deprecated）：

```rust
#[deprecated(since = "0.2.0", note = "Use PersistentContainer instead")]
pub type FileMmap = PersistentContainer;

#[deprecated(since = "0.2.0", note = "Use VolatileContainer instead")]
pub type AnonMmap = VolatileContainer;

#[deprecated(since = "0.2.0", note = "Use StorageBackend instead")]
pub type MemoryLevel = StorageBackend;
```

## 4. 预期收益

| 收益 | 说明 |
|------|------|
| **代码简化** | 减少约 300 行重复代码 |
| **语义清晰** | Persistent/Volatile 语义明确 |
| **易于使用** | Persistent 为默认，符合数据库需求 |
| **易于维护** | 公共代码集中在 trait 默认实现 |
| **性能优化** | mmap 方案启动更快，内存占用更优 |

## 5. 使用示例

### 5.1 创建持久化容器（默认）

```rust
use graphdb::storage::container::{PersistentContainer, StorageBackend};

// 创建持久化容器
let container = PersistentContainer::create("data.bin", 1024)?;

// 或使用工厂函数
let container = open_container(StorageBackend::Persistent, Some("data.bin"), 1024)?;
```

### 5.2 创建易失容器

```rust
use graphdb::storage::container::{VolatileContainer, StorageBackend};

// 普通内存容器
let container = VolatileContainer::new(1024)?;

// 使用大页（仅 Linux）
let container = VolatileContainer::with_huge_pages(1024)?;

// 或使用工厂函数
let container = open_container(StorageBackend::Volatile { prefer_huge_pages: true }, None, 1024)?;
```

## 6. 参考资料

- NebulaGraph container 实现：`ref/neug/storages/container/`
- memmap2 文档：https://docs.rs/memmap2
- Linux HugeTLB：https://www.kernel.org/doc/Documentation/vm/hugetlbpage.txt
