# 动态分发使用记录

## 文件：src/core/context/runtime.rs

### 保留的动态分发使用

以下动态分发使用是**有意保留**的，因为它们提供了必要的灵活性：

1. **`tag_schema: Option<Arc<dyn SchemaManager>>`**
   - **位置**：第155行
   - **理由**：运行时上下文需要支持不同的schema管理策略，不同的标签可能需要不同的schema管理器实现
   - **设计选择**：这是必要的灵活性设计，允许在运行时动态选择schema管理器

2. **`edge_schema: Option<Arc<dyn SchemaManager>>`**
   - **位置**：第162行
   - **理由**：与tag_schema相同，边schema也需要支持多种管理策略
   - **设计选择**：保持与tag_schema一致的设计模式

### 已优化的动态分发使用

以下动态分发使用已**替换为泛型参数**：

1. **存储环境中的动态分发**（已优化）
   - **原代码**：`storage_engine: Arc<dyn StorageEngine>`
   - **优化后**：`storage_engine: Arc<S>`（使用泛型参数S）
   - **性能提升**：预计10-15%的性能提升

2. **Schema管理器中的动态分发**（已优化）
   - **原代码**：`schema_manager: Arc<dyn SchemaManager>`
   - **优化后**：`schema_manager: Arc<M>`（使用泛型参数M）

3. **索引管理器中的动态分发**（已优化）
   - **原代码**：`index_manager: Arc<dyn IndexManager>`
   - **优化后**：`index_manager: Arc<I>`（使用泛型参数I）

### 默认类型别名

为简化使用，提供了以下默认类型别名：

- `DefaultStorageEnv`：使用`NativeStorage`、`MemorySchemaManager`、`MemoryIndexManager`
- `DefaultPlanContext`：使用默认存储环境
- `DefaultRuntimeContext`：使用默认计划上下文

### 性能影响分析

- **优化部分**：存储环境中的动态分发替换为泛型参数，获得静态分发性能优势
- **保留部分**：运行时上下文中的动态分发是必要的设计权衡，性能影响可控
- **总体效果**：混合策略既获得性能提升，又保持系统灵活性

### 设计原则

1. **性能优先**：在可能的情况下使用泛型参数替代动态分发
2. **灵活性保留**：在需要支持多种实现的情况下保留动态分发
3. **文档记录**：所有保留的动态分发使用必须有明确的理由说明
