# GraphDB E2E 测试设计方案

## 1. 概述

### 1.1 测试目标

E2E（端到端）测试旨在验证 GraphDB 作为完整系统的功能正确性，模拟真实用户场景，确保从客户端请求到数据存储的全链路正常工作。

### 1.2 测试范围

- **完整查询流程**：解析 → 验证 → 计划 → 优化 → 执行 → 返回结果
- **多模块协作**：API 层、查询引擎、存储引擎、索引系统的协同工作
- **数据一致性**：DDL、DML、DQL、DCL 操作的数据正确性
- **边界场景**：错误处理、并发访问、资源限制等

### 1.3 与集成测试的区别

| 维度 | 集成测试 | E2E 测试 |
|------|----------|----------|
| 测试粒度 | 模块间接口 | 完整用户场景 |
| 测试目标 | 验证模块协作 | 验证业务流程 |
| 数据准备 | 模拟数据 | 真实业务数据 |
| 执行环境 | 测试环境 | 接近生产环境 |

---

## 2. 测试架构

### 2.1 测试分层

```
┌─────────────────────────────────────────────────────────────┐
│                     E2E 测试层                               │
│  - 完整业务场景测试                                           │
│  - 用户工作流测试                                             │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                   集成测试层（已有）                          │
│  - integration_storage: 存储层集成测试                        │
│  - integration_core: 核心类型与表达式测试                      │
│  - integration_query: 查询引擎组件测试                         │
│  - integration_ddl: 数据定义语言测试                           │
│  - integration_dml: 数据操作语言测试                           │
│  - integration_dql: 数据查询语言测试                           │
│  - integration_dcl: 数据控制语言测试                           │
│  - integration_api: API 模块测试                              │
│  - integration_index: 索引系统测试                             │
│  - integration_management: 管理语句测试                        │
│  - integration_logging: 日志系统测试                           │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                    单元测试层                                │
│  - 各模块内部函数测试                                         │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 E2E 测试目录结构

```
tests/e2e/
├── common/                    # E2E 测试共享工具
│   ├── mod.rs                 # 测试基础设施
│   ├── scenarios.rs           # 场景定义
│   ├── data_generators.rs     # 数据生成器
│   └── assertions.rs          # 断言工具
├── scenarios/                 # 业务场景测试
│   ├── mod.rs
│   ├── social_network.rs      # 社交网络场景
│   ├── e_commerce.rs          # 电商场景
│   ├── knowledge_graph.rs     # 知识图谱场景
│   └── recommendation.rs      # 推荐系统场景
├── workflows/                 # 工作流测试
│   ├── mod.rs
│   ├── schema_evolution.rs    # 模式演进工作流
│   ├── data_migration.rs      # 数据迁移工作流
│   └── backup_restore.rs      # 备份恢复工作流
├── performance/               # 性能场景测试
│   ├── mod.rs
│   ├── bulk_insert.rs         # 批量插入
│   ├── concurrent_queries.rs  # 并发查询
│   └── large_graph.rs         # 大图处理
└── regression/                # 回归测试
    ├── mod.rs
    └── bug_fixes.rs           # Bug 修复验证
```

---

## 3. 测试场景设计

### 3.1 场景一：社交网络应用

#### 场景描述
模拟一个完整的社交网络应用，包括用户管理、好友关系、动态发布、消息传递等核心功能。

#### 测试数据模型

```
顶点类型：
- Person: {id, name, age, city, created_at}
- Post: {id, content, created_at, likes}
- Comment: {id, content, created_at}
- Group: {id, name, description, created_at}

