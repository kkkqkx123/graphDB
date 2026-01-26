已完成


# Context 模块问题清单与修改方案

## 一、上下文层次分析

### 1.1 当前上下文层次

```
QueryContext (执行层核心)
├── RequestContext (请求生命周期)
├── ValidationContext (验证层)
│   ├── BasicValidationContext
│   └── SymbolTable
├── QueryExecutionContext (执行变量)
└── SymbolTable (符号表)

AstContext (AST层核心)
├── qctx: Option<Arc<QueryContext>>
├── statement: Option<Stmt>
├── space: SpaceInfo
└── symbol_table: SymbolTable

QueryAstContext (冗余包装)
├── base: AstContext (真正使用)
├── dependencies: HashMap (未使用)
├── query_variables: HashMap (未使用)
└── expression_contexts: Vec (未使用)
```

### 1.2 各模块对 Context 的使用情况

| 模块 | 使用的 Context | 实际用途 | 评估 |
|------|---------------|---------|------|
| **Parser** | 无需 Context | 输出 Stmt 即可 | ✅ 正确 |
| **Validator** | ValidationContext | 验证语义 | ✅ 正确 |
| **Planner** | AstContext | 规划生成 | ✅ 正确 |
| **Optimizer** | QueryContext | 访问存储和 Schema | ⚠️ 过度 |
| **Executor** | QueryContext | 执行操作 | ✅ 正确 |
| **Scheduler** | 无需 Context | 任务调度 | ✅ 正确 |

### 1.3 核心问题

**AstContext 是真正的核心上下文：**
- 包含对 QueryContext 的引用（可访问运行时资源）
- 包含语句信息（AST）
- 包含符号表
- 包含空间信息
- Planner 直接使用 AstContext

**QueryAstContext 是冗余的：**
- 只是包装 AstContext
- 额外字段（dependencies、query_variables、expression_contexts）未使用
- 应该直接使用 AstContext

**ValidationContext 是独立的：**
- 与 AstContext 的 SymbolTable 功能重叠
- 验证结果未传递给 Planner
- 应该统一到 AstContext

---

## 二、问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 涉及文件 |
|------|----------|----------|----------|----------|
| 2.1 | QueryAstContext 冗余包装 AstContext | 高 | 数据冗余 | `ast/query_ast_context.rs` |
| 2.2 | QueryAstContext 额外字段未使用 | 高 | 代码冗余 | `ast/query_ast_context.rs` |
| 2.3 | ValidationContext 与 AstContext 功能重叠 | 高 | 设计缺陷 | `validate/context.rs` |
| 2.4 | ValidationContext 结果未传递给 Planner | 高 | 数据丢失 | `validator/` |
| 2.5 | 存在多个独立的 SymbolTable | 中 | 数据不一致 | `ast/base.rs`, `validate/context.rs` |
| 2.6 | QueryContext 被过度使用 | 低 | 职责混乱 | `optimizer/` |

---

## 三、详细问题分析

### 问题 2.1: QueryAstContext 冗余包装

**涉及文件**: `src/query/context/ast/query_ast_context.rs`

**当前实现**:
```rust
pub struct QueryAstContext {
    base: AstContext,  // 包装 AstContext
    dependencies: HashMap<String, Vec<String>>,  // 未使用
    query_variables: HashMap<String, VariableInfo>,  // 未使用
    expression_contexts: Vec<ExpressionContext>,  // 未使用
}
```

**使用情况**:
```rust
// QueryPipelineManager 中
let mut ast = QueryAstContext::new(query_text);
ast.set_statement(stmt);  // 设置到 base.AstContext

// Planner 中
fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
    // 直接使用 AstContext
}
```

**分析**:
- QueryAstContext 只是 AstContext 的包装器
- `base_context()` 方法返回 &AstContext，真正使用的是 AstContext
- 额外字段从未被使用

---

### 问题 2.3: ValidationContext 与 AstContext 功能重叠

**涉及文件**: 
- `src/query/context/ast/base.rs`
- `src/query/context/validate/context.rs`

