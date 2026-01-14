# NGQL 与 Cypher 关系分析及目录划分建议

## 概述

本文档分析了 NGQL（NebulaGraph Query Language）与 Cypher 的关系，并提出了 GraphDB parser 模块的目录划分建议。

## NGQL 与 Cypher 的关系

### 1. 语言关系

NGQL 是 Cypher 的**超集（Superset）**，具有以下特点：

#### 1.1 共同特性（Cypher 核心）

NGQL 完全兼容 Cypher 的核心语法：

| 特性 | Cypher | NGQL | 说明 |
|------|--------|-------|------|
| 模式匹配 | ✅ | ✅ | MATCH 语句 |
| 查询结果 | ✅ | ✅ | RETURN 语句 |
| 数据创建 | ✅ | ✅ | CREATE 语句 |
| 数据修改 | ✅ | ✅ | DELETE, SET, REMOVE, MERGE 语句 |
| 数据管道 | ✅ | ✅ | WITH 语句 |
| 数据展开 | ✅ | ✅ | UNWIND 语句 |
| 过滤条件 | ✅ | ✅ | WHERE 子句 |
| 排序和分页 | ✅ | ✅ | ORDER BY, SKIP, LIMIT |
| 函数调用 | ✅ | ✅ | CALL 语句 |
| 聚合函数 | ✅ | ✅ | COUNT, SUM, AVG 等 |

#### 1.2 NGQL 扩展特性

NGQL 在 Cypher 基础上添加了图数据库特有的扩展：

| 特性 | NGQL | 说明 |
|------|-------|------|
| GO 遍历 | ✅ | 基于步数的图遍历 |
| LOOKUP 索引查询 | ✅ | 使用索引快速查找 |
| FETCH 数据获取 | ✅ | 获取顶点/边数据 |
| FIND PATH 路径查找 | ✅ | 查找最短路径 |
| YIELD 结果输出 | ✅ | 灵活的结果输出 |
| 管道操作符 | ✅ | \| 操作符 |
| 集合操作 | ✅ | UNION, INTERSECT, MINUS |
| 管理语句 | ✅ | CREATE SPACE, CREATE TAG 等 |
| EXPLAIN/PROFILE | ✅ | 查询执行计划 |

### 2. 语句分类对比

#### 2.1 NebulaGraph 的语句分类

在 NebulaGraph 中，语句按照功能分为以下类别：

```
nebula-3.8.0/src/parser/
├── Sentence.h                    # 语句基类和 Kind 枚举
├── MatchSentence.cpp/h           # MATCH 语句（Cypher）
├── TraverseSentences.cpp/h       # 遍历语句（NGQL）
│   ├── GoSentence               # GO 语句
│   ├── UnwindSentence           # UNWIND 语句
│   ├── LookupSentence           # LOOKUP 语句
│   └── FindPathSentence         # FIND PATH 语句
├── MutateSentences.cpp/h        # 变异语句（Cypher + NGQL）
│   ├── InsertVerticesSentence   # INSERT 语句
│   ├── UpdateVertexSentence    # UPDATE 语句
│   ├── InsertEdgesSentence     # INSERT 语句
│   └── UpdateEdgeSentence      # UPDATE 语句
├── AdminSentences.cpp/h        # 管理语句（NGQL）
│   ├── CreateSpaceSentence      # CREATE SPACE
│   ├── DropSpaceSentence       # DROP SPACE
│   ├── CreateTagSentence       # CREATE TAG
│   ├── DropTagSentence        # DROP TAG
│   ├── CreateEdgeSentence      # CREATE EDGE
│   └── DropEdgeSentence       # DROP EDGE
├── UserSentences.cpp/h         # 用户管理语句（NGQL）
│   ├── CreateUserSentence       # CREATE USER
│   ├── DropUserSentence        # DROP USER
│   ├── AlterUserSentence       # ALTER USER
│   ├── GrantSentence           # GRANT
│   └── RevokeSentence          # REVOKE
├── ProcessControlSentences.cpp/h # 进程控制语句（NGQL）
│   ├── ShowSessionsSentence     # SHOW SESSIONS
│   ├── KillSessionSentence     # KILL SESSION
│   ├── ShowQueriesSentence     # SHOW QUERIES
│   └── KillQuerySentence       # KILL QUERY
├── Clauses.cpp/h              # 通用子句
│   ├── StepClause              # 步骤子句
│   ├── FromClause              # FROM 子句
│   ├── ToClause                # TO 子句
│   ├── OverClause              # OVER 子句
│   ├── WhereClause             # WHERE 子句
│   ├── YieldClause             # YIELD 子句
│   └── OrderFactors            # ORDER BY 子句
└── ExplainSentence.cpp/h        # EXPLAIN 语句
```

