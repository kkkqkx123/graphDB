# GraphDB 后端功能缺口分析

**文档版本**: v1.0  
**创建日期**: 2026-03-29  
**分析目标**: 支持前端 PRD 所需的后端功能

---

## 1. 分析范围

基于 `docs/frontend/prd_index.md` 和 `docs/frontend/feature_analysis.md` 定义的前端需求，分析当前 GraphDB 后端需要补充的功能。

### 1.1 前端功能需求总览

| 阶段 | 功能模块 | 优先级 | 后端依赖 |
|------|---------|--------|----------|
| 阶段 1 | 连接管理 | P0 | 认证、会话管理 |
| 阶段 2 | 查询控制台 | P0 | 查询执行、历史记录、收藏 |
| 阶段 3 | Schema - Space | P0 | Space CRUD |
| 阶段 4 | Schema - Tag/Edge | P0 | Tag/Edge CRUD |
| 阶段 5 | Schema - 索引 | P0 | 索引管理 |
| 阶段 6 | 图可视化 | P1 | 查询结果图数据格式 |
| 阶段 7 | 数据浏览 | P1 | 数据分页查询 |

---

## 2. 当前 GraphDB 后端能力

### 2.1 已实现功能

基于 `src/api/server/` 代码分析：

| 模块 | 功能 | 状态 | 文件位置 |
|------|------|------|----------|
| HTTP 服务器 | Axum 框架集成 | ✅ | `http/server.rs` |
| 路由 | RESTful API 路由 | ✅ | `http/router.rs` |
| 认证 | JWT Token 认证 | ✅ | `auth/`, `http/handlers/auth.rs` |
| 会话管理 | 会话创建/删除/查询 | ✅ | `session/`, `http/handlers/session.rs` |
| 查询执行 | Cypher 查询执行 | ✅ | `http/handlers/query.rs` |
| 事务管理 | 开始/提交/回滚 | ✅ | `http/handlers/transaction.rs` |
| Schema - Space | 创建/删除/获取 | ✅ | `http/handlers/schema.rs` |
| Schema - Tag | 创建 | ✅ | `http/handlers/schema.rs` |
| Schema - Edge | 创建 | ✅ | `http/handlers/schema.rs` |
| 批量操作 | 批量任务管理 | ✅ | `http/handlers/batch.rs` |
| 健康检查 | 服务状态检查 | ✅ | `http/handlers/health.rs` |
| 统计信息 | 会话/查询/数据库统计 | ✅ | `http/handlers/statistics.rs` |
| 配置管理 | 配置读取/更新 | ✅ | `http/handlers/config.rs` |

### 2.2 当前 API 端点清单

```
# 公开端点 (无需认证)
GET  /v1/health
POST /v1/auth/login
POST /v1/auth/logout

# 需要认证的端点
POST /v1/sessions
GET  /v1/sessions/:id
DELETE /v1/sessions/:id

POST /v1/query
POST /v1/query/validate

POST /v1/transactions
POST /v1/transactions/:id/commit
POST /v1/transactions/:id/rollback

POST /v1/batch
GET  /v1/batch/:id
DELETE /v1/batch/:id
POST /v1/batch/:id/items
POST /v1/batch/:id/execute
POST /v1/batch/:id/cancel

GET  /v1/statistics/sessions/:id
GET  /v1/statistics/queries
GET  /v1/statistics/database
GET  /v1/statistics/system

GET  /v1/config
PUT  /v1/config
GET  /v1/config/:section/:key
PUT  /v1/config/:section/:key
DELETE /v1/config/:section/:key

POST /v1/functions
GET  /v1/functions
GET  /v1/functions/:name
DELETE /v1/functions/:name

POST /v1/schema/spaces
GET  /v1/schema/spaces
GET  /v1/schema/spaces/:name
DELETE /v1/schema/spaces/:name

POST /v1/schema/spaces/:name/tags
GET  /v1/schema/spaces/:name/tags

POST /v1/schema/spaces/:name/edge-types
GET  /v1/schema/spaces/:name/edge-types
```

---

## 3. 功能缺口详细分析

### 3.1 阶段 1: 基础框架和连接管理

#### 缺口清单

| 功能 | 需求描述 | 当前状态 | 缺口等级 |
|------|---------|----------|----------|
| 连接健康检查 | 定期检测连接状态 | ⚠️ 有健康检查端点，但无连接特定检查 | 低 |
| 连接信息获取 | 获取当前连接的主机和用户信息 | ❌ 未实现 | 中 |

