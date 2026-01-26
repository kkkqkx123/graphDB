# Parser 模块问题清单与修改方案

## 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 2.1 | 错误信息缺乏位置详细信息 | 中 | 可用性问题 | 待修复 |
| 2.2 | 表达式解析器不支持所有 NGQL 语法 | 中 | 功能缺失 | 待修复 |
| 2.3 | Token 类型定义不完整 | 低 | 完整性问题 | 待修复 |
| 2.4 | 词法分析器错误处理不够友好 | 低 | 代码质量 | 待修复 |
| 2.5 | Parser 与 QueryAstContext 耦合较紧 | 低 | 设计问题 | 待修复 |

---

## 详细问题分析

### 问题 2.1: 错误信息缺乏位置详细信息

**涉及文件**: `src/query/parser/mod.rs`

**当前实现**:
```rust
fn parse_into_context(
    &mut self,
    query_text: &str,
) -> DBResult<crate::query::context::ast::QueryAstContext> {
    let mut parser = Parser::new(query_text);
    match parser.parse() {
        Ok(stmt) => {
            let mut ast = crate::query::context::ast::QueryAstContext::new(query_text);
            ast.set_statement(stmt);
            Ok(ast)
        }
        Err(e) => Err(DBError::Query(crate::core::error::QueryError::ParseError(
            format!("解析失败: {}", e),  // 错误信息不包含位置
        ))),
    }
}
```

**问题**:
- 用户难以定位语法错误的具体位置
- 无法显示行号和列号
- 调试复杂查询困难

**缺失信息**:
```
当前: "解析失败: unexpected token"
期望: "解析失败 at line 5, column 15: unexpected token 'WHERE'"
```

---

### 问题 2.2: 表达式解析器支持不完整

**涉及文件**: `src/query/parser/expressions/`

**当前支持**:
- 基本算术运算
- 比较运算
- 逻辑运算（AND、OR、NOT）
- 函数调用（部分）

**缺失功能**:
```
✓ 支持:
  - a + b
  - a = 1
  - name CONTAINS 'test'
  
✗ 不支持:
  - CASE WHEN a = 1 THEN 'one' WHEN a = 2 THEN 'two' ELSE 'other' END
  - [x IN list WHERE x > 0 | x * 2]
  - (n)-[*1..5]->(m)  // 路径表达式
  - coalesce(a, b, c)
  - reduce(sum = 0, x IN list | sum + x)
```

**影响**:
- 用户无法使用复杂的条件表达式
- 与 NebulaGraph 语法不完全兼容

---

### 问题 2.3: Token 类型定义不完整

**涉及文件**: `src/query/parser/core/token.rs`

**当前实现**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // 基础类型
    Identifier,
    String,
    Number,
    
    // 关键字
    Match,
    Return,
    Where,
    // ... 部分关键字
    
    // 运算符
    Plus,
    Minus,
    // ... 部分运算符
    
    // 特殊符号
    LeftParen,
    RightParen,
    // ...
    
    // 缺失的关键字和运算符
    // Case, When, Then, Else, End
    // In, Contains, StartsWith, EndsWith
    // Assign (=), NotEqual (<>), GreaterEqual (>=), etc.
}
```

**缺失的 Token**:
- 路径运算符：`*`（多跳）、`+`（一或多跳）
- 集合运算符：`IN`
- 字符串操作：`CONTAINS`、`STARTS WITH`、`ENDS WITH`
- 类型转换：`::`
- 列表推导：`[ ... | ... ]`
- 参数：`$param`

---

### 问题 2.4: 词法分析器错误处理不够友好

**涉及文件**: `src/query/parser/lexer/lexer.rs`

**当前实现**:
```rust
impl Lexer {
    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        // ... 扫描逻辑
        if self.current_char.is_none() {
            return Err(LexerError::UnexpectedEndOfInput);  // 错误信息简单
        }
        // ...
    }
}
```

**问题**:
- 错误信息过于简单
- 没有位置信息
- 无法区分不同类型的词法错误

---

## 修改方案

### 修改方案 2.1: 改进错误信息

**预估工作量**: 1 人天

**修改目标**: 让错误信息包含位置详情

**修改步骤**:

**步骤 1**: 增强 ParseError 定义

```rust
// src/query/parser/core/error.rs

