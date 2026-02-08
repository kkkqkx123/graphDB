# Nebula-Graph 与 GraphDB 执行器对比分析

## 概述

本文档详细对比 Nebula-Graph 3.8.0 和 GraphDB 的执行器实现，分析 GraphDB 缺失的执行器及其对单节点服务的必要性。

---

## 一、执行器统计

### 1.1 总体统计

| 项目 | Nebula-Graph | GraphDB | 差异 |
|------|--------------|---------|------|
| 总执行器数 | 100 | 70 | -30 |
| Admin 执行器 | 50 | 15 | -35 |
| Algo 执行器 | 4 | 3 | -1 |
| Logic 执行器 | 5 | 5 | 0 |
| Maintain 执行器 | 16 | 8 | -8 |
| Mutate 执行器 | 3 | 3 | 0 |
| Query 执行器 | 22 | 36 | +14 |

**说明**: GraphDB 将一些执行器合并或重新组织，因此数量差异不能完全反映功能差异。

---

## 二、Nebula-Graph 执行器完整列表

### 2.1 Admin 执行器（50个）

| 执行器 | 功能 | GraphDB 对应 | 状态 |
|--------|------|--------------|------|
| AddHostsExecutor | 添加主机 | - | ❌ 分布式功能 |
| DropHostsExecutor | 删除主机 | - | ❌ 分布式功能 |
| ShowHostsExecutor | 显示主机 | - | ❌ 分布式功能 |
| CreateUserExecutor | 创建用户 | CreateUserExecutor | ✅ 已实现 |
| DropUserExecutor | 删除用户 | DropUserExecutor | ✅ 已实现 |
| UpdateUserExecutor | 更新用户 | AlterUserExecutor | ✅ 已实现 |
| DescribeUserExecutor | 描述用户 | - | ⚠️ 未实现 |
| ChangePasswordExecutor | 修改密码 | ChangePasswordExecutor | ✅ 已实现 |
| GrantRoleExecutor | 授予角色 | - | ⚠️ 未实现 |
| RevokeRoleExecutor | 撤销角色 | - | ⚠️ 未实现 |
| ListUsersExecutor | 列出用户 | - | ⚠️ 未实现 |
| ListRolesExecutor | 列出角色 | - | ⚠️ 未实现 |
| ListUserRolesExecutor | 列出用户角色 | - | ⚠️ 未实现 |
| ShowCharsetExecutor | 显示字符集 | - | ❌ 非核心功能 |
| ShowCollationExecutor | 显示排序规则 | - | ❌ 非核心功能 |
| ConfigBaseExecutor | 配置基类 | - | ❌ 分布式功能 |
| KillQueryExecutor | 终止查询 | - | ⚠️ 未实现 |
| ShowQueriesExecutor | 显示查询 | - | ⚠️ 未实现 |
| ShowStatsExecutor | 显示统计 | - | ⚠️ 未实现 |
| ShowServiceClientsExecutor | 显示服务客户端 | - | ❌ 分布式功能 |
| ShowMetaLeaderExecutor | 显示 Meta 领导者 | - | ❌ 分布式功能 |
| SignInServiceExecutor | 登录服务 | - | ❌ 分布式功能 |
| SignOutServiceExecutor | 登出服务 | - | ❌ 分布式功能 |
| ShowSessionsExecutor | 显示会话 | - | ⚠️ 部分实现 |
| KillSessionExecutor | 终止会话 | - | ⚠️ 部分实现 |
| UpdateSessionExecutor | 更新会话 | - | ⚠️ 部分实现 |
| AddListenerExecutor | 添加监听器 | - | ❌ 分布式功能 |
| RemoveListenerExecutor | 移除监听器 | - | ❌ 分布式功能 |
| ShowListenerExecutor | 显示监听器 | - | ❌ 分布式功能 |
| ShowPartsExecutor | 显示分区 | - | ❌ 分布式功能 |
| CreateSnapshotExecutor | 创建快照 | - | ❌ 分布式功能 |
| DropSnapshotExecutor | 删除快照 | - | ❌ 分布式功能 |
| ShowSnapshotsExecutor | 显示快照 | - | ❌ 分布式功能 |
| CreateSpaceExecutor | 创建空间 | CreateSpaceExecutor | ✅ 已实现 |
| CreateSpaceAsExecutor | 创建空间（复制） | - | ⚠️ 未实现 |
| DescSpaceExecutor | 描述空间 | DescSpaceExecutor | ✅ 已实现 |
| DropSpaceExecutor | 删除空间 | DropSpaceExecutor | ✅ 已实现 |
| ClearSpaceExecutor | 清空空间 | - | ⚠️ 未实现 |
| ShowSpacesExecutor | 显示空间 | ShowSpacesExecutor | ✅ 已实现 |
| ShowCreateSpaceExecutor | 显示创建空间语句 | - | ⚠️ 未实现 |
| AlterSpaceExecutor | 修改空间 | - | ⚠️ 未实现 |
| SwitchSpaceExecutor | 切换空间 | - | ⚠️ 未实现 |
| SubmitJobExecutor | 提交作业 | - | ❌ 分布式功能 |
| MergeZoneExecutor | 合并分区 | - | ❌ 分布式功能 |
| RenameZoneExecutor | 重命名分区 | - | ❌ 分布式功能 |
| DropZoneExecutor | 删除分区 | - | ❌ 分布式功能 |
| DivideZoneExecutor | 划分分区 | - | ❌ 分布式功能 |
| DescribeZoneExecutor | 描述分区 | - | ❌ 分布式功能 |
| AddHostsIntoZoneExecutor | 添加主机到分区 | - | ❌ 分布式功能 |
| ListZonesExecutor | 列出分区 | - | ❌ 分布式功能 |

