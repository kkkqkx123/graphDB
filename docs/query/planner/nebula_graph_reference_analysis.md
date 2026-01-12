# 基于 Nebula-Graph 的查询规划器架构参考分析

## 概述

本文档通过分析 Nebula-Graph 3.8.0 的查询规划器实现，为当前 GraphDB 项目的架构调整提供参考。Nebula-Graph 作为一个成熟的图数据库，其查询规划器架构经过多年迭代，具有很高的参考价值。

## 1. Nebula-Graph 规划器架构概览

### 1.1 整体架构

```
nebula-3.8.0/src/graph/planner/
├── Planner.h                    # 基础规划器接口
├── SequentialPlanner.h          # 顺序执行规划器
├── PlannersRegister.h           # 规划器注册表
├── match/                       # Cypher MATCH 查询规划
│   ├── CypherClausePlanner.h    # Cypher 子句规划器基类
│   ├── MatchPlanner.h           # MATCH 查询主规划器
│   ├── MatchClausePlanner.h     # MATCH 子句规划器
│   ├── ReturnClausePlanner.h    # RETURN 子句规划器
│   ├── SegmentsConnector.h      # 计划段连接器
│   └── ...                      # 其他子句规划器
├── ngql/                        # NGQL 查询规划
└── plan/                        # 执行计划定义
    ├── PlanNode.h               # 计划节点基类
    └── ExecutionPlan.h          # 执行计划
```

### 1.2 核心设计原则

1. **分层设计**：清晰的分层架构，从基础接口到具体实现
2. **职责分离**：每个规划器有明确的职责范围
3. **可扩展性**：通过注册机制支持新查询类型的扩展
4. **统一接口**：所有规划器遵循统一的接口规范

## 2. 关键架构组件分析

### 2.1 基础规划器接口

#### 2.1.1 Planner 基类设计

```cpp
class Planner {
public:
  virtual ~Planner() = default;

  // 静态规划器映射表，按语句类型组织
  static auto& plannersMap() {
    static std::unordered_map<Sentence::Kind, std::vector<MatchAndInstantiate>> plannersMap;
    return plannersMap;
  }

  // 静态方法：根据 AST 上下文生成计划
  static StatusOr<SubPlan> toPlan(AstContext* astCtx);

  // 纯虚函数：子类必须实现的转换方法
  virtual StatusOr<SubPlan> transform(AstContext* astCtx) = 0;

protected:
  Planner() = default;
};
```

**设计优势**：
- **静态注册机制**：通过静态映射表管理所有规划器
- **统一入口**：`toPlan` 方法提供统一的计划生成入口
- **类型安全**：基于语句类型的强类型匹配

#### 2.1.2 匹配和实例化机制

```cpp
using MatchFunc = std::function<bool(AstContext* astContext)>;
using PlannerInstantiateFunc = std::function<std::unique_ptr<Planner>()>;

struct MatchAndInstantiate {
  MatchAndInstantiate(MatchFunc m, PlannerInstantiateFunc p)
      : match(std::move(m)), instantiate(std::move(p)) {}
  MatchFunc match;
  PlannerInstantiateFunc instantiate;
};
```

**设计优势**：
- **灵活匹配**：通过函数对象实现灵活的匹配逻辑
- **延迟实例化**：只在需要时创建规划器实例
- **解耦设计**：匹配逻辑和实例化逻辑分离

### 2.2 子句规划器架构

#### 2.2.1 CypherClausePlanner 基类

```cpp
class CypherClausePlanner {
public:
  CypherClausePlanner() = default;
  virtual ~CypherClausePlanner() = default;

  // 纯虚函数：子类必须实现的转换方法
  virtual StatusOr<SubPlan> transform(CypherClauseContextBase* clauseCtx) = 0;
};
```

**设计特点**：
- **简洁接口**：只有一个纯虚函数，接口简洁明了
- **上下文驱动**：基于上下文对象的计划生成
- **类型安全**：使用基类指针支持多态

#### 2.2.2 子句规划器实现模式

以 `ReturnClausePlanner` 为例：

```cpp
class ReturnClausePlanner final : public CypherClausePlanner {
public:
  ReturnClausePlanner() = default;

  StatusOr<SubPlan> transform(CypherClauseContextBase* clauseCtx) override;

  Status buildReturn(ReturnClauseContext* rctx, SubPlan& subPlan);
};
```

