# Feature Flags 可选功能方案

**文档版本**: 1.0  
**创建日期**: 2026-03-26  
**适用范围**: 全项目

---

## 一、背景与目标

### 1.1 背景

GraphDB 作为一个功能完整的图数据库，包含了大量功能模块：

- 查询引擎（解析器、规划器、优化器、执行器）
- 缓存系统（计划缓存、CTE 缓存）
- 对象池
- HTTP 服务器
- 用户认证与权限
- 监控统计
- 二级索引
- 事务保存点

对于不同的使用场景，并非所有功能都是必需的。例如：

- **嵌入式场景**: 不需要 HTTP 服务器、用户认证
- **简单查询场景**: 不需要复杂的查询优化器
- **低内存环境**: 不需要缓存系统
- **单用户场景**: 不需要权限控制

### 1.2 目标

1. **模块化编译**: 通过 Feature Flags 控制功能包含
2. **减少二进制体积**: 嵌入式场景可减少 30-50% 代码体积
3. **降低内存占用**: 禁用不必要功能，减少运行时开销
4. **提高启动速度**: 减少初始化模块，加快启动
5. **灵活部署**: 根据场景选择合适的功能组合

---

## 二、Feature Flags 设计

### 2.1 功能分类

| 类别 | 功能 | 说明 | 默认 |
|------|------|------|------|
| **核心功能** | query-engine | 查询引擎（必需） | 始终启用 |
| **缓存** | plan-cache | 查询计划缓存 | 启用 |
| | cte-cache | CTE 结果缓存 | 启用 |
| | object-pool | 执行器对象池 | 启用 |
| **网络** | http-server | HTTP API 服务器 | 启用 |
| **安全** | auth | 用户认证 | 启用 |
| | password-hashing | 密码哈希（bcrypt） | 启用 |
| | permission | 权限控制 | 启用 |
| **优化** | query-optimizer | 查询优化器 | 启用 |
| | secondary-index | 二级索引 | 启用 |
| **事务** | savepoint | 事务保存点 | 启用 |
| | transaction-timeout | 事务超时检测 | 启用 |
| **监控** | metrics | 性能指标收集 | 启用 |
| | query-log | 查询日志 | 禁用 |
| **高级** | async-query | 异步查询执行 | 禁用 |
| | parallel-scan | 并行扫描 | 启用 |

### 2.2 Cargo.toml 配置

```toml
# Cargo.toml

[features]
default = [
    "plan-cache",
    "cte-cache",
    "object-pool",
    "http-server",
    "auth",
    "password-hashing",
    "permission",
    "query-optimizer",
    "secondary-index",
    "savepoint",
    "transaction-timeout",
    "metrics",
    "parallel-scan",
]

# 最小化嵌入式配置 - 最小体积和内存占用
embedded = [
    "query-engine",
]

# 嵌入式完整配置 - 保留核心功能
embedded-full = [
    "plan-cache",
    "object-pool",
    "query-optimizer",
    "secondary-index",
    "savepoint",
]

# 高性能服务器配置
server = [
    "plan-cache",
    "cte-cache",
    "object-pool",
    "http-server",
    "auth",
    "password-hashing",
    "permission",
    "query-optimizer",
    "secondary-index",
    "savepoint",
    "transaction-timeout",
    "metrics",
    "query-log",
    "async-query",
    "parallel-scan",
]

# 开发调试配置
dev = [
    "default",
    "query-log",
]

# ==================== 独立功能开关 ====================

# 缓存相关
plan-cache = []
cte-cache = ["plan-cache"]
object-pool = []

# 网络相关
http-server = ["dep:axum", "dep:tower", "dep:tower-http"]

# 安全相关
auth = ["password-hashing"]
password-hashing = ["dep:bcrypt"]
permission = ["auth"]

# 优化相关
query-optimizer = []
secondary-index = []

# 事务相关
savepoint = []
transaction-timeout = []

# 监控相关
metrics = []
query-log = []

# 高级功能
async-query = ["dep:tokio"]
parallel-scan = ["dep:rayon"]
```

---

## 三、条件编译实现

### 3.1 模块级别条件编译

```rust
// src/lib.rs

// 核心模块（始终编译）
pub mod core;
pub mod storage;
pub mod query;
pub mod transaction;

// 可选模块
#[cfg(feature = "http-server")]
pub mod api;

#[cfg(feature = "metrics")]
pub mod metrics;
```

### 3.2 查询引擎中的条件编译