### 2.2 Algo 执行器（4个）

| 执行器 | 功能 | GraphDB 对应 | 状态 |
|--------|------|--------------|------|
| ShortestPathExecutor | 最短路径 | ShortestPathExecutor | ✅ 已实现 |
| BFSShortestPathExecutor | BFS 最短路径 | BFSShortestExecutor | ✅ 已实现 |
| MultiShortestPathExecutor | 多源最短路径 | MultiShortestPathExecutor | ✅ 已实现 |
| CartesianProductExecutor | 笛卡尔积 | CrossJoinExecutor | ✅ 已实现 |

### 2.3 Logic 执行器（5个）

| 执行器 | 功能 | GraphDB 对应 | 状态 |
|--------|------|--------------|------|
| ArgumentExecutor | 参数传递 | ArgumentExecutor | ✅ 已实现 |
| PassThroughExecutor | 直通 | PassThroughExecutor | ✅ 已实现 |
| LoopExecutor | 循环 | LoopExecutor | ✅ 已实现 |
| SelectExecutor | 选择 | SelectExecutor | ✅ 已实现 |
| StartExecutor | 起始节点 | StartExecutor | ✅ 已实现 |

### 2.4 Maintain 执行器（16个）

| 执行器 | 功能 | GraphDB 对应 | 状态 |
|--------|------|--------------|------|
| CreateTagExecutor | 创建标签 | CreateTagExecutor | ✅ 已实现 |
| DescTagExecutor | 描述标签 | DescTagExecutor | ✅ 已实现 |
| DropTagExecutor | 删除标签 | DropTagExecutor | ✅ 已实现 |
| ShowTagsExecutor | 显示标签 | ShowTagsExecutor | ✅ 已实现 |
| ShowCreateTagExecutor | 显示创建标签语句 | - | ⚠️ 未实现 |
| AlterTagExecutor | 修改标签 | AlterTagExecutor | ✅ 已实现 |
| CreateEdgeExecutor | 创建边类型 | CreateEdgeExecutor | ✅ 已实现 |
| DescEdgeExecutor | 描述边类型 | DescEdgeExecutor | ✅ 已实现 |
| DropEdgeExecutor | 删除边类型 | DropEdgeExecutor | ✅ 已实现 |
| ShowEdgesExecutor | 显示边类型 | ShowEdgesExecutor | ✅ 已实现 |
| ShowCreateEdgeExecutor | 显示创建边类型语句 | - | ⚠️ 未实现 |
| AlterEdgeExecutor | 修改边类型 | AlterEdgeExecutor | ✅ 已实现 |
| CreateTagIndexExecutor | 创建标签索引 | CreateTagIndexExecutor | ✅ 已实现 |
| DropTagIndexExecutor | 删除标签索引 | DropTagIndexExecutor | ✅ 已实现 |
| DescTagIndexExecutor | 描述标签索引 | DescTagIndexExecutor | ✅ 已实现 |
| ShowCreateTagIndexExecutor | 显示创建标签索引语句 | - | ⚠️ 未实现 |
| ShowTagIndexesExecutor | 显示标签索引 | ShowTagIndexesExecutor | ✅ 已实现 |
| ShowTagIndexStatusExecutor | 显示标签索引状态 | - | ⚠️ 未实现 |
| CreateEdgeIndexExecutor | 创建边索引 | CreateEdgeIndexExecutor | ✅ 已实现 |
| DropEdgeIndexExecutor | 删除边索引 | DropEdgeIndexExecutor | ✅ 已实现 |
| DescEdgeIndexExecutor | 描述边索引 | DescEdgeIndexExecutor | ✅ 已实现 |
| ShowCreateEdgeIndexExecutor | 显示创建边索引语句 | - | ⚠️ 未实现 |
| ShowEdgeIndexesExecutor | 显示边索引 | ShowEdgeIndexesExecutor | ✅ 已实现 |
| ShowEdgeIndexStatusExecutor | 显示边索引状态 | - | ⚠️ 未实现 |
| ShowFTIndexesExecutor | 显示全文索引 | - | ❌ 未实现 |
| CreateFTIndexExecutor | 创建全文索引 | - | ❌ 未实现 |
| DropFTIndexExecutor | 删除全文索引 | - | ❌ 未实现 |