边类型：
- KNOWS: Person -> Person {since, strength}
- FOLLOWS: Person -> Person {since}
- POSTED: Person -> Post
- LIKES: Person -> Post {created_at}
- COMMENTED: Person -> Comment
- ON: Comment -> Post
- MEMBER_OF: Person -> Group {joined_at, role}
```

#### 测试用例

**TC-SN-01: 用户注册与好友添加流程**
- 前置条件：空数据库
- 执行步骤：
  1. CREATE SPACE social_network
  2. USE social_network
  3. CREATE TAG Person(name, age, city)
  4. CREATE EDGE KNOWS(since)
  5. INSERT VERTEX Person VALUES 1:('Alice', 25, 'Beijing')
  6. INSERT VERTEX Person VALUES 2:('Bob', 28, 'Shanghai')
  7. INSERT EDGE KNOWS VALUES 1 -> 2:('2024-01-01')
- 预期结果：
  - 所有操作成功执行
  - GO FROM 1 OVER KNOWS 返回 Bob 的信息

**TC-SN-02: 好友关系网络查询**
- 前置条件：已创建 10 个用户，形成 3 层好友关系网络
- 执行步骤：
  1. GO 2 STEPS FROM 1 OVER KNOWS
  2. FIND ALL PATH FROM 1 TO 10 OVER KNOWS
  3. GET SUBGRAPH 3 STEPS FROM 1
- 预期结果：
  - 正确返回多层好友关系
  - 路径查找返回所有可能路径
  - 子图包含指定深度的所有节点和边

**TC-SN-03: 动态发布与互动**
- 前置条件：已创建用户和帖子标签
- 执行步骤：
  1. CREATE TAG Post(content, created_at)
  2. CREATE EDGE POSTED
  3. CREATE EDGE LIKES(created_at)
  4. INSERT VERTEX Post VALUES 100:('Hello World', now())
  5. INSERT EDGE POSTED VALUES 1 -> 100
  6. INSERT EDGE LIKES VALUES 2 -> 100:(now())
  7. MATCH (p:Person)-[:POSTED]->(post:Post)<-[:LIKES]-(liker:Person) RETURN p, post, liker
- 预期结果：
  - 正确返回帖子及其作者和点赞者

**TC-SN-04: 群组管理与成员查询**
- 前置条件：已创建群组标签
- 执行步骤：
  1. CREATE TAG Group(name, description)
  2. CREATE EDGE MEMBER_OF(joined_at, role)
  3. INSERT VERTEX Group VALUES 200:('Rust爱好者', 'Rust编程语言交流')
  4. INSERT EDGE MEMBER_OF VALUES 1 -> 200:(now(), 'admin')
  5. INSERT EDGE MEMBER_OF VALUES 2 -> 200:(now(), 'member')
  6. MATCH (p:Person)-[m:MEMBER_OF]->(g:Group) WHERE g.name == 'Rust爱好者' RETURN p, m.role
- 预期结果：
  - 正确返回群组成员及其角色

**TC-SN-05: 共同好友发现**
- 前置条件：已建立复杂好友关系网络
- 执行步骤：
  1. MATCH (a:Person)-[:KNOWS]->(common:Person)<-[:KNOWS]-(b:Person)
     WHERE a.name == 'Alice' AND b.name == 'Bob'
     RETURN common
- 预期结果：
  - 正确返回 Alice 和 Bob 的共同好友

---

### 3.2 场景二：电商推荐系统

#### 场景描述
模拟电商平台的商品管理、用户行为追踪、推荐算法等核心业务。

#### 测试数据模型

```
顶点类型：
- User: {id, name, age, gender, city}
- Product: {id, name, category, price, brand}
- Category: {id, name, parent_id}
- Order: {id, total_amount, status, created_at}

