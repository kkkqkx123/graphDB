# 执行器工厂设计模式分析

## 当前设计分析

### 现有架构

```
ExecutorFactory
├── storage: Option<Arc<Mutex<S>>>
├── config: ExecutorSafetyConfig
├── recursion_detector: RecursionDetector
├── safety_validator: SafetyValidator<S>
└── builders: Builders<S>
    ├── data_access: DataAccessBuilder<S>
    ├── data_modification: DataModificationBuilder<S>
    ├── data_processing: DataProcessingBuilder<S>
    ├── join: JoinBuilder<S>
    ├── set_operation: SetOperationBuilder<S>
    ├── traversal: TraversalBuilder<S>
    ├── transformation: TransformationBuilder<S>
    ├── control_flow: ControlFlowBuilder<S>
    └── admin: AdminBuilder<S>
```

### 当前问题

1. **过长的 match 语句** - `create_executor` 方法有约 80 个分支
2. **三层嵌套调用** - `self.builders.data_access().build_scan_vertices(...)`
3. **重复代码模式** - 每个分支都是类似的调用结构
4. **违反开闭原则** - 添加新执行器需要修改工厂

## 替代设计模式分析

### 模式一：访问者模式 (Visitor Pattern)

#### 设计

```rust
/// 执行器创建访问者
trait ExecutorVisitor<S: StorageClient> {
    type Output;
    
    fn visit_scan_vertices(&mut self, node: &ScanVerticesNode) -> Self::Output;
    fn visit_scan_edges(&mut self, node: &ScanEdgesNode) -> Self::Output;
    fn visit_get_vertices(&mut self, node: &GetVerticesNode) -> Self::Output;
    // ... 其他 visit 方法
}

/// 计划节点接受访问者
trait PlanNodeVisitable {
    fn accept<S: StorageClient, V: ExecutorVisitor<S>>(
        &self, 
        visitor: &mut V
    ) -> V::Output;
}

/// 创建执行器的访问者实现
struct ExecutorCreator<S: StorageClient> {
    storage: Arc<Mutex<S>>,
    context: ExecutionContext,
    _phantom: PhantomData<S>,
}

impl<S: StorageClient> ExecutorVisitor<S> for ExecutorCreator<S> {
    type Output = Result<ExecutorEnum<S>, QueryError>;
    
    fn visit_scan_vertices(&mut self, node: &ScanVerticesNode) -> Self::Output {
        let executor = GetVerticesExecutor::new(
            node.id(),
            self.storage.clone(),
            None,
            None,
            node.vertex_filter().and_then(|f| f.get_expression()),
            node.limit().map(|l| l as usize),
            self.context.expression_context().clone(),
        );
        Ok(ExecutorEnum::GetVertices(executor))
    }
    
    // ... 其他实现
}
```

#### 优点
- 将创建逻辑分散到各个节点类型
- 易于添加新节点类型（只需实现 accept）
- 类型安全，编译时检查

#### 缺点
- 需要为每个节点类型实现 accept 方法
- 访问者接口可能变得庞大
- 破坏了计划节点的纯粹性

#### 适用性
**不推荐** - 过度设计，增加了不必要的复杂性。

---

### 模式二：类型级联模式 (Type Cascade / Type Registry)

#### 设计

```rust
/// 执行器创建特质
trait ExecutorFactoryFn<S: StorageClient> {
    type Node: PlanNode;
    
    fn create(
        node: &Self::Node,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError>;
}

/// 为每种执行器类型实现
struct ScanVerticesFactory;

impl<S: StorageClient> ExecutorFactoryFn<S> for ScanVerticesFactory {
    type Node = ScanVerticesNode;
    
    fn create(
        node: &Self::Node,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = GetVerticesExecutor::new(
            node.id(),
            storage,
            None,
            None,
            node.vertex_filter().and_then(|f| f.get_expression()),
            node.limit().map(|l| l as usize),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::GetVertices(executor))
    }
}

/// 宏辅助生成 match 分支
macro_rules! register_executor {
    ($registry:ident, $variant:ident, $factory:ty) => {
        $registry.register::<$factory>(
            stringify!($variant),
            |node, storage, context| {
                <$factory as ExecutorFactoryFn<_>>::create(
                    node.downcast_ref().expect("类型检查通过"),
                    storage,
                    context,
                )
            },
        );
    };
}
```

