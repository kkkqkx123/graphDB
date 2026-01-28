## 八、缺失组件清单

本章节整理当前代码库中缺失或实现不完整的组件，包括节点类型、规则类型、验证器等。

### 8.1 缺失的执行器类型

根据对 `ExecutorFactory` 的分析，以下 `PlanNodeEnum` 变体尚未实现对应的执行器：

| 节点类型 | 优先级 | 影响范围 | 状态描述 |
|----------|--------|----------|----------|
| `ScanEdges` | 高 | 边扫描查询 | 返回错误，不支持执行 |
| `GetEdges` | 高 | 边属性获取 | 返回错误，不支持执行 |
| `IndexScan` | 高 | 索引扫描查询 | 返回错误，不支持执行 |
| `Select` | 中 | 条件分支控制 | 返回错误，不支持执行 |

**缺失执行器的影响分析：**

1. **ScanEdges 缺失**：无法执行纯边扫描查询 `MATCH (a)-[e]->(b) RETURN e`，影响图谱探索类查询
2. **GetEdges 缺失**：无法高效获取边属性，影响边过滤和聚合操作
3. **IndexScan 缺失**：无法利用索引加速属性查询，影响带条件的大规模数据查询
4. **Select 缺失**：无法实现复杂的条件分支逻辑，影响流程控制类查询

### 8.2 优化规则实现状态

`rule_enum.rs` 中定义了 33 种优化规则，`rule_registrar.rs` 中已注册全部 33 种。以下是规则的实际实现状态分析：

| 规则类别 | 枚举定义 | 已注册 | 已实现 | 实现状态 |
|----------|----------|--------|--------|----------|
| 逻辑优化规则 | 15 种 | 15 种 | 12 种 | ⚠️ 部分实现 |
| 物理优化规则 | 18 种 | 18 种 | 15 种 | ⚠️ 部分实现 |
| 后优化规则 | 0 种 | 0 种 | 0 种 | ❌ 未定义 |

**逻辑优化规则详细状态：**

| 规则名称 | 注册状态 | 实现文件 | 实现程度 |
|----------|----------|----------|----------|
| `FilterPushDown` | ✅ | [predicate_pushdown.rs](src/query/optimizer/predicate_pushdown.rs) | 完全实现 |
| `PredicatePushDown` | ✅ | [predicate_pushdown.rs](src/query/optimizer/predicate_pushdown.rs) | 完全实现 |
| `ProjectionPushDown` | ✅ | [projection_pushdown.rs](src/query/optimizer/projection_pushdown.rs) | 完全实现 |
| `CombineFilter` | ✅ | [elimination_rules.rs](src/query/optimizer/elimination_rules.rs) | 完全实现 |
| `CollapseProject` | ✅ | [transformation_rules.rs](src/query/optimizer/transformation_rules.rs) | 完全实现 |
| `DedupElimination` | ✅ | [elimination_rules.rs](src/query/optimizer/elimination_rules.rs) | 完全实现 |
| `EliminateFilter` | ✅ | [elimination_rules.rs](src/query/optimizer/elimination_rules.rs) | 完全实现 |
| `EliminateRowCollect` | ✅ | [elimination_rules.rs](src/query/optimizer/elimination_rules.rs) | 完全实现 |
| `RemoveNoopProject` | ✅ | [elimination_rules.rs](src/query/optimizer/elimination_rules.rs) | 完全实现 |
| `EliminateAppendVertices` | ✅ | [elimination_rules.rs](src/query/optimizer/elimination_rules.rs) | 完全实现 |
| `RemoveAppendVerticesBelowJoin` | ✅ | [elimination_rules.rs](src/query/optimizer/elimination_rules.rs) | 完全实现 |
| `TopN` | ✅ | [transformation_rules.rs](src/query/optimizer/transformation_rules.rs) | 完全实现 |
| `MergeGetVerticesAndProject` | ✅ | [operation_merge.rs](src/query/optimizer/operation_merge.rs) | 完全实现 |
| `MergeGetVerticesAndDedup` | ✅ | [operation_merge.rs](src/query/optimizer/operation_merge.rs) | 完全实现 |
| `MergeGetNbrsAndProject` | ✅ | [operation_merge.rs](src/query/optimizer/operation_merge.rs) | 完全实现 |
| `MergeGetNbrsAndDedup` | ✅ | [operation_merge.rs](src/query/optimizer/operation_merge.rs) | 完全实现 |

