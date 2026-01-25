# GraphDB 与 Nebula-Graph 执行器类型对比分析

## 一、执行器类型概览

### 1.1 Nebula-Graph 执行器分类

Nebula-Graph 的执行器按照功能分为六大类别，共计超过 80 个执行器实现。以下是各分类的详细统计：

| 分类 | 目录位置 | 执行器数量 | 主要功能 |
|------|----------|------------|----------|
| 查询执行器 | query/ | 约 35 个 | 数据检索、过滤、聚合、连接等 |
| 算法执行器 | algo/ | 8 个 | 图遍历、路径查找、子图提取 |
| 逻辑执行器 | logic/ | 5 个 | 循环、条件分支、参数传递 |
| 管理执行器 | admin/ | 约 40 个 | 空间管理、用户管理、系统监控 |
| 维护执行器 | maintain/ | 8 个 | 标签、边、索引的创建与维护 |
| 变更执行器 | mutate/ | 5 个 | 插入、更新、删除数据 |

### 1.2 GraphDB 执行器分类

GraphDB 当前的执行器实现按照功能分为四大类别，共计约 25 个执行器实现：

| 分类 | 模块位置 | 执行器数量 | 主要功能 |
|------|----------|------------|----------|
| 数据访问 | data_access.rs | 5 个 | 顶点、边、索引访问 |
| 结果处理 | result_processing/ | 12 个 | 投影、过滤、聚合、排序等 |
| 数据处理 | data_processing/ | 8 个 | 图遍历、连接、集合运算 |
| 逻辑控制 | logic/ | 3 个 | 循环控制 |

## 二、详细执行器对比

### 2.1 查询执行器对比

查询执行器是执行引擎最核心的组成部分，负责处理数据检索和转换操作。以下是两个项目在该类别下的详细对比：

| 执行器名称 | Nebula-Graph | GraphDB | 实现状态 | 备注 |
|------------|--------------|---------|----------|------|
| **数据访问** |
| GetVerticesExecutor | ✅ query/GetVerticesExecutor | ✅ data_access.rs | 完整实现 | 功能基本对等 |
| GetEdgesExecutor | ✅ query/GetEdgesExecutor | ✅ data_access.rs | 基础实现 | 需增强边类型过滤 |
| GetNeighborsExecutor | ✅ query/GetNeighborsExecutor | ⚠️ 部分实现 | 基础实现 | 需完善多跳支持 |
| ScanVerticesExecutor | ✅ query/ScanVerticesExecutor | ❌ 未实现 | - | 可复用 GetVertices |
| ScanEdgesExecutor | ✅ query/ScanEdgesExecutor | ❌ 未实现 | - | 需新增实现 |
| IndexScanExecutor | ✅ query/IndexScanExecutor | ✅ data_access.rs | 基础实现 | 需完善条件扫描 |
| FulltextIndexScanExecutor | ✅ query/FulltextIndexScanExecutor | ❌ 未实现 | - | 依赖全文索引模块 |
| **投影过滤** |
| ProjectExecutor | ✅ query/ProjectExecutor | ✅ result_processing/ | 完整实现 | 功能对等 |
| FilterExecutor | ✅ query/FilterExecutor | ✅ result_processing/ | 完整实现 | 功能对等 |
| UnwindExecutor | ✅ query/UnwindExecutor | ✅ result_processing/ | 完整实现 | 功能对等 |
| AssignExecutor | ✅ query/AssignExecutor | ✅ result_processing/ | 完整实现 | 功能对等 |
| **聚合排序** |
| AggregateExecutor | ✅ query/AggregateExecutor | ✅ result_processing/ | 完整实现 | 功能基本对等 |
| SortExecutor | ✅ query/SortExecutor | ✅ result_processing/ | 完整实现 | 功能对等 |
| TopNExecutor | ✅ query/TopNExecutor | ✅ result_processing/ | 完整实现 | 功能对等 |
| LimitExecutor | ✅ query/LimitExecutor | ✅ result_processing/ | 完整实现 | 功能对等 |
| SampleExecutor | ✅ query/SampleExecutor | ✅ result_processing/ | 完整实现 | 功能对等 |
| DedupExecutor | ✅ query/DedupExecutor | ✅ result_processing/ | 完整实现 | 功能对等 |
| **连接操作** |
| InnerJoinExecutor | ✅ query/InnerJoinExecutor | ✅ data_processing/join/ | 完整实现 | 功能对等 |
| LeftJoinExecutor | ✅ query/LeftJoinExecutor | ✅ data_processing/join/ | 完整实现 | 功能对等 |
| RightJoinExecutor | ✅ query/LeftJoinExecutor | ✅ data_processing/join/ | 完整实现 | 功能对等 |
| FullOuterJoinExecutor | ❌ 未实现 | ✅ data_processing/join/ | 扩展实现 | GraphDB 扩展功能 |
| CrossJoinExecutor | ✅ query/UnionExecutor | ✅ data_processing/join/ | 完整实现 | 功能对等 |
| **数据收集** |
| DataCollectExecutor | ✅ query/DataCollectExecutor | ❌ 未实现 | - | 分布式数据聚合 |
| **路径遍历** |
| ExpandExecutor | ✅ query/ExpandExecutor | ✅ data_processing/graph_traversal/ | 完整实现 | 功能对等 |
| ExpandAllExecutor | ✅ query/ExpandAllExecutor | ✅ data_processing/graph_traversal/ | 完整实现 | 功能对等 |
| TraverseExecutor | ✅ query/TraverseExecutor | ✅ data_processing/graph_traversal/ | 完整实现 | 功能对等 |
| **集合操作** |
| UnionExecutor | ✅ query/UnionExecutor | ❌ 未实现 | - | 需新增实现 |
| UnionAllVersionVarExecutor | ✅ query/UnionAllVersionVarExecutor | ❌ 未实现 | - | 多版本联合 |
| IntersectExecutor | ✅ query/IntersectExecutor | ✅ data_processing/set_operations/ | 完整实现 | 功能对等 |
| MinusExecutor | ✅ query/MinusExecutor | ✅ data_processing/set_operations/ | 完整实现 | 功能对等 |
| **数据转换** |
| AppendVerticesExecutor | ✅ query/AppendVerticesExecutor | ✅ result_processing/transformations/ | 完整实现 | 功能对等 |
| PatternApplyExecutor | ✅ query/PatternApplyExecutor | ✅ result_processing/transformations/ | 完整实现 | 功能对等 |
| RollUpApplyExecutor | ✅ query/RollUpApplyExecutor | ✅ result_processing/transformations/ | 完整实现 | 功能对等 |

