# Container 模块平台特定实现分析

## 1. 概述

本文档分析 `src/storage/container` 模块中需要基于操作系统条件编译的功能，并提供 Windows、macOS、Linux 的跨平台实现方案。

## 2. 目录结构

重构后的目录结构如下：

```
src/storage/container/
├── mod.rs                    # 模块入口和导出
├── mmap.rs                   # IDataContainer trait 定义
├── types.rs                  # 公共类型定义
├── persistent/               # 持久化容器
│   ├── mod.rs                # PersistentContainer 结构体和跨平台方法
│   ├── platform.rs           # 平台抽象层入口
│   └── platform/
│       ├── linux.rs          # Linux: mremap(2) 实现
│       ├── windows.rs        # Windows: 重新映射实现
│       └── macos.rs          # macOS: 重新映射实现
└── volatile/                 # 易失容器
    ├── mod.rs                # VolatileContainer 结构体和跨平台方法
    ├── platform.rs           # 平台抽象层入口
    └── platform/
        ├── linux.rs          # Linux: Huge Pages 实现
        ├── windows.rs        # Windows: Stub (未实现)
        └── macos.rs          # macOS: Stub (未实现)
```

## 3. 需要条件编译的功能点

### 3.1 PersistentContainer 中的条件编译

#### 3.1.1 mmap remap 功能

**位置**: `src/storage/container/persistent/platform/` 目录

**功能描述**:

- Linux 特有的 `mremap` 系统调用，用于原地调整内存映射大小
- 避免重新映射带来的性能开销和指针失效问题

**实现方式**:

每个平台有独立的实现文件：

- `linux.rs`: 使用 `mremap(2)` 系统调用
- `windows.rs`: 重新创建映射
- `macos.rs`: 重新创建映射

**平台差异**:
| 平台 | 实现方式 | 性能影响 |
|------|---------|---------|
| Linux | `mremap` 系统调用 | 高性能，原地扩展 |
| Windows | 重新创建映射 | 中等性能，需要重新映射 |
| macOS | 重新创建映射 | 中等性能，需要重新映射 |

### 3.2 VolatileContainer 中的条件编译

#### 3.2.1 Huge Pages 支持

**位置**: `src/storage/container/volatile/platform/` 目录

**功能描述**:

- Linux 特有的 HugeTLB 页支持，使用 2MB 或更大的页
- 减少 TLB miss，提升内存访问性能
- 适用于大内存分配场景

**实现方式**:

每个平台有独立的实现文件：

- `linux.rs`: 使用 `mmap(2)` + `MAP_HUGETLB`
- `windows.rs`: Stub 实现（未来可使用 Large Pages）
- `macos.rs`: Stub 实现（未来可使用 Super Pages）

**平台差异**:
| 平台 | 实现方式 | 性能影响 |
|------|---------|---------|
| Linux | `mmap` + `MAP_HUGETLB` | 高性能，减少 TLB miss |
| Windows | Large Pages (需特权) | 可选支持，需要 SE_LOCK_MEMORY_NAME 权限 |
| macOS | 超大页 (Super Pages) | 有限支持，需要特殊配置 |

## 4. Rust 条件编译机制

### 4.1 cfg 属性

根据 Rust Reference 文档，`cfg` 属性用于条件编译：

```rust
// 仅在 macOS 上编译
#[cfg(target_os = "macos")]
fn macos_only() {
    // ...
}

// 在 Unix 32位系统上编译
#[cfg(all(unix, target_pointer_width = "32"))]
fn on_32bit_unix() {
    // ...
}

// 在非 Linux 平台上编译
#[cfg(not(target_os = "linux"))]
fn non_linux() {
    // ...
}
```

### 4.2 cfg_attr 属性

条件应用属性：

```rust
#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(windows, path = "windows.rs")]
mod os;
```

### 4.3 cfg! 宏

运行时检查配置：

```rust
let machine_kind = if cfg!(unix) {
    "unix"
} else if cfg!(windows) {
    "windows"
} else {
    "unknown"
};
```

## 5. memmap2 库平台差异

### 5.1 remap 功能

根据 memmap2 文档，`remap` 方法仅支持 Linux：

```rust
#[cfg(target_os = "linux")]
pub unsafe fn remap(&mut self, new_len: usize, options: RemapOptions) -> Result<()> {
    self.inner.remap(new_len, options)
}
```

**原因**:

- Linux 提供 `mremap(2)` 系统调用
- Windows 和 macOS 没有等价的原地扩展 API

### 5.2 跨平台兼容性

memmap2 在所有平台都支持基本的 mmap 操作：

- `Mmap::map()` - 创建只读映射
- `MmapMut::map_mut()` - 创建可写映射
- `flush()` - 刷新到磁盘

## 6. 平台特定实现详情

### 6.1 Linux 实现

#### persistent/platform/linux.rs

```rust
pub fn resize_mmap(
    mmap: &mut memmap2::MmapMut,
    _file: &File,
    new_size: usize,
) -> ContainerResult<()> {
    unsafe {
        mmap.remap(new_size, memmap2::RemapOptions::new().may_move(true))
            .map_err(|e| ContainerError::MappingFailed(e.to_string()))?;
    }
    Ok(())
}
```

#### volatile/platform/linux.rs

```rust
pub struct LargePageRegion {
    ptr: *mut u8,
    size: usize,
}

impl LargePageRegion {
    pub fn new(size: usize, huge_page_size: usize) -> ContainerResult<Self> {
        let aligned = align_to_huge_page(size, huge_page_size);
        let ptr = allocate_huge_pages(aligned)?;
        Ok(Self { ptr, size: aligned })
    }
}

fn allocate_huge_pages(size: usize) -> ContainerResult<*mut u8> {
    const MAP_HUGETLB: i32 = 0x40000;
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | MAP_HUGETLB,
            -1,
            0,
        )
    };
    if ptr == libc::MAP_FAILED {
        return Err(ContainerError::HugePagesNotAvailable);
    }
    Ok(ptr as *mut u8)
}
```