#### 2.2 当前 GraphDB 的语句分类

当前 GraphDB 的 parser 模块结构：

```
src/query/parser/
├── mod.rs                       # 模块入口
├── core/                        # 核心类型
│   ├── error.rs
│   ├── mod.rs
│   └── token.rs
├── ast/                         # 简化版 AST
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
├── cypher/                      # Cypher 解析器
│   ├── mod.rs
│   ├── lexer.rs
│   ├── parser.rs
│   ├── parser_core.rs
│   ├── expression_parser.rs
│   ├── pattern_parser.rs
│   ├── statement_parser.rs
│   ├── clause_parser.rs
│   ├── expression_converter.rs
│   ├── expression_evaluator.rs
│   ├── expression_optimizer.rs
│   ├── cypher_processor.rs
│   └── ast/                    # Cypher AST
│       ├── mod.rs
│       ├── clauses.rs
│       ├── expressions.rs
│       ├── patterns.rs
│       ├── query_types.rs
│       ├── statements.rs
│       └── converters.rs
├── expressions/                 # 表达式转换
│   ├── mod.rs
│   └── expression_converter.rs
├── lexer/                       # 词法分析器
│   ├── mod.rs
│   └── lexer.rs
├── parser/                      # 另一个解析器实现
│   ├── mod.rs
│   ├── expr_parser.rs
│   ├── pattern_parser.rs
│   ├── statement_parser.rs
│   └── utils.rs
└── statements/                  # 语句实现
    ├── mod.rs
    ├── create.rs
    ├── delete.rs
    ├── go.rs
    ├── match_stmt.rs
    └── update.rs
```

### 3. 问题分析

#### 3.1 当前结构的问题

1. **模块重复**：
   - `ast/` 和 `cypher/ast/` 存在重复
   - `parser/` 和 `cypher/` 存在功能重叠
   - `lexer/` 和 `cypher/lexer.rs` 存在重复

2. **职责不清**：
   - `cypher/` 模块已经包含了 NGQL 语句类型（GO, LOOKUP, FETCH 等）
   - 但模块名称仍然是 `cypher`，容易造成混淆

3. **层次混乱**：
   - `statements/` 目录下的文件（go.rs, create.rs 等）与 `cypher/` 模块的关系不明确
   - AST 定义分散在多个位置

4. **扩展性差**：
   - 添加新的 NGQL 语句时，需要在多个地方修改
   - 缺少统一的语句基类

## 目录划分建议

### 方案一：按查询语言分类（推荐）

将 parser 模块按照查询语言类型重新组织，清晰区分 Cypher 和 NGQL：