### 2.2 算法执行器对比

算法执行器负责执行图遍历和路径查找等图算法。以下是两个项目在该类别下的详细对比：

| 执行器名称 | Nebula-Graph | GraphDB | 实现状态 | 备注 |
|------------|--------------|---------|----------|------|
| ShortestPathExecutor | ✅ algo/ShortestPathExecutor | ✅ data_processing/graph_traversal/ | 完整实现 | 功能对等 |
| MultiShortestPathExecutor | ✅ algo/MultiShortestPathExecutor | ✅ data_processing/graph_traversal/ | 完整实现 | 功能对等 |
| BFSShortestPathExecutor | ✅ algo/BFSShortestPathExecutor | ❌ 未实现 | - | 需新增实现 |
| AllPathsExecutor | ✅ algo/AllPathsExecutor | ✅ data_access.rs | 基础实现 | 需完善算法 |
| CartesianProductExecutor | ✅ algo/CartesianProductExecutor | ❌ 未实现 | - | 需新增实现 |
| SubgraphExecutor | ✅ algo/SubgraphExecutor | ❌ 未实现 | - | 需新增实现 |

### 2.3 逻辑控制执行器对比

逻辑控制执行器负责实现流程控制逻辑，包括循环和条件分支。以下是两个项目在该类别下的详细对比：

| 执行器名称 | Nebula-Graph | GraphDB | 实现状态 | 备注 |
|------------|--------------|---------|----------|------|
| StartExecutor | ✅ logic/StartExecutor | ✅ base.rs | 完整实现 | 功能对等 |
| LoopExecutor | ✅ logic/LoopExecutor | ✅ logic/loops.rs | 完整实现 | 功能对等 |
| SelectExecutor | ✅ logic/SelectExecutor | ❌ 未实现 | - | 条件分支控制 |
| PassThroughExecutor | ✅ logic/PassThroughExecutor | ❌ 未实现 | - | 直接透传结果 |
| ArgumentExecutor | ✅ logic/ArgumentExecutor | ❌ 未实现 | - | 参数传递 |

