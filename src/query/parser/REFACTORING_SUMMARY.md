# Parser 模块重构总结

## 概述

本文档总结了 parser 模块的重构工作，按照功能分类重新组织了代码结构。

## 重构内容

### 1. 目录结构

新的目录结构按功能分类：

```
src/query/parser/
├── core/                        # 核心类型（统一）
│   ├── error.rs
│   ├── token.rs
│   └── mod.rs
├── lexer/                       # 统一的词法分析器
│   ├── mod.rs
│   └── lexer.rs
├── ast/                         # 统一的 AST 定义
│   ├── mod.rs
│   ├── types.rs
│   ├── expr.rs
│   ├── stmt.rs
│   ├── pattern.rs
│   ├── expr_parser.rs
│   ├── stmt_parser.rs
│   ├── pattern_parser.rs
│   ├── utils.rs
│   └── visitor.rs
├── statements/                  # 语句实现（按功能分类）
│   ├── mod.rs
│   ├── create.rs                # CREATE 语句
│   ├── delete.rs                # DELETE 语句
│   ├── go.rs                    # GO 语句
│   ├── match_stmt.rs            # MATCH 语句
│   ├── update.rs                # UPDATE 语句
│   ├── query.rs                # 查询语句（MATCH, GO, LOOKUP, FETCH, FIND PATH）
│   ├── traverse.rs             # 遍历语句（GO, FIND PATH）
│   ├── mutate.rs               # 变异语句（CREATE, DELETE, SET, INSERT, UPDATE, MERGE, REMOVE）
│   ├── projection.rs           # 投影语句（RETURN, WITH, YIELD）
│   ├── control_flow.rs         # 控制流语句（UNWIND, PIPE）
│   ├── maintain.rs             # 维护语句（CREATE/ALTER/DROP SPACE/TAG/EDGE/INDEX）
│   └── admin.rs                # 管理语句（用户、权限、进程控制）
├── clauses/                     # 子句实现（通用）
│   ├── mod.rs
│   ├── where_clause.rs          # WHERE 子句
│   ├── order_by.rs             # ORDER BY 子句
│   ├── skip_limit.rs           # SKIP/LIMIT 子句
│   ├── match_clause.rs         # MATCH 子句
│   ├── step.rs                # STEP 子句（NGQL）
│   ├── from_clause.rs         # FROM 子句（NGQL）
│   ├── over_clause.rs         # OVER 子句（NGQL）
│   ├── yield_clause.rs        # YIELD 子句（NGQL）
│   ├── return_clause.rs       # RETURN 子句
│   ├── with_clause.rs         # WITH 子句
│   └── set_clause.rs         # SET 子句
└── expressions/                 # 表达式处理
    ├── mod.rs
    └── expression_converter.rs
```

### 2. 语句分类

#### 查询语句（query.rs）
- MATCH 语句
- GO 语句
- LOOKUP 语句
- FETCH 语句
- FIND PATH 语句

#### 遍历语句（traverse.rs）
- GO 语句
- FIND PATH 语句

#### 变异语句（mutate.rs）
- CREATE 语句
- DELETE 语句
- SET 语句
- INSERT 语句
- UPDATE 语句
- MERGE 语句
- REMOVE 语句

#### 投影语句（projection.rs）
- RETURN 语句
- WITH 语句
- YIELD 语句

#### 控制流语句（control_flow.rs）
- UNWIND 语句
- PIPE 语句

#### 维护语句（maintain.rs）
- CREATE/ALTER/DROP SPACE 语句
- CREATE/ALTER/DROP TAG 语句
- CREATE/ALTER/DROP EDGE 语句
- CREATE/ALTER/DROP INDEX 语句

#### 管理语句（admin.rs）
- CREATE/ALTER/DROP USER 语句
- GRANT/REVOKE 语句
- SHOW 语句
- USE 语句
- EXPLAIN 语句
- PROFILE 语句

### 3. 子句分类

#### 通用子句
- WHERE 子句
- ORDER BY 子句
- SKIP/LIMIT 子句

#### Cypher 特有子句
- MATCH 子句
- RETURN 子句
- WITH 子句

#### NGQL 特有子句
- STEP 子句
- FROM 子句
- OVER 子句
- YIELD 子句

#### 共享子句
- SET 子句

### 4. 主要改进

1. **统一词法分析器**：
   - 不区分 Cypher 和 NGQL
   - 支持所有关键字

2. **统一 AST 定义**：
   - 所有语句类型使用统一的枚举
   - 避免重复定义

3. **按功能分类**：
   - 清晰的功能划分
   - 易于理解和维护

4. **模块化设计**：
   - 每个功能模块独立
   - 减少耦合

### 5. 待完成工作

以下模块已创建框架，但需要实现具体的解析逻辑：

#### 语句模块
- query.rs - 需要实现具体的解析器
- traverse.rs - 需要实现具体的解析器
- mutate.rs - 需要实现具体的解析器
- projection.rs - 需要实现具体的解析器
- control_flow.rs - 需要实现具体的解析器
- maintain.rs - 需要实现具体的解析器
- admin.rs - 需要实现具体的解析器

#### 子句模块
- where_clause.rs - 需要实现具体的解析器
- order_by.rs - 需要实现具体的解析器
- skip_limit.rs - 需要实现具体的解析器
- match_clause.rs - 需要实现具体的解析器
- step.rs - 需要实现具体的解析器
- from_clause.rs - 需要实现具体的解析器
- over_clause.rs - 需要实现具体的解析器
- yield_clause.rs - 需要实现具体的解析器
- return_clause.rs - 需要实现具体的解析器
- with_clause.rs - 需要实现具体的解析器
- set_clause.rs - 需要实现具体的解析器

### 6. 下一步

1. 实现具体的解析器逻辑
2. 添加测试用例
3. 更新文档
4. 性能优化

## 总结

通过这次重构，parser 模块的结构更加清晰，按照功能分类组织代码，避免了 Cypher 和 NGQL 的混淆，提高了可维护性和扩展性。