**物理优化规则详细状态：**

| 规则名称 | 注册状态 | 实现文件 | 实现程度 |
|----------|----------|----------|----------|
| `JoinOptimization` | ✅ | [join_optimization.rs](src/query/optimizer/join_optimization.rs) | 完全实现 |
| `PushLimitDown` | ✅ | [limit_pushdown.rs](src/query/optimizer/limit_pushdown.rs) | 完全实现 |
| `PushLimitDownGetVertices` | ✅ | [limit_pushdown.rs](src/query/optimizer/limit_pushdown.rs) | 完全实现 |
| `PushLimitDownGetNeighbors` | ✅ | [limit_pushdown.rs](src/query/optimizer/limit_pushdown.rs) | 完全实现 |
| `PushLimitDownGetEdges` | ✅ | [limit_pushdown.rs](src/query/optimizer/limit_pushdown.rs) | 完全实现 |
| `PushLimitDownScanVertices` | ✅ | [limit_pushdown.rs](src/query/optimizer/limit_pushdown.rs) | 完全实现 |
| `PushLimitDownScanEdges` | ✅ | [limit_pushdown.rs](src/query/optimizer/limit_pushdown.rs) | 完全实现 |
| `PushLimitDownIndexScan` | ✅ | [limit_pushdown.rs](src/query/optimizer/limit_pushdown.rs) | 完全实现 |
| `PushLimitDownProjectRule` | ✅ | [limit_pushdown.rs](src/query/optimizer/limit_pushdown.rs) | 完全实现 |
| `ScanWithFilterOptimization` | ✅ | [scan_optimization.rs](src/query/optimizer/scan_optimization.rs) | 完全实现 |
| `IndexFullScan` | ✅ | [index_optimization.rs](src/query/optimizer/index_optimization.rs) | 完全实现 |
| `IndexScan` | ✅ | [index_optimization.rs](src/query/optimizer/index_optimization.rs) | 完全实现 |
| `EdgeIndexFullScan` | ✅ | [index_optimization.rs](src/query/optimizer/index_optimization.rs) | 完全实现 |
| `TagIndexFullScan` | ✅ | [index_optimization.rs](src/query/optimizer/index_optimization.rs) | 完全实现 |
| `UnionAllEdgeIndexScan` | ✅ | [index_optimization.rs](src/query/optimizer/index_optimization.rs) | 完全实现 |
| `UnionAllTagIndexScan` | ✅ | [index_optimization.rs](src/query/optimizer/index_optimization.rs) | 完全实现 |
| `OptimizeEdgeIndexScanByFilter` | ✅ | [index_optimization.rs](src/query/optimizer/index_optimization.rs) | 完全实现 |
| `OptimizeTagIndexScanByFilter` | ✅ | [index_optimization.rs](src/query/optimizer/index_optimization.rs) | 完全实现 |

**部分实现但未注册的规则：**

| 规则名称 | 枚举定义 | 注册状态 | 实现文件 | 建议 |
|----------|----------|----------|----------|------|
| `ConstantFolding` | ❌ | ❌ | [constant_folding.rs](src/query/optimizer/constant_folding.rs) | 移至 rule_enum.rs 并注册 |
| `SubQueryOptimization` | ❌ | ❌ | [subquery_optimization.rs](src/query/optimizer/subquery_optimization.rs) | 移至 rule_enum.rs 并注册 |
| `LoopUnrolling` | ❌ | ❌ | [loop_unrolling.rs](src/query/optimizer/loop_unrolling.rs) | 移至 rule_enum.rs 并注册 |
| `PredicateReorder` | ❌ | ❌ | [predicate_reorder.rs](src/query/optimizer/predicate_reorder.rs) | 移至 rule_enum.rs 并注册 |

