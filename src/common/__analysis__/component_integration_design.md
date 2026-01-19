# 组件集成设计分析

## 概述

本文档分析 `fs.rs`、`network.rs`、`charset.rs` 和 `time.rs` 四个组件，对照 nebula-graph 的实现，提出集成方案和设计建议。

---

## 1. fs.rs - 文件系统工具

### Nebula-Graph 实现

**位置**: `nebula-3.8.0/src/common/fs/FileUtils.h`

**核心功能**:
- 文件类型检测（普通文件、目录、符号链接等）
- 文件大小、修改时间获取
- 路径操作（dirname、basename、joinPath、dividePath）
- 文件/目录操作（remove、rename、makeDir）
- 目录遍历（listAllFilesInDir、listAllDirsInDir）
- 文件迭代器（FileLineIterator、DirEntryIterator）
- 磁盘空间查询（free、available）

**设计特点**:
- 静态工具类，所有方法都是静态的
- 使用 `StatusOr<T>` 模式进行错误处理
- 支持递归删除
- 支持通配符模式匹配
- 提供迭代器接口用于遍历

### 当前 Rust 实现

**位置**: `src/common/fs.rs`

**已实现功能**:
- `FileLock` - 文件锁定（独占/共享）
- `acquire_exclusive()` / `acquire_shared()` - 获取锁
- `try_lock()` - 尝试获取锁
- `release()` - 释放锁
- `is_locked()` - 检查锁状态

**差异分析**:
1. **功能范围不同**:
   - Nebula: 全面的文件系统工具类
   - Rust: 专注于文件锁定功能

2. **使用场景不同**:
   - Nebula: 用于数据文件管理、日志文件操作、配置文件读写
   - Rust: 用于并发访问控制

### 集成方案

#### 方案 A：扩展为完整的文件工具类（推荐）

**适用场景**: 需要全面的文件操作功能

**设计**:
```rust
pub struct FileUtils;

impl FileUtils {
    // 路径操作
    pub fn dirname(path: &str) -> String;
    pub fn basename(path: &str) -> String;
    pub fn join_path(dir: &str, filename: &str) -> String;
    pub fn divide_path(path: &str) -> (String, String);
    
    // 文件信息
    pub fn file_size(path: &str) -> Result<u64, FsError>;
    pub fn file_type(path: &str) -> Result<FileType, FsError>;
    pub fn file_last_update_time(path: &str) -> Result<SystemTime, FsError>;
    
    // 文件操作
    pub fn remove(path: &str, recursively: bool) -> Result<(), FsError>;
    pub fn make_dir(dir: &str) -> Result<(), FsError>;
    pub fn rename(src: &str, dst: &str) -> Result<(), FsError>;
    pub fn exists(path: &str) -> bool;
    
    // 目录遍历
    pub fn list_files(dir: &str, pattern: Option<&str>) -> Result<Vec<String>, FsError>;
    pub fn list_dirs(dir: &str, pattern: Option<&str>) -> Result<Vec<String>, FsError>;
    
    // 磁盘空间
    pub fn free_space(path: &str) -> Result<u64, FsError>;
    pub fn available_space(path: &str) -> Result<u64, FsError>;
    
    // 迭代器
    pub fn file_line_iterator(path: &str) -> FileLineIterator;
    pub fn dir_entry_iterator(path: &str) -> DirEntryIterator;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Unknown,
    NotExist,
    Regular,
    Directory,
    SymLink,
    CharDev,
    BlockDev,
    Fifo,
    Socket,
}
```

**集成点**:
1. **存储引擎** (`src/storage/`):
   - 数据文件管理
   - WAL 日志文件操作
   - 快照文件处理

2. **配置管理** (`src/config/`):
   - 配置文件读写
   - 配置目录创建

3. **日志系统** (`src/common/log.rs`):
   - 日志文件轮转
   - 日志目录管理

**优势**:
- 提供统一的文件操作接口
- 减少重复代码
- 便于测试和维护

**劣势**:
- 需要大量开发工作
- 可能与 Rust 标准库功能重复

#### 方案 B：保持专注，仅用于并发控制

**适用场景**: 文件锁定是主要需求

**设计**:
保持当前的 `FileLock` 实现，在需要的地方使用：

