# Context 模块功能迁移分析 - 完整版

## 1. Context 模块整体组件图

```
QueryContext (查询上下文) - 顶级上下文
│
├─ RequestContext (请求上下文)
│  ├─ 请求参数映射
│  └─ 响应对象
│
├─ ExecutionContext (执行上下文)
│  ├─ 变量值存储 (多版本历史)
│  ├─ Result 对象 (包装 Value + Iterator)
│  └─ 版本控制机制
│
├─ ValidateContext (验证上下文)
│  ├─ 空间管理
│  ├─ 变量和列定义
│  ├─ Schema 管理
│  └─ 索引追踪
│
├─ SymbolTable (符号表)
│  ├─ 变量定义 (Variable)
│  ├─ 读写依赖关系
│  └─ 使用计数
│
├─ QueryExpressionContext (表达式求值上下文) ⭐
│  ├─ 变量访问
│  ├─ 属性访问 (标签、边、顶点)
│  ├─ 列访问
│  └─ 内部变量管理
│
├─ Iterator (数据迭代器) ⭐⭐
│  ├─ DefaultIter (默认迭代器)
│  ├─ SequentialIter (顺序迭代器)
│  ├─ GetNeighborsIter (邻居迭代器)
│  ├─ PropIter (属性迭代器)
│  └─ GetNbrsRespDataSetIter (响应迭代器)
│
└─ Result 构建与管理
   ├─ Result (结果包装对象)
   ├─ ResultBuilder (构建器模式)
   └─ 状态管理 (Success/PartialSuccess/Unexecuted)
```

## 2. 当前Rust实现缺失功能详细清单

### 2.1 ❌ 完全缺失：QueryExpressionContext

**Nebula实现特点**：
```cpp
class QueryExpressionContext final : public ExpressionContext {
  // 关键职责：为表达式求值提供运行时上下文
  
  // 1. 变量访问
  const Value& getVar(const std::string& var) const;           // 获取变量值
  const Value& getVersionedVar(const std::string& var, int64_t version) const;
  void setVar(const std::string&, Value val);                 // 设置变量值
  
  // 2. 内部变量管理（用于列表解析等）
  void setInnerVar(const std::string& var, Value val);
  const Value& getInnerVar(const std::string& var) const;
  std::unordered_map<std::string, Value> exprValueMap_;       // 存储内部变量
  
  // 3. 属性访问（从迭代器获取数据）
  const Value& getVarProp(const std::string& var, const std::string& prop) const;
  Value getTagProp(const std::string& tag, const std::string& prop) const;   // tag.prop
  Value getEdgeProp(const std::string& edge, const std::string& prop) const; // edge.prop
  Value getSrcProp(const std::string& tag, const std::string& prop) const;   // $^.prop
  const Value& getDstProp(const std::string& tag, const std::string& prop) const; // $$.prop
  const Value& getInputProp(const std::string& prop) const;    // $-.prop
  
  // 4. 列访问
  const Value& getColumn(int32_t index) const;
  StatusOr<std::size_t> getColumnIndex(const std::string& prop) const;
  StatusOr<std::size_t> getInputPropIndex(const std::string& prop) const;
  
  // 5. 对象获取
  Value getVertex(const std::string& name = "") const;
  Value getEdge() const;
  
  // 6. Iterator上下文设置
  QueryExpressionContext& operator()(Iterator* iter = nullptr) {
    iter_ = iter;
    return *this;
  }
  
  private:
    ExecutionContext* ectx_{nullptr};      // 执行上下文指针
    Iterator* iter_{nullptr};              // 当前行迭代器
    std::unordered_map<std::string, Value> exprValueMap_;
};
```

**使用场景**：
- Filter/Where 表达式求值
- Select 表达式求值
- Return 表达式求值
- 函数参数求值
- 所有需要访问变量、列、属性的地方

**优先级**：🔴 P1（核心功能，许多执行器依赖）

### 2.2 ❌ 完全缺失：Iterator 类体系

