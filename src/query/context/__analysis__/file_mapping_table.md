# 文件映射表 - 当前文件到新结构的映射

## 概述

本表格详细列出了当前 `src/query/context` 目录下的每个文件将如何映射到新的目录结构中。

## 映射表

| 当前文件 | 新位置 | 拆分/合并说明 | 优先级 | 预估工作量 |
|----------|--------|--------------|--------|------------|
| `ast_context.rs` (383行) | `ast/` 目录 | 拆分为多个文件 | 中 | 中等 |
| `execution_context.rs` (315行) | `execution/query_execution.rs` | 直接移动，无需拆分 | 低 | 低 |
| `expression_context.rs` (467行) | `expression/query_expression.rs` | 直接移动，无需拆分 | 低 | 低 |
| `expression_eval_context.rs` (70行) | `expression/eval.rs` | 合并到新文件 | 低 | 低 |
| `query_context.rs` (845行) | 多个文件 | 按功能拆分 | 高 | 高 |
| `request_context.rs` (938行) | `request/` 目录 | 按功能拆分 | 高 | 高 |
| `runtime_context.rs` (412行) | `execution/runtime.rs` | 直接移动，无需拆分 | 中 | 低 |
| `storage_expression_context.rs` (1600行) | `expression/` 目录 | 大幅拆分 | 最高 | 最高 |
| `validate/` 目录 | `validate/` 目录 | 保持现有结构 | 低 | 低 |

## 详细拆分方案

### 1. `ast_context.rs` → `ast/` 目录

**当前内容：**
- 基础AST上下文 (`AstContext`)
- GO查询上下文 (`GoContext`)
- Fetch Vertices上下文 (`FetchVerticesContext`)
- Fetch Edges上下文 (`FetchEdgesContext`)
- Lookup上下文 (`LookupContext`)
- Path查询上下文 (`PathContext`)
- Subgraph上下文 (`SubgraphContext`)
- Maintain上下文 (`MaintainContext`)

**新结构：**
```
ast/
├── mod.rs
├── base.rs                    # AstContext, Starts, Over, StepClause, ExpressionProps
├── common.rs                  # 共享结构定义
└── query_types/
    ├── mod.rs
    ├── go.rs                  # GoContext
    ├── fetch_vertices.rs      # FetchVerticesContext
    ├── fetch_edges.rs         # FetchEdgesContext
    ├── lookup.rs              # LookupContext
    ├── path.rs                # PathContext
    └── subgraph.rs            # SubgraphContext
```

### 2. `query_context.rs` → 多个文件

**当前内容：**
- Schema管理器接口 (`SchemaManager`)
- 索引管理器接口 (`IndexManager`)
- 存储客户端接口 (`StorageClient`)
- 元数据客户端接口 (`MetaClient`)
- 查询上下文主体 (`QueryContext`)
- 执行计划相关 (`ExecutionPlan`, `PlanNode`)
- 各种响应和操作类型

**新结构：**
```
managers/
├── mod.rs
├── schema_manager.rs          # SchemaManager trait, Schema结构
├── index_manager.rs           # IndexManager trait, Index结构
├── storage_client.rs          # StorageClient trait, StorageOperation, StorageResponse
└── meta_client.rs            # MetaClient trait, ClusterInfo, SpaceInfo

execution/
├── mod.rs
└── query_execution.rs        # QueryContext主体，ExecutionPlan，PlanNode
```

### 3. `request_context.rs` → `request/` 目录

**当前内容：**
- 会话信息 (`SessionInfo`)
- 请求参数 (`RequestParams`)
- 响应对象 (`Response`)
- 请求上下文主体 (`RequestContext`)
- 请求状态管理 (`RequestStatus`)
- 自定义属性管理

**新结构：**
```
request/
├── mod.rs
├── session.rs                # SessionInfo
├── parameters.rs              # RequestParams
├── response.rs                # Response
└── base.rs                   # RequestContext主体，RequestStatus
```

### 4. `storage_expression_context.rs` → `expression/` 目录

**当前内容：**
- 字段类型定义 (`FieldType`)
- 字段定义 (`FieldDef`)
- Schema定义 (`Schema`)
- 行读取器 (`RowReaderWrapper`)
- 表达式上下文trait (`ExpressionContext`)
- 存储表达式上下文主体 (`StorageExpressionContext`)

**新结构：**
```
expression/
├── mod.rs
├── storage_expression.rs      # StorageExpressionContext主体
├── eval.rs                    # 合并expression_eval_context.rs
└── schema/
    ├── mod.rs
    ├── types.rs               # FieldType枚举
    ├── schema_def.rs          # FieldDef, Schema
    └── row_reader.rs          # RowReaderWrapper
```

### 5. 其他文件的简单移动

| 当前文件 | 新位置 | 说明 |
|----------|--------|------|
| `execution_context.rs` | `execution/query_execution.rs` | 重命名以区分不同执行上下文 |
| `expression_context.rs` | `expression/query_expression.rs` | 重命名以区分不同表达式上下文 |
| `runtime_context.rs` | `execution/runtime.rs` | 保持名称一致性 |
| `expression_eval_context.rs` | `expression/eval.rs` | 合并到新文件 |

## 依赖关系调整

### 需要更新的导入路径

**当前导入示例：**
```rust
use crate::query::context::QueryContext;
use crate::query::context::RequestContext;
use crate::query::context::StorageExpressionContext;
```

**新导入示例：**
```rust
use crate::query::context::execution::QueryContext;
use crate::query::context::request::RequestContext;
use crate::query::context::expression::StorageExpressionContext;
```

### 模块导出配置

每个新目录的 `mod.rs` 需要正确导出类型：

**`ast/mod.rs` 示例：**
```rust
pub mod base;
pub mod common;
pub mod query_types;

pub use base::*;
pub use common::*;
pub use query_types::*;
```

## 实施顺序建议

### 第一阶段：基础结构
1. 创建新目录结构
2. 移动简单文件 (`execution_context.rs`, `expression_context.rs`, `runtime_context.rs`)
3. 合并小文件 (`expression_eval_context.rs`)

### 第二阶段：复杂拆分
1. 拆分 `storage_expression_context.rs`
2. 拆分 `query_context.rs`
3. 拆分 `request_context.rs`

### 第三阶段：AST重构
1. 拆分 `ast_context.rs`
2. 更新所有依赖关系
3. 运行完整测试

## 测试策略

### 单元测试迁移
- 每个拆分后的文件保持原有测试
- 新增模块边界测试
- 确保接口兼容性

### 集成测试
- 验证模块间协作
- 检查性能回归
- 确认功能完整性

## 风险评估

### 高风险操作
1. `storage_expression_context.rs` 的拆分 - 代码量大，逻辑复杂
2. `query_context.rs` 的拆分 - 依赖关系复杂
3. 模块接口变更 - 可能影响外部调用

### 缓解措施
1. 分步骤实施，每个步骤后运行测试
2. 保持向后兼容的临时接口
3. 详细的代码审查

## 成功指标

### 技术指标
- 所有文件大小控制在500行以内
- 编译时间无明显增加
- 测试覆盖率保持或提升

### 质量指标
- 代码复杂度降低
- 模块职责更清晰
- 依赖关系更合理

---

**文档完成时间**: 2024年
**最后更新**: 当前