```rust
// src/query/mod.rs

pub mod parser;
pub mod planner;
pub mod executor;

// 可选：查询优化器
#[cfg(feature = "query-optimizer")]
pub mod optimizer;

// 可选：缓存
#[cfg(feature = "plan-cache")]
pub mod cache;

// src/query/planner/mod.rs

use crate::query::planning::plan::ExecutionPlan;

#[cfg(feature = "query-optimizer")]
use crate::query::optimizer::OptimizerEngine;

pub struct QueryPlanner {
    // 基础字段...
    
    #[cfg(feature = "query-optimizer")]
    optimizer: Option<OptimizerEngine>,
}

impl QueryPlanner {
    pub fn create_plan(&self, ast: &AST) -> Result<ExecutionPlan, PlanError> {
        let plan = self.build_initial_plan(ast)?;
        
        // 条件编译：仅在启用优化器时进行优化
        #[cfg(feature = "query-optimizer")]
        let plan = if let Some(ref optimizer) = self.optimizer {
            optimizer.optimize(plan)?
        } else {
            plan
        };
        
        Ok(plan)
    }
}
```

### 3.3 执行器中的条件编译

```rust
// src/query/executor/mod.rs

pub mod base;
pub mod data_access;
pub mod data_processing;

// 可选：对象池
#[cfg(feature = "object-pool")]
pub mod object_pool;

// src/query/executor/factory.rs

use crate::query::executor::ExecutorEnum;

pub struct ExecutorFactory<S: StorageClient> {
    storage: Arc<Mutex<S>>,
    
    #[cfg(feature = "object-pool")]
    object_pool: Option<ThreadSafeExecutorPool<S>>,
}

impl<S: StorageClient> ExecutorFactory<S> {
    pub fn create_executor(&self, plan: &PlanNode) -> ExecutorEnum<S> {
        // 尝试从对象池获取
        #[cfg(feature = "object-pool")]
        if let Some(ref pool) = self.object_pool {
            if let Some(executor) = pool.acquire(&plan.executor_type()) {
                return executor;
            }
        }
        
        // 创建新执行器
        self.create_new_executor(plan)
    }
    
    pub fn release_executor(&self, executor: ExecutorEnum<S>) {
        #[cfg(feature = "object-pool")]
        if let Some(ref pool) = self.object_pool {
            pool.release(&executor.executor_type(), executor);
        }
        // 未启用对象池时，执行器将被丢弃
    }
}
```

### 3.4 存储层中的条件编译

```rust
// src/storage/mod.rs

pub mod redb_storage;
pub mod operations;

// 可选：索引系统
#[cfg(feature = "secondary-index")]
pub mod index;

// src/storage/redb_storage.rs

pub struct RedbStorage {
    // 基础字段...
    db: Arc<Database>,
    
    // 可选：索引管理器
    #[cfg(feature = "secondary-index")]
    index_manager: Option<IndexManager>,
}

impl RedbStorage {
    pub fn create_index(&self, index_def: IndexDef) -> Result<(), StorageError> {
        #[cfg(feature = "secondary-index")]
        {
            if let Some(ref manager) = self.index_manager {
                return manager.create_index(index_def);
            }
        }
        
        #[cfg(not(feature = "secondary-index"))]
        {
            Err(StorageError::Unsupported(
                "Secondary index is disabled. Enable 'secondary-index' feature.".to_string()
            ))
        }
    }
}
```

### 3.5 事务管理中的条件编译

```rust
// src/transaction/context.rs

pub struct TransactionContext {
    id: TransactionId,
    state: AtomicCell<TransactionState>,
    
    // 可选：保存点
    #[cfg(feature = "savepoint")]
    savepoint_manager: Option<SavepointManager>,
    
    // 可选：超时检测
    #[cfg(feature = "transaction-timeout")]
    timeout: Option<Duration>,
}

impl TransactionContext {
    #[cfg(feature = "savepoint")]
    pub fn create_savepoint(&self, name: &str) -> Result<SavepointId, TransactionError> {
        match &self.savepoint_manager {
            Some(manager) => manager.create(name),
            None => Err(TransactionError::Unsupported(
                "Savepoint is disabled".to_string()
            )),
        }
    }
    
    #[cfg(not(feature = "savepoint"))]
    pub fn create_savepoint(&self, _name: &str) -> Result<SavepointId, TransactionError> {
        Err(TransactionError::Unsupported(
            "Savepoint feature is not enabled. Compile with 'savepoint' feature.".to_string()
        ))
    }
}
```

### 3.6 API 层中的条件编译

```rust
// src/api/mod.rs

pub mod embedded;

#[cfg(feature = "http-server")]
pub mod server;

// src/api/server/mod.rs

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "permission")]
pub mod permission;

// src/api/server/http_server.rs

use axum::Router;

#[cfg(feature = "auth")]
use crate::api::server::auth::AuthMiddleware;

pub fn create_router() -> Router {
    let mut router = Router::new()
        .route("/query", post(handle_query));
    
    // 条件添加认证中间件
    #[cfg(feature = "auth")]
    {
        router = router.layer(AuthMiddleware::new());
    }
    
    router
}
```