**Nebula实现特点**：
```cpp
class Iterator {
  enum class Kind : uint8_t {
    kDefault,        // 默认常量迭代器
    kGetNeighbors,   // 邻居迭代器
    kSequential,     // 顺序迭代器
    kProp,           // 属性迭代器
  };
  
  // 1. 基本迭代操作
  virtual bool valid() const;              // 检查是否有效
  virtual void next() = 0;                 // 进入下一行
  virtual void erase() = 0;                // 删除当前行
  virtual void unstableErase() = 0;        // 快速删除（破坏顺序）
  virtual void clear() = 0;                // 清空所有行
  virtual void reset(size_t pos = 0);      // 重置位置
  
  // 2. 范围操作
  virtual void select(std::size_t offset, std::size_t count) = 0;  // 选择范围
  virtual void sample(int64_t count) = 0;                          // 采样
  virtual void eraseRange(size_t first, size_t last) = 0;          // 删除范围
  
  // 3. 行访问
  virtual const Row* row() const = 0;
  virtual Row moveRow() = 0;
  virtual size_t size() const = 0;
  bool empty() const { return size() == 0; }
  
  // 4. 列访问（用于表达式求值）
  virtual const Value& getColumn(const std::string& col) const = 0;
  virtual const Value& getColumn(int32_t index) const = 0;
  virtual StatusOr<std::size_t> getColumnIndex(const std::string& col) const = 0;
  
  // 5. 属性访问（图特定）
  virtual const Value& getTagProp(const std::string&, const std::string&) const;
  virtual const Value& getEdgeProp(const std::string&, const std::string&) const;
  virtual Value getVertex(const std::string& name = "");
  virtual Value getEdge() const;
  
  // 6. 工具方法
  virtual std::unique_ptr<Iterator> copy() const = 0;   // 深拷贝
  virtual void setCheckMemory(bool checkMemory);          // 内存检查
  
  // 7. 类型检查
  bool isDefaultIter() const;
  bool isGetNeighborsIter() const;
  bool isSequentialIter() const;
  bool isPropIter() const;
};
```

**具体子类实现**：

#### 2.2.1 DefaultIter
```cpp
class DefaultIter : public Iterator {
  // 用于单个常量值
  // 始终有一个有效行，代表这个值本身
  
  explicit DefaultIter(std::shared_ptr<Value> value, bool checkMemory = false);
  
  virtual bool valid() const override;
  virtual void next() override;           // 移动后无效
  virtual void erase() override;
  virtual void unstableErase() override;
  virtual const Row* row() const override;
  virtual Row moveRow() override;
  virtual size_t size() const override;   // 总是返回 1
};
```

#### 2.2.2 SequentialIter
```cpp
class SequentialIter : public Iterator {
  // 用于 DataSet 的顺序迭代
  // 支持行级操作（添加、删除、修改）
  
  explicit SequentialIter(std::shared_ptr<Value> value, bool checkMemory = false);
  
  virtual void next() override;                           // 指向下一行
  virtual void erase() override;                          // 有序删除
  virtual void unstableErase() override;                  // 快速删除
  virtual void select(std::size_t offset, std::size_t count) override;
  virtual void eraseRange(size_t first, size_t last) override;
  
  virtual const Row* row() const override;                // 当前行
  virtual Row moveRow() override;                         // 移动当前行
  virtual size_t size() const override;                   // DataSet行数
  
  virtual const Value& getColumn(const std::string& col) const override;
  virtual const Value& getColumn(int32_t index) const override;
  
  private:
    std::vector<Row>& rows_;                              // DataSet行列表
    std::vector<Row>::iterator currRow_;
};
```

#### 2.2.3 GetNeighborsIter
```cpp
class GetNeighborsIter : public Iterator {
  // 用于处理邻居查询结果（复杂的树型结构）
  // GetNeighbors 返回树状结构：多个顶点，每个顶点有多条边和邻接顶点
  
  explicit GetNeighborsIter(std::shared_ptr<Value> value, bool checkMemory = false);
  
  // 特殊方法
  virtual Value getVertex(const std::string& name = "") override;  // 获取当前顶点
  virtual const Value& getTagProp(const std::string& tag, const std::string& prop) const override;
  virtual const Value& getEdgeProp(const std::string& edge, const std::string& prop) const override;
  
  // 复杂的状态管理
  // 需要管理：srcVertex -> (edge, dstVertex) 的多层次结构
  
  private:
    std::vector<Vertex> vertices_;       // 遍历的顶点列表
    std::vector<Vertex>::iterator currVertex_;
    
    std::vector<Edge> edges_;            // 当前顶点的边列表
    std::vector<Edge>::iterator currEdge_;
    
    std::vector<Vertex> neighbors_;      // 邻接顶点列表
    std::vector<Vertex>::iterator currNeighbor_;
};
```

