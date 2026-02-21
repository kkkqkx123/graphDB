# Query Context 重构设计方案

## 一、问题诊断

### 1.1 核心问题：类型定义混乱

当前系统存在严重的类型定义问题：

1. **metadata.rs过大**：567行混合了SpaceInfo、TagInfo、EdgeTypeInfo、PropertyDef、UserInfo、InsertVertexInfo等完全不相关的类型
2. **SpaceInfo重复定义**：3个不同版本存在于不同模块
3. **类型归属错误**：数据操作类型（InsertVertexInfo）和用户管理类型（UserInfo）定义在core层
4. **缺乏原子化**：相关类型应该分组定义，而不是混合在一个文件

### 1.2 SpaceInfo重复定义

```
SpaceInfo定义（3个版本）:
├── core::types::metadata::SpaceInfo      (567行文件中的定义)
├── query::context::ast::base::SpaceInfo  (简化版)
└── api::session::client_session::SpaceInfo (会话版)
```

### 1.3 各层Context使用情况

| 层级 | 当前Context | 问题 | 目标 |
|------|------------|------|------|
| Parser | ParseContext | ✅ 独立 | 保持 |
| Validator | AstContext | ❌ 多余中间层 | 删除，直接使用QueryContext |
| Planner | AstContext + QueryContext | ❌ 依赖AstContext | 直接使用QueryContext |
| Optimizer | OptContext | ⚠️ 克隆QueryContext | 使用Arc共享 |
| Executor | ExecutionContext | ✅ 独立 | 保持 |

---

## 二、核心设计原则

### 2.1 原子类型拆分原则

**每个文件只定义一个核心概念**，将metadata.rs拆分为原子类型文件：

```
core/types/
├── space.rs            # SpaceInfo（图空间）
├── tag.rs              # TagInfo（标签）
├── edge.rs             # EdgeTypeInfo（边类型）
├── property.rs         # PropertyDef（属性定义）
├── metadata_version.rs # 版本管理类型
├── expression.rs       # 表达式类型（已有）
├── operators.rs        # 操作符类型（已有）
├── graph_schema.rs     # 图模式类型（已有）
├── query.rs            # QueryType（已有）
├── variable.rs         # VariableInfo（已有）
└── span.rs             # 位置类型（已有）
```

### 2.2 类型归属原则

| 类型类别 | 归属层 | 说明 |
|---------|--------|------|
| 基础Schema类型（SpaceInfo, TagInfo等） | core::types | 原子类型定义 |
| 数据操作类型（InsertVertexInfo等） | storage | 存储层内部使用 |
| 用户管理类型（UserInfo等） | api::session | 会话层使用 |
| 查询处理类型（QueryContext等） | query | 查询层使用 |

### 2.3 Context设计原则

1. **职责单一**：每个Context只包含本层需要的数据
2. **共享而非克隆**：使用Arc在层间共享Context
3. **无中间层**：删除AstContext，Validator直接使用QueryContext
4. **模块自包含**：Context定义在各自模块内

---

## 三、类型系统重构

### 3.1 metadata.rs原子化拆分

将567行的metadata.rs拆分为原子类型文件：

#### space.rs - 图空间类型
```rust
//! 图空间基础类型

use crate::core::types::DataType;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct SpaceInfo {
    pub space_id: u64,
    pub space_name: String,
    pub vid_type: DataType,
    pub comment: Option<String>,
}
```

#### tag.rs - 标签类型
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
```

#### edge.rs - 边类型
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
```

#### property.rs - 属性定义
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
```

#### metadata_version.rs - 版本管理
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
```

### 3.2 SpaceInfo统一

**唯一正确定义**：`core::types::space::SpaceInfo`

**删除**：
- `query::context::ast::base::SpaceInfo`
- `api::session::client_session::SpaceInfo`
- `core::types::space::QuerySpaceInfo`（刚创建，不需要）

### 3.3 非基础类型迁移

| 类型 | 从 | 到 |
|------|-----|-----|
| InsertVertexInfo | core::types::metadata | storage::types |
| InsertEdgeInfo | core::types::metadata | storage::types |
| UpdateInfo | core::types::metadata | storage::types |
| UserInfo | core::types::metadata | api::session::types |
| PasswordInfo | core::types::metadata | api::session::types |
| ClusterInfo | core::types::metadata | 删除（分布式遗留） |
| CharsetInfo | core::types::metadata | 删除（分布式遗留） |

---

## 四、Context架构重构

### 4.1 删除AstContext

**当前**：Validator → AstContext → QueryContext
**新设计**：Validator → QueryContext

AstContext是多余的中间层，直接删除。

### 4.2 QueryContext设计

```rust
// src/query/query_context.rs
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
```

### 4.3 OptContext修改

