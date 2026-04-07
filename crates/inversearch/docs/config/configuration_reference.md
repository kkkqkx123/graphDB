# Inversearch 服务配置项参考

## 完整配置示例

```toml
# Inversearch 服务配置文件
# 文档版本：1.0
# 最后更新：2026-04-07

# ============================================================================
# 服务器配置
# ============================================================================
[server]
# 服务监听主机
# 默认：0.0.0.0
# 环境变量：INVSEARCH_HOST
host = "0.0.0.0"

# 服务监听端口
# 范围：1-65535
# 默认：50051
# 环境变量：INVSEARCH_PORT
port = 50051

# 工作进程数
# 范围：1-32
# 默认：4
# 环境变量：INVSEARCH_WORKERS
workers = 4

# ============================================================================
# 索引配置
# ============================================================================
[index]
# 索引分辨率（精度）
# 范围：1-12
# 默认：9
# 说明：值越大，索引精度越高，但内存占用也越大
resolution = 9

# 分词模式
# 可选值：strict, forward, reverse, full, bidirectional
# 默认：strict
# 说明：控制索引和搜索时的分词策略
tokenize = "strict"

# 索引深度
# 范围：0-10
# 默认：0
# 说明：上下文搜索的深度，0 表示禁用
depth = 0

# 是否启用双向索引
# 默认：true
# 说明：启用后支持正向和反向搜索
bidirectional = true

# 是否启用快速更新
# 默认：false
# 说明：启用后索引更新更快，但可能影响搜索性能
fastupdate = false

# 键值存储大小（可选）
# 范围：1-1000000
# 默认：未设置（禁用）
# keystore = 10000

# ============================================================================
# 缓存配置
# ============================================================================
[cache]
# 是否启用缓存
# 默认：false
# 环境变量：CACHE_ENABLED
enabled = false

# 缓存大小（条目数）
# 范围：1-1000000
# 默认：1000
# 环境变量：CACHE_SIZE
size = 1000

# 缓存 TTL（秒）
# 范围：0-86400
# 默认：3600（1 小时）
# 环境变量：CACHE_TTL
ttl = 3600

# ============================================================================
# 存储配置
# ============================================================================
[storage]
# 是否启用存储
# 默认：true
# 环境变量：STORAGE_ENABLED
enabled = true

# 存储后端类型
# 可选值：cold_warm_cache, file, redis, wal
# 默认：cold_warm_cache
# 环境变量：STORAGE_BACKEND
backend = "cold_warm_cache"

# ============================================================================
# 文件存储配置（需要启用 store-file feature）
# ============================================================================
[storage.file]
# 基础路径
# 默认：./data
# 环境变量：STORAGE_FILE_BASE_PATH
base_path = "./data"

# 是否自动保存
# 默认：true
# 环境变量：STORAGE_FILE_AUTO_SAVE
auto_save = true

# 自动保存间隔（秒）
# 范围：1-86400
# 默认：60
# 环境变量：STORAGE_FILE_SAVE_INTERVAL_SECS
save_interval_secs = 60

# ============================================================================
# Redis 存储配置（需要启用 store-redis feature）
# ============================================================================
# [storage.redis]
# # Redis 连接 URL
# # 默认：redis://127.0.0.1:6379
# # 环境变量：STORAGE_REDIS_URL
# url = "redis://127.0.0.1:6379"

# # 连接池大小
# # 范围：1-100
# # 默认：10
# # 环境变量：STORAGE_REDIS_POOL_SIZE
# pool_size = 10

# ============================================================================
# WAL 存储配置（需要启用 store-wal feature）
# ============================================================================
# [storage.wal]
# # 基础路径
# # 默认：./wal
# # 环境变量：STORAGE_WAL_BASE_PATH
# base_path = "./wal"

# # 最大 WAL 大小（字节）
# # 范围：1048576-1073741824
# # 默认：104857600 (100MB)
# # 环境变量：STORAGE_WAL_MAX_WAL_SIZE
# max_wal_size = 104857600

# # 是否启用压缩
# # 默认：true
# # 环境变量：STORAGE_WAL_COMPRESSION
# compression = true

# # 快照间隔（操作次数）
# # 范围：100-100000
# # 默认：1000
# # 环境变量：STORAGE_WAL_SNAPSHOT_INTERVAL
# snapshot_interval = 1000

# ============================================================================
# 日志配置
# ============================================================================
[logging]
# 日志级别
# 可选值：trace, debug, info, warn, error
# 默认：info
# 环境变量：LOG_LEVEL
level = "info"

# 日志格式
# 可选值：json, text
# 默认：json
# 环境变量：LOG_FORMAT
format = "json"
```

