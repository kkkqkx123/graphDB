# DML 语句增强修改方案

## 文档信息
- 创建日期: 2026-02-17
- 对标版本: NebulaGraph 3.8.0
- 目标: 增强当前 GraphDB 项目的 DML 语句功能，缩小与 NebulaGraph 的功能差距

---

## 1. 现状分析

### 1.1 当前实现概览

| 语句类型 | 解析 | 验证 | 规划 | 执行 |
|---------|------|------|------|------|
| INSERT VERTEX | ✅ | ✅ 基础 | ✅ | ❌ |
| INSERT EDGE | ✅ | ✅ 基础 | ✅ | ❌ |
| UPDATE | ✅ | ✅ 基础 | ❌ | ❌ |
| DELETE VERTEX | ✅ | ✅ 基础 | ❌ | ❌ |
| DELETE EDGE | ✅ | ✅ 基础 | ❌ | ❌ |
| DELETE TAG | ✅ | ✅ 基础 | ❌ | ❌ |

### 1.2 主要缺失功能

1. **Schema 完整验证** - 仅检查非空，无类型检查、默认值处理
2. **多 Tag 插入** - INSERT VERTEX 仅支持单 Tag
3. **双向边处理** - 边操作仅单向，无入边/出边自动处理
4. **高级语法** - IF NOT EXISTS、INSERTABLE、YIELD 等
5. **级联删除** - DELETE VERTEX WITH EDGE 未实现
6. **执行层** - 仅有计划节点，无实际执行器

---

## 2. 修改方案

### 2.1 Schema 验证增强 (P0)

#### 2.1.1 新建文件: `src/query/validator/schema_validator.rs`

**功能职责:**
- 属性存在性验证
- 数据类型匹配验证
- 非空约束检查
- 默认值自动填充
- VID 类型验证

**核心结构:**
```rust
pub struct SchemaValidator<'a> {
    schema_manager: &'a dyn SchemaManager,
}

impl<'a> SchemaValidator<'a> {
    pub fn validate_property_exists(...)
    pub fn validate_property_type(...)
    pub fn validate_not_null(...)
    pub fn fill_default_values(...)
    pub fn validate_vid(...)
}
```

**对标 NebulaGraph:**
- `src/graph/util/SchemaUtil.cpp`
- `src/graph/validator/MutateValidator.cpp` 中的 check 方法

---

### 2.2 INSERT 语句增强 (P0)

#### 2.2.1 AST 修改: `src/query/parser/ast/stmt.rs`

**当前定义:**
```rust
pub enum InsertTarget {
    Vertices {
        tag_name: String,        // 单 Tag
        prop_names: Vec<String>,
        values: Vec<(Expression, Vec<Expression>)>,
    },
    Edge { ... },
}
```

**目标定义:**
```rust
pub enum InsertTarget {
    Vertices {
        tag_items: Vec<TagInsertItem>,  // 多 Tag 支持
        values: Vec<VertexValues>,
        if_not_exists: bool,             // 新增
        ignore_existed_index: bool,      // 新增
    },
    Edge {
        edge_name: String,
        prop_names: Vec<String>,
        edges: Vec<EdgeValues>,
        if_not_exists: bool,             // 新增
        ignore_existed_index: bool,      // 新增
    },
}

pub struct TagInsertItem {
    pub tag_name: String,
    pub prop_names: Vec<String>,
}

pub struct VertexValues {
    pub vid: Expression,
    pub prop_values: Vec<Vec<Expression>>, // 每个 Tag 一组值
}

pub struct EdgeValues {
    pub src: Expression,
    pub dst: Expression,
    pub rank: Option<Expression>,
    pub props: Vec<Expression>,
}
```

#### 2.2.2 解析器修改: `src/query/parser/parser/dml_parser.rs`

