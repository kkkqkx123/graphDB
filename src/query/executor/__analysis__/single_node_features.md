# GraphDB 单节点部署功能规划

## 一、项目定位

GraphDB 是一个面向个人使用和小型应用的单节点图数据库解决方案，移除分布式功能，专注于：

- 轻量级部署
- 高性能查询
- 最小化外部依赖
- 单可执行文件分发

## 二、单节点不必要功能分析

### 2.1 完全跳过的功能（约 45 个执行器）

#### Zone 管理（8 个）- 完全跳过
| 执行器 | Nebula-Graph 位置 | 跳过原因 |
|--------|-------------------|----------|
| ZoneExecutor | admin/ZoneExecutor | 单节点无区域概念 |
| AddHostsIntoZoneExecutor | admin/ZoneExecutor | 无主机概念 |
| DivideZoneExecutor | admin/ZoneExecutor | 无分布式分片 |
| MergeZoneExecutor | admin/ZoneExecutor | 无分布式分片 |
| RenameZoneExecutor | admin/ZoneExecutor | 无区域重命名需求 |
| DropZoneExecutor | admin/ZoneExecutor | 无区域删除 |
| DescribeZoneExecutor | admin/ZoneExecutor | 无区域描述 |
| ListZonesExecutor | admin/ZoneExecutor | 无区域列表 |

#### 主机管理（3 个）- 完全跳过
| 执行器 | 跳过原因 |
|--------|----------|
| AddHostsExecutor | 单节点无需添加主机 |
| DropHostsExecutor | 单节点无需移除主机 |

#### 分片管理（1 个）- 完全跳过
| 执行器 | 跳过原因 |
|--------|----------|
| PartExecutor | 单节点无数据分片 |

#### 分布式监听器（4 个）- 完全跳过
| 执行器 | 跳过原因 |
|--------|----------|
| ListenerExecutor | 无分布式监听需求 |
| AddListenerExecutor | 无分布式监听 |
| RemoveListenerExecutor | 无分布式监听 |
| ShowListenerExecutor | 无分布式监听 |

#### 服务客户端（1 个）- 完全跳过
| 执行器 | 跳过原因 |
|--------|----------|
| ShowServiceClientsExecutor | 分布式场景专用 |

#### Meta 领导者（1 个）- 完全跳过
| 执行器 | 跳过原因 |
|--------|----------|
| ShowMetaLeaderExecutor | 分布式元数据管理 |

#### 多用户权限管理（13 个）- 完全跳过
| 执行器 | 跳过原因 |
|--------|----------|
| CreateUserExecutor | 单节点单人使用 |
| DropUserExecutor | 单节点单人使用 |
| UpdateUserExecutor | 单节点单人使用 |
| GrantRoleExecutor | 无角色权限需求 |
| RevokeRoleExecutor | 无角色权限需求 |
| DescribeUserExecutor | 无多用户需求 |
| ListUsersExecutor | 无多用户需求 |
| ListRolesExecutor | 无角色需求 |
| ListUserRolesExecutor | 无用户角色需求 |
| SignInServiceExecutor | 本地无需远程认证 |
| SignOutServiceExecutor | 本地无需远程认证 |
| SessionExecutor | 简化实现 |
| CharsetExecutor | 可预设固定值 |

#### 作业调度（1 个）- 完全跳过
| 执行器 | 跳过原因 |
|--------|----------|
| SubmitJobExecutor | 无分布式作业 |

#### 复杂查询管理（2 个）- 可简化
| 执行器 | 建议 |
|--------|------|
| ShowQueriesExecutor | 跳过或简化 |
| KillQueryExecutor | 通过操作系统管理 |

### 2.2 可简化的功能（约 10 个执行器）

| 执行器 | 简化方案 |
|--------|----------|
| ShowHostsExecutor | 仅显示本地主机信息 |
| ShowStatsExecutor | 仅显示本地统计 |
| ConfigExecutor | 读取配置文件，非运行时管理 |
| SetConfigExecutor | 修改配置文件 |
| GetConfigExecutor | 读取配置文件 |
| CreateSnapshot | 保留，实现本地快照 |
| DropSnapshot | 保留，删除本地快照 |
| ShowSnapshots | 跳过或简化 |
| ShowTagIndexStatus | 跳过 |
| ShowEdgeIndexStatus | 跳过 |

## 三、必须实现的功能

### 3.1 核心功能清单

#### 空间管理（4 个）
| 执行器 | 功能描述 | 优先级 |
|--------|----------|--------|
| CreateSpaceExecutor | 创建图空间 | 高 |
| DropSpaceExecutor | 删除图空间 | 高 |
| DescSpaceExecutor | 查看图空间详情 | 中 |
| ShowSpacesExecutor | 列出所有图空间 | 中 |

#### 标签管理（5 个）
| 执行器 | 功能描述 | 优先级 |
|--------|----------|--------|
| CreateTagExecutor | 创建标签 | 高 |
| AlterTagExecutor | 修改标签 | 高 |
| DescTagExecutor | 查看标签详情 | 中 |
| DropTagExecutor | 删除标签 | 高 |
| ShowTagsExecutor | 列出标签 | 中 |

#### 边类型管理（5 个）
| 执行器 | 功能描述 | 优先级 |
|--------|----------|--------|
| CreateEdgeExecutor | 创建边类型 | 高 |
| AlterEdgeExecutor | 修改边类型 | 高 |
| DescEdgeExecutor | 查看边类型详情 | 中 |
| DropEdgeExecutor | 删除边类型 | 高 |
| ShowEdgesExecutor | 列出边类型 | 中 |

