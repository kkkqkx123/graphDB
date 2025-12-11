# GraphDB 查询解析器增强计划

本文档详细描述了如何增强 GraphDB 的查询解析器，使其支持与 NebulaGraph 相当的功能。

## 1. 当前状态分析

目前 GraphDB 的查询解析器支持：
- 基础的 CRUD 操作
- 简单的表达式解析
- 基础的 MATCH 模式匹配
- 基础的 WHERE、RETURN、LIMIT 子句

相比 NebulaGraph，缺少的主要功能：
- 复杂的路径模式匹配
- 高级表达式（列表推导、聚合、预测函数）
- 多跳查询和图算法语句
- 完整的词法和类型系统

## 1.1 增强后的词法分析器功能

在第一阶段增强后，GraphDB 的词法分析器现在支持：

- 特殊属性标识符：_id, _type, _src, _dst, _rank
- 图引用标识符：$$ (目标引用), $^ (源引用), $- (输入引用)
- 聚合函数关键字：COUNT, SUM, AVG, MIN, MAX
- 新增关键词：SOURCE, DESTINATION, RANK, INPUT
- 高级操作符和类型：地理空间类型 POINT, LINESTRING, POLYGON
- 时间和持续时间类型：TIMESTAMP, DURATION, DATE, TIME, DATETIME
- 高级操作符：=~ (正则匹配), IS NULL, IS NOT NULL 等

## 2. 实施计划

### 第一阶段：增强词法分析器 (Lexer)

**目标**：扩展当前的词法分析器，支持更多关键字和操作符

需要添加的支持：
- 特殊属性标识符：`_id`, `_type`, `_src`, `_dst`, `_rank`
- 图引用标识符：`$$`, `$^`, `$-`
- 地理空间类型：`POINT`, `LINESTRING`, `POLYGON`
- 高级操作符：`=~`, `IS NULL`, `CONTAINS`, `STARTS WITH`, `ENDS WITH`

**文件**：`src/query/parser/lexer/lexer.rs`
- 扩展现有的 TokenKind 枚举
- 添加新的词法规则
- 增强 read_string 方法以支持转义字符

### 第二阶段：扩展 AST 结构

**目标**：添加缺失的 AST 节点类型

需要添加的 AST 节点：
- 类型转换表达式
- 聚合表达式
- 列表推导表达式
- 预测表达式 (ALL/ANY/SINGLE/NONE)
- 归约表达式
- 范围子表达式
- 边属性表达式
- 复杂的 MATCH 模式节点

**文件**：
- `src/query/parser/ast/expression.rs` - 添加新的表达式类型
- `src/query/parser/ast/statement.rs` - 添加新的语句类型
- `src/query/parser/ast/pattern.rs` - 扩展图模式匹配

### 第三阶段：实现高级表达式解析

**目标**：实现新的表达式解析器

需要实现的解析器：
- 聚合函数解析器
- 列表推导解析器
- 预测函数解析器
- 类型转换表达式解析器
- 复杂子表达式解析器

**文件**：
- `src/query/parser/expressions/mod.rs` - 添加新解析特性
- `src/query/parser/parser/expression_parser.rs` - 扩展表达式解析逻辑

### 第四阶段：实现新的语句解析

**目标**：实现新的语句类型

需要实现的语句：
- GO/LOOKUP 语句
- FIND PATH 语句（最短路径等）
- SUBGRAPH 语句
- SET/YIELD 相关语句
- 复杂的 MATCH 模式

**文件**：
- `src/query/parser/parser/statement_parser.rs` - 扩展语句解析逻辑

### 第五阶段：增强解析器核心

**目标**：增强解析器以支持新增功能

**文件**：
- `src/query/parser/parser/parser.rs` - 添加新的解析方法
- `src/query/parser/core/error.rs` - 扩展错误处理

## 3. 新的目录结构设计