**新增语法支持:**
```sql
-- 多 Tag 插入
INSERT VERTEX tag1(prop1, prop2), tag2(prop3) VALUES "vid":("v1", "v2", "v3")

-- IF NOT EXISTS
INSERT VERTEX IF NOT EXISTS tag(prop) VALUES "vid":("value")

-- 忽略索引
INSERT VERTEX IGNORE_EXISTED_INDEX tag(prop) VALUES "vid":("value")
```

#### 2.2.3 验证器重构: `src/query/validator/insert_validator.rs`

**合并文件:**
- 合并 `insert_vertices_validator.rs` 和 `insert_edges_validator.rs`

**核心验证逻辑:**
1. 验证 Space 已选择
2. 验证 Tag/EdgeType 存在
3. 验证属性名存在于 Schema
4. 验证属性值数量匹配
5. 验证每个属性值类型
6. 验证非空约束
7. 填充默认值
8. 验证 VID 类型

**输出结构:**
```rust
pub enum ValidatedInsert {
    Vertices {
        space_id: i32,
        tag_ids: Vec<i32>,
        vertices: Vec<ValidatedVertex>,
    },
    Edges {
        space_id: i32,
        edge_type_id: i32,
        edges: Vec<ValidatedEdge>,
    },
}

pub struct ValidatedVertex {
    pub vid: Value,
    pub tags: Vec<ValidatedTag>,
}

pub struct ValidatedTag {
    pub tag_id: i32,
    pub props: Vec<(String, Value)>,
}

pub struct ValidatedEdge {
    pub src: Value,
    pub dst: Value,
    pub rank: i64,
    pub props: Vec<(String, Value)>,
}
```

#### 2.2.4 计划节点增强: `src/query/planner/plan/core/nodes/insert_nodes.rs`

**当前问题:**
- 使用 `space_name: String` 而非 `space_id: i32`
- 使用 `tag_name: String` 而非 `tag_id: i32`
- 存储原始表达式而非验证后的值

**目标结构:**
```rust
#[derive(Debug, Clone)]
pub struct VertexInsertInfo {
    pub space_id: i32,
    pub tag_ids: Vec<i32>,
    pub tag_prop_names: Vec<Vec<String>>,
    pub vertices: Vec<NewVertex>,
}

pub struct NewVertex {
    pub vid: Value,
    pub tags: Vec<NewTag>,
}

pub struct NewTag {
    pub tag_id: i32,
    pub props: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct EdgeInsertInfo {
    pub space_id: i32,
    pub edge_type_id: i32,
    pub prop_names: Vec<String>,
    pub edges: Vec<NewEdge>,
    pub if_not_exists: bool,
    pub ignore_existed_index: bool,
}

pub struct NewEdge {
    pub src: Value,
    pub dst: Value,
    pub rank: i64,
    pub props: Vec<Value>,
}
```

#### 2.2.5 规划器修改: `src/query/planner/statements/insert_planner.rs`

**修改点:**
- 调用新的 `InsertValidator` 进行验证
- 使用验证后的 `ValidatedInsert` 构建计划节点
- 支持多 Tag 插入的计划生成

---

### 2.3 UPDATE 语句增强 (P1)

#### 2.3.1 AST 修改: `src/query/parser/ast/stmt.rs`

**当前定义:**
```rust
pub struct UpdateStmt {
    pub span: Span,
    pub target: UpdateTarget,
    pub set_clause: SetClause,
    pub where_clause: Option<Expression>,
}
```

**目标定义:**
```rust
pub struct UpdateStmt {
    pub span: Span,
    pub target: UpdateTarget,
    pub set_clause: SetClause,
    pub where_clause: Option<Expression>,
    pub insertable: bool,              // 新增: 不存在则插入
    pub yield_clause: Option<YieldClause>, // 新增: RETURN 子句
}

pub enum UpdateTarget {
    Vertex {
        vid: Expression,
        tag_name: Option<String>,  // 可选指定 Tag
    },
    Edge {
        src: Expression,
        dst: Expression,
        edge_type: String,
        rank: Option<Expression>,
    },
}
```

#### 2.3.2 解析器修改: `src/query/parser/parser/dml_parser.rs`

