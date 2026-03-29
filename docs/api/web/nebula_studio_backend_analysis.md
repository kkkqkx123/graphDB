# Nebula Studio 后端逻辑分析

**文档版本**: v1.0  
**创建日期**: 2026-03-29  
**分析对象**: ref/nebula-studio-3.10.0/server

---

## 1. 架构概述

Nebula Studio 后端采用 Go 语言开发，基于 go-zero 框架构建，采用典型的分层架构设计。

### 1.1 技术栈

- **框架**: go-zero (微服务框架)
- **数据库**: SQLite (本地元数据存储)
- **ORM**: GORM
- **WebSocket**: 内置支持
- **认证**: JWT + Cookie

### 1.2 目录结构

```
server/
├── api/studio/
│   ├── cmd/                    # 命令入口
│   ├── etc/                    # 配置文件
│   ├── internal/
│   │   ├── config/             # 配置定义
│   │   ├── handler/            # HTTP 处理器 (Controller)
│   │   ├── logic/              # 业务逻辑层
│   │   ├── model/              # 数据模型 (ORM)
│   │   ├── service/            # 服务层
│   │   ├── svc/                # 服务上下文
│   │   └── types/              # 请求/响应类型定义
│   ├── pkg/                    # 公共包
│   │   ├── auth/               # 认证相关
│   │   ├── client/             # Nebula 客户端
│   │   ├── db/                 # 数据库连接
│   │   ├── ecode/              # 错误码
│   │   ├── filestore/          # 文件存储
│   │   ├── llm/                # LLM 集成
│   │   ├── utils/              # 工具函数
│   │   └── ws/                 # WebSocket
│   └── restapi/                # API 定义文件 (.api)
├── go.mod
└── Makefile
```

---

## 2. 核心模块分析

### 2.1 连接管理模块 (gateway)

**文件位置**: `internal/service/gateway.go`, `internal/handler/gateway/`

**功能职责**:
- 数据库连接建立与断开
- NGQL 查询执行（单条和批量）
- 会话管理

**核心接口**:
```go
type GatewayService interface {
    ExecNGQL(request *types.ExecNGQLParams) (*types.AnyResponse, error)
    BatchExecNGQL(request *types.BatchExecNGQLParams) (*types.AnyResponse, error)
    ConnectDB(request *types.ConnectDBParams) error
    DisconnectDB() (*types.AnyResponse, error)
}
```

**API 端点**:
| 方法 | 路径 | 功能 |
|------|------|------|
| POST | /api-nebula/db/connect | 建立连接 |
| POST | /api-nebula/db/disconnect | 断开连接 |
| POST | /api-nebula/db/exec | 执行单条查询 |
| POST | /api-nebula/db/batchExec | 批量执行查询 |

**关键实现细节**:
- 使用连接池管理 Nebula 客户端连接
- 通过 NSID (Session ID) 标识客户端会话
- 支持 Cookie 存储 JWT Token
- 客户端连接有超时回收机制 (SessionExpiredDuration = 3600s)

### 2.2 收藏管理模块 (favorite)

**文件位置**: `internal/service/favorite.go`, `internal/handler/favorite/`

**功能职责**:
- 查询语句收藏管理
- 收藏列表查询
- 收藏删除

**数据模型**:
```go
type Favorite struct {
    BID        string    `gorm:"column:b_id"`
    Host       string    `gorm:"column:host"`
    Username   string    `gorm:"column:username"`
    Content    string    `gorm:"column:content"`  // 收藏的查询语句
    CreateTime time.Time
}
```

**API 端点**:
| 方法 | 路径 | 功能 |
|------|------|------|
| POST | /api/favorites | 添加收藏 |
| GET | /api/favorites | 获取收藏列表 |
| DELETE | /api/favorites/:id | 删除收藏 |
| DELETE | /api/favorites | 清空收藏 |

**特点**:
- 收藏数据按 Host + Username 隔离
- 使用 SQLite 本地存储
- 支持 BID (业务ID) 作为主键

