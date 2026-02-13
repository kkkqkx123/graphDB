# GraphQueryExecutor 未实现方法完整分析报告

## 一、概述

本文档详细分析了 `GraphQueryExecutor` 中未实现的方法，并对照 Nebula-Graph 的实现提供了完整的实现方案。

**文件位置**: `src/query/executor/graph_query_executor.rs`

**分析日期**: 2026-02-13

## 二、未实现方法列表

在 `graph_query_executor.rs#L265-325` 中，以下方法标记为 `#[allow(dead_code)]` 且仅返回未实现错误：

| 方法名 | 语句类型 | 优先级 | 已有实现位置 | 状态 |
|--------|---------|-------|------------|------|
| execute_create | CREATE | 高 | data_modification.rs | 未集成 |
| execute_delete | DELETE | 高 | data_modification.rs | 未集成 |
| execute_update | UPDATE | 高 | data_modification.rs | 未集成 |
| execute_insert | INSERT | 高 | data_modification.rs | 未集成 |
| execute_unwind | UNWIND | 中 | result_processing/transformations/unwind.rs | 未集成 |
| execute_set | SET | 中 | result_processing/transformations/assign.rs | 未集成 |
| execute_use | USE | 高 | admin/space/switch_space.rs | 未集成 |
| execute_show | SHOW | 高 | admin/*/show_*.rs | 未集成 |
| execute_explain | EXPLAIN | 低 | 无 | 需实现 |
| execute_merge | MERGE | 低 | 无 | 需实现 |
| execute_subgraph | SUBGRAPH | 低 | 无 | 需实现 |
| execute_return | RETURN | 低 | 无 | 需实现 |
| execute_with | WITH | 低 | 无 | 需实现 |
| execute_remove | REMOVE | 低 | 无 | 需实现 |
| execute_pipe | PIPE | 低 | 无 | 需实现 |
| execute_query | QUERY | 低 | 无 | 需实现 |
| execute_go | GO | 低 | 无 | 需实现 |
| execute_fetch | FETCH | 低 | data_access.rs | 未集成 |
| execute_lookup | LOOKUP | 低 | 无 | 需实现 |

## 三、Nebula-Graph 架构分析

### 3.1 分层架构

```
Parser (解析层)
  ↓
Planner (规划层) - 生成执行计划
  ↓
Validator (验证层) - 验证语义和权限
  ↓
Executor (执行层) - 实际执行
  ↓
Storage (存储层) - 数据持久化
```

### 3.2 Executor 分类

#### 3.2.1 Admin Executors - 管理操作

- **SpaceExecutor** - 空间管理（创建、删除、显示、切换）
- **SwitchSpaceExecutor** - 切换当前空间
- **TagExecutor** - 标签管理（创建、删除、显示、修改）
- **EdgeExecutor** - 边类型管理（创建、删除、显示、修改）
- **IndexExecutor** - 索引管理（创建、删除、显示、重建）
- **UserExecutor** - 用户管理（创建、删除、修改、授权）

#### 3.2.2 Mutate Executors - 数据修改

- **InsertExecutor** - 插入顶点和边
  - InsertVerticesExecutor - 插入顶点
  - InsertEdgesExecutor - 插入边
- **UpdateExecutor** - 更新顶点和边
  - UpdateVertexExecutor - 更新顶点
  - UpdateEdgeExecutor - 更新边
- **DeleteExecutor** - 删除顶点和边
  - DeleteVerticesExecutor - 删除顶点
  - DeleteEdgesExecutor - 删除边
  - DeleteTagsExecutor - 删除标签

#### 3.2.3 Query Executors - 查询操作

- **ProjectExecutor** - 投影操作（选择列）
- **AssignExecutor** - 赋值操作
- **UnwindExecutor** - 列表展开
- **FilterExecutor** - 过滤操作
- **SortExecutor** - 排序操作
- **LimitExecutor** - 限制结果数量
- **AggregateExecutor** - 聚合操作
- **DedupExecutor** - 去重操作
- **JoinExecutor** - 连接操作
  - InnerJoinExecutor - 内连接
  - LeftJoinExecutor - 左连接
  - RightJoinExecutor - 右连接
  - FullOuterJoinExecutor - 全外连接
- **SetExecutor** - 集合操作
  - UnionExecutor - 并集
  - IntersectExecutor - 交集
  - MinusExecutor - 差集

#### 3.2.4 Algo Executors - 图算法

- **ShortestPathExecutor** - 最短路径
- **AllPathsExecutor** - 所有路径
- **SubgraphExecutor** - 子图查询

## 四、各语句详细分析

### 4.1 INSERT 语句

#### 4.1.1 Nebula-Graph 实现

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
      ivNode->getSpace(), qctx()->rctx()->session()->id(),
      plan->id(), plan->isProfileEnabled());

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

**关键特性**:
1. 使用异步 Future 模式
2. 通过 StorageClient 与存储层交互
3. 支持批量插入
4. 支持 IF NOT EXISTS 选项
5. 支持忽略已存在的索引
6. 完整的错误处理和完整性检查
7. 性能计时和日志记录

#### 4.1.2 当前 GraphDB 实现

**文件**: `src/query/executor/data_modification.rs`

已有完整的 `InsertExecutor` 实现，支持：
- 顶点插入
- 边插入
- 批量操作
- 错误处理

#### 4.1.3 实现建议

```rust
fn execute_insert(&mut self, clause: InsertStmt) -> Result<ExecutionResult, DBError> {
    use crate::query::executor::data_modification::InsertExecutor;
    use crate::query::parser::ast::stmt::InsertTarget;
    use crate::core::Vertex;
    use crate::core::Edge;
    use crate::core::Value;
    use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
    use crate::expression::DefaultExpressionContext;

    match clause.target {
        InsertTarget::Vertices { tag_name, prop_names, values } => {
            let mut vertices = Vec::new();

            for (vid_expr, prop_values) in values {
                let vid = evaluate_expression_to_value(&vid_expr)?;

                let mut properties = std::collections::HashMap::new();
                for (i, prop_name) in prop_names.iter().enumerate() {
                    if i < prop_values.len() {
                        let prop_value = evaluate_expression_to_value(&prop_values[i])?;
                        properties.insert(prop_name.clone(), prop_value);
                    }
                }

                let vertex = Vertex::new_with_properties(
                    vid,
                    vec![tag_name.clone()],
                    properties,
                );
                vertices.push(vertex);
            }

            let mut executor = InsertExecutor::with_vertices(
                self.id,
                self.storage.clone(),
                vertices,
            );
            executor.open()?;
            executor.execute()
        }
        InsertTarget::Edges { edge_name, prop_names, edges } => {
            let mut edge_list = Vec::new();

            for (src_expr, dst_expr, rank_expr, prop_values) in edges {
                let src = evaluate_expression_to_value(&src_expr)?;
                let dst = evaluate_expression_to_value(&dst_expr)?;
                let rank = match rank_expr {
                    Some(ref r) => evaluate_expression_to_int(r)?,
                    None => 0,
                };

                let mut properties = std::collections::HashMap::new();
                for (i, prop_name) in prop_names.iter().enumerate() {
                    if i < prop_values.len() {
                        let prop_value = evaluate_expression_to_value(&prop_values[i])?;
                        properties.insert(prop_name.clone(), prop_value);
                    }
                }

                let edge = Edge::new(
                    src,
                    dst,
                    edge_name.clone(),
                    rank,
                    properties,
                );
                edge_list.push(edge);
            }

            let mut executor = InsertExecutor::with_edges(
                self.id,
                self.storage.clone(),
                edge_list,
            );
            executor.open()?;
            executor.execute()
        }
    }
}

fn evaluate_expression_to_value(expr: &crate::core::types::expression::Expression) -> Result<crate::core::Value, DBError> {
    let mut context = crate::expression::DefaultExpressionContext::new();
    ExpressionEvaluator::evaluate(expr, &mut context)
        .map_err(|e| DBError::Query(crate::core::error::QueryError::ExecutionError(e.to_string())))
}

fn evaluate_expression_to_int(expr: &crate::core::types::expression::Expression) -> Result<i64, DBError> {
    let value = evaluate_expression_to_value(expr)?;
    match value {
        crate::core::Value::Int(i) => Ok(i),
        _ => Err(DBError::Query(crate::core::error::QueryError::ExecutionError(
            "Expected integer value".to_string()
        )))
    }
}
```

### 4.2 UPDATE 语句

#### 4.2.1 Nebula-Graph 实现

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

#### 4.2.2 当前 GraphDB 实现

**文件**: `src/query/executor/data_modification.rs`

已有完整的 `UpdateExecutor` 实现，支持：
- 顶点更新
- 边更新
- 条件表达式
- UPSERT
- RETURN 子句

#### 4.2.3 实现建议

```rust
fn execute_update(&mut self, clause: UpdateStmt) -> Result<ExecutionResult, DBError> {
    use crate::query::executor::data_modification::{UpdateExecutor, VertexUpdate, EdgeUpdate};
    use crate::query::parser::ast::stmt::UpdateTarget;

    match clause.target {
        UpdateTarget::Vertex(vid_expr) => {
            let vid = evaluate_expression_to_value(&vid_expr)?;

            let mut properties = std::collections::HashMap::new();
            for assignment in &clause.set_clause.assignments {
                let value = evaluate_expression_to_value(&assignment.value)?;
                properties.insert(assignment.property.clone(), value);
            }

            let vertex_updates = vec![VertexUpdate {
                vertex_id: vid,
                properties,
                tags_to_add: None,
                tags_to_remove: None,
            }];

            let mut executor = UpdateExecutor::new(
                self.id,
                self.storage.clone(),
                Some(vertex_updates),
                None,
                clause.where_clause.map(|e| e.to_string()),
            )
            .with_insertable(false)
            .with_space("default".to_string());

            executor.open()?;
            executor.execute()
        }
        UpdateTarget::Edge { src, dst, edge_type, rank } => {
            let src_val = evaluate_expression_to_value(&src)?;
            let dst_val = evaluate_expression_to_value(&dst)?;
            let rank_val = match rank {
                Some(ref r) => Some(evaluate_expression_to_int(r)?),
                None => None,
            };

            let mut properties = std::collections::HashMap::new();
            for assignment in &clause.set_clause.assignments {
                let value = evaluate_expression_to_value(&assignment.value)?;
                properties.insert(assignment.property.clone(), value);
            }

            let edge_updates = vec![EdgeUpdate {
                src: src_val,
                dst: dst_val,
                edge_type: edge_type.unwrap_or_default(),
                rank: rank_val,
                properties,
            }];

            let mut executor = UpdateExecutor::new(
                self.id,
                self.storage.clone(),
                None,
                Some(edge_updates),
                clause.where_clause.map(|e| e.to_string()),
            )
            .with_insertable(false)
            .with_space("default".to_string());

            executor.open()?;
            executor.execute()
        }
        _ => Err(DBError::Query(QueryError::ExecutionError(
            format!("UPDATE {:?} 未实现", clause.target)
        )))
    }
}
```

### 4.3 DELETE 语句

#### 4.3.1 Nebula-Graph 实现

**文件**: `nebula-3.8.0/src/graph/executor/mutate/DeleteExecutor.cpp`

```cpp
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

#### 4.3.2 当前 GraphDB 实现

**文件**: `src/query/executor/data_modification.rs`

已有完整的 `DeleteExecutor` 实现，支持：
- 顶点删除
- 边删除
- 条件删除
- 批量操作

#### 4.3.3 实现建议

```rust
fn execute_delete(&mut self, clause: DeleteStmt) -> Result<ExecutionResult, DBError> {
    use crate::query::executor::data_modification::DeleteExecutor;
    use crate::query::parser::ast::stmt::DeleteTarget;

    match clause.target {
        DeleteTarget::Vertices(vertex_exprs) => {
            let mut vertex_ids = Vec::new();
            for expr in vertex_exprs {
                let vid = evaluate_expression_to_value(&expr)?;
                vertex_ids.push(vid);
            }

            let mut executor = DeleteExecutor::new(
                self.id,
                self.storage.clone(),
                Some(vertex_ids),
                None,
                clause.where_clause.map(|e| e.to_string()),
            );
            executor.open()?;
            executor.execute()
        }
        DeleteTarget::Edges { edge_type, edges } => {
            let mut edge_ids = Vec::new();
            for (src_expr, dst_expr, rank_expr) in edges {
                let src = evaluate_expression_to_value(&src_expr)?;
                let dst = evaluate_expression_to_value(&dst_expr)?;
                let rank = match rank_expr {
                    Some(ref r) => Some(evaluate_expression_to_int(r)?),
                    None => None,
                };
                let edge_type_str = edge_type.clone().unwrap_or_default();
                edge_ids.push((src, dst, edge_type_str));
            }

            let mut executor = DeleteExecutor::new(
                self.id,
                self.storage.clone(),
                None,
                Some(edge_ids),
                clause.where_clause.map(|e| e.to_string()),
            );
            executor.open()?;
            executor.execute()
        }
        _ => Err(DBError::Query(QueryError::ExecutionError(
            format!("DELETE {:?} 未实现", clause.target)
        )))
    }
}
```

### 4.4 UNWIND 语句

#### 4.4.1 Nebula-Graph 实现

**文件**: `nebula-3.8.0/src/graph/executor/query/UnwindExecutor.cpp`

```cpp
folly::Future<Status> UnwindExecutor::execute() {
  SCOPED_TIMER(&execTime_);

  auto *unwind = asNode<Unwind>(node());
  auto &inputRes = ectx_->getResult(unwind->inputVar());
  auto iter = inputRes.iter();
  bool emptyInput = inputRes.valuePtr()->type() == Value::Type::DATASET ? false : true;
  QueryExpressionContext ctx(ectx_);
  auto *unwindExpr = unwind->unwindExpr();

  DataSet ds;
  ds.colNames = unwind->colNames();
  for (; iter->valid(); iter->next()) {
    const Value &list = unwindExpr->eval(ctx(iter.get()));
    std::vector<Value> vals = extractList(list);
    for (auto &v : vals) {
      Row row;
      if (!unwind->fromPipe() && !emptyInput) {
        row = *(iter->row());
      }
      row.values.emplace_back(std::move(v));
      ds.rows.emplace_back(std::move(row));
    }
  }
  return finish(ResultBuilder().value(Value(std::move(ds))).build());
}

std::vector<Value> UnwindExecutor::extractList(const Value &val) {
  std::vector<Value> ret;
  if (val.isList()) {
    auto &list = val.getList();
    ret = list.values;
  } else {
    if (!(val.isNull() || val.empty())) {
      ret.push_back(val);
    }
  }
  return ret;
}
```

**关键特性**:
1. 从输入变量获取数据
2. 对每行数据计算表达式
3. 提取列表并展开
4. 保留原始行数据（如果 fromPipe=false）
5. 处理非列表值（包装为单元素列表）
6. 处理空值和空值

#### 4.4.2 当前 GraphDB 实现

**文件**: `src/query/executor/result_processing/transformations/unwind.rs`

已有完整的 `UnwindExecutor` 实现，支持：
- 列表展开
- 多种输入类型（Values, Vertices, Edges, Paths, DataSet）
- 表达式求值
- fromPipe 选项

#### 4.4.3 实现建议

```rust
fn execute_unwind(&mut self, clause: UnwindStmt) -> Result<ExecutionResult, DBError> {
    use crate::query::executor::result_processing::transformations::unwind::UnwindExecutor;

    let mut executor = UnwindExecutor::new(
        self.id,
        self.storage.clone(),
        "_input".to_string(),
        clause.expression,
        vec![clause.variable.clone()],
        false,
    );
    executor.open()?;
    executor.execute()
}
```

### 4.5 SET 语句

#### 4.5.1 Nebula-Graph 实现

SET 在 Nebula-Graph 中有两种含义：

**1. SET 赋值** - 属性赋值
**文件**: `nebula-3.8.0/src/graph/executor/query/AssignExecutor.h`

```cpp
class AssignExecutor final : public Executor {
 public:
  AssignExecutor(const PlanNode *node, QueryContext *qctx)
      : Executor("AssignExecutor", node, qctx) {}

  folly::Future<Status> execute() override;
};
```

**2. SET 操作** - 集合操作（UNION, INTERSECT, MINUS）
**文件**: `nebula-3.8.0/src/graph/executor/query/SetExecutor.h`

```cpp
Status SetExecutor::checkInputDataSets() {
  auto lIter = ectx_->getResult(setNode->leftInputVar()).iter();
  auto rIter = ectx_->getResult(setNode->rightInputVar()).iter();

  auto& lds = leftData->getDataSet();
  auto& rds = rightData->getDataSet();

  if (LIKELY(lds.colNames == rds.colNames)) {
    colNames_ = lds.colNames;
    return Status::OK();
  }

  return Status::Error("Datasets have different columns...");
}
```

**关键特性**:
1. 集合操作需要检查列名一致性
2. 支持多种集合操作类型
3. 赋值操作支持多个变量同时赋值
4. 表达式求值

#### 4.5.2 当前 GraphDB 实现

- 集合操作在 `data_processing/set_operations/` 中实现
- 赋值操作在 `result_processing/transformations/assign.rs` 中实现

#### 4.5.3 实现建议

```rust
fn execute_set(&mut self, clause: SetStmt) -> Result<ExecutionResult, DBError> {
    use crate::query::executor::result_processing::transformations::assign::AssignExecutor;

    let mut assignments = Vec::new();
    for assignment in clause.assignments {
        assignments.push((assignment.property, assignment.value));
    }

    let mut executor = AssignExecutor::new(
        self.id,
        self.storage.clone(),
        assignments,
    );
    executor.open()?;
    executor.execute()
}
```

### 4.6 USE 语句

#### 4.6.1 Nebula-Graph 实现

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

**关键特性**:
1. 从 MetaClient 获取空间信息
2. 设置当前会话的空间
3. 验证权限
4. 会话状态管理
5. 错误处理和日志记录

#### 4.6.2 当前 GraphDB 实现

**文件**: `src/query/executor/admin/space/switch_space.rs`

已有完整的 `SwitchSpaceExecutor` 实现。

#### 4.6.3 实现建议

```rust
fn execute_use(&mut self, clause: UseStmt) -> Result<ExecutionResult, DBError> {
    use crate::query::executor::admin::space::switch_space::SwitchSpaceExecutor;

    let mut executor = SwitchSpaceExecutor::new(
        self.id,
        self.storage.clone(),
        clause.space,
    );
    executor.open()?;
    executor.execute()
}
```

### 4.7 SHOW 语句

#### 4.7.1 Nebula-Graph 实现

SHOW 语句有多种变体，每种都有专门的 Executor：

**1. SHOW SPACES**
**文件**: `nebula-3.8.0/src/graph/executor/admin/SpaceExecutor.cpp`

```cpp
folly::Future<Status> ShowSpacesExecutor::execute() {
  SCOPED_TIMER(&execTime_);

  return qctx()->getMetaClient()->listSpaces().via(runner())
    .thenValue([this](StatusOr<std::vector<meta::SpaceIdName>> resp) {
      if (!resp.ok()) {
        LOG(WARNING) << "Show spaces failed: " << resp.status();
        return resp.status();
      }

      auto spaceItems = std::move(resp).value();

      DataSet dataSet({"Name"});
      std::set<std::string> orderSpaceNames;
      for (auto &space : spaceItems) {
        if (!PermissionManager::canReadSpace(qctx_->rctx()->session(), space.first).ok()) {
          continue;
        }
        orderSpaceNames.emplace(space.second);
      }

      for (auto &name : orderSpaceNames) {
        Row row;
        row.values.emplace_back(name);
        dataSet.rows.emplace_back(std::move(row));
      }

      return finish(ResultBuilder()
                        .value(Value(std::move(dataSet)))
                        .iter(Iterator::Kind::kDefault)
                        .build());
    });
}
```

**2. SHOW TAGS / SHOW EDGES**
类似的实现，从 MetaClient 获取标签或边类型信息。

**关键特性**:
1. 从 MetaClient 获取元数据
2. 权限过滤
3. 结果排序
4. 格式化为 DataSet

#### 4.7.2 当前 GraphDB 实现

- [show_spaces.rs](src/query/executor/admin/space/show_spaces.rs)
- [show_tags.rs](src/query/executor/admin/tag/show_tags.rs)
- [show_edges.rs](src/query/executor/admin/edge/show_edges.rs)

#### 4.7.3 实现建议

```rust
fn execute_show(&mut self, clause: ShowStmt) -> Result<ExecutionResult, DBError> {
    use crate::query::parser::ast::stmt::ShowTarget;

    match clause.target {
        ShowTarget::Spaces => {
            use crate::query::executor::admin::space::show_spaces::ShowSpacesExecutor;
            let mut executor = ShowSpacesExecutor::new(self.id, self.storage.clone());
            executor.open()?;
            executor.execute()
        }
        ShowTarget::Tags => {
            use crate::query::executor::admin::tag::show_tags::ShowTagsExecutor;
            let mut executor = ShowTagsExecutor::new(self.id, self.storage.clone(), String::new());
            executor.open()?;
            executor.execute()
        }
        ShowTarget::Edges => {
            use crate::query::executor::admin::edge::show_edges::ShowEdgesExecutor;
            let mut executor = ShowEdgesExecutor::new(self.id, self.storage.clone(), String::new());
            executor.open()?;
            executor.execute()
        }
        ShowTarget::Tag(tag_name) => {
            use crate::query::executor::admin::tag::desc_tag::DescTagExecutor;
            let mut executor = DescTagExecutor::new(self.id, self.storage.clone(), String::new(), tag_name);
            executor.open()?;
            executor.execute()
        }
        ShowTarget::Edge(edge_name) => {
            use crate::query::executor::admin::edge::desc_edge::DescEdgeExecutor;
            let mut executor = DescEdgeExecutor::new(self.id, self.storage.clone(), String::new(), edge_name);
            executor.open()?;
            executor.execute()
        }
        ShowTarget::Indexes => {
            Err(DBError::Query(QueryError::ExecutionError(
                "SHOW INDEXES 未实现".to_string()
            )))
        }
        ShowTarget::Index(index_name) => {
            Err(DBError::Query(QueryError::ExecutionError(
                format!("SHOW INDEX {} 未实现", index_name)
            )))
        }
        ShowTarget::Users => {
            Err(DBError::Query(QueryError::ExecutionError(
                "SHOW USERS 未实现".to_string()
            )))
        }
        ShowTarget::Roles => {
            Err(DBError::Query(QueryError::ExecutionError(
                "SHOW ROLES 未实现".to_string()
            )))
        }
    }
}
```

### 4.8 DROP 语句的简化实现分析

#### 4.8.1 当前实现

**文件**: `graph_query_executor.rs#L326-367`

```rust
DropTarget::Tags(tag_names) => {
    // 暂时只处理第一个标签，后续可以扩展为批量处理
    if let Some(tag_name) = tag_names.first() {
        let mut executor = admin_executor::DropTagExecutor::new(
            id, self.storage.clone(), String::new(), tag_name.clone()
        );
        executor.open()?;
        executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
    } else {
        Err(DBError::Query(QueryError::ExecutionError("No tag specified".to_string())))
    }
}
```

#### 4.8.2 Nebula-Graph 实现

Nebula-Graph 支持批量删除：

```cpp
folly::Future<StatusOr<bool>> dropTags(
    GraphSpaceID spaceId,
    const std::vector<std::string>& tagNames,
    bool ifExists) {
  // 批量删除多个标签
}
```

#### 4.8.3 差异分析

| 特性 | 当前实现 | Nebula-Graph |
|------|---------|-------------|
| 批量处理 | 只处理第一个 | 支持批量 |
| 错误处理 | 部分失败全部失败 | 可配置部分成功 |
| IF EXISTS | 未完全实现 | 支持 |
| 返回结果 | 简单成功/失败 | 详细错误信息 |

#### 4.8.4 改进建议

```rust
DropTarget::Tags(tag_names) => {
    let mut results = Vec::new();
    let mut has_error = false;

    for tag_name in tag_names {
        let mut executor = admin_executor::DropTagExecutor::new(
            id, self.storage.clone(), String::new(), tag_name.clone()
        );
        executor.open()?;
        match executor.execute() {
            Ok(_) => results.push((tag_name, Ok(()))),
            Err(e) => {
                results.push((tag_name, Err(e.to_string())));
                has_error = true;
            }
        }
    }

    if clause.if_exists {
        Ok(ExecutionResult::Empty)
    } else if has_error {
        let error_msg = results
            .into_iter()
            .filter_map(|(name, r)| r.err().map(|e| format!("{}: {}", name, e)))
            .collect::<Vec<_>>()
            .join("; ");
        Err(DBError::Query(QueryError::ExecutionError(error_msg)))
    } else {
        Ok(ExecutionResult::Empty)
    }
}
```

### 4.9 EXPLAIN 语句

#### 4.9.1 Nebula-Graph 实现

EXPLAIN 不是单独的 Executor，而是在执行前生成执行计划并返回：

```cpp
Status QueryEngine::explain(QueryContext* rctx, const std::string& query) {
  // 1. 解析
  auto result = parser_->parse(query);

  // 2. 验证
  auto status = validator_->validate(result);

  // 3. 规划
  auto plan = planner_->toPlan(result);

  // 4. 返回执行计划
  return explainPlan(plan);
}
```

#### 4.9.2 实现建议

```rust
fn execute_explain(&mut self, clause: ExplainStmt) -> Result<ExecutionResult, DBError> {
    use crate::query::planner::planner::Planner;

    let mut planner = Planner::new();
    let plan = planner.transform(&AstContext::new(None, Some(*clause.statement)))
        .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

    let plan_text = format_execution_plan(&plan)?;

    println!("执行计划:\n{}", plan_text);

    Ok(ExecutionResult::Empty)
}

fn format_execution_plan(plan: &crate::query::planner::plan::Plan) -> Result<String, DBError> {
    let mut output = String::new();

    fn format_node(node: &crate::query::planner::plan::PlanNode, indent: usize, output: &mut String) {
        let indent_str = "  ".repeat(indent);
        output.push_str(&format!("{}+ {}\n", indent_str, node.name()));
        if let Some(desc) = node.description() {
            output.push_str(&format!("{}  {}\n", indent_str, desc));
        }
        for child in node.children() {
            format_node(child, indent + 1, output);
        }
    }

    if let Some(root) = plan.root() {
        format_node(root, 0, &mut output);
    }

    Ok(output)
}
```

## 五、实现优先级建议

基于功能重要性和实现难度，建议按以下优先级实现：

### 高优先级（核心功能）

1. **INSERT** - 数据插入
   - 已有实现，仅需集成
   - 影响数据写入功能

2. **UPDATE** - 数据更新
   - 已有实现，仅需集成
   - 影响数据修改功能

3. **DELETE** - 数据删除
   - 已有实现，仅需集成
   - 影响数据删除功能

4. **USE** - 空间切换
   - 已有实现，仅需集成
   - 影响多空间支持

5. **SHOW** - 元数据查询
   - 已有实现，仅需集成
   - 影响元数据管理

### 中优先级（查询增强）

6. **UNWIND** - 列表展开
   - 已有实现，仅需集成
   - 影响复杂查询能力

7. **SET** - 属性赋值
   - 已有实现，仅需集成
   - 影响数据修改能力

8. **WITH** - 中间结果处理
   - 需要新实现
   - 影响复杂查询能力

### 低优先级（高级功能）

9. **EXPLAIN** - 执行计划
   - 需要新实现
   - 影响调试和优化

10. **MERGE** - 合并操作
    - 需要新实现
    - 影响数据一致性

11. **SUBGRAPH** - 子图查询
    - 需要新实现
    - 影响图分析能力

12. **PIPE** - 管道操作
    - 需要新实现
    - 影响查询灵活性

## 六、代码复用建议

当前代码中已有大量 Executor 实现，但未在 `GraphQueryExecutor` 中使用：

| 语句 | 已有实现位置 | 使用状态 | 集成难度 |
|------|------------|---------|---------|
| INSERT | data_modification.rs | 未使用 | 低 |
| UPDATE | data_modification.rs | 未使用 | 低 |
| DELETE | data_modification.rs | 未使用 | 低 |
| UNWIND | result_processing/transformations/unwind.rs | 未使用 | 低 |
| SET | result_processing/transformations/assign.rs | 未使用 | 低 |
| USE | admin/space/switch_space.rs | 未使用 | 低 |
| SHOW | admin/*/show_*.rs | 未使用 | 低 |