**新增语法支持:**
```sql
-- INSERTABLE 选项
UPDATE VERTEX ON tag "vid" SET prop = value INSERTABLE

-- YIELD 子句
UPDATE VERTEX ON tag "vid" SET prop = value YIELD $^.tag.prop AS new_value

-- 指定 Tag
UPDATE VERTEX ON person "vid" SET name = "new_name"

-- 指定 Rank
UPDATE EDGE ON follow "src" -> "dst" @10 SET since = 2024
```

#### 2.3.3 验证器增强: `src/query/validator/update_validator.rs`

**核心验证逻辑:**
1. 验证 VID/EdgeKey 格式
2. 验证 Tag/EdgeType 存在
3. 验证 SET 属性存在于 Schema
4. 验证属性值类型
5. 验证非空约束
6. 处理 YIELD 子句（验证返回属性）

**输出结构:**
```rust
pub enum ValidatedUpdate {
    Vertex {
        vid: Value,
        tag_id: i32,
        updated_props: Vec<UpdatedProp>,
        insertable: bool,
        return_props: Vec<String>,
        condition: Option<Expression>,
    },
    Edge {
        src: Value,
        dst: Value,
        edge_type_id: i32,
        rank: i64,
        updated_props: Vec<UpdatedProp>,
        insertable: bool,
        return_props: Vec<String>,
        condition: Option<Expression>,
    },
}

pub struct UpdatedProp {
    pub name: String,
    pub value: Value,
}
```

#### 2.3.4 新建计划节点: `src/query/planner/plan/core/nodes/update_nodes.rs`

```rust
define_plan_node! {
    pub struct UpdateVertexNode {
        space_id: i32,
        vid: Value,
        tag_id: i32,
        updated_props: Vec<UpdatedProp>,
        insertable: bool,
        return_props: Vec<String>,
        condition: Option<Expression>,
    }
    enum: UpdateVertex
    input: SingleInputNode
}

define_plan_node! {
    pub struct UpdateEdgeNode {
        space_id: i32,
        src: Value,
        dst: Value,
        edge_type_id: i32,
        rank: i64,
        updated_props: Vec<UpdatedProp>,
        insertable: bool,
        return_props: Vec<String>,
        condition: Option<Expression>,
    }
    enum: UpdateEdge
    input: SingleInputNode
}
```

---

### 2.4 DELETE 语句增强 (P1)

#### 2.4.1 验证器增强: `src/query/validator/delete_validator.rs`

**核心验证逻辑:**
1. 验证 VID/EdgeKey 格式
2. 验证 Tag 存在（DELETE TAG）
3. 处理 WITH EDGE 选项（获取所有 EdgeType）

**输出结构:**
```rust
pub enum ValidatedDelete {
    Vertices(Vec<Value>),
    VerticesWithEdge {
        vids: Vec<Value>,
        edge_types: Vec<EdgeTypeInfo>,
    },
    Edges(Vec<EdgeKey>),
    Tags(Vec<(Value, Vec<i32>)>),  // (vid, tag_ids)
}

pub struct EdgeKey {
    pub src: Value,
    pub dst: Value,
    pub edge_type_id: i32,
    pub rank: i64,
}
```

#### 2.4.2 新建计划节点: `src/query/planner/plan/core/nodes/delete_nodes.rs`

```rust
define_plan_node! {
    pub struct DeleteVerticesNode {
        space_id: i32,
        vids: Vec<Value>,
        with_edge: bool,
        edge_types: Vec<i32>,  // WITH EDGE 时需要
    }
    enum: DeleteVertices
    input: SingleInputNode
}

define_plan_node! {
    pub struct DeleteEdgesNode {
        space_id: i32,
        edge_keys: Vec<EdgeKey>,
    }
    enum: DeleteEdges
    input: SingleInputNode
}

define_plan_node! {
    pub struct DeleteTagsNode {
        space_id: i32,
        vid_tag_pairs: Vec<(Value, Vec<i32>)>,
    }
    enum: DeleteTags
    input: SingleInputNode
}
```