---

## 配置项详细说明

### 服务器配置

#### `server.host`

服务监听的网络接口。

- **类型**: String
- **默认值**: `0.0.0.0`
- **环境变量**: `INVSEARCH_HOST`
- **示例**:
  - `0.0.0.0` - 监听所有网卡
  - `127.0.0.1` - 仅监听本地
  - `::` - IPv6 监听

#### `server.port`

服务监听的端口号。

- **类型**: Integer
- **范围**: 1-65535
- **默认值**: 50051
- **环境变量**: `INVSEARCH_PORT`
- **说明**: 确保端口未被其他服务占用

#### `server.workers`

工作进程数量。

- **类型**: Integer
- **范围**: 1-32
- **默认值**: 4
- **环境变量**: `INVSEARCH_WORKERS`
- **说明**: 
  - 值越大，并发处理能力越强
  - 建议设置为 CPU 核心数

---

### 索引配置

#### `index.resolution`

索引分辨率（精度）。

- **类型**: Integer
- **范围**: 1-12
- **默认值**: 9
- **说明**:
  - 值越大，索引精度越高，搜索越准确
  - 值越大，内存占用和索引构建时间也越大
  - 推荐值：8-10（平衡性能和精度）

#### `index.tokenize`

分词模式。

- **类型**: String
- **可选值**:
  - `strict`: 严格分词（默认）
  - `forward`: 正向分词
  - `reverse`: 反向分词
  - `full`: 完整分词
  - `bidirectional`: 双向分词
- **默认值**: `strict`
- **说明**:
  - `strict`: 精确匹配完整词汇
  - `forward`: 支持前缀匹配
  - `reverse`: 支持后缀匹配
  - `full`: 支持所有分词组合
  - `bidirectional`: 同时支持正向和反向

#### `index.depth`

上下文搜索深度。

- **类型**: Integer
- **范围**: 0-10
- **默认值**: 0
- **说明**:
  - 0 表示禁用上下文搜索
  - 值越大，搜索范围越广，但性能开销越大
  - 推荐值：0-3（根据需求调整）

#### `index.bidirectional`

是否启用双向索引。

- **类型**: Boolean
- **默认值**: true
- **说明**:
  - 启用后支持正向和反向搜索
  - 会增加索引大小和构建时间
  - 推荐启用（除非确定不需要反向搜索）

#### `index.fastupdate`

是否启用快速更新。

- **类型**: Boolean
- **默认值**: false
- **说明**:
  - 启用后索引更新更快
  - 但可能影响搜索性能
  - 适合频繁更新的场景

#### `index.keystore`

键值存储大小（可选）。

- **类型**: Integer
- **范围**: 1-1000000
- **默认值**: 未设置（禁用）
- **说明**:
  - 用于存储文档元数据
  - 值为 0 或 unset 表示禁用

---

### 缓存配置

#### `cache.enabled`

是否启用查询缓存。

- **类型**: Boolean
- **默认值**: false
- **环境变量**: `CACHE_ENABLED`
- **说明**:
  - 启用后相同查询会返回缓存结果
  - 适合读多写少的场景

#### `cache.size`

缓存大小（条目数）。

- **类型**: Integer
- **范围**: 1-1000000
- **默认值**: 1000
- **环境变量**: `CACHE_SIZE`
- **说明**:
  - 值越大，缓存命中率越高
  - 但内存占用也越大

#### `cache.ttl`

缓存生存时间（秒）。

- **类型**: Integer
- **范围**: 0-86400
- **默认值**: 3600（1 小时）
- **环境变量**: `CACHE_TTL`
- **说明**:
  - 0 表示永不过期
  - 值越小，缓存更新越频繁

---

### 存储配置

#### `storage.enabled`

是否启用持久化存储。