### 2.4 管理执行器对比

管理执行器负责处理数据库管理操作，包括空间管理、用户管理等。以下是两个项目在该类别下的详细对比：

| 执行器名称 | Nebula-Graph | GraphDB | 实现状态 | 备注 |
|------------|--------------|---------|----------|------|
| **空间管理** |
| CreateSpaceExecutor | ✅ admin/SpaceExecutor | ❌ 未实现 | - | 需新增实现 |
| DropSpaceExecutor | ✅ admin/SpaceExecutor | ❌ 未实现 | - | 需新增实现 |
| SwitchSpaceExecutor | ✅ admin/SwitchSpaceExecutor | ❌ 未实现 | - | 需新增实现 |
| DescSpaceExecutor | ✅ admin/SpaceExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowSpacesExecutor | ✅ admin/SpaceExecutor | ❌ 未实现 | - | 需新增实现 |
| **用户管理** |
| CreateUserExecutor | ✅ admin/CreateUserExecutor | ❌ 未实现 | - | 需新增实现 |
| DropUserExecutor | ✅ admin/DropUserExecutor | ❌ 未实现 | - | 需新增实现 |
| UpdateUserExecutor | ✅ admin/UpdateUserExecutor | ❌ 未实现 | - | 需新增实现 |
| GrantRoleExecutor | ✅ admin/GrantRoleExecutor | ❌ 未实现 | - | 需新增实现 |
| RevokeRoleExecutor | ✅ admin/RevokeRoleExecutor | ❌ 未实现 | - | 需新增实现 |
| DescribeUserExecutor | ✅ admin/DescribeUserExecutor | ❌ 未实现 | - | 需新增实现 |
| ListUsersExecutor | ✅ admin/ListUsersExecutor | ❌ 未实现 | - | 需新增实现 |
| ListRolesExecutor | ✅ admin/ListRolesExecutor | ❌ 未实现 | - | 需新增实现 |
| ListUserRolesExecutor | ✅ admin/ListUserRolesExecutor | ❌ 未实现 | - | 需新增实现 |
| ChangePasswordExecutor | ✅ admin/ChangePasswordExecutor | ❌ 未实现 | - | 需新增实现 |
| **系统监控** |
| ShowHostsExecutor | ✅ admin/ShowHostsExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowStatsExecutor | ✅ admin/ShowStatsExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowQueriesExecutor | ✅ admin/ShowQueriesExecutor | ❌ 未实现 | - | 需新增实现 |
| KillQueryExecutor | ✅ admin/KillQueryExecutor | ❌ 未实现 | - | 需新增实现 |
| **会话管理** |
| SessionExecutor | ✅ admin/SessionExecutor | ❌ 未实现 | - | 需新增实现 |
| SignInServiceExecutor | ✅ admin/SignInServiceExecutor | ❌ 未实现 | - | 需新增实现 |
| SignOutServiceExecutor | ✅ admin/SignOutServiceExecutor | ❌ 未实现 | - | 需新增实现 |
| **分片管理** |
| PartExecutor | ✅ admin/PartExecutor | ❌ 未实现 | - | 需新增实现 |
| **快照管理** |
| SnapshotExecutor | ✅ admin/SnapshotExecutor | ❌ 未实现 | - | 需新增实现 |
| CreateSnapshotExecutor | ✅ admin/SnapshotExecutor | ❌ 未实现 | - | 需新增实现 |
| DropSnapshotExecutor | ✅ admin/SnapshotExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowSnapshotsExecutor | ✅ admin/SnapshotExecutor | ❌ 未实现 | - | 需新增实现 |
| **配置管理** |
| ConfigExecutor | ✅ admin/ConfigExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowConfigsExecutor | ✅ admin/ConfigExecutor | ❌ 未实现 | - | 需新增实现 |
| SetConfigExecutor | ✅ admin/ConfigExecutor | ❌ 未实现 | - | 需新增实现 |
| GetConfigExecutor | ✅ admin/ConfigExecutor | ❌ 未实现 | - | 需新增实现 |
| **作业管理** |
| SubmitJobExecutor | ✅ admin/SubmitJobExecutor | ❌ 未实现 | - | 需新增实现 |
| **主机管理** |
| AddHostsExecutor | ✅ admin/AddHostsExecutor | ❌ 未实现 | - | 需新增实现 |
| DropHostsExecutor | ✅ admin/DropHostsExecutor | ❌ 未实现 | - | 需新增实现 |
| **元数据领导者** |
| ShowMetaLeaderExecutor | ✅ admin/ShowMetaLeaderExecutor | ❌ 未实现 | - | 需新增实现 |
| **服务客户端** |
| ShowServiceClientsExecutor | ✅ admin/ShowServiceClientsExecutor | ❌ 未实现 | - | 需新增实现 |
| **全文索引** |
| ShowFTIndexesExecutor | ✅ admin/ShowFTIndexesExecutor | ❌ 未实现 | - | 需新增实现 |
| **区域管理** |
| ZoneExecutor | ✅ admin/ZoneExecutor | ❌ 未实现 | - | 需新增实现 |
| AddHostsIntoZoneExecutor | ✅ admin/ZoneExecutor | ❌ 未实现 | - | 需新增实现 |
| DivideZoneExecutor | ✅ admin/ZoneExecutor | ❌ 未实现 | - | 需新增实现 |
| MergeZoneExecutor | ✅ admin/ZoneExecutor | ❌ 未实现 | - | 需新增实现 |
| RenameZoneExecutor | ✅ admin/ZoneExecutor | ❌ 未实现 | - | 需新增实现 |
| DropZoneExecutor | ✅ admin/ZoneExecutor | ❌ 未实现 | - | 需新增实现 |
| DescribeZoneExecutor | ✅ admin/ZoneExecutor | ❌ 未实现 | - | 需新增实现 |
| ListZonesExecutor | ✅ admin/ZoneExecutor | ❌ 未实现 | - | 需新增实现 |
| **监听器管理** |
| ListenerExecutor | ✅ admin/ListenerExecutor | ❌ 未实现 | - | 需新增实现 |
| AddListenerExecutor | ✅ admin/ListenerExecutor | ❌ 未实现 | - | 需新增实现 |
| RemoveListenerExecutor | ✅ admin/ListenerExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowListenerExecutor | ✅ admin/ListenerExecutor | ❌ 未实现 | - | 需新增实现 |
| **字符集与排序规则** |
| CharsetExecutor | ✅ admin/CharsetExecutor | ❌ 未实现 | - | 需新增实现 |

