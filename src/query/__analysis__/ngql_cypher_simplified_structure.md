# NGQL 与 Cypher 关系重新评估及简化目录结构

## 概述

本文档重新评估了 NGQL（NebulaGraph Query Language）与 Cypher 的关系，并基于 NebulaGraph 的实际实现提出了更简化的目录结构方案。

## 关键发现

### 1. NebulaGraph 的实际实现方式

通过深入分析 NebulaGraph 的源代码，发现一个重要事实：**NebulaGraph 的 parser 并没有按照 Cypher/NGQL 来组织代码**。

#### 1.1 语句类型枚举（Sentence.h）

NebulaGraph 将所有语句类型放在同一个枚举中，没有区分 Cypher 和 NGQL：

```cpp
enum class Kind : uint32_t {
    kUnknown,
    kExplain,
    kSequential,
    kGo,                // NGQL 特有
    kSet,
    kPipe,
    kUse,
    kMatch,             // Cypher 特有
    kAssignment,
    kCreateTag,         // NGQL 特有
    kAlterTag,
    kCreateEdge,        // NGQL 特有
    kAlterEdge,
    kDescribeTag,
    kDescribeEdge,
    kCreateTagIndex,
    kCreateEdgeIndex,
    kDropTagIndex,
    kDropEdgeIndex,
    kDescribeTagIndex,
    kDescribeEdgeIndex,
    kDropTag,
    kDropEdge,
    kInsertVertices,    // NGQL 特有
    kUpdateVertex,
    kInsertEdges,       // NGQL 特有
    kUpdateEdge,
    kLookup,            // NGQL 特有
    kCreateSpace,       // NGQL 特有
    kDropSpace,
    kYield,             // NGQL 特有
    kFetchVertices,     // NGQL 特有
    kFetchEdges,        // NGQL 特有
    kFindPath,          // NGQL 特有
    kReturn,            // Cypher 特有
    kUnwind,            // Cypher 特有
    // ... 更多语句类型
};
```

#### 1.2 文件组织方式

NebulaGraph 按照**语句功能**来组织文件，而不是语言类型：

```
nebula-3.8.0/src/parser/
├── Sentence.h                    # 语句基类和 Kind 枚举（统一）
├── MatchSentence.cpp/h           # MATCH 语句（Cypher）
├── TraverseSentences.cpp/h       # 遍历语句（GO, LOOKUP, FIND PATH）
├── MutateSentences.cpp/h        # 变异语句（INSERT, UPDATE, DELETE）
├── MaintainSentences.cpp/h      # 维护语句（CREATE/ALTER/DROP TAG/EDGE/SPACE）
├── AdminSentences.cpp/h         # 管理语句（用户、权限、进程控制）
├── UserSentences.cpp/h          # 用户管理语句
├── ProcessControlSentences.cpp/h # 进程控制语句
├── Clauses.cpp/h                # 通用子句（StepClause, FromClause, OverClause, WhereClause, YieldClause）
└── ExplainSentence.cpp/h        # EXPLAIN 语句
```

### 2. NGQL 与 Cypher 的实际关系

#### 2.1 语法层面的关系

NGQL 确实是 Cypher 的超集，包含 Cypher 的核心语法，但这并不意味着需要在代码层面区分它们。

**Cypher 核心语法**：
- MATCH 语句
- RETURN 语句
- CREATE 语句（简化版）
- DELETE 语句（简化版）
- SET 语句
- REMOVE 语句
- MERGE 语句
- WITH 语句
- UNWIND 语句
- WHERE 子句
- ORDER BY 子句
- SKIP/LIMIT 子句

**NGQL 扩展语法**：
- GO 语句（基于步数的图遍历）
- LOOKUP 语句（索引查询）
- FETCH 语句（获取顶点/边数据）
- FIND PATH 语句（路径查找）
- YIELD 语句（灵活的结果输出）
- 管道操作符（|）
- 管理语句（CREATE SPACE, CREATE TAG, CREATE EDGE 等）
- INSERT 语句（批量插入）
- UPDATE 语句（批量更新）