### 2.5 Mutate 执行器（3个）

| 执行器 | 功能 | GraphDB 对应 | 状态 |
|--------|------|--------------|------|
| InsertExecutor | 插入数据 | InsertExecutor | ✅ 已实现 |
| UpdateExecutor | 更新数据 | UpdateExecutor | ✅ 已实现 |
| DeleteExecutor | 删除数据 | DeleteExecutor | ✅ 已实现 |

### 2.6 Query 执行器（22个）

| 执行器 | 功能 | GraphDB 对应 | 状态 |
|--------|------|--------------|------|
| ScanVerticesExecutor | 扫描顶点 | ScanVerticesExecutor | ✅ 已实现 |
| ScanEdgesExecutor | 扫描边 | ScanEdgesExecutor | ✅ 已实现 |
| GetVerticesExecutor | 获取顶点 | GetVerticesExecutor | ✅ 已实现 |
| GetEdgesExecutor | 获取边 | GetEdgesExecutor | ✅ 已实现 |
| GetNeighborsExecutor | 获取邻居 | GetNeighborsExecutor | ✅ 已实现 |
| GetPropExecutor | 获取属性 | GetPropExecutor | ✅ 已实现 |
| IndexScanExecutor | 索引扫描 | IndexScanExecutor | ✅ 已实现 |
| FulltextIndexScanExecutor | 全文索引扫描 | FulltextIndexScanExecutor | ⚠️ 部分实现 |
| ExpandExecutor | 扩展 | ExpandExecutor | ✅ 已实现 |
| ExpandAllExecutor | 全扩展 | ExpandAllExecutor | ✅ 已实现 |
| TraverseExecutor | 遍历 | TraverseExecutor | ✅ 已实现 |
| FilterExecutor | 过滤 | FilterExecutor | ✅ 已实现 |
| ProjectExecutor | 投影 | ProjectExecutor | ✅ 已实现 |
| AggregateExecutor | 聚合 | AggregateExecutor | ✅ 已实现 |
| SortExecutor | 排序 | SortExecutor | ✅ 已实现 |
| LimitExecutor | 限制 | LimitExecutor | ✅ 已实现 |
| TopNExecutor | TopN | TopNExecutor | ✅ 已实现 |
| JoinExecutor | 连接基类 | BaseJoinExecutor | ✅ 已实现 |
| InnerJoinExecutor | 内连接 | InnerJoinExecutor | ✅ 已实现 |
| LeftJoinExecutor | 左连接 | LeftJoinExecutor | ✅ 已实现 |
| UnionExecutor | 并集 | UnionExecutor | ✅ 已实现 |
| UnionAllVersionVarExecutor | 并集所有版本变量 | UnionAllExecutor | ✅ 已实现 |
| IntersectExecutor | 交集 | IntersectExecutor | ✅ 已实现 |
| MinusExecutor | 差集 | MinusExecutor | ✅ 已实现 |
| DedupExecutor | 去重 | DedupExecutor | ✅ 已实现 |
| AppendVerticesExecutor | 追加顶点 | AppendVerticesExecutor | ✅ 已实现 |
| AssignExecutor | 赋值 | AssignExecutor | ✅ 已实现 |
| UnwindExecutor | 展开 | UnwindExecutor | ✅ 已实现 |
| PatternApplyExecutor | 模式应用 | PatternApplyExecutor | ✅ 已实现 |
| RollUpApplyExecutor | 滚动应用 | RollUpApplyExecutor | ✅ 已实现 |
| SampleExecutor | 采样 | SampleExecutor | ✅ 已实现 |
| SetExecutor | 集合操作 | SetExecutor | ✅ 已实现 |
| DataCollectExecutor | 数据收集 | DataCollectExecutor | ✅ 已实现 |
| ValueExecutor | 值执行器 | - | ⚠️ 未实现 |

---

## 三、GraphDB 执行器完整列表

### 3.1 Admin 执行器（15个）

