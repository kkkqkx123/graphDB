# Parser 模块重构设计方案

## 一、背景与目标

### 1.1 当前问题

经过对 `src/query/parser` 目录的全面分析，发现以下核心问题：

1. **Lexer 实例重复**：解析单个查询时可能创建多个 Lexer 实例
2. **错误处理分散**：LexError 和 ParseError 缺乏统一集成
3. **模块边界模糊**：expressions 目录定位不清
4. **API 暴露过多细节**：内部状态暴露给外部

### 1.2 重构目标

参考 NebulaGraph 的架构设计，制定以下目标：

1. **单例 Lexer 管理**：整个解析过程共享一个 Lexer 实例
2. **统一错误处理**：使用 `Result<T, ParseError>` 模式
3. **清晰的模块边界**：职责分明，依赖清晰
4. **优雅的 API**：最小化公开实现细节

## 二、架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                      Parser API 层                           │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                  Parser struct                         │  │
│  │  - shared_lexer: &Lexer (共享引用)                     │  │
│  │  - context: ParseContext (解析上下文)                  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      词法分析层                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                     Lexer                              │  │
│  │  - input: &str (借用，不持有所有权)                     │  │
│  │  - position: usize                                     │  │
│  │  - current_token: Token                                │  │
│  │  - errors: Vec<LexError>                               │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      语法分析层                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  StmtParser / ExprParser / ClauseParser               │  │
│  │  - lexer: &Lexer (共享引用)                            │  │
│  │  - context: &ParseContext                              │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      AST 层                                  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  - expr.rs (表达式定义)                                │  │
│  │  - stmt.rs (语句定义)                                  │  │
│  │  - pattern.rs (模式定义)                               │  │
│  │  - types.rs (类型定义)                                 │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心组件设计

#### 2.2.1 Lexer 设计

```rust
// 核心原则：Lexer 不持有输入字符串的所有权
pub struct Lexer<'a> {
    input: &'a str,           // 借用，不持有所有权
    position: usize,
    line: usize,
    column: usize,
    errors: Vec<LexError>,    // 收集词法错误
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            position: 0,
            line: 1,
            column: 0,
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Stmt, ParseError> {
        // 返回解析后的语句或错误
    }
}
```

#### 2.2.2 Parser 设计

```rust
// 核心原则：Parser 接收 Lexer 的引用，避免重复创建
pub struct Parser<'a> {
    lexer: &'a mut Lexer<'a>,  // 共享可变引用
    context: ParseContext,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer<'a>) -> Self {
        Parser {
            lexer,
            context: ParseContext::default(),
        }
    }

    pub fn parse(&mut self) -> Result<Stmt, ParseError> {
        self.parse_statement()
    }
}
```

#### 2.2.3 错误处理设计

```rust
// 核心原则：统一错误类型，支持错误链
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
    pub position: Position,
    pub context: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub hints: Vec<String>,
}

impl ParseError {
    pub fn with_hint(mut self, hint: String) -> Self {
        self.hints.push(hint);
        self
    }

    pub fn with_context<E: std::error::Error + Send + Sync + 'static>(
        mut self,
        context: E,
    ) -> Self {
        self.context = Some(Box::new(context));
        self
    }
}

// LexError 自动转换为 ParseError
impl From<LexError> for ParseError {
    fn from(lex_error: LexError) -> Self {
        ParseError::new(
            ParseErrorKind::LexicalError,
            lex_error.message,
            lex_error.position,
        )
    }
}
```

#### 2.2.4 ParseContext 设计

```rust
// 核心原则：统一管理解析过程中的配置和状态
#[derive(Debug, Default)]
pub struct ParseContext {
    pub config: ParseConfig,
    pub warnings: Vec<ParseWarning>,
    pub recursion_depth: usize,
    pub max_recursion_depth: usize,
    pub compat_mode: CompatMode,
}

#[derive(Debug, Clone, Copy)]
pub struct ParseConfig {
    pub max_query_length: usize,
    pub enable_recovery: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompatMode {
    Nebula,
    OpenCypher,
}
```

## 三、模块重组方案

### 3.1 当前目录结构