**实现特点**：
- **final 类**：防止继承，确保实现稳定性
- **分层方法**：`transform` 负责总体流程，`buildReturn` 负责具体构建
- **引用传递**：使用引用传递避免不必要的拷贝

### 2.3 计划连接机制

#### 2.3.1 SegmentsConnector 设计

```cpp
class SegmentsConnector final {
public:
  SegmentsConnector() = delete;  // 禁止实例化

  // 内连接
  static SubPlan innerJoin(QueryContext* qctx,
                           const SubPlan& left,
                           const SubPlan& right,
                           const std::unordered_set<std::string>& intersectedAliases);

  // 左连接
  static SubPlan leftJoin(QueryContext* qctx,
                          const SubPlan& left,
                          const SubPlan& right,
                          const std::unordered_set<std::string>& intersectedAliases);

  // 笛卡尔积
  static SubPlan cartesianProduct(QueryContext* qctx, const SubPlan& left, const SubPlan& right);

  // 模式应用
  static SubPlan rollUpApply(CypherClauseContextBase* ctx,
                             const SubPlan& left,
                             const SubPlan& right,
                             const graph::Path& path);

  // 顺序连接
  static SubPlan addInput(const SubPlan& left, const SubPlan& right, bool copyColNames = false);
};
```

**设计优势**：
- **静态方法**：所有方法都是静态的，无需实例化
- **统一接口**：所有连接操作都有统一的接口
- **上下文感知**：部分方法需要 QueryContext 或 CypherClauseContextBase
- **类型安全**：使用强类型的 SubPlan 参数

### 2.4 计划节点体系

#### 2.4.1 PlanNode 基类设计

```cpp
class PlanNode {
public:
  enum class Kind : uint8_t {
    kUnknown = 0,
    // Query nodes
    kGetNeighbors, kGetVertices, kGetEdges, kExpand, kExpandAll,
    kTraverse, kAppendVertices, kShortestPath,
    // Index nodes
    kIndexScan, kTagIndexFullScan, kTagIndexPrefixScan, kTagIndexRangeScan,
    // Processing nodes
    kFilter, kUnion, kIntersect, kMinus, kProject, kUnwind,
    kSort, kTopN, kLimit, kSample, kAggregate, kDedup,
    // Join nodes
    kInnerJoin, kHashLeftJoin, kHashInnerJoin, kCrossJoin,
    // Logic nodes
    kSelect, kLoop, kPassThrough, kStart,
    // ... 其他节点类型
  };

  // 虚函数接口
  virtual std::unique_ptr<PlanNodeDescription> explain() const;
  virtual void accept(PlanNodeVisitor* visitor);
  virtual PlanNode* clone() const = 0;
  virtual void calcCost();

  // 属性访问方法
  Kind kind() const;
  int64_t id() const;
  QueryContext* qctx() const;
  const std::string& outputVar() const;
  const std::vector<std::string>& colNames() const;

  // 依赖关系管理
  const PlanNode* dep(size_t index = 0) const;
  void setDep(size_t index, const PlanNode* dep);
  void addDep(const PlanNode* dep);
  size_t numDeps() const;

protected:
  PlanNode(QueryContext* qctx, Kind kind);
  virtual ~PlanNode() = default;
};
```

**设计特点**：
- **枚举类型**：使用枚举定义所有节点类型，类型安全
- **虚函数接口**：提供丰富的虚函数接口支持多态
- **依赖管理**：内置依赖关系管理功能
- **上下文关联**：每个节点都关联 QueryContext

#### 2.4.2 专用节点基类

```cpp
// 单依赖节点
class SingleDependencyNode : public PlanNode {
public:
  void dependsOn(const PlanNode* dep) {
    setDep(0, dep);
  }
protected:
  SingleDependencyNode(QueryContext* qctx, Kind kind, const PlanNode* dep);
};

// 单输入节点
class SingleInputNode : public SingleDependencyNode {
protected:
  SingleInputNode(QueryContext* qctx, Kind kind, const PlanNode* dep);
  void copyInputColNames(const PlanNode* input);
};

// 双输入节点
class BinaryInputNode : public PlanNode {
public:
  void setLeftDep(const PlanNode* left);
  void setRightDep(const PlanNode* right);
  const PlanNode* left() const;
  const PlanNode* right() const;
protected:
  BinaryInputNode(QueryContext* qctx, Kind kind, const PlanNode* left, const PlanNode* right);
};
```

