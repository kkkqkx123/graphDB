# Parser 模块任务列表

## 一、概述

本文档列出 Parser 模块所有待实现和待修复的任务项，基于代码分析得出。

## 二、高优先级任务

### 2.1 修复 AST 丢失问题

**文件**: `src/query/query_pipeline_manager.rs`

**问题描述**:
解析成功后，AST 被丢弃，后续处理阶段无法访问解析结果。

**当前代码**:
```rust
fn parse_query(&mut self, query_text: &str) -> DBResult<QueryAstContext> {
    let mut parser = Parser::new(query_text);
    match parser.parse() {
        Ok(_stmt) => {  // ← AST 被丢弃
            let ast = QueryAstContext::new(query_text);
            Ok(ast)
        }
        Err(e) => Err(...),
    }
}
```

**修复方案**:
返回 `(Stmt, QueryAstContext)` 元组，保留解析结果。

---

### 2.2 修复 StmtParser 的 PhantomData

**文件**: `src/query/parser/parser/stmt_parser.rs`

**问题描述**:
```rust
pub struct StmtParser<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> StmtParser<'a> {
    pub fn new(_ctx: &ParseContext<'a>) -> Self {
        Self {
            _phantom: std::marker::PhantomData,  // 未使用 ctx
        }
    }
}
```

**修复方案**:
- 移除 `_phantom`，存储实际使用的 context 引用
- 或保留为解析器状态容器

---

### 2.3 统一错误处理

**文件**: `src/query/executor/factory.rs`

**问题描述**:
多处使用 `.ok()` 静默忽略解析错误。

**当前代码** (约 11 处):
```rust
crate::query::parser::expressions::parse_expression_from_string(f).ok()
```

**修复方案**:
替换为 proper error handling：
```rust
let expr = parse_expression_from_string(f)
    .map_err(|e| DBError::Query(format!("表达式解析失败: {}", e)))?;
```

---

## 三、中优先级任务

### 3.1 更新 AstContext 存储解析结果

**文件**: `src/query/context/ast/base.rs`

**当前状态**:
- `sentence: Option<Stmt>` 字段存在
- 但查询管道中未正确设置

**修复方案**:
- 添加 `set_statement()` 方法
- 确保 `parse_query` 正确调用

---

### 3.2 完善解析器实现

**文件**: `src/query/parser/parser/stmt_parser.rs`

**待完善函数**:

| 函数名 | 状态 | 说明 |
|--------|------|------|
| `parse_match_statement` | ✅ 已实现 | |
| `parse_go_statement` | ✅ 已实现 | |
| `parse_create_statement` | ✅ 已实现 | |
| `parse_delete_statement` | ✅ 已实现 | |
| `parse_update_statement` | ✅ 已实现 | |
| `parse_use_statement` | ✅ 已实现 | |
| `parse_show_statement` | ✅ 已实现 | |
| `parse_explain_statement` | ✅ 已实现 | |
| `parse_lookup_statement` | ✅ 已实现 | |
| `parse_fetch_statement` | ⚠️ 部分实现 | target 硬编码 |
| `parse_unwind_statement` | ✅ 已实现 | |
| `parse_merge_statement` | ⚠️ 部分实现 | 缺少 ON MATCH/ON CREATE 子句 |
| `parse_insert_statement` | ⚠️ 部分实现 | target 硬编码 |
| `parse_return_statement` | ✅ 已实现 | |
| `parse_with_statement` | ✅ 已实现 | |
| `parse_set_statement` | ✅ 已实现 | |
| `parse_remove_statement` | ✅ 已实现 | |
| `parse_pipe_statement` | ✅ 已实现 | |
| `parse_drop_statement` | ❌ 未找到 | 需要检查实现 |
| `parse_desc_statement` | ❌ 未找到 | 需要检查实现 |
| `parse_alter_statement` | ❌ 未找到 | 需要检查实现 |
| `parse_change_password_statement` | ❌ 未找到 | 需要检查实现 |

---

### 3.3 修复测试代码中的 panic

**文件**: `src/query/parser/expressions/expression_converter.rs`

**当前代码**:
```rust
#[test]
fn test_convert_constant_expression() {
    let result = convert_ast_to_graph_expression(&expr);
    assert_eq!(result.unwrap(), GraphExpression::Literal(...));
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_expression() {
        panic!("Expected Literal(Int(42)), got {:?}", result);
    }
}
```

**修复方案**:
- 使用 `assert_eq!` 替代 `panic!`
- 使用 `expect` 替代 `unwrap`

---

## 四、低优先级任务

### 4.1 移除未使用的导入

**检查文件**:
- `src/query/parser/parser/mod.rs`
- `src/query/parser/ast/stmt.rs`
- `src/query/parser/ast/expression.rs`

---

### 4.2 添加更多测试用例

**建议测试覆盖**:
1. 基础查询解析 (MATCH, GO, CREATE, etc.)
2. 复杂表达式解析
3. 错误恢复机制
4. 边界情况处理

---

### 4.3 完善文档注释

**需要完善注释的文件**:
- `src/query/parser/parser/mod.rs`
- `src/query/parser/parser/stmt_parser.rs`
- `src/query/parser/parser/expr_parser.rs`

---

## 五、任务清单

### Phase 1: 核心修复 (P0)

- [x] 修复 AST 丢失问题 (query_pipeline_manager.rs)
- [x] 更新 AstContext 存储解析结果 (ast/base.rs)
- [x] 修复 StmtParser PhantomData (stmt_parser.rs)
- [x] 统一错误处理 - 替换 11 处 .ok() (factory.rs)

### Phase 2: 解析器完善 (P1)

- [x] 完善 parse_fetch_statement (stmt_parser.rs) - 支持 Vertices 和 Edges
- [x] 完善 parse_merge_statement (stmt_parser.rs) - 支持 ON CREATE/ON MATCH 子句
- [x] 完善 parse_insert_statement (stmt_parser.rs) - 支持完整 INSERT VERTEX/EDGE 语法
- [x] 完善 parse_drop_statement (stmt_parser.rs) - 支持可选 IN/ON 子句
- [x] 完善 parse_desc_statement (stmt_parser.rs) - 支持可选 IN 子句
- [x] 检查 parse_alter_statement (stmt_parser.rs) - 已完整
- [x] 检查 parse_change_password_statement (stmt_parser.rs) - 已完整

### Phase 3: 测试与文档 (P2)

- [ ] 修复 expression_converter.rs 中的 panic (tests)
- [ ] 移除未使用的导入
- [ ] 添加解析器单元测试
- [ ] 完善文档注释

---

## 六、验收标准

1. **AST 传递**: 解析后的 Stmt 能正确传递给 Validator 和 Planner
2. **错误处理**: 不再使用 `.ok()` 静默忽略错误
3. **编译通过**: `cargo check` 无错误
4. **测试通过**: `cargo test` 通过率 100%

---

## 七、参考文档

- [PARSER_IMPROVEMENT_PLAN.md](../parser/__analysis__/PARSER_IMPROVEMENT_PLAN.md)
- Nebula Graph 3.8.0 parser 实现