---

### 2.5 执行层实现 (P1)

#### 2.5.1 新建模块: `src/query/executor/mod.rs`

```rust
pub mod insert_executor;
pub mod update_executor;
pub mod delete_executor;

pub struct ExecutionContext<'a> {
    storage: &'a dyn StorageEngine,
}

pub struct ExecutionResult {
    pub affected_rows: usize,
    pub data: Option<Vec<Value>>,
    pub columns: Vec<String>,
}
```

#### 2.5.2 INSERT 执行器: `src/query/executor/insert_executor.rs`

**核心功能:**
- 检查 IF NOT EXISTS
- 插入顶点（多 Tag）
- 插入双向边（出边 + 入边）
- 返回插入数量

```rust
pub struct InsertExecutor<'a> {
    ctx: &'a ExecutionContext<'a>,
}

impl<'a> InsertExecutor<'a> {
    pub fn execute_vertices(&self, node: &InsertVerticesNode) -> Result<ExecutionResult>
    pub fn execute_edges(&self, node: &InsertEdgesNode) -> Result<ExecutionResult>
}
```

#### 2.5.3 UPDATE 执行器: `src/query/executor/update_executor.rs`

**核心功能:**
- 查找顶点/边
- 处理 INSERTABLE（不存在则创建）
- 更新属性
- 处理 YIELD（返回新值）

#### 2.5.4 DELETE 执行器: `src/query/executor/delete_executor.rs`

**核心功能:**
- 删除顶点
- 级联删除边（WITH EDGE）
- 删除边（双向）
- 删除 Tag

---

## 3. 修改优先级

### P0 - 核心功能（必须实现）

| 序号 | 功能 | 文件 | 工作量 |
|-----|------|------|--------|
| 1 | Schema 验证工具模块 | `schema_validator.rs` (新建) | 中等 |
| 2 | INSERT 多 Tag 支持 | `stmt.rs`, `dml_parser.rs` | 中等 |
| 3 | INSERT 验证器重构 | `insert_validator.rs` (合并) | 中等 |
| 4 | 双向边插入 | `insert_executor.rs` | 小 |
| 5 | 计划节点 ID 化 | `insert_nodes.rs` | 小 |

### P1 - 重要功能（建议实现）

| 序号 | 功能 | 文件 | 工作量 |
|-----|------|------|--------|
| 6 | UPDATE INSERTABLE | `stmt.rs`, `update_validator.rs` | 小 |
| 7 | UPDATE YIELD | `stmt.rs`, `update_validator.rs` | 中等 |
| 8 | DELETE WITH EDGE | `delete_validator.rs` | 中等 |
| 9 | 执行层基础框架 | `executor/mod.rs` | 中等 |
| 10 | IF NOT EXISTS | `stmt.rs`, `dml_parser.rs` | 小 |

### P2 - 增强功能（可选实现）

| 序号 | 功能 | 文件 | 工作量 |
|-----|------|------|--------|
| 11 | 默认值填充 | `schema_validator.rs` | 中等 |
| 12 | 类型严格检查 | `schema_validator.rs` | 中等 |
| 13 | IGNORE_EXISTED_INDEX | `stmt.rs` | 小 |
| 14 | EXPLAIN 支持 | 各计划节点 | 中等 |

### P3 - 高级功能（未来实现）

| 序号 | 功能 | 文件 | 工作量 |
|-----|------|------|--------|
| 15 | 批量操作优化 | 执行层 | 大 |
| 16 | 事务支持 | 存储层 | 大 |
| 17 | 异步执行 | 执行层 | 大 |

---

## 4. 实施步骤

### 阶段 1: Schema 验证基础 (Week 1)

1. **Day 1-2**: 创建 `schema_validator.rs`
   - 实现属性存在性验证
   - 实现类型匹配验证
   - 实现 VID 验证