**当前实现**:
```rust
// AstContext 中的符号表
pub struct AstContext {
    pub qctx: Option<Arc<QueryContext>>,
    pub statement: Option<Stmt>,
    pub space: SpaceInfo,
    pub symbol_table: SymbolTable,  // 符号表
}

// ValidationContext 中的符号表
pub struct ValidationContext {
    basic_context: BasicValidationContext,
    symbol_table: SymbolTable,  // 重复的符号表
    // ...
}
```

**问题**:
- 两个上下文都有独立的 SymbolTable
- 验证阶段填充的符号表信息未传递给 Planner
- Planner 无法利用验证结果

---

### 问题 2.4: 验证结果未传递给 Planner

**涉及文件**: `src/query/query_pipeline_manager.rs`

**当前流程**:
```rust
fn execute_query(&mut self, query_text: &str) -> DBResult<ExecutionResult> {
    // 1. 创建查询上下文
    let mut query_context = self.create_query_context(query_text)?;

    // 2. 解析查询
    let ast = self.parse_into_context(query_text)?;  // QueryAstContext

    // 3. 验证查询（使用独立的 ValidationContext）
    self.validate_query(&mut query_context, &ast)?;  // 验证结果丢失

    // 4. 生成执行计划（使用 AstContext，验证结果丢失）
    let execution_plan = self.generate_execution_plan(&mut query_context, &ast)?;
}
```

**问题**:
- 验证阶段的错误、变量信息、类型信息都存储在 ValidationContext
- 验证完成后，这些信息未传递到 Planner
- Planner 无法利用验证阶段的分析结果

---

## 四、修改方案

### 修改方案 4.1: 以 AstContext 为准，删除 QueryAstContext

**预估工作量**: 2-3 人天

**修改策略**: 直接使用 AstContext 作为 Parser 和 Planner 之间的桥梁，删除 QueryAstContext。

**修改步骤**:

**步骤 1**: 修改 Parser 输出

```rust
// src/query/query_pipeline_manager.rs

/// 解析查询文本为 AST 上下文
///
/// 直接生成 AstContext，Parser 输出的 Stmt 会自动设置到上下文中
fn parse_into_context(
    &mut self,
    query_text: &str,
) -> DBResult<AstContext> {
    let mut parser = Parser::new(query_text);
    match parser.parse() {
        Ok(stmt) => {
            let mut ast = AstContext::new(None, Some(stmt));
            ast.set_query_type_from_statement();
            Ok(ast)
        }
        Err(e) => Err(DBError::Query(QueryError::ParseError(
            format!("解析失败: {}", e),
        ))),
    }
}
```

**步骤 2**: 更新 Planner 接口

```rust
// src/query/planner/planner.rs

pub trait Planner {
    fn transform(
        &mut self,
        ast_ctx: &AstContext,  // 直接使用 AstContext
    ) -> Result<SubPlan, PlannerError>;

    fn match_planner(&self, ast_ctx: &AstContext) -> bool;
}
```

**步骤 3**: 删除 QueryAstContext

```rust
// src/query/context/ast/mod.rs

// 删除 QueryAstContext 的导出
// pub use query_ast_context::*;

// 保留 CypherAstContext（如果需要）
pub use cypher_ast_context::{CypherAstContext, VariableVisibility};
```

**步骤 4**: 更新测试

```rust
// 更新所有使用 QueryAstContext 的测试为 AstContext

#[test]
fn test_planner_with_ast_context() {
    let ast = AstContext::from_strings("MATCH", "MATCH (n) RETURN n");
    let planner = MatchPlanner::new();
    assert!(planner.match_planner(&ast));
}
```

---

### 修改方案 4.2: 统一符号表管理

**预估工作量**: 2 人天

**修改策略**: ValidationContext 不再维护独立的 SymbolTable，改为使用 AstContext 中的 SymbolTable。

**修改步骤**:

**步骤 1**: 修改 ValidationContext 结构

