# Query Context 重构实施计划

## 概述

本文档提供详细的、分阶段的实施计划。重构分为两大阶段：
1. **类型系统重构**（优先）：metadata.rs原子化拆分，统一类型定义
2. **Context架构重构**：删除AstContext，重构QueryContext

---

## 第一阶段：类型系统重构（优先）

### 目标
将567行的metadata.rs拆分为原子类型文件，统一SpaceInfo定义。

---

### Phase 1.1: 创建core/types/space.rs

**操作**：创建新文件 `src/core/types/space.rs`

**内容**：图空间基础类型

```rust
//! 图空间基础类型

use crate::core::types::DataType;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

static SPACE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn generate_space_id() -> u64 {
    SPACE_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn reset_space_id_counter() {
    SPACE_ID_COUNTER.store(1, Ordering::SeqCst);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct SpaceInfo {
    pub space_id: u64,
    pub space_name: String,
    pub vid_type: DataType,
    pub comment: Option<String>,
}

impl SpaceInfo {
    pub fn new(space_name: String) -> Self {
        Self {
            space_id: generate_space_id(),
            space_name,
            vid_type: DataType::String,
            comment: None,
        }
    }

    pub fn with_vid_type(mut self, vid_type: DataType) -> Self {
        self.vid_type = vid_type;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}

impl Default for SpaceInfo {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}
```

**验证**：
```bash
cargo check --lib
```

---

### Phase 1.2: 创建core/types/property.rs

**操作**：创建新文件 `src/core/types/property.rs`

**内容**：属性定义基础类型

```rust
//! 属性定义基础类型

use crate::core::{DataType, Value};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct PropertyDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default: Option<Value>,
    pub comment: Option<String>,
}

impl PropertyDef {
    pub fn new(name: String, data_type: DataType) -> Self {
        Self {
            name,
            data_type,
            nullable: false,
            default: None,
            comment: None,
        }
    }

    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    pub fn with_default(mut self, default: Option<Value>) -> Self {
        self.default = default;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}

impl Default for PropertyDef {
    fn default() -> Self {
        Self::new("default".to_string(), DataType::String)
    }
}
```

**验证**：
```bash
cargo check --lib
```

---

### Phase 1.3: 创建core/types/tag.rs

**操作**：创建新文件 `src/core/types/tag.rs`

**内容**：标签基础类型

```rust
//! 标签基础类型

use super::property::PropertyDef;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct TagInfo {
    pub tag_id: i32,
    pub tag_name: String,
    pub properties: Vec<PropertyDef>,
    pub comment: Option<String>,
}

impl TagInfo {
    pub fn new(tag_name: String) -> Self {
        Self {
            tag_id: 0,
            tag_name,
            properties: Vec::new(),
            comment: None,
        }
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDef>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}

impl Default for TagInfo {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}
```

**验证**：
```bash
cargo check --lib
```

---

### Phase 1.4: 创建core/types/edge.rs

**操作**：创建新文件 `src/core/types/edge.rs`

**内容**：边类型基础定义

```rust
//! 边类型基础定义

use super::property::PropertyDef;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct EdgeTypeInfo {
    pub edge_type_id: i32,
    pub edge_type_name: String,
    pub properties: Vec<PropertyDef>,
    pub comment: Option<String>,
}

impl EdgeTypeInfo {
    pub fn new(edge_type_name: String) -> Self {
        Self {
            edge_type_id: 0,
            edge_type_name,
            properties: Vec::new(),
            comment: None,
        }
    }

    pub fn with_properties(mut self, properties: Vec<PropertyDef>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}

impl Default for EdgeTypeInfo {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}
```

**验证**：
```bash
cargo check --lib
```

---

### Phase 1.5: 创建core/types/metadata_version.rs

**操作**：创建新文件 `src/core/types/metadata_version.rs`

**内容**：元数据版本管理类型