### 8.3 验证器实现状态

当前 `ValidationFactory` 注册了 27 种验证器，但仅有 14 种实现了专门的验证逻辑：

**专门实现的验证器（14 种）：**

| 验证器 | 实现文件 | 验证功能 |
|--------|----------|----------|
| `UseValidator` | [use_validator.rs](src/query/validator/use_validator.rs) | USE 语句验证 |
| `GetSubgraphValidator` | [get_subgraph_validator.rs](src/query/validator/get_subgraph_validator.rs) | GET SUBGRAPH 语句验证 |
| `LookupValidator` | [lookup_validator.rs](src/query/validator/lookup_validator.rs) | LOOKUP 语句验证 |
| `YieldValidator` | [yield_validator.rs](src/query/validator/yield_validator.rs) | YIELD 子句验证 |
| `UnwindValidator` | [unwind_validator.rs](src/query/validator/unwind_validator.rs) | UNWIND 语句验证 |
| `OrderByValidator` | [order_by_validator.rs](src/query/validator/order_by_validator.rs) | ORDER BY 子句验证 |
| `LimitValidator` | [limit_validator.rs](src/query/validator/limit_validator.rs) | LIMIT 子句验证 |
| `FetchEdgesValidator` | [fetch_edges_validator.rs](src/query/validator/fetch_edges_validator.rs) | FETCH EDGES 语句验证 |
| `FetchVerticesValidator` | [fetch_vertices_validator.rs](src/query/validator/fetch_vertices_validator.rs) | FETCH VERTICES 语句验证 |
| `MatchValidator` | [match_validator.rs](src/query/validator/match_validator.rs) | MATCH 语句验证 |
| `GoValidator` | [go_validator.rs](src/query/validator/go_validator.rs) | GO 语句验证 |
| `PipeValidator` | [pipe_validator.rs](src/query/validator/pipe_validator.rs) | PIPE 语句验证 |
| `SequentialValidator` | [sequential_validator.rs](src/query/validator/sequential_validator.rs) | SEQUENTIAL 语句验证 |
| `SetValidator` | [set_validator.rs](src/query/validator/set_validator.rs) | SET 语句验证 |
| `FindPathValidator` | [find_path_validator.rs](src/query/validator/find_path_validator.rs) | FIND PATH 语句验证 |

**使用默认验证器的语句（13 种）：**

| 语句类型 | 注册名称 | 问题描述 |
|----------|----------|----------|
| INSERT VERTICES | `INSERT_VERTICES` | 缺乏属性完整性检查和类型验证 |
| INSERT EDGES | `INSERT_EDGES` | 缺乏源目顶点存在性检查 |
| UPDATE | `UPDATE` | 缺乏条件表达式验证和幂等性检查 |
| DELETE | `DELETE` | 缺乏级联删除语义验证 |
| CREATE SPACE | `CREATE_SPACE` | 缺乏分区和副本参数验证 |
| DROP SPACE | `DROP_SPACE` | 缺乏确认机制验证 |
| CREATE TAG | `CREATE_TAG` | 缺乏属性定义完整性验证 |
| ALTER TAG | `ALTER_TAG` | 缺乏属性修改合法性验证 |
| DROP TAG | `DROP_TAG` | 缺乏依赖检查 |
| CREATE EDGE | `CREATE_EDGE` | 缺乏属性定义完整性验证 |
| ALTER EDGE | `ALTER_EDGE` | 缺乏属性修改合法性验证 |
| DROP EDGE | `DROP_EDGE` | 缺乏依赖检查 |
| SHOW SPACES/TAGS/EDGES | `SHOW_*` | 缺乏权限验证 |

