# NebulaGraph src/graph 目录结构与功能分析

## 目录概览

`nebula-3.8.0/src/graph` 目录是 NebulaGraph 图数据库的**查询处理引擎**核心模块，包含了从查询解析、验证、规划、优化到执行的完整流程。下面详细说明各子目录的功能。

---

## 核心模块详解

### 1. **context/** - 执行上下文管理
**功能**: 为查询执行提供运行时上下文和状态管理

**包含内容**:
- `QueryContext`: 每个查询请求的完整上下文信息，贯穿解析、规划、优化、执行全过程
- `ExecutionContext`: 执行阶段的上下文，包含中间结果、变量绑定等
- `Iterator`: 数据集迭代器接口，用于处理结果集遍历
- `Symbols`: 符号表管理，记录查询中的变量和别名
- `ValidateContext`: 验证阶段的上下文
- `QueryExpressionContext`: 表达式求值的上下文
- `Result`: 查询结果封装
- **子目录**:
  - `ast/`: AST（抽象语法树）上下文，包含 Cypher 和 nGQL 的 AST 上下文
  - `iterator/`: 多种迭代器实现（DefaultIter、PropIter、SequentialIter 等）

**关键接口**:
- Schema 管理、索引管理、存储客户端访问
- 对象池管理，支持内存复用
- 字符集信息管理

---

### 2. **validator/** - 查询验证器
**功能**: 对 AST 进行语义检查和类型验证

**主要职责**:
- 验证查询语句的合法性（语义检查）
- 检查表达式类型一致性
- 权限验证（ACL）
- 生成执行计划（调用 Planner）

**核心验证器**:
- `Validator`: 基础验证器，定义验证流程
- `MatchValidator`: MATCH 语句验证
- `GoValidator`: GO 遍历语句验证
- `FetchVerticesValidator`: 获取顶点验证
- `FetchEdgesValidator`: 获取边验证
- `LookupValidator`: 索引查询验证
- `MaintainValidator`: CREATE/ALTER/DROP DDL 验证
- `MutateValidator`: INSERT/UPDATE/DELETE 数据修改验证
- `AdminValidator`: 管理命令验证
- `GroupByValidator`, `OrderByValidator`, `LimitValidator`: 子句验证
- `ACLValidator`: 权限检查
- 其他: UnwindValidator, PipeValidator, SetValidator, UseValidator 等

**验证流程**:
```
spaceChosen() → validateImpl() → checkPermission() → toPlan()
```

---

### 3. **planner/** - 查询计划生成器
**功能**: 将验证通过的 AST 转化为执行计划 DAG（有向无环图）

**核心组件**:
- `Planner`: 基础计划生成器，支持多种方言的查询语言
- `PlannersRegister`: 计划生成器注册机制
- `SequentialPlanner`: 顺序计划生成器

