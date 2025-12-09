# QueryContext 对比分析与实现方案

## 1. 整体架构对比

### Nebula-Graph C++实现
```cpp
// 从RequestContext到ExecutionContext的完整链路
QueryContext
  ├── RequestContext<ExecutionResponse>  // 请求级上下文
  ├── ValidateContext                     // 验证阶段上下文
  ├── ExecutionContext                    // 执行阶段上下文
  ├── SymbolTable                         // 符号表（变量跟踪）
  ├── ExecutionPlan                       // 执行计划
  └── 管理器与客户端
      ├── SchemaManager
      ├── IndexManager
      ├── StorageClient
      └── MetaClient
```

### 当前Rust实现
```rust
// 已基本实现核心结构，但缺少细节
QueryContext
  ├── RequestContext（占位符）
  ├── ValidateContext                     // ✓ 已实现
  ├── ExecutionContext                    // ✓ 已实现（简化版）
  ├── SymbolTable                         // ✓ 已实现
  ├── ObjectPool<String>                  // ⚠️ 使用String占位符
  └── 管理器与客户端（占位符）
```

## 2. 关键差异分析

### 2.1 ExecutionContext - 版本控制机制

**Nebula-Graph 实现**
```cpp
// 关键特性：支持多版本历史记录
struct ExecutionContext {
  // 版本常量定义
  static constexpr int64_t kLatestVersion = 0;      // 最新版本
  static constexpr int64_t kOldestVersion = 1;      // 最旧版本
  static constexpr int64_t kPreviousOneVersion = -1;  // 前一版本

  // 多版本存储：string -> vector<Result>
  std::unordered_map<std::string, std::vector<Result>> valueMap_;
  
  // 关键方法
  const Result& getVersionedResult(const std::string& name, int64_t version);
  void truncHistory(const std::string& name, size_t numVersionsToKeep);
  const std::vector<Result>& getHistory(const std::string& name);
  
  // 线程安全：使用RWSpinLock
  mutable folly::RWSpinLock lock_;
};
```

**当前Rust实现缺陷**
- ✗ 没有多版本历史记录
- ✗ 没有版本控制常量
- ✗ 没有历史截断机制
- ⚠️ 线程安全机制不完整

### 2.2 SymbolTable - 变量生命周期管理

**Nebula-Graph 实现**
```cpp
struct Variable {
  std::string name;
  Value::Type type;
  std::vector<std::string> colNames;
  
  // 关键：追踪变量的读写依赖
  std::unordered_set<PlanNode*> readBy;      // 哪些节点读取该变量
  std::unordered_set<PlanNode*> writtenBy;   // 哪些节点写入该变量
  
  std::atomic<uint64_t> userCount{0};        // 使用计数
};

class SymbolTable {
  // 变量生命周期管理方法
  bool readBy(const std::string& varName, PlanNode* node);
  bool writtenBy(const std::string& varName, PlanNode* node);
  bool deleteReadBy(const std::string& varName, PlanNode* node);
  bool updateReadBy(const std::string& oldVar, const std::string& newVar, PlanNode* node);
  
  // 线程安全
  mutable folly::RWSpinLock lock_;
};
```

**当前Rust实现缺陷**
- ✗ 没有变量读写依赖追踪
- ✗ 没有使用计数机制
- ✗ 没有变量生命周期管理方法
- ✗ 实现过于简化

### 2.3 ValidateContext - 空间与模式管理

**Nebula-Graph 实现**
```cpp
class ValidateContext {
  // 空间管理
  std::vector<SpaceInfo> spaces_;          // 空间栈（支持嵌套）
  
  // 变量和列定义
  std::unordered_map<std::string, ColsDef> vars_;  // 变量 -> 列定义
  
  // 生成器
  std::unique_ptr<AnonVarGenerator> anonVarGen_;
  std::unique_ptr<AnonColGenerator> anonColGen_;
  
  // Schema和索引跟踪
  std::unordered_map<std::string, std::shared_ptr<const meta::NebulaSchemaProvider>> schemas_;
  std::unordered_set<std::string> indexes_;
  
  // 方法
  void switchToSpace(SpaceInfo space);
  const ColsDef& getVar(const std::string& var);
  void registerVariable(std::string var, ColsDef cols);
  void addSchema(const std::string& name, const std::shared_ptr<...>& schema);
};
```