#### 优点
- 每个执行器类型独立定义创建逻辑
- 符合开闭原则
- 易于单元测试

#### 缺点
- 需要运行时类型转换（downcast）
- 宏增加了复杂性
- 失去了编译时 exhaustive check

#### 适用性
**谨慎考虑** - 运行时类型转换有风险。

---

### 模式三：组合模式 + 静态分发（推荐）

#### 设计

```rust
/// 执行器创建器特质 - 按类别组织
trait ExecutorCreator<S: StorageClient> {
    fn create(
        &self,
        node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Option<Result<ExecutorEnum<S>, QueryError>>;
}

/// 数据访问创建器
struct DataAccessCreator;

impl<S: StorageClient> ExecutorCreator<S> for DataAccessCreator {
    fn create(
        &self,
        node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Option<Result<ExecutorEnum<S>, QueryError>> {
        match node {
            PlanNodeEnum::ScanVertices(n) => Some(create_scan_vertices(n, storage, context)),
            PlanNodeEnum::ScanEdges(n) => Some(create_scan_edges(n, storage, context)),
            PlanNodeEnum::GetVertices(n) => Some(create_get_vertices(n, storage, context)),
            // ... 其他数据访问节点
            _ => None,
        }
    }
}

/// 组合创建器
struct CompositeExecutorCreator<S: StorageClient> {
    creators: Vec<Box<dyn ExecutorCreator<S>>>,
}

impl<S: StorageClient> CompositeExecutorCreator<S> {
    fn new() -> Self {
        Self {
            creators: vec![
                Box::new(DataAccessCreator),
                Box::new(DataModificationCreator),
                Box::new(DataProcessingCreator),
                // ... 其他创建器
            ],
        }
    }
    
    fn create(
        &self,
        node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        for creator in &self.creators {
            if let Some(result) = creator.create(node, storage.clone(), context) {
                return result;
            }
        }
        Err(QueryError::ExecutionError(
            format!("Unsupported plan node: {:?}", node)
        ))
    }
}
```

#### 优点
- 将庞大的 match 分解为多个小的 match
- 每个类别独立管理
- 易于扩展新类别
- 保持静态分发

#### 缺点
- 需要遍历创建器列表（性能影响极小）
- 失去了编译时 exhaustive check

#### 适用性
**推荐** - 平衡了可维护性和性能。

---

### 模式四：代码生成 + 静态注册（最佳实践）

#### 设计

```rust
/// 使用过程宏自动生成创建逻辑

// 定义执行器元数据
#[derive(ExecutorFactory)]
#[node_type = "ScanVertices"]
#[executor_type = "GetVerticesExecutor"]
struct ScanVerticesExecutorFactory;

// 宏自动生成：
// impl ExecutorFactoryFn for ScanVerticesExecutorFactory {
//     fn create(...) -> Result<ExecutorEnum<S>, QueryError> {
//         // 自动生成的创建逻辑
//     }
// }

/// 编译时注册的静态表
mod generated {
    use super::*;
    
    // 宏生成的静态 match
    pub fn create_executor<S: StorageClient>(
        node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        match node {
            // 自动生成的分支
            PlanNodeEnum::ScanVertices(n) => {
                ScanVerticesExecutorFactory::create(n, storage, context)
            }
            // ... 其他分支
        }
    }
}
```

#### 优点
- 自动生成重复代码
- 保持静态分发
- 编译时 exhaustive check
- 声明式定义

#### 缺点
- 需要编写过程宏（复杂性高）
- 调试困难

#### 适用性
**长期推荐** - 适合大型项目，但初期投入大。

---

### 模式五：函数指针表（简单实用）

#### 设计

