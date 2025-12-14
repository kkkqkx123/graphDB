# GraphDB 查询解析器重构实施计划

## 1. 重构架构设计

### 1.1 新的 AST 设计

**核心枚举定义：**
```rust
// src/query/parser/ast_v2/mod.rs

/// 简化的 AST 节点枚举
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Constant(ConstantExpr),
    Variable(VariableExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    FunctionCall(FunctionCallExpr),
    PropertyAccess(PropertyAccessExpr),
    List(ListExpr),
    Map(MapExpr),
    Case(CaseExpr),
    Subscript(SubscriptExpr),
    Predicate(PredicateExpr),
}

/// 语句枚举
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Query(QueryStmt),
    Create(CreateStmt),
    Match(MatchStmt),
    Delete(DeleteStmt),
    Update(UpdateStmt),
    Go(GoStmt),
    Fetch(FetchStmt),
    Use(UseStmt),
    Show(ShowStmt),
    Explain(ExplainStmt),
    Lookup(LookupStmt),      // 新增
    Subgraph(SubgraphStmt),   // 新增
    FindPath(FindPathStmt),   // 新增
}
```

### 1.2 简化的结构体定义

**表达式结构体示例：**
```rust
// src/query/parser/ast_v2/expr.rs

#[derive(Debug, Clone, PartialEq)]
pub struct ConstantExpr {
    pub span: Span,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub span: Span,
    pub left: Box<Expr>,
    pub op: BinaryOp,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallExpr {
    pub span: Span,
    pub name: String,
    pub args: Vec<Expr>,
    pub distinct: bool,
}
```

### 1.3 简化的访问者模式

**新的访问者接口：**
```rust
// src/query/parser/ast_v2/visitor.rs

pub trait ExprVisitor {
    type Result;
    
    fn visit_expr(&mut self, expr: &Expr) -> Self::Result {
        match expr {
            Expr::Constant(e) => self.visit_constant(e),
            Expr::Binary(e) => self.visit_binary(e),
            Expr::Unary(e) => self.visit_unary(e),
            Expr::FunctionCall(e) => self.visit_function_call(e),
            // ... 其他表达式类型
            _ => self.visit_default(expr),
        }
    }
    
    fn visit_constant(&mut self, expr: &ConstantExpr) -> Self::Result;
    fn visit_binary(&mut self, expr: &BinaryExpr) -> Self::Result;
    fn visit_unary(&mut self, expr: &UnaryExpr) -> Self::Result;
    fn visit_function_call(&mut self, expr: &FunctionCallExpr) -> Self::Result;
    
    fn visit_default(&mut self, _expr: &Expr) -> Self::Result {
        // 默认实现
        unimplemented!()
    }
}

pub trait StmtVisitor {
    type Result;
    
    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::Result {
        match stmt {
            Stmt::Query(s) => self.visit_query(s),
            Stmt::Create(s) => self.visit_create(s),
            Stmt::Match(s) => self.visit_match(s),
            // ... 其他语句类型
            _ => self.visit_default(stmt),
        }
    }
    
    fn visit_query(&mut self, stmt: &QueryStmt) -> Self::Result;
    fn visit_create(&mut self, stmt: &CreateStmt) -> Self::Result;
    fn visit_match(&mut self, stmt: &MatchStmt) -> Self::Result;
    
    fn visit_default(&mut self, _stmt: &Stmt) -> Self::Result {
        // 默认实现
        unimplemented!()
    }
}
```

## 2. 渐进式迁移策略

### 2.1 阶段1：创建兼容性层

**兼容性包装器：**
```rust
// src/query/parser/compat/mod.rs

/// 兼容性包装器 - 将新 AST 转换为旧 AST
pub struct AstCompat;

impl AstCompat {
    /// 将新表达式转换为旧表达式
    pub fn convert_expr(expr: &Expr) -> Box<dyn Expression> {
        match expr {
            Expr::Constant(e) => Box::new(ConstantExpr::new(e.value.clone(), e.span)),
            Expr::Binary(e) => {
                let left = Self::convert_expr(&e.left);
                let right = Self::convert_expr(&e.right);
                Box::new(BinaryExpr::new(left, e.op, right, e.span))
            }
            // ... 其他转换
            _ => panic!("Unsupported expression type"),
        }
    }
    
    /// 将旧表达式转换为新表达式
    pub fn convert_expr_back(expr: &dyn Expression) -> Expr {
        // 实现反向转换逻辑
        unimplemented!()
    }
}
```

