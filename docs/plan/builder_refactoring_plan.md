# Builder 结构改造方案

## 现状分析

### 当前结构

```
Builders (包含9个builder实例)
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

### 调用链

```rust
// 3层嵌套调用
self.builders.data_access().build_scan_vertices(node, storage, context)
```

### 存在的问题

1. **结构冗余** - 每个 builder 都是无状态的空结构体（只有 `PhantomData`），但需要创建9个实例
2. **调用链过长** - 3层嵌套增加了代码复杂度
3. **违反开闭原则** - 添加新执行器类型需要修改多个地方

## 改造方案

### 目标

1. 简化 `Builders` 结构体
2. 缩短调用链
3. 保持向后兼容（可选）
4. 提高代码可维护性

### 方案一：简化 Builders 结构体（推荐）

#### 步骤1：修改 Builder 方法为关联函数

将 builder 方法从实例方法改为关联函数（去掉 `&self` 参数）：

```rust
// 修改前
impl<S: StorageClient + Send + 'static> DataAccessBuilder<S> {
    pub fn build_scan_vertices(
        &self,
        node: &ScanVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // ...
    }
}

// 修改后
impl<S: StorageClient + Send + 'static> DataAccessBuilder<S> {
    pub fn build_scan_vertices(
        node: &ScanVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // ...
    }
}
```

#### 步骤2：简化 Builders 结构体

```rust
// 修改前
pub struct Builders<S: StorageClient + 'static> {
    data_access: DataAccessBuilder<S>,
    data_modification: DataModificationBuilder<S>,
    data_processing: DataProcessingBuilder<S>,
    join: JoinBuilder<S>,
    set_operation: SetOperationBuilder<S>,
    traversal: TraversalBuilder<S>,
    transformation: TransformationBuilder<S>,
    control_flow: ControlFlowBuilder<S>,
    admin: AdminBuilder<S>,
}

// 修改后
pub struct Builders<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}
```

#### 步骤3：在 Builders 中直接暴露 builder 方法

```rust
impl<S: StorageClient + Send + 'static> Builders<S> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Build ScanVertices executor
    pub fn build_scan_vertices(
        &self,
        node: &ScanVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        DataAccessBuilder::build_scan_vertices(node, storage, context)
    }

    // ... 其他方法
}
```

#### 步骤4：简化调用链

```rust
// 修改前
self.builders.data_access().build_scan_vertices(node, storage, context)

// 修改后
self.builders.build_scan_vertices(node, storage, context)
```

### 方案二：完全扁平化（可选）

如果希望进一步简化，可以完全移除 builder 分类，直接在 `ExecutorFactory` 中创建执行器：

```rust
impl<S: StorageClient + Send + 'static> ExecutorFactory<S> {
    fn create_data_access_executor(
        &self,
        node: &DataAccessNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        match node {
            DataAccessNode::ScanVertices(n) => {
                let executor = GetVerticesExecutor::new(...);
                Ok(ExecutorEnum::GetVertices(executor))
            }
            // ...
        }
    }
}
```

**优点**：
- 最简化的结构
- 没有中间层

**缺点**：
- 所有创建逻辑集中在 `ExecutorFactory`
- 失去分类组织

## 实施步骤

### 阶段1：准备（1天）

1. 创建测试覆盖
2. 备份当前代码
3. 准备回滚方案

### 阶段2：逐个迁移 Builder（每类1天）

按以下顺序迁移：

1. **DataAccessBuilder** - 最简单，作为试点
2. **DataModificationBuilder**
3. **DataProcessingBuilder**
4. **JoinBuilder**
5. **SetOperationBuilder**
6. **TraversalBuilder**
7. **TransformationBuilder**
8. **ControlFlowBuilder**
9. **AdminBuilder** - 最复杂，最后处理

每个 builder 的迁移步骤：

1. 修改 builder 方法为关联函数（去掉 `&self`）
2. 在 `Builders` 中添加对应的直接方法
3. 更新 `ExecutorFactory` 中的调用
4. 运行测试验证

### 阶段3：清理（1天）

1. 移除旧的 accessor 方法
2. 更新文档
3. 运行完整测试套件

## 风险与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 编译错误 | 中 | 高 | 逐个 builder 迁移，每次验证编译 |
| 运行时错误 | 低 | 高 | 保持现有测试覆盖，每次迁移后运行测试 |
| 性能回归 | 低 | 中 | 关联函数调用与实例方法性能相同 |
| 代码冲突 | 中 | 中 | 快速完成迁移，避免长时间分支 |

## 预期收益

1. **代码简化**：`Builders` 结构体从9个字段减少到1个
2. **调用简化**：调用链从3层减少到2层
3. **内存优化**：减少9个空结构体实例的创建（虽然影响很小）
4. **可维护性**：更清晰的代码结构

## 回滚方案

如果需要回滚：

1. 恢复 `Builders` 结构体的9个字段
2. 恢复 builder 方法的 `&self` 参数
3. 恢复 `ExecutorFactory` 中的调用链

所有修改都是局部的，可以安全回滚。

## 参考文件

- `src/query/executor/factory/builders/mod.rs`
- `src/query/executor/factory/builders/data_access_builder.rs`
- `src/query/executor/factory/executor_factory.rs`