| 执行器 | 功能 | Nebula-Graph 对应 |
|--------|------|------------------|
| CreateSpaceExecutor | 创建空间 | CreateSpaceExecutor |
| DescSpaceExecutor | 描述空间 | DescSpaceExecutor |
| DropSpaceExecutor | 删除空间 | DropSpaceExecutor |
| ShowSpacesExecutor | 显示空间 | ShowSpacesExecutor |
| CreateTagExecutor | 创建标签 | CreateTagExecutor |
| DescTagExecutor | 描述标签 | DescTagExecutor |
| DropTagExecutor | 删除标签 | DropTagExecutor |
| ShowTagsExecutor | 显示标签 | ShowTagsExecutor |
| AlterTagExecutor | 修改标签 | AlterTagExecutor |
| CreateEdgeExecutor | 创建边类型 | CreateEdgeExecutor |
| DescEdgeExecutor | 描述边类型 | DescEdgeExecutor |
| DropEdgeExecutor | 删除边类型 | DropEdgeExecutor |
| ShowEdgesExecutor | 显示边类型 | ShowEdgesExecutor |
| AlterEdgeExecutor | 修改边类型 | AlterEdgeExecutor |
| CreateUserExecutor | 创建用户 | CreateUserExecutor |
| DropUserExecutor | 删除用户 | DropUserExecutor |
| AlterUserExecutor | 更新用户 | UpdateUserExecutor |
| ChangePasswordExecutor | 修改密码 | ChangePasswordExecutor |
| CreateTagIndexExecutor | 创建标签索引 | CreateTagIndexExecutor |
| DropTagIndexExecutor | 删除标签索引 | DropTagIndexExecutor |
| DescTagIndexExecutor | 描述标签索引 | DescTagIndexExecutor |
| ShowTagIndexesExecutor | 显示标签索引 | ShowTagIndexesExecutor |
| RebuildTagIndexExecutor | 重建标签索引 | - |
| CreateEdgeIndexExecutor | 创建边索引 | CreateEdgeIndexExecutor |
| DropEdgeIndexExecutor | 删除边索引 | DropEdgeIndexExecutor |
| DescEdgeIndexExecutor | 描述边索引 | DescEdgeIndexExecutor |
| ShowEdgeIndexesExecutor | 显示边索引 | ShowEdgeIndexesExecutor |
| RebuildEdgeIndexExecutor | 重建边索引 | - |

### 3.2 Data Access 执行器（8个）

| 执行器 | 功能 | Nebula-Graph 对应 |
|--------|------|------------------|
| GetVerticesExecutor | 获取顶点 | GetVerticesExecutor |
| GetEdgesExecutor | 获取边 | GetEdgesExecutor |
| ScanEdgesExecutor | 扫描边 | ScanEdgesExecutor |
| GetNeighborsExecutor | 获取邻居 | GetNeighborsExecutor |
| GetPropExecutor | 获取属性 | GetPropExecutor |
| IndexScanExecutor | 索引扫描 | IndexScanExecutor |
| AllPathsExecutor | 所有路径 | - |
| ScanVerticesExecutor | 扫描顶点 | ScanVerticesExecutor |

### 3.3 Data Modification 执行器（5个）

| 执行器 | 功能 | Nebula-Graph 对应 |
|--------|------|------------------|
| InsertExecutor | 插入数据 | InsertExecutor |
| UpdateExecutor | 更新数据 | UpdateExecutor |
| DeleteExecutor | 删除数据 | DeleteExecutor |
| CreateIndexExecutor | 创建索引 | - |
| DropIndexExecutor | 删除索引 | - |

### 3.4 Data Processing 执行器（约20个）

| 执行器 | 功能 | Nebula-Graph 对应 |
|--------|------|------------------|
| ExpandExecutor | 扩展 | ExpandExecutor |
| ExpandAllExecutor | 全扩展 | ExpandAllExecutor |
| TraverseExecutor | 遍历 | TraverseExecutor |
| ShortestPathExecutor | 最短路径 | ShortestPathExecutor |
| BFSShortestExecutor | BFS 最短路径 | BFSShortestPathExecutor |
| MultiShortestPathExecutor | 多源最短路径 | MultiShortestPathExecutor |
| InnerJoinExecutor | 内连接 | InnerJoinExecutor |
| LeftJoinExecutor | 左连接 | LeftJoinExecutor |
| RightJoinExecutor | 右连接 | - |
| FullOuterJoinExecutor | 全外连接 | - |
| CrossJoinExecutor | 笛卡尔积 | CartesianProductExecutor |
| UnionExecutor | 并集 | UnionExecutor |
| UnionAllExecutor | 并集所有 | UnionAllVersionVarExecutor |
| IntersectExecutor | 交集 | IntersectExecutor |
| MinusExecutor | 差集 | MinusExecutor |
| SetExecutor | 集合操作 | SetExecutor |

### 3.5 Result Processing 执行器（约15个）

| 执行器 | 功能 | Nebula-Graph 对应 |
|--------|------|------------------|
| FilterExecutor | 过滤 | FilterExecutor |
| ProjectExecutor | 投影 | ProjectExecutor |
| AggregateExecutor | 聚合 | AggregateExecutor |
| GroupByExecutor | 分组 | - |
| HavingExecutor | 分组过滤 | - |
| SortExecutor | 排序 | SortExecutor |
| LimitExecutor | 限制 | LimitExecutor |
| TopNExecutor | TopN | TopNExecutor |
| DedupExecutor | 去重 | DedupExecutor |
| SampleExecutor | 采样 | SampleExecutor |
| AppendVerticesExecutor | 追加顶点 | AppendVerticesExecutor |
| AssignExecutor | 赋值 | AssignExecutor |
| UnwindExecutor | 展开 | UnwindExecutor |
| PatternApplyExecutor | 模式应用 | PatternApplyExecutor |
| RollUpApplyExecutor | 滚动应用 | RollUpApplyExecutor |