### 2.5 维护执行器对比

维护执行器负责处理数据库 Schema 维护操作，包括标签、边和索引的创建与修改。以下是两个项目在该类别下的详细对比：

| 执行器名称 | Nebula-Graph | GraphDB | 实现状态 | 备注 |
|------------|--------------|---------|----------|------|
| **标签管理** |
| TagExecutor | ✅ maintain/TagExecutor | ❌ 未实现 | - | 需新增实现 |
| CreateTagExecutor | ✅ maintain/TagExecutor | ❌ 未实现 | - | 需新增实现 |
| AlterTagExecutor | ✅ maintain/TagExecutor | ❌ 未实现 | - | 需新增实现 |
| DescTagExecutor | ✅ maintain/TagExecutor | ❌ 未实现 | - | 需新增实现 |
| DropTagExecutor | ✅ maintain/TagExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowTagsExecutor | ✅ maintain/TagExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowCreateTagExecutor | ✅ maintain/TagExecutor | ❌ 未实现 | - | 需新增实现 |
| **边管理** |
| EdgeExecutor | ✅ maintain/EdgeExecutor | ❌ 未实现 | - | 需新增实现 |
| CreateEdgeExecutor | ✅ maintain/EdgeExecutor | ❌ 未实现 | - | 需新增实现 |
| AlterEdgeExecutor | ✅ maintain/EdgeExecutor | ❌ 未实现 | - | 需新增实现 |
| DescEdgeExecutor | ✅ maintain/EdgeExecutor | ❌ 未实现 | - | 需新增实现 |
| DropEdgeExecutor | ✅ maintain/EdgeExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowEdgesExecutor | ✅ maintain/EdgeExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowCreateEdgeExecutor | ✅ maintain/EdgeExecutor | ❌ 未实现 | - | 需新增实现 |
| **标签索引管理** |
| TagIndexExecutor | ✅ maintain/TagIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| CreateTagIndexExecutor | ✅ maintain/TagIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| DropTagIndexExecutor | ✅ maintain/TagIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| DescTagIndexExecutor | ✅ maintain/TagIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowTagIndexesExecutor | ✅ maintain/TagIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowTagIndexStatusExecutor | ✅ maintain/TagIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowCreateTagIndexExecutor | ✅ maintain/TagIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| **边索引管理** |
| EdgeIndexExecutor | ✅ maintain/EdgeIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| CreateEdgeIndexExecutor | ✅ maintain/EdgeIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| DropEdgeIndexExecutor | ✅ maintain/EdgeIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| DescEdgeIndexExecutor | ✅ maintain/EdgeIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowEdgeIndexesExecutor | ✅ maintain/EdgeIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowEdgeIndexStatusExecutor | ✅ maintain/EdgeIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| ShowCreateEdgeIndexExecutor | ✅ maintain/EdgeIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| **全文索引管理** |
| FTIndexExecutor | ✅ maintain/FTIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| CreateFTIndexExecutor | ✅ maintain/FTIndexExecutor | ❌ 未实现 | - | 需新增实现 |
| DropFTIndexExecutor | ✅ maintain/FTIndexExecutor | ❌ 未实现 | - | 需新增实现 |

