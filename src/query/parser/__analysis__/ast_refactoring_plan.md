# AST 模块重构方案

## 一、问题概述

### 1.1 当前架构问题

`src/query/parser/ast` 目录存在以下主要问题：

| 问题类型 | 具体表现 | 影响程度 |
|---------|---------|---------|
| 解析器重复 | `ast/expr_parser.rs` 与 `parser/expr_parser.rs` 功能完全重复 | 高 |
| 解析器重复 | `ast/stmt_parser.rs` 与 `parser/main_parser.rs` 功能重叠 | 高 |
| 解析器重复 | `ast/pattern_parser.rs` 与 `parser/pattern_parser.rs` 功能重叠 | 高 |
| 类型重复 | `ast/types.rs` 与 `parser/core/token.rs` 存在定义重复 | 中 |
| 依赖混乱 | `stmt.rs` 中的 `ReturnClause` 直接依赖 `clauses` 模块类型 | 中 |
| 工厂分散 | AST 构建逻辑分散在 `utils.rs` 和各子句实现中 | 低 |

### 1.2 重构目标

- 消除解析器代码重复，建立统一的解析入口
- 明确模块边界，解耦 AST 定义与解析实现
- 统一 AST 工厂入口，提供一致的节点创建方式
- 保持代码可维护性，确保测试覆盖率

---

## 二、分阶段修改方案

### 第一阶段：统一表达式解析器

**目标**：消除 `ast/expr_parser.rs` 与 `parser/expr_parser.rs` 的重复

**文件操作**：

| 操作 | 源文件 | 目标文件 |
|-----|-------|---------|
| 保留 | `parser/expr_parser.rs` | - |
| 删除 | `ast/expr_parser.rs` | - |

**修改步骤**：

1. **对比两个文件差异**
   - 检查 `parser/expr_parser.rs` 是否包含 `ast/expr_parser.rs` 的所有功能
   - 识别两个文件中的特有实现，补充缺失部分

2. **完善 `parser/expr_parser.rs`**
   - 确保所有表达式解析方法完整
   - 添加缺失的辅助方法
   - 统一错误处理方式

3. **更新 `ast/mod.rs`**
   ```rust
   // 修改前
   pub mod expr_parser;
   pub use expr_parser::*;

   // 修改后
   // expr_parser 已删除，解析功能由 parser/expr_parser.rs 提供
   ```

4. **更新引用位置**
   - 检查所有引用 `ast::expr_parser` 的位置
   - 修改为引用 `parser::expr_parser` 或通过 `ast::Expr` 类型间接使用

**验证方法**：

```bash
cargo test --lib parser::expr_parser
cargo test --lib ast::tests
```

**风险评估**：

| 风险 | 级别 | 缓解措施 |
|-----|-----|---------|
| 解析行为不一致 | 高 | 运行完整测试套件，对比两个解析器的输出 |
| 遗漏方法实现 | 中 | 代码审查，确保所有公开方法都有实现 |

---

### 第二阶段：统一语句解析器 ✅ 已完成

**目标**：消除 `ast/stmt_parser.rs` 与 `parser/main_parser.rs` 的重复

**文件操作**：

| 操作 | 源文件 | 目标文件 |
|-----|-------|---------|
| 保留 | `parser/main_parser.rs` | 重命名为 `stmt_parser.rs` |
| 删除 | `ast/stmt_parser.rs` | -修改**：

1. ✅ **评估 `parser |

**已完成的/main_parser.rs` 完整性**
   - 检查包含所有语句类型的解析方法
   - 对比 `ast/stmt_parser.rs` 中的方法列表

2. ✅ **重命名并扩展文件**
   ```
   parser/main_parser.rs -> parser/stmt_parser.rs (新建)
   ```

3. ✅ **更新 `parser/mod.rs`**
   ```rust
   mod stmt_parser;
   pub use stmt_parser::StmtParser;
   ```

4. ✅ **迁移并统一实现**
   - 从 `ast/stmt_parser.rs` 补充缺失的语句解析方法
   - 统一方法命名和参数风格
   - 添加独立的 `StmtParser` 结构体
   - 保留 `impl Parser` 的方法实现

5. ✅ **更新 `parser/parser/mod.rs`**
   ```rust
   mod stmt_parser;
   pub use stmt_parser::StmtParser;
   ```

