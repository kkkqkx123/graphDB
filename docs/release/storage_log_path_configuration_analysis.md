# 存储路径与日志路径配置分析

## 概述

本文档分析 graphDB 项目中存储路径和日志路径的配置方式，以及日志大小限制的具体语义。

---

## 1. 存储路径配置

### 1.1 配置结构

存储路径在 [`DatabaseConfig`](../../src/config/mod.rs#L8-L20) 结构体中定义：

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 存储路径
    pub storage_path: String,
    /// 最大连接数
    pub max_connections: usize,
    /// 事务超时时间（秒）
    pub transaction_timeout: u64,
}
```

### 1.2 默认值

```rust
impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9758,
            storage_path: "data/graphdb".to_string(),  // 默认相对路径
            max_connections: 10,
            transaction_timeout: 30,
        }
    }
}
```

### 1.3 路径解析逻辑

存储路径通过 [`resolve_storage_path`](../../src/config/mod.rs#L203-L238) 方法解析：

| 路径类型 | 处理方式 |
|---------|---------|
| 绝对路径 | 直接使用 |
| 以 `~` 开头 | 展开为用户主目录（如 `~/data` → `/home/user/data`） |
| 相对路径 | 基于可执行文件所在目录解析为绝对路径 |

### 1.4 配置文件示例

```toml
[database]
host = "127.0.0.1"
port = 9758
storage_path = "data/graphdb"
max_connections = 10
transaction_timeout = 30
```

---

## 2. 日志路径配置

### 2.1 配置结构

日志配置在 [`LogConfig`](../../src/config/mod.rs#L40-L55) 结构体中定义：

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogConfig {
    /// 日志级别
    pub level: String,
    /// 日志目录
    pub dir: String,
    /// 日志文件名
    pub file: String,
    /// 单个日志文件最大大小（字节）
    pub max_file_size: u64,
    /// 最大日志文件数量
    pub max_files: usize,
}
```

### 2.2 默认值

```rust
impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            dir: "logs".to_string(),           // 默认日志目录
            file: "graphdb".to_string(),       // 默认日志文件名
            max_file_size: 100 * 1024 * 1024,  // 100MB
            max_files: 5,
        }
    }
}
```

### 2.3 配置文件示例

```toml
[log]
level = "info"
dir = "logs"
file = "graphdb"
max_file_size = 104857600  # 100MB
max_files = 5
```

### 2.4 日志初始化

日志系统通过 [`logging::init`](../../src/utils/logging.rs#L25-L52) 初始化：

```rust
pub fn init(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let handle = Logger::try_with_str(&config.log.level)?
        .log_to_file(
            FileSpec::default()
                .basename(&config.log.file)
                .directory(&config.log.dir),
        )
        .rotate(
            Criterion::Size(config.log.max_file_size),
            Naming::Numbers,
            Cleanup::KeepLogFiles(config.log.max_files),
        )
        .write_mode(WriteMode::Async)
        .append()
        .start()?;
    // ...
}
```

---

## 3. 日志大小限制分析

### 3.1 关键结论

**`max_file_size` 设置的是单个日志文件的大小限制，而非所有日志文件的总大小限制。**

### 3.2 证据说明

#### 3.2.1 代码注释

[`LogConfig`](../../src/config/mod.rs#L50) 结构体中明确注释：

```rust
/// 单个日志文件最大大小（字节）
pub max_file_size: u64,
```

#### 3.2.2 日志轮转配置

在 [`logging.rs`](../../src/utils/logging.rs#L37-L41) 中，使用 `flexi_logger` 的轮转机制：

```rust
.rotate(
    Criterion::Size(config.log.max_file_size),   // 按大小触发轮转
    Naming::Numbers,                              // 数字命名方式
    Cleanup::KeepLogFiles(config.log.max_files),  // 保留文件数量
)
```

#### 3.2.3 flexi_logger 行为

| 配置项 | 作用 |
|-------|------|
| `Criterion::Size(max_file_size)` | 当单个日志文件达到指定大小时触发轮转 |
| `Naming::Numbers` | 轮转文件使用数字命名（如 `graphdb.1.log`, `graphdb.2.log`） |
| `Cleanup::KeepLogFiles(max_files)` | 最多保留指定数量的日志文件，旧文件自动删除 |

### 3.3 日志轮转机制

当当前日志文件写满 `max_file_size` 时，触发轮转：

1. 当前文件 `graphdb.log` 重命名为 `graphdb.1.log`
2. 创建新的 `graphdb.log` 继续写入
3. 如果文件数量超过 `max_files`，最旧的文件被删除

**示例轮转过程**（假设 `max_files = 5`）：

```
初始状态:
  graphdb.log (写入中)

写满 100MB 后:
  graphdb.log → graphdb.1.log
  graphdb.log (新建，继续写入)

多次轮转后:
  graphdb.log   (当前写入)
  graphdb.1.log (最新归档)
  graphdb.2.log
  graphdb.3.log
  graphdb.4.log
  graphdb.5.log (最旧归档)

再次轮转时:
  graphdb.5.log 被删除
  graphdb.4.log → graphdb.5.log
  ...
  graphdb.log → graphdb.1.log
  graphdb.log (新建)
```

### 3.4 存储空间计算

| 配置项 | 默认值 | 说明 |
|-------|--------|------|
| `max_file_size` | 100MB | 单个日志文件最大大小 |
| `max_files` | 5 | 最大日志文件数量（包含当前写入文件） |

**理论最大占用空间** = `max_file_size` × `max_files` = 100MB × 5 = **500MB**

**实际占用空间** = 当前写入文件大小 + 已归档文件大小 ≤ 500MB

---

## 4. 配置建议

### 4.1 存储路径建议

- **开发环境**：使用默认相对路径 `data/graphdb`
- **生产环境**：建议使用绝对路径，如 `/var/lib/graphdb/data`
- **多实例部署**：每个实例使用独立的存储路径

### 4.2 日志配置建议

| 场景 | max_file_size | max_files | 理论上限 |
|-----|---------------|-----------|---------|
| 开发/测试 | 10MB | 3 | 30MB |
| 小型生产环境 | 50MB | 5 | 250MB |
| 中型生产环境 | 100MB | 10 | 1GB |
| 大型生产环境 | 200MB | 20 | 4GB |

### 4.3 配置示例（生产环境）

```toml
[database]
host = "0.0.0.0"
port = 9758
storage_path = "/var/lib/graphdb/data"
max_connections = 100
transaction_timeout = 60

[log]
level = "warn"           # 生产环境建议 warn 级别
dir = "/var/log/graphdb"
file = "graphdb"
max_file_size = 104857600  # 100MB
max_files = 10
```

---

## 5. 相关代码文件

| 文件 | 说明 |
|-----|------|
| [src/config/mod.rs](../../src/config/mod.rs) | 配置结构体定义和路径解析 |
| [src/utils/logging.rs](../../src/utils/logging.rs) | 日志系统初始化 |
| [config.toml](../../config.toml) | 默认配置文件 |
| [tests/integration_logging.rs](../../tests/integration_logging.rs) | 日志集成测试 |

---

## 6. 总结

1. **存储路径**：支持相对路径、绝对路径和 `~` 展开，相对路径基于可执行文件目录解析
2. **日志路径**：由目录 `dir` 和文件名 `file` 组成，默认分别为 `logs` 和 `graphdb`
3. **日志大小限制**：`max_file_size` 是单个文件大小限制，配合 `max_files` 控制总日志量
4. **日志轮转**：使用 `flexi_logger` 的按大小轮转机制，自动清理旧日志文件