#### 2.2.4 PropIter
```cpp
class PropIter : public Iterator {
  // 用于处理顶点/边属性查询
  // 类似 SequentialIter，但针对属性数据优化
  
  explicit PropIter(std::shared_ptr<Value> value, bool checkMemory = false);
  
  // 支持顶点和边属性访问
  virtual const Value& getTagProp(const std::string& tag, const std::string& prop) const override;
  virtual const Value& getEdgeProp(const std::string& edge, const std::string& prop) const override;
};
```

**优先级**：🔴 P1（执行引擎基础，影响所有数据处理）

### 2.3 ❌ 不完整：Result 和 ResultBuilder

**当前Rust实现**：
```rust
#[derive(Debug, Clone)]
pub struct ExecutionResponse;  // 占位符
```

**Nebula实现**：
```cpp
class Result final {
  enum class State : uint8_t {
    kUnExecuted,        // 未执行
    kPartialSuccess,    // 部分成功（有警告/错误但继续）
    kSuccess,           // 成功
  };
  
  // 1. 值访问
  std::shared_ptr<Value> valuePtr() const;
  const Value& value() const;
  Value&& moveValue();
  
  // 2. 状态管理
  State state() const;
  
  // 3. 大小信息
  size_t size() const;
  
  // 4. 迭代器管理
  std::unique_ptr<Iterator> iter() const&;     // 拷贝迭代器
  std::unique_ptr<Iterator> iter() &&;         // 移动迭代器
  Iterator* iterRef() const;                   // 获取引用
  
  // 5. 列名称
  std::vector<std::string> getColNames() const;
  
  // 6. 内存管理
  void checkMemory(bool checkMemory);
  
  private:
    struct Core {
      bool checkMemory{false};
      State state;
      std::string msg;
      std::shared_ptr<Value> value;
      std::unique_ptr<Iterator> iter;
    };
};

class ResultBuilder final {
  Result build();
  
  ResultBuilder& checkMemory(bool checkMemory = false);
  ResultBuilder& value(Value&& value);
  ResultBuilder& value(std::shared_ptr<Value> value);
  ResultBuilder& iter(std::unique_ptr<Iterator> iter);
  ResultBuilder& iter(Iterator::Kind kind);          // 自动创建迭代器
  ResultBuilder& state(Result::State state);
  ResultBuilder& msg(std::string&& msg);
};
```

**优先级**：🔴 P1（执行上下文依赖）

### 2.4 ⚠️ 不完整：ExecutionContext 的完整功能

**缺失的关键方法**：

```cpp
// 1. 内存管理和检查
void checkMemory(const std::string& name, bool checkMemory);

// 2. 垃圾回收集成
void dropResult(const std::string& name);  // 支持异步GC

// 3. 高级版本管理
const Result& getVersionedResult(const std::string& name, int64_t version) const;
void setVersionedResult(const std::string& name, Result&& result, int64_t version);
const std::vector<Result>& getHistory(const std::string& name) const;

// 4. 迭代器管理
// Result 包含 Iterator，需要支持迭代器的完整生命周期管理
```

**线程安全**：
- Nebula: `folly::RWSpinLock` (高性能自旋锁)
- 当前Rust: `std::sync::RwLock` (互斥锁)

**优先级**：🟡 P2（已有基础，需要增强）

### 2.5 ⚠️ 缺失：ValidateContext 的高级功能

```cpp
// 缺失的功能：
1. 空间栈管理（支持嵌套空间切换）
   std::vector<SpaceInfo> spaces_;
   
2. 完整的 Schema 管理
   std::unordered_map<std::string, std::shared_ptr<const meta::NebulaSchemaProvider>> schemas_;
   
3. 索引追踪
   std::unordered_set<std::string> indexes_;
   
4. 列定义
   std::unordered_map<std::string, ColsDef> vars_;
   
5. 生成器集成
   std::unique_ptr<AnonVarGenerator> anonVarGen_;
   std::unique_ptr<AnonColGenerator> anonColGen_;
   
6. 创建空间追踪
   std::unordered_set<std::string> createSpaces_;
```