### 2.2 阶段2：并行运行

**双模式解析器：**
```rust
// src/query/parser/parser_v2/mod.rs

pub struct ParserV2 {
    lexer: Lexer,
    compat_mode: bool,  // 是否启用兼容模式
}

impl ParserV2 {
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
            compat_mode: false,
        }
    }
    
    /// 解析表达式（新版本）
    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        if self.compat_mode {
            // 使用兼容模式
            let old_expr = self.parse_expr_old()?;
            Ok(AstCompat::convert_expr_back(&*old_expr))
        } else {
            // 使用新版本解析
            self.parse_expr_new()
        }
    }
    
    /// 新版本表达式解析
    fn parse_expr_new(&mut self) -> Result<Expr, ParseError> {
        // 实现新版本解析逻辑
        unimplemented!()
    }
    
    /// 旧版本表达式解析（兼容）
    fn parse_expr_old(&mut self) -> Result<Box<dyn Expression>, ParseError> {
        // 调用现有解析逻辑
        unimplemented!()
    }
}
```

### 2.3 阶段3：完全迁移

**最终迁移步骤：**
1. 更新所有使用解析器的代码
2. 移除兼容性层
3. 删除旧版本代码
4. 性能优化和测试

## 3. 具体实施步骤

### 3.1 第一周：基础架构

**任务清单：**
- [ ] 创建 `src/query/parser/ast_v2` 目录结构
- [ ] 实现核心枚举和结构体定义
- [ ] 创建简化的访问者接口
- [ ] 编写基础单元测试

**代码示例：**
```rust
// 创建新的 AST 模块结构
src/query/parser/
├── ast_v2/
│   ├── mod.rs          # 模块导出
│   ├── expr.rs         # 表达式定义
│   ├── stmt.rs         # 语句定义
│   ├── visitor.rs      # 访问者模式
│   └── tests/          # 测试文件
├── compat/
│   ├── mod.rs          # 兼容性层
│   └── converter.rs     # 转换器实现
└── parser_v2/
    ├── mod.rs          # 新解析器
    └── expr_parser.rs   # 表达式解析
```

### 3.2 第二周：表达式迁移

**任务清单：**
- [ ] 迁移所有表达式类型
- [ ] 实现表达式工厂
- [ ] 创建兼容性包装器
- [ ] 编写迁移测试

**代码示例：**
```rust
// 表达式工厂实现
pub struct ExprFactory;

impl ExprFactory {
    pub fn constant(value: Value, span: Span) -> Expr {
        Expr::Constant(ConstantExpr { value, span })
    }
    
    pub fn binary(left: Expr, op: BinaryOp, right: Expr, span: Span) -> Expr {
        Expr::Binary(BinaryExpr {
            span,
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }
    
    // ... 其他工厂方法
}
```

### 3.3 第三周：语句迁移

**任务清单：**
- [ ] 迁移所有语句类型
- [ ] 实现语句解析器
- [ ] 创建语句访问者
- [ ] 集成测试

**代码示例：**
```rust
// 新版本语句解析
impl ParserV2 {
    pub fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let token = self.lexer.peek()?;
        
        match token.kind {
            TokenKind::Match => self.parse_match_stmt(),
            TokenKind::Create => self.parse_create_stmt(),
            TokenKind::Delete => self.parse_delete_stmt(),
            // ... 其他语句类型
            _ => self.parse_query_stmt(),
        }
    }
    
    fn parse_match_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.lexer.expect(TokenKind::Match)?;
        
        let patterns = self.parse_patterns()?;
        let where_clause = self.parse_optional_where()?;
        let return_clause = self.parse_return()?;
        
        Ok(Stmt::Match(MatchStmt {
            span: Span::from_tokens(...),
            patterns,
            where_clause,
            return_clause,
        }))
    }
}
```