2. **Day 3-4**: 修改 AST 定义
   - 更新 `InsertTarget` 支持多 Tag
   - 添加 `if_not_exists` 和 `ignore_existed_index`

3. **Day 5**: 修改解析器
   - 支持多 Tag 语法
   - 支持 IF NOT EXISTS 语法

### 阶段 2: INSERT 增强 (Week 2)

1. **Day 1-2**: 重构 INSERT 验证器
   - 合并 vertices 和 edges 验证器
   - 集成 SchemaValidator
   - 实现多 Tag 验证

2. **Day 3**: 优化计划节点
   - 修改 `insert_nodes.rs` 使用 ID
   - 添加预处理数据结构

3. **Day 4-5**: 实现 INSERT 执行器
   - 创建 `insert_executor.rs`
   - 实现双向边插入

### 阶段 3: UPDATE 增强 (Week 3)

1. **Day 1-2**: 修改 AST 和解析器
   - 添加 `insertable` 和 `yield_clause`
   - 支持新语法

2. **Day 3-4**: 增强 UPDATE 验证器
   - 支持 INSERTABLE
   - 支持 YIELD 验证

3. **Day 5**: 创建 UPDATE 计划节点和执行器

### 阶段 4: DELETE 增强 (Week 4)

1. **Day 1-2**: 增强 DELETE 验证器
   - 支持 WITH EDGE

2. **Day 3-4**: 创建 DELETE 计划节点和执行器

3. **Day 5**: 集成测试

---

## 5. 测试策略

### 5.1 单元测试

每个验证器、执行器都需要独立的单元测试：

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_validate_multi_tag_insert() { ... }
    
    #[test]
    fn test_validate_if_not_exists() { ... }
    
    #[test]
    fn test_validate_type_mismatch() { ... }
    
    #[test]
    fn test_validate_not_null_constraint() { ... }
}
```

### 5.2 集成测试

创建 `tests/dml_integration_test.rs`:

```rust
#[test]
fn test_insert_vertex_and_fetch() { ... }

#[test]
fn test_update_with_yield() { ... }

#[test]
fn test_delete_vertex_with_edge() { ... }
```

### 5.3 边界条件测试

- NULL 值处理
- 默认值填充
- 类型转换边界
- 空值/空列表
- 超长字符串

---

## 6. 风险评估

### 6.1 技术风险

| 风险 | 影响 | 缓解措施 |
|-----|------|----------|
| Schema 验证性能 | 中 | 缓存 Schema 信息 |
| 双向边一致性 | 高 | 事务包装或补偿机制 |
| 内存使用 | 中 | 流式处理大数据量 |

### 6.2 兼容性风险

- 语法变更可能影响现有查询
- 建议保留向后兼容的解析

---

## 7. 参考文档

### 7.1 NebulaGraph 源码参考

- `src/graph/validator/MutateValidator.cpp`
- `src/graph/planner/plan/Mutate.h`
- `src/graph/executor/mutate/InsertExecutor.cpp`
- `src/graph/executor/mutate/UpdateExecutor.cpp`
- `src/graph/executor/mutate/DeleteExecutor.cpp`

### 7.2 相关文档

- NebulaGraph DML 语法文档
- 当前项目 Schema 设计文档
- 存储层接口文档

---

## 8. 附录

### 8.1 命名对照表

| 当前项目 | NebulaGraph | 说明 |
|---------|-------------|------|
| Tag | Tag | 顶点标签 |
| EdgeType | EdgeType | 边类型 |
| Space | Space | 图空间 |
| Property | Property/Column | 属性/列 |
| VID | VertexID | 顶点标识 |

### 8.2 类型对照表

| 当前项目 | NebulaGraph | 说明 |
|---------|-------------|------|
| DataType::String | Value::STRING | 字符串 |
| DataType::Int | Value::INT | 整数 |
| DataType::Float | Value::FLOAT | 浮点数 |
| DataType::Bool | Value::BOOL | 布尔 |
| DataType::Null | Value::NULL | 空值 |

---

**文档结束**