---

## 四、配置运行时检查

### 4.1 功能检测 API

```rust
// src/config/feature_flags.rs (新建)

//! 功能标志运行时检测
//! 
//! 提供运行时检查功能是否启用的 API

/// 功能标志检测结构体
pub struct FeatureFlags;

impl FeatureFlags {
    /// 检查是否启用了查询计划缓存
    pub const fn plan_cache_enabled() -> bool {
        cfg!(feature = "plan-cache")
    }
    
    /// 检查是否启用了 CTE 缓存
    pub const fn cte_cache_enabled() -> bool {
        cfg!(feature = "cte-cache")
    }
    
    /// 检查是否启用了对象池
    pub const fn object_pool_enabled() -> bool {
        cfg!(feature = "object-pool")
    }
    
    /// 检查是否启用了 HTTP 服务器
    pub const fn http_server_enabled() -> bool {
        cfg!(feature = "http-server")
    }
    
    /// 检查是否启用了认证
    pub const fn auth_enabled() -> bool {
        cfg!(feature = "auth")
    }
    
    /// 检查是否启用了查询优化器
    pub const fn query_optimizer_enabled() -> bool {
        cfg!(feature = "query-optimizer")
    }
    
    /// 检查是否启用了二级索引
    pub const fn secondary_index_enabled() -> bool {
        cfg!(feature = "secondary-index")
    }
    
    /// 检查是否启用了保存点
    pub const fn savepoint_enabled() -> bool {
        cfg!(feature = "savepoint")
    }
    
    /// 检查是否启用了指标收集
    pub const fn metrics_enabled() -> bool {
        cfg!(feature = "metrics")
    }
    
    /// 获取所有启用的功能列表
    pub fn enabled_features() -> Vec<&'static str> {
        let mut features = vec!["query-engine"];
        
        if Self::plan_cache_enabled() {
            features.push("plan-cache");
        }
        if Self::cte_cache_enabled() {
            features.push("cte-cache");
        }
        if Self::object_pool_enabled() {
            features.push("object-pool");
        }
        if Self::http_server_enabled() {
            features.push("http-server");
        }
        if Self::auth_enabled() {
            features.push("auth");
        }
        if Self::query_optimizer_enabled() {
            features.push("query-optimizer");
        }
        if Self::secondary_index_enabled() {
            features.push("secondary-index");
        }
        if Self::savepoint_enabled() {
            features.push("savepoint");
        }
        if Self::metrics_enabled() {
            features.push("metrics");
        }
        
        features
    }
}

/// 功能信息结构体
#[derive(Debug, Clone)]
pub struct FeatureInfo {
    pub name: &'static str,
    pub enabled: bool,
    pub description: &'static str,
}

/// 获取所有功能信息
pub fn get_all_features() -> Vec<FeatureInfo> {
    vec![
        FeatureInfo {
            name: "plan-cache",
            enabled: FeatureFlags::plan_cache_enabled(),
            description: "Query plan caching for prepared statements",
        },
        FeatureInfo {
            name: "cte-cache",
            enabled: FeatureFlags::cte_cache_enabled(),
            description: "CTE query result caching",
        },
        FeatureInfo {
            name: "object-pool",
            enabled: FeatureFlags::object_pool_enabled(),
            description: "Executor object pooling",
        },
        FeatureInfo {
            name: "http-server",
            enabled: FeatureFlags::http_server_enabled(),
            description: "HTTP API server",
        },
        FeatureInfo {
            name: "auth",
            enabled: FeatureFlags::auth_enabled(),
            description: "User authentication",
        },
        FeatureInfo {
            name: "query-optimizer",
            enabled: FeatureFlags::query_optimizer_enabled(),
            description: "Query plan optimization",
        },
        FeatureInfo {
            name: "secondary-index",
            enabled: FeatureFlags::secondary_index_enabled(),
            description: "Secondary index support",
        },
        FeatureInfo {
            name: "savepoint",
            enabled: FeatureFlags::savepoint_enabled(),
            description: "Transaction savepoints",
        },
        FeatureInfo {
            name: "metrics",
            enabled: FeatureFlags::metrics_enabled(),
            description: "Performance metrics collection",
        },
    ]
}
```

### 4.2 启动时功能报告

```rust
// src/lib.rs 或 src/main.rs

pub fn print_feature_info() {
    println!("GraphDB Features:");
    println!("================");
    
    for feature in get_all_features() {
        let status = if feature.enabled { "✓" } else { "✗" };
        println!("  {} {} - {}", status, feature.name, feature.description);
    }
}
```