```rust
// 在存储引擎中使用
impl StorageEngine for RocksDBStorage {
    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError> {
        // 获取文件锁防止并发修改
        let _lock = FileLock::acquire_exclusive(&self.db_path)?;
        // ...
    }
}
```

**集成点**:
1. **存储引擎**: 防止并发写入
2. **索引管理**: 保护索引文件
3. **元数据管理**: 防止并发修改

**优势**:
- 实现简单
- 专注核心功能
- 易于维护

**劣势**:
- 功能有限
- 其他文件操作需要使用标准库

### 推荐方案

**推荐方案 B**，原因：
1. Rust 标准库 (`std::fs`) 已经提供了完善的文件操作
2. 避免重复造轮子
3. `FileLock` 是独特的功能，标准库没有提供
4. 减少维护成本

---

## 2. network.rs - 网络工具

### Nebula-Graph 实现

**位置**: `nebula-3.8.0/src/common/network/NetworkUtils.h`

**核心功能**:
- 主机名获取
- IPv4 地址获取（从设备、列表）
- 端口管理（动态端口范围、端口使用情况、可用端口）
- 主机解析（DNS 解析）
- IP 地址转换（整数 <-> 字符串）
- Peer 字符串解析

**设计特点**:
- 静态工具类
- 使用 `StatusOr<T>` 模式
- 支持网络设备枚举
- 提供测试用的端口获取功能

### 当前 Rust 实现

**位置**: `src/common/network.rs`

**已实现功能**:
- `ClientService` - 客户端服务
- `serve_client()` - 服务客户端连接
- `handle_request()` - 处理请求
- `parse_request()` - 解析请求
- `process_ping()` / `process_status()` / `process_query()` - 处理不同命令

**差异分析**:
1. **功能范围不同**:
   - Nebula: 网络工具类（IP、端口、DNS）
   - Rust: 客户端服务实现

2. **抽象层次不同**:
   - Nebula: 底层网络工具
   - Rust: 应用层服务

### 集成方案

#### 方案 A：实现完整的网络工具类（推荐）

**适用场景**: 需要底层网络功能

**设计**:
```rust
pub struct NetworkUtils;

impl NetworkUtils {
    // 主机信息
    pub fn get_hostname() -> Result<String, NetworkError>;
    
    // IPv4 地址
    pub fn get_ipv4_from_device(device: &str) -> Result<String, NetworkError>;
    pub fn list_ipv4s() -> Result<Vec<String>, NetworkError>;
    pub fn list_devices_and_ipv4s() -> Result<Vec<(String, String)>, NetworkError>;
    
    // 端口管理
    pub fn get_dynamic_port_range() -> (u16, u16);
    pub fn get_ports_in_use() -> HashSet<u16>;
    pub fn get_available_port() -> u16; // 仅用于测试
    
    // 主机解析
    pub fn resolve_host(host: &str, port: i32) -> Result<Vec<HostAddr>, NetworkError>;
    
    // IP 转换
    pub fn int_to_ipv4(ip: u32) -> String;
    pub fn ipv4_to_int(ip: &str) -> Result<u32, NetworkError>;
    
    // Peer 解析
    pub fn parse_peers(peers_str: &str) -> Result<Vec<HostAddr>, NetworkError>;
    pub fn peers_to_string(hosts: &[HostAddr]) -> String;
    
    // 验证
    pub fn validate_host_or_ip(host_or_ip: &str) -> Result<(), NetworkError>;
}

#[derive(Debug, Clone)]
pub struct HostAddr {
    pub host: String,
    pub port: u16,
}
```

**集成点**:
1. **API 服务** (`src/api/`):
   - 服务绑定地址选择
   - 集群配置
   - 端口冲突检测

2. **客户端** (`src/clients/`):
   - 连接地址解析
   - 负载均衡

3. **配置管理** (`src/config/`):
   - 网络配置验证
   - 自动发现

**优势**:
- 提供完整的网络工具
- 便于服务发现和配置
- 支持复杂的网络拓扑

**劣势**:
- 需要平台特定的代码（Unix/Windows）
- 需要处理网络接口枚举

#### 方案 B：保持客户端服务，添加网络工具

**适用场景**: 应用层服务是主要需求