### 3.6 Logic 执行器（5个）

| 执行器 | 功能 | Nebula-Graph 对应 |
|--------|------|------------------|
| ArgumentExecutor | 参数传递 | ArgumentExecutor |
| PassThroughExecutor | 直通 | PassThroughExecutor |
| LoopExecutor | 循环 | LoopExecutor |
| WhileLoopExecutor | While 循环 | - |
| ForLoopExecutor | For 循环 | - |
| SelectExecutor | 选择 | SelectExecutor |

### 3.7 Special Executors（3个）

| 执行器 | 功能 | Nebula-Graph 对应 |
|--------|------|------------------|
| ArgumentExecutor | 参数传递 | ArgumentExecutor |
| PassThroughExecutor | 直通 | PassThroughExecutor |
| DataCollectExecutor | 数据收集 | DataCollectExecutor |

---

## 四、缺失执行器分析

### 4.1 完全缺失的执行器（按类别）

#### 4.1.1 分布式相关执行器（20个）

| 执行器 | 功能 | 单节点必要性 | 优先级 |
|--------|------|--------------|--------|
| AddHostsExecutor | 添加主机 | ❌ 不需要 | - |
| DropHostsExecutor | 删除主机 | ❌ 不需要 | - |
| ShowHostsExecutor | 显示主机 | ❌ 不需要 | - |
| ShowPartsExecutor | 显示分区 | ❌ 不需要 | - |
| MergeZoneExecutor | 合并分区 | ❌ 不需要 | - |
| RenameZoneExecutor | 重命名分区 | ❌ 不需要 | - |
| DropZoneExecutor | 删除分区 | ❌ 不需要 | - |
| DivideZoneExecutor | 划分分区 | ❌ 不需要 | - |
| DescribeZoneExecutor | 描述分区 | ❌ 不需要 | - |
| AddHostsIntoZoneExecutor | 添加主机到分区 | ❌ 不需要 | - |
| ListZonesExecutor | 列出分区 | ❌ 不需要 | - |
| CreateSnapshotExecutor | 创建快照 | ❌ 不需要 | - |
| DropSnapshotExecutor | 删除快照 | ❌ 不需要 | - |
| ShowSnapshotsExecutor | 显示快照 | ❌ 不需要 | - |
| SubmitJobExecutor | 提交作业 | ❌ 不需要 | - |
| ShowServiceClientsExecutor | 显示服务客户端 | ❌ 不需要 | - |
| ShowMetaLeaderExecutor | 显示 Meta 领导者 | ❌ 不需要 | - |
| SignInServiceExecutor | 登录服务 | ❌ 不需要 | - |
| SignOutServiceExecutor | 登出服务 | - | ❌ 不需要 |
| AddListenerExecutor | 添加监听器 | ❌ 不需要 | - |
| RemoveListenerExecutor | 移除监听器 | ❌ 不需要 | - |
| ShowListenerExecutor | 显示监听器 | ❌ 不需要 | - |

**结论**: 这些执行器完全依赖分布式架构，单节点场景下不需要。

#### 4.1.2 非核心功能执行器（3个）

| 执行器 | 功能 | 单节点必要性 | 优先级 |
|--------|------|--------------|--------|
| ShowCharsetExecutor | 显示字符集 | ⚠️ 低 | 低 |
| ShowCollationExecutor | 显示排序规则 | ⚠️ 低 | 低 |
| ConfigBaseExecutor | 配置基类 | ⚠️ 低 | 低 |

**结论**: 这些执行器功能非核心，可以后续根据需求添加。

#### 4.1.3 全文索引相关执行器（4个）

| 执行器 | 功能 | 单节点必要性 | 优先级 |
|--------|------|--------------|--------|
| ShowFTIndexesExecutor | 显示全文索引 | ⚠️ 中 | 中 |
| CreateFTIndexExecutor | 创建全文索引 | ⚠️ 中 | 中 |
| DropFTIndexExecutor | 删除全文索引 | ⚠️ 中 | 中 |
| FulltextIndexScanExecutor | 全文索引扫描 | ⚠️ 中 | 中 |

**结论**: 全文索引是重要功能，但非核心，可以后续实现。

### 4.2 部分缺失的执行器（按类别）

#### 4.2.1 用户和权限管理执行器（6个）

| 执行器 | 功能 | 单节点必要性 | 优先级 |
|--------|------|--------------|--------|
| DescribeUserExecutor | 描述用户 | ⚠️ 中 | 中 |
| GrantRoleExecutor | 授予角色 | ⚠️ 高 | 高 |
| RevokeRoleExecutor | 撤销角色 | ⚠️ 高 | 高 |
| ListUsersExecutor | 列出用户 | ⚠️ 中 | 中 |
| ListRolesExecutor | 列出角色 | ⚠️ 中 | 中 |
| ListUserRolesExecutor | 列出用户角色 | ⚠️ 中 | 中 |