6. ✅ **更新 `parser/mod.rs`**
   ```rust
   pub use parser::StmtParser;
   ```

7. ✅ **删除旧文件**
   - `parser/main_parser.rs`
   - `ast/stmt_parser.rs`

**关键变更**：
- `parser/stmt_parser.rs` 包含两种实现：
  - `StmtParser` 独立结构体（与 `ExprParser` 模式一致）
  - `impl Parser` 的方法实现

**验证方法**：

```bash
cargo test --lib parser::stmt_parser
cargo test --lib statements
```

---

### 第三阶段：统一模式解析器

**目标**：消除 `ast/pattern_parser.rs` 与 `parser/pattern_parser.rs` 的重复

**文件操作**：

| 操作 | 源文件 | 目标文件 |
|-----|-------|---------|
| 保留 | `parser/pattern_parser.rs` | - |
| 删除 | `ast/pattern_parser.rs` | - |

**修改步骤**：

1. **对比两个文件实现**
   - 识别共有的解析方法
   - 识别各自特有的扩展功能

2. **合并到 `parser/pattern_parser.rs`**
   - 补充缺失的模式解析方法
   - 保持接口兼容性

3. **更新 `ast/mod.rs`**
   ```rust
   // 删除以下内容
   pub mod pattern_parser;
   pub use pattern_parser::*;
   ```

**验证方法**：

```bash
cargo test --lib parser::pattern_parser
```

---

### 第四阶段：重构类型定义

**目标**：统一 Token 和 Span 等基础类型定义

**文件操作**：

| 操作 | 文件 | 说明 |
|-----|-----|-----|
| 保留 | `ast/types.rs` | 保留 `Span`、`Position` 等通用类型 |
| 修改 | `parser/core/token.rs` | 重构 TokenKind，与 ast/types.rs 保持一致 |
| 删除 | - | 无文件删除，修改为主 |

**修改步骤**：

1. **分析现有类型定义**
   ```
   ast/types.rs:
   - TokenKind (词法单元类型)
   - Token (词法单元)
   - Span (位置范围)
   - Position (行列位置)

   parser/core/token.rs:
   - Token (需要对比内容)
   - TokenKind (需要对比内容)
   ```

2. **确定类型归属**
   - `Span`、`Position` 保留在 `ast/types.rs`（通用类型）
   - `TokenKind` 考虑迁移到 `parser/core/token.rs`（解析专用）
   - 或者保留在 `ast/types.rs`，在 `parser/core/token.rs` 中重新导出