**设计**:
保持当前的 `ClientService`，添加网络工具模块：

```rust
// 在 network.rs 中添加
pub mod utils {
    pub use super::NetworkUtils;
}

// 在 API 服务中使用
use crate::common::network::{ClientService, utils::NetworkUtils};

pub async fn start_service(config: Config) -> Result<()> {
    // 使用网络工具获取可用端口
    let port = if config.port == 0 {
        NetworkUtils::get_available_port()
    } else {
        config.port
    };
    
    // 解析主机地址
    let addr = NetworkUtils::resolve_host(&config.host, port)?;
    
    // 启动服务
    let service = ClientService::new(addr);
    service.serve().await?;
}
```

**集成点**:
1. **API 服务启动**: 地址解析和端口选择
2. **集群管理**: 节点发现和通信
3. **健康检查**: 网络连通性检测

**优势**:
- 保持现有服务实现
- 渐进式添加功能
- 易于测试

**劣势**:
- 功能分离不够清晰
- 可能造成命名冲突

### 推荐方案

**推荐方案 A**，原因：
1. 提供完整的网络工具集
2. 符合 nebula-graph 的设计理念
3. 便于后续功能扩展
4. 清晰的模块边界

---

## 3. charset.rs - 字符集工具

### Nebula-Graph 实现

**位置**: `nebula-3.8.0/src/common/charset/Charset.h`

**核心功能**:
- 字符集支持检查
- 排序规则（collation）检查
- 字符集和排序规则匹配验证
- 默认排序规则获取
- 字符集描述信息

**设计特点**:
- 单例模式（`instance()`）
- 使用 `Status` 模式进行错误处理
- 支持字符集描述信息
- 当前只支持 UTF-8

### 当前 Rust 实现

**位置**: `src/common/charset.rs`

**已实现功能**:
- `Encoding` 枚举（Utf8, Utf16, Latin1, Gbk, Big5, Utf8Bom）
- `CharsetUtils` 工具类
- `detect_encoding()` - 编码检测（BOM + 启发式）
- `decode_with_encoding()` - 解码
- `encode_with_encoding()` - 编码
- `convert_encoding()` - 编码转换
- 字符串操作（大小写、回文检测等）

**差异分析**:
1. **功能范围**:
   - Nebula: 字符集和排序规则管理
   - Rust: 编码检测和转换

2. **抽象层次**:
   - Nebula: 数据库层面的字符集配置
   - Rust: 数据层面的编码处理

### 集成方案

#### 方案 A：扩展为字符集管理器（推荐）

**适用场景**: 需要数据库级别的字符集支持

**设计**:
```rust
pub struct CharsetDesc {
    pub charset_name: String,
    pub default_collation: String,
    pub supported_collations: Vec<String>,
    pub description: String,
    pub max_char_length: i32,
}

pub struct CharsetManager {
    supported_charsets: HashSet<String>,
    supported_collations: HashSet<String>,
    charset_descriptions: HashMap<String, CharsetDesc>,
}

impl CharsetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            supported_charsets: HashSet::new(),
            supported_collations: HashSet::new(),
            charset_descriptions: HashMap::new(),
        };
        
        // 注册支持的字符集
        manager.register_charset(CharsetDesc {
            charset_name: "utf8".to_string(),
            default_collation: "utf8_bin".to_string(),
            supported_collations: vec!["utf8_bin".to_string()],
            description: "UTF-8 Unicode".to_string(),
            max_char_length: 4,
        });
        
        // 可以添加更多字符集
        // manager.register_charset(CharsetDesc { ... });
        
        manager
    }
    
    pub fn is_support_charset(&self, charset_name: &str) -> bool;
    pub fn is_support_collate(&self, collate_name: &str) -> bool;
    pub fn charset_and_collate_match(&self, charset_name: &str, collate_name: &str) -> bool;
    pub fn get_default_collation_by_charset(&self, charset_name: &str) -> Option<String>;
    pub fn get_charset_by_collation(&self, collate_name: &str) -> Option<String>;
    pub fn get_charset_desc(&self) -> &HashMap<String, CharsetDesc>;
}

// 保持现有的编码检测和转换工具
pub mod encoding {
    pub use super::CharsetUtils;
    pub use super::Encoding;
}
```

