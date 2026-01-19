# 符号表模块与 nebula-graph 对比分析

## 概述

本文档对比分析当前 Rust 实现的符号表模块与 nebula-graph 原始实现的差异，识别需要集成的功能和完全多余的功能。

## nebula-graph 符号表实现分析

### 核心数据结构

#### Variable 结构（nebula-3.8.0/src/graph/context/Symbols.h）

```cpp
struct Variable {
  std::string name;
  Value::Type type{Value::Type::DATASET};
  std::vector<std::string> colNames;  // Valid if type is dataset
  
  std::unordered_set<PlanNode*> readBy;   // 读取该变量的计划节点集合
  std::unordered_set<PlanNode*> writtenBy; // 写入该变量的计划节点集合
  
  std::atomic<uint64_t> userCount{0};     // 变量使用计数
};
```

**关键发现**：
- nebula-graph 中**没有** SymbolType 枚举，只有一种 Variable 类型
- 变量类型通过 `Value::Type` 表示（DATASET, BOOL, INT, STRING 等）
- `readBy` 和 `writtenBy` 直接存储 `PlanNode*` 指针

#### SymbolTable 结构

```cpp
class SymbolTable final {
  ObjectPool* objPool_;
  ExecutionContext* ectx_;
  folly::RWSpinLock lock_;
  std::unordered_map<std::string, Variable*> vars_;
  
  // 核心方法
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
};
```

### nebula-graph 中的实际使用场景

#### 1. PlanNode 构造时自动注册变量

**文件**：nebula-3.8.0/src/graph/planner/plan/PlanNode.cpp

```cpp
PlanNode::PlanNode(QueryContext* qctx, Kind kind) : qctx_(qctx), kind_(kind) {
  id_ = qctx_->genId();
  auto varName = folly::stringPrintf("__%s_%ld", toString(kind_), id_);
  auto* variable = qctx_->symTable()->newVariable(varName);
  outputVar_ = variable;
  qctx_->symTable()->writtenBy(varName, this);  // 自动标记写入依赖
}
```

**关键点**：
- 每个 PlanNode 在构造时自动创建输出变量
- 变量名格式：`__<NodeType>_<NodeId>`（如 `__GetNeighbors_42`）
- 自动调用 `writtenBy` 标记写入关系

#### 2. PlanNode 输入/输出变量管理

**文件**：nebula-3.8.0/src/graph/planner/plan/PlanNode.cpp

```cpp
void PlanNode::setInputVar(const std::string& varname) {
  inputVars_.emplace_back(varname);
  auto varPtr = qctx_->symTable()->getVar(varname);
  if (varPtr) {
    qctx_->symTable()->readBy(varPtr->name, this);  // 标记读取依赖
  }
}

void PlanNode::setOutputVar(const std::string& var) {
  auto* outputVarPtr = qctx_->symTable()->getVar(var);
  if (outputVarPtr) {
    qctx_->symTable()->updateWrittenBy(oldVar, var, this);  // 更新写入依赖
  }
  oldVar = var;
  outputVar_ = outputVarPtr;
}
```

#### 3. 优化器中的数据流验证

**文件**：nebula-3.8.0/src/graph/optimizer/OptRule.cpp

```cpp
bool OptRule::checkDataflowDeps(OptContext *ctx,
                                const MatchedResult &matched,
                                const std::string &var,
                                bool isRoot) const {
  auto node = matched.node;
  auto planNode = node->node();
  const auto &outVarName = planNode->outputVar();
  
  auto symTbl = ctx->qctx()->symTable();
  auto outVar = symTbl->getVar(outVarName);
  
  // 检查数据流是否与控制流一致
  if (!isRoot) {
    for (auto pnode : outVar->readBy) {  // 遍历读取该变量的所有节点
      auto optGNode = ctx->findOptGroupNodeByPlanNodeId(pnode->id());
      // 忽略 Argument plan node 引入的数据依赖
      if (!optGNode || optGNode->node()->kind() == graph::PlanNode::Kind::kArgument) continue;
      
      const auto &deps = optGNode->dependencies();
      auto found = std::find(deps.begin(), deps.end(), node->group());
      if (found == deps.end()) {
        VLOG(2) << ctx->qctx()->symTable()->toString();
        return false;
      }
    }
  }
  return true;
}
```

**关键点**：
- 优化器使用 `readBy` 来验证数据流依赖关系
- 检查每个读取变量的节点是否在依赖列表中
- Argument 节点作为特殊情况处理

#### 4. 优化器规则中使用变量信息