```rust
// src/query/context/validate/context.rs

pub struct ValidationContext {
    basic_context: BasicValidationContext,
    schema_manager: Option<Arc<dyn SchemaProvider>>,
    anon_var_gen: AnonVarGenerator,
    anon_col_gen: AnonColGenerator,
    // 删除独立的 symbol_table
    // 使用传入的 AstContext.symbol_table
    schemas: HashMap<String, SchemaInfo>,
    query_parts: Vec<QueryPart>,
    alias_types: HashMap<String, AliasType>,
    validation_errors: Vec<ValidationError>,
}

impl ValidationContext {
    /// 使用外部符号表创建验证上下文
    pub fn with_symbol_table(symbol_table: SymbolTable) -> Self {
        Self {
            basic_context: BasicValidationContext::new(),
            schema_manager: None,
            anon_var_gen: GeneratorFactory::create_anon_var_generator(),
            anon_col_gen: GeneratorFactory::create_anon_col_generator(),
            schemas: HashMap::new(),
            query_parts: Vec::new(),
            alias_types: HashMap::new(),
            validation_errors: Vec::new(),
        }
    }

    /// 获取符号表（从 AstContext 获取）
    pub fn get_symbol_table(&self) -> Option<&SymbolTable> {
        None  // 不再维护独立的符号表
    }
}
```

**步骤 2**: 修改验证器接口

```rust
// src/query/validator/validation_interface.rs

pub trait Validator {
    /// 验证 AST 上下文，验证结果会写入 AstContext.symbol_table
    fn validate(
        &self,
        ast_ctx: &mut AstContext,  // 传入可变的 AstContext
    ) -> Result<(), ValidationError>;
}
```

**步骤 3**: 修改验证器实现

```rust
// src/query/validator/go_validator.rs

impl Validator for GoValidator {
    fn validate(
        &self,
        ast_ctx: &mut AstContext,
    ) -> Result<(), ValidationError> {
        // 1. 从 AstContext 获取语句
        let stmt = ast_ctx.statement()
            .ok_or_else(|| ValidationError::missing_entity("语句", "GO"))?;

        // 2. 验证，并将符号表信息写入 AstContext
        let go_stmt = self.extract_go_statement(stmt)?;
        self.validate_go_statement(go_stmt, ast_ctx)?;

        // 符号表更新会直接写入 ast_ctx.symbol_table
        Ok(())
    }
}
```

---

### 修改方案 4.3: 将验证结果写入 AstContext

**预估工作量**: 3-4 人天

**修改策略**: 验证器将验证结果（变量、类型、别名）写入 AstContext.symbol_table，Planner 直接使用。

**修改步骤**:

**步骤 1**: 扩展 AstContextTrait

```rust
// src/query/context/ast/base.rs

pub trait AstContextTrait {
    fn get_query_context(&self) -> Option<Arc<QueryContext>>;
    fn get_statement(&self) -> Option<Stmt>;
    fn get_space_info(&self) -> SpaceInfo;
    fn lookup_variable(&self, name: &str) -> Option<VariableInfo>;
    
    // 新增：验证结果管理
    fn add_variable(&mut self, name: String, info: VariableInfo);
    fn add_error(&mut self, error: ValidationError);
    fn errors(&self) -> &[ValidationError];
    fn has_errors(&self) -> bool;
}
```

**步骤 2**: 实现验证结果存储

```rust
// src/query/context/ast/base.rs

impl AstContext {
    pub fn new(qctx: Option<Arc<QueryContext>>, sentence: Option<Stmt>) -> Self {
        Self {
            qctx,
            sentence,
            space: SpaceInfo::default(),
            symbol_table: SymbolTable::new(),
            query_type: QueryType::default(),
            // 新增：验证错误存储
            validation_errors: Vec::new(),
        }
    }
    
    // 验证结果管理
    pub fn add_variable(&mut self, name: String, info: VariableInfo) {
        self.symbol_table.add_variable(name, info);
    }
    
    pub fn add_error(&mut self, error: ValidationError) {
        self.validation_errors.push(error);
    }
    
    pub fn errors(&self) -> &[ValidationError] {
        &self.validation_errors
    }
    
    pub fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }
    
    // 清除错误（用于重试）
    pub fn clear_errors(&mut self) {
        self.validation_errors.clear();
    }
}
```

**步骤 3**: 修改验证器写入验证结果