#### 索引管理（10 个）
| 执行器 | 功能描述 | 优先级 |
|--------|----------|--------|
| CreateTagIndexExecutor | 创建标签索引 | 高 |
| DropTagIndexExecutor | 删除标签索引 | 高 |
| DescTagIndexExecutor | 查看标签索引详情 | 中 |
| ShowTagIndexesExecutor | 列出标签索引 | 中 |
| CreateEdgeIndexExecutor | 创建边索引 | 高 |
| DropEdgeIndexExecutor | 删除边索引 | 高 |
| DescEdgeIndexExecutor | 查看边索引详情 | 中 |
| ShowEdgeIndexesExecutor | 列出边索引 | 中 |
| RebuildTagIndexExecutor | 重建标签索引 | 中 |
| RebuildEdgeIndexExecutor | 重建边索引 | 中 |

#### 数据变更（5 个）
| 执行器 | 功能描述 | 优先级 |
|--------|----------|--------|
| InsertExecutor | 插入数据 | 高 |
| DeleteExecutor | 删除数据 | 高 |
| UpdateExecutor | 更新数据 | 高 |
| UpsertExecutor | 插入或更新 | 中 |

#### 安全功能（1 个）
| 执行器 | 功能描述 | 优先级 |
|--------|----------|--------|
| ChangePasswordExecutor | 修改密码 | 中 |

### 3.3 实现优先级排序

```
第一阶段（核心功能）
├── 空间管理（4个）
├── 标签管理（5个）
├── 边类型管理（5个）
├── 索引管理（8个）
└── 数据变更（3个：Insert、Delete、Update）

第二阶段（扩展功能）
├── 索引重建（2个）
├── UpsertExecutor（1个）
└── ChangePasswordExecutor（1个）

第三阶段（可选功能）
├── 快照管理（2-3个）
└── 配置管理（3个）
```

## 四、实现目录结构

```
src/query/executor/
├── admin/                          # 管理执行器
│   ├── mod.rs
│   ├── space/                      # 空间管理
│   │   ├── mod.rs
│   │   ├── create_space.rs
│   │   ├── drop_space.rs
│   │   ├── desc_space.rs
│   │   └── show_spaces.rs
│   ├── tag/                        # 标签管理
│   │   ├── mod.rs
│   │   ├── create_tag.rs
│   │   ├── alter_tag.rs
│   │   ├── desc_tag.rs
│   │   ├── drop_tag.rs
│   │   └── show_tags.rs
│   ├── edge/                       # 边类型管理
│   │   ├── mod.rs
│   │   ├── create_edge.rs
│   │   ├── alter_edge.rs
│   │   ├── desc_edge.rs
│   │   ├── drop_edge.rs
│   │   └── show_edges.rs
│   └── index/                      # 索引管理
│       ├── mod.rs
│       ├── tag_index.rs
│       └── edge_index.rs
├── mutate/                         # 数据变更执行器
│   ├── mod.rs
│   ├── insert.rs
│   ├── delete.rs
│   ├── update.rs
│   └── upsert.rs
└── maintain/                       # 维护执行器（可选）
    ├── mod.rs
    └── snapshot.rs
```

## 五、架构设计原则

### 5.1 统一的执行器基类

所有管理执行器继承统一的基础结构：

```rust
/// 基础管理执行器
#[derive(Clone, Debug)]
pub struct BaseAdminExecutor<S: StorageEngine> {
    pub base: BaseExecutor<S>,
    pub admin_type: AdminType,
}

/// 管理操作类型
pub enum AdminType {
    Space(SpaceOperation),
    Tag(TagOperation),
    Edge(EdgeOperation),
    Index(IndexOperation),
}
```

### 5.2 简化的结果模型

管理执行器返回统一的结果格式：

```rust
/// 管理操作结果
pub enum AdminResult {
    /// 操作成功，无返回数据
    Success,
    /// 操作成功，返回数据列表
    DataSet(DataSet),
    /// 操作失败
    Error(String),
}
```

### 5.3 单节点存储集成

管理执行器直接操作存储引擎，无需分布式协调：

```rust
impl<S: StorageEngine> CreateSpaceExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, space_info: SpaceInfo) -> Self {
        Self {
            base: BaseExecutor::new(id, "CreateSpaceExecutor".to_string(), storage),
            space_info,
        }
    }
}
```

## 六、总结

| 类别 | 原始数量 | 单节点实现 | 跳过/简化 |
|------|----------|------------|-----------|
| 空间管理 | 4 | 4 | 0 |
| 标签管理 | 5 | 5 | 0 |
| 边类型管理 | 5 | 5 | 0 |
| 索引管理 | 10 | 8 | 2（跳过状态查询） |
| 数据变更 | 5 | 4 | 1（可选） |
| 安全功能 | 1 | 1 | 0 |
| Zone 管理 | 8 | 0 | 8 |
| 主机管理 | 3 | 0 | 3 |
| 分片管理 | 1 | 0 | 1 |
| 用户权限 | 13 | 0 | 13 |
| 其他管理 | 10 | 3 | 7 |
| **总计** | **65** | **30** | **35** |

单节点部署可将管理类执行器的实现工作量减少约 **54%**，同时保持数据库的核心功能完整性。
