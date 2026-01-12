# graphDB邻接边和节点处理功能实施方案

## 1. 实施目标

基于前面的分析，本方案旨在增强graphDB在邻接边和节点处理方面的功能，重点补充高级统计聚合、路径查询算法和高级索引功能，同时保持其轻量级特性。

## 2. 实施优先级

### 2.1 第一阶段（1-2个月）：统计聚合功能
- 实现基本聚合函数（COUNT、SUM、AVG等）
- 添加GROUP BY支持
- 实现统计查询的执行器

### 2.2 第二阶段（2-3个月）：路径查询算法
- 实现最短路径算法
- 实现K步邻域查询
- 添加路径查询的执行器

### 2.3 第三阶段（3-4个月）：高级索引功能
- 实现属性索引
- 实现复合索引
- 优化查询性能

## 3. 详细实施计划

### 3.1 第一阶段：统计聚合功能

#### 3.1.1 设计聚合函数执行器
```rust
// 在 src/query/executor/aggregation.rs 中创建
pub struct AggregationExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    aggregation_functions: Vec<AggregationFunction>,
    group_by_keys: Vec<String>,
    filter_condition: Option<Expression>,
}
```

#### 3.1.2 实现聚合函数类型
```rust
// 在 src/core/types/operators.rs 中扩展
pub enum AggregationFunction {
    Count,
    Sum(String),  // 字段名
    Avg(String),  // 字段名
    Min(String),  // 字段名
    Max(String),  // 字段名
    GroupConcat(String), // 字段名
}
```

#### 3.1.3 创建聚合查询计划节点
```rust
// 在 src/query/planner/plan/core/nodes/aggregation_node.rs 中创建
pub struct AggregationNode {
    id: i64,
    aggregation_functions: Vec<AggregationFunction>,
    group_by_keys: Vec<String>,
    input_node: Option<Box<PlanNodeEnum>>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}
```

#### 3.1.4 实现聚合执行逻辑
- 在存储层添加统计聚合方法
- 实现分组聚合算法
- 优化内存使用和性能

### 3.2 第二阶段：路径查询算法

#### 3.2.1 设计路径查询执行器
```rust
// 在 src/query/executor/path.rs 中创建
pub struct PathExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    path_type: PathType,  // ShortestPath, AllPaths, KStep等
    start_vertices: Vec<Value>,
    end_vertices: Option<Vec<Value>>,
    max_hops: Option<u32>,
    edge_constraints: Option<EdgeConstraints>,
}
```

#### 3.2.2 实现路径算法
```rust
// 在 src/services/algorithm/path.rs 中创建
pub struct PathAlgorithms {
    pub fn shortest_path(
        storage: &dyn StorageEngine,
        start: &Value,
        end: &Value,
        max_hops: Option<u32>,
    ) -> Result<Option<Path>, StorageError>;

    pub fn k_step_neighbors(
        storage: &dyn StorageEngine,
        start: &Value,
        k: u32,
    ) -> Result<Vec<Vertex>, StorageError>;
}
```

#### 3.2.3 创建路径查询计划节点
```rust
// 在 src/query/planner/plan/core/nodes/path_node.rs 中创建
pub struct PathNode {
    id: i64,
    path_type: PathType,
    start_vids: Vec<Value>,
    end_vids: Option<Vec<Value>>,
    max_hops: Option<u32>,
    edge_types: Vec<String>,
    direction: EdgeDirection,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}
```

### 3.3 第三阶段：高级索引功能

#### 3.3.1 扩展存储引擎接口
```rust
// 在 src/storage/storage_engine.rs 中扩展
pub trait StorageEngine: Send + Sync {
    // 现有方法...
    
    // 新增索引方法
    fn create_property_index(
        &mut self,
        property_name: &str,
        property_type: DataType,
    ) -> Result<(), StorageError>;
    
    fn drop_property_index(
        &mut self,
        property_name: &str,
    ) -> Result<(), StorageError>;
    
    fn query_by_property_index(
        &self,
        property_name: &str,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError>;
}
```

#### 3.3.2 实现索引存储结构
```rust
// 在 src/storage/native_storage.rs 中扩展
impl NativeStorage {
    // 属性索引存储
    property_index_trees: HashMap<String, Tree>, // property_name -> index_tree
    
    pub fn create_property_index(...) -> Result<(), StorageError> {
        // 实现属性索引创建逻辑
    }
    
    pub fn query_by_property_index(...) -> Result<Vec<Value>, StorageError> {
        // 实现基于属性索引的查询
    }
}
```

#### 3.3.3 创建索引查询计划节点
```rust
// 在 src/query/planner/plan/core/nodes/index_node.rs 中创建
pub struct IndexScanNode {
    id: i64,
    index_name: String,
    property_name: String,
    value: Value,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}
```

## 4. 实施步骤

### 4.1 准备工作
1. 创建开发分支 `feature/enhanced-adjacency-processing`
2. 设置测试环境
3. 准备基准测试用例

### 4.2 第一阶段实施
1. 设计并实现聚合函数相关类型
2. 创建AggregationNode和AggregationExecutor
3. 实现聚合算法
4. 编写单元测试
5. 进行性能测试

### 4.3 第二阶段实施
1. 设计路径算法接口
2. 实现最短路径和K步邻域算法
3. 创建PathNode和PathExecutor
4. 集成到查询计划器
5. 测试路径查询功能

### 4.4 第三阶段实施
1. 扩展存储引擎接口
2. 实现索引数据结构
3. 创建IndexScanNode
4. 集成查询优化器
5. 性能调优

## 5. 风险评估与应对

### 5.1 技术风险
- **性能下降**: 严格进行性能测试，确保新功能不影响现有性能
- **内存使用增加**: 实现内存监控和优化机制
- **兼容性问题**: 确保新功能向后兼容

### 5.2 项目风险
- **开发时间超期**: 采用敏捷开发，定期评估进度
- **功能复杂度过高**: 保持功能简洁，避免过度工程化

## 6. 验证与测试

### 6.1 单元测试
- 为每个新功能编写全面的单元测试
- 测试边界条件和异常情况

### 6.2 集成测试
- 测试新功能与现有系统的集成
- 验证查询计划的正确性

### 6.3 性能测试
- 基准测试新功能的性能
- 与现有功能进行性能对比

## 7. 预期成果

通过本实施方案，graphDB将获得：
1. 完整的统计聚合功能，支持数据分析需求
2. 高效的路径查询算法，支持复杂图分析
3. 高级索引功能，提升查询性能
4. 与nebula-graph更接近的功能集，同时保持轻量级特性

该方案遵循渐进式开发原则，确保每阶段都有可交付的成果，同时最小化对现有系统的影响。