# 执行器工厂模块设计文档

**文档版本**: 1.0  
**创建日期**: 2025-12-30  
**参考源**: NebulaGraph 3.8.0  
**模块位置**: `src/query/executor/factory.rs`

## 1. 模块概述

执行器工厂模块负责根据执行计划节点（PlanNode）创建对应的执行器实例，实现 NebulaGraph 风格的工厂模式，为查询执行提供统一的执行器创建接口。

### 1.1 核心职责
- 根据 PlanNode 类型创建对应的执行器
- 管理执行器的生命周期和依赖注入
- 提供执行器创建的验证和错误处理
- 支持执行器的配置和参数传递

### 1.2 设计原则
- **工厂模式**: 统一创建接口，隐藏具体实现
- **类型安全**: 编译期类型检查，避免运行时错误
- **依赖注入**: 自动注入执行器所需的依赖项
- **可扩展性**: 支持新执行器类型的无缝添加

## 2. 架构设计

### 2.1 核心数据结构

```rust
// src/query/executor/factory.rs
pub struct ExecutorFactory<S: StorageEngine> {
    // 存储引擎实例
    storage: Arc<Mutex<S>>,
    
    // 执行上下文工厂
    context_factory: Arc<dyn ExecutionContextFactory>,
    
    // 执行器注册表
    executor_registry: HashMap<PlanNodeKind, ExecutorCreator<S>>,
}

// 执行器创建器类型别名
type ExecutorCreator<S> = fn(
    plan_node: &PlanNodeEnum,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<Box<dyn Executor<S>>, QueryError>;

// 执行器类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlanNodeKind {
    // 数据访问类
    GetVertices,
    GetEdges,
    GetNeighbors,
    ScanVertices,
    ScanEdges,
    
    // 数据处理类
    Filter,
    Project,
    Aggregate,
    Sort,
    Limit,
    TopN,
    
    // 数据修改类
    InsertVertex,
    InsertEdge,
    UpdateVertex,
    DeleteVertex,
    
    // 管理类
    CreateSpace,
    ShowSpaces,
}
```

### 2.2 接口设计

```rust
// 执行器工厂接口
trait ExecutorFactoryTrait<S: StorageEngine> {
    // 创建执行器
    fn create_executor(
        &self,
        plan_node: &PlanNodeEnum,
        context: &ExecutionContext,
    ) -> Result<Box<dyn Executor<S>>, QueryError>;
    
    // 注册执行器创建器
    fn register_executor(
        &mut self,
        kind: PlanNodeKind,
        creator: ExecutorCreator<S>,
    ) -> Result<(), FactoryError>;
    
    // 验证执行器配置
    fn validate_plan_node(&self, plan_node: &PlanNodeEnum) -> Result<(), ValidationError>;
    
    // 获取支持的执行器类型
    fn get_supported_kinds(&self) -> Vec<PlanNodeKind>;
}
```

## 3. NebulaGraph 对标分析

### 3.1 NebulaGraph 实现参考

NebulaGraph 中的执行器工厂主要通过以下机制实现：

```cpp
// nebula-3.8.0/src/graph/executor/Executor.h
class Executor {
public:
    // 工厂方法：根据 PlanNode 类型创建对应 Executor
    static Executor* makeExecutor(QueryContext* qctx, const PlanNode* node);
};

// 具体实现
Executor* Executor::makeExecutor(QueryContext* qctx, const PlanNode* node) {
    switch (node->kind()) {
        case PlanNode::Kind::kGetVertices:
            return pool->makeAndAdd<GetVerticesExecutor>(node, qctx);
        case PlanNode::Kind::kFilter:
            return pool->makeAndAdd<FilterExecutor>(node, qctx);
        case PlanNode::Kind::kProject:
            return pool->makeAndAdd<ProjectExecutor>(node, qctx);
        // ... 更多执行器类型
        default:
            LOG(ERROR) << "Unknown plan node kind: " << node->kind();
            return nullptr;
    }
}
```

### 3.2 关键差异与改进

| 特性 | NebulaGraph | GraphDB 设计 | 改进点 |
|------|-------------|--------------|--------|
| 创建机制 | 静态工厂方法 | 动态注册机制 | 更灵活的扩展性 |
| 依赖管理 | 构造函数参数 | 依赖注入 | 更好的解耦 |
| 错误处理 | 返回 nullptr | Result 类型 | 更安全的错误处理 |
| 线程安全 | 非线程安全 | 线程安全设计 | 支持并发创建 |

## 4. 核心功能实现

### 4.1 工厂初始化

