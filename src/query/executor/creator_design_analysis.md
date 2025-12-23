# 执行器创建器设计分析

## 当前工厂模式分析

### 工厂模式的优点

1. **解耦创建逻辑**：将执行器的创建逻辑与使用逻辑分离
2. **易于扩展**：添加新的执行器类型只需注册新的创建器
3. **统一接口**：通过 `ExecutorCreator` trait 提供统一的创建接口
4. **类型安全**：编译时确保执行器类型的正确性

### 工厂模式的问题

1. **复杂性增加**：需要为每种执行器类型创建对应的创建器
2. **间接性**：增加了一层抽象，可能影响性能
3. **样板代码**：大量相似的创建器实现

## 替代设计方案

### 1. 直接匹配模式

```rust
impl<S: StorageEngine + 'static + std::fmt::Debug> ExecutorFactory<S> {
    pub fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        match plan_node.kind() {
            PlanNodeKind::ScanVertices => Ok(Box::new(ScanVerticesExecutor::new(storage))),
            PlanNodeKind::Filter => Ok(Box::new(FilterExecutor::new(storage))),
            PlanNodeKind::Project => Ok(Box::new(ProjectExecutor::new(storage))),
            // 其他类型...
            kind => Err(QueryError::ExecutionError(format!("未支持的执行器类型: {:?}", kind))),
        }
    }
}
```

**优点**：
- 简单直接，没有额外的抽象层
- 性能更好，没有动态分发开销
- 代码集中，易于理解和维护

**缺点**：
- 违反开闭原则，添加新类型需要修改工厂代码
- 所有创建逻辑集中在一个方法中，可能变得庞大

### 2. 宏驱动的注册模式

```rust
macro_rules! register_executors {
    ($factory:expr, $storage:ident => {
        $($kind:ident => $executor_type:ident),* $(,)?
    }) => {
        $(
            $factory.register_creator(
                PlanNodeKind::$kind,
                Box::new(|plan_node, storage: Arc<Mutex<S>>| {
                    Ok(Box::new($executor_type::new(plan_node, storage)) as Box<dyn Executor<S>>)
                })
            );
        )*
    };
}

impl<S: StorageEngine + 'static + std::fmt::Debug> ExecutorFactory<S> {
    fn register_default_creators(&mut self) {
        register_executors!(self, storage => {
            ScanVertices => ScanVerticesExecutor,
            ScanEdges => ScanEdgesExecutor,
            Filter => FilterExecutor,
            Project => ProjectExecutor,
            // 其他类型...
        });
    }
}
```

**优点**：
- 减少样板代码
- 保持工厂模式的扩展性
- 类型安全

**缺点**：
- 宏的调试困难
- 代码可读性可能降低

### 3. 建造者模式

```rust
pub struct ExecutorBuilder<S: StorageEngine> {
    storage: Arc<Mutex<S>>,
    config: ExecutorConfig,
}

impl<S: StorageEngine + 'static + std::fmt::Debug> ExecutorBuilder<S> {
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage,
            config: ExecutorConfig::default(),
        }
    }
    
    pub fn build_from_plan(&self, plan_node: &dyn PlanNode) -> Result<Box<dyn Executor<S>>, QueryError> {
        match plan_node.kind() {
            PlanNodeKind::ScanVertices => self.build_scan_vertices(plan_node),
            PlanNodeKind::Filter => self.build_filter(plan_node),
            // 其他类型...
        }
    }
    
    fn build_scan_vertices(&self, plan_node: &dyn PlanNode) -> Result<Box<dyn Executor<S>>, QueryError> {
        // 具体的构建逻辑
        Ok(Box::new(ScanVerticesExecutor::new(
            self.storage.clone(),
            self.config.clone(),
            plan_node,
        )))
    }
}
```

**优点**：
- 更灵活的配置选项
- 可以分步骤构建复杂的执行器
- 易于测试

**缺点**：
- 增加了复杂性
- 可能过度设计

## 推荐方案

考虑到当前项目的需求和复杂度，我推荐使用**直接匹配模式**，原因如下：

1. **简单性**：项目目前处于早期阶段，简单直接的实现更合适
2. **性能**：避免不必要的动态分发开销
3. **可维护性**：所有创建逻辑集中在一个地方，易于理解和修改
4. **渐进式演进**：未来如果需要更复杂的创建逻辑，可以再重构为工厂模式

## 实现建议

```rust
impl<S: StorageEngine + 'static + std::fmt::Debug> ExecutorFactory<S> {
    pub fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        match plan_node.kind() {
            // 基础执行器
            PlanNodeKind::Start => Ok(Box::new(crate::query::executor::base::StartExecutor::new())),
            PlanNodeKind::Unknown => Ok(Box::new(crate::query::executor::base::DefaultExecutor::new())),
            
            // 数据访问执行器
            PlanNodeKind::ScanVertices => {
                // TODO: 实现扫描顶点执行器
                Err(QueryError::ExecutionError("扫描顶点执行器尚未实现".to_string()))
            }
            PlanNodeKind::ScanEdges => {
                // TODO: 实现扫描边执行器
                Err(QueryError::ExecutionError("扫描边执行器尚未实现".to_string()))
            }
            
            // 结果处理执行器
            PlanNodeKind::Filter => {
                // TODO: 实现过滤执行器
                Err(QueryError::ExecutionError("过滤执行器尚未实现".to_string()))
            }
            PlanNodeKind::Project => {
                // TODO: 实现投影执行器
                Err(QueryError::ExecutionError("投影执行器尚未实现".to_string()))
            }
            PlanNodeKind::Limit => {
                // TODO: 实现限制执行器
                Err(QueryError::ExecutionError("限制执行器尚未实现".to_string()))
            }
            PlanNodeKind::Sort => {
                // TODO: 实现排序执行器
                Err(QueryError::ExecutionError("排序执行器尚未实现".to_string()))
            }
            PlanNodeKind::Aggregate => {
                // TODO: 实现聚合执行器
                Err(QueryError::ExecutionError("聚合执行器尚未实现".to_string()))
            }
            
            // 数据处理执行器
            PlanNodeKind::HashInnerJoin | PlanNodeKind::HashLeftJoin | PlanNodeKind::CartesianProduct => {
                // TODO: 实现连接执行器
                Err(QueryError::ExecutionError("连接执行器尚未实现".to_string()))
            }
            
            // 图遍历执行器
            PlanNodeKind::Expand => {
                // TODO: 实现扩展执行器
                Err(QueryError::ExecutionError("扩展执行器尚未实现".to_string()))
            }
            
            kind => Err(QueryError::ExecutionError(format!("未知的执行器类型: {:?}", kind))),
        }
    }
}
```

这种实现方式：
1. 简单直接，易于理解
2. 性能好，没有额外的抽象层
3. 易于调试和测试
4. 为未来实现具体的执行器提供了清晰的框架

随着项目的发展，如果发现需要更复杂的创建逻辑，可以再考虑重构为更高级的设计模式。