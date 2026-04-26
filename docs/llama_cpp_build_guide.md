说明：在执行命令前，先执行& 'D:\softwares\Visual Studio\Common7\Tools\Launch-VsDevShell.ps1'; 以启用vs环境来执行编译

示例：& 'D:\softwares\Visual Studio\Common7\Tools\Launch-VsDevShell.ps1'; cargo clippy --all-targets --all-features 2>&1

# llama.cpp 构建指南

## 环境要求

### 必需工具

- **Rust**: 1.80+
- **CMake**: 3.30+（已安装在 Visual Studio 2022 中）
- **Visual Studio 2022**: 位于 `D:\softwares\Visual Studio`
- **Vulkan SDK**: 已检测到（支持 GPU 加速）

### Visual Studio 环境配置

#### 方案一：使用 Visual Studio Developer PowerShell（推荐）

1. 打开开始菜单
2. 搜索 "Visual Studio 2022"
3. 选择 "Developer PowerShell for VS 2022"（64位）

在该环境中，cmake 和编译器环境已自动配置好。

#### 方案二：手动配置环境变量

在普通 PowerShell 中运行：

```powershell
# 配置 Visual Studio 环境（64位）
& "D:\softwares\Visual Studio\VC\Auxiliary\Build\vcvars64.bat"
```

#### 方案三：使用 cargo-vcvars（自动化工具）

```powershell
# 安装 cargo-vcvars
cargo install cargo-vcvars

# 使用 cargo-vcvars 自动配置环境
cargo vcvars exec -- cargo build --features llama_cpp
```

## 构建步骤

### 1. 清理之前的构建缓存（可选）

```powershell
cargo clean
```

### 2. 启用 llama_cpp feature 构建

```powershell
# Debug 构建
cargo build --features llama_cpp

# Release 构建（推荐用于生产环境）
cargo build --features llama_cpp --release
```

### 3. 运行代码质量检查

```powershell
# Clippy 检查
cargo clippy --features llama_cpp --all-targets

# 格式化代码
cargo fmt
```

### 4. 运行测试

```powershell
# 运行所有测试
cargo test --features llama_cpp

# 运行特定模块的测试
cargo test --features llama_cpp --package code-context-engine --lib embedding::llama_cpp_provider
```

## 构建配置说明

### Cargo.toml 配置

```toml
[dependencies]
llama-cpp-2 = { version = "0.1", optional = true }

[features]
default = []
llama_cpp = ["llama-cpp-2"]
```

### 构建参数

llama.cpp 构建脚本使用以下 CMake 参数：

```cmake
-DLLAMA_BUILD_TESTS=OFF
-DLLAMA_BUILD_EXAMPLES=OFF
-DLLAMA_BUILD_SERVER=OFF
-DLLAMA_BUILD_TOOLS=OFF
-DLLAMA_BUILD_COMMON=ON
-DLLAMA_CURL=OFF
-DCMAKE_BUILD_PARALLEL_LEVEL=16
-DGGML_NATIVE=OFF
-DBUILD_SHARED_LIBS=OFF
-DGGML_OPENMP=ON
```

### GPU 支持

- **Vulkan SDK**: 自动检测并启用
- **GPU 层数**: 通过 `n_gpu_layers` 参数控制（默认：0，CPU-only）
- **KV Cache 卸载**: 通过 `offload_kqv` 参数控制（默认：true）

## 使用示例

### 创建 LlamaCppProvider

```rust
use code_context_engine::embedding::LlamaCppProvider;

let provider = LlamaCppProvider::new(
    "/path/to/model.gguf".to_string(),
    "mean".to_string(),  // pooling type: cls, mean, last, rank, none
    Some(768),           // embedding dimension
    Some(4096),          // context window size
    Some(8),             // number of threads
    Some(16),            // number of threads for batch
    Some(512),           // physical batch size
    Some(256),           // micro-batch size
    Some(true),          // offload K, Q, V to GPU
    Some(1000),          // number of GPU layers
)?;
```

### 生成嵌入向量

```rust
let texts = vec!["Hello, world!", "This is a test."];
let embeddings = provider.embed(&texts).await?;
```

## 故障排除

### 错误：cmake not found

**原因**: 未在 Visual Studio Developer PowerShell 环境中运行

**解决方案**:
1. 使用 "Developer PowerShell for VS 2022"
2. 或手动运行：`& "D:\softwares\Visual Studio\VC\Auxiliary\Build\vcvars64.bat"`

### 错误：Vulkan SDK not found

**原因**: Vulkan SDK 未安装或未在 PATH 中

**解决方案**:
- Vulkan SDK 应位于 `E:\softwares\vulkan\Bin`
- 确保 PATH 包含该路径

### 错误：OpenMP not found

**原因**: Visual Studio 未安装 C++ 工作负载

**解决方案**:
1. 运行 Visual Studio Installer
2. 修改安装，添加 "Desktop development with C++" 工作负载

### 构建时间过长

**原因**: 首次构建需要编译 llama.cpp C++ 代码

**预期时间**: 10-30 分钟（取决于硬件）

**优化**:
- 使用 `-DCMAKE_BUILD_PARALLEL_LEVEL` 增加并行度
- 使用 Release 模式构建：`--release`

## 性能优化建议

### GPU 加速

```rust
// 启用 GPU 层卸载（推荐）
let provider = LlamaCppProvider::new(
    model_path,
    "mean".to_string(),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(true),   // offload_kqv
    Some(1000),   // n_gpu_layers（根据 GPU 内存调整）
)?;
```

### CPU 模式

```rust
// CPU-only 模式（适用于无 GPU 环境）
let provider = LlamaCppProvider::new(
    model_path,
    "mean".to_string(),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(false),  // offload_kqv
    Some(0),      // n_gpu_layers
)?;
```

### 批处理优化

```rust
let provider = LlamaCppProvider::new(
    model_path,
    "mean".to_string(),
    None,
    None,
    Some(8),      // n_threads
    Some(16),     // n_threads_batch
    Some(512),    // n_batch
    Some(256),    // n_ubatch
    None,
    None,
)?;
```

## 相关文件

- `src/embedding/llama_cpp_provider.rs` - llama.cpp 嵌入提供者实现
- `src/embedding/config.rs` - 嵌入配置管理
- `src/llm/config.rs` - LLM 配置管理
- `Cargo.toml` - 项目依赖配置

## 参考资料

- [llama.cpp 官方文档](https://github.com/ggerganov/llama.cpp)
- [llama-cpp-2 Rust 绑定](https://crates.io/crates/llama-cpp-2)
- [Vulkan SDK](https://vulkan.lunarg.com/)
