# GraphDB C API 集成测试

本目录包含 GraphDB C API 的集成测试，分为 Rust 测试和 C 测试两部分。

## 目录结构

```
tests/c_api/
├── tests.c              # C 测试源代码
├── CMakeLists.txt       # CMake 构建配置
├── build.ps1            # PowerShell 构建脚本
└── README.md            # 本文件
```

## 测试类型

### 1. Rust 集成测试

**文件位置**: `tests/integration_c_api.rs`

**特点**:
- 直接调用 Rust 的 `extern "C"` 函数
- 使用 Rust 测试框架
- 测试运行速度快
- 易于调试

**运行方式**:
```bash
# 运行所有 C API 集成测试
cargo test --test integration_c_api

# 运行特定测试
cargo test --test integration_c_api test_c_api_database_open_close

# 显示测试输出
cargo test --test integration_c_api -- --nocapture

# 运行测试并显示详细信息
cargo test --test integration_c_api -- --show-output
```

### 2. C 集成测试

**文件位置**: `tests/c_api/tests.c`

**特点**:
- 完全模拟真实的 C 调用环境
- 测试 C 编译器的兼容性
- 验证头文件的正确性
- 更接近实际使用场景

**构建方式**:

#### 方式1: 使用 PowerShell 脚本 (推荐)

```powershell
# 进入测试目录
cd tests/c_api

# 构建测试（debug 模式）
.\build.ps1

# 构建测试（release 模式）
.\build.ps1 -BuildMode release

# 构建并运行测试
.\build.ps1 -Run

# 清理并重新构建
.\build.ps1 -Clean -Run
```

#### 方式2: 使用 CMake

```bash
# 进入测试目录
cd tests/c_api

# 创建构建目录
mkdir build
cd build

# 配置 CMake
cmake ..

# 构建
cmake --build .

# 运行测试
ctest --verbose
```

#### 方式3: 手动编译

**Windows (MSVC)**:
```cmd
cl.exe /W4 /I../../include /Febuild\bin\graphdb_c_api_tests.exe tests.c /link /LIBPATH:../../target/debug graphdb.lib ws2_32.lib
```

**Windows (MinGW)**:
```bash
gcc -Wall -Wextra -I../../include -L../../target/debug -o build/bin/graphdb_c_api_tests.exe tests.c -lgraphdb -lws2_32
```

**Linux/macOS**:
```bash
gcc -Wall -Wextra -I../../include -L../../target/debug -o build/bin/graphdb_c_api_tests tests.c -lgraphdb -lpthread -ldl -lm
```

## 测试覆盖范围

### 数据库生命周期测试
- ✅ 数据库打开/关闭
- ✅ 库版本获取
- ✅ 空参数处理
- ✅ 多个数据库实例

### 会话管理测试
- ✅ 会话创建/销毁
- ✅ 自动提交模式
- ✅ 空参数处理
- ✅ 多个会话实例

### 查询执行测试
- ✅ 简单查询执行
- ✅ 参数化查询
- ✅ 空参数处理

### 结果处理测试
- ✅ 结果集元数据（列数、行数）
- ✅ 列名获取
- ✅ 值获取（整数、字符串）
- ✅ 空参数处理

### 事务管理测试
- ✅ 事务开始/提交
- ✅ 事务开始/回滚
- ✅ 只读事务
- ✅ 保存点管理
- ✅ 空参数处理

### 预编译语句测试
- ✅ 语句准备/释放
- ✅ 参数绑定（NULL、布尔、整数、浮点、字符串）
- ✅ 按名称绑定
- ✅ 语句重置
- ✅ 空参数处理

### 批量操作测试
- ✅ 批量插入器创建/释放
- ✅ 添加顶点/边
- ✅ 批量执行
- ✅ 缓冲计数
- ✅ 空参数处理

### 错误处理测试
- ✅ 错误码转换
- ✅ 错误描述获取
- ✅ 错误消息获取

### 集成场景测试
- ✅ 完整工作流程
- ✅ 并发会话

## 前置条件

### Rust 测试
- Rust 工具链已安装
- GraphDB 项目已编译

### C 测试
- C 编译器已安装（GCC、Clang 或 MSVC）
- GraphDB 库已编译
- C 头文件已生成（`include/graphdb.h`）

## 常见问题

### 1. 找不到 GraphDB 库

**问题**: 链接时提示找不到 graphdb.lib 或 libgraphdb.a

**解决**:
```bash
# 先编译 GraphDB 项目
cargo build --debug
# 或
cargo build --release
```

### 2. 找不到头文件

**问题**: 编译时提示找不到 graphdb.h

**解决**: 确保包含路径正确：
```bash
-I../../include
```

### 3. 运行时找不到库

**问题**: Windows 下运行测试时提示找不到 graphdb.dll

**解决**: 将库目录添加到 PATH：
```powershell
$env:PATH = "target\debug;$env:PATH"
```

### 4. 测试失败

**问题**: 某些测试失败

**解决**:
1. 检查 GraphDB 库是否正确编译
2. 清理测试文件后重新运行
3. 使用 `--nocapture` 查看详细输出
4. 检查是否有权限问题

## 持续集成

### GitHub Actions 示例

```yaml
name: C API Tests

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build
        run: cargo build --release
      - name: Run Rust tests
        run: cargo test --test integration_c_api --release

  c-tests:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v2
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build library
        run: cargo build --release
      - name: Build C tests
        run: |
          cd tests/c_api
          if [ "$RUNNER_OS" == "Windows" ]; then
            powershell -File build.ps1 -BuildMode release
          else
            mkdir -p build && cd build
            cmake .. && cmake --build .
          fi
      - name: Run C tests
        run: |
          cd tests/c_api
          if [ "$RUNNER_OS" == "Windows" ]; then
            ./build/bin/graphdb_c_api_tests.exe
          else
            ./build/graphdb_c_api_tests
          fi
```

## 贡献指南

添加新测试时：

1. 在 `tests/integration_c_api.rs` 中添加 Rust 测试
2. 在 `tests/c_api/tests.c` 中添加对应的 C 测试
3. 确保两种测试覆盖相同的功能
4. 更新本文档的测试覆盖范围
5. 运行所有测试确保通过

## 许可证

与 GraphDB 项目保持一致。