### 8.4 Stmt 与验证器映射分析

`Stmt` 枚举定义了 25 种语句类型，与 `ValidationFactory` 注册的 27 种验证器存在以下映射问题：

| Stmt 变体 | kind() 返回值 | 验证器注册 | 状态 |
|-----------|---------------|------------|------|
| `Stmt::Query` | `"QUERY"` | 无对应验证器 | ❌ 缺失 |
| `Stmt::Create` | `"CREATE"` | 无对应验证器 | ❌ 缺失 |
| `Stmt::Match` | `"MATCH"` | `"MATCH"` | ✅ 匹配 |
| `Stmt::Delete` | `"DELETE"` | `"DELETE"` | ⚠️ 默认验证 |
| `Stmt::Update` | `"UPDATE"` | `"UPDATE"` | ⚠️ 默认验证 |
| `Stmt::Go` | `"GO"` | `"GO"` | ✅ 匹配 |
| `Stmt::Fetch` | `"FETCH"` | `"FETCH_VERTICES"/"FETCH_EDGES"` | ⚠️ 需拆分 |
| `Stmt::Use` | `"USE"` | `"USE"` | ✅ 匹配 |
| `Stmt::Show` | `"SHOW"` | `"SHOW_SPACES"` 等 | ⚠️ 需拆分 |
| `Stmt::Explain` | `"EXPLAIN"` | 无对应验证器 | ❌ 缺失 |
| `Stmt::Lookup` | `"LOOKUP"` | `"LOOKUP"` | ✅ 匹配 |
| `Stmt::Subgraph` | `"SUBGRAPH"` | `"GET_SUBGRAPH"` | ⚠️ 名称不匹配 |
| `Stmt::FindPath` | `"FIND_PATH"` | `"FIND_PATH"` | ✅ 匹配 |
| `Stmt::Insert` | `"INSERT"` | `"INSERT_VERTICES"/"INSERT_EDGES"` | ⚠️ 需拆分 |
| `Stmt::Merge` | `"MERGE"` | 无对应验证器 | ❌ 缺失 |
| `Stmt::Unwind` | `"UNWIND"` | `"UNWIND"` | ✅ 匹配 |
| `Stmt::Return` | `"RETURN"` | `"YIELD"` | ⚠️ 名称不匹配 |
| `Stmt::With` | `"WITH"` | 无对应验证器 | ❌ 缺失 |
| `Stmt::Set` | `"SET"` | `"SET"` | ✅ 匹配 |
| `Stmt::Remove` | `"REMOVE"` | 无对应验证器 | ❌ 缺失 |
| `Stmt::Pipe` | `"PIPE"` | `"PIPE"` | ✅ 匹配 |
| `Stmt::Drop` | `"DROP"` | `"DROP_*"` | ⚠️ 需拆分 |
| `Stmt::Desc` | `"DESC"` | 无对应验证器 | ❌ 缺失 |
| `Stmt::Alter` | `"ALTER"` | `"ALTER_TAG"/"ALTER_EDGE"` | ⚠️ 需拆分 |
| `Stmt::ChangePassword` | `"CHANGE_PASSWORD"` | 无对应验证器 | ❌ 缺失 |

### 8.5 Visitor 层缺失分析

**PlanNodeVisitor 实现状态：**

[plan_node_visitor.rs](src/query/visitor/plan_node_visitor.rs) 已实现完整的 `PlanNodeVisitor` trait，支持访问所有 60+ 种 `PlanNodeEnum` 变体。✅ 已完成

**StmtVisitor 实现状态：**

当前代码库缺少 `StmtVisitor` trait，无法统一遍历 `Stmt` AST。❌ 缺失

