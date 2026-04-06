# Inversearch 存储工厂使用指南

## 概述

Inversearch 现在使用 **StorageFactory** 模式来创建和管理存储后端。这提供了：
- 统一的存储创建接口
- 灵活的配置方式
- 清晰的职责分离
- 易于测试和扩展

## 快速开始

### 1. 使用配置文件

创建 `config.toml`：

```toml
[storage]
enabled = true
backend = "cold_warm_cache"  # 或 "file", "redis", "wal"

# 文件存储配置
[storage.file]
base_path = "./data"
auto_save = true
save_interval_secs = 60

# Redis 存储配置
# [storage.redis]
# url = "redis://127.0.0.1:6379"
# pool_size = 10

# WAL 存储配置
# [storage.wal]
# base_path = "./wal"
# max_wal_size = 104857600
# compression = true
# snapshot_interval = 1000
```

加载配置并创建存储：

```rust
use inversearch::config::Config;
use inversearch::storage::StorageFactory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从文件加载配置
    let config = Config::from_file("config.toml")?;
    
    // 使用工厂创建存储
    let storage = StorageFactory::from_config(&config).await?;
    
    // 使用存储...
    Ok(())
}
```

### 2. 使用 Builder 模式

```rust
use inversearch::config::{Config, StorageConfig, StorageBackend};
use inversearch::storage::StorageFactory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用 Builder 创建存储配置
    let storage_config = StorageConfig::builder()
        .enabled(true)
        .backend(StorageBackend::Redis)
        .redis(inversearch::config::RedisConfig {
            url: "redis://localhost:6379".to_string(),
            pool_size: 20,
        })
        .build();
    
    let config = Config {
        storage: storage_config,
        ..Default::default()
    };
    
    let storage = StorageFactory::from_config(&config).await?;
    
    Ok(())
}
```

### 3. 直接使用工厂方法

```rust
use inversearch::config::StorageConfig;
use inversearch::storage::StorageFactory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 直接创建特定类型的存储
    let storage_config = StorageConfig::default();
    
    // 创建文件存储
    #[cfg(feature = "store-file")]
    {
        let storage = StorageFactory::create_file(&storage_config)?;
    }
    
    // 创建 Redis 存储
    #[cfg(feature = "store-redis")]
    {
        let storage = StorageFactory::create_redis(&storage_config).await?;
    }
    
    // 创建冷热缓存
    let storage = StorageFactory::create_cold_warm_cache().await?;
    
    Ok(())
}
```

## 存储后端类型

### ColdWarmCache（冷热缓存）

**默认存储后端**，适用于大多数场景。

```toml
[storage]
enabled = true
backend = "cold_warm_cache"
```

**特点**：
- 内存缓存 + 持久化存储
- 自动管理热数据和冷数据
- 高性能读取
- 支持后台持久化

**适用场景**：
- 通用搜索应用
- 需要平衡性能和持久性
- 中小规模数据

### FileStorage（文件存储）

```toml
[storage]
enabled = true
backend = "file"

[storage.file]
base_path = "./data"
auto_save = true
save_interval_secs = 300
```

**特点**：
- 本地文件持久化
- 简单可靠
- 无需额外服务

**适用场景**：
- 单机部署
- 数据量适中
- 需要简单持久化

### RedisStorage（Redis 存储）

```toml
[storage]
enabled = true
backend = "redis"

[storage.redis]
url = "redis://127.0.0.1:6379"
pool_size = 20
```

**特点**：
- 分布式存储
- 高性能
- 支持集群

**适用场景**：
- 分布式系统
- 大规模数据
- 需要高可用

### WALStorage（WAL 预写日志）

```toml
[storage]
enabled = true
backend = "wal"

[storage.wal]
base_path = "./wal"
max_wal_size = 104857600
compression = true
snapshot_interval = 1000
```

**特点**：
- 预写日志保证数据一致性
- 支持崩溃恢复
- 可配置快照

**适用场景**：
- 对数据一致性要求高
- 需要崩溃恢复
- 写入频繁

## 配置方式对比

### 配置文件（推荐）

**优点**：
- 集中管理配置
- 易于版本控制
- 支持多环境

**示例**：
```rust
let config = Config::from_file("config.toml")?;
let storage = StorageFactory::from_config(&config).await?;
```

### 环境变量

**优点**：
- 适合容器化部署
- 敏感信息安全

**示例**：
```bash
export INVERSEARCH_STORAGE_ENABLED=true
export INVERSEARCH_STORAGE_BACKEND=redis
export INVERSEARCH_STORAGE_REDIS_URL=redis://localhost:6379
```

```rust
let config = Config::from_env()?;
let storage = StorageFactory::from_config(&config).await?;
```

### Builder 模式

**优点**：
- 编程方式灵活配置
- 类型安全
- 支持运行时决策

**示例**：
```rust
let storage_config = StorageConfig::builder()
    .enabled(true)
    .backend(StorageBackend::Redis)
    .redis(RedisConfig {
        url: "redis://localhost:6379".to_string(),
        pool_size: 20,
    })
    .build();
```

## 使用 StorageInterface

所有存储后端都实现了 `StorageInterface` trait：