**优先级**：🟡 P2（语义分析依赖）

### 2.6 ⚠️ 缺失：SymbolTable 的完整实现

**当前问题**：
- 没有实现完整的变量读写依赖关系管理
- 没有计划节点参与 (PlanNode)
- 没有对象池集成

**缺失方法**：
```rust
// 变量生命周期管理
pub fn delete_read_by(&self, var_name: &str, node_id: &str) -> Result<()>;
pub fn delete_written_by(&self, var_name: &str, node_id: &str) -> Result<()>;
pub fn update_read_by(&self, old_var: &str, new_var: &str, node_id: &str) -> Result<()>;
pub fn update_written_by(&self, old_var: &str, new_var: &str, node_id: &str) -> Result<()>;

// 对象池集成
// Variable 应该从对象池分配，而不是堆上
```

**优先级**：🟡 P2（优化需要）

### 2.7 ⚠️ 缺失：RequestContext 的真实实现

```cpp
// 参数映射
std::unordered_map<std::string, Value> parameterMap() const {
  // 需要支持从请求中提取参数
}

// 响应对象
ExecutionResponse& resp() {
  // 需要支持状态码、错误消息等
}
```

**优先级**：🟡 P2（需求依赖）

## 3. 功能优先级与实现路线

### Phase 1: 核心迭代功能 (1-2 周)

```
[ ] 1. Result 和 ResultBuilder 完整实现
    ├─ State 枚举 (Success/PartialSuccess/Unexecuted)
    ├─ Message 字段
    ├─ Iterator 集成
    └─ 测试

[ ] 2. Iterator 基类实现
    ├─ 基本迭代接口
    ├─ 列访问接口
    ├─ 类型判断方法
    └─ 内存检查机制

[ ] 3. DefaultIter 实现
    ├─ 单值迭代
    ├─ 有效性检查
    └─ 行访问

[ ] 4. SequentialIter 实现
    ├─ DataSet 迭代
    ├─ 行级操作 (add/delete/modify)
    ├─ 范围操作 (select/erase)
    └─ 性能优化 (unstable erase)
```

### Phase 2: 表达式求值上下文 (1 周)

```
[ ] 1. QueryExpressionContext 核心
    ├─ ExecutionContext 集成
    ├─ Iterator 集成
    ├─ 变量访问接口
    └─ 内部变量管理

[ ] 2. 属性访问接口
    ├─ VarProp ($a.prop)
    ├─ TagProp (tag.prop)
    ├─ EdgeProp (edge.prop)
    ├─ SrcProp ($^.prop)
    ├─ DstProp ($$.prop)
    ├─ InputProp ($-.prop)
    └─ 列索引查询

[ ] 3. 对象获取接口
    ├─ getVertex()
    ├─ getEdge()
    └─ getColumn()
```

### Phase 3: 复杂迭代器 (1.5 周)

```
[ ] 1. GetNeighborsIter 实现
    ├─ 树状结构管理
    ├─ 顶点遍历
    ├─ 边遍历
    ├─ 邻接顶点遍历
    ├─ 属性访问
    └─ 内存优化

[ ] 2. PropIter 实现
    ├─ 属性查询优化
    ├─ 顶点属性访问
    ├─ 边属性访问
    └─ 性能优化

[ ] 3. GetNbrsRespDataSetIter 实现
    ├─ 响应处理
    └─ 批量操作
```

### Phase 4: 增强和优化 (1 周)

```
[ ] 1. ExecutionContext 增强
    ├─ 完整的 Result 支持
    ├─ 版本管理优化
    ├─ 内存检查
    └─ 异步 GC 集成

[ ] 2. ValidateContext 增强
    ├─ 空间栈管理
    ├─ Schema 支持
    ├─ 索引追踪
    └─ 生成器集成

[ ] 3. SymbolTable 增强
    ├─ 计划节点集成
    ├─ 对象池支持
    └─ 依赖关系优化
```

## 4. 关键实现细节

### 4.1 Iterator 内存管理

```rust
// Iterator 的生命周期与 Result 绑定
pub struct Result {
    value: Arc<Value>,              // 共享所有权
    state: ResultState,
    msg: String,
    iter: Box<dyn Iterator>,       // 拥有所有权
    check_memory: bool,
}

// 防止内存泄漏
impl Drop for Result {
    fn drop(&mut self) {
        // 清理可能关联的资源
    }
}
```