**当前Rust实现缺陷**
- ✗ 没有完整的列定义类型
- ✗ 没有匿名列生成器
- ✗ Schema存储机制不完善

### 2.4 QueryContext 初始化流程

**Nebula-Graph 实现**
```cpp
QueryContext::QueryContext(RequestContextPtr rctx,
                           meta::SchemaManager* sm,
                           meta::IndexManager* im,
                           storage::StorageClient* storage,
                           meta::MetaClient* metaClient,
                           CharsetInfo* charsetInfo)
    : rctx_(std::move(rctx)),
      sm_(DCHECK_NOTNULL(sm)),
      im_(DCHECK_NOTNULL(im)),
      storageClient_(DCHECK_NOTNULL(storage)),
      metaClient_(DCHECK_NOTNULL(metaClient)),
      charsetInfo_(DCHECK_NOTNULL(charsetInfo)) {
  init();
}

void QueryContext::init() {
  objPool_ = std::make_unique<ObjectPool>();
  ep_ = std::make_unique<ExecutionPlan>();
  ectx_ = std::make_unique<ExecutionContext>();
  
  // 关键：参数复制到ExecutionContext
  if (rctx_) {
    for (auto item : rctx_->parameterMap()) {
      ectx_->setValue(std::move(item.first), std::move(item.second));
    }
  }
  
  idGen_ = std::make_unique<IdGenerator>(0);
  
  // 关键：SymbolTable和ValidateContext的关联初始化
  symTable_ = std::make_unique<SymbolTable>(objPool_.get(), ectx_.get());
  vctx_ = std::make_unique<ValidateContext>(
    std::make_unique<AnonVarGenerator>(symTable_.get())
  );
}
```

**当前Rust实现缺陷**
- ✗ 没有完整的初始化流程
- ✗ 没有参数复制机制
- ✗ 组件间关联不足

## 3. 正式实现方案

### 3.1 第一步：完善 ExecutionContext