```rust
//! 元数据版本管理类型

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct MetadataVersion {
    pub version: i32,
    pub timestamp: i64,
    pub description: String,
}

impl Default for MetadataVersion {
    fn default() -> Self {
        Self {
            version: 1,
            timestamp: chrono::Utc::now().timestamp_millis(),
            description: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct SchemaVersion {
    pub version: i32,
    pub space_id: u64,
    pub created_at: i64,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct SchemaHistory {
    pub space_id: u64,
    pub versions: Vec<SchemaVersion>,
    pub current_version: i64,
    pub timestamp: i64,
}

impl Default for SchemaHistory {
    fn default() -> Self {
        Self {
            space_id: 0,
            versions: Vec::new(),
            current_version: 0,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}
```

**验证**：
```bash
cargo check --lib
```

---

### Phase 1.6: 修改core/types/mod.rs

**文件**：`src/core/types/mod.rs`

**操作**：
1. 添加新的原子类型模块
2. 从metadata导出改为从新模块导出
3. 删除 `pub mod space;` 的QuerySpaceInfo导出

**修改后**：
```rust
pub mod edge;
pub mod expression;
pub mod graph_schema;
pub mod metadata_version;
pub mod operators;
pub mod property;
// pub mod space;  // 删除旧space模块
pub mod space;
pub mod span;
pub mod query;
pub mod tag;
pub mod variable;

// 从原子模块导出基础Schema类型
pub use self::edge::EdgeTypeInfo;
pub use self::property::PropertyDef;
pub use self::space::{SpaceInfo, generate_space_id, reset_space_id_counter};
pub use self::tag::TagInfo;

// 从metadata_version导出版本类型
pub use self::metadata_version::{MetadataVersion, SchemaVersion, SchemaHistory};

// 保留metadata的其他导出（暂时，后续迁移）
pub use self::metadata::{
    // ... 其他非基础类型
};

// 删除：pub use self::space::QuerySpaceInfo;
```

**验证**：
```bash
cargo check --lib 2>&1 | grep -E "SpaceInfo|TagInfo|EdgeTypeInfo|PropertyDef"
```

---

### Phase 1.7: 删除core/types/space.rs中的QuerySpaceInfo

**文件**：`src/core/types/space.rs`（刚创建的文件）

**操作**：这个文件刚创建，包含QuerySpaceInfo，不需要，删除整个文件

**注意**：在Phase 1.1中我们创建了新的space.rs，这里需要确认是否覆盖了旧文件

**验证**：
```bash
cargo check --lib 2>&1 | grep "QuerySpaceInfo"
# 应该报错，记录所有使用QuerySpaceInfo的地方
```

---

### Phase 1.8: 删除query/context/ast/base.rs中的SpaceInfo

**文件**：`src/query/context/ast/base.rs`

**操作**：
1. 删除本地的 `SpaceInfo` 结构体定义
2. 添加导入：`use crate::core::types::SpaceInfo;`
3. 修改 `AstContext` 中的 `space` 字段类型

**修改后**：
```rust
use crate::core::types::SpaceInfo;  // 新增

// 删除以下定义：
// pub struct SpaceInfo { ... }

pub struct AstContext {
    // ... 其他字段
    space: Option<SpaceInfo>,  // 使用core::types::SpaceInfo
    // ...
}
```

**验证**：
```bash
cargo check --lib 2>&1 | grep -E "AstContext|SpaceInfo"
```

---

### Phase 1.9: 删除api/session/client_session.rs中的SpaceInfo

**文件**：`src/api/session/client_session.rs`

**操作**：
1. 删除本地的 `SpaceInfo` 定义
2. 修改 `Session` 结构体，直接使用字段

**修改后**：
```rust
// 删除：
// pub struct SpaceInfo {
//     pub name: String,
//     pub id: i64,
// }

pub struct Session {
    pub session_id: i64,
    pub user_name: String,
    pub space_id: Option<i64>,
    pub space_name: Option<String>,
    // ...
}
```

**验证**：
```bash
cargo check --lib 2>&1 | grep -E "client_session|SpaceInfo"
```

---

### Phase 1.10: 创建storage/types.rs

**操作**：创建新文件 `src/storage/types.rs`

**内容**：数据操作类型