| 访问者类型 | 实现状态 | 功能描述 |
|------------|----------|----------|
| `PlanNodeVisitor` | ✅ 已实现 | 支持 60+ 节点类型的访问 |
| `ExpressionVisitor` | ✅ 已存在 | 支持表达式遍历（多个文件） |
| `StmtVisitor` | ❌ 缺失 | 语句级别的 AST 访问 |
| `PlanNodeTransformer` | ❌ 缺失 | PlanNode 转换支持 |
| `StmtTransformer` | ❌ 缺失 | 语句级别的 AST 转换 |

## 九、问题解决方案

### 9.1 验证策略增强方案

当前验证策略存在的主要问题：
1. 大量语句使用默认验证器，缺乏专门的验证逻辑
2. 缺乏属性完整性、类型检查、依赖关系验证
3. 权限验证尚未实现

**解决方案：实现专门的验证器**

**方案一：逐个实现专门的验证器**

针对缺失的语句类型，实现专门的验证器：

```rust
// 示例：InsertVerticesValidator 实现
pub struct InsertVerticesValidator {
    base: Validator,
    space_id: Option<u32>,
}

impl InsertVerticesValidator {
    pub fn new() -> Self {
        Self {
            base: Validator::new(),
            space_id: None,
        }
    }
}

impl ValidatorImpl for InsertVerticesValidator {
    fn validate_impl(&mut self, stmt: &InsertStmt) -> Result<(), ValidationError> {
        // 1. 检查图空间是否已选择
        self.space_id = self.check_space()?;

        // 2. 验证标签存在性
        self.validate_tag_exists(stmt.tag())?;

        // 3. 验证属性完整性
        self.validate_property_completeness(stmt.tag(), stmt.properties())?;

        // 4. 验证属性类型
        self.validate_property_types(stmt.tag(), stmt.properties())?;

        // 5. 验证 VertexID 格式
        self.validate_vertex_id_format(stmt.vertices())?;

        Ok(())
    }

    fn validate_tag_exists(&self, tag: &TagName) -> Result<(), ValidationError> {
        // 查询元数据验证标签存在性
        let tag_info = self.meta_client().get_tag_info(self.space_id, tag)?;
        if tag_info.is_none() {
            return Err(ValidationError::TagNotFound(tag.clone()));
        }
        Ok(())
    }

    fn validate_property_completeness(
        &self,
        tag: &TagName,
        properties: &[PropertyDef],
    ) -> Result<(), ValidationError> {
        // 检查必需属性是否都已提供
        let tag_props = self.get_tag_properties(tag)?;
        for prop in &tag_props {
            if prop.is_required() && !properties.iter().any(|p| p.name == prop.name) {
                return Err(ValidationError::MissingRequiredProperty(prop.name.clone()));
            }
        }
        Ok(())
    }
}
```

**方案二：实现通用验证框架**

使用组合模式实现更灵活的验证框架：

```rust
// 验证策略组合特质
pub trait ValidationStrategy {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<(), ValidationError>;
}

// 组合验证器
pub struct CompositeValidator {
    strategies: Vec<Box<dyn ValidationStrategy>>,
}

impl CompositeValidator {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn add_strategy<S: ValidationStrategy + 'static>(&mut self, strategy: S) {
        self.strategies.push(Box::new(strategy));
    }
}

impl ValidationStrategy for CompositeValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<(), ValidationError> {
        for strategy in &self.strategies {
            strategy.validate(ctx)?;
        }
        Ok(())
    }
}

// 预定义的验证策略
pub struct ExistenceValidationStrategy;
pub struct TypeCheckValidationStrategy;
pub struct IntegrityValidationStrategy;
pub struct PermissionValidationStrategy;
```

**推荐实施方案：**

建议采用**方案一**，针对高优先级语句逐个实现专门的验证器：
1. `InsertVerticesValidator`：优先级高
2. `InsertEdgesValidator`：优先级高
3. `UpdateValidator`：优先级高
4. `DeleteValidator`：优先级高
5. `CreateSpaceValidator`：优先级中
6. `DropSpaceValidator`：优先级中
7. `MergeValidator`：优先级中

