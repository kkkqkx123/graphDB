# Symbol 模块重构分析报告

## 1. 概述

本报告对比分析 GraphDB 的 Rust 实现与原生 NebulaGraph C++ 实现中符号表（SymbolTable）的设计差异，并提出重构建议。

**分析文件位置：**
- NebulaGraph C++ 源码：`nebula-3.8.0/src/graph/context/Symbols.{h,cpp}`
- 当前 Rust 实现：`src/core/symbol/`

---

## 2. NebulaGraph C++ 实现分析

### 2.1 核心数据结构

#### Variable 结构体

```cpp
struct Variable {
  std::string name;
  Value::Type type{Value::Type::DATASET};
  std::vector<std::string> colNames;
  std::unordered_set<PlanNode*> readBy;
  std::unordered_set<PlanNode*> writtenBy;
  std::atomic<uint64_t> userCount{0};
};
```

**设计特点：**
- 依赖关系（readBy/writtenBy）直接嵌入 Variable 结构内
- type 使用 NebulaGraph 的 Value::Type 枚举
- colNames 存储输出列名（类型为 DATASET 时有效）
- userCount 原子计数器用于追踪变量使用频率

#### SymbolTable 类

```cpp
class SymbolTable final {
 public:
  explicit SymbolTable(ObjectPool* objPool, ExecutionContext* ectx);
  
  bool existsVar(const std::string& varName) const;
  Variable* newVariable(const std::string& name);
  bool readBy(const std::string& varName, PlanNode* node);
  bool writtenBy(const std::string& varName, PlanNode* node);
  bool deleteReadBy(const std::string& varName, PlanNode* node);
  bool deleteWrittenBy(const std::string& varName, PlanNode* node);
  bool updateReadBy(const std::string& oldVar, const std::string& newVar, PlanNode* node);
  bool updateWrittenBy(const std::string& oldVar, const std::string& newVar, PlanNode* node);
  Variable* getVar(const std::string& varName);
  std::string toString() const;

 private:
  void addVar(std::string varName, Variable* variable);
  ObjectPool* objPool_{nullptr};
  ExecutionContext* ectx_{nullptr};
  mutable folly::RWSpinLock lock_;
  std::unordered_map<std::string, Variable*> vars_;
};
```

### 2.2 关键设计决策

| 决策点 | C++ 实现 | 说明 |
|--------|----------|------|
| 内存管理 | ObjectPool | 预先分配内存，所有权由 SymbolTable 控制 |
| 并发控制 | folly::RWSpinLock | 读写分离锁，优化读多写少场景 |
| 依赖集成 | 内嵌在 Variable | PlanNode 指针直接存储在 readBy/writtenBy |
| 上下文关联 | 强依赖 ExecutionContext | 创建变量时调用 `ectx_->initVar(name)` |

### 2.3 依赖关系示例

从 `SymbolsTest.cpp` 中的测试用例可以看出依赖关系的实际使用：

```cpp
// GO 查询的变量依赖关系
// __Start_1: writtenBy {1}, readBy {}
// __Expand_2: writtenBy {2}, readBy {3}
// __Project_4: writtenBy {4}, readBy {9}
// ...
```

---

## 3. 当前 Rust 实现分析

### 3.1 核心数据结构

#### Symbol 结构体

```rust
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub created_at: std::time::SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Variable,
    Alias,
    Parameter,
    Function,
}
```

#### SymbolTable 结构体

```rust
pub struct SymbolTable {
    symbols: Arc<RwLock<HashMap<String, Symbol>>>,
    dependency_tracker: Arc<RwLock<DependencyTracker>>,
}
```

#### DependencyTracker 独立结构

```rust
pub struct DependencyTracker {
    dependencies: HashMap<String, VariableDependencies>,
}

pub struct VariableDependencies {
    pub variable_name: String,
    pub readers: HashSet<PlanNodeRef>,
    pub writers: HashSet<PlanNodeRef>,
    pub dependencies: Vec<Dependency>,
    pub user_count: std::sync::atomic::AtomicU64,
}
```

### 3.2 架构差异

| 方面 | C++ 实现 | Rust 实现 |
|------|----------|-----------|
| 依赖存储位置 | Variable 内部 | 独立的 DependencyTracker |
| 内存管理 | ObjectPool | Arc<RwLock<T>> |
| 并发控制 | folly::RWSpinLock | std::sync::RwLock |
| 类型系统 | Value::Type | 自定义 SymbolType |
| PlanNode 引用 | 原始指针 * | 包装的 PlanNodeRef |