**集成点**:
1. **Schema 管理** (`src/api/service/schema_manager.rs`):
   - Tag/Edge 类型创建时指定字符集
   - 验证字符集和排序规则

2. **数据导入导出** (`src/storage/`):
   - 数据导入时的编码转换
   - 数据导出时的编码选择

3. **查询引擎** (`src/query/`):
   - 字符串比较时的排序规则
   - 字符串函数的字符集感知

**优势**:
- 完整的字符集管理
- 支持数据库级别的字符集配置
- 便于国际化支持

**劣势**:
- 需要设计字符集元数据存储
- 增加系统复杂度

#### 方案 B：保持编码工具，添加字符集验证

**适用场景**: 编码转换是主要需求

**设计**:
保持当前的 `CharsetUtils`，添加字符集验证：

```rust
impl CharsetUtils {
    // 现有的编码检测和转换方法...
    
    // 添加字符集验证
    pub fn is_supported_charset(charset_name: &str) -> bool {
        matches!(charset_name.to_lowercase().as_str(), 
            "utf8" | "utf-8" | "latin1" | "gbk" | "big5")
    }
    
    pub fn is_supported_collation(collation_name: &str) -> bool {
        matches!(collation_name.to_lowercase().as_str(),
            "utf8_bin" | "utf8_general_ci" | "binary")
    }
    
    pub fn validate_charset_and_collation(
        charset_name: &str, 
        collation_name: &str
    ) -> Result<(), String> {
        if !Self::is_supported_charset(charset_name) {
            return Err(format!("不支持的字符集: {}", charset_name));
        }
        if !Self::is_supported_collation(collation_name) {
            return Err(format!("不支持的排序规则: {}", collation_name));
        }
        Ok(())
    }
}

// 在 Schema 管理中使用
impl SchemaManager {
    pub fn create_tag(&mut self, name: &str, charset: &str) -> Result<(), SchemaError> {
        // 验证字符集
        CharsetUtils::validate_charset_and_collation(charset, "utf8_bin")?;
        // ...
    }
}
```

**集成点**:
1. **Schema 管理**: 字符集验证
2. **数据导入**: 编码检测和转换
3. **API 层**: 请求参数验证

**优势**:
- 实现简单
- 保持现有编码工具
- 易于扩展

**劣势**:
- 不支持数据库级别的字符集配置
- 字符集元数据管理有限

### 推荐方案

**推荐方案 A**，原因：
1. 符合数据库系统的设计理念
2. 提供完整的字符集管理
3. 便于后续支持更多字符集
4. 与 nebula-graph 的架构一致

---

## 4. time.rs - 时间工具

### Nebula-Graph 实现

**位置**: `nebula-3.8.0/src/common/datatypes/Date.h`

**核心功能**:
- `Date` 结构体（年、月、日）
- 日期算术运算（加减天数）
- 日期比较（==、<）
- 日期格式化（toString）
- 闰年判断
- 月份天数计算
- 与 `Duration` 的交互

**设计特点**:
- 值类型（POD）
- 支持日期算术
- 内置格式化和比较
- 使用 UTC 时间

### 当前 Rust 实现

**位置**: `src/common/time.rs`

**已实现功能**:
- `TimeUtils` 工具类
- `Date` 结构体（年、月、日）
- `Time` 结构体（时、分、秒、毫秒）
- `DateTime` 结构体（日期 + 时间）
- 日期验证（`is_valid_date()`）
- 时间戳转换
- 日期格式化
- 日期解析

**差异分析**:
1. **功能范围**:
   - Nebula: 专注于日期类型和算术
   - Rust: 包含时间、日期时间、格式化等

2. **设计理念**:
   - Nebula: 数据库内部使用的日期类型
   - Rust: 通用的工具类

### 集成方案

#### 方案 A：保持现有实现，优化性能（推荐）

**适用场景**: 当前实现已经满足需求

**设计**:
优化现有实现，添加数据库特定的功能：

