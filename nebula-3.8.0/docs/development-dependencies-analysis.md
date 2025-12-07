# NebulaGraph 3.8.0 开发依赖分析

## 项目概述
NebulaGraph 是一个分布式、可扩展、快速的图数据库。本文档分析了构建和贡献 NebulaGraph 3.8.0 所需的开发依赖。

## 构建系统及核心依赖

### 主要构建系统
- **CMake** (最低版本 3.9.0)：项目的主构建系统
- **C++ 编译器**：GCC 或 Clang，支持 C++11 ABI
- **Ninja 或 Make**：在 CMake 配置后执行构建

### 核心开发依赖

#### 系统要求
- Linux 操作系统（构建脚本专门检查 Linux）
- GLIBC 版本兼容性（测试支持从 2.17 到 2.34 的版本）

#### 必需的 C++ 库（通过第三方系统）
根据 `cmake/nebula/ThirdPartyConfig.cmake` 文件，需要以下库：

1. **Google 库**
   - Gflags：命令行标志处理
   - Glog：日志库
   - Googletest：测试框架

2. **Meta/Facebook 库**
   - Folly：Facebook 开发的 C++ 库
   - Fbthrift：Facebook 的 Thrift RPC 实现
   - Wangle：基于 folly 的客户端/服务器框架
   - Fizz：基于 folly 的 TLS 实现
   - Sodium：加密库
   - Proxygen：HTTP 框架
   - Fatal：元编程库

3. **数据处理库**
   - RocksDB：嵌入式键值存储
   - DoubleConversion：快速十进制字符串转换
   - Snappy：快速压缩/解压缩
   - Zstd：压缩算法
   - Bzip2：压缩库
   - LZMA：压缩库

4. **系统库**
   - Libevent：事件通知库
   - Jemalloc：内存分配器（可选）
   - Libunwind：确定程序调用链的库
   - OpenSSL：SSL/TLS 库
   - Boost：C++ 库集合
   - ZLIB：压缩库
   - Breakpad：崩溃报告库（可选）

5. **构建工具**
   - Bison（最低版本 3.0.5）：解析器生成器
   - Flex：扫描器生成器

#### 构建配置依赖
- **开发工具**：`build-essential`, `cmake`, `ninja-build`
- **系统库**：`libbz2-dev`, `libssl-dev`, `libsnappy-dev`, `libdouble-conversion-dev` 等
- **编译器工具**：`clang-format` 用于代码格式化

## 依赖管理方式

### 第三方安装
项目使用预构建的第三方依赖系统：
- 预编译依赖从 `https://oss-cdn.nebula-graph.com.cn/third-party/` 下载
- 当前使用版本 3.3 的第三方包
- 根据系统 GLIBC 和 GCC 版本自动检测和安装
- `third-party/install-third-party.sh` 脚本管理此过程

### CMake 配置
- 系统按以下顺序查找第三方依赖：
  1. CMake 参数 `-DNEBULA_THIRDPARTY_ROOT=path`
  2. `${CMAKE_BINARY_DIR}/third-party/install`（如果存在）
  3. 环境变量 `NEBULA_THIRDPARTY_ROOT=path`
  4. `/opt/vesoft/third-party`（如果存在）
  5. 自动下载并安装到 `${CMAKE_BINARY_DIR}/third-party/install`

## 开发环境设置

### 手动安装方式
1. 安装系统依赖：
   ```bash
   # Ubuntu/Debian 示例
   sudo apt-get update
   sudo apt-get install build-essential cmake git wget tar bzip2
   ```

2. 使用第三方脚本安装 C++ 构建依赖：
   ```bash
   cd third-party
   ./install-third-party.sh --prefix=/opt/vesoft/third-party/3.3
   ```

3. 构建项目：
   ```bash
   mkdir build
   cd build
   cmake -DENABLE_JEMALLOC=ON -DENABLE_ASAN=OFF ..
   make -j$(nproc)
   ```

### 基于 Docker 的开发
- 项目提供 Dockerfiles，使用 `vesoft/nebula-dev:centos7` 作为基础开发镜像
- 其中包含所有必要的构建工具和预安装的依赖
- 推荐用于一致性开发环境的方法

## 开发特定依赖

### 测试依赖
- Googletest/GMock 用于单元测试
- 通过 CMake 中的 `-DENABLE_TESTING=ON` 启用

### 可选功能依赖
- **Jemalloc**：通过 `-DENABLE_JEMALLOC=ON` 启用
- **地址消毒器**：通过 `-DENABLE_ASAN=ON` 启用
- **Breakpad**：通过 `-DENABLE_BREAKPAD=ON` 启用（需要调试信息）

### 开发工具
- **Clang Format**：用于代码格式一致性
- **CMake**：用于构建配置
- **Git**：用于版本控制
- **wget/curl**：用于下载依赖

## 构建过程依赖

完整构建过程需要：
1. C++14 兼容的编译器（GCC 7+ 或 Clang）
2. 正确定位的第三方依赖
3. 适当的系统资源（内存、磁盘空间）
4. 构建工具（make 或 ninja）

## 平台支持
- 仅支持 Linux（安装脚本明确检查 Linux）
- 主要支持 x86_64 架构
- ARM64 支持可能有限（非 x86_64 上禁用 Breakpad）

## 常见依赖问题排查

1. **缺少第三方依赖**：构建系统将尝试自动下载
2. **编译器 ABI 不匹配**：使用正确的 C++11 ABI 设置（开/关）
3. **GLIBC 版本兼容性**：系统自动选择兼容的预构建依赖
4. **缺少系统包**：安装上述提到的基础开发包

## 结论

NebulaGraph 3.8.0 有一个全面的依赖管理系统，严重依赖预构建的第三方库。项目使用 CMake 作为构建系统，需要带有特定系统库的 Linux 环境。推荐的方法是使用基于 Docker 的开发环境或让自动化脚本处理第三方依赖安装。