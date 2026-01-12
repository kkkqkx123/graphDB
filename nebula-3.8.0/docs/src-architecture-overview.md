# NebulaGraph 源码架构设计概述

## 目录结构说明

NebulaGraph 是一个开源的分布式图数据库，其源代码采用模块化设计，主要分为以下几个核心组件：

### 1. 客户端层 (src/clients)
- 包含与各服务通信的客户端实现
- 分为 graph、meta 和 storage 三个子模块，分别对应各个服务的客户端接口
- 提供了连接池、负载均衡等客户端功能

### 2. 公共库 (src/common)
- 包含整个项目共享的基础库和工具类
- 主要模块包括：
  - algorithm：算法实现
  - base：基础类型定义和宏
  - charset：字符集处理
  - conf：配置管理
  - context：上下文管理
  - datatypes：数据类型定义
  - expression：表达式解析和计算
  - fs：文件系统操作
  - function：内置函数
  - geo：地理信息处理
  - graph：图相关的基础数据结构
  - http：HTTP 客户端/服务器基础
  - id：ID 管理
  - log：日志系统
  - memory：内存管理
  - meta：元数据相关通用函数
  - network：网络相关工具
  - ssl：SSL/TLS 支持
  - stats：统计指标
  - thread：线程池和并发工具
  - thrift：Thrift 序列化/反序列化
  - time：时间处理
  - utils：通用工具函数

### 3. 图查询引擎 (src/graph)
- 负责处理图查询请求的核心模块
- 主要组件包括：
  - context：执行上下文
  - executor：执行器，负责实际执行计划
  - gc：垃圾收集
  - optimizer：查询优化器
  - planner：查询计划生成器
  - scheduler：调度器
  - service：服务入口点
  - session：会话管理
  - stats：统计信息
  - validator：验证器，校验语法和权限
  - visitor：遍历器模式实现

### 4. 存储引擎 (src/storage)
- 负责实际数据存储的核心模块
- 主要组件包括：
  - admin：管理命令处理
  - context：存储上下文
  - exec：执行器
  - http：HTTP 接口
  - index：索引实现
  - kv：键值对操作
  - mutate：修改操作（插入、更新、删除）
  - query：查询操作
  - stats：统计信息
  - transaction：事务处理
- 核心类：StorageServer、GraphStorageServiceHandler、BaseProcessor

### 5. 元数据管理 (src/meta)
- 负责集群元数据管理
- 主要组件包括：
  - http：HTTP 接口
  - processors：各种元数据处理逻辑
  - stats：统计信息
  - upgrade：升级相关逻辑
- 核心类：MetaServiceHandler、ActiveHostsMan、MetaVersionMan

### 6. KV 存储层 (src/kvstore)
- 基于 RocksDB 的持久化存储层
- 提供分布式 KV 存储能力
- 主要组件包括：
  - listener：监听器
  - plugins：插件系统
  - raftex：RAFT 协议实现
  - stats：统计信息
  - wal：预写日志
- 核心类：NebulaStore、RocksEngine、Part、KVEngine

### 7. 服务进程 (src/daemons)
- 包含各个服务的主进程实现
- 文件说明：
  - GraphDaemon.cpp：图服务主进程
  - MetaDaemon.cpp：元数据服务主进程
  - StorageDaemon.cpp：存储服务主进程
  - StandAloneDaemon.cpp：单机版启动入口

### 8. 查询解析器 (src/parser)
- 提供 GQL (Graph Query Language) 解析功能
- 使用 Flex/Bison 实现词法和语法分析
- 处理各种类型的 SQL 语句，包括：
  - 管理语句 (AdminSentences)
  - 维护语句 (MaintainSentences)
  - 变更语句 (MutateSentences)
  - 遍历语句 (TraverseSentences)
  - 用户管理语句 (UserSentences)

### 9. Web 服务 (src/webservice)
- 提供 HTTP 接口用于监控和管理
- 包括健康检查、性能统计、运行时参数调整等功能
- 组件包括路由、状态处理器、标志管理器等

### 10. 编解码 (src/codec)
- 数据编解码相关功能
- 负责数据在网络传输或持久化时的格式转换

### 11. 控制台 (src/console)
- 提供命令行交互界面
- 用于与 NebulaGraph 集群进行交互

### 12. 工具 (src/tools)
- 包含各种辅助工具
- 如数据导入导出工具等

### 13. 版本信息 (src/version)
- 版本号和构建信息管理

### 14. 接口定义 (src/interface)
- 包含 Thrift 接口定义文件
- 定义了各服务间的通信协议

## 架构特点

1. **对称分布式架构**：计算和存储分离，支持水平扩展
2. **多副本一致性**：使用 RAFT 协议保证强一致性
3. **模块化设计**：各组件职责明确，便于维护和扩展
4. **OpenCypher 兼容**：支持类 SQL 的图查询语言
5. **丰富的内置函数**：提供多种图分析算法

## 关键依赖关系

- graph 服务依赖 common、parser、kvstore、storage、meta
- storage 服务依赖 common、kvstore、meta
- meta 服务依赖 common、kvstore
- clients 层调用 graph、storage、meta 服务