**子目录**:
- **match/** - Cypher MATCH 语句的计划生成
  - `MatchPlanner`, `MatchClausePlanner`, `MatchPathPlanner`, `MatchSolver`
  - `EdgeIndexSeek`, `LabelIndexSeek`, `PropIndexSeek`: 索引寻址策略
  - `VertexIdSeek`, `ScanSeek`: 顶点定位策略
  - `ShortestPathPlanner`: 最短路径规划
  - `StartVidFinder`, `SegmentsConnector`: 路径规划辅助
  - `OrderByClausePlanner`, `WithClausePlanner`, `WhereClausePlanner`: 子句规划
  - `UnwindClausePlanner`, `YieldClausePlanner`: 特殊子句规划

- **ngql/** - nGQL 语言的计划生成（NebulaGraph 特有语言）
  - `GoPlanner`: GO 语句规划
  - `FetchVerticesPlanner`, `FetchEdgesPlanner`: 数据获取规划
  - `LookupPlanner`: 索引查询规划
  - `PathPlanner`: 路径查询规划
  - `SubgraphPlanner`: 子图查询规划
  - `MaintainPlanner`: DDL 语句规划

- **plan/** - 执行计划节点定义
  - `PlanNode`: 执行计划的基础节点
  - `ExecutionPlan`: 完整执行计划
  - `Query.h`, `Admin.h`, `Mutate.h`, `Maintain.h`, `Algo.h`, `Logic.h`, `Scan.h`: 不同类型的计划节点

---

### 4. **optimizer/** - 查询优化器
**功能**: 对执行计划进行转换和优化，生成更高效的计划

**核心组件**:
- `Optimizer`: 主优化器，使用规则引擎优化计划
- `OptContext`: 优化上下文
- `OptGroup`: 优化组，表示等价的计划
- `OptRule`: 优化规则基类

**子目录**:
- **rule/** - 50+ 优化规则
  - **推下规则** (Push-down rules):
    - `PushFilterDown*`: 将过滤条件下推到存储层
    - `PushLimitDown*`: 将限制条件下推
    - `PushTopNDown*`: TopN 下推
  - **索引优化规则**:
    - `IndexScanRule`, `EdgeIndexFullScanRule`, `TagIndexFullScanRule`
    - `OptimizeTagIndexScanByFilterRule`, `OptimizeEdgeIndexScanByFilterRule`
    - `UnionAll*IndexScanRule`: 多索引合并
    - `GeoPredicateIndexScanRule`: 地理信息索引优化
  - **消除冗余规则**:
    - `EliminateFilterRule`, `EliminateAppendVerticesRule`, `RemoveNoopProjectRule`
  - **合并规则**:
    - `CombineFilterRule`, `CollapseProjectRule`
    - `MergeGetNbrs*Rule`: 邻接点获取合并
    - `MergeGetVertices*Rule`: 顶点获取合并
  - **其他规则**:
    - `TopNRule`, `GetEdgesTransformRule`, `CartesianProduct`

---

### 5. **executor/** - 执行引擎
**功能**: 根据优化后的执行计划执行具体的数据库操作

**核心组件**:
- `Executor`: 执行器基类，所有执行器的父类
- `StorageAccessExecutor`: 存储层访问执行器

**子目录**:
- **query/** - 查询操作执行器（40+ 执行器）
  - `GetVerticesExecutor`, `GetEdgesExecutor`: 数据获取
  - `GetNeighborsExecutor`: 邻接点查询
  - `GetPropExecutor`: 属性获取
  - `ScanVerticesExecutor`, `ScanEdgesExecutor`: 扫描操作
  - `IndexScanExecutor`, `FulltextIndexScanExecutor`: 索引扫描
  - `FilterExecutor`, `ProjectExecutor`: 过滤和投影
  - `AggregateExecutor`: 聚合操作
  - `SortExecutor`, `LimitExecutor`, `TopNExecutor`: 排序和限制
  - `JoinExecutor`, `InnerJoinExecutor`, `LeftJoinExecutor`: 联接操作
  - `UnionExecutor`, `IntersectExecutor`, `MinusExecutor`: 集合操作
  - `TraverseExecutor`, `ExpandExecutor`: 图遍历
  - `UnwindExecutor`, `AssignExecutor`: 特殊操作
  - `DataCollectExecutor`, `DedupExecutor`: 数据收集和去重

- **mutate/** - 数据修改执行器
  - `InsertExecutor`: 插入操作
  - `UpdateExecutor`: 更新操作
  - `DeleteExecutor`: 删除操作

- **maintain/** - 维护操作执行器
  - `TagExecutor`, `EdgeExecutor`: 标签和边类型操作
  - `TagIndexExecutor`, `EdgeIndexExecutor`: 索引操作
  - `FTIndexExecutor`: 全文索引操作

- **admin/** - 管理命令执行器
  - 用户管理: `CreateUserExecutor`, `DropUserExecutor`, `UpdateUserExecutor`
  - 角色管理: `GrantRoleExecutor`, `RevokeRoleExecutor`, `ListRolesExecutor`
  - 配置管理: `ConfigExecutor`
  - 监听器: `ListenerExecutor`
  - 快照: `SnapshotExecutor`
  - 查询管理: `KillQueryExecutor`, `ShowQueriesExecutor`
  - 其他: `AddHostsExecutor`, `ShowHostsExecutor`, `ZoneExecutor` 等

- **algo/** - 算法执行器
  - `ShortestPathExecutor`: 最短路径
  - `AllPathsExecutor`: 所有路径
  - `SubgraphExecutor`: 子图提取
  - `CartesianProductExecutor`: 笛卡尔积

- **logic/** - 逻辑控制执行器
  - `StartExecutor`: 启动节点
  - `SelectExecutor`: 分支选择
  - `LoopExecutor`: 循环
  - `ArgumentExecutor`, `PassThroughExecutor`: 流程控制

---

### 6. **scheduler/** - 执行调度器
**功能**: 协调和调度执行器的执行顺序

**核心组件**:
- `Scheduler`: 调度器基类
- `AsyncMsgNotifyBasedScheduler`: 基于异步消息通知的调度器

**职责**:
- 管理执行器间的依赖关系
- 并行执行独立的执行器
- 处理执行器完成事件

---

### 7. **service/** - 服务层
**功能**: 提供图数据库服务接口和请求处理

**核心组件**:
- `GraphService`, `GraphServer`: 服务启动和请求处理
- `GraphFlags`: 系统配置参数
- `QueryEngine`: 查询引擎，协调验证、规划、优化、执行
- `QueryInstance`: 查询实例管理

**安全相关**:
- `Authenticator`: 身份验证基类
- `PasswordAuthenticator`: 密码身份验证
- `CloudAuthenticator`: 云平台身份验证
- `PermissionManager`: 权限管理
- `PermissionCheck`: 权限检查

**其他**:
- `RequestContext`: 请求上下文

---

### 8. **session/** - 会话管理
**功能**: 管理客户端连接和会话状态

**核心组件**:
- `ClientSession`: 客户端会话
- `GraphSessionManager`: 会话管理器

**职责**:
- 会话生命周期管理
- 连接状态维护
- 会话数据存储

---

### 9. **visitor/** - 表达式访问器
**功能**: 对表达式树进行各种操作和分析（访问者设计模式）

**核心访问器** (16+ 类):
- `DeduceTypeVisitor`: 推导表达式类型
- `DeducePropsVisitor`: 推导属性访问
- `DeduceAliasTypeVisitor`: 推导别名类型
- `ExtractFilterExprVisitor`: 提取过滤表达式
- `ExtractPropExprVisitor`: 提取属性表达式
- `ExtractGroupSuiteVisitor`: 提取分组信息
- `FoldConstantExprVisitor`: 常量折叠优化
- `RewriteVisitor`: 表达式重写
- `EvaluableExprVisitor`: 判断表达式可否求值
- `FindVisitor`: 查找特定表达式
- `PropertyTrackerVisitor`: 跟踪属性使用
- `PrunePropertiesVisitor`: 属性裁剪
- `ValidatePatternExpressionVisitor`: 验证模式表达式
- `VidExtractVisitor`: 提取顶点 ID

---

### 10. **util/** - 工具函数库
**功能**: 为各个模块提供辅助工具函数

**包含内容**:
- `ExpressionUtils`: 表达式处理工具
- `SchemaUtil`: 模式工具
- `IndexUtil`: 索引工具
- `ValidateUtil`: 验证工具
- `OptimizerUtils`: 优化工具
- `PlannerUtil`: 规划工具
- `ParserUtil`: 解析工具
- `FTIndexUtils`: 全文索引工具
- `IdGenerator`: ID 生成
- `AnonVarGenerator`: 匿名变量生成
- `AnonColGenerator`: 匿名列生成
- `AstUtils`: AST 工具
- `Constants`: 常量定义
- `ToJson`: 转 JSON 工具
- `ZoneUtil`: 分区工具
- `Utils`: 通用工具

---

### 11. **stats/** - 统计信息
**功能**: 收集和管理查询执行的统计信息

**核心组件**:
- `GraphStats`: 图统计信息类

**用途**:
- 计划优化的统计数据
- 性能监控
- 执行分析

---

### 12. **gc/** - 垃圾回收
**功能**: 对象生命周期管理

**核心组件**:
- `GC`: 垃圾回收管理

---

## 查询处理流程

```
┌─────────────┐
│   Parser    │ 解析 (parser module)
└──────┬──────┘
       │ Sentence AST
       ▼
┌─────────────────────────┐
│     Validator           │ 验证 (validator module)
│ - 语义检查              │
│ - 类型推导              │ 使用 visitor 模块
│ - 权限检查              │
└──────┬──────────────────┘
       │ ValidateContext
       ▼
┌──────────────────────────┐
│      Planner             │ 规划 (planner module)
│ - Match 规划             │
│ - nGQL 规划              │
│ - 生成执行计划 DAG       │
└──────┬───────────────────┘
       │ ExecutionPlan
       ▼
┌──────────────────────────┐
│     Optimizer            │ 优化 (optimizer module)
│ - 应用优化规则           │
│ - 生成更优的执行计划     │ 使用 visitor 模块
└──────┬───────────────────┘
       │ OptimizedPlan
       ▼
┌──────────────────────────┐
│    Scheduler             │ 调度 (scheduler module)
│ - 依赖关系分析           │
│ - 执行顺序规划           │
└──────┬───────────────────┘
       │ ExecutionOrder
       ▼
┌──────────────────────────┐
│     Executor             │ 执行 (executor module)
│ - Query/Mutate/Admin     │
│ - 存储层交互             │ 使用 context 模块
│ - 结果返回               │
└──────┬───────────────────┘
       │ Result
       ▼
┌──────────────────────────┐
│   GraphService           │ 返回结果
└──────────────────────────┘
```

---

## 关键概念

### 1. **执行计划（Execution Plan）**
- 由 PlanNode 组成的有向无环图（DAG）
- 每个 PlanNode 代表一个执行步骤
- 优化后的计划被交给执行引擎执行

### 2. **上下文（Context）**
- `QueryContext`: 贯穿整个查询处理生命周期
- `ValidateContext`: 验证阶段的数据
- `ExecutionContext`: 执行阶段的中间结果

### 3. **访问者模式（Visitor Pattern）**
- 广泛用于表达式分析和转换
- 类型推导、属性提取、常量折叠等都基于此模式

### 4. **规则引擎（Rule-based Optimization）**
- 50+ 优化规则
- 使用费用模型选择最优计划
- 支持自定义优化规则

---

## 数据流向

```
客户端请求
    ↓
GraphService (service/)
    ↓
QueryEngine
    ↓
Parser → AST
    ↓
Validator (validator/) → 语义验证 + ValidateContext
                            ↓ (visitor/)
                            类型推导、权限检查
    ↓
Planner (planner/) → ExecutionPlan
    ↓
Optimizer (optimizer/) → 应用规则 + OptimizedPlan
                            ↓ (visitor/)
                            表达式分析
    ↓
Scheduler (scheduler/) → ExecutionOrder
    ↓
Executor (executor/) → 执行计划节点
    ↓
StorageClient → 存储层访问
    ↓
Result → ResponseContext
    ↓
客户端响应
```

---

## 模块间依赖关系

```
service/ (最上层)
  ├── validator/
  │   ├── planner/
  │   │   └── plan/ (执行计划节点定义)
  │   ├── visitor/
  │   └── util/
  ├── planner/
  │   ├── plan/
  │   ├── match/
  │   └── ngql/
  ├── optimizer/
  │   ├── rule/
  │   ├── visitor/
  │   └── util/
  ├── scheduler/
  ├── executor/
  │   ├── query/
  │   ├── mutate/
  │   ├── maintain/
  │   ├── admin/
  │   ├── algo/
  │   └── logic/
  └── context/ (贯穿全流程)
      ├── ast/
      ├── iterator/
      └── visitor/
```

---

## 模块特点总结

| 模块 | 职责 | 输入 | 输出 | 关键数据结构 |
|------|------|------|------|------------|
| context | 上下文管理 | - | QueryContext | QueryContext, ExecutionContext |
| validator | 语义验证 | AST | ValidateContext | Symbols, ExprProps |
| planner | 计划生成 | ValidateContext | ExecutionPlan | PlanNode DAG |
| optimizer | 计划优化 | ExecutionPlan | OptimizedPlan | OptGroup, OptRule |
| scheduler | 执行调度 | ExecutionPlan | ExecutionOrder | Executor 依赖图 |
| executor | 执行计划 | ExecutionPlan | Result | 执行结果 |
| visitor | 表达式分析 | Expression | 分析结果 | 各种 Visitor 类 |
| util | 工具函数 | 各类数据 | 工具结果 | 辅助函数 |
| service | 服务层 | 客户端请求 | 响应结果 | GraphService |
| session | 会话管理 | 连接 | 会话状态 | ClientSession |
| stats | 统计信息 | 执行数据 | 统计结果 | GraphStats |
| gc | 垃圾回收 | 对象 | 回收状态 | GC |

---

## 总体架构特点

1. **清晰的分层架构**: 从高层的服务接口到底层的存储访问
2. **规则驱动的优化**: 灵活的优化规则系统
3. **访问者模式**: 表达式分析和转换
4. **异步执行**: 使用 folly::Future 支持异步执行
5. **内存管理**: 对象池和内存追踪
6. **完整的 SQL 支持**: 支持 Cypher 和 nGQL 两种查询语言
7. **高效的迭代器**: 支持多种数据遍历方式