```rust
// src/query/optimizer/plan/context.rs
use std::sync::Arc;

pub struct OptContext {
    qctx: Arc<QueryContext>,  // 共享Arc，不克隆
    changed: bool,
    // ...
}

impl OptContext {
    pub fn new(qctx: Arc<QueryContext>) -> Self {
        Self { qctx, changed: false, ... }
    }
}
```

### 4.4 调用链

```
1. 接收查询请求
   ↓
2. api::session创建RequestContext
   ↓
3. QueryContext::new(Arc<RequestContext>) -> Arc<QueryContext>
   ↓
4. Parser::parse(query_string) -> Stmt
   ↓
5. Validator::validate(&stmt, Arc<QueryContext>)
   ↓
6. Planner::plan(&stmt, Arc<QueryContext>)
   ↓
7. Optimizer::optimize(plan, Arc<QueryContext>)
   ↓
8. Executor::execute(&plan, &mut ExecutionContext)
```

---

## 五、目录结构

### 5.1 最终目录结构

```
src/
├── core/
│   ├── types/
│   │   ├── mod.rs              # 统一导出
│   │   ├── space.rs            # SpaceInfo
│   │   ├── tag.rs              # TagInfo
│   │   ├── edge.rs             # EdgeTypeInfo
│   │   ├── property.rs         # PropertyDef
│   │   ├── metadata_version.rs # MetadataVersion等
│   │   ├── expression.rs       # Expression
│   │   ├── operators.rs        # 操作符
│   │   ├── graph_schema.rs     # 图模式
│   │   ├── query.rs            # QueryType
│   │   ├── variable.rs         # VariableInfo
│   │   └── span.rs             # 位置
│   └── symbol/
│       ├── mod.rs
│       └── symbol_table.rs     # SymbolTable
│
├── api/
│   └── session/
│       ├── mod.rs
│       ├── request_context.rs  # RequestContext
│       ├── client_session.rs   # ClientSession（删除SpaceInfo定义）
│       └── types.rs            # UserInfo, PasswordInfo等
│
├── query/
│   ├── mod.rs
│   ├── query_context.rs        # QueryContext
│   ├── parser/
│   │   └── parser/
│   │       └── parse_context.rs
│   ├── validator/
│   │   ├── mod.rs
│   │   ├── validator_trait.rs  # StatementValidator（修改参数）
│   │   └── ...
│   ├── planner/
│   │   ├── mod.rs
│   │   ├── planner.rs          # Planner（修改参数）
│   │   └── plan/
│   ├── optimizer/
│   │   ├── optimizer_impl.rs   # Optimizer（修改参数）
│   │   └── plan/
│   │       └── context.rs      # OptContext（修改）
│   └── executor/
│       └── context.rs          # ExecutionContext
│
└── storage/
    ├── mod.rs
    ├── types.rs                # InsertVertexInfo等
    └── metadata/
        ├── mod.rs
        ├── schema_manager.rs
        └── types.rs            # SchemaVersion等
```

### 5.2 删除的文件

```
删除：
- src/core/types/metadata.rs（拆分为原子类型文件）
- src/core/types/space.rs（QuerySpaceInfo，不需要）
- src/query/context/（整个目录）
  - ast/base.rs（AstContext, SpaceInfo）
  - ast/common.rs
  - ast/mod.rs
  - request_context.rs（已迁移）
  - runtime_context.rs
  - symbol/（已迁移）
  - components.rs
  - mod.rs
```

---

## 六、接口变更

### 6.1 StatementValidator Trait

```rust
// 当前
pub trait StatementValidator {
    fn validate(&mut self, ast: &mut AstContext) -> Result<...>;
}

// 新
pub trait StatementValidator {
    fn validate(&mut self, stmt: &Stmt, qctx: Arc<QueryContext>) -> Result<...>;
}
```

### 6.2 Optimizer

```rust
// 当前
pub fn optimize(&mut self, plan: ExecutionPlan, query_context: &mut QueryContext) -> Result<...>;

// 新
pub fn optimize(&mut self, plan: ExecutionPlan, qctx: Arc<QueryContext>) -> Result<...>;
```

### 6.3 Planner

```rust
// 当前
pub fn plan(&self, ast: &AstContext, qctx: &QueryContext) -> Result<...>;

// 新
pub fn plan(&self, stmt: &Stmt, qctx: Arc<QueryContext>) -> Result<...>;
```

---

## 七、验证标准

- [ ] metadata.rs已拆分为原子类型文件（space.rs, tag.rs, edge.rs, property.rs等）
- [ ] SpaceInfo只有一个定义（core::types::space::SpaceInfo）
- [ ] 没有重复的类型定义
- [ ] AstContext已完全删除
- [ ] Validator直接使用QueryContext
- [ ] QueryContext使用Arc<RequestContext>
- [ ] OptContext使用Arc<QueryContext>而非克隆
- [ ] src/query/context/目录已删除
- [ ] 没有循环依赖
- [ ] cargo check --lib通过
- [ ] cargo test通过