### 9.2 AST 级别访问者支持方案

当前缺少 `StmtVisitor` 和 `StmtTransformer`，导致：
1. 无法进行全局 AST 转换
2. 跨模块的 AST 分析逻辑重复
3. 新增分析逻辑需要修改多处代码

**解决方案：实现 StmtVisitor 和 StmtTransformer**

**方案一：基于现有 PlanNodeVisitor 的设计模式**

```rust
//! Stmt 访问者 trait
//!
//! 提供统一的 Stmt 遍历接口，支持语句级别的分析和转换。

use crate::query::parser::ast::stmt::Stmt;
use crate::query::parser::ast::*;

/// Stmt 访问者 trait
///
/// 提供统一的 Stmt 遍历接口，简化语句级别的分析和转换。
pub trait StmtVisitor {
    /// 访问结果的类型
    type Result;

    /// 访问查询语句
    fn visit_query(&mut self, stmt: &QueryStmt) -> Self::Result {
        self.visit_default()
    }

    /// 访问创建语句
    fn visit_create(&mut self, stmt: &CreateStmt) -> Self::Result {
        self.visit_default()
    }

    /// 访问匹配语句
    fn visit_match(&mut self, stmt: &MatchStmt) -> Self::Result {
        self.visit_default()
    }

    /// 访问删除语句
    fn visit_delete(&mut self, stmt: &DeleteStmt) -> Self::Result {
        self.visit_default()
    }

    /// 访问更新语句
    fn visit_update(&mut self, stmt: &UpdateStmt) -> Self::Result {
        self.visit_default()
    }

    /// 访问插入语句
    fn visit_insert(&mut self, stmt: &InsertStmt) -> Self::Result {
        self.visit_default()
    }

    /// 访问合并语句
    fn visit_merge(&mut self, stmt: &MergeStmt) -> Self::Result {
        self.visit_default()
    }

    /// 默认访问方法
    fn visit_default(&mut self) -> Self::Result;
}

/// Stmt 转换器 trait
///
/// 支持在遍历过程中修改 AST 结构。
pub trait StmtTransformer: StmtVisitor {
    /// 转换查询语句
    fn transform_query(&mut self, stmt: &QueryStmt) -> Option<QueryStmt> {
        None
    }

    /// 转换创建语句
    fn transform_create(&mut self, stmt: &CreateStmt) -> Option<CreateStmt> {
        None
    }

    /// 转换匹配语句
    fn transform_match(&mut self, stmt: &MatchStmt) -> Option<MatchStmt> {
        None
    }

    /// 转换删除语句
    fn transform_delete(&mut self, stmt: &DeleteStmt) -> Option<DeleteStmt> {
        None
    }

    /// 转换更新语句
    fn transform_update(&mut self, stmt: &UpdateStmt) -> Option<UpdateStmt> {
        None
    }

    /// 转换插入语句
    fn transform_insert(&mut self, stmt: &InsertStmt) -> Option<InsertStmt> {
        None
    }
}

/// AST 遍历器实现
///
/// 遍历整个 AST 树，调用访问者的方法。
pub struct AstTraverser;

impl AstTraverser {
    /// 遍历语句并返回访问结果
    pub fn traverse<R: Default>(&self, stmt: &Stmt, visitor: &mut impl StmtVisitor<Result = R>) -> R {
        match stmt {
            Stmt::Query(s) => visitor.visit_query(s),
            Stmt::Create(s) => visitor.visit_create(s),
            Stmt::Match(s) => visitor.visit_match(s),
            Stmt::Delete(s) => visitor.visit_delete(s),
            Stmt::Update(s) => visitor.visit_update(s),
            Stmt::Insert(s) => visitor.visit_insert(s),
            Stmt::Merge(s) => visitor.visit_merge(s),
            // ... 其他语句类型
            _ => visitor.visit_default(),
        }
    }

    /// 深度优先遍历所有子节点
    pub fn traverse_children<R: Default>(
        &self,
        stmt: &Stmt,
        visitor: &mut impl StmtVisitor<Result = R>,
    ) {
        match stmt {
            Stmt::Query(s) => {
                for inner in &s.statements {
                    self.traverse(inner, visitor);
                }
            }
            Stmt::Match(s) => {
                // 遍历 MATCH 的模式部分
                for pattern in s.patterns() {
                    self.traverse_pattern(pattern, visitor);
                }
                // 遍历 WHERE 子句
                if let Some(where_cond) = &s.where_clause {
                    self.traverse_expression(where_cond, visitor);
                }
            }
            // ... 其他语句类型的子节点遍历
            _ => {}
        }
    }
}

/// AST 转换器实现
///
/// 在遍历过程中应用转换逻辑。
pub struct AstTransformer;

impl AstTransformer {
    /// 转换语句并返回转换后的语句
    pub fn transform(
        &self,
        stmt: &Stmt,
        transformer: &mut impl StmtTransformer,
    ) -> Option<Stmt> {
        match stmt {
            Stmt::Query(s) => {
                if let Some(transformed) = transformer.transform_query(s) {
                    return Some(Stmt::Query(transformed));
                }
            }
            Stmt::Create(s) => {
                if let Some(transformed) = transformer.transform_create(s) {
                    return Some(Stmt::Create(transformed));
                }
            }
            // ... 其他语句类型的转换
        }
        None
    }
}
```

