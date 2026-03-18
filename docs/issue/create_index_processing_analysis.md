# CREATE INDEX 处理逻辑问题分析

## 问题概述

当前 GraphDB 项目中 CREATE INDEX 语句的处理流程存在严重缺陷，导致用户无法通过 SQL 成功创建索引。

## 当前处理流程

```
SQL: CREATE INDEX idx ON Person(name)
        ↓
┌─────────────────────────────────────────────────────────────────┐
│ 1. 解析层 (ddl_parser.rs)                                        │
│    语法: CREATE INDEX <name> ON <entity>(<properties>)         │
│    解析为: CreateTarget::Index { name, on, properties }        │
│    ⚠️ 问题: 没有区分 TAG INDEX 和 EDGE INDEX                   │
└─────────────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────────────┐
│ 2. 验证层 (create_validator.rs)                                  │
│    明确返回错误: "CreateValidator 不支持 CREATE TAG/EDGE/INDEX" │
│    ❌ 完全没有验证逻辑                                          │
└─────────────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────────────┐
│ 3. 规划层 (planner)                                             │
│    CreatePlanner 只处理: CREATE (n:Label {}) / CREATE ()-[]->()│
│    ❌ 缺少 INDEX 的规划器                                        │
└─────────────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────────────┐
│ 4. 执行层 (executor)                                             │
│    存储层已实现: create_tag_index / create_edge_index           │
│    执行器已实现: CreateTagIndexExecutor / CreateEdgeIndexExecutor│
│    ⚠️ 由于没有 planner，可能没有被调用                           │
└─────────────────────────────────────────────────────────────────┘
```

## 详细问题分析

### 1. 解析层问题

**文件**: `src/query/parser/parser/ddl_parser.rs`

**当前语法**:
```sql
CREATE INDEX name ON entity(prop1, prop2)
```

**问题**: 
- 语法与 NebulaGraph 不兼容
- NebulaGraph 使用:
  - `CREATE TAG INDEX idx ON Person(name)` - 创建标签索引
  - `CREATE EDGE INDEX idx ON KNOWS(name)` - 创建边索引

**当前解析代码** (行 249-280):
```rust
} else if ctx.match_token(TokenKind::Index) {
    // 解析 CREATE INDEX
    let mut if_not_exists = false;
    if ctx.match_token(TokenKind::If) {
        ctx.expect_token(TokenKind::Not)?;
        ctx.expect_token(TokenKind::Exists)?;
        if_not_exists = true;
    }
    let name = ctx.expect_identifier()?;
    ctx.expect_token(TokenKind::On)?;
    let on = ctx.expect_identifier()?;
    ctx.expect_token(TokenKind::LParen)?;
    // ...
```

无法区分是 Tag Index 还是 Edge Index。

### 2. 验证层问题

**文件**: `src/query/validator/statements/create_validator.rs`

**当前代码** (行 654-660):
```rust
CreateTarget::Tag { .. }
| CreateTarget::EdgeType { .. }
| CreateTarget::Index { .. } => Err(ValidationError::new(
    "CreateValidator 不支持 CREATE TAG/EDGE/INDEX，请使用 DDL 验证器".to_string(),
    ValidationErrorType::SemanticError,
))
```

**问题**:
- 验证器明确拒绝处理 `CreateTarget::Index`
- 没有专门的 DDL 验证器来处理 Index 创建
- `admin_validator.rs` 只处理 SHOW/DESC Index，没有处理 CREATE

### 3. 规划层问题

**文件**: `src/query/planner/planner.rs`

**当前代码** (行 162-182):
```rust
Stmt::Create(_)
| Stmt::Drop(_)
| Stmt::Show(_)
| Stmt::Desc(_)
| Stmt::Alter(_)
// ... => Some(PlannerEnum::Maintain(MaintainPlanner::new()))
```

**问题**:
- 虽然 `MaintainPlanner` 存在，但没有处理 `CreateTarget::Index` 的代码
- 没有生成 `CreateTagIndexNode` 或 `CreateEdgeIndexNode` 的逻辑

### 4. 执行层 (正常)

**存储层** (`src/storage/redb_storage.rs:473-510`):
- `create_tag_index()` - 已实现
- `create_edge_index()` - 已实现

**执行器** (`src/query/executor/admin/index/`):
- `CreateTagIndexExecutor` - 已实现
- `CreateEdgeIndexExecutor` - 已实现

## 实现合理性评估

| 方面 | 问题 | 严重程度 |
|------|------|----------|
| **语法设计** | 未区分 Tag/Edge Index，与主流图数据库不兼容 | 🔴 高 |
| **验证层** | 完全缺失 CREATE INDEX 的验证逻辑 | 🔴 高 |
| **规划层** | 缺少将 Index 创建转换为执行计划的逻辑 | 🔴 高 |
| **执行层** | 实现已存在但无法被调用 | 🟡 中 |

## 改进方案

### 1. 修改解析器语法

支持:
- `CREATE TAG INDEX name ON TagName(prop1, ...)`
- `CREATE EDGE INDEX name ON EdgeType(prop1, ...)`

### 2. 添加验证器

在 `src/query/validator/ddl/` 目录下创建 `index_validator.rs`:
- 验证索引名是否已存在
- 验证目标 Tag/Edge 是否存在
- 验证属性是否属于目标 Tag/Edge

### 3. 添加规划器

在 `MaintainPlanner` 中处理 Index 创建:
- 生成对应的 `CreateTagIndexNode` / `CreateEdgeIndexNode`

### 4. 执行层

无需修改 (已实现)

## 结论

当前实现是不完整的。虽然存储层和执行层已经实现了完整的索引创建功能，但由于解析层语法设计有缺陷、验证层直接拒绝处理、规划层完全缺失，导致用户无法通过 SQL 语句成功创建索引。这是一个需要系统性修复的功能缺陷。

---

*生成时间: 2026-03-18*
