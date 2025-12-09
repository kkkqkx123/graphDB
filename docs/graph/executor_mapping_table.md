# Executor 模块映射表

## 快速参考表

### 按优先级排序

#### ⭐⭐⭐ 第一优先级（必须迁移）

| NebulaGraph 文件 | 类名 | 新架构位置 | 描述 |
|---|---|---|---|
| `executor/Executor.h/cpp` | `Executor` | `src/query/executor/base.rs` | 基础执行器类 |
| `executor/StorageAccessExecutor.h/cpp` | `StorageAccessExecutor` | `src/query/executor/data_access.rs` | 存储访问基类 |
| `executor/query/GetNeighborsExecutor.h/cpp` | `GetNeighborsExecutor` | `src/query/executor/data_access.rs` | 获取相邻节点 |
| `executor/query/GetVerticesExecutor.h/cpp` | `GetVerticesExecutor` | `src/query/executor/data_access.rs` | 获取节点 |
| `executor/query/GetEdgesExecutor.h/cpp` | `GetEdgesExecutor` | `src/query/executor/data_access.rs` | 获取边 |
| `executor/query/FilterExecutor.h/cpp` | `FilterExecutor` | `src/query/executor/data_processing.rs` | WHERE/FILTER执行 |
| `executor/query/ExpandExecutor.h/cpp` | `ExpandExecutor` | `src/query/executor/data_processing.rs` | 路径扩展 |
| `executor/query/ExpandAllExecutor.h/cpp` | `ExpandAllExecutor` | `src/query/executor/data_processing.rs` | 返回所有路径 |
| `executor/query/TraverseExecutor.h/cpp` | `TraverseExecutor` | `src/query/executor/data_processing.rs` | 图遍历 |
| `executor/query/ProjectExecutor.h/cpp` | `ProjectExecutor` | `src/query/executor/result_processing.rs` | 列投影 |
| `executor/query/AggregateExecutor.h/cpp` | `AggregateExecutor` | `src/query/executor/result_processing.rs` | 聚合函数 |
| `executor/query/SortExecutor.h/cpp` | `SortExecutor` | `src/query/executor/result_processing.rs` | ORDER BY |
| `executor/query/LimitExecutor.h/cpp` | `LimitExecutor` | `src/query/executor/result_processing.rs` | LIMIT/OFFSET |
| `executor/mutate/InsertExecutor.h/cpp` | `InsertExecutor` | `src/query/executor/data_modification.rs` | 插入数据 |
| `executor/mutate/UpdateExecutor.h/cpp` | `UpdateExecutor` | `src/query/executor/data_modification.rs` | 更新数据 |
| `executor/mutate/DeleteExecutor.h/cpp` | `DeleteExecutor` | `src/query/executor/data_modification.rs` | 删除数据 |
| `executor/logic/StartExecutor.h/cpp` | `StartExecutor` | `src/query/executor/base.rs` | 查询起点 |
| `executor/algo/ShortestPathExecutor.h/cpp` | `ShortestPathExecutor` | `src/query/executor/data_processing.rs` | 最短路径 |

#### ⭐⭐ 第二优先级（重要迁移）