**建议**:
1. 在 `GraphQueryExecutor` 中直接调用这些已有实现
2. 统一错误处理和资源管理
3. 添加执行上下文传递机制

## 七、架构调整建议

### 7.1 当前问题

1. **职责过重** - 所有语句的执行都在一个类中
2. **代码重复** - 每个方法都要处理 open/execute/close
3. **难以扩展** - 添加新语句需要修改核心类

### 7.2 建议架构

```
GraphQueryExecutor (协调器)
  ↓
  ├─> AdminExecutors (管理操作)
  │   ├─> SpaceExecutor
  │   ├─> TagExecutor
  │   ├─> EdgeExecutor
  │   └─> IndexExecutor
  ├─> MutateExecutors (数据修改)
  │   ├─> InsertExecutor
  │   ├─> UpdateExecutor
  │   └─> DeleteExecutor
  ├─> QueryExecutors (查询操作)
  │   ├─> ProjectExecutor
  │   ├─> AssignExecutor
  │   ├─> UnwindExecutor
  │   └─> FilterExecutor
  └─> AlgoExecutors (算法操作)
      ├─> ShortestPathExecutor
      └─> AllPathsExecutor
```

### 7.3 实现模式

```rust
impl<S: StorageClient> GraphQueryExecutor<S> {
    fn execute_statement(&mut self, statement: Stmt) -> Result<ExecutionResult, DBError> {
        match statement {
            Stmt::Insert(clause) => self.execute_insert(clause),
            Stmt::Update(clause) => self.execute_update(clause),
            Stmt::Delete(clause) => self.execute_delete(clause),
            // ... 其他语句
        }
    }

    fn execute_insert(&mut self, clause: InsertStmt) -> Result<ExecutionResult, DBError> {
        let executor = self.create_insert_executor(clause)?;
        self.run_executor(executor)
    }

    fn run_executor<E: Executor<S>>(&mut self, mut executor: E) -> Result<ExecutionResult, DBError> {
        executor.open()?;
        let result = executor.execute();
        executor.close()?;
        result
    }
}
```

## 八、总结

当前 GraphDB 的实现已经具备了大部分 Executor 的基础实现，但存在以下问题：

1. **未集成** - 已有的 Executor 未在 `GraphQueryExecutor` 中使用
2. **简化实现** - DROP 等语句只实现了部分功能
3. **缺少协调** - 缺少统一的执行流程管理

**核心建议**:
1. 将已有的 Executor 集成到 `GraphQueryExecutor` 中
2. 完善批量处理和错误处理逻辑
3. 添加执行上下文和变量管理机制
4. 按优先级逐步实现未完成的功能

这样的实现既符合 Nebula-Graph 的架构设计，又能充分利用已有的代码基础。
