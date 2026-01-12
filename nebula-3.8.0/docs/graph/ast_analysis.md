从你提供的Nebula Graph AST源码文件可以看出，这些文件展示了查询语句在**编译前端**的核心数据结构。让我解析这些AST结构如何体现nGQL的查询原理。

## 📁 AST文件架构概览

```
graph/context/ast/
├── AstContext.h          # AST上下文基类
├── CypherAstContext.h    # OpenCypher兼容语法AST结构
└── QueryAstContext.h     # nGQL原生语法AST结构
```

## 🔬 AST结构如何体现查询原理

### 1. **查询语句的统一处理入口（AstContext.h）**
所有查询都通过统一的`AstContext`结构进入系统：
```cpp
struct AstContext {
  QueryContext* qctx;      // 查询执行上下文
  Sentence* sentence;      // 解析后的语句
  SpaceInfo space;         // 图空间信息
};
```
**原理体现**：nGQL将不同查询类型（Cypher风格和原生GO/FETCH等）**统一抽象**，为后续的查询优化和执行提供一致的接口。

### 2. **Cypher兼容查询的原理体现（CypherAstContext.h）**
这部分实现了**OpenCypher模式匹配**的AST结构，体现了模式匹配查询的原理：

```cpp
// 模式匹配的核心结构
struct Path final {
  std::vector<NodeInfo> nodeInfos;    // 节点模式
  std::vector<EdgeInfo> edgeInfos;    // 边模式
  PathBuildExpression* pathBuild;     // 路径构建表达式
};

// 子句链式处理
struct QueryPart final {
  std::vector<std::unique_ptr<MatchClauseContext>> matchs;
  std::unique_ptr<CypherClauseContextBase> boundary;  // WITH/UNWIND/RETURN
};
```

**原理特点**：
- **声明式模式描述**：将`(a:Person)-[:KNOWS]->(b)`这样的模式分解为`NodeInfo`和`EdgeInfo`
- **子句管道处理**：查询被分解为多个`QueryPart`，每个部分包含MATCH子句和边界子句
- **别名系统管理**：通过`aliasesAvailable`和`aliasesGenerated`管理变量作用域

### 3. **原生nGQL查询的原理体现（QueryAstContext.h）**
这部分体现了nGQL**命令式遍历**的查询原理：

```cpp
// GO查询的核心结构
struct GoContext final : AstContext {
  Starts from;              // 起始点：表达式/变量/管道
  StepClause steps;         // 步数限制
  Over over;                // 边类型和方向
  Expression* filter;       // 过滤条件
  YieldColumns* yieldExpr;  // 返回列
  
  // 执行优化相关
  std::string vidsVar;      // 顶点ID变量
  bool joinInput;           // 是否连接输入
  bool joinDst;             // 是否连接目标属性
};
```

**原理特点**：
- **明确的遍历指令**：`from`（从哪开始）→ `over`（沿什么边）→ `steps`（走几步）
- **执行上下文分离**：查询逻辑（AST）与执行状态（QueryContext）分离
- **管道化结果处理**：通过`inputVarName`支持多个语句的管道操作

## 🏗️ 查询编译执行流程

基于这些AST结构，nGQL查询的执行流程如下：

### 阶段1：解析与AST构建
```cpp
// 1. 解析查询文本为Sentence
MatchSentence* match = parser.parse("MATCH (n:Person) RETURN n.name");

// 2. 构建对应AST上下文
auto* cypherCtx = qctx->objPool()->make<CypherContext>();
cypherCtx->qctx = qctx;
cypherCtx->sentence = match;
```

### 阶段2：语义分析与校验
- **Cypher查询**：检查模式语法、变量作用域、别名冲突
- **原生查询**：验证图空间、Schema存在性、属性有效性

### 阶段3：逻辑计划生成
AST结构被转换为**逻辑算子计划**：
- **Cypher**：`MatchClauseContext` → `Traverse`算子 + `Filter`算子
- **GO查询**：`GoContext` → `GetNeighbors`算子 + `Project`算子

### 阶段4：分布式执行计划生成
```cpp
// 关键：将逻辑计划转换为分布式可执行计划
PlanNode* goNode = planner->transform(goContext);
// 生成类似：GetNeighbors(vertexId) -> Project(properties) -> Dedup()
```

## 🔄 Cypher与nGQL的AST差异对比

| **对比维度** | **Cypher AST (CypherAstContext.h)** | **nGQL AST (QueryAstContext.h)** |
|-------------|-----------------------------------|--------------------------------|
| **设计理念** | 模式描述为中心，声明式 | 遍历指令为中心，命令式 |
| **核心结构** | `Path` + `QueryPart`链 | `Starts` + `Over` + `Yield` |
| **变量管理** | 复杂的作用域和别名系统 | 简单的输入/输出变量传递 |
| **扩展性** | 易于添加新的模式子句 | 易于添加新的遍历参数 |
| **执行友好** | 需要复杂优化转换为遍历 | 直接映射为存储层操作 |

## 💡 从AST看查询语言设计哲学

1. **Cypher风格**：**用户友好优先**
   - AST结构贴近用户写的查询语法
   - 需要复杂的编译器优化转换为底层操作
   - 适合复杂模式匹配查询

2. **原生nGQL**：**执行效率优先**
   - AST结构贴近存储层的操作原语
   - 减少编译优化开销，执行路径更短
   - 适合大规模分布式遍历

3. **统一架构**：**兼容性设计**
   - 两种风格共享`AstContext`基类
   - 都转换为统一的逻辑/物理计划
   - 共享优化器和执行引擎

## 🎯 总结：nGQL查询原理的核心

从这些AST源码可以看出，Nebula Graph的查询处理是**分层抽象**的：

```
用户查询文本
    ↓
Parser（生成Sentence）
    ↓
AST构建（CypherContext/GoContext等）← 你提供的源码文件
    ↓
逻辑计划（LogicalPlan）
    ↓
物理计划（PhysicalPlan）
    ↓
分布式执行（Storage/Graph层）
```

**关键洞察**：nGQL通过在AST层支持**两种不同风格的抽象**（声明式Cypher和命令式GO），实现了既**用户友好**又**执行高效**的查询系统。这种设计让开发者可以根据查询复杂度选择合适的语法，而底层执行引擎能统一高效处理。

这种架构解释了为什么nGQL能在兼容OpenCypher的同时，保持对大规模分布式遍历的高性能支持。