use thiserror::Error;

/// 解析错误
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("语法错误 at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
        expected: Vec<String>,
        found: Option<String>,
    },
    
    #[error("词法错误 at line {line}, column {column}: {message}")]
    LexerError {
        line: usize,
        column: usize,
        message: String,
        context: Option<String>,
    },
    
    #[error("解析失败: {message}")]
    ParseError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("意外的输入结束")]
    UnexpectedEndOfInput {
        line: usize,
        column: usize,
    },
}

impl ParseError {
    pub fn syntax_error(
        line: usize,
        column: usize,
        message: String,
        expected: Vec<String>,
        found: Option<String>,
    ) -> Self {
        ParseError::SyntaxError {
            line,
            column,
            message,
            expected,
            found,
        }
    }
    
    pub fn lexer_error(line: usize, column: usize, message: String) -> Self {
        ParseError::LexerError {
            line,
            column,
            message,
            context: None,
        }
    }
    
    pub fn with_context(mut self, context: String) -> Self {
        match &mut self {
            ParseError::LexerError { context: ctx, .. } => {
                *ctx = Some(context);
            }
            _ => {}
        }
        self
    }
}
```

**步骤 2**: 增强 Lexer 以追踪位置

```rust
// src/query/parser/lexer/lexer.rs

#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
    current_char: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Self {
            input,
            position: 0,
            line: 1,
            column: 0,
            current_char: None,
        };
        lexer.bump();
        lexer
    }
    
    pub fn location(&self) -> (usize, usize) {
        (self.line, self.column)
    }
    
    pub fn peek_location(&self, offset: usize) -> (usize, usize) {
        // 返回 lookahead 位置
        let mut line = self.line;
        let mut column = self.column;
        let mut pos = self.position;
        
        for _ in 0..offset.min(self.input.len() - pos) {
            if self.input.as_bytes()[pos] == b'\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
            pos += 1;
        }
        
        (line, column)
    }
    
    fn error<T>(&self, message: String) -> Result<T, ParseError> {
        let (line, column) = self.location();
        Err(ParseError::lexer_error(line, column, message))
    }
}
```

**步骤 3**: 更新 Parser 以使用新错误类型

```rust
// src/query/parser/parser/stmt_parser.rs

impl<'a> StmtParser<'a> {
    pub fn parse_match(&mut self) -> Result<Stmt, ParseError> {
        // ...
        if self.current_token.kind != TokenKind::LeftParen {
            let (line, column) = self.lexer.location();
            return Err(ParseError::syntax_error(
                line,
                column,
                format!("Expected '(', found '{}'", self.current_token.text),
                vec!["(".to_string()],
                Some(self.current_token.text.clone()),
            ));
        }
        // ...
    }
}
```

---

### 修改方案 2.2: 扩展表达式解析器

**预估工作量**: 5-7 人天

**修改目标**: 支持所有 NGQL 表达式语法

**修改步骤**:

**步骤 1**: 添加缺失的表达式类型

```rust
// src/query/parser/ast/types.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Value),
    Variable(String),
    Property {
        object: Box<Expression>,
        property: String,
    },
    // ... 现有类型
    
    // 新增类型
    Case {
        test_expr: Option<Box<Expression>>,
        when_then_pairs: Vec<(Expression, Expression)>,
        default: Option<Box<Expression>>,
    },
    
    ListComprehension {
        variable: String,
        source: Box<Expression>,
        filter: Option<Box<Expression>>,
        map: Option<Box<Expression>>,
    },
    
    PathPattern {
        start: Box<Expression>,
        edge: PathEdge,
        end: Box<Expression>,
    },
    
    Function {
        name: String,
        args: Vec<Expression>,
        distinct: bool,
    },
    
    Coalesce {
        args: Vec<Expression>,
    },
    
    Reduce {
        accumulator: String,
        init: Box<Expression>,
        variable: String,
        source: Box<Expression>,
        expression: Box<Expression>,
    },
}
```

**步骤 2**: 添加 CASE 表达式解析

```rust
// src/query/parser/parser/expr_parser.rs

