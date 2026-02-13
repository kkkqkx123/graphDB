# GraphDB 语句实现完整方案

## 文档说明

本文档基于 NebulaGraph 3.8.0 的查询语法文档和代码实现，整理了所有语句的语法、实现逻辑，并提供了分阶段的执行方案。

**参考文档**: `nebula-3.8.0/docs/nebula_graph_query_syntax.md`
**参考代码**: `nebula-3.8.0/src/graph/executor/`

---

## 一、语句分类

### 1.1 按功能分类

| 分类 | 语句数量 | 说明 |
|------|---------|------|
| DDL (数据定义) | 15 | 创建、修改、删除 Tag、Edge、Space、Index |
| DML (数据操作) | 6 | 插入、更新、删除顶点和边 |
| DQL (数据查询) | 3 | FETCH、LOOKUP、GO |
| 图遍历查询 | 3 | GO、FIND PATH、GET SUBGRAPH |
| 图模式匹配 | 1 | MATCH |
| 管道操作 | 3 | 管道、赋值、SET 操作 |
| 子句 | 5 | WHERE、YIELD、ORDER BY、LIMIT、GROUP BY |
| 空间管理 | 8 | CREATE、ALTER、DROP、DESC、CLEAR、USE SPACE |
| 索引管理 | 8 | CREATE、ALTER、DROP、DESC、REBUILD INDEX |
| 用户权限管理 | 6 | CREATE、ALTER、DROP USER，GRANT、REVOKE |
| 集群管理 | 10 | HOST、ZONE、LISTENER 管理 |
| 会话管理 | 4 | SHOW、KILL SESSIONS、QUERIES |
| 配置管理 | 3 | SHOW、GET、UPDATE CONFIGS |
| 其他命令 | 10 | EXPLAIN、PROFILE、SNAPSHOT、JOB 等 |

### 1.2 按优先级分类

#### 高优先级（核心功能）
- INSERT、UPDATE、DELETE（数据操作）
- USE、SHOW（空间和元数据管理）
- GO、FETCH（图遍历和查询）
- MATCH（图模式匹配）

#### 中优先级（增强功能）
- CREATE、ALTER、DROP Tag/Edge（Schema 管理）
- CREATE、ALTER、DROP Space（空间管理）
- CREATE、DROP Index（索引管理）
- UNWIND、SET、WITH（查询增强）

#### 低优先级（高级功能）
- FIND PATH、GET SUBGRAPH（图算法）
- 用户权限管理
- 集群管理
- 会话管理
- 配置管理

---

## 二、DDL 语句（数据定义语言）

### 2.1 CREATE TAG

#### 语法
```sql
CREATE TAG [IF NOT EXISTS] <tag_name> (
  <prop_name> <data_type> [NULL | NOT NULL] [DEFAULT <default_value>] [COMMENT '<comment>'] [, ...]
) [TTL_DURATION = <ttl_duration>] [TTL_COL = <prop_name>] [COMMENT '<comment>']
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/maintain/TagExecutor.cpp`

```cpp
folly::Future<Status> CreateTagExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *ctNode = asNode<CreateTag>(node());
  auto spaceId = ctNode->getSpaceId();
  auto tagName = ctNode->getTagName();

  return qctx()->getMetaClient()->createTag(
      spaceId,
      tagName,
      ctNode->getSchema(),
      ctNode->getSchemaProp(),
      ctNode->getIfNotExists())
  .via(runner())
  .thenValue([this, spaceId, tagName](StatusOr<TagID> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    return finish(ResultBuilder()
                      .value(Value(spaceId))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

**关键步骤**:
1. 从 PlanNode 获取空间 ID、Tag 名称、Schema
2. 调用 MetaClient 创建 Tag
3. 返回执行结果

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/tag/create_tag.rs`

**集成状态**: 未集成到 GraphQueryExecutor

#### 实现建议

```rust
fn execute_create_tag(&mut self, clause: CreateTagStmt) -> Result<ExecutionResult, DBError> {
    use crate::query::executor::admin::tag::create_tag::CreateTagExecutor;

    let mut executor = CreateTagExecutor::new(
        self.id,
        self.storage.clone(),
        clause.space_name,
        clause.tag_name,
        clause.properties,
    );
    executor.open()?;
    executor.execute()
}
```

---

### 2.2 ALTER TAG

#### 语法
```sql
ALTER TAG <tag_name>
  | ADD (<prop_name> <data_type> [NULL | NOT NULL] [DEFAULT <default_value>] [COMMENT '<comment>'] [, ...])
  | CHANGE (<prop_name> <data_type> [NULL | NOT NULL] [DEFAULT <default_value>] [COMMENT '<comment>'] [, ...])
  | DROP (<prop_name> [, ...])
  [TTL_DURATION = <ttl_duration>] [TTL_COL = <prop_name>] [COMMENT '<comment>']
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/maintain/TagExecutor.cpp`

```cpp
folly::Future<Status> AlterTagExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *atNode = asNode<AlterTag>(node());
  auto spaceId = atNode->getSpaceId();
  auto tagName = atNode->getTagName();

  return qctx()->getMetaClient()->alterTag(
      spaceId,
      tagName,
      atNode->getSchemaItems(),
      atNode->getSchemaProp())
  .via(runner())
  .thenValue([this](StatusOr<bool> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    return finish(ResultBuilder()
                      .value(Value(resp.value()))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/tag/alter_tag.rs`

**集成状态**: 未集成到 GraphQueryExecutor