```rust
use inversearch::storage::StorageInterface;
use std::sync::Arc;

async fn use_storage(storage: Arc<dyn StorageInterface>) {
    // 挂载索引
    // storage.mount(&index).await?;
    
    // 提交变更
    // storage.commit(&index, false, false).await?;
    
    // 查询数据
    // let results = storage.get("keyword", None, 10, 0, false, false).await?;
    
    // 健康检查
    // let healthy = storage.health_check().await?;
    
    // 获取统计信息
    // let info = storage.info().await?;
}
```

## 错误处理

```rust
use inversearch::storage::StorageFactory;
use inversearch::error::InversearchError;

async fn create_storage() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    
    match StorageFactory::from_config(&config).await {
        Ok(storage) => {
            println!("Storage created successfully");
            Ok(())
        }
        Err(InversearchError::StorageError(msg)) => {
            eprintln!("Storage error: {}", msg);
            Err(msg.into())
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
            Err(Box::new(e))
        }
    }
}
```

## 故障降级

StorageFactory 自动支持故障降级：

```rust
use inversearch::storage::StorageFactory;

async fn create_storage_with_fallback() {
    let config = Config::default();
    
    let storage = match StorageFactory::from_config(&config).await {
        Ok(storage) => storage,
        Err(e) => {
            eprintln!("Failed to create primary storage: {}", e);
            eprintln!("Falling back to cold-warm cache");
            // 自动降级到 ColdWarmCache
            StorageFactory::create_cold_warm_cache().await.unwrap()
        }
    };
}
```

## 特性标志

在 `Cargo.toml` 中配置：

```toml
[dependencies]
inversearch-service = { path = "./inversearch" }

[features]
default = ["store-cold-warm-cache"]
store-file = []
store-redis = ["redis"]
store-wal = []
```

编译选项：

```bash
# 仅启用冷热缓存
cargo build

# 启用文件存储
cargo build --features store-file

# 启用 Redis 存储
cargo build --features store-redis

# 启用所有存储
cargo build --features "store-file,store-redis,store-wal"
```

## 性能优化建议

### ColdWarmCache

```toml
[storage]
backend = "cold_warm_cache"
# 调整缓存大小
# 在 cold_warm_cache 配置中设置
```

### FileStorage

```toml
[storage.file]
# 增加保存间隔（减少 I/O）
save_interval_secs = 300
# 启用压缩（如果支持）
compression = true
```

### RedisStorage

```toml
[storage.redis]
# 根据并发量调整连接池
pool_size = 30
# 启用连接保持
min_idle = 5
```

### WALStorage

```toml
[storage.wal]
# 增加 WAL 大小（减少检查点频率）
max_wal_size = 209715200
# 启用压缩
compression = true
```

## 监控和调试

### 获取存储信息

```rust
use inversearch::storage::StorageInterface;

async fn monitor_storage(storage: Arc<dyn StorageInterface>) {
    let info = storage.info().await?;
    println!("Storage: {}", info.name);
    println!("Version: {}", info.version);
    println!("Backend: {:?}", info.backend);
}
```

### 日志配置

```toml
[logging]
level = "debug"  # 或 "info", "warn", "error"
format = "json"
```

## 最佳实践

### 1. 选择合适的存储后端

- **开发/测试**: ColdWarmCache（简单、无需配置）
- **生产环境**:
  - 小规模 (< 100 万文档): FileStorage 或 ColdWarmCache
  - 中等规模 (100-1000 万): RedisStorage
  - 大规模 (> 1000 万): RedisStorage 集群或 WALStorage

### 2. 配置管理

- 使用配置文件管理不同环境
- 敏感信息使用环境变量
- 为不同索引使用不同的 `base_path`

### 3. 连接池配置（Redis）

```toml
[storage.redis]
# 根据并发量调整
pool_size = 20          # 高并发增加
min_idle = 5            # 保持最小空闲连接
max_lifetime_secs = 300 # 连接生命周期
```

### 4. 持久化策略（File/WAL）

```toml
[storage.file]
auto_save = true
save_interval_secs = 300  # 5 分钟保存一次

[storage.wal]
snapshot_interval = 1000  # 1000 次操作后快照
```

## 常见问题

### Q: 如何切换存储后端？

A: 只需修改配置文件中的 `backend` 字段：

```toml
# 从 ColdWarmCache 切换到 Redis
[storage]
backend = "redis"

[storage.redis]
url = "redis://localhost:6379"
```

### Q: 存储创建失败怎么办？

A: StorageFactory 会自动降级到 ColdWarmCache。也可以手动处理错误：

```rust
match StorageFactory::from_config(&config).await {
    Ok(storage) => storage,
    Err(e) => {
        eprintln!("Error: {}", e);
        // 使用默认存储
        StorageFactory::create_cold_warm_cache().await?
    }
}
```

### Q: 如何测试存储？

A: 使用单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_storage_factory() {
        let config = Config::default();
        let storage = StorageFactory::from_config(&config).await.unwrap();
        assert!(storage.info().await.is_ok());
    }
}
```

## 总结

通过 StorageFactory，Inversearch 提供了：

✅ **统一的创建接口** - `StorageFactory::from_config()`
✅ **灵活的配置方式** - 文件、环境变量、Builder
✅ **多种存储后端** - ColdWarmCache、File、Redis、WAL
✅ **自动故障降级** - 创建失败自动降级
✅ **清晰的职责分离** - 工厂负责创建，服务负责使用

这种设计使得存储管理变得简单，只需修改配置即可切换不同的存储后端，无需更改业务代码。