```rust
impl<S: StorageEngine> ExecutorFactory<S> {
    pub fn new(storage: Arc<Mutex<S>>, context_factory: Arc<dyn ExecutionContextFactory>) -> Self {
        let mut factory = ExecutorFactory {
            storage,
            context_factory,
            executor_registry: HashMap::new(),
        };
        
        // 注册默认执行器
        factory.register_default_executors();
        
        factory
    }
    
    // 注册默认执行器集合
    fn register_default_executors(&mut self) {
        self.register_executor(PlanNodeKind::Filter, Self::create_filter_executor)
            .expect("Failed to register Filter executor");
        
        self.register_executor(PlanNodeKind::Project, Self::create_project_executor)
            .expect("Failed to register Project executor");
        
        self.register_executor(PlanNodeKind::Sort, Self::create_sort_executor)
            .expect("Failed to register Sort executor");
        
        self.register_executor(PlanNodeKind::Limit, Self::create_limit_executor)
            .expect("Failed to register Limit executor");
        
        self.register_executor(PlanNodeKind::TopN, Self::create_topn_executor)
            .expect("Failed to register TopN executor");
        
        // ... 注册更多执行器
    }
}
```

### 4.2 执行器创建逻辑

```rust
impl<S: StorageEngine> ExecutorFactoryTrait<S> for ExecutorFactory<S> {
    fn create_executor(
        &self,
        plan_node: &PlanNodeEnum,
        context: &ExecutionContext,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        // 1. 验证执行器类型和配置
        self.validate_plan_node(plan_node)?;
        
        // 2. 获取执行器类型
        let kind = self.get_plan_node_kind(plan_node);
        
        // 3. 查找对应的创建器
        let creator = self.executor_registry
            .get(&kind)
            .ok_or_else(|| QueryError::ExecutorNotSupported(kind.to_string()))?;
        
        // 4. 创建执行器实例
        let executor = creator(plan_node, self.storage.clone(), context)?;
        
        // 5. 验证执行器状态
        executor.validate()?;
        
        Ok(executor)
    }
    
    fn register_executor(
        &mut self,
        kind: PlanNodeKind,
        creator: ExecutorCreator<S>,
    ) -> Result<(), FactoryError> {
        if self.executor_registry.contains_key(&kind) {
            return Err(FactoryError::ExecutorAlreadyRegistered(kind));
        }
        
        self.executor_registry.insert(kind, creator);
        Ok(())
    }
}
```

### 4.3 具体执行器创建实现

```rust
impl<S: StorageEngine> ExecutorFactory<S> {
    // Filter 执行器创建
    fn create_filter_executor(
        plan_node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        let filter_node = plan_node.as_filter()
            .ok_or(QueryError::InvalidPlanNodeType("Filter".to_string()))?;
        
        Ok(Box::new(FilterExecutor::new(
            filter_node.clone(),
            storage,
            context.clone(),
        )))
    }
    
    // Sort 执行器创建
    fn create_sort_executor(
        plan_node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        let sort_node = plan_node.as_sort()
            .ok_or(QueryError::InvalidPlanNodeType("Sort".to_string()))?;
        
        Ok(Box::new(SortExecutor::new(
            sort_node.clone(),
            storage,
            context.clone(),
        )))
    }
    
    // TopN 执行器创建
    fn create_topn_executor(
        plan_node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        let topn_node = plan_node.as_topn()
            .ok_or(QueryError::InvalidPlanNodeType("TopN".to_string()))?;
        
        Ok(Box::new(TopNExecutor::new(
            topn_node.clone(),
            storage,
            context.clone(),
        )))
    }
}
```

## 5. 集成设计

### 5.1 与调度器集成

```rust
// 调度器使用工厂创建执行器
pub struct Scheduler<S: StorageEngine> {
    executor_factory: Arc<ExecutorFactory<S>>,
    // ... 其他字段
}

impl<S: StorageEngine> Scheduler<S> {
    pub async fn schedule_plan(&self, plan: &ExecutionPlan) -> Result<QueryResult, QueryError> {
        // 1. 构建执行器树
        let executor_tree = self.build_executor_tree(plan)?;
        
        // 2. 执行查询
        let result = self.execute_tree(executor_tree).await?;
        
        Ok(result)
    }
    
    fn build_executor_tree(&self, plan: &ExecutionPlan) -> Result<ExecutorTree<S>, QueryError> {
        let root_node = plan.root();
        let context = self.create_execution_context()?;
        
        // 使用工厂创建执行器
        let root_executor = self.executor_factory
            .create_executor(root_node, &context)?;
        
        // 递归构建执行器树
        self.build_tree_recursive(root_executor, root_node, &context)
    }
}
```