**结论**: 权限管理是重要功能，建议实现。

#### 4.2.2 查询和会话管理执行器（6个）

| 执行器 | 功能 | 单节点必要性 | 优先级 |
|--------|------|--------------|--------|
| KillQueryExecutor | 终止查询 | ⚠️ 高 | 高 |
| ShowQueriesExecutor | 显示查询 | ⚠️ 高 | 高 |
| ShowStatsExecutor | 显示统计 | ⚠️ 高 | 高 |
| ShowSessionsExecutor | 显示会话 | ⚠️ 中 | 中 |
| KillSessionExecutor | 终止会话 | ⚠️ 中 | 中 |
| UpdateSessionExecutor | 更新会话 | ⚠️ 低 | 低 |

**结论**: 查询管理和统计信息对单节点也很重要，建议实现。

#### 4.2.3 空间管理执行器（4个）

| 执行器 | 功能 | 单节点必要性 | 优先级 |
|--------|------|--------------|--------|
| CreateSpaceAsExecutor | 创建空间（复制） | ⚠️ 低 | 低 |
| ClearSpaceExecutor | 清空空间 | ⚠️ 中 | 中 |
| ShowCreateSpaceExecutor | 显示创建空间语句 | ⚠️ 低 | 低 |
| AlterSpaceExecutor | 修改空间 | ⚠️ 中 | 中 |
| SwitchSpaceExecutor | 切换空间 | ⚠️ 高 | 高 |

**结论**: 切换空间是重要功能，建议实现。

#### 4.2.4 Schema 管理执行器（4个）

| 执行器 | 功能 | 单节点必要性 | 优先级 |
|--------|------|--------------|--------|
| ShowCreateTagExecutor | 显示创建标签语句 | ⚠️ 低 | 低 |
| ShowCreateEdgeExecutor | 显示创建边类型语句 | ⚠️ 低 | 低 |
| ShowCreateTagIndexExecutor | 显示创建标签索引语句 | ⚠️ 低 | 低 |
| ShowCreateEdgeIndexExecutor | 显示创建边索引语句 | ⚠️ 低 | 低 |

**结论**: 这些是辅助功能，优先级较低。

#### 4.2.5 索引状态执行器（2个）

| 执行器 | 功能 | 单节点必要性 | 优先级 |
|--------|------|--------------|--------|
| ShowTagIndexStatusExecutor | 显示标签索引状态 | ⚠️ 中 | 中 |
| ShowEdgeIndexStatusExecutor | 显示边索引状态 | ⚠️ 中 | 中 |

**结论**: 索引状态监控是重要功能，建议实现。

#### 4.2.6 查询执行器（1个）

| 执行器 | 功能 | 单节点必要性 | 优先级 |
|--------|------|--------------|--------|
| ValueExecutor | 值执行器 | ⚠️ 低 | 低 |

**结论**: 值执行器是辅助功能，优先级较低。

---

## 五、单节点服务必要执行器评估

### 5.1 高优先级执行器（建议实现）

| 执行器 | 功能 | 必要性说明 |
|--------|------|-----------|
| GrantRoleExecutor | 授予角色 | 权限管理是数据库安全的基础 |
| RevokeRoleExecutor | 撤销角色 | 权限管理是数据库安全的基础 |
| KillQueryExecutor | 终止查询 | 防止长时间运行的查询占用资源 |
| ShowQueriesExecutor | 显示查询 | 监控和管理正在运行的查询 |
| ShowStatsExecutor | 显示统计 | 性能监控和优化的重要工具 |
| SwitchSpaceExecutor | 切换空间 | 多空间管理是图数据库的基本功能 |

**实现建议**: 这些执行器对单节点服务的可用性、安全性和可维护性至关重要，建议优先实现。

### 5.2 中优先级执行器（建议实现）

| 执行器 | 功能 | 必要性说明 |
|--------|------|-----------|
| DescribeUserExecutor | 描述用户 | 用户管理的辅助功能 |
| ListUsersExecutor | 列出用户 | 用户管理的辅助功能 |
| ListRolesExecutor | 列出角色 | 权限管理的辅助功能 |
| ListUserRolesExecutor | 列出用户角色 | 权限管理的辅助功能 |
| ShowSessionsExecutor | 显示会话 | 会话管理的基础功能 |
| KillSessionExecutor | 终止会话 | 会话管理的基础功能 |
| ClearSpaceExecutor | 清空空间 | 空间管理的重要功能 |
| AlterSpaceExecutor | 修改空间 | 空间管理的重要功能 |
| ShowTagIndexStatusExecutor | 显示标签索引状态 | 索引管理的重要功能 |
| ShowEdgeIndexStatusExecutor | 显示边索引状态 | 索引管理的重要功能 |
| ShowFTIndexesExecutor | 显示全文索引 | 全文索引的基础功能 |
| CreateFTIndexExecutor | 创建全文索引 | 全文索引的基础功能 |
| DropFTIndexExecutor | 删除全文索引 | 全文索引的基础功能 |
| FulltextIndexScanExecutor | 全文索引扫描 | 全文索引的基础功能 |