```
parser/
├── core/
│   ├── mod.rs
│   ├── error.rs
│   └── token.rs
├── lexer/
│   ├── mod.rs
│   └── lexer.rs
├── parser/
│   ├── mod.rs
│   ├── expr_parser.rs
│   ├── stmt_parser.rs
│   ├── clause_parser.rs
│   ├── pattern_parser.rs
│   └── utils.rs
├── expressions/
│   ├── mod.rs
│   └── expression_converter.rs
└── ast/
    ├── mod.rs
    ├── expr.rs
    ├── stmt.rs
    ├── pattern.rs
    ├── types.rs
    ├── utils.rs
    └── visitor.rs
```

### 3.2 重组后目录结构

```
parser/
├── mod.rs                    # 主入口，统一导出
├── core/
│   ├── mod.rs               # 核心类型导出
│   ├── error.rs             # 统一错误类型
│   ├── token.rs             # Token 和 TokenKind 定义
│   ├── position.rs          # Position 和 Span
│   └── context.rs           # ParseContext 和 ParseConfig
├── lexer/
│   ├── mod.rs               # Lexer 导出
│   ├── lexer.rs             # Lexer 实现
│   └── error.rs             # LexError 定义
├── parser/
│   ├── mod.rs               # Parser 主结构和导出
│   ├── expr_parser.rs       # 表达式解析
│   ├── stmt_parser.rs       # 语句解析
│   ├── clause_parser.rs     # 子句解析
│   ├── pattern_parser.rs    # 模式解析
│   └── utils.rs             # 解析工具函数
├── ast/
│   ├── mod.rs               # AST 模块导出
│   ├── expr.rs              # 表达式 AST
│   ├── stmt.rs              # 语句 AST
│   ├── pattern.rs           # 模式 AST
│   ├── types.rs             # 类型定义
│   ├── visitor.rs           # 访问者模式
│   └── utils.rs             # AST 工具函数
└── tests/
    ├── mod.rs
    └── integration_tests.rs # 集成测试
```

### 3.3 主要变更说明

#### 3.3.1 删除 expressions 目录

原 `expressions/expression_converter.rs` 的功能：
- 如果用于调试/测试目的，移入 `ast/utils.rs`
- 如果用于内部转换，考虑合并到 AST 模块

#### 3.3.2 拆分 core/context.rs

将 ParseContext 从 Parser 中分离，单独管理配置和状态。

#### 3.3.3 合并小文件

- `parser/utils.rs` → 根据功能分散到各解析器
- `ast/utils.rs` → 合并到 `ast/mod.rs` 或保留为工具模块

## 四、实现步骤

### 阶段一：基础架构重构（Week 1）

#### 步骤 1.1：创建统一的错误处理

```rust
// core/error.rs
#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    LexicalError,
    SyntaxError,
    UnexpectedToken,
    // ...
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
    pub position: Position,
    pub context: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub hints: Vec<String>,
}

impl std::fmt::Display for ParseError {
    // 实现格式化输出
}
```

#### 步骤 1.2：重构 Lexer

修改 `lexer/lexer.rs`，使其不持有输入字符串的所有权：

```rust
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
    errors: Vec<LexError>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            position: 0,
            line: 1,
            column: 0,
            errors: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn take_errors(&mut self) -> Vec<LexError> {
        std::mem::take(&mut self.errors)
    }
}
```

#### 步骤 1.3：创建 ParseContext

```rust
// core/context.rs
#[derive(Debug, Default)]
pub struct ParseContext {
    pub config: ParseConfig,
    pub warnings: Vec<ParseWarning>,
    pub recursion_depth: usize,
    pub max_recursion_depth: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct ParseConfig {
    pub max_query_length: usize,
    pub enable_recovery: bool,
}
```

### 阶段二：Parser 重构（Week 2）

#### 步骤 2.1：统一 Parser 结构

```rust
// parser/mod.rs
pub struct Parser<'a> {
    lexer: &'a mut Lexer<'a>,
    context: ParseContext,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer<'a>) -> Self {
        Parser {
            lexer,
            context: ParseContext::default(),
        }
    }

    pub fn with_context(lexer: &'a mut Lexer<'a>, context: ParseContext) -> Self {
        Parser { lexer, context }
    }

    pub fn parse(&mut self) -> Result<Stmt, ParseError> {
        self.parse_statement()
    }
}
```

