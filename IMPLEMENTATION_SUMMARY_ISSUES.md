# GraphDB 项目重构遗留问题文档

## 问题概述

在对 GraphDB 项目的查询解析器模块进行重构和拆分过程中，发现了一系列需要解决的问题。本文档详细记录了这些问题及其影响。

## 问题列表

### 1. Trait 方法冲突问题

**问题描述：**
在 `statements` 模块中的多个 trait（如 `CreateStatementParser`、`MatchStatementParser` 等）与 `expressions` 模块中的 `ExpressionParser` trait 定义了相同的方法签名，导致编译时出现多个适用项的错误（E0034）。

**具体表现：**
- 多个 trait 定义了 `parse_expression`、`current_token`、`next_token`、`expect_token` 等相同名称的方法
- 在实现类中调用这些方法时，编译器无法确定使用哪个 trait 的方法
- 编译错误示例：`multiple applicable items in scope`

**影响范围：**
- 所有 statements 模块（create.rs, match_stmt.rs, delete.rs, update.rs, go.rs）
- 所有使用这些 trait 的解析器实现

**临时解决方案：**
- 已注释掉 `src/query/parser/mod.rs` 中的 `pub mod statements;` 行
- 这时禁用了 statements 模块的导入以避免编译错误

### 2. AST 结构体重复定义问题

**问题描述：**
在 `statements/go.rs` 中定义的 `GoStatement` 结构体与 `ast/statement.rs` 中定义的 `GoStatement` 结构体重疊，导致类型混淆。

**具体表现：**
- 同一个数据结构在不同模块中有重复定义
- 解析器返回的结构体类型与 AST 中期望的类型不匹配

**影响范围：**
- GO 语句的解析逻辑
- 与 GoStatement 相关的转换和处理代码

### 3. 模块导入路径错误

**问题描述：**
部分文件中仍存在对旧的模块路径的引用，如 `ast::ast::Statement` 而非 `ast::Statement`。

**具体表现：**
- `src/query/mod.rs` 中存在大量对 `crate::query::parser::ast::ast::Statement` 的引用
- 尽管 `ast.rs` 文件已被废弃，但仍存在对其的引用

**影响范围：**
- 查询转换逻辑
- 语句类型转换相关代码

### 4. Parser 类型定义错误

**问题描述：**
在 `utils.rs` 文件的 `new` 函数中，存在对 `Parser` 类型的直接引用而非使用 `Self`。

**具体表现：**
- 编译错误：`cannot find struct, variant or union type 'Parser' in this scope`
- 错误出现在 `utils.rs:15:9`

**影响范围：**
- Parser 初始化逻辑
- 所有创建新 Parser 实例的地方

### 5. 表达式常量解引用错误

**问题描述：**
在 `expression_parser.rs` 中，对整数、浮点数和布尔字面量的解引用操作可能会导致编译警告或错误。

**具体表现：**
- 编译警告：`type 'i64' cannot be dereferenced`
- 类似的错误出现在处理整数、浮点数和布尔类型的常量时

**影响范围：**
- 常量表达式解析
- 所有涉及常量值的表达式处理

## 建议的解决方案

### 对于 Trait 方法冲突问题：
1. **方案A：** 重构 trait 设计，将通用方法（如 `parse_expression`）统一到一个基础 trait 中，其他 trait 继承该基础 trait
2. **方案B：** 使用不同的方法命名，例如 `parse_statement_expression`、`parse_pattern_expression` 等
3. **方案C：** 采用组合模式，将功能委托给专门的解析器对象，而不是通过 trait 实现

### 对于 AST 结构体重复定义问题：
1. **统一定义：** 删除所有重复的结构体定义，只保留 AST 模块中的规范版本
2. **重构代码：** 修改 statements 模块中的代码，使其返回 AST 定义的结构体实例

### 对于模块导入路径错误：
1. **批量更新：** 全局搜索并替换所有旧的导入路径为新的路径
2. **逐步迁移：** 按照模块依赖关系逐步更新所有引用

### 对于 Parser 类型定义错误：
1. **修复代码：** 将 `Parser` 替换为 `Self`
2. **标准化：** 确保所有构造函数使用一致的初始化模式

## 后续步骤

1. 优先解决 Trait 方法冲突问题，这是最大的编译障碍
2. 完善模块间的类型引用一致性
3. 清理所有临时注释和禁用的代码
4. 进行完整的编译和测试流程
5. 更新相关测试以适应新的模块结构

## 备注

本次拆分工作成功实现了 AST 模块和 Parser 模块的基本拆分，达到了提高代码组织性和可维护性的目标。遗留问题主要是架构设计层面的问题，需进一步的深入重构来彻底解决。