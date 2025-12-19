# 编译修复检查清单

## 已完成 ✓

- [x] CypherClauseKind: 添加 Copy, Eq derive
- [x] WhereClauseContext: 添加 paths, aliases_available, aliases_generated
- [x] ReturnClauseContext: 添加 order_by, pagination, distinct
- [x] YieldClauseContext: 添加所有投影、聚合、分组相关字段
- [x] OrderByClauseContext: 添加 indexed_order_factors
- [x] clause_planner.rs: 移除不必要的 clone()
- [x] 简化规划器实现为占位符（为了通过编译）

## 需要完成

### 关键结构体初始化
- [ ] 在结构体中添加 Default 实现（可选）
- [ ] 确保所有新字段在创建上下文时正确初始化

### 规划器实现恢复
- [ ] return_clause_planner.rs: 恢复完整实现
- [ ] where_clause_planner.rs: 恢复完整实现
- [ ] yield_planner.rs: 恢复完整实现
- [ ] order_by_planner.rs: 恢复完整实现
- [ ] pagination_planner.rs: 恢复完整实现
- [ ] with_clause_planner.rs: 恢复完整实现
- [ ] unwind_planner.rs: 恢复完整实现
- [ ] projection_planner.rs: 恢复完整实现

### 测试修复
- [ ] clause_planner.rs: 更新 test_base_clause_planner_validate_context_failure
- [ ] 其他规划器测试：验证新字段访问

### 编译验证
- [ ] `cargo check` 无编译错误
- [ ] `cargo test --lib` 所有单元测试通过
- [ ] `cargo build --release` 成功构建

## 关键修改位置参考

| 文件 | 行号 | 修改 |
|------|------|------|
| base_validator.rs | 347 | CypherClauseKind: 添加 Copy, Eq |
| base_validator.rs | 399 | WhereClauseContext: +4字段 |
| base_validator.rs | 408 | ReturnClauseContext: +3字段 |
| base_validator.rs | 423 | OrderByClauseContext: +1字段 |
| base_validator.rs | 457 | YieldClauseContext: +13字段 |
| clause_planner.rs | 92 | 移除 clone() |
| clause_planner.rs | 280 | 测试: 更新 WhereClauseContext 构造 |

## 常见陷阱

1. **字段初始化**: 所有新字段都是必需的，需要在每个构造点初始化
2. **clone vs Copy**: CypherClauseKind 现在是 Copy，不需要显式 clone()
3. **HashMap 初始化**: 使用 `HashMap::new()` 或 `std::collections::HashMap::new()`
4. **Vec 初始化**: 使用 `vec![]` 或 `Vec::new()`
5. **Option 类型**: Some(...) 或 None，不能省略

## 测试命令

```bash
# 快速类型检查
cargo check --message-format=short 2>&1 | Select-String "error\[E"

# 详细错误
cargo check 2>&1 | Select-String "error\[E" -Context 3

# 特定文件检查
cargo check --lib 2>&1 | findstr "return_clause_planner"

# 完整编译
cargo build --release 2>&1 | tail -50
```

## 下一步优化

完成基本修复后，考虑：
1. 为所有上下文结构实现 Default trait
2. 添加 Builder 模式简化创建
3. 添加字段验证方法
4. 优化存储（使用 SmallVec 等）