**文件**：nebula-3.8.0/src/graph/optimizer/rule/PushFilterDownInnerJoinRule.cpp

```cpp
auto symTable = octx->qctx()->symTable();
std::vector<std::string> leftVarColNames = symTable->getVar(leftVar.first)->colNames;
```

**文件**：nebula-3.8.0/src/graph/optimizer/rule/PushLimitDownProjectRule.cpp

```cpp
auto *varPtr = octx->qctx()->symTable()->getVar(projInputVar);
```

**关键点**：
- 优化器规则使用 `getVar()` 获取变量信息
- 主要使用 `colNames` 字段进行类型检查和优化

#### 5. Planner 中使用依赖信息

**文件**：nebula-3.8.0/src/graph/planner/ngql/GoPlanner.cpp

```cpp
auto* varPtr = qctx->symTable()->getVar(varName);
DCHECK_EQ(varPtr->writtenBy.size(), 1);  // 检查变量只被一个节点写入
for (auto node : varPtr->writtenBy) {
  // 处理写入节点
}
```

**关键点**：
- 使用 `writtenBy` 检查变量的唯一写入者
- 用于验证查询计划的正确性

## 当前 Rust 实现分析

### SymbolType 枚举对比

```rust
pub enum SymbolType {
    Variable,   // ✓ 被使用
    Alias,      // ✗ 未使用
    Parameter,  // ✗ 未使用
    Function,   // ✗ 未使用
    Dataset,    // ✓ 被使用
    Vertex,     // ✗ 未使用
    Edge,       // ✗ 未使用
    Path,       // ✗ 未使用
}
```

**问题**：
- nebula-graph 中**没有** SymbolType 枚举
- 变量类型通过 `Value::Type` 表示（DATASET, BOOL, INT, STRING 等）
- 当前实现过度设计了符号类型系统

### Symbol 结构对比

```rust
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,      // ✗ 过度设计
    pub col_names: Vec<String>,       // ✓ 对应 colNames
    pub readers: HashSet<PlanNodeRef>, // ✓ 对应 readBy
    pub writers: HashSet<PlanNodeRef>, // ✓ 对应 writtenBy
    pub user_count: Arc<AtomicU64>,   // ✗ 未实际使用
    pub created_at: SystemTime,       // ✗ nebula-graph 中没有
}
```

**问题**：
- `symbol_type` 字段过度设计，nebula-graph 中没有
- `user_count` 在 nebula-graph 中存在但未被实际使用
- `created_at` 是多余字段

### SymbolTable 方法对比

| 方法 | nebula-graph | Rust 实现 | 实际使用 |
|------|-------------|-----------|----------|
| `existsVar` / `has_variable` | ✓ | ✓ | ✓ |
| `newVariable` / `new_variable` | ✓ | ✓ | ✓ |
| `getVar` / `get_variable` | ✓ | ✓ | ✓ |
| `readBy` / `read_by` | ✓ | ✓ | ✗ 仅测试 |
| `writtenBy` / `written_by` | ✓ | ✓ | ✗ 仅测试 |
| `deleteReadBy` / `delete_read_by` | ✓ | ✓ | ✗ 未使用 |
| `deleteWrittenBy` / `delete_written_by` | ✓ | ✓ | ✗ 未使用 |
| `updateReadBy` / `update_read_by` | ✓ | ✓ | ✗ 未使用 |
| `updateWrittenBy` / `update_written_by` | ✓ | ✓ | ✗ 未使用 |
| `toString` / `to_string` | ✓ | ✓ | ✗ 未使用 |
| `get_readers` | ✗ | ✓ | ✗ 仅测试 |
| `get_writers` | ✗ | ✓ | ✗ 仅测试 |
| `get_variables_read_by` | ✗ | ✓ | ✗ 未使用 |
| `get_variables_written_by` | ✗ | ✓ | ✗ 未使用 |
| `detect_write_conflicts` | ✗ | ✓ | ✗ 仅测试 |
| `rename_variable` | ✗ | ✓ | ✗ 仅测试 |
| `new_dataset` | ✗ | ✓ | ✓ |

**问题**：
- 添加了 nebula-graph 中不存在的方法（`get_readers`, `get_writers`, `detect_write_conflicts` 等）
- 核心方法（`readBy`, `writtenBy`, `updateReadBy`, `updateWrittenBy`）未被实际使用

## 集成需求分析

### 需要集成的功能

#### 1. PlanNode 构造时自动注册变量

**目标模块**：`src/query/planner/plan/core/nodes/`