```rust
//! 执行上下文模块 - 管理查询执行期间的变量值和版本历史

use std::collections::HashMap;
use std::sync::RwLock;

/// 执行结果 - 包装 Value 和其他元数据
#[derive(Debug, Clone)]
pub struct Result {
    value: Value,
    // 后续可扩展：Iterator类型，执行统计信息等
}

impl Result {
    pub fn new(value: Value) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn move_value(self) -> Value {
        self.value
    }

    pub fn empty() -> Self {
        Self {
            value: Value::Empty,
        }
    }
}

/// 执行上下文 - 管理变量的多版本历史
pub struct ExecutionContext {
    // 版本控制常量
    // 0 是最新版本，-1 是前一个，以此类推
    // 1 是最旧版本，2 是次旧，以此类推

    // 变量名 -> [版本历史]（前面是最新，后面是最旧）
    value_map: RwLock<HashMap<String, Vec<Result>>>,
}

impl ExecutionContext {
    pub const LATEST_VERSION: i64 = 0;
    pub const OLDEST_VERSION: i64 = 1;
    pub const PREVIOUS_ONE_VERSION: i64 = -1;

    pub fn new() -> Self {
        Self {
            value_map: RwLock::new(HashMap::new()),
        }
    }

    /// 初始化变量
    pub fn init_var(&self, name: &str) {
        let mut map = self.value_map.write().unwrap();
        map.entry(name.to_string()).or_insert_with(Vec::new);
    }

    /// 设置值（自动创建新版本）
    pub fn set_value(&self, name: &str, value: Value) -> Result<()> {
        let mut builder = ResultBuilder::new();
        builder.value(value);
        self.set_result(name, builder.build())
    }

    /// 设置结果（自动创建新版本）
    pub fn set_result(&self, name: &str, result: Result) -> Result<()> {
        let mut map = self.value_map.write().unwrap();
        let hist = map.entry(name.to_string()).or_insert_with(Vec::new);
        hist.push(result);
        Ok(())
    }

    /// 获取最新版本的值
    pub fn get_value(&self, name: &str) -> Result<Value> {
        self.get_result(name).map(|r| r.value().clone())
    }

    /// 获取最新版本的结果
    pub fn get_result(&self, name: &str) -> Result<Result> {
        let map = self.value_map.read().unwrap();
        Ok(map
            .get(name)
            .and_then(|hist| hist.last())
            .cloned()
            .unwrap_or_else(Result::empty))
    }

    /// 获取指定版本的结果
    /// version: 0 是最新，-1 是前一个，1 是最旧
    pub fn get_versioned_result(&self, name: &str, version: i64) -> Result<Result> {
        let hist = self.get_history(name)?;
        let size = hist.len();
        if size == 0 {
            return Ok(Result::empty());
        }

        let idx = if version >= 0 {
            // 正索引：1 是最旧，2 是次旧...
            size.saturating_sub(version as usize)
        } else {
            // 负索引：-1 是最新前一个，-2 是更前面...
            (size as i64 + version) as usize
        };

        if idx >= size {
            Ok(Result::empty())
        } else {
            Ok(hist[idx].clone())
        }
    }

    /// 获取变量的所有历史版本（前面是最新，后面是最旧）
    pub fn get_history(&self, name: &str) -> Result<Vec<Result>> {
        let map = self.value_map.read().unwrap();
        Ok(map
            .get(name)
            .cloned()
            .unwrap_or_default())
    }

    /// 获取变量版本数
    pub fn num_versions(&self, name: &str) -> Result<usize> {
        let map = self.value_map.read().unwrap();
        Ok(map.get(name).map(|h| h.len()).unwrap_or(0))
    }

    /// 只保留最近的N个版本
    pub fn trunc_history(&self, name: &str, num_versions_to_keep: usize) -> Result<()> {
        let mut map = self.value_map.write().unwrap();
        if let Some(hist) = map.get_mut(name) {
            if hist.len() > num_versions_to_keep {
                // 保留最新的N个版本（从后往前数）
                let start = hist.len() - num_versions_to_keep;
                hist.drain(0..start);
            }
        }
        Ok(())
    }

    /// 删除变量的结果
    pub fn drop_result(&self, name: &str) -> Result<()> {
        let mut map = self.value_map.write().unwrap();
        map.remove(name);
        Ok(())
    }

    /// 检查变量是否存在
    pub fn exist(&self, name: &str) -> bool {
        let map = self.value_map.read().unwrap();
        map.contains_key(name)
    }
}

impl Clone for ExecutionContext {
    fn clone(&self) -> Self {
        let map = self.value_map.read().unwrap();
        Self {
            value_map: RwLock::new(map.clone()),
        }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }

/// 结果构建器
pub struct ResultBuilder {
    value: Value,
}

impl ResultBuilder {
    pub fn new() -> Self {
        Self {
            value: Value::Empty,
        }
    }

    pub fn value(mut self, value: Value) -> Self {
        self.value = value;
        self
    }

    pub fn build(self) -> Result {
        Result::new(self.value)
    }
}

impl Default for ResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

### 3.2 第二步：完善 SymbolTable

```rust
//! 符号表模块 - 管理变量定义和生命周期

use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

/// 列定义
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColDef {
    pub name: String,
    pub col_type: ValueType,  // 对应nebula的Value::Type
}

pub type ColsDef = Vec<ColDef>;

/// 变量定义 - 追踪变量的元数据和依赖关系
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub var_type: ValueType,
    pub col_names: Vec<String>,  // 如果是DATASET类型，包含列名
    
    // 读写依赖追踪
    pub read_by: HashSet<String>,     // 读取该变量的计划节点ID
    pub written_by: HashSet<String>,  // 写入该变量的计划节点ID
    
    pub user_count: std::sync::atomic::AtomicU64,
}