| NebulaGraph 文件 | 类名 | 新架构位置 | 描述 |
|---|---|---|---|
| `executor/query/JoinExecutor.h/cpp` | `JoinExecutor` | `src/query/executor/data_processing.rs` | INNER JOIN |
| `executor/query/LeftJoinExecutor.h/cpp` | `LeftJoinExecutor` | `src/query/executor/data_processing.rs` | LEFT OUTER JOIN |
| `executor/query/InnerJoinExecutor.h/cpp` | `InnerJoinExecutor` | `src/query/executor/data_processing.rs` | INNER JOIN |
| `executor/query/UnionExecutor.h/cpp` | `UnionExecutor` | `src/query/executor/data_processing.rs` | UNION（去重） |
| `executor/query/UnwindExecutor.h/cpp` | `UnwindExecutor` | `src/query/executor/data_processing.rs` | UNWIND |
| `executor/query/DedupExecutor.h/cpp` | `DedupExecutor` | `src/query/executor/result_processing.rs` | DISTINCT去重 |
| `executor/query/TopNExecutor.h/cpp` | `TopNExecutor` | `src/query/executor/result_processing.rs` | TOP N优化 |
| `executor/query/DataCollectExecutor.h/cpp` | `DataCollectExecutor` | `src/query/executor/result_processing.rs` | 收集结果 |
| `executor/query/IndexScanExecutor.h/cpp` | `IndexScanExecutor` | `src/query/executor/data_access.rs` | 索引扫描 |
| `executor/query/ScanVerticesExecutor.h/cpp` | `ScanVerticesExecutor` | `src/query/executor/data_access.rs` | 全表扫描节点 |
| `executor/query/ScanEdgesExecutor.h/cpp` | `ScanEdgesExecutor` | `src/query/executor/data_access.rs` | 全表扫描边 |
| `executor/query/AssignExecutor.h/cpp` | `AssignExecutor` | `src/query/executor/data_processing.rs` | 变量赋值 |
| `executor/query/IntersectExecutor.h/cpp` | `IntersectExecutor` | `src/query/executor/data_processing.rs` | INTERSECT |
| `executor/query/MinusExecutor.h/cpp` | `MinusExecutor` | `src/query/executor/data_processing.rs` | MINUS/EXCEPT |
| `executor/query/UnionAllVersionVarExecutor.h/cpp` | `UnionAllVersionVarExecutor` | `src/query/executor/data_processing.rs` | UNION ALL |
| `executor/query/AppendVerticesExecutor.h/cpp` | `AppendVerticesExecutor` | `src/query/executor/data_processing.rs` | 追加顶点 |
| `executor/query/GetPropExecutor.h/cpp` | `GetPropExecutor` | `src/query/executor/data_access.rs` | 获取属性 |
| `executor/logic/SelectExecutor.h/cpp` | `SelectExecutor` | `src/query/scheduler/` | 执行计划选择 |
| `executor/logic/LoopExecutor.h/cpp` | `LoopExecutor` | `src/query/executor/data_processing.rs` | 循环执行 |
| `executor/algo/AllPathsExecutor.h/cpp` | `AllPathsExecutor` | `src/query/executor/data_processing.rs` | 所有路径 |
| `executor/algo/SubgraphExecutor.h/cpp` | `SubgraphExecutor` | `src/query/executor/data_processing.rs` | 子图提取 |
| `executor/admin/SessionExecutor.h/cpp` | `SessionExecutor` | `src/api/session.rs` | 会话管理 |
| `executor/admin/SpaceExecutor.h/cpp` | `SpaceExecutor` | `src/api/space.rs` | 图空间管理 |
| `executor/maintain/TagIndexExecutor.h/cpp` | `TagIndexExecutor` | `src/index/tag_index.rs` | 标签索引管理 |
| `executor/maintain/EdgeIndexExecutor.h/cpp` | `EdgeIndexExecutor` | `src/index/edge_index.rs` | 边索引管理 |

#### ⭐ 第三优先级（可选/简化）

| NebulaGraph 文件 | 类名 | 新架构位置 | 描述 |
|---|---|---|---|
| `executor/query/ValueExecutor.h/cpp` | `ValueExecutor` | `src/query/executor/data_processing.rs` | 常量值 |
| `executor/query/SetExecutor.h/cpp` | `SetExecutor` | `src/query/executor/result_processing.rs` | SET语句 |
| `executor/query/SampleExecutor.h/cpp` | `SampleExecutor` | `src/query/executor/result_processing.rs` | 随机采样 |
| `executor/query/PatternApplyExecutor.h/cpp` | `PatternApplyExecutor` | `src/query/executor/data_processing.rs` | 模式匹配 |
| `executor/query/RollUpApplyExecutor.cpp` | `RollUpApplyExecutor` | `src/query/executor/data_processing.rs` | ROLLUP操作 |
| `executor/query/FulltextIndexScanExecutor.h/cpp` | `FulltextIndexScanExecutor` | `src/query/executor/data_access.rs` | 全文索引扫描 |
| `executor/logic/ArgumentExecutor.h/cpp` | `ArgumentExecutor` | `src/query/executor/base.rs` | 参数处理 |
| `executor/logic/PassThroughExecutor.h/cpp` | `PassThroughExecutor` | `src/query/executor/base.rs` | 直通执行器 |
| `executor/admin/ConfigExecutor.h/cpp` | `ConfigExecutor` | `src/api/config.rs` | 配置管理 |
| `executor/admin/ShowQueriesExecutor.h/cpp` | `ShowQueriesExecutor` | `src/api/show_queries.rs` | 显示查询 |
| `executor/admin/ShowStatsExecutor.h/cpp` | `ShowStatsExecutor` | `src/api/show_stats.rs` | 显示统计 |
| `executor/maintain/FTIndexExecutor.h/cpp` | `FTIndexExecutor` | `src/index/fulltext_index.rs` | 全文索引管理 |
| `executor/admin/CharsetExecutor.h/cpp` | `CharsetExecutor` | `src/api/charset.rs` | 字符集管理 |
| `executor/admin/CreateUserExecutor.h/cpp` | `CreateUserExecutor` | `src/api/user.rs` | 创建用户 |
| `executor/admin/DropUserExecutor.h/cpp` | `DropUserExecutor` | `src/api/user.rs` | 删除用户 |
| `executor/admin/UpdateUserExecutor.h/cpp` | `UpdateUserExecutor` | `src/api/user.rs` | 更新用户 |
| `executor/admin/ListUsersExecutor.h/cpp` | `ListUsersExecutor` | `src/api/user.rs` | 列表用户 |
| `executor/admin/DescribeUserExecutor.h/cpp` | `DescribeUserExecutor` | `src/api/user.rs` | 查看用户 |
| `executor/admin/ChangePasswordExecutor.h/cpp` | `ChangePasswordExecutor` | `src/api/security.rs` | 修改密码 |
| `executor/algo/BFSShortestPathExecutor.h/cpp` | `BFSShortestPathExecutor` | `src/query/executor/data_processing.rs` | BFS最短路径 |
| `executor/algo/MultiShortestPathExecutor.h/cpp` | `MultiShortestPathExecutor` | `src/query/executor/data_processing.rs` | 多源最短路径 |
| `executor/algo/BatchShortestPath.h/cpp` | `BatchShortestPath` | `src/query/executor/data_processing.rs` | 批量最短路径 |
| `executor/algo/CartesianProductExecutor.h/cpp` | `CartesianProductExecutor` | `src/query/executor/data_processing.rs` | 笛卡尔积 |