**集成点**：
- 在 `PlanNode` trait 的实现中，构造时自动创建输出变量
- 变量名格式：`__<NodeType>_<NodeId>`
- 自动调用 `written_by` 标记写入关系

**示例代码**：
```rust
impl PlanNode for SomeNode {
    fn new(qctx: &QueryContext) -> Self {
        let id = qctx.gen_id();
        let var_name = format!("__SomeNode_{}", id);
        let _ = qctx.sym_table().new_variable(&var_name);
        qctx.sym_table().written_by(&var_name, PlanNodeRef::new(id));
        
        Self { id, output_var: var_name, ... }
    }
}
```

#### 2. PlanNode 输入/输出变量管理

**目标模块**：`src/query/planner/plan/core/nodes/`

**集成点**：
- 在 `set_input_var` 方法中调用 `read_by`
- 在 `set_output_var` 方法中调用 `update_written_by`

**示例代码**：
```rust
impl SomeNode {
    fn set_input_var(&mut self, qctx: &QueryContext, var_name: &str) {
        self.input_vars.push(var_name.to_string());
        if let Some(_) = qctx.sym_table().get_variable(var_name) {
            let _ = qctx.sym_table().read_by(var_name, PlanNodeRef::new(self.id));
        }
    }
    
    fn set_output_var(&mut self, qctx: &QueryContext, var_name: &str) {
        if let Some(_) = qctx.sym_table().get_variable(var_name) {
            let _ = qctx.sym_table().update_written_by(&self.output_var, var_name, PlanNodeRef::new(self.id));
        }
        self.output_var = var_name;
    }
}
```

#### 3. 优化器中的数据流验证

**目标模块**：`src/query/optimizer/`

**集成点**：
- 在优化规则中使用 `readBy` 验证数据流依赖
- 检查每个读取变量的节点是否在依赖列表中

**示例代码**：
```rust
fn check_dataflow_deps(ctx: &OptContext, matched: &MatchedResult, var: &str, is_root: bool) -> bool {
    let node = matched.node;
    let plan_node = node.plan_node();
    let out_var_name = plan_node.output_var();
    
    let sym_tbl = ctx.qctx().sym_table();
    let out_var = sym_tbl.get_variable(out_var_name);
    
    if let Some(var) = out_var {
        if !is_root {
            for pnode_ref in &var.readers {
                let opt_g_node = ctx.find_opt_group_node_by_plan_node_id(pnode_ref.node_id());
                // 忽略 Argument plan node 引入的数据依赖
                if let Some(g_node) = opt_g_node {
                    if g_node.node().kind() == PlanNodeKind::Argument {
                        continue;
                    }
                }
                
                let deps = opt_g_node.dependencies();
                if !deps.contains(&node.group()) {
                    log::warn!("{}", ctx.qctx().sym_table().to_string());
                    return false;
                }
            }
        }
    }
    
    true
}
```

#### 4. 优化器规则中使用变量信息

**目标模块**：`src/query/optimizer/rule/`

**集成点**：
- 在优化规则中使用 `getVar()` 获取变量信息
- 使用 `colNames` 字段进行类型检查和优化

**示例代码**：
```rust
fn push_filter_down_inner_join(ctx: &mut OptContext, group_node: &OptGroupNode) -> Result<OptGroupNode, String> {
    let sym_table = ctx.qctx().sym_table();
    let left_var = sym_table.get_variable(&left_var_name);
    
    if let Some(var) = left_var {
        let left_var_col_names = &var.col_names;
        // 使用列名进行优化
    }
    
    // ...
}
```

### 完全多余的功能

#### 1. SymbolType 枚举中的大部分变体

**多余项**：
- `SymbolType::Alias`
- `SymbolType::Parameter`
- `SymbolType::Function`
- `SymbolType::Vertex`
- `SymbolType::Edge`
- `SymbolType::Path`

**原因**：
- nebula-graph 中没有 SymbolType 枚举
- 变量类型通过 `Value::Type` 表示（DATASET, BOOL, INT, STRING 等）
- 当前实现过度设计了符号类型系统

**建议**：
- 移除 `SymbolType` 枚举
- 使用 `Value::Type` 表示变量类型
- 保留 `Variable` 和 `Dataset` 作为变量类型（通过 `Value::Type` 区分）

#### 2. Symbol 结构中的多余字段

**多余项**：
- `symbol_type: SymbolType` - 过度设计
- `user_count: Arc<AtomicU64>` - 未实际使用
- `created_at: SystemTime` - nebula-graph 中没有