**实现建议**: 这些执行器对单节点服务的功能完整性很重要，建议在中期实现。

### 5.3 低优先级执行器（可选实现）

| 执行器 | 功能 | 必要性说明 |
|--------|------|-----------|
| ShowCharsetExecutor | 显示字符集 | 非核心功能，国际化需求低 |
| ShowCollationExecutor | 显示排序规则 | 非核心功能，国际化需求低 |
| ConfigBaseExecutor | 配置基类 | 配置管理可通过其他方式实现 |
| CreateSpaceAsExecutor | 创建空间（复制） | 非常用功能 |
| ShowCreateSpaceExecutor | 显示创建空间语句 | 辅助功能，可通过其他方式实现 |
| ShowCreateTagExecutor | 显示创建标签语句 | 辅助功能，可通过其他方式实现 |
| ShowCreateEdgeExecutor | 显示创建边类型语句 | 辅助功能，可通过其他方式实现 |
| ShowCreateTagIndexExecutor | 显示创建标签索引语句 | 辅助功能，可通过其他方式实现 |
| ShowCreateEdgeIndexExecutor | 显示创建边索引语句 | 辅助功能，可通过其他方式实现 |
| UpdateSessionExecutor | 更新会话 | 非核心功能 |
| ValueExecutor | 值执行器 | 辅助功能 |

**实现建议**: 这些执行器对单节点服务的核心功能影响较小，可以根据实际需求选择性实现。

---

## 六、GraphDB 独有执行器

### 6.1 新增执行器（相比 Nebula-Graph）

| 执行器 | 功能 | 说明 |
|--------|------|------|
| GroupByExecutor | 分组 | 将聚合和分组分离，更灵活 |
| HavingExecutor | 分组过滤 | 将聚合和分组过滤分离，更灵活 |
| RightJoinExecutor | 右连接 | 补充连接类型 |
| FullOuterJoinExecutor | 全外连接 | 补充连接类型 |
| WhileLoopExecutor | While 循环 | 更明确的循环类型 |
| ForLoopExecutor | For 循环 | 更明确的循环类型 |
| AllPathsExecutor | 所有路径 | 图算法扩展 |
| RebuildTagIndexExecutor | 重建标签索引 | 索引维护功能 |
| RebuildEdgeIndexExecutor | 重建边索引 | 索引维护功能 |
| HashLeftJoinExecutor | 哈希左连接 | 性能优化 |
| HashInnerJoinExecutor | 哈希内连接 | 性能优化 |

**说明**: 这些执行器是 GraphDB 根据单节点场景的需求和性能优化而新增的，提供了更丰富的功能和更好的性能。

---

## 七、实现建议

### 7.1 短期实现（1-2个月）

**高优先级执行器**:
1. GrantRoleExecutor / RevokeRoleExecutor - 权限管理
2. KillQueryExecutor / ShowQueriesExecutor - 查询管理
3. ShowStatsExecutor - 统计信息
4. SwitchSpaceExecutor - 空间切换

**预期收益**:
- 提升数据库安全性和可管理性
- 提供更好的查询监控能力
- 改善用户体验

### 7.2 中期实现（3-6个月）

**中优先级执行器**:
1. 用户和权限管理辅助功能
2. 会话管理功能
3. 空间管理完善
4. 索引状态监控
5. 全文索引基础功能

**预期收益**:
- 完善数据库功能
- 提升运维便利性
- 增强查询能力

### 7.3 长期实现（6-12个月）

**低优先级执行器**:
1. 国际化支持（字符集、排序规则）
2. 辅助功能（显示创建语句等）
3. 高级特性

**预期收益**:
- 提升产品完整性
- 满足特殊需求
- 增强竞争力

---

## 八、总结

### 8.1 执行器对比总结

| 类别 | Nebula-Graph | GraphDB | 缺失数量 | 必要缺失 |
|------|--------------|---------|----------|----------|
| Admin | 50 | 15 | 35 | 6 |
| Algo | 4 | 3 | 1 | 0 |
| Logic | 5 | 5 | 0 | 0 |
| Maintain | 16 | 8 | 8 | 2 |
| Mutate | 3 | 3 | 0 | 0 |
| Query | 22 | 36 | -14 | 1 |
| **总计** | **100** | **70** | **30** | **9** |

### 8.2 关键发现

1. **分布式功能**: GraphDB 完全移除了 20 个分布式相关执行器，这是合理的架构简化。

2. **核心功能**: GraphDB 保留了所有核心查询和数据处理功能，并在此基础上进行了扩展。

3. **管理功能**: GraphDB 简化了管理功能，但保留了最基本的空间、标签、边类型和索引管理。

4. **新增功能**: GraphDB 新增了一些执行器，提供了更丰富的连接类型、循环类型和性能优化。

