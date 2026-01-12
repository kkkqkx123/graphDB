# 上下文管理统一方案

## 问题分析

当前存在两个ValidateContext实现：
1. `src/query/context/validate/context.rs` - 功能完整，包含Schema管理、生成器等
2. `src/query/validator/validate_context.rs` - 功能简单，实现了ValidationContext trait

## 重构策略

### 阶段1：增强context/validate/context.rs
1. 为`context::validate::ValidateContext`实现`ValidationContext` trait
2. 添加缺失的QueryPart和AliasType管理功能
3. 统一错误处理机制

### 阶段2：迁移validator模块
1. 修改validator模块使用context版本的ValidateContext
2. 更新所有依赖文件
3. 移除validator/validate_context.rs

### 阶段3：清理和优化
1. 清理不再需要的代码
2. 优化接口设计
3. 更新文档和测试

## 实施步骤

### 步骤1：增强context/validate/context.rs
- 添加QueryPart和AliasType字段
- 实现ValidationContext trait
- 统一错误处理

### 步骤2：更新类型定义
- 统一Space、Column、Variable等类型定义
- 确保类型兼容性

### 步骤3：迁移使用方
- 更新validator模块
- 更新visitor模块
- 更新其他依赖模块

### 步骤4：清理旧代码
- 移除validator/validate_context.rs
- 更新mod.rs文件
- 清理导入

## 风险评估

- **低风险**：context版本功能更完整，迁移相对安全
- **兼容性**：需要确保所有使用方的兼容性
- **测试**：需要全面测试确保功能正常

## 预期收益

1. 消除代码重复
2. 统一上下文管理
3. 提高代码可维护性
4. 减少不一致性风险