impl<'a> ExprParser<'a> {
    pub fn parse_case_expression(&mut self) -> Result<Expression, ParseError> {
        // 解析 CASE [expression] WHEN ... THEN ... ELSE ... END
        self.expect(TokenKind::Case)?;
        
        let test_expr = if self.peek_token().kind != TokenKind::When {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        
        let mut when_then_pairs = Vec::new();
        while self.peek_token().kind == TokenKind::When {
            self.expect(TokenKind::When)?;
            let when_expr = self.parse_expression()?;
            self.expect(TokenKind::Then)?;
            let then_expr = self.parse_expression()?;
            when_then_pairs.push((when_expr, then_expr));
        }
        
        let default = if self.peek_token().kind == TokenKind::Else {
            self.expect(TokenKind::Else)?;
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        
        self.expect(TokenKind::End)?;
        
        Ok(Expression::Case {
            test_expr,
            when_then_pairs,
            default,
        })
    }
    
    pub fn parse_list_comprehension(&mut self) -> Result<Expression, ParseError> {
        // 解析 [variable IN source | expression]
        self.expect(TokenKind::LeftBracket)?;
        
        // 解析变量
        let variable = match self.peek_token().kind {
            TokenKind::Identifier => {
                let var = self.current_token.text.clone();
                self.bump();
                var
            }
            _ => {
                return Err(ParseError::syntax_error(
                    self.lexer.location().0,
                    self.lexer.location().1,
                    "Expected variable in list comprehension".to_string(),
                    vec!["identifier".to_string()],
                    Some(self.current_token.text.clone()),
                ));
            }
        };
        
        self.expect(TokenKind::In)?;
        let source = Box::new(self.parse_expression()?);
        
        // 解析过滤和映射
        let (filter, map) = if self.peek_token().kind == TokenKind::Pipe {
            self.expect(TokenKind::Pipe)?;
            let map_expr = self.parse_expression()?;
            (None, Box::new(map_expr))
        } else if self.peek_token().kind == TokenKind::Where {
            self.expect(TokenKind::Where)?;
            let filter_expr = Box::new(self.parse_expression()?);
            let map_expr = if self.peek_token().kind == TokenKind::Pipe {
                self.expect(TokenKind::Pipe)?;
                Box::new(self.parse_expression()?)
            } else {
                Box::new(Expression::Variable(variable.clone()))
            };
            (Some(filter_expr), map_expr)
        } else {
            (None, Box::new(Expression::Variable(variable.clone())))
        };
        
        self.expect(TokenKind::RightBracket)?;
        
        Ok(Expression::ListComprehension {
            variable,
            source,
            filter,
            map: Some(map_expr),
        })
    }
}
```

**步骤 3**: 添加函数解析

```rust
impl<'a> ExprParser<'a> {
    pub fn parse_function_call(&mut self) -> Result<Expression, ParseError> {
        let name = self.current_token.text.clone();
        self.expect(TokenKind::Identifier)?;
        
        self.expect(TokenKind::LeftParen)?;
        
        let mut args = Vec::new();
        if self.peek_token().kind != TokenKind::RightParen {
            // 检查 DISTINCT
            let distinct = if self.peek_token().kind == TokenKind::Distinct {
                self.bump();
                true
            } else {
                false
            };
            
            // 解析参数列表
            loop {
                args.push(self.parse_expression()?);
                if self.peek_token().kind == TokenKind::Comma {
                    self.expect(TokenKind::Comma)?;
                } else {
                    break;
                }
            }
        }
        
        self.expect(TokenKind::RightParen)?;
        
        // 特殊函数处理
        match name.to_uppercase().as_str() {
            "COALESCE" => {
                return Ok(Expression::Coalesce { args });
            }
            "REDUCE" => {
                // REDUCE(accumulator = init, variable IN source | expression)
                return self.parse_reduce_expression(name, args);
            }
            _ => {}
        }
        
        Ok(Expression::Function {
            name,
            args,
            distinct,
        })
    }
    
    fn parse_reduce_expression(
        &mut self,
        _name: String,
        _args: Vec<Expression>,
    ) -> Result<Expression, ParseError> {
        // REDUCE 的特殊解析逻辑
        // 格式: REDUCE(accumulator = 0, x IN list | accumulator + x)
        unimplemented!("REDUCE expression parsing")
    }
}
```

---

### 修改方案 2.3: 完善 Token 类型

**预估工作量**: 1 人天

**修改代码**:

```rust
// src/query/parser/core/token.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // 基础类型
    Identifier,
    Parameter,          // $param
    String,
    Integer,
    Float,
    Boolean,
    
    // 关键字 - 数据操作
    Match,
    Optional,           // OPTIONAL
    Return,
    Yield,
    Where,
    With,
    Order,              // ORDER
    By,                 // BY
    Skip,
    Limit,
    Sample,
    
    // 关键字 - 模式匹配
    As,
    Node,               // (n) 内部使用
    Edge,               // -[e]- 内部使用
    
    // 关键字 - 条件
    Case,
    When,
    Then,
    Else,
    End,
    
    // 关键字 - 集合
    In,
    Contains,
    Starts,             // STARTS
    WithKeyword,        // WITH (需要区分)
    Ends,               // ENDS
    
    // 关键字 - 路径
    Single,             // + (一或多跳)
    ZeroOrMore,         // * (多跳)
    Until,              // .. (范围)
    
    // 关键字 - 聚合
    Collect,
    Count,
    Sum,
    Avg,
    Min,
    Max,
    AggregationFunction,
    
    // 关键字 - 类型
    Is,
    Of,
    
    // 运算符 - 比较
    Equal,              // =
    Assign,             // := (赋值)
    NotEqual,           // <> 或 !=
    LessThan,           // <
    GreaterThan,        // >
    LessEqual,          // <=
    GreaterEqual,       // >=
    
    // 运算符 - 算术
    Plus,               // +
    Minus,              // -
    Multiply,           // *
    Divide,             // /
    Modulo,             // %
    Power,              // ^
    
    // 运算符 - 逻辑
    And,
    Or,
    Not,
    Xor,
    
    // 特殊符号
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,              // :
    DoubleColon,        // ::
    Semicolon,
    Dot,                // .
    Pipe,               // |
    At,                 // @
    Undirected,         // ~ (无向边)
    
    // 特殊
    Eoi,                // End of Input
    Unknown,
}
```

---

### 修改方案 2.4: 改进词法分析器错误处理

**预估工作量**: 1 人天

**修改代码**:

```rust
// src/query/parser/lexer/lexer.rs

#[derive(Debug, Error)]
pub enum LexerErrorKind {
    UnexpectedCharacter,
    UnterminatedString,
    InvalidNumber,
    UnterminatedBlockComment,
}

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
    current_char: Option<char>,
    errors: Vec<ParseError>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Self {
            input,
            position: 0,
            line: 1,
            column: 0,
            current_char: None,
            errors: Vec::new(),
        };
        lexer.bump();
        lexer
    }
    
    pub fn had_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn take_errors(&mut self) -> Vec<ParseError> {
        std::mem::take(&mut self.errors)
    }
    
    fn add_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }
    
    pub fn next_token(&mut self) -> Result<Token, ParseError> {
        // 跳过空白
        while self.current_char.map(|c| c.is_whitespace()).unwrap_or(false) {
            self.bump();
        }
        
        if self.current_char.is_none() {
            return Ok(Token::new(TokenKind::Eoi, "".to_string()));
        }
        
        let (line, column) = self.location();
        
        // 识别字符串
        if self.current_char == Some('\'') || self.current_char == Some('"') {
            return self.scan_string().map_err(|e| {
                self.add_error(e.clone());
                e
            });
        }
        
        // 识别数字
        if self.current_char.map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return self.scan_number().map_err(|e| {
                self.add_error(e.clone());
                e
            });
        }
        
        // 识别标识符和关键字
        if self.current_char.map(|c| c.is_ascii_alphabetic() || c == '_' || c == '$').unwrap_or(false) {
            return self.scan_identifier_or_keyword();
        }
        
        // 识别运算符和符号
        let token = self.scan_operator_or_symbol()?;
        
        // 记录错误但继续扫描
        if let Ok(ref t) = token {
            if t.kind == TokenKind::Unknown {
                let error = ParseError::lexer_error(
                    line,
                    column,
                    format!("Unexpected character: '{}'", self.current_char.unwrap()),
                );
                self.add_error(error);
            }
        }
        
        token
    }
    
    fn scan_string(&mut self) -> Result<Token, ParseError> {
        let quote_char = self.current_char.unwrap();
        self.bump();
        
        let start = self.position;
        let mut value = String::new();
        
        while let Some(c) = self.current_char {
            if c == quote_char {
                // 检查转义
                if self.input.as_bytes().get(self.position - 1) == Some(&b'\\') {
                    // 处理转义序列
                    value.push(self.handle_escape_sequence()?);
                } else {
                    // 结束字符串
                    break;
                }
            } else if c == '\n' {
                return Err(ParseError::lexer_error(
                    self.line,
                    self.column,
                    "Unterminated string literal".to_string(),
                ));
            } else {
                value.push(c);
            }
            self.bump();
        }
        
        let value = self.input[start..self.position - 1].to_string();
        self.bump(); // 跳过结束引号
        
        Ok(Token::new(TokenKind::String, value))
    }
    
    fn handle_escape_sequence(&mut self) -> Result<char, ParseError> {
        self.bump(); // 跳过反斜杠
        match self.current_char {
            Some('n') => Ok('\n'),
            Some('t') => Ok('\t'),
            Some('r') => Ok('\r'),
            Some('\'') => Ok('\''),
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('0') => Ok('\0'),
            Some('u') if self.peek_unicode() => {
                // Unicode 转义
                self.scan_unicode_escape()
            }
            Some(c) => {
                Err(ParseError::lexer_error(
                    self.line,
                    self.column,
                    format!("Unknown escape sequence: \\{}", c),
                ))
            }
            None => Err(ParseError::lexer_error(
                self.line,
                self.column,
                "Incomplete escape sequence".to_string(),
            )),
        }
    }
}
```

---

## 修改优先级

| 序号 | 修改方案 | 优先级 | 预估工作量 | 依赖 |
|------|----------|--------|------------|------|
| 2.1 | 改进错误信息 | 高 | 1 人天 | 无 |
| 2.2 | 扩展表达式解析器 | 高 | 5-7 人天 | 2.3 |
| 2.3 | 完善 Token 类型 | 中 | 1 人天 | 无 |
| 2.4 | 改进词法分析器错误处理 | 中 | 1 人天 | 2.1 |

---

## 测试建议

### 测试用例 1: 错误位置信息

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_error_with_location() {
        let result = Parser::new("MATCH (n WHERE n.age >").parse();
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        
        // 验证错误包含位置信息
        match error {
            ParseError::SyntaxError { line, column, .. } => {
                assert!(line > 0);
                assert!(column > 0);
            }
            _ => panic!("Expected SyntaxError"),
        }
    }
    
    #[test]
    fn test_case_expression_parsing() {
        let query = "MATCH (n) RETURN CASE n.status WHEN 'active' THEN 1 ELSE 0 END AS status_value";
        let result = Parser::new(query).parse();
        
        assert!(result.is_ok());
        // 验证 CASE 表达式被正确解析
    }
    
    #[test]
    fn test_list_comprehension_parsing() {
        let query = "RETURN [x IN [1,2,3,4,5] WHERE x > 2 | x * 10] AS result";
        let result = Parser::new(query).parse();
        
        assert!(result.is_ok());
    }
}
```

---

## 风险与注意事项

### 风险 1: 表达式解析器复杂度

- **风险**: 新增表达式类型可能引入解析歧义
- **缓解措施**: 充分的单元测试，覆盖边界情况
- **实现**: 使用解析器组合子处理复杂表达式

### 风险 2: 性能影响

- **风险**: 错误位置追踪可能影响词法分析性能
- **缓解措施**: 仅在需要时计算位置信息
- **实现**: 位置信息惰性计算