---

## 五、使用场景配置

### 5.1 嵌入式最小配置

适用于资源受限的嵌入式设备：

```toml
# Cargo.toml
[dependencies]
graphdb = { version = "0.1.0", default-features = false, features = ["embedded"] }
```

**特点**:
- 二进制体积最小（约减少 50%）
- 内存占用最低
- 仅支持基础查询
- 无网络功能

### 5.2 嵌入式完整配置

适用于需要完整功能但不需要网络的场景：

```toml
# Cargo.toml
[dependencies]
graphdb = { version = "0.1.0", default-features = false, features = ["embedded-full"] }
```

**特点**:
- 保留查询优化和索引
- 支持缓存
- 无 HTTP 服务器
- 无认证

### 5.3 服务器配置

适用于高性能服务器部署：

```toml
# Cargo.toml
[dependencies]
graphdb = { version = "0.1.0", features = ["server"] }
```

**特点**:
- 启用所有功能
- 支持异步查询
- 完整监控
- 查询日志

### 5.4 自定义配置

```toml
# Cargo.toml
[dependencies]
graphdb = { 
    version = "0.1.0", 
    default-features = false,
    features = [
        "plan-cache",
        "query-optimizer",
        "secondary-index",
        "http-server",
        "metrics",
    ]
}
```

---

## 六、实施步骤

### 阶段一：添加 Feature Flags 到 Cargo.toml（低风险）

1. 修改根目录 `Cargo.toml`
2. 定义所有功能标志
3. 配置默认功能和预设

### 阶段二：添加条件编译到核心模块（中风险）

1. 修改 `src/query/mod.rs`
2. 修改 `src/query/planner/mod.rs`
3. 修改 `src/query/executor/mod.rs`
4. 修改 `src/storage/mod.rs`
5. 修改 `src/transaction/mod.rs`

### 阶段三：添加条件编译到可选模块（中风险）

1. 修改 `src/api/mod.rs`
2. 修改 `src/metrics/mod.rs`（如果存在）

### 阶段四：创建功能检测 API（低风险）

1. 创建 `src/config/feature_flags.rs`
2. 实现 `FeatureFlags` 结构体
3. 添加启动时功能报告

### 阶段五：测试和验证（高风险）

1. 测试每种功能组合
2. 验证最小配置编译通过
3. 验证完整配置编译通过
4. 运行功能测试套件

---

## 七、预期收益

| 配置 | 二进制体积 | 内存占用 | 启动时间 | 适用场景 |
|------|-----------|----------|----------|----------|
| `embedded` | -50% | -40% | -30% | IoT 设备、嵌入式 |
| `embedded-full` | -30% | -20% | -15% | 桌面应用、单机版 |
| `default` | 基准 | 基准 | 基准 | 通用场景 |
| `server` | +20% | +30% | +10% | 高性能服务器 |

---

## 八、注意事项

### 8.1 功能依赖

某些功能依赖于其他功能：

- `cte-cache` 依赖于 `plan-cache`
- `auth` 依赖于 `password-hashing`
- `permission` 依赖于 `auth`
- `http-server` 依赖于 `tokio`（外部依赖）

### 8.2 编译错误处理

当禁用某些功能时，需要确保：

1. 代码能够编译通过
2. 运行时返回适当的错误信息
3. 文档中说明功能依赖关系

### 8.3 测试覆盖

需要为以下配置添加 CI 测试：

- `embedded` 配置
- `embedded-full` 配置
- `default` 配置
- `server` 配置

---

## 九、示例代码

### 9.1 运行时功能检测

```rust
use graphdb::config::FeatureFlags;

fn main() {
    // 检查功能是否启用
    if FeatureFlags::plan_cache_enabled() {
        println!("Query plan cache is enabled");
    } else {
        println!("Query plan cache is disabled - consider enabling for better performance");
    }
    
    // 打印所有功能
    for feature in get_all_features() {
        println!("{}: {}", 
            feature.name, 
            if feature.enabled { "enabled" } else { "disabled" }
        );
    }
}
```

### 9.2 条件初始化

```rust
use graphdb::GraphDB;
use graphdb::config::FeatureFlags;

fn initialize_db() -> GraphDB {
    let mut builder = GraphDB::builder();
    
    // 根据功能标志配置
    if FeatureFlags::plan_cache_enabled() {
        builder = builder.with_plan_cache(PlanCacheConfig::default());
    }
    
    if FeatureFlags::object_pool_enabled() {
        builder = builder.with_object_pool(ObjectPoolConfig::default());
    }
    
    #[cfg(feature = "http-server")]
    {
        builder = builder.with_http_server(HttpConfig::default());
    }
    
    builder.build()
}
```
