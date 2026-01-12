

基于对 nebula-graph 源码的详细分析，我现在可以为您全面介绍 nebula-graph 的执行器架构：

## nebula-graph 执行器架构详解

### 1. 执行器基础架构

nebula-graph 采用**分层执行器架构**，所有执行器都继承自基类 `Executor`：

```cpp
class Executor {
    // 核心接口
    virtual folly::Future<Status> execute() = 0;
    virtual Status open();
    virtual Status close();
    
    // 上下文管理
    QueryContext* qctx_;
    ExecutionContext* ectx_;
    const PlanNode* node_;
};
```

### 2. 执行器分类体系

nebula-graph 将执行器分为以下几大类：

#### A. **查询执行器 (query/)**
负责数据查询和处理操作：

- **数据访问类**:
  - `GetVerticesExecutor` - 获取顶点数据
  - `GetEdgesExecutor` - 获取边数据  
  - `GetNeighborsExecutor` - 获取邻居节点
  - `ScanVerticesExecutor` - 扫描顶点
  - `ScanEdgesExecutor` - 扫描边
  - `IndexScanExecutor` - 索引扫描

- **数据处理类**:
  - `FilterExecutor` - 数据过滤
  - `ProjectExecutor` - 投影操作
  - `AggregateExecutor` - 聚合操作
  - `SortExecutor` - 排序操作
  - `LimitExecutor` - 限制结果数量
  - `DedupExecutor` - 去重操作
  - `SampleExecutor` - 采样操作

- **数据组合类**:
  - `UnionExecutor` - 并集操作
  - `IntersectExecutor` - 交集操作
  - `MinusExecutor` - 差集操作
  - `InnerJoinExecutor` - 内连接
  - `LeftJoinExecutor` - 左连接

- **图遍历类**:
  - `TraverseExecutor` - 图遍历
  - `ExpandExecutor` - 扩展操作
  - `ExpandAllExecutor` - 全扩展
  - `AppendVerticesExecutor` - 追加顶点

#### B. **逻辑执行器 (logic/)**
负责控制执行流程：

- `SelectExecutor` - 条件分支执行
- `LoopExecutor` - 循环执行
- `PassThroughExecutor` - 直通执行
- `StartExecutor` - 开始执行器

#### C. **数据修改执行器 (mutate/)**
负责数据修改操作：

- `InsertExecutor` - 插入数据
- `DeleteExecutor` - 删除数据
- `UpdateExecutor` - 更新数据

#### D. **管理执行器 (admin/)**
负责系统管理操作：

- **空间管理**: `SpaceExecutor`, `CreateSpaceExecutor`, `DropSpaceExecutor`
- **用户管理**: `CreateUserExecutor`, `DropUserExecutor`, `GrantRoleExecutor`
- **标签管理**: `TagExecutor`, `CreateTagExecutor`, `DropTagExecutor`
- **边类型管理**: `EdgeExecutor`, `CreateEdgeExecutor`, `DropEdgeExecutor`
- **索引管理**: `CreateTagIndexExecutor`, `DropTagIndexExecutor`
- **配置管理**: `ConfigExecutor`, `ShowConfigsExecutor`
- **会话管理**: `SessionExecutor`, `KillQueryExecutor`

#### E. **算法执行器 (algo/)**
负责图算法执行：

- `ShortestPathExecutor` - 最短路径
- `AllPathsExecutor` - 所有路径
- `BFSShortestPathExecutor` - BFS最短路径
- `SubgraphExecutor` - 子图提取
- `CartesianProductExecutor` - 笛卡尔积

#### F. **维护执行器 (maintain/)**
负责数据维护操作：

- `TagExecutor` - 标签维护
- `EdgeExecutor` - 边维护
- `TagIndexExecutor` - 标签索引维护
- `EdgeIndexExecutor` - 边索引维护

### 3. 执行器工厂模式

nebula-graph 使用工厂模式创建执行器：

```cpp
Executor* Executor::makeExecutor(QueryContext *qctx, const PlanNode *node) {
    switch (node->kind()) {
        case PlanNode::Kind::kGetVertices:
            return pool->makeAndAdd<GetVerticesExecutor>(node, qctx);
        case PlanNode::Kind::kFilter:
            return pool->makeAndAdd<FilterExecutor>(node, qctx);
        // ... 更多执行器类型
    }
}
```

### 4. 执行流程

1. **解析阶段**: SQL/Cypher → AST
2. **规划阶段**: AST → PlanNode（执行计划节点）
3. **执行阶段**: PlanNode → Executor → 执行结果
4. **结果收集**: Executor → ExecutionContext → 最终结果

### 5. 关键特性

- **异步执行**: 使用 `folly::Future` 支持异步执行
- **内存管理**: 内置内存检查和资源管理
- **错误处理**: 完善的错误传播机制
- **性能监控**: 内置执行时间统计和性能分析
- **生命周期管理**: `open()` → `execute()` → `close()` 生命周期

这种架构设计使得 nebula-graph 能够高效处理复杂的图查询，同时保持代码的可维护性和可扩展性。