impl Variable {
    pub fn new(name: String) -> Self {
        Self {
            name,
            var_type: ValueType::Dataset,
            col_names: Vec::new(),
            read_by: HashSet::new(),
            written_by: HashSet::new(),
            user_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn to_string(&self) -> String {
        format!("Variable(name={}, type={:?})", self.name, self.var_type)
    }
}

/// 符号表 - 管理所有变量的定义和生命周期
pub struct SymbolTable {
    // 变量名 -> 变量定义
    vars: RwLock<HashMap<String, Variable>>,
    
    // 对象池和执行上下文的引用（用于变量初始化）
    obj_pool: Option<Box<dyn std::any::Any>>,
    ectx: Option<Box<ExecutionContext>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            vars: RwLock::new(HashMap::new()),
            obj_pool: None,
            ectx: None,
        }
    }

    pub fn with_context(ectx: ExecutionContext) -> Self {
        Self {
            vars: RwLock::new(HashMap::new()),
            obj_pool: None,
            ectx: Some(Box::new(ectx)),
        }
    }

    /// 创建新变量
    pub fn new_variable(&self, name: &str) -> Result<()> {
        let var = Variable::new(name.to_string());
        
        // 如果有ExecutionContext，初始化变量
        if let Some(ectx) = &self.ectx {
            ectx.init_var(name);
        }

        let mut vars = self.vars.write().unwrap();
        vars.insert(name.to_string(), var);
        Ok(())
    }

    /// 检查变量是否存在
    pub fn has_variable(&self, name: &str) -> bool {
        let vars = self.vars.read().unwrap();
        vars.contains_key(name)
    }

