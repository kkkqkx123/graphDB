# GraphDB 后端分析文档

**文档版本**: v1.0  
**创建日期**: 2026-03-29  
**目标读者**: 后端开发人员、架构师

---

## 文档列表

| 文档 | 描述 | 优先级 |
|------|------|--------|
| [nebula_studio_backend_analysis.md](./nebula_studio_backend_analysis.md) | Nebula Studio 后端架构和功能分析 | 参考 |
| [backend_gap_analysis.md](./backend_gap_analysis.md) | GraphDB 后端功能缺口详细分析 | **必读** |
| [architecture_recommendation.md](./architecture_recommendation.md) | 后端架构方案建议 | **必读** |

---

## 快速导航

### 对于前端开发人员

如果你需要了解后端 API 的设计，请阅读：
- [backend_gap_analysis.md](./backend_gap_analysis.md) - 查看需要实现的 API 列表

### 对于后端开发人员

如果你需要开始实现后端功能，请按顺序阅读：
1. [architecture_recommendation.md](./architecture_recommendation.md) - 了解架构方案
2. [backend_gap_analysis.md](./backend_gap_analysis.md) - 查看详细的功能缺口和 API 设计
3. [nebula_studio_backend_analysis.md](./nebula_studio_backend_analysis.md) - 参考 Nebula Studio 的实现

---

## 核心结论

### 1. 架构方案

**推荐方案**: 在 `src/api/web/` 下创建新的 Web 管理模块（混合方案）

```
src/api/
├── server/          # 核心 HTTP API (已有)
└── web/             # Web 管理界面 API (新增)
    ├── handlers/    # HTTP 处理器
    ├── services/    # 业务逻辑
    ├── models/      # 数据模型
    └── storage/     # 元数据存储
```

### 2. 需要实现的功能

#### P0 - 必须实现 (约 15 天)

| 功能 | 预估工作量 | 对应前端阶段 |
|------|-----------|-------------|
| 查询历史 API | 2 天 | 阶段 2 |
| 查询收藏 API | 2 天 | 阶段 2 |
| 批量查询执行 | 1 天 | 阶段 2 |
| Space 列表/详情/统计 | 2 天 | 阶段 3 |
| Tag/Edge 列表/详情 | 2 天 | 阶段 4 |
| Tag/Edge 修改/删除 | 2 天 | 阶段 4 |
| 索引管理 API | 3 天 | 阶段 5 |
| 数据浏览 API | 1 天 | 阶段 7 |

#### P1 - 建议实现 (约 4 天)

| 功能 | 预估工作量 | 对应前端阶段 |
|------|-----------|-------------|
| 图数据格式优化 | 1 天 | 阶段 6 |
| 节点/边详情查询 | 1 天 | 阶段 6 |
| 邻居节点查询 | 1 天 | 阶段 6 |
| 统计信息 API | 1 天 | 阶段 3-7 |

### 3. API 路径规划

```
# 核心 API (已有)
/v1/health
/v1/auth/*
/v1/sessions/*
/v1/query
/v1/transactions/*
/v1/batch/*
/v1/schema/spaces/* (基础 CRUD)

# Web 管理 API (新增)
/web/v1/queries/history
/web/v1/queries/favorites
/web/v1/schema/spaces/*/tags
/web/v1/schema/spaces/*/edge-types
/web/v1/schema/spaces/*/indexes
/web/v1/data/*
/web/v1/graph/*
```

---

## 实施路线图

```
Week 1: 基础搭建
├── Day 1-2: 创建 web 模块目录结构
├── Day 3-4: 实现元数据存储 (SQLite)
└── Day 5: 实现查询历史 API

Week 2: 查询功能
├── Day 1-2: 实现查询收藏 API
├── Day 3: 实现批量查询执行
└── Day 4-5: Schema 列表/详情 API

Week 3: Schema 管理
├── Day 1-2: Tag/Edge 列表/详情
├── Day 3-4: Tag/Edge 修改/删除
└── Day 5: 索引管理 API

Week 4: 数据浏览与优化
├── Day 1-2: 数据浏览 API
├── Day 3: 图数据查询 API
└── Day 4-5: 测试与优化
```

---

## 技术选型

| 组件 | 选型 | 理由 |
|------|------|------|
| Web 框架 | Axum | 与现有代码一致 |
| 元数据存储 | SQLite | 轻量、可靠 |
| ORM | sqlx | 异步支持、类型安全 |
| 序列化 | serde | Rust 标准 |
| 错误处理 | thiserror | 标准做法 |

## 条件编译说明

Web 管理功能作为 `server` feature 的一部分，**不需要额外的 feature flag**。

```toml
# Cargo.toml
[features]
default = ["redb", "embedded", "server", "c-api"]
server = [
    "dep:axum",
    "dep:tower",
    "dep:tower-http",
    "dep:http",
    "dep:sqlx"  # Web 管理需要的 SQLite 支持
]
```

### 编译命令

```bash
# 完整功能（包含 Web 管理）
cargo build --features server

# 仅嵌入式（不包含 Web 管理）
cargo build --features embedded

# 运行服务端（包含 Web 管理）
cargo run --features server
```

### 代码结构

```
src/api/
├── mod.rs
├── core/
├── embedded/
└── server/           # #[cfg(feature = "server")]
    ├── mod.rs
    ├── http/
    ├── auth/
    └── web/          # Web 管理功能（作为 server 子模块）
        ├── mod.rs
        ├── handlers/
        ├── services/
        └── storage/
```

---

## 参考文档

- [前端 PRD 索引](../frontend/prd_index.md)
- [前端功能分析](../frontend/feature_analysis.md)
- [Server API 实现计划](../api/server/api_implementation_plan.md)