**设计优势**：
- **特化基类**：为不同类型的节点提供特化的基类
- **类型安全**：通过继承体系确保类型安全
- **便利方法**：提供便利方法简化节点操作

### 2.5 执行计划结构

#### 2.5.1 SubPlan 结构

```cpp
struct SubPlan {
  // 子计划的根节点和尾节点
  PlanNode* root{nullptr};
  PlanNode* tail{nullptr};

  // 添加起始节点
  void appendStartNode(QueryContext* qctx);
};
```

**设计特点**：
- **简单结构**：只包含根节点和尾节点，结构简单
- **辅助方法**：提供辅助方法简化操作
- **指针语义**：使用指针而非智能指针，简化内存管理

#### 2.5.2 ExecutionPlan 类

```cpp
class ExecutionPlan final {
public:
  explicit ExecutionPlan(PlanNode* root = nullptr);
  ~ExecutionPlan();

  // 根节点管理
  void setRoot(PlanNode* root);
  PlanNode* root() const;

  // 性能分析
  void addProfileStats(int64_t planNodeId, ProfilingStats&& profilingStats);
  void describe(PlanDescription* planDesc);

  // 配置管理
  void setExplainFormat(const std::string& format);
  bool isProfileEnabled();

private:
  int32_t optimizeTimeInUs_{0};
  int64_t id_{-1};
  PlanNode* root_{nullptr};
  PlanDescription* planDescription_{nullptr};
  std::string explainFormat_;
};
```

**设计特点**：
- **封装完整**：封装了执行计划的所有相关信息
- **性能支持**：内置性能分析和描述功能
- **配置灵活**：支持不同的解释格式

## 3. 与当前 GraphDB 架构的对比分析

### 3.1 相似之处

#### 3.1.1 整体架构模式
- **分层设计**：两者都采用分层设计，从基础接口到具体实现
- **规划器注册**：都使用注册机制管理规划器
- **子句规划器**：都有专门的子句规划器处理不同子句

#### 3.1.2 核心概念
- **SubPlan 概念**：都使用 SubPlan 表示子计划
- **计划节点**：都有丰富的计划节点类型体系
- **连接机制**：都有专门的连接机制处理计划组合

### 3.2 关键差异

#### 3.2.1 接口设计差异

**Nebula-Graph**：
```cpp
// 简洁的接口设计
class CypherClausePlanner {
  virtual StatusOr<SubPlan> transform(CypherClauseContextBase* clauseCtx) = 0;
};
```

**当前 GraphDB**：
```rust
// 更复杂的接口设计
pub trait CypherClausePlanner: std::fmt::Debug {
    fn transform(
        &mut self,
        clause_ctx: &CypherClauseContext,
    ) -> Result<SubPlan, PlannerError>;
}
```

**差异分析**：
- Nebula-Graph 接口更简洁，只有一个核心方法
- 当前 GraphDB 接口更复杂，包含 Debug trait
- Nebula-Graph 使用基类指针，当前 GraphDB 使用 trait

#### 3.2.2 计划连接差异

**Nebula-Graph**：
```cpp
// 静态方法，无需实例化
class SegmentsConnector final {
  static SubPlan innerJoin(QueryContext* qctx, ...);
  static SubPlan leftJoin(QueryContext* qctx, ...);
  static SubPlan cartesianProduct(QueryContext* qctx, ...);
};
```

**当前 GraphDB**：
```rust
// 实例方法，需要创建实例
pub struct SegmentsConnector;
impl SegmentsConnector {
    pub fn new() -> Self { Self }
    pub fn inner_join(&self, left: SubPlan, right: SubPlan, ...) -> SubPlan;
}
```

**差异分析**：
- Nebula-Graph 使用静态方法，当前 GraphDB 使用实例方法
- Nebula-Graph 需要 QueryContext 参数，当前 GraphDB 不需要
- Nebula-Graph 的设计更简洁，当前 GraphDB 的设计更面向对象

#### 3.2.3 节点创建差异

**Nebula-Graph**：
```cpp
// 直接构造，没有工厂模式
auto filterNode = std::make_unique<FilterNode>(qctx, inputNode);
```

**当前 GraphDB**：
```rust
// 使用工厂模式
let filter_node = Arc::new(SingleInputNode::new(PlanNodeKind::Filter, input_node));
```