```rust
// src/query/validator/match_validator.rs

impl Validator for MatchValidator {
    fn validate(
        &self,
        ast_ctx: &mut AstContext,
    ) -> Result<(), ValidationError> {
        let stmt = ast_ctx.statement()
            .ok_or_else(|| ValidationError::missing_entity("语句", "MATCH"))?;

        let match_stmt = self.extract_match_statement(stmt)?;

        // 验证 pattern 中的变量
        for pattern in &match_stmt.patterns {
            self.validate_pattern_variables(pattern, ast_ctx)?;
        }

        // 验证 WHERE 子句
        if let Some(where_clause) = &match_stmt.where_clause {
            self.validate_where_clause(where_clause, ast_ctx)?;
        }

        // 验证 RETURN 子句
        if let Some(return_clause) = &match_stmt.return_clause {
            self.validate_return_columns(return_clause, ast_ctx)?;
        }

        Ok(())
    }

    fn validate_pattern_variables(
        &self,
        pattern: &Pattern,
        ast_ctx: &mut AstContext,
    ) -> Result<(), ValidationError> {
        for node in &pattern.nodes {
            // 为每个命名变量添加到符号表
            if let Some(name) = &node.alias {
                let var_info = VariableInfo {
                    name: name.clone(),
                    var_type: VariableType::Vertex,
                    source: Some(node.clone()),
                };
                ast_ctx.add_variable(name.clone(), var_info);
            }
        }
        Ok(())
    }
}
```

---

### 修改方案 4.4: 简化 QueryContext 的使用

**预估工作量**: 2 人天

**修改策略**: 优化器只访问必要的 QueryContext 组件，不传递整个 Context。

**修改步骤**:

**步骤 1**: 定义优化所需的最小接口

```rust
// src/query/context/minimal_context.rs

/// 优化器所需的最小上下文接口
pub trait OptimizationContext {
    fn get_schema(&self, space: &str) -> Option<SchemaInfo>;
    fn get_index(&self, space: &str, column: &str) -> Option<IndexInfo>;
    fn storage_client(&self) -> Option<&dyn StorageClient>;
    fn schema_manager(&self) -> Option<&dyn SchemaManager>;
}
```

**步骤 2**: 实现优化上下文接口

```rust
impl OptimizationContext for QueryContext {
    fn get_schema(&self, space: &str) -> Option<SchemaInfo> {
        self.schema_manager
            .as_ref()
            .and_then(|sm| sm.get_schema(space))
    }
    
    fn get_index(&self, space: &str, column: &str) -> Option<IndexInfo> {
        self.index_manager
            .as_ref()
            .and_then(|im| im.get_index(space, column))
    }
    
    fn storage_client(&self) -> Option<&dyn StorageClient> {
        self.storage_client.as_deref()
    }
    
    fn schema_manager(&self) -> Option<&dyn SchemaManager> {
        self.schema_manager.as_deref()
    }
}
```

**步骤 3**: 修改优化器接口

```rust
// src/query/optimizer/engine/optimizer.rs

impl Optimizer {
    pub fn optimize(
        &mut self,
        plan: ExecutionPlan,
        opt_ctx: &dyn OptimizationContext,  // 使用最小接口
    ) -> Result<ExecutionPlan, OptimizerError> {
        // 使用 opt_ctx 进行优化
        // 不再需要完整的 QueryContext
    }
}
```

---

## 五、删除冗余代码

### 5.1 删除 QueryAstContext

```bash
# 删除文件
rm src/query/context/ast/query_ast_context.rs

# 修改导出
# src/query/context/ast/mod.rs
pub mod base;
pub mod common;
pub mod cypher_ast_context;
pub mod query_types;

pub use base::{AstContext, QueryType, VariableInfo};
pub use common::*;
pub use cypher_ast_context::{CypherAstContext, VariableVisibility};
pub use query_types::*;
```

### 5.2 清理未使用的字段

```rust
// src/query/context/ast/base.rs

// 修改 AstContext，移除不必要的字段
pub struct AstContext {
    pub qctx: Option<Arc<QueryContext>>,
    pub statement: Option<Stmt>,
    pub space: SpaceInfo,
    pub symbol_table: SymbolTable,
    pub query_type: QueryType,
    pub validation_errors: Vec<ValidationError>,  // 新增：存储验证错误
}
```

