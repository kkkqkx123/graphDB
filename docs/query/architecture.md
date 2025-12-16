

现在我明白了这些模块之间的关系。让我为您详细解释 query 执行器与 context、planner/ngql 目录的关系：

## Query 执行器与 Context、Planner/NGQL 的关系架构

### 1. 整体架构关系图

```
查询处理流程:
Parser → Planner → Executor → Storage
  ↓        ↓         ↓         ↓
AST → ExecutionPlan → Executor → StorageEngine
  ↓        ↓         ↓         ↓
Context ← Context ← Context ← Context
```

### 2. 各模块职责分析

#### A. **Context 模块** (`src/query/context/`)
- **职责**: 提供查询执行过程中的上下文管理
- **核心组件**:
  - `ast/`: AST相关上下文
  - `execution/`: 执行上下文
  - `managers/`: 各种管理器（变量、结果等）
  - `validate/`: 验证上下文

#### B. **Planner 模块** (`src/query/planner/`)
- **职责**: 将AST转换为执行计划
- **核心组件**:
  - `plan/`: 执行计划节点定义
  - `ngql/`: NGQL特定规划器（GO、LOOKUP等）
  - `match_planning/`: MATCH语句规划

#### C. **Executor 模块** (`src/query/executor/`)
- **职责**: 执行计划并产生结果
- **核心组件**:
  - `base/`: 基础执行器
  - `cypher/`: Cypher特定执行器
  - `traits/`: 执行器特征定义

### 3. 数据流向和交互关系

#### A. **查询处理流程**:
1. **解析阶段**: `Parser` → `AST`
2. **规划阶段**: `Planner` + `Context` → `ExecutionPlan`
3. **执行阶段**: `Executor` + `Context` → `Results`
4. **存储阶段**: `Executor` → `StorageEngine`

#### B. **Context 在各阶段的作用**:
```rust
// 1. 解析阶段
let mut ast_context = CypherAstContext::new(query_text);
// 解析器填充AST上下文信息

// 2. 规划阶段
let mut exec_context = ExecutionContext::new();
// 规划器使用上下文创建执行计划

// 3. 执行阶段
let mut executor = CypherExecutor::new(storage);
executor.set_context(exec_context);
// 执行器使用上下文执行查询
```

### 4. NGQL 与 Cypher 的关系

#### A. **NGQL 规划器** (`src/query/planner/ngql/`):
- 处理 Nebula Graph Query Language (NGQL)
- 包含: `GoPlanner`, `LookupPlanner`, `PathPlanner` 等
- 针对图数据库特有的查询优化

#### B. **Cypher 执行器** (`src/query/executor/cypher/`):
- 处理 Cypher 查询语言
- 可以复用 NGQL 规划器的执行计划
- 提供统一的执行接口

### 5. 具体交互示例

```rust
// 1. 创建上下文
let mut context = QueryExecutionContext::new();

// 2. 解析查询
let ast = parser.parse("MATCH (n:Person) RETURN n.name")?;

// 3. 规划阶段 - 可能使用NGQL规划器
let planner = CypherPlanner::new();
let execution_plan = planner.plan(ast, &context)?;

// 4. 执行阶段
let mut executor = CypherExecutor::new(storage);
executor.set_context(context);
let results = executor.execute_plan(execution_plan).await?;
```

### 6. 架构优势

1. **模块分离**: 每个模块职责单一，便于维护
2. **语言无关**: 执行器可以支持多种查询语言
3. **上下文共享**: 各阶段共享上下文信息
4. **可扩展性**: 易于添加新的查询语言支持

### 7. 当前实现建议

基于这个架构分析，我建议：

1. **保持现有结构**: 当前的模块划分是合理的
2. **增强集成**: 加强 Cypher 执行器与现有 Context 的集成
3. **复用规划器**: Cypher 可以复用 NGQL 的部分规划逻辑
4. **统一接口**: 提供统一的执行器接口，支持多种查询语言

这种架构设计符合 nebula-graph 的设计理念，同时保持了良好的模块化和可扩展性。