### 3.4 第四周：访问者模式迁移

**任务清单：**
- [ ] 迁移现有访问者实现
- [ ] 优化类型检查逻辑
- [ ] 性能基准测试
- [ ] 文档更新

**代码示例：**
```rust
// 简化的类型检查访问者
pub struct TypeCheckerV2 {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ExprVisitor for TypeCheckerV2 {
    type Result = ();
    
    fn visit_binary(&mut self, expr: &BinaryExpr) -> Self::Result {
        // 类型检查逻辑
        match expr.op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                // 检查操作数类型
                self.check_numeric_operands(expr);
            }
            BinaryOp::And | BinaryOp::Or => {
                // 检查布尔操作数
                self.check_boolean_operands(expr);
            }
            _ => {}
        }
        
        // 递归检查子表达式
        self.visit_expr(&expr.left);
        self.visit_expr(&expr.right);
    }
    
    // ... 其他访问方法
}
```

## 4. 测试策略

### 4.1 单元测试

**表达式测试：**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constant_expr() {
        let expr = ExprFactory::constant(Value::Int(42), Span::default());
        assert!(matches!(expr, Expr::Constant(_)));
    }
    
    #[test]
    fn test_binary_expr() {
        let left = ExprFactory::constant(Value::Int(5), Span::default());
        let right = ExprFactory::constant(Value::Int(3), Span::default());
        let expr = ExprFactory::binary(left, BinaryOp::Add, right, Span::default());
        
        assert!(matches!(expr, Expr::Binary(_)));
    }
}
```

### 4.2 集成测试

**解析器测试：**
```rust
#[test]
fn test_parser_compatibility() {
    let input = "MATCH (n) RETURN n";
    
    // 新旧解析器对比测试
    let mut old_parser = Parser::new(input);
    let old_result = old_parser.parse_query();
    
    let mut new_parser = ParserV2::new(input);
    let new_result = new_parser.parse_query();
    
    // 验证结果等价性
    assert_eq!(old_result.is_ok(), new_result.is_ok());
}
```

### 4.3 性能测试

**基准测试：**
```rust
#[bench]
fn bench_expression_parsing(b: &mut Bencher) {
    let input = "(a + b) * (c - d) / e";
    
    b.iter(|| {
        let mut parser = ParserV2::new(input);
        parser.parse_expr().unwrap()
    });
}
```

## 5. 风险缓解措施

### 5.1 代码质量保证

**代码审查：**
- 每个提交都需要代码审查
- 重点检查类型安全和性能
- 确保向后兼容性

**自动化测试：**
- 建立完整的测试套件
- 集成持续集成流程
- 性能回归测试

### 5.2 回滚策略

**功能开关：**
```rust
// 配置开关，支持快速回滚
#[derive(Clone)]
pub struct ParserConfig {
    pub use_new_parser: bool,
    pub enable_compat_mode: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            use_new_parser: false,  // 默认使用旧版本
            enable_compat_mode: true,
        }
    }
}
```

**版本控制：**
- 使用特性分支开发
- 保留旧版本代码直到稳定
- 支持快速回滚到旧版本

## 6. 预期成果验收标准

### 6.1 技术指标

**性能指标：**
- 解析速度提升 20% 以上
- 内存使用减少 30% 以上
- 编译时间减少 15% 以上

**代码质量指标：**
- 代码行数减少 40% 以上
- 测试覆盖率达到 85% 以上
- 类型安全错误减少 90% 以上

### 6.2 功能完整性

**语法支持：**
- 支持 nebula-graph 95% 的查询语法
- 新增 LOOKUP、SUBGRAPH、FIND PATH 语句
- 错误处理覆盖率达到 90% 以上

## 7. 总结

通过采用枚举+结构体的简化设计，结合渐进式迁移策略，可以安全高效地重构 GraphDB 查询解析器。该方案既保证了代码质量，又确保了项目的稳定性。

建议按照实施计划分阶段推进，每个阶段完成后进行充分的测试和验证，确保重构过程的平稳进行。