#### ❌ 不迁移（分布式特定）

| NebulaGraph 文件 | 类名 | 原因 |
|---|---|---|
| `executor/maintain/TagExecutor.h/cpp` | `TagExecutor` | 分布式元数据管理 |
| `executor/maintain/EdgeExecutor.h/cpp` | `EdgeExecutor` | 分布式元数据管理 |
| `executor/admin/ShowHostsExecutor.h/cpp` | `ShowHostsExecutor` | 分布式主机管理 |
| `executor/admin/AddHostsExecutor.h/cpp` | `AddHostsExecutor` | 分布式主机管理 |
| `executor/admin/DropHostsExecutor.h/cpp` | `DropHostsExecutor` | 分布式主机管理 |
| `executor/admin/ZoneExecutor.h/cpp` | `ZoneExecutor` | 分布式区域管理 |
| `executor/admin/PartExecutor.h/cpp` | `PartExecutor` | 分布式分区管理 |
| `executor/admin/ListenerExecutor.h/cpp` | `ListenerExecutor` | 分布式监听器 |
| `executor/admin/SnapshotExecutor.h/cpp` | `SnapshotExecutor` | 分布式快照 |
| `executor/admin/SubmitJobExecutor.h/cpp` | `SubmitJobExecutor` | 分布式任务 |
| `executor/admin/KillQueryExecutor.h/cpp` | `KillQueryExecutor` | 分布式查询控制 |
| `executor/admin/ShowMetaLeaderExecutor.h/cpp` | `ShowMetaLeaderExecutor` | 分布式Meta信息 |
| `executor/admin/SignInServiceExecutor.h/cpp` | `SignInServiceExecutor` | 分布式服务认证 |
| `executor/admin/SignOutServiceExecutor.h/cpp` | `SignOutServiceExecutor` | 分布式服务认证 |
| `executor/admin/ShowServiceClientsExecutor.h/cpp` | `ShowServiceClientsExecutor` | 分布式服务管理 |
| `executor/admin/GrantRoleExecutor.h/cpp` | `GrantRoleExecutor` | 分布式权限管理（可删除） |
| `executor/admin/RevokeRoleExecutor.h/cpp` | `RevokeRoleExecutor` | 分布式权限管理（可删除） |
| `executor/admin/ListUserRolesExecutor.h/cpp` | `ListUserRolesExecutor` | 分布式权限管理（可删除） |
| `executor/admin/ListRolesExecutor.h/cpp` | `ListRolesExecutor` | 分布式权限管理（可删除） |

---

## 按类别分类

### 数据访问执行器

**目标位置**: `src/query/executor/data_access.rs`

```
├── GetNeighborsExecutor         ⭐⭐⭐
├── GetVerticesExecutor          ⭐⭐⭐
├── GetEdgesExecutor             ⭐⭐⭐
├── GetPropExecutor              ⭐⭐
├── IndexScanExecutor            ⭐⭐
├── ScanVerticesExecutor         ⭐⭐
├── ScanEdgesExecutor            ⭐⭐
└── FulltextIndexScanExecutor    ⭐
```