```
src/query/parser/
├── core/
│   ├── token.rs          # 扩展的 Token 结构
│   ├── error.rs          # 扩展的错误处理
│   └── types.rs          # 新增类型定义
├── lexer/
│   ├── lexer.rs          # 增强的词法分析器
│   └── mod.rs
├── ast/
│   ├── expression.rs     # 扩展的表达式 AST
│   ├── statement.rs      # 扩展的语句 AST
│   ├── pattern.rs        # 扩展的图模式 AST
│   ├── types.rs          # AST 类型定义
│   └── mod.rs
├── expressions/
│   ├── expression_parser.rs  # 基础表达式解析
│   ├── aggregation.rs        # 聚合表达式解析
│   ├── predicate.rs          # 预测函数解析
│   ├── list_comprehension.rs # 列表推导解析
│   └── mod.rs
├── statements/
│   ├── go.rs               # GO 语句解析
│   ├── match.rs            # MATCH 语句增强
│   ├── path.rs             # 路径查询语句
│   ├── lookup.rs           # LOOKUP 语句
│   └── mod.rs
├── parser/
│   ├── expression_parser.rs  # 表达式解析器
│   ├── statement_parser.rs   # 语句解析器
│   ├── pattern_parser.rs     # 图模式解析器
│   ├── utils.rs              # 解析器工具
│   └── mod.rs
├── query_parser.rs           # 查询解析器入口
├── mod.rs
└── tests.rs                  # 解析器测试
```

## 4. 实现细节

### 4.1 Token 扩展

在 `core/token.rs` 中添加：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ... 现有枚举 ...
    
    // 特殊属性
    IdProp,      // _id
    TypeProp,    // _type
    SrcIdProp,   // _src
    DstIdProp,   // _dst
    RankProp,    // _rank
    InputRef,    // $-
    SrcRef,      // $^
    DstRef,      // $$
    
    // 扩展操作符
    NotIn,
    Contains,
    StartsWith,
    EndsWith,
    IsNull,
    IsNotNull,
    IsEmpty,
    IsNotEmpty,
    Regex,       // =~
    
    // 地理空间类型
    Geography,
    Point,
    Linestring,
    Polygon,
    
    // 日期时间类型
    Timestamp,
    Date,
    Time,
    Datetime,
    Duration,
    
    // 聚合函数
    Count,
    Sum,
    Avg,
    Min,
    Max,
    
    // 预测函数
    All,
    Any,
    Single,
    None,
    Reduce,
    Exists,
    
    // 路径相关
    Steps,
    Path,
    Shortest,
    AllShortestPaths,
    NoLoop,
    Subgraph,
    Bidirect,
    Both,
    Out,
    In,
}
```

### 4.2 AST 扩展

在 `ast/expression.rs` 中添加：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // ... 现有枚举 ...
    
    // 聚合表达式
    Aggregation(Box<AggregationExpression>),
    
    // 列表推导
    ListComprehension(Box<ListComprehensionExpression>),
    
    // 预测函数 (ALL/ANY/SINGLE/NONE)
    Predicate(Box<PredicateExpression>),
    
    // 归约表达式
    Reduce(Box<ReduceExpression>),
    
    // 类型转换
    TypeCast(Box<TypeCastExpression>),
    
    // 子查询
    SubQuery(Box<SubQueryExpression>),
    
    // 特殊属性访问
    SpecialProperty(SpecialPropertyType, Option<Box<Expression>>),
    
    // 边属性访问
    EdgeProperty(Box<Expression>, EdgePropertyType),
    
    // 范围子表达式
    SubscriptRange(Box<Expression>, Option<Box<Expression>>, Option<Box<Expression>>),
}
```

### 4.3 语句扩展

在 `ast/statement.rs` 中添加：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // ... 现有枚举 ...
    
    // GO 语句
    Go(GoStatement),
    
    // 查找路径
    FindPath(FindPathStatement),
    
    // LOOKUP 语句
    Lookup(LookupStatement),
    
    // 子图查询
    Subgraph(SubgraphStatement),
    
    // 集合操作
    Set(SetStatement),
    
    // 管道操作
    Piped(PipedStatement),
}
```

## 5. 测试计划

为每个阶段创建相应的测试：

- `lexer/lexer_test.rs` - 词法分析器测试
- `ast/ast_test.rs` - AST 结构测试
- `expressions/expression_test.rs` - 表达式解析测试
- `statements/statement_test.rs` - 语句解析测试
- `integration/integration_test.rs` - 集成测试

## 6. 时间安排

- 第一阶段：2周 - 增强词法分析器
- 第二阶段：3周 - 扩展 AST 结构
- 第三阶段：4周 - 实现高级表达式解析
- 第四阶段：3周 - 实现新的语句解析
- 第五阶段：2周 - 增强解析器核心
- 总计：约 14 周

## 7. 风险评估

- **复杂性风险**：图模式匹配和路径查询逻辑复杂
- **性能风险**：新增功能可能影响解析器性能
- **兼容性风险**：扩展语法可能影响现有功能

## 8. 总结

通过实施这个计划，GraphDB 的查询解析器将具备与 NebulaGraph 相当的功能，支持更复杂的图查询和分析操作。这将大大提升 GraphDB 的功能完整性和实用性。