- **类型**: Boolean
- **默认值**: true
- **环境变量**: `STORAGE_ENABLED`
- **说明**:
  - 启用后数据会持久化保存
  - 禁用则数据仅保存在内存中

#### `storage.backend`

存储后端类型。

- **类型**: String
- **可选值**:
  - `cold_warm_cache`: 冷热双层缓存（默认，推荐）
  - `file`: 文件存储
  - `redis`: Redis 存储
  - `wal`: 预写日志存储
- **默认值**: `cold_warm_cache`
- **环境变量**: `STORAGE_BACKEND`
- **说明**:
  - `cold_warm_cache`: 结合内存和文件存储，性能最佳
  - `file`: 简单的文件存储，适合单机部署
  - `redis`: 分布式存储，适合多实例部署
  - `wal`: 高可靠性存储，适合关键数据

---

### 文件存储配置

**注意**: 需要启用 `store-file` feature

#### `storage.file.base_path`

文件存储基础路径。

- **类型**: String
- **默认值**: `./data`
- **环境变量**: `STORAGE_FILE_BASE_PATH`

#### `storage.file.auto_save`

是否自动保存。

- **类型**: Boolean
- **默认值**: true
- **环境变量**: `STORAGE_FILE_AUTO_SAVE`
- **说明**:
  - 启用后定期自动保存数据
  - 禁用则需手动调用保存接口

#### `storage.file.save_interval_secs`

自动保存间隔（秒）。

- **类型**: Integer
- **范围**: 1-86400
- **默认值**: 60
- **环境变量**: `STORAGE_FILE_SAVE_INTERVAL_SECS`
- **说明**:
  - 值越小，数据丢失风险越低
  - 但 IO 开销越大

---

### Redis 存储配置

**注意**: 需要启用 `store-redis` feature

#### `storage.redis.url`

Redis 服务器连接 URL。

- **类型**: String
- **格式**: `redis://[host]:[port]`
- **默认值**: `redis://127.0.0.1:6379`
- **环境变量**: `STORAGE_REDIS_URL`

#### `storage.redis.pool_size`

Redis 连接池大小。

- **类型**: Integer
- **范围**: 1-100
- **默认值**: 10
- **环境变量**: `STORAGE_REDIS_POOL_SIZE`

---

### WAL 存储配置

**注意**: 需要启用 `store-wal` feature

#### `storage.wal.base_path`

WAL 文件存储路径。

- **类型**: String
- **默认值**: `./wal`
- **环境变量**: `STORAGE_WAL_BASE_PATH`

#### `storage.wal.max_wal_size`

最大 WAL 大小（字节）。

- **类型**: Integer
- **范围**: 1048576-1073741824
- **默认值**: 104857600 (100MB)
- **环境变量**: `STORAGE_WAL_MAX_WAL_SIZE`
- **说明**: 达到上限后会触发快照和清理

#### `storage.wal.compression`

是否启用压缩。

- **类型**: Boolean
- **默认值**: true
- **环境变量**: `STORAGE_WAL_COMPRESSION`
- **说明**:
  - 启用后减少磁盘占用
  - 但会增加 CPU 开销

#### `storage.wal.snapshot_interval`

快照间隔（操作次数）。

- **类型**: Integer
- **范围**: 100-100000
- **默认值**: 1000
- **环境变量**: `STORAGE_WAL_SNAPSHOT_INTERVAL`
- **说明**:
  - 值越小，数据恢复越快
  - 但 IO 开销越大

---

### 日志配置

#### `logging.level`

日志级别。

- **类型**: String
- **可选值**: `trace`, `debug`, `info`, `warn`, `error`
- **默认值**: `info`
- **环境变量**: `LOG_LEVEL`
- **说明**:
  - `trace`: 最详细，包含所有调试信息
  - `debug`: 详细调试信息
  - `info`: 一般信息（推荐）
  - `warn`: 仅警告和错误
  - `error`: 仅错误

#### `logging.format`

日志格式。

- **类型**: String
- **可选值**: `json`, `text`
- **默认值**: `json`
- **环境变量**: `LOG_FORMAT`
- **说明**:
  - `json`: 结构化日志，适合日志收集系统
  - `text`: 人类可读格式，适合开发调试

---

## 环境变量优先级