### 4.2 QueryExpressionContext 与多层上下文的关联

```rust
pub struct QueryExpressionContext {
    // 三层上下文关联
    ectx: Arc<ExecutionContext>,           // 变量值
    iter: Arc<Mutex<Option<Box<dyn Iterator>>>>,  // 当前行
    expr_value_map: RwLock<HashMap<String, Value>>, // 表达式变量
}

// 支持链式设置
impl QueryExpressionContext {
    pub fn with_iterator(mut self, iter: Box<dyn Iterator>) -> Self {
        self.iter = Arc::new(Mutex::new(Some(iter)));
        self
    }
}
```

### 4.3 Iterator 的复制策略

```rust
pub trait Iterator {
    // 深拷贝（用于保存状态）
    fn copy(&self) -> Box<dyn Iterator>;
    
    // 移动（用于性能优化）
    fn take(&mut self) -> Box<dyn Iterator>;
}

impl Clone for Box<dyn Iterator> {
    fn clone(&self) -> Self {
        self.copy()
    }
}
```

## 5. 测试覆盖

### 必须的测试用例

```rust
#[test]
fn test_result_builder_with_iterator() {
    let value = Value::DataSet(...);
    let iter = SequentialIter::new(Arc::new(value));
    
    let result = ResultBuilder::new()
        .value(Value::DataSet(...))
        .iter(Iterator::Kind::Sequential)
        .state(ResultState::Success)
        .build();
    
    assert_eq!(result.state(), ResultState::Success);
    assert_eq!(result.size(), 10);
}

#[test]
fn test_sequential_iter_operations() {
    let mut iter = SequentialIter::new(...);
    
    iter.next();
    let row = iter.row();
    
    iter.erase();
    assert!(iter.valid());
    
    iter.select(0, 5);
    assert_eq!(iter.size(), 5);
}

#[test]
fn test_query_expression_context_integration() {
    let ectx = ExecutionContext::new();
    ectx.set_value("x", Value::Int(42)).unwrap();
    
    let iter = DefaultIter::new(Arc::new(Value::Int(42)));
    
    let mut qectx = QueryExpressionContext::new(Arc::new(ectx));
    qectx = qectx.with_iterator(Box::new(iter));
    
    let val = qectx.get_var("x").unwrap();
    assert_eq!(val, Value::Int(42));
}

#[test]
fn test_property_access_through_iterator() {
    let vertex_data = create_vertex_dataset();
    let iter = PropIter::new(Arc::new(vertex_data));
    
    let prop_val = iter.get_tag_prop("person", "name").unwrap();
    assert_eq!(prop_val, Value::String("Alice".to_string()));
}
```

## 6. 外部依赖关系

### Context 模块被以下模块依赖
```
Parser ──┐
         ├──> Context
Planner ─┤
         ├──> SymbolTable
         │    ValidateContext
Optimizer┤
         ├──> SymbolTable
         │    ExecutionContext
Executor ─────> QueryExpressionContext
                Iterator
                Result
```

### Context 模块的依赖

```
Core (Value, DataSet)
  ↑
  ├─ Result
  ├─ Iterator
  └─ ExecutionContext
  
Storage (SchemaManager, IndexManager)
  ↑
  ├─ ValidateContext
  └─ QueryContext
```

## 7. 实现检查清单

### 第一阶段（P1）必需功能

- [ ] Result 和 State 枚举
- [ ] ResultBuilder 构建器
- [ ] Iterator 基类接口
- [ ] DefaultIter 实现
- [ ] SequentialIter 完整实现
- [ ] QueryExpressionContext 核心（变量和列访问）
- [ ] 完整的单元测试

### 第二阶段（P2）重要功能

- [ ] GetNeighborsIter 实现
- [ ] PropIter 实现
- [ ] QueryExpressionContext 属性访问
- [ ] ValidateContext 增强
- [ ] SymbolTable 增强
- [ ] 集成测试和性能基准

### 第三阶段（P3）优化[rust中的实现需要调整]

- [ ] 异步 GC 集成
- [ ] 内存检查机制
- [ ] 缓存优化
- [ ] 并发性能测试