```rust
//! 存储层数据操作类型

use crate::core::Value;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct InsertVertexInfo {
    pub space_id: u64,
    pub vertex_id: Value,
    pub tag_name: String,
    pub props: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct InsertEdgeInfo {
    pub space_id: u64,
    pub src_vertex_id: Value,
    pub dst_vertex_id: Value,
    pub edge_name: String,
    pub rank: i64,
    pub props: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UpdateTarget {
    pub space_name: String,
    pub label: String,
    pub id: Value,
    pub prop: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum UpdateOp {
    Set,
    Add,
    Subtract,
    Append,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UpdateInfo {
    pub update_target: UpdateTarget,
    pub update_op: UpdateOp,
    pub value: Value,
}
```

**修改** `src/storage/mod.rs`：
```rust
pub mod types;
// ...
pub use types::*;
```

**验证**：
```bash
cargo check --lib
```

---

### Phase 1.11: 创建api/session/types.rs

**操作**：创建新文件 `src/api/session/types.rs`

**内容**：用户管理类型

```rust
//! API会话层用户管理类型

use crate::core::StorageError;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UserInfo {
    pub username: String,
    pub password_hash: String,
    pub is_locked: bool,
    pub max_queries_per_hour: i32,
    pub max_updates_per_hour: i32,
    pub max_connections_per_hour: i32,
    pub max_user_connections: i32,
    pub created_at: i64,
    pub last_login_at: Option<i64>,
    pub password_changed_at: i64,
}

impl UserInfo {
    pub fn new(username: String, password: String) -> Result<Self, StorageError> {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| StorageError::DbError(format!("密码加密失败: {}", e)))?;
        
        let now = chrono::Utc::now().timestamp_millis();
        
        Ok(Self {
            username,
            password_hash,
            is_locked: false,
            max_queries_per_hour: 0,
            max_updates_per_hour: 0,
            max_connections_per_hour: 0,
            max_user_connections: 0,
            created_at: now,
            last_login_at: None,
            password_changed_at: now,
        })
    }

    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password_hash).unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct PasswordInfo {
    pub username: Option<String>,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct UserAlterInfo {
    pub username: String,
    pub is_locked: Option<bool>,
    pub max_queries_per_hour: Option<i32>,
    pub max_updates_per_hour: Option<i32>,
    pub max_connections_per_hour: Option<i32>,
    pub max_user_connections: Option<i32>,
}
```

**修改** `src/api/session/mod.rs`：
```rust
pub mod types;
// ...
pub use types::*;
```

**验证**：
```bash
cargo check --lib
```

---

### Phase 1.12: 删除并清理metadata.rs

**操作**：
1. 删除 `src/core/types/metadata.rs`
2. 修改 `src/core/types/mod.rs`，删除metadata模块引用
3. 更新所有使用metadata类型的导入

**修改** `src/core/types/mod.rs`：
```rust
// 删除：pub mod metadata;
// 删除：pub use self::metadata::{...};
```

**验证**：
```bash
cargo check --lib 2>&1 | grep -E "metadata|Metadata"
```

---

### Phase 1.13: 更新所有导入路径

**需要更新的文件**（41个文件）：
- 所有使用 `core::types::metadata::*` 的文件
- 所有使用 `core::types::{SpaceInfo, TagInfo, ...}` 的文件

**常见替换**：
```rust
// 旧
use crate::core::types::metadata::SpaceInfo;
use crate::core::types::{InsertVertexInfo, UserInfo};

// 新
use crate::core::types::SpaceInfo;  // 从space.rs导出
use crate::storage::types::InsertVertexInfo;
use crate::api::session::types::UserInfo;
```

**验证**：
```bash
cargo check --lib
```

---

### 第一阶段验证清单

- [ ] `core/types/space.rs` 已创建，包含SpaceInfo
- [ ] `core/types/property.rs` 已创建，包含PropertyDef
- [ ] `core/types/tag.rs` 已创建，包含TagInfo
- [ ] `core/types/edge.rs` 已创建，包含EdgeTypeInfo
- [ ] `core/types/metadata_version.rs` 已创建，包含版本类型
- [ ] `core/types/mod.rs` 已更新导出
- [ ] `query/context/ast/base.rs` 中的SpaceInfo已删除
- [ ] `api/session/client_session.rs` 中的SpaceInfo已删除
- [ ] `storage/types.rs` 已创建，包含数据操作类型
- [ ] `api/session/types.rs` 已创建，包含用户管理类型
- [ ] `core/types/metadata.rs` 已删除
- [ ] 所有导入路径已更新
- [ ] `cargo check --lib` 通过

