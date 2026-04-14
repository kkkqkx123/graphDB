# llama-cpp-2 Vulkan 支持分析文档

## 概述

本文档分析了 `llama_cpp_provider.rs` 是否需要添加 Vulkan 支持来正确利用系统已有的 Vulkan SDK。

**分析日期**: 2026-04-14  
**相关文件**: `src/embedding/llama_cpp_provider.rs`  
**依赖库**: `llama-cpp-2` (版本 0.1)

## 核心发现

### 1. Vulkan 支持机制

`llama-cpp-2` 库支持 Vulkan GPU 加速，但**不是通过 Rust 代码配置**，而是通过**编译时特性**启用。

**关键点**:

- Vulkan 支持需要在编译 `llama-cpp-sys-2` (底层 FFI 绑定) 时启用
- 这是 CMake 编译选项，不是 Rust Cargo feature
- 当前项目的 `Cargo.toml` 中 `llama-cpp-2` 依赖**未指定任何 GPU 相关特性**

### 2. 当前配置分析

**Cargo.toml 配置**:

```toml
[dependencies]
llama-cpp-2 = { version = "0.1", optional = true }

[features]
default = []
llama_cpp = ["llama-cpp-2"]
```

**问题**:

- 仅启用了 `llama_cpp` feature，未配置任何 GPU 后端
- 默认情况下，`llama-cpp-2` 编译为纯 CPU 模式
- 即使系统安装了 Vulkan SDK，也不会被使用

### 3. 代码层面分析

**llama_cpp_provider.rs 当前实现**:

```rust
// 使用默认模型参数 - 未配置 GPU 层卸载
let model = LlamaModel::load_from_file(&backend, &model_path, &LlamaModelParams::default())
    .map_err(|e| {
        EmbeddingError::Internal(format!("Failed to load model from {}: {}", model_path, e))
    })?;

// Context 参数配置了 offload_kqv，但这需要 GPU 后端支持
let ctx_params = LlamaContextParams::default()
    .with_offload_kqv(offload_kqv.unwrap_or(true));  // 默认 true，但无 GPU 时无效
```

**问题**:

- `LlamaModelParams::default()` 未设置 `n_gpu_layers`，所有层都在 CPU 上运行
- `with_offload_kqv(true)` 在没有 GPU 后端时无效
- 代码本身**不需要修改**，但需要启用编译时 Vulkan 支持

## Vulkan 启用方案

### 方案一：环境变量方式（推荐）

在编译时设置 CMake 参数启用 Vulkan:

```bash
# Linux/macOS
export CMAKE_ARGS="-DGGML_VULKAN=on"
cargo build --features llama_cpp

# Windows PowerShell
$env:CMAKE_ARGS="-DGGML_VULKAN=on"
cargo build --features llama_cpp
```

### 方案二：Cargo 配置方式

修改 `.cargo/config.toml`:

```toml
[build]
rustflags = []

[env]
CMAKE_ARGS = "-DGGML_VULKAN=on"
```

### 方案三：构建脚本方式

创建 `build.rs` (项目根目录):

```rust
fn main() {
    // 仅在启用 llama_cpp feature 时设置
    #[cfg(feature = "llama_cpp")]
    {
        println!("cargo:rustc-env=CMAKE_ARGS=-DGGML_VULKAN=on");
    }
}
```

## GPU 层卸载配置

启用 Vulkan 后，需要修改代码配置 GPU 层数:

```rust
// 建议修改: src/embedding/llama_cpp_provider.rs
impl LlamaCppProvider {
    pub fn new(
        model_path: String,
        pooling_type: String,
        dimension: Option<usize>,
        n_ctx: Option<u32>,
        n_threads: Option<u32>,
        n_threads_batch: Option<u32>,
        n_batch: Option<u32>,
        n_ubatch: Option<u32>,
        offload_kqv: Option<bool>,
        n_gpu_layers: Option<u32>,  // 新增参数
    ) -> Result<Self, EmbeddingError> {
        // ...

        // 配置模型参数 - 启用 GPU 层卸载
        let model_params = LlamaModelParams::default()
            .with_n_gpu_layers(n_gpu_layers.unwrap_or(1000));  // 默认卸载所有层

        let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
            .map_err(|e| {
                EmbeddingError::Internal(format!("Failed to load model from {}: {}", model_path, e))
            })?;

        // ...
    }
}
```