### 2.6 变更执行器对比

变更执行器负责处理数据修改操作，包括插入、更新和删除。以下是两个项目在该类别下的详细对比：

| 执行器名称 | Nebula-Graph | GraphDB | 实现状态 | 备注 |
|------------|--------------|---------|----------|------|
| **插入操作** |
| InsertExecutor | ✅ mutate/InsertExecutor | ✅ data_modification.rs | 基础实现 | 需增强批量插入 |
| InsertVerticesExecutor | ✅ mutate/InsertExecutor | ⚠️ 部分实现 | 基础实现 | 复用 InsertExecutor |
| InsertEdgesExecutor | ✅ mutate/InsertExecutor | ⚠️ 部分实现 | 基础实现 | 复用 InsertExecutor |
| **删除操作** |
| DeleteExecutor | ✅ mutate/DeleteExecutor | ❌ 未实现 | - | 需新增实现 |
| DeleteVerticesExecutor | ✅ mutate/DeleteExecutor | ❌ 未实现 | - | 需新增实现 |
| DeleteEdgesExecutor | ✅ mutate/DeleteExecutor | ❌ 未实现 | - | 需新增实现 |
| DeleteTagsExecutor | ✅ mutate/DeleteExecutor | ❌ 未实现 | - | 需新增实现 |
| **更新操作** |
| UpdateExecutor | ✅ mutate/UpdateExecutor | ❌ 未实现 | - | 需新增实现 |
| UpdateVertexExecutor | ✅ mutate/UpdateExecutor | ❌ 未实现 | - | 需新增实现 |
| UpdateEdgeExecutor | ✅ mutate/UpdateExecutor | ❌ 未实现 | - | 需新增实现 |

## 三、执行器实现状态统计

### 3.1 按功能类别统计

| 功能类别 | Nebula-Graph 数量 | GraphDB 已实现 | GraphDB 缺失 | 实现率 |
|----------|-------------------|----------------|--------------|--------|
| 查询执行器 | 35 | 22 | 13 | 62.9% |
| 算法执行器 | 8 | 3 | 5 | 37.5% |
| 逻辑控制执行器 | 5 | 2 | 3 | 40.0% |
| 管理执行器 | 40 | 0 | 40 | 0.0% |
| 维护执行器 | 22 | 0 | 22 | 0.0% |
| 变更执行器 | 5 | 1 | 4 | 20.0% |
| **总计** | **115** | **28** | **87** | **24.3%** |

### 3.2 按优先级分类的缺失执行器

**高优先级（核心查询功能）**
- GetNeighborsExecutor（完善实现）
- ScanEdgesExecutor
- ScanVerticesExecutor
- DataCollectExecutor
- UnionExecutor
- SelectExecutor
- DeleteExecutor / UpdateExecutor 系列

