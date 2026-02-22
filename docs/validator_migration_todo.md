# 验证器迁移待办清单

## 迁移状态总览

- **已完成**: 6 个验证器
- **待完成**: 27 个验证器
- **总计**: 33 个验证器

## 已完成 ✅

1. ✅ `use_validator.rs`
2. ✅ `match_validator.rs`
3. ✅ `go_validator.rs`
4. ✅ `lookup_validator.rs`
5. ✅ `fetch_vertices_validator.rs`
6. ✅ `insert_vertices_validator.rs`

## 待完成 ⏳

### DML 验证器 (3个)
- [ ] `insert_edges_validator.rs`
- [ ] `update_validator.rs`
- [ ] `delete_validator.rs`

### 查询验证器 (12个)
- [ ] `fetch_edges_validator.rs`
- [ ] `find_path_validator.rs`
- [ ] `get_subgraph_validator.rs`
- [ ] `group_by_validator.rs`
- [ ] `limit_validator.rs`
- [ ] `order_by_validator.rs`
- [ ] `pipe_validator.rs`
- [ ] `query_validator.rs`
- [ ] `return_validator.rs`
- [ ] `sequential_validator.rs`
- [ ] `set_operation_validator.rs`
- [ ] `unwind_validator.rs`
- [ ] `with_validator.rs`
- [ ] `yield_validator.rs`

### DDL 验证器 (3个)
- [ ] `create_validator.rs`
- [ ] `drop_validator.rs`
- [ ] `alter_validator.rs`

### 管理验证器 (9个)
- [ ] `admin_validator.rs`
- [ ] `acl_validator.rs`
- [ ] `assignment_validator.rs`
- [ ] `explain_validator.rs`
- [ ] `merge_validator.rs`
- [ ] `remove_validator.rs`
- [ ] `set_validator.rs`
- [ ] `update_config_validator.rs`

## 迁移完成后删除的文件

### 目录
```
❌ src/query/context/          # 整个目录
```

### 文件
```
❌ src/query/context/mod.rs
❌ src/query/context/ast_context.rs
❌ src/query/context/symbol/mod.rs
❌ src/query/context/symbol/symbol_table.rs
❌ src/query/context/symbol/symbol_entry.rs
```

## 验证命令

```bash
# 编译检查
cargo check --lib

# 运行测试
cargo test --lib

# 代码检查
cargo clippy --lib

# 格式化
cargo fmt
```

## 迁移步骤速查

每个验证器需要修改：

1. **导入语句**
   - 删除: `use crate::query::context::ast::AstContext;`
   - 添加: `use std::sync::Arc; use crate::query::QueryContext; use crate::query::parser::ast::Stmt;`

2. **方法签名**
   - 旧: `fn validate(&mut self, ast: &mut AstContext) -> Result<ValidationResult, ValidationError>`
   - 新: `fn validate(&mut self, stmt: &Stmt, qctx: Arc<QueryContext>) -> Result<ValidationResult, ValidationError>`

3. **空间检查**
   - 旧: `ast.query_context().is_none()`
   - 新: `qctx.space_info().is_none()`

4. **space_id 获取**
   - 旧: `ast.space().space_id.map(|id| id as u64).unwrap_or(0)`
   - 新: `qctx.space_info().map(|info| info.space_id).unwrap_or(0)`

5. **语句获取**
   - 删除: `let stmt = ast.sentence().ok_or_else(...)?;`
   - 直接使用参数: `stmt`

---

**最后更新**: 2026-02-21
**迁移指南**: `docs/validator_migration_guide.md`