    /// 获取变量定义
    pub fn get_var(&self, name: &str) -> Result<Variable> {
        let vars = self.vars.read().unwrap();
        Ok(vars
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Variable {} not found", name))?)
    }

    /// 标记变量被某个节点读取
    pub fn read_by(&self, var_name: &str, node_id: &str) -> Result<()> {
        let mut vars = self.vars.write().unwrap();
        if let Some(var) = vars.get_mut(var_name) {
            var.read_by.insert(node_id.to_string());
            Ok(())
        } else {
            Err(format!("Variable {} not found", var_name))
        }
    }

    /// 标记变量被某个节点写入
    pub fn written_by(&self, var_name: &str, node_id: &str) -> Result<()> {
        let mut vars = self.vars.write().unwrap();
        if let Some(var) = vars.get_mut(var_name) {
            var.written_by.insert(node_id.to_string());
            Ok(())
        } else {
            Err(format!("Variable {} not found", var_name))
        }
    }

    /// 删除读取依赖
    pub fn delete_read_by(&self, var_name: &str, node_id: &str) -> Result<()> {
        let mut vars = self.vars.write().unwrap();
        if let Some(var) = vars.get_mut(var_name) {
            var.read_by.remove(node_id);
            Ok(())
        } else {
            Err(format!("Variable {} not found", var_name))
        }
    }

    /// 删除写入依赖
    pub fn delete_written_by(&self, var_name: &str, node_id: &str) -> Result<()> {
        let mut vars = self.vars.write().unwrap();
        if let Some(var) = vars.get_mut(var_name) {
            var.written_by.remove(node_id);
            Ok(())
        } else {
            Err(format!("Variable {} not found", var_name))
        }
    }

    /// 更新变量的读取依赖（用于优化时变量重命名）
    pub fn update_read_by(
        &self,
        old_var: &str,
        new_var: &str,
        node_id: &str,
    ) -> Result<()> {
        self.delete_read_by(old_var, node_id)?;
        self.read_by(new_var, node_id)
    }

    /// 更新变量的写入依赖
    pub fn update_written_by(
        &self,
        old_var: &str,
        new_var: &str,
        node_id: &str,
    ) -> Result<()> {
        self.delete_written_by(old_var, node_id)?;
        self.written_by(new_var, node_id)
    }

    pub fn to_string(&self) -> String {
        let vars = self.vars.read().unwrap();
        let var_list: Vec<String> = vars
            .values()
            .map(|v| v.to_string())
            .collect();
        format!("SymbolTable: [{}]", var_list.join(", "))
    }
}

impl Clone for SymbolTable {
    fn clone(&self) -> Self {
        let vars = self.vars.read().unwrap();
        Self {
            vars: RwLock::new(vars.clone()),
            obj_pool: None,  // 不克隆指针
            ectx: self.ectx.as_ref().map(|e| Box::new((*e).clone())),
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
```

### 3.3 第三步：增强 ValidateContext

```rust
//! 验证上下文模块 - 管理查询验证阶段的元数据

use std::collections::{HashMap, HashSet};

/// 空间信息
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub name: String,
    pub id: u32,
    // 后续可扩展：分区信息、副本因子等
}

/// 增强的ValidateContext
pub struct ValidateContext {
    // 空间栈（支持嵌套切换）
    spaces: Vec<SpaceInfo>,
    
    // 变量及其列定义
    vars: HashMap<String, ColsDef>,
    
    // Schema存储
    schemas: HashMap<String, SchemaProvider>,
    
    // 索引追踪
    indexes: HashSet<String>,
    
    // 创建的空间
    create_spaces: HashSet<String>,
    
    // 生成器
    anon_var_gen: Box<dyn AnonVarGen>,
    anon_col_gen: Box<dyn AnonColGen>,
}

impl ValidateContext {
    pub fn new(var_gen: Box<dyn AnonVarGen>) -> Self {
        Self {
            spaces: Vec::new(),
            vars: HashMap::new(),
            schemas: HashMap::new(),
            indexes: HashSet::new(),
            create_spaces: HashSet::new(),
            anon_var_gen: var_gen,
            anon_col_gen: Box::new(DefaultAnonColGen),
        }
    }

    pub fn switch_to_space(&mut self, space: SpaceInfo) {
        self.spaces.push(space);
    }

    pub fn current_space(&self) -> Option<&SpaceInfo> {
        self.spaces.last()
    }

    pub fn get_var(&self, var: &str) -> Option<&ColsDef> {
        self.vars.get(var)
    }

    pub fn exist_var(&self, var: &str) -> bool {
        self.vars.contains_key(var)
    }

    pub fn register_variable(&mut self, var: String, cols: ColsDef) {
        self.vars.insert(var, cols);
    }

    pub fn add_schema(&mut self, name: String, schema: SchemaProvider) {
        self.schemas.insert(name, schema);
    }

    pub fn get_schema(&self, name: &str) -> Option<&SchemaProvider> {
        self.schemas.get(name)
    }

    pub fn add_index(&mut self, index_name: String) {
        self.indexes.insert(index_name);
    }

    pub fn has_index(&self, index_name: &str) -> bool {
        self.indexes.contains(index_name)
    }

    pub fn add_space(&mut self, space_name: String) {
        self.create_spaces.insert(space_name);
    }

    pub fn has_space(&self, space_name: &str) -> bool {
        self.create_spaces.contains(space_name)
    }
}
```

### 3.4 第四步：重构 QueryContext

```rust
//! 查询上下文模块（增强版）

/// 完整的QueryContext初始化
impl QueryContext {
    /// 新建查询上下文（完整构造）
    pub fn new_with_context(
        rctx: RequestContext,
        schema_manager: SchemaManager,
        index_manager: IndexManager,
        storage_client: StorageClient,
        meta_client: MetaClient,
        charset_info: CharsetInfo,
    ) -> Self {
        // 创建核心组件
        let ectx = ExecutionContext::new();
        let obj_pool = ObjectPool::new(1000);
        let id_gen = IdGenerator::new(0);
        
        // 创建符号表和验证上下文，建立它们之间的关联
        let sym_table = SymbolTable::with_context(ectx.clone());
        let anon_var_gen = AnonVarGenerator::new(sym_table.clone());
        let mut vctx = ValidateContext::new(Box::new(anon_var_gen));
        
        // 复制请求参数到执行上下文
        for (param_name, param_value) in rctx.parameters() {
            let _ = ectx.set_value(&param_name, param_value);
        }
        
        Self {
            rctx: Some(Box::new(rctx)),
            vctx,
            ectx,
            plan: None,
            schema_manager: Some(Box::new(schema_manager)),
            index_manager: Some(Box::new(index_manager)),
            storage_client: Some(Box::new(storage_client)),
            meta_client: Some(Box::new(meta_client)),
            charset_info: Some(Box::new(charset_info)),
            obj_pool,
            id_gen,
            sym_table,
            killed: AtomicBool::new(false),
        }
    }

    /// 初始化变量（在语义分析阶段调用）
    pub fn init_variable(&mut self, var_name: &str) -> Result<()> {
        self.sym_table.new_variable(var_name)?;
        Ok(())
    }

    /// 设置变量的列定义（在语义分析阶段）
    pub fn set_variable_cols(&mut self, var_name: &str, cols: ColsDef) -> Result<()> {
        self.vctx.register_variable(var_name.to_string(), cols);
        Ok(())
    }

    /// 跟踪变量的读写依赖
    pub fn add_read_dependency(&mut self, var_name: &str, plan_node_id: &str) -> Result<()> {
        self.sym_table.read_by(var_name, plan_node_id)
    }

    pub fn add_write_dependency(&mut self, var_name: &str, plan_node_id: &str) -> Result<()> {
        self.sym_table.written_by(var_name, plan_node_id)
    }

    /// 获取变量的完整定义（包括类型和列信息）
    pub fn get_variable_info(&self, var_name: &str) -> Result<(Variable, ColsDef)> {
        let var = self.sym_table.get_var(var_name)?;
        let cols = self.vctx.get_var(var_name).cloned().unwrap_or_default();
        Ok((var, cols))
    }
}
```

## 4. 实现优先级

### 优先级1（核心功能）
- [ ] 完善ExecutionContext版本控制
- [ ] 实现SymbolTable变量生命周期管理
- [ ] 完整化QueryContext初始化流程

### 优先级2（关键支持）
- [ ] 实现真实的SchemaProvider
- [ ] 实现变量读写依赖追踪的持久化
- [ ] 添加线程安全测试

### 优先级3（优化）
- [ ] 版本历史自动清理
- [ ] 性能优化（内存池、缓存）
- [ ] 详细的诊断和监控

## 5. 关键测试场景

```rust
#[test]
fn test_execution_context_versioning() {
    let ctx = ExecutionContext::new();
    
    // 设置初始值
    ctx.set_value("x", Value::Int(1)).unwrap();
    assert_eq!(ctx.get_value("x").unwrap(), Value::Int(1));
    
    // 更新值（创建新版本）
    ctx.set_value("x", Value::Int(2)).unwrap();
    assert_eq!(ctx.get_value("x").unwrap(), Value::Int(2));
    
    // 获取版本历史
    let hist = ctx.get_history("x").unwrap();
    assert_eq!(hist.len(), 2);
    
    // 获取前一版本
    let prev = ctx.get_versioned_result("x", -1).unwrap();
    assert_eq!(prev.value(), &Value::Int(1));
}

#[test]
fn test_symbol_table_dependencies() {
    let st = SymbolTable::new();
    
    st.new_variable("result").unwrap();
    st.read_by("result", "node_1").unwrap();
    st.written_by("result", "node_0").unwrap();
    
    let var = st.get_var("result").unwrap();
    assert!(var.read_by.contains("node_1"));
    assert!(var.written_by.contains("node_0"));
}
```

## 6. 迁移路线图

1. **阶段1**：实现ExecutionContext多版本支持 (1-2天)
2. **阶段2**：增强SymbolTable依赖追踪 (1-2天)
3. **阶段3**：完善QueryContext初始化流程 (1-2天)
4. **阶段4**：集成测试和文档 (1天)
5. **阶段5**：性能优化和监控 (1-2天)