```
src/query/parser/
├── mod.rs                       # 模块入口
├── core/                        # 核心类型（共享）
│   ├── error.rs
│   ├── token.rs
│   └── mod.rs
├── lexer/                       # 统一的词法分析器
│   ├── mod.rs
│   └── lexer.rs                # 支持 Cypher + NGQL 关键字
├── ast/                         # 统一的 AST 定义
│   ├── mod.rs
│   ├── base.rs                  # 语句基类和枚举
│   ├── expressions.rs            # 表达式定义
│   ├── patterns.rs              # 模式定义
│   ├── clauses.rs               # 子句定义
│   └── visitors.rs              # 访问者模式
├── cypher/                      # Cypher 专用
│   ├── mod.rs
│   ├── parser.rs                # Cypher 语句解析器
│   ├── statements/
│   │   ├── mod.rs
│   │   ├── match.rs            # MATCH 语句
│   │   ├── return.rs           # RETURN 语句
│   │   ├── create.rs           # CREATE 语句
│   │   ├── delete.rs           # DELETE 语句
│   │   ├── set.rs              # SET 语句
│   │   ├── remove.rs           # REMOVE 语句
│   │   ├── merge.rs            # MERGE 语句
│   │   ├── with.rs             # WITH 语句
│   │   └── unwind.rs           # UNWIND 语句
│   └── clauses/
│       ├── mod.rs
│       ├── match_clause.rs
│       ├── where_clause.rs
│       ├── return_clause.rs
│       └── with_clause.rs
├── ngql/                        # NGQL 专用
│   ├── mod.rs
│   ├── parser.rs                # NGQL 语句解析器
│   ├── statements/
│   │   ├── mod.rs
│   │   ├── go.rs               # GO 语句
│   │   ├── lookup.rs           # LOOKUP 语句
│   │   ├── fetch.rs            # FETCH 语句
│   │   ├── find_path.rs        # FIND PATH 语句
│   │   ├── yield.rs            # YIELD 语句
│   │   ├── pipe.rs             # 管道操作
│   │   └── set_ops.rs          # 集合操作
│   ├── clauses/
│   │   ├── mod.rs
│   │   ├── step_clause.rs
│   │   ├── from_clause.rs
│   │   ├── over_clause.rs
│   │   ├── to_clause.rs
│   │   └── yield_clause.rs
│   └── admin/                   # 管理语句
│       ├── mod.rs
│       ├── space.rs            # SPACE 管理
│       ├── tag.rs             # TAG 管理
│       ├── edge.rs            # EDGE 管理
│       └── index.rs           # INDEX 管理
├── expression/                 # 表达式处理
│   ├── mod.rs
│   ├── parser.rs               # 表达式解析器
│   ├── evaluator.rs            # 表达式求值器
│   ├── optimizer.rs            # 表达式优化器
│   └── converter.rs           # 表达式转换器
└── utils/                      # 工具函数
    ├── mod.rs
    └── helpers.rs
```

**优点**：
- 清晰区分 Cypher 和 NGQL
- 易于扩展新的 NGQL 特性
- 避免模块重复

**缺点**：
- 需要大量的文件重构

### 方案二：按功能分类（平衡）

按照语句的功能特性进行分类，而不是语言类型：

```
src/query/parser/
├── mod.rs                       # 模块入口
├── core/                        # 核心类型
│   ├── error.rs
│   ├── token.rs
│   └── mod.rs
├── lexer/                       # 词法分析器
│   ├── mod.rs
│   └── lexer.rs
├── ast/                         # AST 定义
│   ├── mod.rs
│   ├── base.rs                  # 语句基类
│   ├── expressions.rs
│   ├── patterns.rs
│   └── clauses.rs
├── traversal/                  # 遍历相关（NGQL）
│   ├── mod.rs
│   ├── go.rs
│   ├── find_path.rs
│   └── clauses/
│       ├── step_clause.rs
│       ├── from_clause.rs
│       ├── over_clause.rs
│       └── to_clause.rs
├── query/                      # 查询相关（Cypher + NGQL）
│   ├── mod.rs
│   ├── match.rs
│   ├── lookup.rs
│   ├── fetch.rs
│   └── clauses/
│       ├── where_clause.rs
│       └── yield_clause.rs
├── mutation/                   # 数据修改（Cypher + NGQL）
│   ├── mod.rs
│   ├── create.rs
│   ├── delete.rs
│   ├── update.rs
│   ├── insert.rs
│   └── merge.rs
├── projection/                 # 结果输出（Cypher + NGQL）
│   ├── mod.rs
│   ├── return.rs
│   ├── with.rs
│   ├── yield.rs
│   └── clauses/
│       ├── return_clause.rs
│       └── with_clause.rs
├── control_flow/               # 控制流（Cypher + NGQL）
│   ├── mod.rs
│   ├── unwind.rs
│   ├── pipe.rs
│   └── set_ops.rs
├── admin/                      # 管理语句（NGQL）
│   ├── mod.rs
│   ├── space.rs
│   ├── tag.rs
│   ├── edge.rs
│   ├── index.rs
│   └── user.rs
├── expression/                 # 表达式处理
│   ├── mod.rs
│   ├── parser.rs
│   ├── evaluator.rs
│   ├── optimizer.rs
│   └── converter.rs
└── utils/                      # 工具函数
    └── mod.rs
```