---

## 4. 主要差异与问题

### 4.1 架构过于复杂

**问题描述：**
当前 Rust 实现将依赖跟踪分离到独立的 `DependencyTracker`，导致：
- 增加了不必要的间接层
- API 调用路径变长
- 难以与 C++ 实现对齐

**C++ 做法：**
依赖关系直接存储在 `Variable.readBy` 和 `Variable.writtenBy` 中，查询和更新操作简单直接。

### 4.2 类型系统不完整

**问题描述：**
当前 `SymbolType` 枚举缺少关键类型：
- 缺少 `DATASET` 类型（C++ 中 Variable 默认类型）
- 缺少 `VERTEX` 和 `EDGE` 类型
- 未与 GraphDB 的 `DataType` 系统集成

**C++ 做法：**
直接使用 `Value::Type` 枚举，包含所有 NebulaGraph 值类型。

### 4.3 缺少上下文集成

**问题描述：**
当前实现中 `SymbolTable` 与 `ExecutionContext` 完全独立：
- 创建变量时不会初始化上下文
- 无法追踪变量的版本历史
- 与 QueryContext 的集成不完整

**C++ 做法：**
```cpp
Variable* SymbolTable::newVariable(const std::string& name) {
  auto* variable = objPool_->makeAndAdd<Variable>(name);
  addVar(name, variable);
  ectx_->initVar(name);  // 同步初始化执行上下文
  return variable;
}
```

### 4.4 PlanNodeRef 抽象过度

**问题描述：**
`PlanNodeRef` 是自定义的包装类型，相比 C++ 的原始指针增加了复杂度：
- 需要额外的序列化/克隆逻辑
- 与实际的执行计划系统解耦
- 增加了维护负担

**C++ 做法：**
直接使用 `PlanNode*` 原始指针，简洁明了。

### 4.5 用户计数未充分利用

**C++ 实现：**
- `Variable.userCount` 原子计数器用于追踪变量使用频率
- 可用于优化：清理未使用的变量、决定变量保留策略

**Rust 实现：**
- 虽有 `user_count` 字段，但未被生产代码使用

---

## 5. 重构建议

### 5.1 简化架构：内嵌依赖关系

**目标：** 将 `DependencyTracker` 的功能合并到 `Symbol` 结构中

```rust
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub data_type: Option<DataType>,      // 新增：数据类型
    pub col_names: Vec<String>,           // 新增：列名列表
    pub readers: HashSet<PlanNodeRef>,    // 内嵌读取者
    pub writers: HashSet<PlanNodeRef>,    // 内嵌写入者
    pub user_count: Arc<AtomicU64>,       // 内嵌使用计数
    pub created_at: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Variable,
    Alias,
    Parameter,
    Function,
    Dataset,     // 新增：对应 C++ 的 DATASET
    Vertex,      // 新增：顶点类型
    Edge,        // 新增：边类型
    Path,        // 新增：路径类型
}
```

**优势：**
- 消除 `DependencyTracker` 间接层
- API 调用更直接
- 与 C++ 架构对齐

### 5.2 集成执行上下文

**目标：** 在创建变量时同步初始化执行上下文

```rust
impl SymbolTable {
    pub fn new_variable(
        &self, 
        name: &str, 
        execution_context: &ExecutionContext
    ) -> Result<Symbol, String> {
        // 检查是否已存在
        if self.has_variable(name) {
            return Err(format!("Variable '{}' already exists", name));
        }
        
        // 同步初始化执行上下文
        execution_context.init_var(name);
        
        // 创建符号
        let symbol = Symbol::new(name.to_string(), SymbolType::Variable);
        // ... 插入逻辑
        Ok(symbol)
    }
}
```

### 5.3 完善类型系统

**目标：** 与 GraphDB 的 `DataType` 系统集成

```rust
pub enum SymbolType {
    Variable,
    Alias, 
    Parameter,
    Function,
    Dataset(DataType),           // 关联数据类型
    Vertex,
    Edge,
    Path,
}
```

### 5.4 优化并发控制

**当前问题：** 使用 `Arc<RwLock<HashMap>>` 嵌套，锁粒度较粗

**建议方案：** 参照 C++ 的 `folly::RWSpinLock`，优化读写分离

```rust
pub struct SymbolTable {
    // 使用更细粒度的锁策略
    symbols: DashMap<String, Symbol>,  // 并发 HashMap
}
```

### 5.5 添加缺失功能