---

### 2.3 DESCRIBE TAG

#### 语法
```sql
DESCRIBE TAG <tag_name>
DESC TAG <tag_name>
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/maintain/TagExecutor.cpp`

```cpp
folly::Future<Status> DescTagExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *dtNode = asNode<DescTag>(node());
  auto spaceId = dtNode->getSpaceId();
  auto tagName = dtNode->getTagName();

  return qctx()->getMetaClient()->getTag(spaceId, tagName)
  .via(runner())
  .thenValue([this](StatusOr<meta::cpp2::Schema> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    auto schema = std::move(resp).value();
    DataSet dataSet({"Field", "Type", "Null", "Default"});
    for (auto &col : schema.columns) {
      Row row;
      row.values.emplace_back(col.name);
      row.values.emplace_back(Value(typeToString(col.type)));
      row.values.emplace_back(Value(col.nullable ? "YES" : "NO"));
      row.values.emplace_back(col.default_value_ref().has_value()
                                ? Value(col.default_value_ref().value())
                                : Value());
      dataSet.rows.emplace_back(std::move(row));
    }
    return finish(ResultBuilder()
                      .value(Value(std::move(dataSet)))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/tag/desc_tag.rs`

**集成状态**: 已集成到 execute_show (SHOW TAG <name>)

---

### 2.4 DROP TAG

#### 语法
```sql
DROP TAG [IF EXISTS] <tag_name>
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/maintain/TagExecutor.cpp`

```cpp
folly::Future<Status> DropTagExecutor::execute() {
  SCOPED_TIMER(&executeTime_);
  auto *dtNode = asNode<DropTag>(node());
  auto spaceId = dtNode->getSpaceId();
  auto tagName = dtNode->getTagName();

  return qctx()->getMetaClient()->dropTag(
      spaceId,
      tagName,
      dtNode->getIfExists())
  .via(runner())
  .thenValue([this](StatusOr<bool> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    return finish(ResultBuilder()
                      .value(Value(resp.value()))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/tag/drop_tag.rs`

**集成状态**: 已集成到 execute_drop（支持批量删除）

---

### 2.5 CREATE EDGE

#### 语法
```sql
CREATE EDGE [IF NOT EXISTS] <edge_name> (
  <prop_name> <data_type> [NULL | NOT NULL] [DEFAULT <default_value>] [COMMENT '<comment>'] [, ...]
) [TTL_DURATION = <ttl_duration>] [TTL_COL = <prop_name>] [COMMENT '<comment>']
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/maintain/EdgeExecutor.cpp`

```cpp
folly::Future<Status> CreateEdgeExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *ceNode = asNode<CreateEdge>(node());
  auto spaceId = ceNode->getSpaceId();
  auto edgeName = ceNode->getEdgeName();

  return qctx()->getMetaClient()->createEdge(
      spaceId,
      edgeName,
      ceNode->getSchema(),
      ceNode->getSchemaProp(),
      ceNode->getIfNotExists())
  .via(runner())
  .thenValue([this, spaceId, edgeName](StatusOr<EdgeType> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    return finish(ResultBuilder()
                      .value(Value(spaceId))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/edge/create_edge.rs`

**集成状态**: 未集成到 GraphQueryExecutor

---

### 2.6 ALTER EDGE

#### 语法
```sql
ALTER EDGE <edge_name>
  | ADD (<prop_name> <data_type> [NULL | NOT NULL] [DEFAULT <default_value>] [COMMENT '<comment>'] [, ...])
  | CHANGE (<prop_name> <data_type> [NULL | NOT NULL] [DEFAULT <default_value>] [COMMENT '<comment>'] [, ...])
  | DROP (<prop_name> [, ...])
  [TTL_DURATION = <ttl_duration>] [TTL_COL = <prop_name>] [COMMENT '<comment>']
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/maintain/EdgeExecutor.cpp`

```cpp
folly::Future<Status> AlterEdgeExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *aeNode = asNode<AlterEdge>(node());
  auto spaceId = aeNode->getSpaceId();
  auto edgeName = aeNode->getEdgeName();

  return qctx()->getMetaClient()->alterEdge(
      spaceId,
      edgeName,
      aeNode->getSchemaItems(),
      aeNode->getSchemaProp())
  .via(runner())
  .thenValue([this](StatusOr<bool> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    return finish(ResultBuilder()
                      .value(Value(resp.value()))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/edge/alter_edge.rs`

**集成状态**: 未集成到 GraphQueryExecutor

---

### 2.7 DESCRIBE EDGE

#### 语法
```sql
DESCRIBE EDGE <edge_name>
DESC EDGE <edge_name>
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/maintain/EdgeExecutor.cpp`

```cpp
folly::Future<Status> DescEdgeExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *deNode = asNode<DescEdge>(node());
  auto spaceId = deNode->getSpaceId();
  auto edgeName = deNode->getEdgeName();

  return qctx()->getMetaClient()->getEdge(spaceId, edgeName)
  .via(runner())
  .thenValue([this](StatusOr<meta::cpp2::Schema> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    auto schema = std::move(resp).value();
    DataSet dataSet({"Field", "Type", "Null", "Default"});
    for (auto &col : schema.columns) {
      Row row;
      row.values.emplace_back(col.name);
      row.values.emplace_back(Value(typeToString(col.type)));
      row.values.emplace_back(Value(col.nullable ? "YES" : "NO"));
      row.values.emplace_back(col.default_value_ref().has_value()
                                ? Value(col.default_value_ref().value())
                                : Value());
      dataSet.rows.emplace_back(std::move(row));
    }
    return finish(ResultBuilder()
                      .value(Value(std::move(dataSet)))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/edge/desc_edge.rs`