**优点**：
- 按功能组织，更符合实际使用场景
- 减少重复代码
- 易于理解每个模块的职责

**缺点**：
- Cypher 和 NGQL 的界限不够清晰
- 可能需要跨模块引用

### 方案三：渐进式重构（实用）

在现有结构基础上进行渐进式改进，最小化重构成本：

```
src/query/parser/
├── mod.rs                       # 模块入口
├── core/                        # 核心类型（保持不变）
├── lexer/                       # 词法分析器（保持不变）
├── ast/                         # 统一的 AST
│   ├── mod.rs
│   ├── base.rs                  # 语句基类
│   ├── expressions.rs
│   ├── patterns.rs
│   └── clauses.rs
├── statements/                  # 语句实现（扩展）
│   ├── mod.rs
│   ├── cypher/                  # Cypher 语句
│   │   ├── mod.rs
│   │   ├── match.rs
│   │   ├── return.rs
│   │   ├── create.rs
│   │   ├── delete.rs
│   │   ├── set.rs
│   │   ├── remove.rs
│   │   ├── merge.rs
│   │   ├── with.rs
│   │   └── unwind.rs
│   ├── ngql/                    # NGQL 语句（新增）
│   │   ├── mod.rs
│   │   ├── go.rs
│   │   ├── lookup.rs
│   │   ├── fetch.rs
│   │   ├── find_path.rs
│   │   └── yield.rs
│   ├── admin/                   # 管理语句（新增）
│   │   ├── mod.rs
│   │   ├── space.rs
│   │   ├── tag.rs
│   │   ├── edge.rs
│   │   └── index.rs
│   └── common/                  # 共享语句
│       ├── mod.rs
│       ├── pipe.rs
│       └── set_ops.rs
├── expression/                 # 表达式处理
│   ├── mod.rs
│   ├── parser.rs
│   ├── evaluator.rs
│   ├── optimizer.rs
│   └── converter.rs
└── utils/                      # 工具函数
    └── mod.rs
```

**优点**：
- 最小化重构成本
- 保持现有代码结构
- 渐进式改进

**缺点**：
- 仍然存在一些模块重复
- 需要仔细规划迁移路径

## 推荐方案

基于以上分析，我推荐**方案一（按查询语言分类）**，原因如下：

1. **清晰的职责划分**：
   - Cypher 和 NGQL 是两个不同的查询语言，应该分开处理
   - 便于独立测试和维护

2. **易于扩展**：
   - 添加新的 Cypher 特性只需修改 `cypher/` 模块
   - 添加新的 NGQL 特性只需修改 `ngql/` 模块

3. **符合 NebulaGraph 的设计**：
   - NebulaGraph 也是按照语句类型组织文件
   - 便于参考和对比

4. **长期可维护性**：
   - 避免模块间的耦合
   - 便于未来可能的查询语言扩展

## 实施步骤

### 阶段一：准备阶段（1周）

1. 创建新的目录结构
2. 迁移共享代码到 `core/`
3. 统一 AST 定义到 `ast/`

### 阶段二：Cypher 模块迁移（2周）

1. 迁移 Cypher 语句解析器
2. 迁移 Cypher 子句解析器
3. 更新测试用例

### 阶段三：NGQL 模块实现（3-4周）

1. 实现 GO 语句解析器
2. 实现 LOOKUP 语句解析器
3. 实现 FETCH 语句解析器
4. 实现 FIND PATH 语句解析器
5. 实现管理语句解析器

### 阶段四：清理和优化（1-2周）

1. 删除重复的模块
2. 更新文档
3. 性能优化
4. 完整测试

## 总结

NGQL 是 Cypher 的超集，在 Cypher 基础上添加了图数据库特有的扩展。当前 GraphDB 的 parser 模块存在模块重复、职责不清、层次混乱等问题。

推荐采用**按查询语言分类**的目录划分方案，将 parser 模块重新组织为：
- `cypher/` - Cypher 专用
- `ngql/` - NGQL 专用
- `ast/` - 统一的 AST
- `expression/` - 表达式处理
- `core/` - 核心类型

这种划分方式清晰、易于扩展、符合 NebulaGraph 的设计，适合长期维护和发展。