### 5.2 与执行上下文集成

```rust
// 执行器工厂需要 ExecutionContext 来创建执行器
impl<S: StorageEngine> ExecutorFactory<S> {
    fn create_executor_with_context(
        &self,
        plan_node: &PlanNodeEnum,
    ) -> Result<(Box<dyn Executor<S>>, ExecutionContext), QueryError> {
        // 创建新的执行上下文
        let context = self.context_factory.create_context()?;
        
        // 使用上下文创建执行器
        let executor = self.create_executor(plan_node, &context)?;
        
        Ok((executor, context))
    }
}
```

## 6. 错误处理设计

### 6.1 错误类型定义

```rust
// 工厂特定错误类型
#[derive(Debug, thiserror::Error)]
pub enum FactoryError {
    #[error("执行器类型 {0} 已注册")]
    ExecutorAlreadyRegistered(PlanNodeKind),
    
    #[error("执行器类型 {0} 未找到")]
    ExecutorNotFound(PlanNodeKind),
    
    #[error("执行器创建失败: {0}")]
    CreationFailed(String),
    
    #[error("执行器配置无效: {0}")]
    InvalidConfiguration(String),
}

// 验证错误类型
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("PlanNode 类型不匹配")]
    PlanNodeTypeMismatch,
    
    #[error("缺少必要的配置参数")]
    MissingRequiredParameters,
    
    #[error("配置参数无效: {0}")]
    InvalidParameter(String),
}
```

### 6.2 验证逻辑实现

```rust
impl<S: StorageEngine> ExecutorFactory<S> {
    fn validate_plan_node(&self, plan_node: &PlanNodeEnum) -> Result<(), ValidationError> {
        // 1. 验证 PlanNode 类型是否支持
        let kind = self.get_plan_node_kind(plan_node);
        if !self.executor_registry.contains_key(&kind) {
            return Err(ValidationError::PlanNodeTypeMismatch);
        }
        
        // 2. 验证 PlanNode 配置
        self.validate_plan_node_configuration(plan_node)?;
        
        // 3. 验证存储引擎兼容性
        self.validate_storage_compatibility(plan_node)?;
        
        Ok(())
    }
    
    fn validate_plan_node_configuration(&self, plan_node: &PlanNodeEnum) -> Result<(), ValidationError> {
        match plan_node {
            PlanNodeEnum::Filter(node) => {
                if node.condition.is_none() {
                    return Err(ValidationError::MissingRequiredParameters);
                }
            }
            PlanNodeEnum::Sort(node) => {
                if node.sort_items.is_empty() {
                    return Err(ValidationError::MissingRequiredParameters);
                }
            }
            PlanNodeEnum::TopN(node) => {
                if node.limit == 0 {
                    return Err(ValidationError::InvalidParameter("limit cannot be zero".to_string()));
                }
            }
            // ... 其他类型验证
            _ => {}
        }
        
        Ok(())
    }
}
```

## 7. 性能优化

### 7.1 创建性能优化
- 使用对象池减少内存分配
- 缓存常用执行器实例
- 预编译执行器创建逻辑

### 7.2 内存优化
- 使用智能指针管理执行器生命周期
- 最小化执行器实例大小
- 共享不可变配置数据

## 8. 测试策略

### 8.1 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_executor_creation() {
        let factory = create_test_factory();
        let plan_node = create_test_filter_node();
        let context = create_test_context();
        
        let executor = factory.create_executor(&plan_node, &context).unwrap();
        assert!(executor.is::<FilterExecutor>());
    }
    
    #[test]
    fn test_executor_registration() {
        let mut factory = create_test_factory();
        
        // 测试重复注册
        let result = factory.register_executor(PlanNodeKind::Filter, dummy_creator);
        assert!(result.is_err());
    }
}
```

### 8.2 集成测试
- 执行器创建性能测试
- 并发创建压力测试
- 端到端查询执行测试

## 9. 未来扩展

### 9.1 计划功能
- 支持动态执行器加载
- 添加执行器性能监控
- 支持执行器热更新

### 9.2 优化方向
- 执行器创建预编译优化
- 支持执行器配置模板
- 添加执行器版本管理

## 10. 总结

执行器工厂模块为 GraphDB 提供了 NebulaGraph 级别的执行器创建能力，通过现代化的 Rust 设计实现了更好的类型安全性和扩展性。该设计遵循 NebulaGraph 的工厂模式理念，同时充分利用 Rust 语言特性进行优化，为查询执行提供了可靠的基础设施。