### 2.3 Schema 快照模块 (schema)

**文件位置**: `internal/service/schema.go`, `internal/handler/schema/`

**功能职责**:
- Schema 可视化快照保存
- Schema 快照查询

**数据模型**:
```go
type SchemaSnapshot struct {
    BID        string
    Host       string
    Username   string
    Space      string
    Snapshot   string  // JSON 格式的 Schema 信息
    CreateTime time.Time
    UpdateTime time.Time
}
```

**API 端点**:
| 方法 | 路径 | 功能 |
|------|------|------|
| GET | /api/schema/snapshot | 获取 Schema 快照 |
| POST | /api/schema/snapshot | 更新 Schema 快照 |

### 2.4 文件管理模块 (file)

**文件位置**: `internal/service/file.go`, `internal/handler/file/`

**功能职责**:
- 文件上传
- 文件列表查询
- 文件删除
- 文件配置更新 (CSV 解析配置)

**数据模型**:
```go
type File struct {
    BID        string
    Name       string
    Size       int64
    Sample     string  // 文件样本内容
    WithHeader bool    // 是否有表头
    Delimiter  string  // 分隔符
    CreateTime time.Time
}
```

**API 端点**:
| 方法 | 路径 | 功能 |
|------|------|------|
| PUT | /api/files | 上传文件 |
| GET | /api/files | 获取文件列表 |
| DELETE | /api/files | 删除文件 |
| POST | /api/files/update | 更新文件配置 |

### 2.5 导入任务模块 (importtask)

**文件位置**: `internal/service/import.go`, `internal/handler/importtask/`

**功能职责**:
- 数据导入任务创建
- 导入任务管理
- 导入日志查看
- 任务草稿保存

**数据模型**:
```go
type TaskInfo struct {
    BID        string
    Name       string
    Space      string
    Status     string  // pending, running, completed, failed
    Config     string  // 导入配置 JSON
    RawConfig  string
    LogNames   string
    CreateTime time.Time
    UpdateTime time.Time
}
```

**API 端点**:
| 方法 | 路径 | 功能 |
|------|------|------|
| POST | /api/import-tasks | 创建导入任务 |
| POST | /api/import-tasks/draft | 创建任务草稿 |
| GET | /api/import-tasks | 获取任务列表 |
| GET | /api/import-tasks/:id | 获取任务详情 |
| DELETE | /api/import-tasks/:id | 删除任务 |
| POST | /api/import-tasks/:id/stop | 停止任务 |

### 2.6 数据源模块 (datasource)

**文件位置**: `internal/service/datasource.go`, `internal/handler/datasource/`

**功能职责**:
- 外部数据源配置 (S3, SFTP, OSS)
- 数据源文件浏览
- 数据源文件预览

**支持的数据源类型**:
- S3 (AWS S3 兼容)
- SFTP
- OSS (阿里云)
- Local (本地文件)

### 2.7 Sketch 模块 (sketches)

**文件位置**: `internal/service/sketch.go`, `internal/handler/sketches/`

**功能职责**:
- 可视化建模草图保存
- 草图列表管理

---

## 3. 公共组件分析

### 3.1 认证模块 (pkg/auth)

**核心功能**:
- JWT Token 生成与解析
- 连接参数解析和验证
- Cookie 管理

**关键结构**:
```go
type AuthData struct {
    NSID     string  // 会话ID
    Address  string  // 主机地址
    Port     int     // 端口
    Username string  // 用户名
}
```

### 3.2 Nebula 客户端 (pkg/client)

**核心功能**:
- 连接池管理
- 会话池管理
- 查询执行

**关键实现**:
```go
type Client struct {
    graphClient    *nebula.ConnectionPool
    RequestChannel chan ChannelRequest
    CloseChannel   chan bool
    sessionPool    *SessionPool
    parameterMap   ParameterMap
}
```

**特性**:
- 支持参数化查询
- 连接复用和回收
- 自动会话管理

### 3.3 WebSocket 支持 (pkg/ws)