3. **统一策略（推荐）**

   **方案 A：类型集中在 ast/types.rs**
   ```rust
   // parser/core/token.rs
   pub use crate::query::parser::ast::Token;
   pub use crate::query::parser::ast::TokenKind;
   ```

   **方案 B：Token 专用类型在 parser/core/**
   ```rust
   // ast/types.rs
   pub use crate::query::parser::core::TokenKind;
   ```

4. **更新引用**
   - 统一所有文件的导入路径
   - 保持代码一致性

**验证方法**：

```bash
cargo check
cargo test --lib
```

---

### 第五阶段：解耦 AST 与解析层依赖

**目标**：消除 `stmt.rs` 中对 `clauses` 模块类型的直接依赖

**当前问题**：

```rust
// stmt.rs 中的 ReturnClause
pub struct ReturnClause {
    pub span: Span,
    pub items: Vec<ReturnItem>,
    pub limit: Option<super::super::clauses::LimitClause>,  // 问题所在
    pub skip: Option<super::super::clauses::SkipClause>,
}
```

**解决方案**：定义独立的子句类型

**文件操作**：

| 操作 | 文件 | 说明 |
|-----|-----|-----|
| 修改 | `ast/types.rs` | 添加 `LimitClause`、`SkipClause`、`SampleClause` 定义 |
| 修改 | `ast/stmt.rs` | 使用 `ast/types.rs` 中的子句类型 |

**修改步骤**：

1. **在 `ast/types.rs` 中添加子句类型定义**

   ```rust
   // ast/types.rs

   /// 限制子句
   #[derive(Debug, Clone, PartialEq)]
   pub struct LimitClause {
       pub span: Span,
       pub count: usize,
   }

   /// 跳过的子句
   #[derive(Debug, Clone, PartialEq)]
   pub struct SkipClause {
       pub span: Span,
       pub count: usize,
   }

   /// 采样子句
   #[derive(Debug, Clone, PartialEq)]
   pub struct SampleClause {
       pub span: Span,
       pub count: usize,
   }
   ```

2. **修改 `ast/stmt.rs` 中的 ReturnClause**

   ```rust
   // 修改前
   pub limit: Option<super::super::clauses::LimitClause>,
   pub skip: Option<super::super::clauses::SkipClause>,

   // 修改后
   pub limit: Option<LimitClause>,
   pub skip: Option<SkipClause>,
   ```

3. **更新解析层的类型转换**

   在 `clauses/return_clause_impl.rs` 中：

   ```rust
   impl ReturnClauseImpl {
       /// 转换为 AST 层的 ReturnClause
       pub fn to_ast(&self) -> ast::ReturnClause {
           ast::ReturnClause {
               span: self.span,
               items: self.items.iter().map(|i| i.to_ast()).collect(),
               limit: self.limit.map(|l| ast::LimitClause {
                   span: l.span,
                   count: l.count,
               }),
               skip: self.skip.map(|s| ast::SkipClause {
                   span: s.span,
                   count: s.count,
               }),
               sample: self.sample.map(|s| ast::SampleClause {
                   span: s.span,
                   count: s.count,
               }),
               distinct: self.distinct,
           }
       }
   }
   ```

4. **删除 `ast/types.rs` 中旧有导入**（如果有）

**验证方法**：

```bash
cargo check
cargo test --lib ast
cargo test --lib clauses
```

---

### 第六阶段：统一 AST 工厂入口

**目标**：集中 AST 构建逻辑，提供一致的节点创建方式

**当前状态**：

| 工厂位置 | 用途 |
|---------|-----|
| `ast/utils.rs` | `ExprFactory`、`StmtFactory` 通用工厂 |
| `clauses/return_clause_impl.rs` | `ReturnClauseImpl` 自己的构建逻辑 |
| `clauses/where_clause_impl.rs` | `WhereClauseImpl` 自己的构建逻辑 |
| ... | 其他子句实现 |

**修改步骤**：

1. **扩展 `ast/utils.rs` 中的工厂类**

   ```rust
   // ast/utils.rs

   pub struct ExprFactory;

   impl ExprFactory {
       // ... 现有方法

       /// 从 ReturnClauseImpl 创建 ReturnClause
       pub fn return_clause_from_impl(
           impl_clause: &super::super::clauses::ReturnClauseImpl,
       ) -> super::stmt::ReturnClause {
           super::stmt::ReturnClause {
               span: impl_clause.span,
               items: impl_clause
                   .items
                   .iter()
                   .map(|item| super::expr::Expr::from_impl(item))
                   .collect(),
               distinct: impl_clause.distinct,
               limit: impl_clause.limit.as_ref().map(|l| LimitClause {
                   span: l.span,
                   count: l.count,
               }),
               skip: impl_clause.skip.as_ref().map(|s| SkipClause {
                   span: s.span,
                   count: s.count,
               }),
               sample: impl_clause.sample.as_ref().map(|s| SampleClause {
                   span: s.span,
                   count: s.count,
               }),
           }
       }
   }
   ```

2. **在子句实现中添加转换方法**

   ```rust
   // clauses/return_clause_impl.rs

   impl ReturnClauseImpl {
       /// 使用工厂创建 AST 节点
       pub fn to_ast(&self) -> ast::ReturnClause {
           ast::ExprFactory::return_clause_from_impl(self)
       }
   }
   ```

3. **可选：删除各子句实现中的重复逻辑**

   如果确认工厂类功能完整，可以删除子句实现中的 `to_ast` 方法，仅保留工厂类的使用。

**验证方法**：

```bash
cargo test --lib utils
cargo test --lib ast::tests
```

---

## 三、文件变更汇总

### 3.1 删除的文件

| 文件路径 | 删除原因 |
|---------|---------|
| `ast/expr_parser.rs` | 功能与 `parser/expr_parser.rs` 重复 |
| `ast/stmt_parser.rs` | 功能与 `parser/main_parser.rs` 重复（将重命名） |
| `ast/pattern_parser.rs` | 功能与 `parser/pattern_parser.rs` 重复 |

### 3.2 修改的文件

| 文件路径 | 修改内容 |
|---------|---------|
| `ast/mod.rs` | 删除三个解析器模块的导出 |
| `parser/mod.rs` | 更新模块声明 |
| `parser/main_parser.rs` | 重命名为 `stmt_parser.rs`，补充缺失方法 |
| `ast/types.rs` | 添加 `LimitClause`、`SkipClause`、`SampleClause` 定义 |
| `ast/stmt.rs` | 使用 `ast/types.rs` 中的子句类型 |
| `clauses/return_clause_impl.rs` | 添加 `to_ast` 转换方法 |
| `clauses/where_clause_impl.rs` | 统一 AST 构建方式 |
| `clauses/*.rs` | 需要类型转换的子句实现 |

### 3.3 新增的文件

| 文件路径 | 说明 |
|---------|-----|
| `parser/stmt_parser.rs` | 由 `parser/main_parser.rs` 重命名而来 |

---

## 四、测试验证清单

### 4.1 单元测试

```bash
# 解析器测试
cargo test --lib parser::expr_parser
cargo test --lib parser::stmt_parser
cargo test --lib parser::pattern_parser

# AST 测试
cargo test --lib ast::tests
cargo test --lib ast::expr_tests

# 子句测试
cargo test --lib clauses
```

### 4.2 集成测试

```bash
# 完整解析流程测试
cargo test --lib query::parser

# 执行器测试
cargo test --lib query::executor
```

### 4.3 回归测试

```bash
# 运行所有测试
cargo test --lib

# 特别关注：
# - query::visitor 模块的所有测试
# - query::validator 模块的所有测试
# - query::executor 模块的所有测试
```

---

## 五、风险控制

### 5.1 回退策略

每个阶段完成后，执行以下回退检查：

1. 代码编译检查
   ```bash
   cargo check
   ```

2. 核心测试通过
   ```bash
   cargo test --lib query::parser::ast::tests
   ```

3. 如果发现问题，使用 Git 回退
   ```bash
   git checkout <commit-hash>
   ```

### 5.2 阶段隔离

建议每个阶段独立提交：

```
Phase 1: Remove expr_parser duplication
Phase 2: Remove stmt_parser duplication  
Phase 3: Remove pattern_parser duplication
Phase 4: Refactor type definitions
Phase 5: Decouple AST from clauses
Phase 6: Unify AST factories
```

### 5.3 沟通协调

- 每个阶段开始前，在团队中同步变更内容
- 阶段完成后，分享测试结果和变更日志
- 如果涉及其他模块的修改，提前通知相关开发者

---

## 六、时间估算

| 阶段 | 预估时间 | 依赖 |
|-----|---------|-----|
| 第一阶段 | 2-4 小时 | 无 |
| 第二阶段 | 4-6 小时 | 第一阶段完成 |
| 第三阶段 | 1-2 小时 | 第一阶段完成 |
| 第四阶段 | 2-4 小时 | 第一阶段完成 |
| 第五阶段 | 4-8 小时 | 第四阶段完成 |
| 第六阶段 | 2-4 小时 | 第五阶段完成 |
| **总计** | **15-28 小时** | - |

---

## 七、附录

### 7.1 相关文件路径

```
src/query/parser/ast/
├── mod.rs
├── types.rs          # 基础类型定义
├── expr.rs           # 表达式 AST
├── stmt.rs           # 语句 AST
├── pattern.rs        # 模式 AST
├── visitor.rs        # 访问者模式
├── utils.rs          # 工具函数
├── tests.rs          # 测试模块
├── expr_parser.rs    # [待删除] 重复的表达式解析器
├── stmt_parser.rs    # [待删除] 重复的语句解析器
└── pattern_parser.rs # [待删除] 重复的模式解析器

src/query/parser/
├── mod.rs
├── parser/
│   ├── mod.rs
│   ├── expr_parser.rs     # 表达式解析器
│   ├── main_parser.rs     # [待重命名] 语句解析器
│   └── pattern_parser.rs  # 模式解析器
├── clauses/
│   ├── return_clause.rs
│   ├── return_clause_impl.rs
│   └── ...
├── statements/
│   ├── match_stmt.rs
│   └── ...
└── core/
    ├── token.rs
    └── error.rs
```

### 7.2 参考资料

- [NebulaGraph Parser 架构分析](docs/architecture/parser.md)
- [Rust 最佳实践](https://rust-lang-nursery.github.io/api-guidelines/)
- 项目测试用例：`tests/parser_tests.rs`