**需要补充的 API**:
```rust
// 获取当前连接信息
GET /v1/sessions/:id/info
Response: {
    "session_id": "uuid",
    "host": "localhost",
    "port": 7001,
    "username": "user",
    "connected_at": "2026-03-29T10:00:00Z",
    "space_id": 1,
    "space_name": "test"
}
```

### 3.2 阶段 2: 查询控制台

#### 缺口清单

| 功能 | 需求描述 | 当前状态 | 缺口等级 |
|------|---------|----------|----------|
| 查询历史记录 | 保存用户查询历史 | ❌ 未实现 | **高** |
| 查询收藏 | 收藏常用查询 | ❌ 未实现 | **高** |
| 批量查询执行 | 一次执行多条查询 | ⚠️ 有批量操作但用于数据导入 | **高** |
| 结构化结果返回 | JSON 格式结果 | ⚠️ 已实现基础格式，需完善 | 中 |
| 查询取消 | 中断长时间查询 | ❌ 未实现 | 中 |
| 查询验证 | 语法检查 | ✅ 已实现 | - |

**需要补充的 API**:

```rust
// ========== 查询历史 ==========
// 添加历史记录
POST /v1/queries/history
Request: {
    "query": "MATCH (n) RETURN n LIMIT 10",
    "execution_time_ms": 150,
    "rows_returned": 10,
    "success": true
}

// 获取历史记录列表
GET /v1/queries/history?limit=50&offset=0
Response: {
    "items": [
        {
            "id": "uuid",
            "query": "MATCH (n) RETURN n LIMIT 10",
            "executed_at": "2026-03-29T10:00:00Z",
            "execution_time_ms": 150,
            "rows_returned": 10,
            "success": true
        }
    ],
    "total": 100
}

// 删除单条历史
DELETE /v1/queries/history/:id

// 清空历史
DELETE /v1/queries/history

// ========== 查询收藏 ==========
// 添加收藏
POST /v1/queries/favorites
Request: {
    "name": "获取所有节点",
    "query": "MATCH (n) RETURN n LIMIT 10",
    "description": "可选描述"
}
Response: {
    "id": "uuid",
    "name": "获取所有节点",
    "query": "MATCH (n) RETURN n LIMIT 10",
    "created_at": "2026-03-29T10:00:00Z"
}

// 获取收藏列表
GET /v1/queries/favorites
Response: {
    "items": [
        {
            "id": "uuid",
            "name": "获取所有节点",
            "query": "MATCH (n) RETURN n LIMIT 10",
            "description": "",
            "created_at": "2026-03-29T10:00:00Z"
        }
    ]
}

// 更新收藏
PUT /v1/queries/favorites/:id
Request: {
    "name": "新名称",
    "query": "MATCH (n) RETURN n",
    "description": "更新描述"
}

// 删除收藏
DELETE /v1/queries/favorites/:id

// ========== 批量查询执行 ==========
// 执行多条查询
POST /v1/query/batch
Request: {
    "queries": [
        "MATCH (n) RETURN count(n)",
        "MATCH ()-[r]->() RETURN count(r)"
    ],
    "space": "test"  // 可选
}
Response: {
    "results": [
        {
            "query": "MATCH (n) RETURN count(n)",
            "success": true,
            "data": {...},
            "execution_time_ms": 100
        },
        {
            "query": "MATCH ()-[r]->() RETURN count(r)",
            "success": true,
            "data": {...},
            "execution_time_ms": 120
        }
    ]
}
```

### 3.3 阶段 3: Schema - Space 管理

#### 缺口清单

| 功能 | 需求描述 | 当前状态 | 缺口等级 |
|------|---------|----------|----------|
| Space 列表 | 获取所有 Space | ⚠️ 返回空列表 | **高** |
| Space 详情 | 获取 Space 配置信息 | ⚠️ 仅返回 ID | **高** |
| Space 统计 | 节点数、边数统计 | ❌ 未实现 | 中 |

**需要补充的 API**:

```rust
// 获取 Space 列表
GET /v1/schema/spaces
Response: {
    "spaces": [
        {
            "id": 1,
            "name": "test",
            "vid_type": "STRING",
            "partition_num": 100,
            "replica_factor": 1,
            "comment": "测试空间",
            "created_at": "2026-03-29T10:00:00Z"
        }
    ]
}

// 获取 Space 详情
GET /v1/schema/spaces/:name/details
Response: {
    "id": 1,
    "name": "test",
    "vid_type": "STRING",
    "partition_num": 100,
    "replica_factor": 1,
    "comment": "测试空间",
    "created_at": "2026-03-29T10:00:00Z",
    "statistics": {
        "tag_count": 5,
        "edge_type_count": 3,
        "index_count": 8,
        "estimated_vertex_count": 10000,
        "estimated_edge_count": 50000
    }
}

// 获取 Space 统计信息
GET /v1/schema/spaces/:name/statistics
Response: {
    "space_id": 1,
    "space_name": "test",
    "tags": [
        {
            "name": "Person",
            "count": 5000
        }
    ],
    "edge_types": [
        {
            "name": "KNOWS",
            "count": 20000
        }
    ],
    "total_vertices": 10000,
    "total_edges": 50000
}
```

### 3.4 阶段 4: Schema - Tag/Edge 管理

#### 缺口清单

| 功能 | 需求描述 | 当前状态 | 缺口等级 |
|------|---------|----------|----------|
| Tag 列表 | 获取 Space 下所有 Tag | ⚠️ 返回空列表 | **高** |
| Tag 详情 | 获取 Tag 属性定义 | ❌ 未实现 | **高** |
| Tag 修改 | 添加/删除属性 | ❌ 未实现 | **高** |
| Tag 删除 | 删除 Tag | ❌ 未实现 | **高** |
| Edge 列表 | 获取 Space 下所有 Edge | ⚠️ 返回空列表 | **高** |
| Edge 详情 | 获取 Edge 属性定义 | ❌ 未实现 | **高** |
| Edge 修改 | 添加/删除属性 | ❌ 未实现 | **高** |
| Edge 删除 | 删除 Edge | ❌ 未实现 | **高** |

**需要补充的 API**:

```rust
// ========== Tag 管理 ==========
// 获取 Tag 列表
GET /v1/schema/spaces/:name/tags
Response: {
    "tags": [
        {
            "id": 1,
            "name": "Person",
            "property_count": 3,
            "index_count": 2,
            "created_at": "2026-03-29T10:00:00Z"
        }
    ]
}

// 获取 Tag 详情
GET /v1/schema/spaces/:name/tags/:tag_name
Response: {
    "id": 1,
    "name": "Person",
    "properties": [
        {
            "name": "name",
            "data_type": "STRING",
            "nullable": false,
            "default_value": null
        },
        {
            "name": "age",
            "data_type": "INT",
            "nullable": true,
            "default_value": null
        }
    ],
    "indexes": [
        {
            "name": "idx_person_name",
            "type": "INDEX",
            "fields": ["name"]
        }
    ],
    "created_at": "2026-03-29T10:00:00Z"
}

// 修改 Tag (添加属性)
PUT /v1/schema/spaces/:name/tags/:tag_name
Request: {
    "add_properties": [
        {
            "name": "email",
            "data_type": "STRING",
            "nullable": true
        }
    ],
    "drop_properties": ["age"]
}

// 删除 Tag
DELETE /v1/schema/spaces/:name/tags/:tag_name

// ========== Edge Type 管理 ==========
// 获取 Edge Type 列表
GET /v1/schema/spaces/:name/edge-types
Response: {
    "edge_types": [
        {
            "id": 1,
            "name": "KNOWS",
            "property_count": 1,
            "index_count": 1,
            "created_at": "2026-03-29T10:00:00Z"
        }
    ]
}

// 获取 Edge Type 详情
GET /v1/schema/spaces/:name/edge-types/:edge_name
Response: {
    "id": 1,
    "name": "KNOWS",
    "properties": [
        {
            "name": "since",
            "data_type": "INT",
            "nullable": true,
            "default_value": null
        }
    ],
    "indexes": [
        {
            "name": "idx_knows_since",
            "type": "INDEX",
            "fields": ["since"]
        }
    ],
    "created_at": "2026-03-29T10:00:00Z"
}

// 修改 Edge Type
PUT /v1/schema/spaces/:name/edge-types/:edge_name
Request: {
    "add_properties": [...],
    "drop_properties": [...]
}

// 删除 Edge Type
DELETE /v1/schema/spaces/:name/edge-types/:edge_name
```

### 3.5 阶段 5: Schema - 索引管理