5. **必要缺失**: 有 9 个执行器对单节点服务是必要的，建议优先实现。

### 8.3 架构优势

**GraphDB 相比 Nebula-Graph 的优势**:
1. 更简洁的架构，专注于单节点场景
2. 更好的性能优化（批量操作、哈希连接等）
3. 更灵活的模块化设计
4. 更丰富的连接类型和循环类型
5. 更清晰的代码组织

**需要改进的地方**:
1. 权限管理功能需要完善
2. 查询管理和监控需要加强
3. 全文索引功能需要实现
4. 一些辅助功能可以补充

---

## 九、附录

### 9.1 执行器分类索引

#### 9.1.1 按功能分类

- **数据访问**: GetVerticesExecutor, GetEdgesExecutor, ScanEdgesExecutor, GetNeighborsExecutor, GetPropExecutor, IndexScanExecutor, ScanVerticesExecutor
- **数据修改**: InsertExecutor, UpdateExecutor, DeleteExecutor
- **数据处理**: ExpandExecutor, ExpandAllExecutor, TraverseExecutor, ShortestPathExecutor, BFSShortestExecutor, MultiShortestPathExecutor, InnerJoinExecutor, LeftJoinExecutor, RightJoinExecutor, FullOuterJoinExecutor, CrossJoinExecutor, UnionExecutor, UnionAllExecutor, IntersectExecutor, MinusExecutor
- **结果处理**: FilterExecutor, ProjectExecutor, AggregateExecutor, GroupByExecutor, HavingExecutor, SortExecutor, LimitExecutor, TopNExecutor, DedupExecutor, SampleExecutor, AppendVerticesExecutor, AssignExecutor, UnwindExecutor, PatternApplyExecutor, RollUpApplyExecutor
- **逻辑控制**: ArgumentExecutor, PassThroughExecutor, LoopExecutor, WhileLoopExecutor, ForLoopExecutor, SelectExecutor
- **管理操作**: CreateSpaceExecutor, DescSpaceExecutor, DropSpaceExecutor, ShowSpacesExecutor, CreateTagExecutor, DescTagExecutor, DropTagExecutor, ShowTagsExecutor, AlterTagExecutor, CreateEdgeExecutor, DescEdgeExecutor, DropEdgeExecutor, ShowEdgesExecutor, AlterEdgeExecutor, CreateUserExecutor, DropUserExecutor, AlterUserExecutor, ChangePasswordExecutor, CreateTagIndexExecutor, DropTagIndexExecutor, DescTagIndexExecutor, ShowTagIndexesExecutor, RebuildTagIndexExecutor, CreateEdgeIndexExecutor, DropEdgeIndexExecutor, DescEdgeIndexExecutor, ShowEdgeIndexesExecutor, RebuildEdgeIndexExecutor

#### 9.1.2 按优先级分类

- **高优先级（建议实现）**: GrantRoleExecutor, RevokeRoleExecutor, KillQueryExecutor, ShowQueriesExecutor, ShowStatsExecutor, SwitchSpaceExecutor
- **中优先级（建议实现）**: DescribeUserExecutor, ListUsersExecutor, ListRolesExecutor, ListUserRolesExecutor, ShowSessionsExecutor, KillSessionExecutor, ClearSpaceExecutor, AlterSpaceExecutor, ShowTagIndexStatusExecutor, ShowEdgeIndexStatusExecutor, ShowFTIndexesExecutor, CreateFTIndexExecutor, DropFTIndexExecutor, FulltextIndexScanExecutor
- **低优先级（可选实现）**: ShowCharsetExecutor, ShowCollationExecutor, ConfigBaseExecutor, CreateSpaceAsExecutor, ShowCreateSpaceExecutor, ShowCreateTagExecutor, ShowCreateEdgeExecutor, ShowCreateTagIndexExecutor, ShowCreateEdgeIndexExecutor, UpdateSessionExecutor, ValueExecutor
- **不需要（分布式功能）**: AddHostsExecutor, DropHostsExecutor, ShowHostsExecutor, ShowPartsExecutor, MergeZoneExecutor, RenameZoneExecutor, DropZoneExecutor, DivideZoneExecutor, DescribeZoneExecutor, AddHostsIntoZoneExecutor, ListZonesExecutor, CreateSnapshotExecutor, DropSnapshotExecutor, ShowSnapshotsExecutor, SubmitJobExecutor, ShowServiceClientsExecutor, ShowMetaLeaderExecutor, SignInServiceExecutor, SignOutServiceExecutor, AddListenerExecutor, RemoveListenerExecutor, ShowListenerExecutor

### 9.2 参考文档

- Nebula-Graph 执行器文档: https://docs.nebula-graph.io/3.8.0/
- GraphDB 架构文档: [nebula_executor_architecture_comparison.md](file:///d:/项目/database/graphDB/src/query/executor/__analysis__/nebula_executor_architecture_comparison.md)