---

## 六、修改优先级

| 序号 | 修改方案 | 优先级 | 预估工作量 | 依赖 |
|------|----------|--------|------------|------|
| 4.1 | 以 AstContext 为准，删除 QueryAstContext | 高 | 2-3 人天 | 无 |
| 4.2 | 统一符号表管理 | 高 | 2 人天 | 4.1 |
| 4.3 | 将验证结果写入 AstContext | 高 | 3-4 人天 | 4.1, 4.2 |
| 4.4 | 简化 QueryContext 使用 | 低 | 2 人天 | 无 |

---

## 七、修改后的上下文层次

```
QueryContext (运行时资源管理)
├── RequestContext (请求信息)
├── SchemaManager (Schema管理)
├── IndexManager (索引管理)
├── StorageClient (存储访问)
└── SymbolTable (全局符号表)

AstContext (查询级上下文 - 核心)
├── qctx: Arc<QueryContext> (运行时资源引用)
├── statement: Stmt (解析后的语句)
├── space: SpaceInfo (当前图空间)
├── symbol_table: SymbolTable (查询级符号表)
│   └── 验证阶段填充
└── validation_errors: Vec<ValidationError> (验证错误)

ExecutionContext (执行级上下文)
├── variables: HashMap<String, Value> (变量值)
└── execution_metrics: ExecutionMetrics (执行指标)
```

---

## 八、测试建议

### 测试用例 1: AstContext 作为唯一 AST 上下文

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ast_context_as_only_context() {
        let query = "MATCH (n:Player) RETURN n.name";
        let ast = AstContext::from_strings("MATCH", query);
        
        // 验证基本字段
        assert_eq!(ast.query_type(), QueryType::ReadQuery);
        assert!(ast.statement().is_some());
        
        // 验证可设置语句
        let mut ast = AstContext::new(None, None);
        ast.set_statement(Stmt::Match(MatchStmt::default()));
        assert!(ast.statement().is_some());
    }
    
    #[test]
    fn test_variable_management() {
        let mut ast = AstContext::new(None, None);
        
        // 添加变量
        ast.add_variable("n".to_string(), VariableInfo {
            name: "n".to_string(),
            var_type: VariableType::Vertex,
            source: None,
        });
        
        // 验证变量查找
        assert!(ast.lookup_variable("n").is_some());
        assert!(ast.lookup_variable("m").is_none());
    }
}
```

### 测试用例 2: 验证结果传递

```rust
#[test]
fn test_validation_result_passing() {
    let mut ast = AstContext::from_strings(
        "MATCH", 
        "MATCH (n:Player)-[e:PLAY]->(m:Team) RETURN n.name"
    );
    
    // 模拟验证器填充符号表
    ast.add_variable("n".to_string(), VariableInfo {
        name: "n".to_string(),
        var_type: VariableType::Vertex,
        source: None,
    });
    
    ast.add_variable("m".to_string(), VariableInfo {
        name: "m".to_string(),
        var_type: VariableType::Vertex,
        source: None,
    });
    
    // Planner 使用验证后的符号表
    let planner = MatchPlanner::new();
    assert!(planner.match_planner(&ast));
    
    // 验证变量信息已传递
    assert!(ast.lookup_variable("n").is_some());
    assert!(ast.lookup_variable("m").is_some());
}
```

---

## 九、风险与注意事项

### 风险 1: 现有代码兼容性

- **风险**: 大量代码使用 QueryAstContext
- **缓解措施**: 分阶段修改，先修改核心模块
- **实现**: 先修改 QueryPipelineManager 和 Planner，再修改测试

### 风险 2: SymbolTable 线程安全

- **风险**: 验证阶段和规划阶段可能并行访问 SymbolTable
- **缓解措施**: 单线程查询处理模型下，使用 &mut 引用
- **实现**: 明确查询处理的串行模型

### 风险 3: 验证错误丢失

- **风险**: 验证错误未正确传递到执行层
- **缓解措施**: 在 AstContext 中保留验证错误
- **实现**: 执行前检查 ast_ctx.has_errors()
