# 全文检索扩展文档

本目录包含 GraphDB 全文检索功能的 gRPC 服务集成设计方案。

## 设计原则

- **保持轻量**: GraphDB 核心保持简洁，全文检索作为可选扩展
- **服务解耦**: 利用现有 ref/bm25 和 ref/inversearch 服务，通过 gRPC 通信
- **本地优先**: localhost 通信开销可忽略，适合本地部署场景
- **存储隔离**: 全文索引与图数据完全分离，避免存储冲突

## 文档列表

| 文档 | 说明 |
|------|------|
| [fulltext_integration_design.md](./fulltext_integration_design.md) | 全文检索服务集成设计方案 |
| [fulltext_implementation_plan.md](./fulltext_implementation_plan.md) | 详细实现计划 |
| [fulltext_api_reference.md](./fulltext_api_reference.md) | API 参考文档 |

## 架构概览

```
GraphDB 核心进程          全文检索服务进程 (ref/bm25 或 ref/inversearch)
┌─────────────────┐       ┌─────────────────────────────┐
│  gRPC 客户端     │◄────►│  gRPC 服务端 (已存在)        │
│  (需实现)        │       │  - IndexDocument            │
└─────────────────┘       │  - Search                   │
         │                │  - DeleteDocument           │
         │ 数据同步        └─────────────────────────────┘
         ▼
┌─────────────────┐
│  Redb 存储      │
│  (图数据)       │
└─────────────────┘
```

## 参考项目

- `ref/bm25/` - 基于 Tantivy 的 BM25 服务
- `ref/inversearch/` - 自定义倒排索引服务