**方案二：使用宏简化访问者实现**

```rust
/// 宏：自动实现 StmtVisitor
#[macro_export]
macro_rules! impl_stmt_visitor {
    ($ty:ty, $result:ty) => {
        impl StmtVisitor for $ty {
            type Result = $result;

            fn visit_query(&mut self, stmt: &QueryStmt) -> Self::Result {
                self.visit_default()
            }

            fn visit_create(&mut self, stmt: &CreateStmt) -> Self::Result {
                self.visit_default()
            }

            // ... 其他方法的默认实现
        }
    };
}

/// 使用示例：实现一个收集所有变量名的访问者
pub struct VariableCollector {
    variables: Vec<String>,
}

impl VariableCollector {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
        }
    }
}

impl_stmt_visitor!(VariableCollector, ());

impl StmtVisitor for VariableCollector {
    fn visit_match(&mut self, stmt: &MatchStmt) -> Self::Result {
        for pattern in stmt.patterns() {
            self.collect_from_pattern(pattern);
        }
    }

    fn visit_return(&mut self, stmt: &ReturnStmt) -> Self::Result {
        for item in &stmt.items {
            self.variables.push(item.alias.clone());
        }
    }
}
```

**推荐实施方案：**

建议采用**方案一**，实现完整的 `StmtVisitor` 和 `StmtTransformer` 体系：

**实现优先级：**

1. **第一阶段**：实现 `StmtVisitor` trait 和 `AstTraverser`
2. **第二阶段**：实现 `StmtTransformer` trait 和 `AstTransformer`
3. **第三阶段**：迁移现有的跨模块分析逻辑到访问者模式

**预期收益：**

1. **代码复用**：避免跨模块的 AST 分析逻辑重复
2. **可维护性**：新增分析逻辑只需实现访问者接口
3. **可扩展性**：支持自定义的 AST 转换和分析
4. **一致性**：与现有的 `PlanNodeVisitor` 保持一致的设计模式

## 十、参考文档

- [查询模块操作类型分析](query_operation_type_analysis.md)
- [改进后的架构设计文档](improved_architecture_design.md)
- [分阶段修改计划](phased_modification_plan.md)
- [模块问题与解决方案](modules_issues_and_solutions.md)