#### 步骤 2.2：消除 ExprParser 和 StmtParser 中的重复 Lexer

原有问题：
```rust
// stmt_parser.rs (旧)
pub struct StmtParser {
    lexer: Lexer,              // 重复的 Lexer
    expr_parser: ExprParser,   // ExprParser 内部又有 Lexer
}
```

解决方案：
```rust
// parser/mod.rs (新)
pub struct ExprParser<'a> {
    lexer: &'a mut Lexer<'a>,
}

pub struct StmtParser<'a> {
    lexer: &'a mut Lexer<'a>,
    expr_parser: ExprParser<'a>,
}

impl<'a> StmtParser<'a> {
    pub fn new(lexer: &'a mut Lexer<'a>) -> Self {
        StmtParser {
            lexer,
            expr_parser: ExprParser::new(lexer),
        }
    }
}
```

### 阶段三：模块重组（Week 3）

#### 步骤 3.1：删除 expressions 目录

将 `expression_converter.rs` 的功能迁移到合适的位置：
- 转换函数 → `ast/utils.rs`
- 测试 → `ast/tests.rs` 或单独的测试模块

#### 步骤 3.2：拆分 core 模块

```
core/
├── mod.rs
 # ParseError, ParseErrors├── error.rs     , ParseErrorKind
├── token.rs      # Token, TokenKind
├── position.rs   # Position, Span
└── context.rs    # ParseContext, ParseConfig
```

### 阶段四：API 优化（Week 4）

#### 步骤 4.1：简化公共 API

```rust
// parser/mod.rs

/// 解析查询字符串，返回解析后的语句或错误
pub fn parse_query(input: &str) -> Result<Stmt, ParseError> {
    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    parser.parse()
}

/// 解析表达式字符串，返回解析后的表达式或错误
pub fn parse_expression(input: &str) -> Result<Expr, ParseError> {
    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    parser.parse_expression()
}
```

#### 步骤 4.2：添加高级配置

```rust
pub struct ParseOptions {
    pub max_query_length: usize,
    pub enable_recovery: bool,
    pub record_warnings: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        ParseOptions {
            max_query_length: 1024 * 1024, // 1MB
            enable_recovery: false,
            record_warnings: true,
        }
    }
}

pub fn parse_query_with_options(
    input: &str,
    options: ParseOptions,
) -> Result<Stmt, ParseError> {
    let mut lexer = Lexer::new(input);
    let context = ParseContext::with_config(ParseConfig::from(options));
    let mut parser = Parser::with_context(&mut lexer, context);
    parser.parse()
}
```

## 五、测试计划

### 5.1 单元测试

- Lexer 单元测试：词法分析正确性
- Parser 单元测试：语法分析正确性
- 错误处理测试：各种错误场景

### 5.2 集成测试

- 与 Validator 模块的集成
- 与 Planner 模块的集成
- 与 Executor 模块的集成

### 5.3 兼容性测试

- NebulaGraph 兼容语法测试
- OpenCypher 兼容语法测试

## 六、风险评估

### 6.1 高风险项

1. **Lex 生命周期管理**：需要仔细处理借用关系
2. **错误处理兼容性**：现有代码依赖旧错误类型的需要更新

### 6.2 缓解措施

1. 分阶段重构，每个阶段都有可运行的代码
2. 保留旧 API 的同时添加新 API
3. 充分的测试覆盖

## 七、预期收益

1. **性能提升**：消除重复的 Lexer 实例，减少内存占用
2. **可维护性提升**：模块边界清晰，职责分明
3. **错误处理改进**：统一的错误类型和报告机制
4. **API 优化**：更友好的公共接口

## 八、参考实现

本设计方案参考了以下 NebulaGraph 源码：

1. `src/parser/GQLParser.h` - 解析器主入口设计
2. `src/parser/GraphScanner.h` - 词法分析器设计
3. `src/common/base/ErrorOr.h` - 错误处理模式
4. `src/common/expression/Expression.h` - AST 设计模式