边类型：
- VIEWED: User -> Product {view_time, duration}
- ADDED_TO_CART: User -> Product {added_at, quantity}
- PURCHASED: User -> Product {order_id, quantity, price}
- BELONGS_TO: Product -> Category
- SIMILAR_TO: Product -> Product {similarity_score}
- BOUGHT_TOGETHER: Product -> Product {frequency}
```

#### 测试用例

**TC-EC-01: 商品目录管理**
- 前置条件：空数据库
- 执行步骤：
  1. 创建商品空间
  2. 创建 Category、Product 标签
  3. 插入多级分类数据
  4. 插入商品数据并建立分类关系
  5. 查询某分类下的所有商品
- 预期结果：
  - 商品正确关联到分类
  - 支持多级分类查询

**TC-EC-02: 用户行为追踪**
- 前置条件：已创建商品和用户
- 执行步骤：
  1. 记录用户浏览行为
  2. 记录用户加购行为
  3. 记录用户购买行为
  4. 查询用户行为路径
- 预期结果：
  - 完整记录用户行为序列
  - 支持行为漏斗分析

**TC-EC-03: 相似商品推荐**
- 前置条件：已建立商品相似度关系
- 执行步骤：
  1. MATCH (p:Product)-[s:SIMILAR_TO]->(similar:Product)
     WHERE p.id == 100
     RETURN similar ORDER BY s.similarity_score DESC LIMIT 10
- 预期结果：
  - 返回最相似的商品列表

**TC-EC-04: 协同过滤推荐**
- 前置条件：已存在大量用户购买记录
- 执行步骤：
  1. 找到与目标用户购买行为相似的其他用户
  2. 获取相似用户购买但目标用户未购买的商品
  3. 按购买频率排序推荐
- 预期结果：
  - 返回个性化推荐商品列表

**TC-EC-05: 购物车放弃分析**
- 前置条件：存在加购但未购买记录
- 执行步骤：
  1. MATCH (u:User)-[:ADDED_TO_CART]->(p:Product)
     WHERE NOT (u)-[:PURCHASED]->(p)
     RETURN u, p
- 预期结果：
  - 正确识别购物车放弃行为

---

### 3.3 场景三：知识图谱应用

#### 场景描述
构建一个知识图谱，支持实体关系查询、知识推理、路径发现等功能。

#### 测试数据模型

```
顶点类型：
- Entity: {id, name, type, description}
- Concept: {id, name, domain}
- Document: {id, title, content, source}

边类型：
- RELATES_TO: Entity -> Entity {relation_type, confidence}
- INSTANCE_OF: Entity -> Concept
- MENTIONS: Document -> Entity {frequency}
- PART_OF: Entity -> Entity
- CAUSES: Entity -> Entity
```

#### 测试用例

**TC-KG-01: 实体关系查询**
- 前置条件：已构建知识图谱
- 执行步骤：
  1. MATCH (e:Entity)-[r:RELATES_TO]->(related:Entity)
     WHERE e.name == 'Rust' RETURN r.relation_type, related
- 预期结果：
  - 返回与 Rust 相关的所有实体及关系类型

**TC-KG-02: 多跳关系推理**
- 前置条件：已建立实体关系链
- 执行步骤：
  1. MATCH path = (start:Entity)-[:RELATES_TO*1..3]->(end:Entity)
     WHERE start.name == 'Programming Language'
     RETURN path LIMIT 100
- 预期结果：
  - 返回指定深度内的所有关系路径

**TC-KG-03: 知识一致性验证**
- 前置条件：存在循环关系
- 执行步骤：
  1. 检测循环引用
  2. 验证关系对称性
  3. 检查属性一致性
- 预期结果：
  - 正确识别知识图谱中的不一致性

---

### 3.4 场景四：模式演进工作流

#### 场景描述
测试数据库模式在实际业务中的演进过程，包括标签修改、属性变更等。

#### 测试用例

**TC-SE-01: 标签属性添加**
- 前置条件：已存在 Person 标签
- 执行步骤：
  1. ALTER TAG Person ADD (email STRING)
  2. ALTER TAG Person ADD (phone STRING DEFAULT 'unknown')
  3. 插入包含新属性的数据
  4. 查询新旧数据
- 预期结果：
  - 旧数据默认值为空或默认值
  - 新数据包含完整属性

**TC-SE-02: 标签属性修改**
- 前置条件：已存在带属性的标签
- 执行步骤：
  1. ALTER TAG Person CHANGE age age INT64
  2. 验证现有数据类型转换
- 预期结果：
  - 属性类型成功修改
  - 现有数据正确处理

**TC-SE-03: 标签属性删除**
- 前置条件：标签存在多个属性
- 执行步骤：
  1. ALTER TAG Person DROP (phone)
  2. 查询数据验证属性已移除
- 预期结果：
  - 属性从标签定义中移除
  - 查询不再返回该属性

---

### 3.5 场景五：并发与性能测试

#### 测试用例

**TC-PF-01: 并发写入测试**
- 前置条件：空数据库
- 执行步骤：
  1. 10 个并发任务同时插入数据
  2. 每个任务插入 1000 个顶点
  3. 验证数据完整性
- 预期结果：
  - 所有数据正确写入
  - 无数据丢失或冲突

**TC-PF-02: 并发查询测试**
- 前置条件：已存在大量数据
- 执行步骤：
  1. 50 个并发任务执行不同查询
  2. 查询类型包括 MATCH、GO、FETCH
  3. 监控响应时间和资源使用
- 预期结果：
  - 所有查询成功返回
  - 响应时间在可接受范围内

**TC-PF-03: 批量数据导入**
- 前置条件：空数据库
- 执行步骤：
  1. 导入 100 万顶点数据
  2. 导入 1000 万边数据
  3. 验证导入后查询性能
- 预期结果：
  - 导入成功完成
  - 查询性能满足要求

**TC-PF-04: 大图遍历性能**
- 前置条件：已存在 1000 万节点、1 亿边的图
- 执行步骤：
  1. 执行 5 层深度遍历
  2. 执行全图最短路径查找
  3. 执行子图提取
- 预期结果：
  - 复杂查询在合理时间内完成

---

## 4. 测试数据管理

### 4.1 数据生成策略

| 数据类型 | 生成方式 | 规模 |
|----------|----------|------|
| 小规模测试 | 硬编码数据 | 10-100 条 |
| 中等规模测试 | 随机生成器 | 1000-10000 条 |
| 大规模测试 | 数据生成脚本 | 100万-1000万条 |
| 真实场景数据 | 样本数据集 | 根据场景定 |

### 4.2 数据清理策略

- 每个测试用例独立数据空间
- 测试完成后自动清理
- 支持测试失败时保留数据用于调试

---

## 5. 测试执行策略

### 5.1 执行环境

```
开发环境：
- 本地运行，使用内存存储或临时文件存储
- 快速反馈，用于开发和调试