**集成状态**: 已集成到 execute_show (SHOW EDGE <name>)

---

### 2.8 DROP EDGE

#### 语法
```sql
DROP EDGE [IF EXISTS] <edge_name>
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/maintain/EdgeExecutor.cpp`

```cpp
folly::Future<Status> DropEdgeExecutor::execute() {
  SCOPED_TIMER(&executeTime_);
  auto *deNode = asNode<DropEdge>(node());
  auto spaceId = deNode->getSpaceId();
  auto edgeName = deNode->getEdgeName();

  return qctx()->getMetaClient()->dropEdge(
      spaceId,
      edgeName,
      deNode->getIfExists())
  .via(runner())
  .thenValue([this](StatusOr<bool> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    return finish(ResultBuilder()
                      .value(Value(resp.value()))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/edge/drop_edge.rs`

**集成状态**: 已集成到 execute_drop（支持批量删除）

---

### 2.9 CREATE SPACE

#### 语法
```sql
CREATE SPACE [IF NOT EXISTS] <space_name>
  [PARTITION_NUM = <partition_num>]
  [REPLICA_FACTOR = <replica_factor>]
  [VID_TYPE = <vid_type>]
  [CHARSET = <charset>]
  [COLLATE = <collate>]
  [ON <zone_name> [, <zone_name> ...]]
  [COMMENT '<comment>']
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/admin/SpaceExecutor.cpp`

```cpp
folly::Future<Status> CreateSpaceExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *csNode = asNode<CreateSpace>(node());
  auto spaceName = csNode->getSpaceName();

  return qctx()->getMetaClient()->createSpace(
      spaceName,
      csNode->getSpaceDesc(),
      csNode->getIfNotExists())
  .via(runner())
  .thenValue([this, spaceName](StatusOr<GraphSpaceID> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    auto spaceId = resp.value();
    return finish(ResultBuilder()
                      .value(Value(spaceId))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/space/create_space.rs`

**集成状态**: 未集成到 GraphQueryExecutor

---

### 2.10 DROP SPACE

#### 语法
```sql
DROP SPACE [IF EXISTS] <space_name>
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/admin/SpaceExecutor.cpp`

```cpp
folly::Future<Status> DropSpaceExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *dsNode = asNode<DropSpace>(node());
  auto spaceName = dsNode->getSpaceName();

  return qctx()->getMetaClient()->dropSpace(
      spaceName,
      dsNode->getIfExists())
  .via(runner())
  .thenValue([this](StatusOr<bool> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    return finish(ResultBuilder()
                      .value(Value(resp.value()))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/space/drop_space.rs`

**集成状态**: 已集成到 execute_drop

---

### 2.11 USE SPACE

#### 语法
```sql
USE <space_name>
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/admin/SwitchSpaceExecutor.cpp`

