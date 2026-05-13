# 序列化使用分析报告

> 分析日期: 2026-05-13

## 概述

对 GraphDB 项目中 serde/oxicode 序列化的全面分析。项目共有 **527+ 个 serde derive** 和 **649+ 个序列化调用点**。核心存储引擎避免了 serde 存储实际记录数据，但存在若干问题。

## 序列化分类

### A. 网络/API 通信 (JSON) — ✅ 最佳实践

| 位置 | 类型 |
|------|------|
| `src/api/server/http/handlers/` | HTTP 请求/响应类型 |
| `src/api/server/web/handlers/` | Web API 端点 |
| `src/api/embedded/result.rs` | Embedded API 输出 |
| `crates/vector-client/` | OpenAI 兼容 API 客户端 |
| `graphdb-cli/` | CLI 输出/导入导出 |

serde JSON 用于 HTTP/wire 协议是 Rust 生态标准实践。

### B. 配置文件 (TOML) — ✅ 最佳实践

| 位置 | 类型 |
|------|------|
| `src/config/mod.rs` | 主配置 |
| `src/search/config.rs` | 搜索配置 |
| `src/query/cache/config.rs` | 查询缓存配置 |
| `crates/bm25/` | BM25 配置 |
| `crates/inversearch/` | 反向索引配置 |
| `crates/vector-client/` | 向量客户端配置 |

### C. 内存临时序列化 — ✅ 最佳实践

| 位置 | 类型 |
|------|------|
| `src/transaction/types.rs` | 事务状态（内存中） |
| `src/query/executor/` | 查询执行器内部结构 |
| `src/core/types/expr/` | 表达式表示 |
| `src/sync/` | 同步消息类型 |
| 各处双 derive (`serde+oxicode`) 类型 | 内存表示 |

### D. 磁盘/持久化存储 — ⚠️ 存在问题的

#### D1. WAL 恢复中属性值使用 JSON 反序列化

- **文件**: `src/storage/engine/property_graph/recovery.rs:91,115`
- **严重程度**: 🔴 高
- **问题**: `serde_json::from_slice(value)` 用于解码 WAL 中的属性值，但事务层使用自定义二进制编码 (`transaction/codec.rs`)。
- **影响**: 
  - 如果 UpdateVertexProp/UpdateEdgeProp WAL 条目被写入，恢复时会用错误格式解码
  - 失败时静默降级为 `Value::Empty` 导致数据静默丢失
- **修复**: 改用 `bytes_to_value` + 返回错误而非静默降级

#### D2. WAL 记录类型有无用的 serde derives

- **文件**: `src/transaction/wal/types.rs:640-703`
- **严重程度**: 🟠 中
- **涉及类型**: `InsertVertexRedo`, `InsertEdgeRedo`, `UpdateVertexPropRedo`, `UpdateEdgePropRedo`, `CreateVertexTypeRedo`, `CreateEdgeTypeRedo`, `DeleteVertexRedo`, `DeleteEdgeRedo`
- **问题**: 同时 derive `Serialize, Deserialize` 和 `Encode, Decode`，但实际序列化仅使用 oxicode
- **影响**: 代码混乱，误导维护者

#### D3. serialize_redo 有多余的 serde::Serialize 约束

- **文件**: 
  - `src/transaction/insert_transaction.rs:296`
  - `src/transaction/update_transaction.rs:692`
- **严重程度**: 🟡 低
- **问题**: 泛型函数约束 `U: serde::Serialize + oxicode::Encode`，但实际只用了 `oxicode::Encode`

#### D4. 核心 Value 类型有三套编码路径

- **文件**: `src/core/value/value_def.rs:28` (derive + 手动 `to_bytes/from_bytes`)
- **文件**: `src/transaction/codec.rs` (手动 `value_to_bytes/bytes_to_value`)
- **严重程度**: 🟡 中
- **问题**:
  - `value_def.rs` 手动编码: 用于 `property_table.rs` 的边缘属性持久化
  - `codec.rs` 手动编码: 用于 `transaction.rs` 的事务属性编解码
  - serde/oxicode derive: 用于 JSON API 和 oxicode 序列化
  - 两套手动编码使用不同 tag ID、支持不同类型集合
- **影响**: 维护负担，修改 Value 类型需同步三处

#### D5. 元数据持久化使用 JSON (可接受)

| 位置 | 描述 | 评估 |
|------|------|------|
| `schema_manager.rs` | Schema 快照为 JSON | 🟢 元数据小，可读性有益 |
| `snapshot_manager.rs` | 快照元数据为 JSON | 🟢 同上 |
| `vertex_table.rs` | 顶点表 schema 为 JSON | 🟢 仅 schema 元数据 |
| `search/manager.rs` | 搜索索引元数据为 JSON | 🟢 同上 |

#### D6. 反向索引持久化使用 serde 格式

- **文件**: `crates/inversearch/src/api/embedded/index.rs:240-344`
- **严重程度**: 🟡 低
- **问题**: 文档存储使用 JSON，索引数据通过 MessagePack (oxicode bridge)
- **影响**: 无校验和、无版本化迁移路径（作为外部 crate 风险可控）

## 建议行动项

### 优先级 1 - 立即修复

1. **`recovery.rs`**: 将 `serde_json::from_slice` 替换为 `bytes_to_value`，移除静默降级
2. **`wal/types.rs`**: 移除 8 个 WAL 类型的 `Serialize, Deserialize` derives
3. **`insert_transaction.rs` / `update_transaction.rs`**: 移除 `serialize_redo` 的 `serde::Serialize` 约束

### 优先级 2 - 后续改进

1. 评估并消除 `value_def.rs` 和 `codec.rs` 之间重复的手动编码
2. 实现 UpdateVertexProp/UpdateEdgeProp WAL 写入（当前死代码）
3. 为元数据 JSON 文件添加版本控制和校验和
4. 建立 CI 检查，防止在核心存储路径引入 serde JSON 序列化

## 合规部分摘要

| 类别 | 数量 | 状态 |
|------|------|------|
| 网络/API 通信 | ~40 类型, ~139 调用点 | ✅ 最佳实践 |
| 配置文件 | ~20 类型 | ✅ 最佳实践 |
| 内存临时序列化 | ~35 类型 | ✅ 最佳实践 |
| 磁盘数据持久化 | ~10 类型, ~15 调用点 | 🔴 1 个高优先级问题 |