**差异分析**：
- Nebula-Graph 直接构造节点，当前 GraphDB 使用工厂模式
- Nebula-Graph 使用智能指针，当前 GraphDB 使用 Arc
- Nebula-Graph 的节点类型更具体，当前 GraphDB 的节点类型更通用

### 3.3 优劣势分析

#### 3.3.1 Nebula-Graph 的优势

1. **简洁性**：接口设计更简洁，易于理解和维护
2. **性能**：使用指针而非智能指针，性能更好
3. **成熟度**：经过多年迭代，架构更加成熟
4. **一致性**：整体设计更加一致

#### 3.3.2 当前 GraphDB 的优势

1. **类型安全**：Rust 的类型系统提供更好的类型安全
2. **内存安全**：Rust 的所有权系统确保内存安全
3. **并发安全**：Rust 的并发模型更安全
4. **现代化**：使用更现代的语言特性

## 4. 架构调整建议

### 4.1 接口简化建议

#### 4.1.1 简化 CypherClausePlanner 接口

**当前接口**：
```rust
pub trait CypherClausePlanner: std::fmt::Debug {
    fn transform(
        &mut self,
        clause_ctx: &CypherClauseContext,
    ) -> Result<SubPlan, PlannerError>;
}
```

**建议调整为**：
```rust
pub trait CypherClausePlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
    ) -> Result<SubPlan, PlannerError>;
}
```

**调整理由**：
- 移除 Debug trait，简化接口
- 添加 input_plan 参数，明确输入依赖
- 使用 &self 而非 &mut self，减少可变性需求

#### 4.1.2 统一规划器注册机制

**当前机制**：
```rust
// 分散的注册逻辑
registry.add_planner("MATCH".to_string(), match_and_instantiate);
```

**建议调整为**：
```rust
// 集中的注册机制
impl PlannerRegistry {
    pub fn register_match_planners(&mut self) {
        self.register_planner(
            SentenceKind::Match,
            MatchPlanner::match_ast_ctx,
            MatchPlanner::make,
        );
    }
}
```

**调整理由**：
- 使用枚举类型而非字符串，类型更安全
- 集中注册逻辑，更易维护
- 参考 Nebula-Graph 的注册模式

### 4.2 计划连接机制调整

#### 4.2.1 静态化连接方法

**当前实现**：
```rust
pub struct SegmentsConnector;
impl SegmentsConnector {
    pub fn new() -> Self { Self }
    pub fn inner_join(&self, ...) -> SubPlan;
}
```

**建议调整为**：
```rust
pub struct SegmentsConnector;
impl SegmentsConnector {
    pub fn inner_join(
        qctx: &QueryContext,
        left: &SubPlan,
        right: &SubPlan,
        intersected_aliases: &HashSet<String>,
    ) -> Result<SubPlan, PlannerError>;
}
```

**调整理由**：
- 使用静态方法，无需实例化
- 添加 QueryContext 参数，提供更多上下文
- 参考 Nebula-Graph 的设计模式

#### 4.2.2 统一连接接口

**建议新增统一接口**：
```rust
pub trait ConnectionStrategy {
    fn connect(
        &self,
        qctx: &QueryContext,
        left: &SubPlan,
        right: &SubPlan,
        params: &ConnectionParams,
    ) -> Result<SubPlan, PlannerError>;
}

pub enum ConnectionType {
    InnerJoin,
    LeftJoin,
    Cartesian,
    RollUpApply,
    PatternApply,
    Sequential,
}
```

**调整理由**：
- 提供统一的连接接口
- 支持不同类型的连接策略
- 便于扩展新的连接类型

### 4.3 节点体系调整

#### 4.3.1 简化节点创建

**当前实现**：
```rust
// 使用工厂模式
let node = Arc::new(SingleInputNode::new(PlanNodeKind::Filter, input));
```

**建议调整为**：
```rust
// 直接构造
let node = FilterNode::new(qctx, input)?;
```

**调整理由**：
- 简化节点创建过程
- 提供更具体的节点类型
- 参考 Nebula-Graph 的直接构造模式

#### 4.3.2 增强节点类型系统

**建议新增**：
```rust
// 更具体的节点类型
pub struct FilterNode {
    base: SingleInputNode,
    condition: Expression,
}

impl FilterNode {
    pub fn new(qctx: &QueryContext, input: Arc<dyn PlanNode>, condition: Expression) -> Result<Self, PlannerError> {
        Ok(Self {
            base: SingleInputNode::new(PlanNodeKind::Filter, input),
            condition,
        })
    }
    
    pub fn condition(&self) -> &Expression {
        &self.condition
    }
}
```