CI 环境：
- 使用文件存储，模拟真实环境
- 完整执行所有 E2E 测试
- 生成测试报告和覆盖率

预发布环境：
- 使用生产级配置
- 执行性能测试
- 验证部署流程
```

### 5.2 执行顺序

```
阶段 1: 基础功能验证
  - 场景一：社交网络（基础 CRUD）
  - 场景四：模式演进

阶段 2: 复杂查询验证
  - 场景一：社交网络（复杂查询）
  - 场景二：电商推荐
  - 场景三：知识图谱

阶段 3: 性能与并发
  - 场景五：并发与性能

阶段 4: 回归测试
  - 执行所有场景作为回归验证
```

---

## 6. 成功标准

### 6.1 功能正确性

- 所有测试用例通过率 100%
- 数据一致性检查通过
- 边界条件正确处理

### 6.2 性能指标

| 指标 | 目标值 |
|------|--------|
| 简单查询响应时间 | < 100ms |
| 复杂查询响应时间 | < 5s |
| 批量导入速度 | > 10000 条/秒 |
| 并发查询吞吐量 | > 100 QPS |

### 6.3 稳定性

- 长时间运行无内存泄漏
- 并发操作无死锁
- 错误恢复机制正常工作

---

## 7. 实施计划

### 7.1 阶段划分

| 阶段 | 内容 | 预计时间 |
|------|------|----------|
| 阶段 1 | 搭建 E2E 测试框架和基础设施 | 1 周 |
| 阶段 2 | 实现场景一（社交网络）测试 | 1 周 |
| 阶段 3 | 实现场景二（电商推荐）测试 | 1 周 |
| 阶段 4 | 实现场景三（知识图谱）测试 | 1 周 |
| 阶段 5 | 实现场景四、五测试 | 1 周 |
| 阶段 6 | CI 集成和优化 | 1 周 |

### 7.2 优先级

**P0（必须实现）**
- 社交网络基础功能测试
- 模式演进工作流测试
- 并发写入测试

**P1（重要）**
- 电商推荐场景测试
- 性能基准测试
- 并发查询测试

**P2（可选）**
- 知识图谱场景测试
- 大规模数据测试
- 回归测试套件

---

## 8. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 测试执行时间过长 | 影响 CI 效率 | 并行执行、选择性执行 |
| 测试数据准备复杂 | 维护成本高 | 数据生成工具化 |
| 测试环境不稳定 | 误报率高 | 重试机制、环境隔离 |
| 测试覆盖不足 | 遗漏缺陷 | 基于风险的测试设计 |