```rust
// 保持现有的 Date、Time、DateTime 结构体
// 添加数据库特定的功能

impl Date {
    // 添加日期算术
    pub fn add_days(&self, days: i64) -> Date;
    pub fn sub_days(&self, days: i64) -> Date;
    
    // 添加与 Duration 的交互
    pub fn add_duration(&self, duration: &Duration) -> Date;
    pub fn sub_duration(&self, duration: &Duration) -> Date;
    
    // 添加序列化和反序列化
    pub fn to_bytes(&self) -> [u8; 3]; // 年(2) + 月(1)
    pub fn from_bytes(bytes: &[u8]) -> Date;
    
    // 添加数据库存储优化
    pub fn to_int(&self) -> i64; // 自纪元以来的天数
    pub fn from_int(days: i64) -> Date;
}

// 在存储引擎中使用
impl StorageEngine for RocksDBStorage {
    fn scan_by_date_range(&self, start: Date, end: Date) -> Result<Vec<Vertex>, StorageError> {
        // 使用优化的日期比较
        let start_int = start.to_int();
        let end_int = end.to_int();
        // ...
    }
}
```

**集成点**:
1. **存储引擎** (`src/storage/`):
   - 日期范围查询
   - 日期索引
   - 日期序列化

2. **查询引擎** (`src/query/`):
   - 日期函数实现
   - 日期比较优化
   - 日期字面量解析

3. **API 层** (`src/api/`):
   - 日期参数验证
   - 日期格式转换

**优势**:
- 保持现有实现
- 添加数据库特定优化
- 易于维护

**劣势**:
- 可能与标准库的 `chrono` 功能重复

#### 方案 B：使用 chrono 库，添加适配层

**适用场景**: 需要完整的时间日期功能

**设计**:
使用成熟的 `chrono` 库，添加适配层：

```rust
pub use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Duration, Utc};

// 添加数据库特定的适配
pub trait DateAdapter {
    fn to_db_bytes(&self) -> [u8; 3];
    fn from_db_bytes(bytes: &[u8]) -> Self;
    fn to_db_int(&self) -> i64;
    fn from_db_int(days: i64) -> Self;
}

impl DateAdapter for NaiveDate {
    fn to_db_bytes(&self) -> [u8; 3] {
        // 年(2) + 月(1)
        let year = self.year() as u16;
        let month = self.month() as u8;
        [
            (year >> 8) as u8,
            (year & 0xFF) as u8,
            month,
        ]
    }
    
    fn from_db_bytes(bytes: &[u8]) -> Self {
        let year = ((bytes[0] as u16) << 8) | (bytes[1] as u16);
        let month = bytes[2];
        Self::from_ymd_opt(year as i32, month as u32, 1).unwrap()
    }
    
    fn to_db_int(&self) -> i64 {
        // 计算自纪元以来的天数
        self.signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
            .num_days()
    }
    
    fn from_db_int(days: i64) -> Self {
        NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
            + Duration::days(days)
    }
}
```

**集成点**:
1. **所有模块**: 统一使用 chrono
2. **存储引擎**: 使用适配层进行序列化
3. **查询引擎**: 使用 chrono 的丰富功能

**优势**:
- 使用成熟、经过测试的库
- 丰富的功能集
- 社区支持和文档完善

**劣势**:
- 需要添加适配层
- 可能增加依赖

### 推荐方案

**推荐方案 A**，原因：
1. 当前实现已经满足需求
2. 避免引入新的依赖
3. 可以针对数据库场景优化
4. 保持代码简洁

---

## 集成优先级

### 高优先级（立即集成）

1. **fs.rs - FileLock**:
   - **原因**: 存储引擎需要并发控制
   - **集成点**: `src/storage/` 模块
   - **方案**: 方案 B（保持专注）

2. **charset.rs - 编码验证**:
   - **原因**: Schema 管理需要字符集验证
   - **集成点**: `src/api/service/schema_manager.rs`
   - **方案**: 方案 B（保持编码工具，添加验证）

### 中优先级（后续集成）

3. **network.rs - 网络工具**:
   - **原因**: API 服务需要网络配置
   - **集成点**: `src/api/mod.rs`
   - **方案**: 方案 A（完整网络工具）

4. **time.rs - 日期优化**:
   - **原因**: 查询引擎需要日期函数
   - **集成点**: `src/query/`
   - **方案**: 方案 A（优化现有实现）

---

## 实施计划

### 第一阶段（1-2 周）