#### 2.2 语义层面的关系

从语义角度看，NGQL 和 Cypher 的语句可以按照功能分类：

| 功能类别 | Cypher | NGQL | 说明 |
|---------|--------|-------|------|
| **查询** | MATCH | GO, LOOKUP, FETCH | 查询图数据 |
| **遍历** | - | GO, FIND PATH | 图遍历 |
| **变异** | CREATE, DELETE, SET, REMOVE, MERGE | INSERT, UPDATE, DELETE | 修改图数据 |
| **投影** | RETURN, WITH | YIELD | 输出结果 |
| **控制流** | UNWIND | PIPE | 控制数据流 |
| **管理** | - | CREATE/ALTER/DROP SPACE/TAG/EDGE | 管理图结构 |
| **索引** | - | CREATE/ALTER/DROP TAG/EDGE INDEX | 管理索引 |

### 3. 为什么不需要区分 Cypher 和 NGQL

#### 3.1 技术原因

1. **统一的词法分析器**：
   - Cypher 和 NGQL 共享相同的关键字集合
   - 词法分析器不需要区分语言类型

2. **统一的 AST 结构**：
   - 所有语句都可以用统一的 AST 表示
   - 语句类型通过枚举区分，而不是通过语言类型

3. **统一的解析器**：
   - 解析器根据关键字识别语句类型
   - 不需要预先知道是 Cypher 还是 NGQL

4. **共享的子句**：
   - WHERE 子句在 Cypher 和 NGQL 中通用
   - ORDER BY、SKIP、LIMIT 等子句也通用

#### 3.2 架构原因

1. **简化设计**：
   - 不需要维护两套独立的解析器
   - 减少代码重复

2. **提高可维护性**：
   - 修改语句类型时，只需修改一个地方
   - 更容易添加新的语句类型

3. **符合实际使用场景**：
   - 用户不会关心是 Cypher 还是 NGQL
   - 用户只关心能否执行特定的查询

#### 3.3 NebulaGraph 的实践

NebulaGraph 的实践证明：
- 不区分 Cypher 和 NGQL 是可行的
- 按功能组织代码更合理
- 统一的解析器更易于维护

## 简化的目录结构方案

### 方案：按功能分类（推荐）

基于 NebulaGraph 的实践和上述分析，推荐采用按功能分类的目录结构：

```
src/query/parser/
├── mod.rs                       # 模块入口
├── core/                        # 核心类型（统一）
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
├── statements/                  # 语句实现（按功能分类）
│   ├── mod.rs
│   ├── query.rs                 # 查询语句（MATCH, GO, LOOKUP, FETCH）
│   ├── traverse.rs              # 遍历语句（GO, FIND PATH）
│   ├── mutate.rs                # 变异语句（CREATE, DELETE, SET, INSERT, UPDATE）
│   ├── projection.rs            # 投影语句（RETURN, WITH, YIELD）
│   ├── control_flow.rs          # 控制流语句（UNWIND, PIPE）
│   ├── maintain.rs              # 维护语句（CREATE/ALTER/DROP SPACE/TAG/EDGE）
│   ├── admin.rs                 # 管理语句（用户、权限、进程控制）
│   └── explain.rs               # EXPLAIN 语句
├── clauses/                     # 子句实现（通用）
│   ├── mod.rs
│   ├── where.rs                 # WHERE 子句
│   ├── order_by.rs              # ORDER BY 子句
│   ├── skip_limit.rs            # SKIP/LIMIT 子句
│   ├── match_clause.rs          # MATCH 子句
│   ├── step.rs                  # STEP 子句（NGQL）
│   ├── from.rs                  # FROM 子句（NGQL）
│   ├── over.rs                  # OVER 子句（NGQL）
│   └── yield.rs                 # YIELD 子句（NGQL）
├── expression/                 # 表达式处理
│   ├── mod.rs
│   ├── parser.rs                # 表达式解析器
│   ├── evaluator.rs             # 表达式求值器
│   ├── optimizer.rs             # 表达式优化器
│   └── converter.rs            # 表达式转换器
└── utils/                      # 工具函数
    ├── mod.rs
    └── helpers.rs
```