#### 缺口清单

| 功能 | 需求描述 | 当前状态 | 缺口等级 |
|------|---------|----------|----------|
| 索引列表 | 获取所有索引 | ❌ 未实现 | **高** |
| 索引创建 | 创建 Tag/Edge 索引 | ❌ 未实现 | **高** |
| 索引删除 | 删除索引 | ❌ 未实现 | **高** |
| 索引状态 | 查看索引构建状态 | ❌ 未实现 | 中 |
| 索引重建 | 重建索引 | ❌ 未实现 | 低 |

**需要补充的 API**:

```rust
// 获取索引列表
GET /v1/schema/spaces/:name/indexes
Response: {
    "indexes": [
        {
            "id": 1,
            "name": "idx_person_name",
            "type": "INDEX",  // INDEX, UNIQUE, FULLTEXT
            "entity_type": "TAG",  // TAG, EDGE
            "entity_name": "Person",
            "fields": ["name"],
            "status": "ACTIVE",  // ACTIVE, BUILDING, FAILED
            "created_at": "2026-03-29T10:00:00Z"
        }
    ]
}

// 创建索引
POST /v1/schema/spaces/:name/indexes
Request: {
    "name": "idx_person_age",
    "type": "INDEX",
    "entity_type": "TAG",
    "entity_name": "Person",
    "fields": ["age"],
    "comment": "可选描述"
}
Response: {
    "id": 2,
    "name": "idx_person_age",
    "status": "BUILDING"
}

// 获取索引详情
GET /v1/schema/spaces/:name/indexes/:index_name
Response: {
    "id": 1,
    "name": "idx_person_name",
    "type": "INDEX",
    "entity_type": "TAG",
    "entity_name": "Person",
    "fields": ["name"],
    "status": "ACTIVE",
    "progress": 100,  // 构建进度
    "created_at": "2026-03-29T10:00:00Z",
    "updated_at": "2026-03-29T10:00:00Z"
}

// 删除索引
DELETE /v1/schema/spaces/:name/indexes/:index_name

// 重建索引
POST /v1/schema/spaces/:name/indexes/:index_name/rebuild
Response: {
    "task_id": "uuid",
    "status": "STARTED"
}
```

### 3.6 阶段 6: 图可视化

#### 缺口清单

| 功能 | 需求描述 | 当前状态 | 缺口等级 |
|------|---------|----------|----------|
| 图数据格式 | 返回适合可视化的图数据 | ⚠️ 基础实现 | 中 |
| 节点详情 | 获取单个节点详细信息 | ❌ 未实现 | 中 |
| 边详情 | 获取单条边详细信息 | ❌ 未实现 | 中 |
| 邻居查询 | 获取节点的邻居节点 | ❌ 未实现 | 中 |

**需要补充的 API**:

```rust
// 获取节点详情
GET /v1/graph/vertices/:vid?space=:space_name
Response: {
    "vid": "123",
    "tags": [
        {
            "name": "Person",
            "properties": {
                "name": "Alice",
                "age": 30
            }
        }
    ]
}

// 获取边详情
GET /v1/graph/edges?src=:src_vid&dst=:dst_vid&type=:edge_type&rank=:rank&space=:space_name
Response: {
    "src": "123",
    "dst": "456",
    "type": "KNOWS",
    "rank": 0,
    "properties": {
        "since": 2020
    }
}

// 获取邻居节点
GET /v1/graph/vertices/:vid/neighbors?space=:space_name&direction=:direction
Response: {
    "vid": "123",
    "neighbors": [
        {
            "vid": "456",
            "type": "KNOWS",
            "direction": "OUT",
            "properties": {...}
        }
    ]
}
```

### 3.7 阶段 7: 数据浏览

#### 缺口清单

| 功能 | 需求描述 | 当前状态 | 缺口等级 |
|------|---------|----------|----------|
| Tag 数据浏览 | 按 Tag 查询节点 | ❌ 未实现 | **高** |
| Edge 数据浏览 | 按 Edge 查询边 | ❌ 未实现 | **高** |
| 数据筛选 | 属性条件筛选 | ❌ 未实现 | 中 |
| 分页查询 | 大数据集分页 | ❌ 未实现 | **高** |

**需要补充的 API**:

```rust
// 按 Tag 浏览数据
GET /v1/data/spaces/:name/tags/:tag_name/vertices?limit=20&offset=0&filter=:filter
Response: {
    "total": 1000,
    "items": [
        {
            "vid": "123",
            "properties": {
                "name": "Alice",
                "age": 30
            }
        }
    ]
}

// 按 Edge Type 浏览数据
GET /v1/data/spaces/:name/edge-types/:edge_name/edges?limit=20&offset=0
Response: {
    "total": 5000,
    "items": [
        {
            "src": "123",
            "dst": "456",
            "rank": 0,
            "properties": {
                "since": 2020
            }
        }
    ]
}
```

---

## 4. 功能缺口汇总

### 4.1 按优先级分类

#### P0 - 必须实现 (阻塞前端开发)

| 序号 | 功能模块 | 预估工作量 | 依赖 |
|------|---------|-----------|------|
| 1 | 查询历史 API | 2 天 | 元数据存储 |
| 2 | 查询收藏 API | 2 天 | 元数据存储 |
| 3 | 批量查询执行 | 1 天 | - |
| 4 | Space 列表/详情 | 1 天 | Core API |
| 5 | Tag/Edge 列表/详情 | 2 天 | Core API |
| 6 | Tag/Edge 修改/删除 | 2 天 | Core API |
| 7 | 索引管理 API | 3 天 | Core API |
| 8 | 数据浏览 API | 2 天 | - |

**P0 总计**: 约 15 天

#### P1 - 建议实现 (重要功能)

| 序号 | 功能模块 | 预估工作量 | 依赖 |
|------|---------|-----------|------|
| 1 | Space 统计信息 | 1 天 | 统计模块 |
| 2 | 图数据格式优化 | 1 天 | - |
| 3 | 节点/边详情查询 | 1 天 | - |
| 4 | 邻居节点查询 | 1 天 | - |

**P1 总计**: 约 4 天

#### P2 - 可选实现 (增强功能)

| 序号 | 功能模块 | 预估工作量 | 依赖 |
|------|---------|-----------|------|
| 1 | 索引重建 | 1 天 | - |
| 2 | 查询取消 | 2 天 | 异步查询架构 |

**P2 总计**: 约 3 天

### 4.2 按模块分类

```
元数据管理模块 (新增)
├── 查询历史 CRUD
├── 查询收藏 CRUD
└── 存储: SQLite/JSON 文件

Schema 管理模块 (扩展)
├── Space 列表/详情/统计
├── Tag 列表/详情/修改/删除
├── Edge 列表/详情/修改/删除
└── 索引 列表/创建/删除/状态

数据浏览模块 (新增)
├── Tag 数据分页查询
├── Edge 数据分页查询
└── 属性筛选

图数据模块 (新增)
├── 节点详情
├── 边详情
└── 邻居查询
```

---

## 5. 技术实现建议

### 5.1 元数据存储方案

对于查询历史和收藏等前端专属功能，需要选择存储方案：

| 方案 | 优点 | 缺点 | 推荐度 |
|------|------|------|--------|
| SQLite | 轻量、SQL支持 | 额外依赖 | ⭐⭐⭐⭐ |
| JSON 文件 | 简单、无依赖 | 并发性能差 | ⭐⭐⭐ |
| 内存存储 | 最简单 | 重启丢失 | ⭐⭐ |

**建议**: 使用 SQLite 存储元数据，与 Nebula Studio 保持一致。

### 5.2 API 版本管理

建议新增 API 使用 `/v1/` 前缀，与现有 API 保持一致。

### 5.3 错误处理

统一错误响应格式：
```json
{
    "success": false,
    "error": {
        "code": "SCHEMA_TAG_NOT_FOUND",
        "message": "Tag 'Person' not found in space 'test'",
        "details": null
    }
}
```

---

## 6. 实施建议

### 6.1 实施顺序

```
第一阶段 (P0核心)
├── 1. 元数据存储模块搭建
├── 2. 查询历史/收藏 API
├── 3. 批量查询执行
└── 4. Schema 列表/详情 API

第二阶段 (P0完善)
├── 5. Tag/Edge 修改删除
├── 6. 索引管理 API
└── 7. 数据浏览 API

第三阶段 (P1增强)
├── 8. 统计信息 API
└── 9. 图数据查询 API
```

### 6.2 与前端配合

- 后端优先实现 P0 功能
- 前端可以 mock P1/P2 功能进行开发
- 建议前后端并行开发，通过 API 文档约定接口