当配置文件和环境变量同时存在时，**环境变量优先级更高**。

示例：

```bash
# 配置文件设置
# config.toml
[server]
host = "0.0.0.0"
port = 50051

[index]
resolution = 9

# 环境变量覆盖
export INVSEARCH_PORT=8080
export INVSEARCH_RESOLUTION=10

# 实际使用值：
# server.host = "0.0.0.0"  (配置文件)
# server.port = 8080  (环境变量优先)
# index.resolution = 10  (环境变量优先)
```

---

## 配置验证规则

Inversearch 服务在启动时会自动验证配置的有效性（需要实现验证模块）：

| 配置项 | 验证规则 |
|--------|----------|
| `server.port` | 必须在 [1, 65535] 范围内 |
| `server.workers` | 必须在 [1, 32] 范围内 |
| `index.resolution` | 必须在 [1, 12] 范围内 |
| `index.tokenize` | 必须是有效分词模式 |
| `index.depth` | 必须在 [0, 10] 范围内 |
| `cache.size` | 必须在 [1, 1000000] 范围内 |
| `cache.ttl` | 必须在 [0, 86400] 范围内 |
| `logging.level` | 必须是有效日志级别 |
| `logging.format` | 必须是有效日志格式 |

---

## 最佳实践

### 1. 开发环境配置

```toml
[server]
host = "127.0.0.1"
port = 50051
workers = 2

[index]
resolution = 8
tokenize = "strict"
depth = 0
bidirectional = true
fastupdate = false

[cache]
enabled = false

[storage]
enabled = false

[logging]
level = "debug"
format = "text"
```

### 2. 生产环境配置（单机）

```toml
[server]
host = "0.0.0.0"
port = 50051
workers = 8

[index]
resolution = 9
tokenize = "strict"
depth = 2
bidirectional = true
fastupdate = false

[cache]
enabled = true
size = 10000
ttl = 3600

[storage]
enabled = true
backend = "cold_warm_cache"

[storage.file]
base_path = "/var/lib/inversearch/data"
auto_save = true
save_interval_secs = 60

[logging]
level = "info"
format = "json"
```

### 3. 生产环境配置（分布式）

```toml
[server]
host = "0.0.0.0"
port = 50051
workers = 16

[index]
resolution = 10
tokenize = "bidirectional"
depth = 3
bidirectional = true
fastupdate = false

[cache]
enabled = true
size = 50000
ttl = 1800

[storage]
enabled = true
backend = "redis"

[storage.redis]
url = "redis://redis-cluster:6379"
pool_size = 20

[logging]
level = "info"
format = "json"
```

### 4. 高性能搜索配置

```toml
[server]
host = "0.0.0.0"
port = 50051
workers = 32

[index]
resolution = 10
tokenize = "full"
depth = 5
bidirectional = true
fastupdate = false

[cache]
enabled = true
size = 100000
ttl = 7200

[storage]
enabled = true
backend = "cold_warm_cache"

[storage.file]
base_path = "/mnt/ssd/inversearch/data"
auto_save = true
save_interval_secs = 30

[logging]
level = "warn"
format = "json"
```

---

## Feature 配置

Inversearch 使用 Cargo features 控制编译选项：

### 可用 Features

```toml
[features]
default = ["embedded", "store"]
embedded = []  # 嵌入式库模式
service = ["tonic", "prost", "tokio/full", "prost-build", "tonic-build"]  # 服务模式
store = ["store-cold-warm-cache"]  # 存储支持
store-cold-warm-cache = ["store-wal", "store-file"]  # 冷热双层缓存
store-file = []  # 文件存储
store-redis = ["redis", "bb8"]  # Redis 存储
store-wal = []  # WAL 存储
```

### 使用示例

**嵌入式库模式**：

```toml
[dependencies]
inversearch-service = { path = "../inversearch", default-features = false, features = ["embedded"] }
```

**服务模式（带文件存储）**：

```toml
[dependencies]
inversearch-service = { path = "../inversearch", features = ["service", "store-file"] }
```

**完整服务模式**：

```toml
[dependencies]
inversearch-service = { path = "../inversearch", features = ["service", "store", "store-redis"] }
```

---

**文档版本**: 1.0  
**最后更新**: 2026-04-07