**调整理由**：
- 提供更具体的节点类型
- 封装节点特定的逻辑
- 提高类型安全性

### 4.4 数据流管理调整

#### 4.4.1 明确数据流方向

**建议新增数据流抽象**：
```rust
pub trait DataFlowNode {
    fn input_requirements(&self) -> Vec<VariableRequirement>;
    fn output_provides(&self) -> Vec<VariableProvider>;
    fn can_start_flow(&self) -> bool;
    fn flow_direction(&self) -> FlowDirection;
}

pub enum FlowDirection {
    Source,     // 数据源：MATCH, LOOKUP
    Transform,  // 转换：WHERE, WITH, UNWIND
    Output,     // 输出：RETURN, YIELD
    Combine,    // 组合：UNION, JOIN
}
```

**调整理由**：
- 明确数据流方向
- 支持数据流验证
- 防止起始节点创建问题

#### 4.4.2 增强上下文传递

**建议调整上下文设计**：
```rust
pub struct PlanningContext {
    pub query_context: QueryContext,
    pub available_variables: HashMap<String, VariableType>,
    pub generated_variables: HashMap<String, VariableType>,
    pub current_scope: Vec<String>,
    pub flow_stack: Vec<FlowDirection>,
}

impl PlanningContext {
    pub fn push_scope(&mut self, direction: FlowDirection) {
        self.flow_stack.push(direction);
    }
    
    pub fn pop_scope(&mut self) -> Option<FlowDirection> {
        self.flow_stack.pop()
    }
    
    pub fn current_flow_direction(&self) -> FlowDirection {
        self.flow_stack.last().copied().unwrap_or(FlowDirection::Source)
    }
}
```

**调整理由**：
- 提供更丰富的上下文信息
- 支持数据流方向跟踪
- 防止架构不一致性问题

## 5. 实施路线图

### 5.1 第一阶段：接口调整（1-2 周）

1. **简化 CypherClausePlanner 接口**
   - 移除不必要的 trait 约束
   - 添加 input_plan 参数
   - 统一错误处理

2. **统一规划器注册机制**
   - 使用枚举类型替代字符串
   - 集中注册逻辑
   - 改进匹配机制

### 5.2 第二阶段：连接机制调整（2-3 周）

1. **静态化连接方法**
   - 将 SegmentsConnector 方法改为静态
   - 添加 QueryContext 参数
   - 统一连接接口

2. **实现统一连接策略**
   - 定义 ConnectionStrategy trait
   - 实现不同连接策略
   - 支持策略选择

### 5.3 第三阶段：节点体系调整（3-4 周）

1. **简化节点创建**
   - 提供具体的节点类型
   - 简化创建过程
   - 保持类型安全

2. **增强节点类型系统**
   - 实现具体的节点类型
   - 封装节点特定逻辑
   - 提供便利方法

### 5.4 第四阶段：数据流管理（2-3 周）

1. **实现数据流抽象**
   - 定义 DataFlowNode trait
   - 实现流方向跟踪
   - 添加数据流验证

2. **增强上下文传递**
   - 重新设计 PlanningContext
   - 支持流栈管理
   - 提供上下文验证

## 6. 风险评估与缓解

### 6.1 主要风险

1. **兼容性风险**：接口调整可能影响现有代码
2. **性能风险**：新的抽象可能影响性能
3. **复杂性风险**：新的机制可能增加复杂性

### 6.2 缓解措施

1. **渐进式迁移**：分阶段实施，保持向后兼容
2. **性能测试**：每个阶段都进行性能测试
3. **文档完善**：提供详细的迁移指南和文档

## 7. 结论

通过分析 Nebula-Graph 的查询规划器架构，我们可以看到其设计的简洁性和成熟度。虽然当前 GraphDB 项目在类型安全和内存安全方面有优势，但在架构一致性和简洁性方面还有改进空间。

建议的架构调整方案在保持 Rust 语言优势的同时，借鉴 Nebula-Graph 的成熟设计，可以显著提升架构的一致性、可维护性和可扩展性。

关键改进点包括：
1. 简化接口设计，提高一致性
2. 统一连接机制，减少重复代码
3. 增强节点类型系统，提高类型安全
4. 实现数据流抽象，防止架构问题

通过分阶段实施这些改进，可以在保持系统稳定性的同时，逐步提升架构质量。