#### 5.5.1 变量重命名

```rust
impl SymbolTable {
    pub fn rename_variable(&self, old_name: &str, new_name: &str) -> Result<(), String> {
        // 参照 C++ updateReadBy/updateWrittenBy 实现
        // 需要同时更新 readBy 和 writtenBy 中的引用
    }
}
```

#### 5.5.2 冲突检测

```rust
impl SymbolTable {
    pub fn detect_write_conflicts(&self) -> Vec<(String, Vec<PlanNodeRef>)> {
        self.symbols
            .iter()
            .filter(|(_, sym)| sym.writers.len() > 1)
            .map(|(name, sym)| (name.clone(), sym.writers.iter().cloned().collect()))
            .collect()
    }
}
```

---

## 6. 重构优先级

### 高优先级（P0）

| 任务 | 描述 | 影响范围 |
|------|------|----------|
| 内嵌依赖关系 | 将 DependencyTracker 功能合并到 Symbol | symbol_table.rs, mod.rs |
| 完善类型系统 | 添加 Dataset/Vertex/Edge/Path 类型 | symbol_table.rs, types.rs |
| 上下文集成 | SymbolTable 与 ExecutionContext 关联 | query_execution.rs, symbol_table.rs |

### 中优先级（P1）

| 任务 | 描述 | 影响范围 |
|------|------|----------|
| 优化并发控制 | 评估 DashMap 或其他并发结构 | symbol_table.rs |
| 冲突检测 | 实现 detect_write_conflicts 生产调用 | symbol_table.rs |
| 用户计数利用 | 利用 user_count 进行优化 | 全局 |

### 低优先级（P2）

| 任务 | 描述 | 影响范围 |
|------|------|----------|
| 简化 PlanNodeRef | 评估是否需要简化抽象 | plan_node_ref.rs |
| 文档完善 | 补充 API 文档和使用示例 | symbol/*.rs |

---

## 7. 实施步骤

### 阶段一：架构调整

1. 修改 `Symbol` 结构体，添加 `readers`、`writers`、`col_names` 字段
2. 修改 `SymbolType` 枚举，添加缺失类型
3. 移除 `DependencyTracker` 独立结构，或将其降级为内部辅助类
4. 更新 `SymbolTable` 方法签名，移除对 `DependencyTracker` 的依赖

### 阶段二：上下文集成

1. 在 `QueryContext` 中建立 `SymbolTable` 与 `ExecutionContext` 的关联
2. 修改 `new_variable` 方法，调用 `execution_context.init_var()`
3. 更新 `ValidationContext` 中的符号表使用方式

### 阶段三：功能完善

1. 实现变量重命名功能（包含依赖关系更新）
2. 实现冲突检测功能
3. 完善单元测试，覆盖重构后的 API

### 阶段四：优化和验证

1. 性能测试，对比重构前后的性能差异
2. 并发安全性验证
3. 内存使用优化

---

## 8. 风险评估

| 风险 | 级别 | 缓解措施 |
|------|------|----------|
| 重构导致现有功能破坏 | 中 | 渐进式重构，保持 API 兼容 |
| 并发控制复杂度增加 | 低 | 使用成熟并发库（DashMap） |
| 与现有代码集成困难 | 中 | 逐步集成，频繁测试 |

---

## 9. 参考资料

- **C++ 源码：** `nebula-3.8.0/src/graph/context/Symbols.{h,cpp}`
- **测试用例：** `nebula-3.8.0/src/graph/validator/test/SymbolsTest.cpp`
- **执行上下文：** `nebula-3.8.0/src/graph/context/ExecutionContext.h`
- **查询上下文：** `nebula-3.8.0/src/graph/context/QueryContext.h`

---

## 10. 总结

当前 Rust 实现的 `symbol` 模块设计过于复杂，与原生 C++ 实现存在显著差异。主要问题包括：

1. **过度抽象**：独立的 `DependencyTracker` 增加了不必要的间接层
2. **类型不完整**：缺少关键类型（Dataset、Vertex、Edge）
3. **上下文脱节**：未与执行上下文集成
4. **功能未充分利用**：依赖跟踪和用户计数功能未被生产代码使用

通过本报告提出的重构方案，可以：
- 简化架构，消除间接层
- 完善类型系统，与 GraphDB 整体设计对齐
- 加强上下文集成，提升模块协同效率
- 释放潜在功能，为后续优化提供基础

建议按照优先级分阶段实施，优先完成架构调整和上下文集成。