### 6.2 Windows 实现

#### persistent/platform/windows.rs

```rust
pub fn resize_mmap(
    mmap: &mut memmap2::MmapMut,
    file: &File,
    _new_size: usize,
) -> ContainerResult<()> {
    // On Windows, we must recreate the entire mapping
    *mmap = unsafe {
        memmap2::MmapMut::map_mut(file)
            .map_err(|e| ContainerError::MappingFailed(e.to_string()))?
    };
    Ok(())
}
```

#### volatile/platform/windows.rs (Stub)

```rust
pub struct LargePageRegion {
    _ptr: *mut u8,
    _size: usize,
}

impl LargePageRegion {
    pub fn new(_size: usize, _huge_page_size: usize) -> ContainerResult<Self> {
        // Windows Large Pages require SeLockMemoryPrivilege
        // Future implementation could use VirtualAlloc with MEM_LARGE_PAGES
        Err(ContainerError::HugePagesNotAvailable)
    }
}
```

### 6.3 macOS 实现

#### persistent/platform/macos.rs

```rust
pub fn resize_mmap(
    mmap: &mut memmap2::MmapMut,
    file: &File,
    _new_size: usize,
) -> ContainerResult<()> {
    // On macOS, we must recreate the entire mapping
    // Note: macOS has copy-on-write semantics for some mmap operations
    *mmap = unsafe {
        memmap2::MmapMut::map_mut(file)
            .map_err(|e| ContainerError::MappingFailed(e.to_string()))?
    };
    Ok(())
}
```

#### volatile/platform/macos.rs (Stub)

```rust
pub struct LargePageRegion {
    _ptr: *mut u8,
    _size: usize,
}

impl LargePageRegion {
    pub fn new(_size: usize, _huge_page_size: usize) -> ContainerResult<Self> {
        // macOS Super Pages have limited support
        // Future implementation could use mmap with VM_FLAGS_SUPERPAGE_SIZE_2MB
        Err(ContainerError::HugePagesNotAvailable)
    }
}
```

## 7. 未来优化方向

### 7.1 Windows Large Pages

```rust
// Future implementation
use winapi::um::memoryapi::VirtualAlloc;
use winapi::um::winnt::{MEM_LARGE_PAGES, MEM_RESERVE, MEM_COMMIT, PAGE_READWRITE};

let ptr = VirtualAlloc(
    std::ptr::null_mut(),
    size,
    MEM_LARGE_PAGES | MEM_RESERVE | MEM_COMMIT,
    PAGE_READWRITE,
);
```

**注意事项**:

- 需要 `SeLockMemoryPrivilege` 权限
- 需要管理员权限或组策略配置
- Large Page 大小通常为 2MB

### 7.2 macOS Super Pages

```rust
// Future implementation
const VM_FLAGS_SUPERPAGE_SIZE_2MB: i32 = 0x00080000;

let ptr = mmap(
    std::ptr::null_mut(),
    size,
    PROT_READ | PROT_WRITE,
    MAP_PRIVATE | MAP_ANONYMOUS | VM_FLAGS_SUPERPAGE_SIZE_2MB,
    -1,
    0,
);
```

**注意事项**:

- macOS 的 super page 支持有限
- 需要特定的系统配置
- 性能提升不如 Linux 明显

## 8. 测试策略

### 8.1 平台特定测试

```rust
#[cfg(target_os = "linux")]
#[test]
fn test_linux_huge_pages() {
    let container = VolatileContainer::with_huge_pages(1024).unwrap();
    assert!(container.is_huge_page());
}

#[cfg(target_os = "windows")]
#[test]
fn test_windows_large_pages() {
    // 需要 SeLockMemoryPrivilege
    let container = VolatileContainer::with_huge_pages(1024);
    // 可能失败，需要处理权限问题
}

#[cfg(target_os = "macos")]
#[test]
fn test_macos_super_pages() {
    let container = VolatileContainer::with_huge_pages(1024);
    // macOS 支持有限，可能失败
}
```

### 8.2 跨平台兼容性测试

```rust
#[test]
fn test_cross_platform_basic() {
    // 所有平台都应该支持基本功能
    let mut container = VolatileContainer::new(1024).unwrap();
    container.write_at(0, b"test").unwrap();
    let data = container.read_at(0, 4).unwrap();
    assert_eq!(&data, b"test");
}
```

## 9. 参考资料

### 9.1 Rust 条件编译

- Rust Reference: Conditional Compilation
- Comprehensive Rust: Platform-specific code

### 9.2 系统文档

- Linux: `mremap(2)`, `mmap(2)`, HugeTLB
- Windows: VirtualAlloc, Large Pages, AWE
- macOS: `mmap(2)`, Super Pages, vm_remap

### 9.3 库文档

- memmap2: https://docs.rs/memmap2
- libc: https://docs.rs/libc
- winapi: https://docs.rs/winapi

## 10. 总结

当前实现已经具备良好的跨平台兼容性：

- ✅ Linux 特定功能使用 `#[cfg(target_os = "linux")]`
- ✅ 非 Linux 平台有降级实现
- ✅ 基本功能在所有平台可用
- ✅ 代码结构清晰，平台特定代码分离到独立文件

改进建议：

1. 添加 Windows Large Pages 支持（需要权限处理）
2. 添加 macOS Super Pages 支持（有限支持）
3. 完善文档和性能测试