**功能**:
- 实时日志推送
- 查询结果流式返回
- LLM 聊天功能

**中间件**:
- ngql: 查询执行
- batch_ngql: 批量查询
- llm: AI 聊天
- logger: 日志推送

---

## 4. 数据流分析

### 4.1 查询执行流程

```
1. 用户 -> POST /api-nebula/db/exec
2. Handler (gateway/execngqlhandler.go)
3. Logic (gateway/execngqllogic.go)
4. Service (gateway.go::ExecNGQL)
5. Client (pkg/client/client.go::Execute)
6. Nebula Graph 数据库
7. 返回结果 -> Service -> Logic -> Handler -> 用户
```

### 4.2 连接建立流程

```
1. 用户 -> POST /api-nebula/db/connect
2. Handler (gateway/connecthandler.go)
3. Service (gateway.go::ConnectDB)
4. Auth (pkg/auth/authorize.go::ParseConnectDBParams)
   - 验证连接参数
   - 创建 Nebula 客户端
   - 生成 JWT Token
5. 设置 Cookie
6. 返回成功响应
```

---

## 5. 与 GraphDB 的对比分析

### 5.1 功能对比

| 功能模块 | Nebula Studio | GraphDB 当前状态 | 差异分析 |
|---------|---------------|-----------------|----------|
| 连接管理 | ✅ 完整实现 | ✅ 已实现 | GraphDB 已实现基本连接管理 |
| 查询执行 | ✅ 单条+批量 | ✅ 已实现 | 功能对等 |
| 收藏管理 | ✅ SQLite存储 | ❌ 未实现 | GraphDB 需要新增 |
| Schema快照 | ✅ SQLite存储 | ❌ 未实现 | GraphDB 需要新增 |
| 文件管理 | ✅ 本地+S3/SFTP | ❌ 未实现 | GraphDB 不需要复杂文件管理 |
| 导入任务 | ✅ 完整任务流 | ⚠️ 部分实现 | GraphDB 有 batch API |
| 数据源 | ✅ 多数据源 | ❌ 未实现 | GraphDB 不需要 |
| Sketch | ✅ 可视化建模 | ❌ 未实现 | GraphDB 不需要 |
| LLM集成 | ✅ AI功能 | ❌ 未实现 | GraphDB 不需要 |
| 查询历史 | ❌ 未实现 | ❌ 未实现 | 两者都需要 |

### 5.2 架构差异

| 方面 | Nebula Studio | GraphDB |
|------|---------------|---------|
| 语言 | Go | Rust |
| 框架 | go-zero | Axum |
| 元数据存储 | SQLite | 待定 |
| 目标数据库 | Nebula Graph | GraphDB (自身) |
| 部署方式 | 独立服务 | 内嵌或独立 |

---

## 6. 关键设计模式

### 6.1 分层架构

```
Handler (HTTP入口)
    ↓
Logic (业务逻辑编排)
    ↓
Service (业务服务)
    ↓
Model/Pkg (数据访问/外部调用)
```

### 6.2 服务上下文模式

```go
type ServiceContext struct {
    Config          config.Config
    IDGenerator     idx.Generator
    ResponseHandler response.Handler
}
```

### 6.3 错误处理模式

- 使用错误码 (ecode) 统一管理错误
- 支持错误链和上下文信息
- HTTP 响应统一封装

---

## 7. 总结

Nebula Studio 后端设计清晰，采用经典的分层架构，职责分离明确。对于 GraphDB 前端项目，可以借鉴以下设计：

1. **收藏管理**: 需要实现类似的收藏功能，存储用户收藏的查询语句
2. **查询历史**: 需要实现查询历史记录功能
3. **Schema 快照**: 可选实现，用于 Schema 可视化
4. **认证机制**: 参考 JWT + Cookie 方案
5. **错误处理**: 参考统一的错误码设计

不需要实现的功能：
- 复杂的数据源管理 (S3, SFTP)
- 可视化建模 (Sketch)
- LLM 集成
- 复杂的导入任务管理
