# src/query/context 目录结构调整实施计划

## 项目概述

本计划详细描述了如何将 `src/query/context` 目录从当前结构重构为建议的新结构。

## 当前状态分析

### 问题识别
1. **文件大小不均衡**: 3个文件超过800行，1个文件达到1600行
2. **职责混杂**: 单个文件承担过多不同职责
3. **依赖关系复杂**: 存在潜在的循环依赖风险

### 重构目标
1. **文件大小优化**: 每个文件控制在300-500行以内
2. **职责单一化**: 每个文件专注于单一职责
3. **依赖关系清晰化**: 建立清晰的模块层次结构

## 实施阶段

### 第一阶段：准备阶段（预计1-2天）

#### 1.1 创建备份和测试基准
```bash
# 创建代码备份
cp -r src/query/context src/query/context_backup
```

#### 1.2 创建新的目录结构
```bash
# 创建新目录结构
mkdir -p src/query/context/{ast,execution,expression,request,managers}
mkdir -p src/query/context/ast/query_types
mkdir -p src/query/context/expression/schema
```

#### 1.3 更新根模块文件
创建新的 `src/query/context/mod.rs`：

```rust
//! 查询上下文模块 - 重构版本
//! 
//! 新的模块结构：
//! - ast/: AST相关上下文
//! - execution/: 执行相关上下文  
//! - expression/: 表达式相关上下文
//! - request/: 请求相关上下文
//! - managers/: 管理器接口
//! - validate/: 验证上下文（保持现有结构）

pub mod ast;
pub mod execution;
pub mod expression;
pub mod request;
pub mod managers;
pub mod validate;

// 重新导出主要类型
pub use ast::*;
pub use execution::*;
pub use expression::*;
pub use request::*;
pub use managers::*;
pub use validate::*;
```

### 第二阶段：拆分超大文件（预计3-5天）

#### 2.1 拆分 `storage_expression_context.rs` (1600行)

**步骤：**
1. 创建 `expression/schema/mod.rs`
2. 提取字段类型定义到 `expression/schema/types.rs`
3. 提取Schema定义到 `expression/schema/schema_def.rs`
4. 提取行读取器到 `expression/schema/row_reader.rs`
5. 保留核心逻辑在 `expression/storage_expression.rs`

**依赖关系调整：**
- 更新所有引用 `storage_expression_context` 的导入
- 确保新的模块结构正确导出

#### 2.2 拆分 `query_context.rs` (845行)

**步骤：**
1. 创建 `managers/mod.rs`
2. 提取Schema管理器接口到 `managers/schema_manager.rs`
3. 提取索引管理器接口到 `managers/index_manager.rs`
4. 提取存储客户端接口到 `managers/storage_client.rs`
5. 提取元数据客户端接口到 `managers/meta_client.rs`
6. 保留查询上下文主体在 `execution/query_execution.rs`

#### 2.3 拆分 `request_context.rs` (938行)

**步骤：**
1. 创建 `request/mod.rs`
2. 提取会话管理到 `request/session.rs`
3. 提取参数管理到 `request/parameters.rs`
4. 提取响应管理到 `request/response.rs`
5. 保留基础请求上下文在 `request/base.rs`

### 第三阶段：重构其他文件（预计2-3天）

#### 3.1 重构 `ast_context.rs` (383行)

**步骤：**
1. 创建 `ast/mod.rs`
2. 提取基础AST上下文到 `ast/base.rs`
3. 提取共享结构定义到 `ast/common.rs`
4. 按查询类型拆分到 `ast/query_types/` 目录

#### 3.2 合并小文件

**步骤：**
1. 将 `expression_eval_context.rs` (70行) 合并到 `expression/eval.rs`
2. 删除原文件

#### 3.3 更新现有文件位置

**步骤：**
1. 移动 `execution_context.rs` 到 `execution/query_execution.rs`
2. 移动 `expression_context.rs` 到 `expression/query_expression.rs`
3. 移动 `runtime_context.rs` 到 `execution/runtime.rs`

### 第四阶段：依赖关系优化（预计1-2天）

#### 4.1 清理循环依赖

**检查点：**
- 确保基础层不依赖高层模块
- 验证模块间的导入关系
- 消除不必要的依赖

#### 4.2 优化模块接口

**工作：**
- 为每个模块定义清晰的公共接口
- 添加必要的文档注释
- 确保类型可见性正确

### 第五阶段：测试验证（预计1-2天）

#### 5.1 运行完整测试套件

```bash
# 运行所有上下文相关测试
cargo test --test "*context*" -- --test-threads=1

# 运行集成测试
cargo test --test "*integration*" -- --test-threads=1
```

#### 5.2 性能基准测试

**检查：**
- 编译时间变化
- 运行时性能
- 内存使用情况

#### 5.3 代码质量检查

**工具：**
```bash
# 代码格式化
cargo fmt

# 代码检查
cargo clippy

# 文档生成
cargo doc
```

## 风险管理和回滚计划

### 风险识别

1. **编译错误风险**: 高 - 由于大量文件移动和重构
2. **功能回归风险**: 中 - 需要仔细测试
3. **性能影响风险**: 低 - 主要是结构重构

### 缓解措施

1. **分阶段实施**: 每个阶段完成后进行测试
2. **版本控制**: 使用git分支管理变更
3. **持续集成**: 每个提交都运行测试

### 回滚计划

**如果遇到严重问题：**

```bash
# 恢复到备份
rm -rf src/query/context
cp -r src/query/context_backup src/query/context

# 重新编译
cargo build
```

## 成功标准

### 技术标准
1. **编译通过**: 所有代码能够正常编译
2. **测试通过**: 所有现有测试通过
3. **性能稳定**: 性能没有明显下降
4. **依赖清晰**: 模块依赖关系清晰合理

### 代码质量标准
1. **文件大小**: 每个文件控制在500行以内
2. **职责单一**: 每个文件专注于单一职责
3. **文档完整**: 所有公共接口都有文档注释
4. **测试覆盖**: 关键功能都有测试覆盖

## 时间估算

| 阶段 | 任务 | 预计时间 | 负责人 |
|------|------|----------|--------|
| 第一阶段 | 准备工作 | 1-2天 | 开发团队 |
| 第二阶段 | 拆分超大文件 | 3-5天 | 主要开发人员 |
| 第三阶段 | 重构其他文件 | 2-3天 | 开发团队 |
| 第四阶段 | 依赖优化 | 1-2天 | 架构师 |
| 第五阶段 | 测试验证 | 1-2天 | QA团队 |
| **总计** | **全部工作** | **8-14天** | **跨职能团队** |

## 后续工作

### 重构后的优化
1. **代码审查**: 团队内部代码审查
2. **性能优化**: 基于新结构进行性能优化
3. **功能扩展**: 更容易添加新功能

### 监控和维护
1. **代码质量监控**: 定期检查文件大小和复杂度
2. **依赖关系审计**: 定期检查模块依赖
3. **文档更新**: 保持文档与代码同步

## 结论

本次重构将显著提升 `src/query/context` 目录的代码质量、可维护性和可扩展性。通过分阶段实施和严格的风险管理，可以确保重构过程平稳进行。

---

**计划制定时间**: 2024年
**预计开始时间**: 待定
**预计完成时间**: 待定