---

## 第二阶段：Context架构重构

### Phase 2.1: 创建新的QueryContext

**操作**：创建新文件 `src/query/query_context.rs`

```rust
//! 查询上下文

use std::sync::{Arc, RwLock};
use crate::api::session::RequestContext;
use crate::core::symbol::SymbolTable;
use crate::core::types::SpaceInfo;
use crate::storage::metadata::{SchemaManager, IndexMetadataManager};
use crate::storage::StorageClient;

pub struct QueryContext {
    rctx: Arc<RequestContext>,
    sym_table: RwLock<SymbolTable>,
    schema_manager: Option<Arc<dyn SchemaManager>>,
    index_metadata_manager: Option<Arc<dyn IndexMetadataManager>>,
    storage_client: Option<Arc<dyn StorageClient>>,
    space_info: Option<SpaceInfo>,
}

impl QueryContext {
    pub fn new(rctx: Arc<RequestContext>) -> Self {
        Self {
            rctx,
            sym_table: RwLock::new(SymbolTable::new()),
            schema_manager: None,
            index_metadata_manager: None,
            storage_client: None,
            space_info: None,
        }
    }

    // getter方法...
    pub fn request_context(&self) -> &RequestContext {
        &self.rctx
    }

    pub fn sym_table(&self) -> &RwLock<SymbolTable> {
        &self.sym_table
    }

    pub fn space_info(&self) -> Option<&SpaceInfo> {
        self.space_info.as_ref()
    }

    pub fn set_space_info(&mut self, space_info: SpaceInfo) {
        self.space_info = Some(space_info);
    }
}
```

**修改** `src/query/mod.rs`：
```rust
pub mod query_context;
pub use query_context::QueryContext;
```

**验证**：
```bash
cargo check --lib
```

---

### Phase 2.2: 修改StatementValidator Trait

**文件**：`src/query/validator/validator_trait.rs`

**操作**：
1. 移除AstContext参数
2. 添加Arc<QueryContext>参数

```rust
use std::sync::Arc;
use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;

pub trait StatementValidator {
    fn validate(
        &mut self,
        stmt: &Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError>;
}
```

**验证**：
```bash
cargo check --lib 2>&1 | grep -E "StatementValidator|trait"
```

---

### Phase 2.3: 更新所有验证器实现

**文件**：`src/query/validator/*.rs`（40+个文件）

**操作**：修改每个验证器的validate方法

```rust
// 旧
fn validate(&mut self, ast: &mut AstContext) -> Result<...> {
    let sym_table = ast.symbol_table();
    let space = ast.space();
}

// 新
fn validate(&mut self, stmt: &Stmt, qctx: Arc<QueryContext>) -> Result<...> {
    let sym_table = qctx.sym_table().read();
    let space = qctx.space_info();
    match stmt {
        Stmt::Match(m) => { /* ... */ }
        // ...
    }
}
```

**验证**：
```bash
cargo check --lib 2>&1 | grep -E "validator|impl.*Validator"
```

---

### Phase 2.4: 修改OptContext

**文件**：`src/query/optimizer/plan/context.rs`

**操作**：
1. 将Rc改为Arc
2. 修改new方法接收Arc<QueryContext>

```rust
use std::sync::Arc;

pub struct OptContext {
    qctx: Arc<QueryContext>,
    changed: bool,
    // ...
}

impl OptContext {
    pub fn new(qctx: Arc<QueryContext>) -> Self {
        Self { qctx, changed: false, ... }
    }
}
```

**验证**：
```bash
cargo check --lib 2>&1 | grep -E "OptContext|QueryContext"
```

---

### Phase 2.5: 修改Optimizer

**文件**：`src/query/optimizer/optimizer_impl.rs`

**操作**：修改optimize方法签名