**中优先级（扩展查询功能）**
- BFSShortestPathExecutor
- CartesianProductExecutor
- SubgraphExecutor
- FulltextIndexScanExecutor
- UnionAllVersionVarExecutor
- PassThroughExecutor / ArgumentExecutor

**低优先级（管理维护功能）**
- 所有管理执行器（40 个）
- 所有维护执行器（22 个）

## 四、设计架构对比

### 4.1 执行器基类设计

**Nebula-Graph 设计**

Nebula-Graph 采用单继承体系，所有执行器继承自统一的 `Executor` 基类：

- 文件位置：[Executor.h](file:///d:/项目/database/graphDB/nebula-3.8.0/src/graph/executor/Executor.h)
- 核心方法：`execute()`、`open()`、`close()`
- 辅助方法：`finish()` 用于存储结果、`dependsOn()` 用于建立依赖关系

**GraphDB 设计**

GraphDB 采用 Trait 组合设计，执行器通过实现多个 Trait 来获得功能：

- 核心 Trait：`Executor<S: StorageEngine>`
- 可选 Trait：`HasStorage<S>`、`HasInput<S>`、`InputExecutor<S>`
- 基础结构：`BaseExecutor<S>`、`BaseResultProcessor<S>`

### 4.2 执行器创建机制

**Nebula-Graph 工厂方法**

在 [Executor.cpp](file:///d:/项目/database/graphDB/nebula-3.8.0/src/graph/executor/Executor.cpp) 中，通过静态方法 `makeExecutor()` 创建执行器：

- 使用对象池（ObjectPool）分配执行器实例
- 支持递归创建依赖的执行器
- 集成生命周期分析功能

**GraphDB 执行器工厂**

在 [factory.rs](file:///d:/项目/database/graphDB/src/query/executor/factory.rs) 中，通过 `ExecutorFactory` 创建执行器：

- 直接实例化执行器
- 支持计划节点到执行器的映射
- 集成递归检测和安全验证

### 4.3 执行结果处理

**Nebula-Graph 结果模型**

- 使用 `Value` 类型作为通用结果载体
- 通过 `DataSet` 结构表示关系型结果集
- 使用 `Iterator` 接口遍历结果

**GraphDB 结果模型**

- 使用 `ExecutionResult` 枚举表示多种结果类型
- 支持 `Values`、`Vertices`、`Edges`、`DataSet`、`Paths` 等
- 结果处理分散在多个模块中

## 五、迁移建议

### 5.1 优先实现清单

基于功能重要性和实现复杂度，建议按以下优先级实现缺失的执行器：

**第一阶段（核心查询支持）**
1. 完善 GetNeighborsExecutor 实现
2. 实现 ScanVerticesExecutor 和 ScanEdgesExecutor
3. 实现 UnionExecutor（集合操作）
4. 实现 DeleteExecutor 和 UpdateExecutor 系列

**第二阶段（扩展查询支持）**
1. 实现 DataCollectExecutor
2. 实现 SelectExecutor（条件分支）
3. 实现 BFSShortestPathExecutor
4. 实现 FulltextIndexScanExecutor

**第三阶段（管理功能支持）**
1. 实现空间管理执行器（CreateSpaceExecutor、SwitchSpaceExecutor 等）
2. 实现用户管理执行器
3. 实现维护执行器（标签、边、索引管理）

### 5.2 代码复用建议

对于管理类和维护类执行器，由于其逻辑相对简单且 Nebula-Graph 有成熟的实现参考，建议：

1. 直接参考 Nebula-Graph 的实现逻辑
2. 适配 GraphDB 的存储接口
3. 复用 GraphDB 现有的错误处理和结果模型

## 六、总结

通过对比分析可以发现，GraphDB 在查询执行器的实现上已经达到了 Nebula-Graph 约 60% 的覆盖率，核心的投影、过滤、聚合、排序、连接等操作已经完整实现。然而，在算法执行器、逻辑控制执行器方面仍有较大的提升空间，特别是管理类和维护类执行器目前完全未实现，这限制了 GraphDB 作为独立数据库系统的完整性。

从架构设计角度看，GraphDB 采用的 Trait 组合设计相比 Nebula-Graph 的单继承体系更加灵活，但当前存在的代码重复和设计不一致问题需要在后续迭代中逐步解决。