### 数据处理执行器

**目标位置**: `src/query/executor/data_processing.rs`

```
├── FilterExecutor               ⭐⭐⭐
├── ExpandExecutor               ⭐⭐⭐
├── ExpandAllExecutor            ⭐⭐⭐
├── TraverseExecutor             ⭐⭐⭐
├── ShortestPathExecutor         ⭐⭐⭐
├── JoinExecutor                 ⭐⭐
├── LeftJoinExecutor             ⭐⭐
├── InnerJoinExecutor            ⭐⭐
├── UnionExecutor                ⭐⭐
├── UnionAllVersionVarExecutor   ⭐⭐
├── UnwindExecutor               ⭐⭐
├── IntersectExecutor            ⭐⭐
├── MinusExecutor                ⭐⭐
├── AppendVerticesExecutor       ⭐⭐
├── AssignExecutor               ⭐⭐
├── LoopExecutor                 ⭐⭐
├── AllPathsExecutor             ⭐⭐
├── SubgraphExecutor             ⭐⭐
├── BFSShortestPathExecutor      ⭐
├── MultiShortestPathExecutor    ⭐
├── BatchShortestPath            ⭐
├── CartesianProductExecutor     ⭐
├── ValueExecutor                ⭐
├── PatternApplyExecutor         ⭐
└── RollUpApplyExecutor          ⭐
```

### 结果处理执行器

**目标位置**: `src/query/executor/result_processing.rs`

```
├── ProjectExecutor              ⭐⭐⭐
├── AggregateExecutor            ⭐⭐⭐
├── SortExecutor                 ⭐⭐⭐
├── LimitExecutor                ⭐⭐⭐
├── DedupExecutor                ⭐⭐
├── TopNExecutor                 ⭐⭐
├── DataCollectExecutor          ⭐⭐
├── SetExecutor                  ⭐
└── SampleExecutor               ⭐
```

### 数据修改执行器

**目标位置**: `src/query/executor/data_modification.rs`

```
├── InsertExecutor               ⭐⭐⭐
├── UpdateExecutor               ⭐⭐⭐
└── DeleteExecutor               ⭐⭐⭐
```

### 基础和逻辑执行器

**目标位置**: `src/query/executor/base.rs`

```
├── Executor (base class)        ⭐⭐⭐
├── StartExecutor                ⭐⭐⭐
├── ArgumentExecutor             ⭐
└── PassThroughExecutor          ⭐
```

### 调度执行器

**目标位置**: `src/query/scheduler/`

```
└── SelectExecutor               ⭐⭐
```

### 索引执行器

**目标位置**: `src/index/`

```
├── TagIndexExecutor             ⭐⭐
├── EdgeIndexExecutor            ⭐⭐
└── FTIndexExecutor              ⭐
```

### API执行器

**目标位置**: `src/api/`

```
├── SessionExecutor              ⭐⭐
├── SpaceExecutor                ⭐⭐
├── ConfigExecutor               ⭐
├── ShowQueriesExecutor          ⭐
├── ShowStatsExecutor            ⭐
├── CharsetExecutor              ⭐
├── CreateUserExecutor           ⭐
├── DropUserExecutor             ⭐
├── UpdateUserExecutor           ⭐
├── ListUsersExecutor            ⭐
├── DescribeUserExecutor         ⭐
└── ChangePasswordExecutor       ⭐
```

---

## 统计信息

| 类别 | 总数 | ⭐⭐⭐ | ⭐⭐ | ⭐ | ❌ |
|---|---|---|---|---|---|
| 数据访问 | 8 | 3 | 4 | 1 | 0 |
| 数据处理 | 27 | 4 | 13 | 10 | 0 |
| 结果处理 | 9 | 4 | 3 | 2 | 0 |
| 数据修改 | 3 | 3 | 0 | 0 | 0 |
| 基础/逻辑 | 4 | 2 | 1 | 1 | 0 |
| 调度 | 1 | 0 | 1 | 0 | 0 |
| 索引 | 3 | 0 | 2 | 1 | 0 |
| API | 12 | 0 | 2 | 10 | 0 |
| **待删除** | **19** | - | - | - | **19** |
| **总计** | **86** | 16 | 26 | 25 | 19 |

**迁移统计**:
- 需要迁移: 67 个执行器（78%）
- 需要删除: 19 个执行器（22%）
- 第一优先级: 16 个（迁移总数的 24%）
- 第二优先级: 26 个（迁移总数的 39%）
- 第三优先级: 25 个（迁移总数的 37%）

