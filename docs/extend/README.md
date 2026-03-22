# 全文检索扩展文档

本目录包含 GraphDB 全文检索功能的设计和实现文档。

## 文档列表

| 文档 | 说明 |
|------|------|
| [architecture_comparison.md](./architecture_comparison.md) | 架构方案对比分析，内嵌式 vs gRPC 服务的详细对比 |
| [fulltext_search_design.md](./fulltext_search_design.md) | 全文检索功能设计方案，包含架构设计、核心类型定义和实现细节 |
| [fulltext_implementation_plan.md](./fulltext_implementation_plan.md) | 详细实现计划，包含任务分解、时间安排和里程碑 |
| [fulltext_api_reference.md](./fulltext_api_reference.md) | API 参考文档，包含 SQL 语法和 Rust API 接口 |

## 快速开始

### 1. 阅读设计方案

首先阅读 [fulltext_search_design.md](./fulltext_search_design.md) 了解整体设计方案：
- 参考项目分析（BM25、Inversearch）
- 架构设计
- 核心类型定义
- Tantivy 实现方案
- 内置实现方案

### 2. 查看实现计划

然后查看 [fulltext_implementation_plan.md](./fulltext_implementation_plan.md) 了解具体实施步骤：
- 阶段划分
- 任务分解
- 时间安排
- 验收标准

### 3. 参考 API 文档

开发时参考 [fulltext_api_reference.md](./fulltext_api_reference.md)：
- SQL 语法扩展
- Rust API 接口
- 配置选项
- 使用示例

## 参考项目

- `ref/bm25/` - 基于 Tantivy 的 BM25 实现
- `ref/inversearch/` - 自定义倒排索引实现

## 相关模块

- `src/storage/` - 存储层
- `src/query/` - 查询层
- `src/core/types/` - 核心类型定义
