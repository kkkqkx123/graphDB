# NebulaGraph 架构依赖关系分析

## 概述

本文档分析了 NebulaGraph 项目中不同子模块之间的依赖关系，并识别了其架构的层次划分。NebulaGraph 采用分层架构设计，各模块职责明确，依赖关系清晰。

## 模块依赖分析

### 1. Common 模块 (src/common)
- **职责**: 提供整个项目的基础库和通用工具
- **内容**:
  - 基础数据类型 (datatypes)
  - 表达式解析 (expression)
  - 通用函数 (function)
  - 时间处理 (time)
  - 网络工具 (network)
  - 线程池 (thread)
  - 文件系统操作 (fs)
  - 地理信息处理 (geo)
  - 编解码工具 (codec) 等
- **依赖性**: 不依赖其他业务模块，被所有其他模块依赖

### 2. KVStore 模块 (src/kvstore)
- **职责**: 提供分布式键值存储能力
- **内容**:
  - 基于 RocksDB 的存储引擎 (RocksEngine)
  - RAFT 协议实现 (raftex)
  - 存储分区管理 (Part)
  - 存储迭代器 (KVIterator)
  - 存储快照管理 (NebulaSnapshotManager)
- **依赖性**: 依赖 Common 模块，被 Meta 和 Storage 模块依赖

### 3. Meta 模块 (src/meta)
- **职责**: 管理集群元数据和拓扑信息
- **内容**:
  - 元数据服务处理器 (MetaServiceHandler)
  - Schema 管理
  - 空间和分区管理
  - 用户和权限管理
  - 集群配置管理
  - 升级管理 (upgrade)
- **依赖性**: 依赖 Common 和 KVStore 模块，被 Graph 和 Storage 模块依赖

### 4. Storage 模块 (src/storage)
- **职责**: 负责实际的图数据存储和检索
- **内容**:
  - 存储服务处理器 (GraphStorageServiceHandler)
  - 数据修改操作 (mutate)
  - 数据查询操作 (query)
  - 索引管理 (index)
  - 管理任务 (admin)
  - 事务处理 (transaction)
  - 存储执行器 (exec)
- **依赖性**: 依赖 Common、KVStore 和 Meta 模块，被 Graph 模块依赖

### 5. Graph 模块 (src/graph)
- **职责**: 图查询引擎，处理 GQL 查询请求
- **内容**:
  - 服务入口 (service)
  - 查询计划器 (planner)
  - 查询优化器 (optimizer)
  - 查询执行器 (executor)
  - 查询调度器 (scheduler)
  - 语法验证器 (validator)
  - 表达式分析器 (visitor)
  - 会话管理 (session)
  - 认证模块 (auth)
- **依赖性**: 依赖 Common、Meta 和 Storage 模块

### 6. 客户端模块 (src/clients)
- **职责**: 提供各服务的客户端实现
- **内容**:
  - 图服务客户端 (graph)
  - 元数据服务客户端 (meta)
  - 存储服务客户端 (storage)
- **依赖性**: 依赖 Common、Interface 以及相应服务模块

### 7. 解析器模块 (src/parser)
- **职责**: GQL 语句解析
- **依赖性**: 依赖 Common 模块

### 8. 服务守护进程 (src/daemons)
- **职责**: 各服务的主入口点
- **内容**:
  - Graph 服务守护进程 (graphd)
  - Meta 服务守护进程 (metad)
  - Storage 服务守护进程 (storaged)
  - 独立模式守护进程 (standalone)
- **依赖性**: 依赖项目中几乎所有模块

## 架构层次划分

NebulaGraph 的架构遵循严格的分层设计，依赖关系呈有向无环图结构：

```
    ┌─────────────────┐
    │   Interface     │  (Thrift 接口定义)
    │  (顶层共享)     │
    └─────────┬───────┘
              │
    ┌─────────▼─────────┐
    │      Common       │  (基础库)
    │  (最底层共享)     │
    └─────────┬───────┬─┘
              │       │
              │       ▼
              │  ┌──────────┐
              │  │  KVStore │  (分布式存储)
              │  │ (中间层) │
              │  └─────┬────┘
              ▼         │
        ┌─────────┐     │
        │   Meta  │     │  (元数据管理)
        │(中间层) │     │
        └─────┬───┘     │
              │         │
        ┌─────▼─────────▼──┐
        │     Storage      │  (数据存储)
        │   (中间层)       │
        └─────────┬────────┘
                  │
        ┌─────────▼────────┐
        │      Graph       │  (查询处理)
        │   (中间层)       │
        └─────────┬────────┘
                  │
        ┌─────────▼────────┐
        │     Clients      │  (客户端)
        │   (高层)         │
        └─────────┬────────┘
                  │
        ┌─────────▼────────┐
        │     Daemons      │  (可执行程序)
        │   (顶层)         │
        └──────────────────┘
```

## 关键依赖关系总结

1. **Common** 模块是基础，被所有其他模块直接或间接依赖
2. **KVStore** 依赖 Common，为 Meta 和 Storage 提供存储支持
3. **Meta** 依赖 Common 和 KVStore，为整个系统提供元数据管理
4. **Storage** 依赖 Common、KVStore 和 Meta，是实际的数据存储层
5. **Graph** 依赖 Common、Meta 和 Storage，是查询处理核心
6. **Daemons** 依赖大部分模块，是可执行文件的入口

## 设计原则

1. **分层架构**: 严格按照依赖方向，高层模块可依赖低层模块，但低层不依赖高层
2. **模块化设计**: 各模块职责单一，接口清晰
3. **服务化**: 每个主要功能模块都可以作为独立服务运行
4. **可扩展性**: 通过清晰的接口定义，便于功能扩展

## 结论

NebulaGraph 采用良好的分层架构设计，模块间依赖关系清晰，符合高内聚低耦合的设计原则。这种架构既保证了系统的稳定性，也为未来的功能扩展提供了良好的基础。