**建议**：
- 移除 `symbol_type` 字段
- 移除 `user_count` 字段（nebula-graph 中存在但未被实际使用）
- 移除 `created_at` 字段

#### 3. SymbolTable 中的多余方法

**多余项**：
- `get_readers()` - nebula-graph 中没有，仅测试使用
- `get_writers()` - nebula-graph 中没有，仅测试使用
- `get_variables_read_by()` - nebula-graph 中没有，未使用
- `get_variables_written_by()` - nebula-graph 中没有，未使用
- `detect_write_conflicts()` - nebula-graph 中没有，仅测试使用
- `rename_variable()` - nebula-graph 中没有，仅测试使用
- `to_string()` - nebula-graph 中有但仅用于调试

**建议**：
- 移除 `get_readers()` 方法
- 移除 `get_writers()` 方法
- 移除 `get_variables_read_by()` 方法
- 移除 `get_variables_written_by()` 方法
- 移除 `detect_write_conflicts()` 方法
- 移除 `rename_variable()` 方法
- 保留 `to_string()` 方法（用于调试）

#### 4. Symbol 中的多余方法

**多余项**：
- `increment_user_count()` - 未实际使用
- `get_user_count()` - 未实际使用

**建议**：
- 移除 `increment_user_count()` 方法
- 移除 `get_user_count()` 方法

## 重构建议

### 短期优化（立即可执行）

1. **简化 SymbolType 枚举**
   - 移除未使用的变体（Alias, Parameter, Function, Vertex, Edge, Path）
   - 仅保留 `Variable` 和 `Dataset`

2. **移除多余字段**
   - 移除 `Symbol::created_at` 字段
   - 移除 `Symbol::user_count` 字段

3. **移除多余方法**
   - 移除 `get_readers()`, `get_writers()`, `get_variables_read_by()`, `get_variables_written_by()`
   - 移除 `detect_write_conflicts()`, `rename_variable()`
   - 移除 `increment_user_count()`, `get_user_count()`

### 中期优化（需要集成）

1. **在 PlanNode 中集成符号表**
   - 在 PlanNode 构造时自动创建输出变量
   - 在 `set_input_var` 方法中调用 `read_by`
   - 在 `set_output_var` 方法中调用 `update_written_by`

2. **在优化器中使用符号表**
   - 在优化规则中使用 `getVar()` 获取变量信息
   - 使用 `readBy` 验证数据流依赖关系

### 长期优化（架构调整）

1. **移除 SymbolType 枚举**
   - 使用 `Value::Type` 表示变量类型
   - 与 nebula-graph 保持一致

2. **简化 Symbol 结构**
   - 仅保留必要的字段（name, type, col_names, readers, writers）
   - 移除所有调试和统计字段

3. **统一符号表接口**
   - 与 nebula-graph 的接口保持一致
   - 简化方法签名和返回类型

## 总结

### 当前问题

1. **过度设计**：SymbolType 枚举包含 8 种类型，但实际只使用 2 种
2. **功能未落地**：依赖关系跟踪功能已实现但未在实际代码中使用
3. **架构不匹配**：符号表与计划节点系统未集成
4. **冗余代码**：大量方法和字段未被实际使用

### 关键差异

| 特性 | nebula-graph | Rust 实现 | 差异 |
|------|-------------|-----------|------|
| SymbolType | 无 | 8 种变体 | 过度设计 |
| 变量类型 | Value::Type | SymbolType | 不一致 |
| PlanNode 集成 | 自动注册 | 未集成 | 未实现 |
| 优化器使用 | 数据流验证 | 未使用 | 未实现 |
| 冲突检测 | 无 | 有 | 多余 |

### 优先级

**高优先级**（立即执行）：
- 移除未使用的 SymbolType 变体
- 移除多余的字段和方法
- 简化 Symbol 结构

**中优先级**（近期执行）：
- 在 PlanNode 中集成符号表
- 在优化器中使用符号表信息

**低优先级**（长期执行）：
- 移除 SymbolType 枚举
- 统一符号表接口

## 参考资料

- nebula-graph 符号表实现：`nebula-3.8.0/src/graph/context/Symbols.h`
- nebula-graph PlanNode 实现：`nebula-3.8.0/src/graph/planner/plan/PlanNode.cpp`
- nebula-graph 优化器实现：`nebula-3.8.0/src/graph/optimizer/OptRule.cpp`
- 当前 Rust 符号表实现：`src/core/symbol/symbol_table.rs`