1. **fs.rs 集成**:
   - 在 `RocksDBStorage` 中使用 `FileLock`
   - 在 `IndexManager` 中使用 `FileLock`
   - 编写并发访问测试

2. **charset.rs 集成**:
   - 在 `SchemaManager` 中添加字符集验证
   - 在数据导入时使用编码检测
   - 编写字符集测试

### 第二阶段（2-3 周）

3. **network.rs 集成**:
   - 实现 `NetworkUtils` 工具类
   - 在 API 服务启动时使用
   - 添加网络配置验证

4. **time.rs 优化**:
   - 添加日期算术运算
   - 优化日期序列化
   - 在查询引擎中使用

### 第三阶段（1-2 周）

5. **集成测试**:
   - 端到端集成测试
   - 性能测试
   - 文档更新

---

## 总结

| 组件 | Nebula-Graph 功能 | 当前 Rust 实现 | 推荐方案 | 优先级 |
|------|------------------|----------------|------------|--------|
| fs.rs | 完整文件工具类 | FileLock（文件锁定） | 方案 B（保持专注） | 高 |
| network.rs | 网络工具类 | ClientService（客户端服务） | 方案 A（完整工具） | 中 |
| charset.rs | 字符集管理器 | 编码检测和转换 | 方案 A（字符集管理器） | 高 |
| time.rs | 日期类型和算术 | 通用时间工具 | 方案 A（优化现有） | 中 |

### 关键设计原则

1. **渐进式集成**: 不一次性重写，逐步添加功能
2. **保持兼容**: 不破坏现有 API
3. **性能优先**: 针对数据库场景优化
4. **易于测试**: 每个阶段都有可测试的成果
5. **文档完善**: 及时更新文档和示例

### 风险和缓解

1. **平台兼容性**:
   - 风险：网络和文件操作的平台差异
   - 缓解：使用条件编译和抽象层

2. **性能影响**:
   - 风险：额外的抽象层可能影响性能
   - 缓解：基准测试和优化

3. **维护成本**:
   - 风险：增加的代码需要维护
   - 缓解：清晰的模块边界和文档

---

## 附录：代码示例

### fs.rs 集成示例

```rust
// 在存储引擎中使用 FileLock
use crate::common::fs::FileLock;

impl RocksDBStorage {
    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError> {
        // 获取独占锁
        let _lock = FileLock::acquire_exclusive(&self.lock_file_path)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        
        // 执行事务
        // ...
    }
}
```

### charset.rs 集成示例

```rust
// 在 Schema 管理中使用字符集验证
use crate::common::charset::CharsetUtils;

impl SchemaManager {
    pub fn create_tag(
        &mut self,
        name: &str,
        charset: &str,
        collation: &str,
    ) -> Result<(), SchemaError> {
        // 验证字符集和排序规则
        CharsetUtils::validate_charset_and_collation(charset, collation)
            .map_err(|e| SchemaError::InvalidCharset(e))?;
        
        // 创建 Tag
        // ...
    }
}
```

### network.rs 集成示例

```rust
// 在 API 服务启动时使用网络工具
use crate::common::network::NetworkUtils;

pub async fn start_service(config: Config) -> Result<()> {
    // 解析主机地址
    let addresses = NetworkUtils::resolve_host(&config.host, config.port)
        .map_err(|e| anyhow::anyhow!("地址解析失败: {}", e))?;
    
    // 选择地址
    let addr = addresses.first()
        .ok_or_else(|| anyhow::anyhow!("没有可用的地址"))?;
    
    // 启动服务
    let listener = TcpListener::bind(addr).await?;
    // ...
}
```

### time.rs 集成示例

```rust
// 在查询引擎中使用优化的日期操作
use crate::common::time::Date;

impl QueryExecutor {
    fn execute_date_range_query(
        &self,
        start: Date,
        end: Date,
    ) -> Result<Vec<Vertex>, QueryError> {
        // 使用优化的日期比较
        let start_int = start.to_int();
        let end_int = end.to_int();
        
        // 执行查询
        self.storage.scan_by_date_range(start_int, end_int)
    }
}
```

---

**文档版本**: 1.0  
**最后更新**: 2026-01-19  
**作者**: GraphDB Team