### 优点

1. **清晰的功能划分**：
   - 每个模块负责一类功能
   - 易于理解和维护

2. **避免重复**：
   - 统一的词法分析器
   - 统一的 AST 定义
   - 统一的解析器

3. **易于扩展**：
   - 添加新的语句类型只需在相应的功能模块中添加
   - 不需要修改多个地方

4. **符合 NebulaGraph 的实践**：
   - 参考成熟的设计
   - 降低学习成本

### 与之前方案的对比

| 特性 | 按语言分类 | 按功能分类（推荐） |
|------|-----------|-------------------|
| 模块数量 | 多 | 少 |
| 代码重复 | 多 | 少 |
| 可维护性 | 中 | 高 |
| 扩展性 | 中 | 高 |
| 符合 NebulaGraph 实践 | 否 | 是 |
| 学习曲线 | 陡 | 平缓 |

## 实施步骤

### 阶段一：准备阶段（1周）

1. 创建新的目录结构
2. 迁移共享代码到 `core/`
3. 统一 AST 定义到 `ast/`

### 阶段二：词法分析器和 AST（1周）

1. 统一词法分析器（合并 `lexer/` 和 `cypher/lexer.rs`）
2. 统一 AST 定义（合并 `ast/` 和 `cypher/ast/`）
3. 定义统一的语句类型枚举

### 阶段三：语句实现（2-3周）

1. 实现 `statements/query.rs`（MATCH, GO, LOOKUP, FETCH）
2. 实现 `statements/traverse.rs`（GO, FIND PATH）
3. 实现 `statements/mutate.rs`（CREATE, DELETE, SET, INSERT, UPDATE）
4. 实现 `statements/projection.rs`（RETURN, WITH, YIELD）
5. 实现 `statements/control_flow.rs`（UNWIND, PIPE）
6. 实现 `statements/maintain.rs`（CREATE/ALTER/DROP SPACE/TAG/EDGE）
7. 实现 `statements/admin.rs`（用户、权限、进程控制）
8. 实现 `statements/explain.rs`（EXPLAIN）

### 阶段四：子句实现（1周）

1. 实现通用子句（WHERE, ORDER BY, SKIP/LIMIT）
2. 实现 NGQL 特有子句（STEP, FROM, OVER, YIELD）
3. 实现 Cypher 特有子句（MATCH）

### 阶段五：清理和优化（1周）

1. 删除重复的模块
2. 更新文档
3. 性能优化
4. 完整测试

## 总结

通过深入分析 NebulaGraph 的实际实现，发现：

1. **不需要区分 Cypher 和 NGQL**：
   - NebulaGraph 的 parser 并没有按语言类型组织代码
   - 统一的词法分析器、AST 和解析器更合理

2. **推荐按功能分类**：
   - 查询语句（MATCH, GO, LOOKUP, FETCH）
   - 遍历语句（GO, FIND PATH）
   - 变异语句（CREATE, DELETE, SET, INSERT, UPDATE）
   - 投影语句（RETURN, WITH, YIELD）
   - 控制流语句（UNWIND, PIPE）
   - 维护语句（CREATE/ALTER/DROP SPACE/TAG/EDGE）
   - 管理语句（用户、权限、进程控制）

3. **简化的目录结构**：
   - 减少模块数量
   - 避免代码重复
   - 提高可维护性和扩展性
   - 符合 NebulaGraph 的实践

这种设计更符合实际需求，更易于维护和扩展，是 GraphDB parser 模块的最佳选择。