```rust
/// 创建函数类型
 type CreateFn<S> = fn(
    &PlanNodeEnum,
    Arc<Mutex<S>>,
    &ExecutionContext,
) -> Option<Result<ExecutorEnum<S>, QueryError>>;

/// 静态创建函数表
static CREATE_FNS: &[CreateFn] = &[
    try_create_data_access,
    try_create_data_modification,
    try_create_data_processing,
    // ... 其他类别
];

/// 数据访问创建函数
fn try_create_data_access<S: StorageClient>(
    node: &PlanNodeEnum,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Option<Result<ExecutorEnum<S>, QueryError>> {
    match node {
        PlanNodeEnum::ScanVertices(n) => Some(create_scan_vertices(n, storage, context)),
        PlanNodeEnum::ScanEdges(n) => Some(create_scan_edges(n, storage, context)),
        // ... 其他数据访问节点
        _ => None,
    }
}

/// 主创建函数
pub fn create_executor<S: StorageClient>(
    node: &PlanNodeEnum,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    for create_fn in CREATE_FNS {
        if let Some(result) = create_fn(node, storage.clone(), context) {
            return result;
        }
    }
    Err(QueryError::ExecutionError(
        format!("Unsupported plan node: {:?}", node)
    ))
}
```

#### 优点
- 极其简单
- 零运行时开销（函数指针内联）
- 易于理解和维护

#### 缺点
- 需要手动维护函数表
- 失去了编译时 exhaustive check

#### 适用性
**短期推荐** - 简单有效，易于实施。

---

## 综合推荐方案

### 阶段1：函数分解（立即实施）

将 `create_executor` 中的 match 分解为多个辅助函数：

```rust
impl<S: StorageClient> ExecutorFactory<S> {
    pub fn create_executor(
        &mut self,
        plan_node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        self.validate_plan_node(plan_node)?;
        
        if self.config.enable_recursion_detection {
            self.recursion_detector
                .validate_executor(plan_node.id(), plan_node.name())
                .map_err(|e| QueryError::ExecutionError(e.to_string()))?;
        }

        match plan_node {
            PlanNodeEnum::Start(n) => self.create_start_executor(n, context),
            
            // 按类别分组
            n if is_data_access(n) => self.create_data_access_executor(n, storage, context),
            n if is_data_modification(n) => self.create_data_modification_executor(n, storage, context),
            n if is_data_processing(n) => self.create_data_processing_executor(n, storage, context),
            // ... 其他类别
            
            _ => Err(QueryError::ExecutionError(
                format!("Unsupported plan node: {:?}", plan_node)
            )),
        }
    }
    
    fn create_data_access_executor(
        &self,
        node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        match node {
            PlanNodeEnum::ScanVertices(n) => {
                let executor = GetVerticesExecutor::new(...);
                Ok(ExecutorEnum::GetVertices(executor))
            }
            // ... 其他数据访问节点
            _ => unreachable!(),
        }
    }
}
```

### 阶段2：类别提取（后续优化）

将每个类别的创建逻辑提取到独立模块：

```
src/query/executor/factory/
├── creators/
│   ├── mod.rs          # 组合创建器
│   ├── data_access.rs  # 数据访问创建器
│   ├── data_modification.rs
│   ├── data_processing.rs
│   └── ...
├── executor_factory.rs
└── mod.rs
```

### 阶段3：宏生成（长期优化）

考虑使用过程宏自动生成创建逻辑：

```rust
#[derive(Executor)]
#[node(ScanVerticesNode)]
struct GetVerticesExecutor {
    // ...
}
```

## 结论

| 模式 | 复杂度 | 可维护性 | 性能 | 推荐度 |
|------|--------|----------|------|--------|
| 当前设计 | 低 | 低 | 高 | - |
| 访问者模式 | 高 | 中 | 高 | ⭐⭐ |
| 类型级联 | 中 | 高 | 中 | ⭐⭐⭐ |
| 组合模式 | 中 | 高 | 高 | ⭐⭐⭐⭐ |
| 代码生成 | 高 | 高 | 高 | ⭐⭐⭐⭐⭐ |
| 函数指针表 | 低 | 中 | 高 | ⭐⭐⭐⭐ |

**最终建议**：
1. **短期**：采用函数分解，将大 match 分解为多个辅助函数
2. **中期**：采用组合模式，将类别创建器提取到独立模块
3. **长期**：考虑使用过程宏自动生成创建逻辑