```cpp
folly::Future<Status> SwitchSpaceExecutor::execute() {
  memory::MemoryCheckOffGuard guard;
  SCOPED_TIMER(&execTime_);

  auto *spaceToNode = asNode<SwitchSpace>(node());
  auto spaceName = spaceToNode->getSpaceName();

  return qctx()->getMetaClient()->getSpace(spaceName).via(runner())
    .thenValue([spaceName, this](StatusOr<meta::cpp2::SpaceItem> resp) {
      if (!resp.ok()) {
        LOG(WARNING) << "Switch space :`" << spaceName << "' fail: " << resp.status();
        return resp.status();
      }

      auto spaceId = resp.value().get_space_id();

      if (!qctx() || !qctx()->rctx() || qctx_->rctx()->session() == nullptr) {
        return Status::Error("Session not found");
      }

      auto *session = qctx_->rctx()->session();
      NG_RETURN_IF_ERROR(PermissionManager::canReadSpace(session, spaceId));

      const auto &properties = resp.value().get_properties();

      SpaceInfo spaceInfo;
      spaceInfo.id = spaceId;
      spaceInfo.name = spaceName;
      spaceInfo.spaceDesc = std::move(properties);
      qctx_->rctx()->session()->setSpace(std::move(spaceInfo));

      LOG(INFO) << "Graph switched to `" << spaceName << "', space id: " << spaceId;
      return Status::OK();
    });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/space/switch_space.rs`

**集成状态**: 已集成到 execute_use

---

### 2.12 CREATE TAG INDEX

#### 语法
```sql
CREATE TAG INDEX [IF NOT EXISTS] <index_name>
  ON <tag_name> (<prop_name> [, <prop_name> ...])
  [WITH (S2_MAX_LEVEL = <level>, S2_MAX_CELLS = <cells>)]
  [COMMENT '<comment>']
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/maintain/TagIndexExecutor.cpp`

```cpp
folly::Future<Status> CreateTagIndexExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *ctiNode = asNode<CreateTagIndex>(node());
  auto spaceId = ctiNode->getSpaceId();
  auto indexName = ctiNode->getIndexName();

  return qctx()->getMetaClient()->createTagIndex(
      spaceId,
      indexName,
      ctiNode->getSchema(),
      ctiNode->getIfNotExists())
  .via(runner())
  .thenValue([this, spaceId, indexName](StatusOr<IndexID> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    return finish(ResultBuilder()
                      .value(Value(spaceId))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/index/tag_index.rs`

**集成状态**: 未集成到 GraphQueryExecutor

---

### 2.13 DROP TAG INDEX

#### 语法
```sql
DROP TAG INDEX [IF EXISTS] <index_name>
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/maintain/TagIndexExecutor.cpp`

```cpp
folly::Future<Status> DropTagIndexExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *dtiNode = asNode<DropTagIndex>(node());
  auto spaceId = dtiNode->getSpaceId();
  auto indexName = dtiNode->getIndexName();

  return qctx()->getMetaClient()->dropTagIndex(
      spaceId,
      indexName,
      dtiNode->getIfExists())
  .via(runner())
  .thenValue([this](StatusOr<bool> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    return finish(ResultBuilder()
                      .value(Value(resp.value()))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/index/drop_tag_index.rs`

**集成状态**: 已集成到 execute_drop

---

### 2.14 CREATE EDGE INDEX

#### 语法
```sql
CREATE EDGE INDEX [IF NOT EXISTS] <index_name>
  ON <edge_name> (<prop_name> [, <prop_name> ...])
  [WITH (S2_MAX_LEVEL = <level>, S2_MAX_CELLS = <cells>)]
  [COMMENT '<comment>']
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/maintain/EdgeIndexExecutor.cpp`

```cpp
folly::Future<Status> CreateEdgeIndexExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *ceiNode = asNode<CreateEdgeIndex>(node());
  auto spaceId = ceiNode->getSpaceId();
  auto indexName = ceiNode->getIndexName();

  return qctx()->getMetaClient()->createEdgeIndex(
      spaceId,
      indexName,
      ceiNode->getSchema(),
      ceiNode->getIfNotExists())
  .via(runner())
  .thenValue([this, spaceId, indexName](StatusOr<IndexID> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    return finish(ResultBuilder()
                      .value(Value(spaceId))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/index/edge_index.rs`

**集成状态**: 未集成到 GraphQueryExecutor

---

### 2.15 DROP EDGE INDEX

#### 语法
```sql
DROP EDGE INDEX [IF EXISTS] <index_name>
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/maintain/EdgeIndexExecutor.cpp`

```cpp
folly::Future<Status> DropEdgeIndexExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *deiNode = asNode<DropEdgeIndex>(node());
  auto spaceId = deiNode->getSpaceId();
  auto indexName = deiNode->getIndexName();

  return qctx()->getMetaClient()->dropEdgeIndex(
      spaceId,
      indexName,
      deiNode->getIfExists())
  .via(runner())
  .thenValue([this](StatusOr<bool> resp) {
    if (!resp.ok()) {
      return resp.status();
    }
    return finish(ResultBuilder()
                      .value(Value(resp.value()))
                      .iter(Iterator::Kind::kDefault)
                      .build());
  });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/admin/index/drop_edge_index.rs`

**集成状态**: 已集成到 execute_drop

---

## 三、DML 语句（数据操作语言）

### 3.1 INSERT VERTEX

#### 语法
```sql
INSERT VERTEX [IF NOT EXISTS] <tag_name> (<prop_name> [, ...]) [IGNORE_EXISTED_INDEX]
{ VALUES | VALUE } <vid>: (<prop_value> [, ...]) [, <vid>: (<prop_value> [, ...]) ...]
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/mutate/InsertExecutor.cpp`

```cpp
folly::Future<Status> InsertVerticesExecutor::execute() {
  return insertVertices();
}

folly::Future<Status> InsertVerticesExecutor::insertVertices() {
  SCOPED_TIMER(&execTime_);

  auto *ivNode = asNode<InsertVertices>(node());
  time::Duration addVertTime;
  auto plan = qctx()->plan();
  StorageClient::CommonRequestParam param(
      ivNode->getSpace(), qctx()->rctx()->session()->id(), plan->id(), plan->isProfileEnabled());

  return qctx()
      ->getStorageClient()
      ->addVertices(param,
                    ivNode->getVertices(),
                    ivNode->getPropNames(),
                    ivNode->getIfNotExists(),
                    ivNode->getIgnoreExistedIndex())
      .via(runner())
      .ensure([addVertTime]() {
        VLOG(1) << "Add vertices time: " << addVertTime.elapsedInUSec() << "us";
      })
      .thenValue([this](storage::StorageRpcResponse<storage::cpp2::ExecResponse> resp) {
        memory::MemoryCheckGuard guard;
        SCOPED_TIMER(&execTime_);
        NG_RETURN_IF_ERROR(handleCompleteness(resp, false));
        return Status::OK();
      });
}
```

**关键步骤**:
1. 从 PlanNode 获取空间 ID、顶点数据、属性名称
2. 调用 StorageClient 的 addVertices 方法
3. 处理响应和完整性检查
4. 返回执行结果

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/data_modification.rs` (InsertExecutor)

**集成状态**: 已集成到 execute_insert

**实现细节**:
- 支持顶点插入
- 支持边插入
- 支持批量操作
- 支持表达式求值

---

### 3.2 INSERT EDGE

#### 语法
```sql
INSERT EDGE [IF NOT EXISTS] <edge_name> ([<prop_name> [, ...]]) [IGNORE_EXISTED_INDEX]
{ VALUES | VALUE }
<src_vid> -> <dst_vid>[@<rank>]: (<prop_value> [, ...])
[, <src_vid> -> <dst_vid>[@<rank>]: (<prop_value> [, ...]) ...]
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/mutate/InsertExecutor.cpp`

```cpp
folly::Future<Status> InsertEdgesExecutor::execute() {
  return insertEdges();
}

folly::Future<Status> InsertEdgesExecutor::insertEdges() {
  SCOPED_TIMER(&execTime_);

  auto *ieNode = asNode<InsertEdges>(node());
  time::Duration addEdgeTime;
  auto plan = qctx()->plan();
  StorageClient::CommonRequestParam param(
      ieNode->getSpace(), qctx()->rctx()->session()->id(), plan->id(), plan->isProfileEnabled());
  param.useExperimentalFeature = false;
  return qctx()
      ->getStorageClient()
      ->addEdges(param,
                 ieNode->getEdges(),
                 ieNode->getPropNames(),
                 ieNode->getIfNotExists(),
                 ieNode->getIgnoreExistedIndex())
      .via(runner())
      .ensure(
          [addEdgeTime]() { VLOG(1) << "Add edge time: " << addEdgeTime.elapsedInUSec() << "us"; })
      .thenValue([this](storage::StorageRpcResponse<storage::cpp2::ExecResponse> resp) {
        memory::MemoryCheckGuard guard;
        SCOPED_TIMER(&execTime_);
        NG_RETURN_IF_ERROR(handleCompleteness(resp, false));
        return Status::OK();
      });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/data_modification.rs` (InsertExecutor)

**集成状态**: 已集成到 execute_insert

---

### 3.3 UPDATE VERTEX

#### 语法
```sql
UPDATE VERTEX <vid>
  [SET <update_item> [, ...]]
  [WHEN <condition>]
  [YIELD <return_item> [, ...]]
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/mutate/UpdateExecutor.cpp`

```cpp
folly::Future<Status> UpdateVertexExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *uvNode = asNode<UpdateVertex>(node());
  yieldNames_ = uvNode->getYieldNames();
  time::Duration updateVertTime;
  auto plan = qctx()->plan();
  auto sess = qctx()->rctx()->session();
  StorageClient::CommonRequestParam param(
      uvNode->getSpaceId(), sess->id(), plan->id(), plan->isProfileEnabled());

  return qctx()
      ->getStorageClient()
      ->updateVertex(param,
                     uvNode->getVId(),
                     uvNode->getTagId(),
                     uvNode->getUpdatedProps(),
                     uvNode->getInsertable(),
                     uvNode->getReturnProps(),
                     uvNode->getCondition())
      .via(runner())
      .ensure([updateVertTime]() {
        VLOG(1) << "Update vertice time: " << updateVertTime.elapsedInUSec() << "us";
      })
      .thenValue([this](StatusOr<storage::cpp2::UpdateResponse> resp) {
        memory::MemoryCheckGuard guard;
        SCOPED_TIMER(&execTime_);
        if (!resp.ok()) {
          LOG(WARNING) << "Update vertices fail: " << resp.status();
          return resp.status();
        }
        auto value = std::move(resp).value();
        for (auto &code : value.get_result().get_failed_parts()) {
          NG_RETURN_IF_ERROR(handleErrorCode(code.get_code(), code.get_part_id()));
        }
        if (value.props_ref().has_value()) {
          auto status = handleResult(std::move(*value.props_ref()));
          if (!status.ok()) {
            return status.status();
          }
          return finish(ResultBuilder()
                            .value(std::move(status).value())
                            .iter(Iterator::Kind::kDefault)
                            .build());
        }
        return Status::OK();
      });
}
```

**关键特性**:
1. 支持 UPSERT（insertable 参数）
2. 支持条件更新（condition）
3. 支持 RETURN 子句返回更新后的属性
4. 支持表达式求值
5. 部分失败处理
6. 性能计时

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/data_modification.rs` (UpdateExecutor)

**集成状态**: 已集成到 execute_update

---

### 3.4 UPDATE EDGE

#### 语法
```sql
UPDATE EDGE <src_vid> -> <dst_vid>[@<rank>] OF <edge_name>
  [SET <update_item> [, ...]]
  [WHEN <condition>]
  [YIELD <return_item> [, ...]]
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/mutate/UpdateExecutor.cpp`

```cpp
folly::Future<Status> UpdateEdgeExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  auto *ueNode = asNode<UpdateEdge>(node());
  storage::cpp2::EdgeKey edgeKey;
  edgeKey.src_ref() = ueNode->getSrcId();
  edgeKey.ranking_ref() = ueNode->getRank();
  edgeKey.edge_type_ref() = ueNode->getEdgeType();
  edgeKey.dst_ref() = ueNode->getDstId();
  yieldNames_ = ueNode->getYieldNames();

  time::Duration updateEdgeTime;
  auto plan = qctx()->plan();
  StorageClient::CommonRequestParam param(
      ueNode->getSpaceId(), qctx()->rctx()->session()->id(), plan->id(), plan->isProfileEnabled());

  return qctx()
      ->getStorageClient()
      ->updateEdge(param,
                   edgeKey,
                   ueNode->getUpdatedProps(),
                   ueNode->getInsertable(),
                   ueNode->getReturnProps(),
                   ueNode->getCondition())
      .via(runner())
      .ensure([updateEdgeTime]() {
        VLOG(1) << "Update edge time: " << updateEdgeTime.elapsedInUSec() << "us";
      })
      .thenValue([this](StatusOr<storage::cpp2::UpdateResponse> resp) {
        memory::MemoryCheckGuard guard;
        SCOPED_TIMER(&execTime_);
        if (!resp.ok()) {
          LOG(WARNING) << "Update edge fail: " << resp.status();
          return resp.status();
        }
        auto value = std::move(resp).value();
        for (auto &code : value.get_result().get_failed_parts()) {
          NG_RETURN_IF_ERROR(handleErrorCode(code.get_code(), code.get_part_id()));
        }
        if (value.props_ref().has_value()) {
          auto status = handleResult(std::move(*value.props_ref()));
          if (!status.ok()) {
            return status.status();
          }
          return finish(ResultBuilder()
                            .value(std::move(status).value())
                            .iter(Iterator::Kind::kDefault)
                            .build());
        }
        return Status::OK();
      });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/data_modification.rs` (UpdateExecutor)

**集成状态**: 已集成到 execute_update

---

### 3.5 DELETE VERTEX

#### 语法
```sql
DELETE VERTEX <vid> [, <vid> ...] [WITH EDGE]
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/mutate/DeleteExecutor.cpp`

```cpp
folly::Future<Status> DeleteVerticesExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  return deleteVertices();
}

folly::Future<Status> DeleteVerticesExecutor::deleteVertices() {
  auto* dvNode = asNode<DeleteVertices>(node());
  auto vidRef = dvNode->getVidRef();
  std::vector<Value> vertices;
  const auto& spaceInfo = qctx()->rctx()->session()->space();
  if (vidRef != nullptr) {
    auto inputVar = dvNode->inputVar();
    if (inputVar.empty()) {
      DCHECK(dvNode->dep() != nullptr);
      auto* gn = static_cast<const SingleInputNode*>(dvNode->dep())->dep();
      DCHECK(gn != nullptr);
      inputVar = static_cast<const SingleInputNode*>(gn)->inputVar();
    }
    DCHECK(!inputVar.empty());
    auto& inputResult = ectx_->getResult(inputVar);
    auto iter = inputResult.iter();
    vertices.reserve(iter->size());

    QueryExpressionContext ctx(ectx_);
    for (; iter->valid(); iter->next()) {
      auto val = Expression::eval(vidRef, ctx(iter.get()));
      if (val.isNull() || val.empty()) {
        continue;
      }
      if (!SchemaUtil::isValidVid(val, *spaceInfo.spaceDesc.vid_type_ref())) {
        std::stringstream ss;
        ss << "Wrong vid type `" << val.type() << "', value `" << val.toString() << "'";
        return Status::Error(ss.str());
      }
      vertices.emplace_back(std::move(val));
    }
  }

  if (vertices.empty()) {
    return Status::OK();
  }

  auto spaceId = spaceInfo.id;
  time::Duration deleteVertTime;
  auto plan = qctx()->plan();
  StorageClient::CommonRequestParam param(
      spaceId, qctx()->rctx()->session()->id(), plan->id(), plan->isProfileEnabled());

  return qctx()
      ->getStorageClient()
      ->deleteVertices(param, std::move(vertices))
      .via(runner())
      .ensure([deleteVertTime]() {
        VLOG(1) << "Delete vertices time: " << deleteVertTime.elapsedInUSec() << "us";
      })
      .thenValue([this](storage::StorageRpcResponse<storage::cpp2::ExecResponse> resp) {
        memory::MemoryCheckGuard guard;
        SCOPED_TIMER(&execTime_);
        NG_RETURN_IF_ERROR(handleCompleteness(resp, false));
        return Status::OK();
      });
}
```

**关键特性**:
1. 从输入变量中提取要删除的顶点/边
2. 支持表达式求值获取 ID
3. 支持条件删除
4. 批量删除优化
5. VID 类型验证
6. 空值和空值过滤

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/data_modification.rs` (DeleteExecutor)

**集成状态**: 已集成到 execute_delete

---

### 3.6 DELETE EDGE

#### 语法
```sql
DELETE EDGE <edge_name> <src_vid> -> <dst_vid>[@<rank>] [, <src_vid> -> <dst_vid>[@<rank>] ...]
```

#### Nebula-Graph 实现逻辑

**文件**: `nebula-3.8.0/src/graph/executor/mutate/DeleteExecutor.cpp`

```cpp
folly::Future<Status> DeleteEdgesExecutor::execute() {
  SCOPED_TIMER(&execTime_);
  return deleteEdges();
}

folly::Future<Status> DeleteEdgesExecutor::deleteEdges() {
  auto* deNode = asNode<DeleteEdges>(node());
  auto edgeKeyRef = deNode->getEdgeKeyRef();
  std::vector<storage::cpp2::EdgeKey> edgeKeys;
  const auto& spaceInfo = qctx()->rctx()->session()->space();

  if (edgeKeyRef != nullptr) {
    auto inputVar = deNode->inputVar();
    DCHECK(!inputVar.empty());
    auto& inputResult = ectx_->getResult(inputVar);
    auto iter = inputResult.iter();
    edgeKeys.reserve(iter->size());

    QueryExpressionContext ctx(ectx_);
    for (; iter->valid(); iter->next()) {
      auto val = Expression::eval(edgeKeyRef, ctx(iter.get()));
      if (val.isNull() || val.empty()) {
        continue;
      }
      if (!val.isEdge()) {
        std::stringstream ss;
        ss << "Wrong edge type `" << val.type() << "'";
        return Status::Error(ss.str());
      }
      edgeKeys.emplace_back(val.getEdge());
    }
  }

  if (edgeKeys.empty()) {
    return Status::OK();
  }

  auto spaceId = spaceInfo.id;
  time::Duration deleteEdgeTime;
  auto plan = qctx()->plan();
  StorageClient::CommonRequestParam param(
      spaceId, qctx()->rctx()->session()->id(), plan->id(), plan->isProfileEnabled());

  return qctx()
      ->getStorageClient()
      ->deleteEdges(param, std::move(edgeKeys))
      .via(runner())
      .ensure([deleteEdgeTime]() {
        VLOG(1) << "Delete edge time: " << deleteEdgeTime.elapsedInUSec() << "us";
      })
      .thenValue([this](storage::StorageRpcResponse<storage::cpp2::ExecResponse> resp) {
        memory::MemoryCheckGuard guard;
        SCOPED_TIMER(&execTime_);
        NG_RETURN_IF_ERROR(handleCompleteness(resp, false));
        return Status::OK();
      });
}
```

#### 当前 GraphDB 实现状态

**已有实现**: `src/query/executor/data_modification.rs` (DeleteExecutor)

**集成状态**: 已集成到 execute_delete

---

## 四、分阶段执行方案

### 阶段一：核心数据操作（已完成）

**目标**: 实现基本的数据插入、更新、删除功能

**包含语句**:
1. INSERT VERTEX/EDGE ✅
2. UPDATE VERTEX/EDGE ✅
3. DELETE VERTEX/EDGE ✅
4. USE SPACE ✅
5. SHOW SPACES/TAGS/EDGES ✅
6. DROP TAG/EDGE/SPACE ✅
7. UNWIND ✅
8. SET ✅
9. EXPLAIN ✅

**完成状态**: 100%

**下一步**: 进入阶段二

---

### 阶段二：Schema 管理（进行中）

**目标**: 实现完整的 Tag、Edge、Space、Index 管理功能

**包含语句**:
1. CREATE TAG/EDGE
2. ALTER TAG/EDGE
3. DESCRIBE TAG/EDGE
4. CREATE SPACE
5. ALTER SPACE
6. DESCRIBE SPACE
7. CREATE TAG/EDGE INDEX
8. DESCRIBE TAG/EDGE INDEX
9. SHOW TAG/EDGE INDEXES

**实现优先级**:

| 优先级 | 语句 | 复杂度 | 预计工作量 | 已有实现 | 集成状态 |
|--------|------|--------|-----------|---------|---------|
| 高 | CREATE TAG/EDGE | 中 | 2天 | ✅ | ❌ |
| 高 | DESCRIBE TAG/EDGE | 低 | 1天 | ✅ | ✅ |
| 高 | CREATE SPACE | 中 | 2天 | ✅ | ❌ |
| 中 | ALTER TAG/EDGE | 中 | 2天 | ✅ | ❌ |
| 中 | DROP TAG/EDGE | 低 | 1天 | ✅ | ✅ |
| 中 | CREATE TAG/EDGE INDEX | 中 | 2天 | ✅ | ❌ |
| 低 | DESCRIBE TAG/EDGE INDEX | 低 | 1天 | ✅ | ✅ |
| 低 | ALTER SPACE | 中 | 2天 | ❌ | ❌ |
| 低 | SHOW TAG/EDGE INDEXES | 低 | 1天 | ❌ | ❌ |

**总计**: 约14天

---

### 阶段三：数据查询（待开始）

**目标**: 实现基本的数据查询功能

**包含语句**:
1. FETCH PROP ON TAG
2. FETCH PROP ON EDGE
3. LOOKUP

**实现优先级**:

| 优先级 | 语句 | 复杂度 | 预计工作量 | 已有实现 | 集成状态 |
|--------|------|--------|-----------|---------|---------|
| 高 | FETCH PROP ON TAG | 中 | 2天 | ✅ | ❌ |
| 高 | FETCH PROP ON EDGE | 中 | 2天 | ✅ | ❌ |
| 中 | LOOKUP | 高 | 3天 | ❌ | ❌ |

**总计**: 约7天

---

### 阶段四：图遍历（待开始）

**目标**: 实现图遍历和路径查询功能

**包含语句**:
1. GO
2. FIND PATH
3. GET SUBGRAPH

**实现优先级**:

| 优先级 | 语句 | 复杂度 | 预计工作量 | 已有实现 | 集成状态 |
|--------|------|--------|-----------|---------|---------|
| 高 | GO | 高 | 5天 | ❌ | ❌ |
| 中 | FIND PATH | 高 | 5天 | ❌ | ❌ |
| 低 | GET SUBGRAPH | 高 | 5天 | ❌ | ❌ |

**总计**: 约15天

---

### 阶段五：图模式匹配（待开始）

**目标**: 实现 Cypher 风格的图模式匹配

**包含语句**:
1. MATCH
2. OPTIONAL MATCH
3. MATCH SET 操作（UNION、INTERSECT、MINUS）

**实现优先级**:

| 优先级 | 语句 | 复杂度 | 预计工作量 | 已有实现 | 集成状态 |
|--------|------|--------|-----------|---------|---------|
| 高 | MATCH | 高 | 7天 | ❌ | ❌ |
| 中 | OPTIONAL MATCH | 高 | 5天 | ❌ | ❌ |
| 低 | MATCH SET 操作 | 中 | 3天 | ❌ | ❌ |

**总计**: 约15天

---

### 阶段六：查询增强（待开始）

**目标**: 实现查询增强功能

**包含语句**:
1. WHERE 子句
2. YIELD 子句
3. ORDER BY 子句
4. LIMIT 子句
5. GROUP BY 子句
6. HAVING 子句
7. DISTINCT
8. 管道操作（|）
9. 赋值操作（=）

**实现优先级**:

| 优先级 | 语句 | 复杂度 | 预计工作量 | 已有实现 | 集成状态 |
|--------|------|--------|-----------|---------|---------|
| 高 | WHERE | 中 | 2天 | ✅ | ❌ |
| 高 | YIELD | 中 | 2天 | ✅ | ❌ |
| 高 | ORDER BY | 中 | 2天 | ✅ | ❌ |
| 高 | LIMIT | 低 | 1天 | ✅ | ❌ |
| 中 | GROUP BY | 高 | 4天 | ✅ | ❌ |
| 中 | HAVING | 中 | 2天 | ❌ | ❌ |
| 中 | DISTINCT | 低 | 1天 | ❌ | ❌ |
| 低 | 管道操作 | 中 | 2天 | ❌ | ❌ |
| 低 | 赋值操作 | 低 | 1天 | ✅ | ✅ |

**总计**: 约17天

---

### 阶段七：高级功能（待开始）

**目标**: 实现高级功能

**包含语句**:
1. 用户权限管理（CREATE/ALTER/DROP USER, GRANT/REVOKE）
2. 集群管理（ADD/DROP HOSTS, SHOW HOSTS）
3. 会话管理（SHOW/KILL SESSIONS, SHOW/KILL QUERIES）
4. 配置管理（SHOW/GET/UPDATE CONFIGS）
5. 快照管理（CREATE/DROP SNAPSHOT）
6. 任务管理（SUBMIT/SHOW/RECOVER JOB）

**实现优先级**:

| 优先级 | 语句 | 复杂度 | 预计工作量 | 已有实现 | 集成状态 |
|--------|------|--------|-----------|---------|---------|
| 低 | 用户权限管理 | 高 | 10天 | ❌ | ❌ |
| 低 | 集群管理 | 高 | 10天 | ❌ | ❌ |
| 低 | 会话管理 | 中 | 5天 | ❌ | ❌ |
| 低 | 配置管理 | 中 | 5天 | ❌ | ❌ |
| 低 | 快照管理 | 中 | 5天 | ❌ | ❌ |
| 低 | 任务管理 | 中 | 5天 | ❌ | ❌ |

**总计**: 约40天

---

## 五、总结

### 5.1 当前完成情况

| 阶段 | 状态 | 完成度 |
|------|------|--------|
| 阶段一：核心数据操作 | ✅ 已完成 | 100% |
| 阶段二：Schema 管理 | 🔄 进行中 | 30% |
| 阶段三：数据查询 | ⏳ 待开始 | 0% |
| 阶段四：图遍历 | ⏳ 待开始 | 0% |
| 阶段五：图模式匹配 | ⏳ 待开始 | 0% |
| 阶段六：查询增强 | ⏳ 待开始 | 0% |
| 阶段七：高级功能 | ⏳ 待开始 | 0% |

### 5.2 总体进度

- **已完成语句**: 9 个
- **进行中语句**: 0 个
- **待实现语句**: 60+ 个
- **总体完成度**: 约 15%

### 5.3 下一步行动

1. **立即开始**: 阶段二 - Schema 管理
   - 集成 CREATE TAG/EDGE
   - 集成 CREATE SPACE
   - 集成 ALTER TAG/EDGE
   - 集成 CREATE TAG/EDGE INDEX

2. **短期目标**: 完成阶段二和阶段三
   - 完成所有 Schema 管理功能
   - 实现基本的数据查询功能

3. **中期目标**: 完成阶段四和阶段五
   - 实现图遍历功能
   - 实现图模式匹配功能

4. **长期目标**: 完成所有阶段
   - 实现完整的 NebulaGraph 语法支持
   - 提供高性能的图数据库功能

---

## 附录：参考资料

### A. Nebula-Graph 源码结构

```
nebula-3.8.0/src/graph/executor/
├── admin/           # 管理操作执行器
├── algo/            # 图算法执行器
├── logic/           # 逻辑控制执行器
├── maintain/        # 维护操作执行器
├── mutate/          # 数据修改执行器
├── query/           # 查询执行器
└── test/            # 测试代码
```

### B. GraphDB 源码结构

```
src/query/executor/
├── admin/           # 管理操作执行器
├── data_access.rs   # 数据访问执行器
├── data_modification.rs  # 数据修改执行器
├── graph_query_executor.rs  # 图查询执行器
├── result_processing/  # 结果处理执行器
└── traits.rs        # 执行器特征
```

### C. 关键文件对照

| 功能 | Nebula-Graph | GraphDB |
|------|-------------|---------|
| Tag 管理 | maintain/TagExecutor.cpp | admin/tag/*.rs |
| Edge 管理 | maintain/EdgeExecutor.cpp | admin/edge/*.rs |
| Space 管理 | admin/SpaceExecutor.cpp | admin/space/*.rs |
| Index 管理 | maintain/*IndexExecutor.cpp | admin/index/*.rs |
| 数据插入 | mutate/InsertExecutor.cpp | data_modification.rs |
| 数据更新 | mutate/UpdateExecutor.cpp | data_modification.rs |
| 数据删除 | mutate/DeleteExecutor.cpp | data_modification.rs |
| 图遍历 | query/TraverseExecutor.cpp | 未实现 |
| 图匹配 | query/PatternApplyExecutor.cpp | 未实现 |