```rust
// 旧
pub fn optimize(&mut self, plan: ExecutionPlan, query_context: &mut QueryContext) -> Result<...>;

// 新
pub fn optimize(&mut self, plan: ExecutionPlan, qctx: Arc<QueryContext>) -> Result<...> {
    let mut opt_ctx = OptContext::new(qctx);  // 不克隆
    // ...
}
```

**验证**：
```bash
cargo check --lib 2>&1 | grep -E "optimize|QueryContext"
```

---

### Phase 2.6: 修改Planner

**文件**：`src/query/planner/planner.rs`

**操作**：修改plan方法签名

```rust
// 旧
pub fn plan(&self, ast: &AstContext, qctx: &QueryContext) -> Result<...>;

// 新
pub fn plan(&self, stmt: &Stmt, qctx: Arc<QueryContext>) -> Result<...>;
```

**验证**：
```bash
cargo check --lib 2>&1 | grep -E "planner|Planner"
```

---

### Phase 2.7: 删除query/context/目录

**操作**：删除整个 `src/query/context/` 目录

**包含文件**：
- ast/base.rs
- ast/common.rs
- ast/mod.rs
- request_context.rs
- runtime_context.rs
- symbol/
- components.rs
- mod.rs

**验证**：
```bash
cargo check --lib 2>&1 | grep -E "context|Context"
```

---

### Phase 2.8: 更新所有导入路径

**操作**：更新所有使用 `query::context::*` 的导入

**常见替换**：
```rust
// 旧
use crate::query::context::AstContext;
use crate::query::context::QueryContext;
use crate::query::context::symbol::SymbolTable;

// 新
use crate::query::QueryContext;
use crate::core::symbol::SymbolTable;
```

**验证**：
```bash
cargo check --lib
```

---

### 第二阶段验证清单

- [ ] 新的QueryContext已创建
- [ ] StatementValidator trait已修改
- [ ] 所有验证器实现已更新
- [ ] OptContext使用Arc<QueryContext>
- [ ] Optimizer不再克隆QueryContext
- [ ] Planner接口已更新
- [ ] src/query/context/目录已删除
- [ ] 所有导入路径已更新
- [ ] `cargo check --lib` 通过

---

## 最终验证

### 编译检查
```bash
cargo check --lib
cargo check --bins
cargo check --tests
```

### 运行测试
```bash
cargo test --lib
```

### 检查循环依赖
```bash
cargo tree --duplicates
```

---

## 附录：类型映射表

| 类型 | 旧位置 | 新位置 |
|------|--------|--------|
| SpaceInfo | core::types::metadata | core::types::space |
| TagInfo | core::types::metadata | core::types::tag |
| EdgeTypeInfo | core::types::metadata | core::types::edge |
| PropertyDef | core::types::metadata | core::types::property |
| MetadataVersion | core::types::metadata | core::types::metadata_version |
| SchemaVersion | core::types::metadata | core::types::metadata_version |
| InsertVertexInfo | core::types::metadata | storage::types |
| InsertEdgeInfo | core::types::metadata | storage::types |
| UpdateInfo | core::types::metadata | storage::types |
| UserInfo | core::types::metadata | api::session::types |
| PasswordInfo | core::types::metadata | api::session::types |
| ClusterInfo | core::types::metadata | 删除 |
| CharsetInfo | core::types::metadata | 删除 |
| AstContext | query::context::ast | 删除 |
| QueryContext | query::context | query::query_context |
| SymbolTable | query::context::symbol | core::symbol |

## 附录：导入路径变更

| 旧路径 | 新路径 |
|--------|--------|
| crate::core::types::metadata::SpaceInfo | crate::core::types::SpaceInfo |
| crate::core::types::metadata::TagInfo | crate::core::types::TagInfo |
| crate::core::types::metadata::EdgeTypeInfo | crate::core::types::EdgeTypeInfo |
| crate::core::types::metadata::PropertyDef | crate::core::types::PropertyDef |
| crate::query::context::AstContext | 删除 |
| crate::query::context::QueryContext | crate::query::QueryContext |
| crate::query::context::symbol::SymbolTable | crate::core::symbol::SymbolTable |