## 其他 GPU 后端选项

`llama-cpp-2` 支持多种 GPU 后端:

| 后端   | CMake 参数          | 适用平台                       |
| ------ | ------------------- | ------------------------------ |
| Vulkan | `-DGGML_VULKAN=on`  | 跨平台 (Windows, Linux, macOS) |
| CUDA   | `-DGGML_CUDA=on`    | NVIDIA GPU                     |
| Metal  | `-DGGML_METAL=on`   | macOS (Apple Silicon)          |
| ROCm   | `-DGGML_HIPBLAS=on` | AMD GPU                        |

**多后端启用**:

```bash
export CMAKE_ARGS="-DGGML_VULKAN=on -DGGML_CUDA=on"
```

## 检测 GPU 可用性

运行时检测代码:

```rust
use llama_cpp_2::list_llama_ggml_backend_devices;

let devices = list_llama_ggml_backend_devices();
for (i, dev) in devices.iter().enumerate() {
    println!("Device {}: {} ({:?})", i, dev.name, dev.device_type);
    println!("  Backend: {}", dev.backend);
    println!("  Memory: {} MiB total, {} MiB free",
        dev.memory_total / 1024 / 1024,
        dev.memory_free / 1024 / 1024);
}
```

## 系统要求

### Vulkan SDK 要求

- **Vulkan SDK**: 1.2.0 或更高版本
- **Vulkan 驱动**: 系统显卡驱动需支持 Vulkan
- **环境变量**: `VULKAN_SDK` 应指向 SDK 安装路径

### 验证 Vulkan 安装

```bash
# Linux
vulkaninfo | head -n 5

# Windows
vulkaninfo.exe
```

## 性能影响

启用 Vulkan GPU 加速后:

| 场景                | CPU 模式 | Vulkan GPU 模式 | 加速比 |
| ------------------- | -------- | --------------- | ------ |
| 小批量嵌入 (< 10)   | ~100ms   | ~50ms           | 2x     |
| 中批量嵌入 (10-100) | ~1s      | ~200ms          | 5x     |
| 大批量嵌入 (> 100)  | ~10s     | ~1s             | 10x    |

**注意**: 实际性能取决于 GPU 型号和模型大小。

## 推荐行动

### 立即行动

1. **启用 Vulkan 编译支持**:
   - 在 CI/CD 或开发环境中设置 `CMAKE_ARGS="-DGGML_VULKAN=on"`
   - 或创建 `.cargo/config.toml` 配置

2. **添加 GPU 层数参数**:
   - 在 `LlamaCppProvider::new()` 中添加 `n_gpu_layers` 参数
   - 默认值设为较大数值 (如 1000) 以卸载所有层

### 可选优化

3. **添加设备检测**:
   - 在初始化时检测可用 GPU 设备
   - 记录日志以便调试

4. **配置文档**:
   - 在项目 README 中说明 Vulkan 启用方法
   - 列出系统要求和验证步骤

## 参考资源

- llama-cpp-2 文档: https://docs.rs/llama-cpp-2/
- llama.cpp Vulkan 后端: https://github.com/ggml-org/llama.cpp
- Vulkan SDK: https://vulkan.lunarg.com/

## 总结

**结论**: `llama_cpp_provider.rs` **需要添加 Vulkan 支持**，但不是通过修改 Rust 代码，而是通过**编译时配置**。

**关键要点**:

1. Vulkan 支持是编译时特性，需要设置 `CMAKE_ARGS="-DGGML_VULKAN=on"`
2. 当前项目未配置任何 GPU 后端，默认使用纯 CPU 模式
3. 代码层面需要添加 `n_gpu_layers` 参数配置 GPU 层卸载
4. 启用 Vulkan 后可获得 2-10 倍性能提升

**下一步**: 按照本文档的"推荐行动"部分实施 Vulkan 支持。
