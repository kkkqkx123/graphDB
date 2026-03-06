# GraphDB C API 集成测试

本目录包含 GraphDB C API 的集成测试。

## 目录结构

```
tests/c_api/
├── tests.c              # C 测试源代码
├── CMakeLists.txt       # CMake 构建配置
├── build_msvc.ps1       # MSVC 构建脚本（推荐）
└── README.md            # 本文件
```

## 前置条件

1. **Rust 工具链**：用于编译 GraphDB 库
2. **MSVC 编译器**：Windows 平台推荐使用 Visual Studio 的 MSVC 工具链
3. **GraphDB 库已编译**：运行 `cargo build --lib` 生成库文件

## 构建方式

### 方式1: 使用 PowerShell 脚本（推荐，Windows）

```powershell
# 进入项目根目录
cd graphDB

# 构建并运行测试
.\tests\c_api\build_msvc.ps1 -Run

# 仅构建（debug 模式）
.\tests\c_api\build_msvc.ps1

# 构建 release 版本
.\tests\c_api\build_msvc.ps1 -BuildMode release

# 清理并重新构建
.\tests\c_api\build_msvc.ps1 -Clean -Run
```

### 方式2: 使用 CMake

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

### 方式3: 手动编译（MSVC）

```cmd
cl.exe /W4 /I../../include /Febuild\bin\graphdb_c_api_tests.exe tests.c /link /LIBPATH:../../target/debug graphdb.dll.lib ws2_32.lib
```

## 测试覆盖范围

### 数据库生命周期测试
- 数据库打开/关闭
- 库版本获取
- 空参数处理

### 会话管理测试
- 会话创建/销毁
- 自动提交模式
- 空参数处理

### 查询执行测试
- 简单查询执行
- 空参数处理

### 结果处理测试
- 结果集元数据（列数、行数）
- 空参数处理

### 事务管理测试
- 事务开始/提交
- 事务开始/回滚
- 空参数处理

### 预编译语句测试
- 语句准备/释放
- 空参数处理

### 批量操作测试
- 批量插入器创建/释放
- 空参数处理

### 错误处理测试
- 错误码转换
- 错误描述获取
- 错误消息获取

### 集成场景测试
- 完整工作流程

## 常见问题

### 1. 找不到 GraphDB 库

**解决**：先编译 GraphDB 项目
```bash
cargo build --lib
```

### 2. 运行时找不到 DLL

**解决**：将库目录添加到 PATH
```powershell
$env:PATH = "target\debug;$env:PATH"
```

### 3. 编译错误 C2016

**原因**：头文件中存在空结构体

**解决**：确保 `include/graphdb.h` 中的结构体定义包含 `_dummy` 成员

## 注意事项

- 测试使用 MSVC 工具链编译，确保 Visual Studio 环境变量已配置
- 测试程序会自动清理生成的测试数据库文件
- 所有测试用例均为独立测试，可单独运行
