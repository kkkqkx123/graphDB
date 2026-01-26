# GraphDB Query 模块问题清单与修改方案

## 概述

本文档对 GraphDB 查询引擎各模块进行系统性分析，列出每个模块存在的问题及其严重程度，并提供详细的修改方案。问题按照优先级排序，修改方案包含实现细节和预估工作量。

---

## 一、QueryPipelineManager 模块

**模块路径**: `src/query/query_pipeline_manager.rs`

### 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 1.1 | `validate_query` 函数中 `query_context` 参数未使用 | 高 | 设计缺陷 | 待修复 |
| 1.2 | `generate_execution_plan` 函数中 `query_context` 参数未使用 | 高 | 设计缺陷 | 待修复 |
| 1.3 | UUID 生成使用 `from_ne_bytes` 方式不标准 | 中 | 代码质量问题 | 待修复 |
| 1.4 | 错误处理重复使用 `format!` 字符串拼接 | 低 | 性能问题 | 待修复 |
| 1.5 | 缺乏查询处理各阶段的性能监控 | 低 | 缺失功能 | 待修复 |

### 详细问题分析

#### 问题 1.1: validate_query 函数中 query_context 参数未使用

**问题代码**:
```rust
fn validate_query(
    &mut self,
    _query_context: &mut QueryContext,  // 未使用
    ast: &QueryAstContext,
) -> DBResult<()> {
    self.validator.validate_unified()...
}
```

**影响范围**:
- 验证阶段无法利用查询上下文中的信息
- 调用者可能期望上下文被更新，但实际没有

#### 问题 1.2: generate_execution_plan 函数中 query_context 参数未使用

**问题代码**:
```rust
fn generate_execution_plan(
    &mut self,
    _query_context: &mut QueryContext,  // 未使用
    ast: &QueryAstContext,
) -> DBResult<ExecutionPlan> {
    let ast_ctx = ast.base_context();  // 只使用 AstContext
    self.planner.transform(ast_ctx)...
}
```

**影响范围**:
- 规划器无法获取完整的查询上下文信息
- `query_variables`、`expression_contexts` 等信息丢失

#### 问题 1.3: UUID 生成方式不标准

**问题代码**:
```rust
let uuid = uuid::Uuid::new_v4();
let uuid_bytes = uuid.as_bytes();
let id = i64::from_ne_bytes([
    uuid_bytes[0], uuid_bytes[1], uuid_bytes[2], uuid_bytes[3],
    uuid_bytes[4], uuid_bytes[5], uuid_bytes[6], uuid_bytes[7],
]);
```

**问题**:
- 只使用 UUID 的前 8 字节，存在碰撞风险
- 不是标准的使用方式

### 修改方案

#### 修改方案 1.1-1.2: 重构数据传递

**预估工作量**: 2-3 人天

**修改步骤**:

1. 修改 `validate_query` 函数签名和实现：
```rust
/// 验证查询的语义正确性
///
/// # 参数
/// * `query_context` - 将被更新以包含验证结果
/// * `ast` - 解析后的 AST 上下文
///
/// # 返回
/// * 验证后的 QueryAstContext（包含验证结果）
fn validate_query(
    &mut self,
    query_context: &mut QueryContext,
    ast: &mut QueryAstContext,
) -> DBResult<()> {
    // 使用 query_context 进行验证
    let validation_result = self.validator.validate_with_context(
        query_context,
        ast
    )?;
    
    // 将验证结果存储到 ast 中
    ast.set_validation_result(validation_result);
    
    Ok(())
}
```

2. 修改 `generate_execution_plan` 函数签名和实现：
```rust
/// 生成执行计划
///
/// # 参数
/// * `query_context` - 查询上下文（将被传递给规划器）
/// * `ast` - 已验证的 AST 上下文
///
/// # 返回
/// * 生成的执行计划
fn generate_execution_plan(
    &mut self,
    query_context: &mut QueryContext,
    ast: &QueryAstContext,
) -> DBResult<ExecutionPlan> {
    // 使用完整的 QueryAstContext
    self.planner.transform_with_context(query_context, ast)
}
```

3. 修改 `Planner` trait：
```rust
pub trait Planner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;
    fn transform_with_context(
        &mut self,
        query_context: &mut QueryContext,
        ast_ctx: &QueryAstContext,
    ) -> Result<ExecutionPlan, PlannerError>;
}
```

#### 修改方案 1.3: 改进 UUID 生成

**预估工作量**: 0.5 人天

**修改代码**:
```rust
/// 生成唯一的计划 ID
fn generate_plan_id() -> i64 {
    // 使用更安全的方式生成 ID
    let uuid = uuid::Uuid::new_v4();
    // 将 UUID 转换为 i128，然后取模以适应 i64 范围
    let uuid_i128 = uuid.as_u128();
    (uuid_i128 % i64::MAX as u128) as i64
}
```

#### 修改方案 1.5: 添加性能监控

**预估工作量**: 1-2 人天

**修改代码**:
```rust
use std::time::{Duration, Instant};

pub struct QueryPipelineMetrics {
    pub parse_duration: Duration,
    pub validate_duration: Duration,
    pub plan_duration: Duration,
    pub optimize_duration: Duration,
    pub execute_duration: Duration,
}

impl<S: StorageEngine + 'static> QueryPipelineManager<S> {
    pub async fn execute_query_with_metrics(
        &mut self,
        query_text: &str,
    ) -> DBResult<(ExecutionResult, QueryPipelineMetrics)> {
        let start = Instant::now();
        
        // 各阶段计时
        let parse_start = Instant::now();
        let mut query_context = self.create_query_context(query_text)?;
        let ast = self.parse_into_context(query_text)?;
        let parse_duration = parse_start.elapsed();
        
        let validate_start = Instant::now();
        self.validate_query(&mut query_context, &ast)?;
        let validate_duration = validate_start.elapsed();
        
        let plan_start = Instant::now();
        let execution_plan = self.generate_execution_plan(&mut query_context, &ast)?;
        let plan_duration = plan_start.elapsed();
        
        let optimize_start = Instant::now();
        let optimized_plan = self.optimize_execution_plan(&mut query_context, execution_plan)?;
        let optimize_duration = optimize_start.elapsed();
        
        let execute_start = Instant::now();
        let result = self.execute_plan(&mut query_context, optimized_plan).await?;
        let execute_duration = execute_start.elapsed();
        
        let metrics = QueryPipelineMetrics {
            parse_duration,
            validate_duration,
            plan_duration,
            optimize_duration,
            execute_duration,
        };
        
        Ok((result, metrics))
    }
}
```

---

## 二、Parser 模块

**模块路径**: `src/query/parser/`

### 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 2.1 | 错误信息缺乏位置详细信息 | 中 | 可用性问题 | 待修复 |
| 2.2 | 表达式解析器不支持所有 NGQL 语法 | 中 | 功能缺失 | 待修复 |
| 2.3 | Token 类型定义不完整 | 低 | 完整性问题 | 待修复 |
| 2.4 | 词法分析器错误处理不够友好 | 低 | 代码质量 | 待修复 |

### 详细问题分析

#### 问题 2.1: 错误信息缺乏位置详细信息

**当前实现**:
```rust
Err(e) => Err(DBError::Query(QueryError::ParseError(
    format!("解析失败: {}", e),
))),
```

**问题**:
- 没有包含具体的行号和列号
- 用户难以定位语法错误位置

#### 问题 2.2: 表达式解析器支持不完整

**当前实现**: 只支持部分表达式类型

**缺失功能**:
- CASE 表达式
- 列表推导式
- 路径表达式

### 修改方案

#### 修改方案 2.1: 改进错误信息

**预估工作量**: 1 人天

**修改代码**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("语法错误 at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },
    
    #[error("词法错误 at line {line}, column {column}: {message}")]
    LexerError {
        line: usize,
        column: usize,
        message: String,
    },
    
    #[error("解析失败: {message}")]
    ParseError {
        message: String,
    },
}

impl ParseError {
    pub fn with_position(line: usize, column: usize, message: String) -> Self {
        ParseError::SyntaxError { line, column, message }
    }
}
```

#### 修改方案 2.2: 扩展表达式解析器

**预估工作量**: 3-5 人天

**修改步骤**:

1. 添加缺失的表达式类型定义：
```rust
// 在 ast/types.rs 中添加
#[derive(Debug, Clone)]
pub enum Expression {
    // ... 现有类型
    Case {
        test_expr: Option<Box<Expression>>,
        when_then_pairs: Vec<(Expression, Expression)>,
        default: Option<Box<Expression>>,
    },
    ListComprehension {
        variable: String,
        source: Box<Expression>,
        filter: Option<Box<Expression>>,
        map: Option<Box<Expression>>,
    },
    Path {
        elements: Vec<PathElement>,
    },
}
```

2. 添加表达式解析逻辑：
```rust
impl ExprParser {
    fn parse_case_expression(&mut self) -> Result<Expression, ParseError> {
        // 解析 CASE 表达式
        self.expect(TokenKind::Case)?;
        // ...
    }
    
    fn parse_list_comprehension(&mut self) -> Result<Expression, ParseError> {
        // 解析列表推导式
        self.expect(TokenKind::LeftBracket)?;
        // ...
    }
}
```

---

## 三、Validator 模块

**模块路径**: `src/query/validator/`

### 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 3.1 | 使用独立的 ValidationContext 而非 QueryAstContext | 高 | 架构问题 | 待修复 |
| 3.2 | 验证结果未存储回 AST 上下文 | 高 | 数据丢失 | 待修复 |
| 3.3 | validate_unified 方法不接收参数 | 中 | 设计缺陷 | 待修复 |
| 3.4 | 错误类型与 DBError 不统一 | 中 | 一致性问题 | 待修复 |
| 3.5 | 验证器工厂配置不够灵活 | 低 | 扩展性问题 | 待修复 |

### 详细问题分析

#### 问题 3.1-3.2: 验证上下文与 AST 上下文分离

**问题代码**:
```rust
// base_validator.rs
pub struct Validator {
    context: ValidationContext,  // 独立创建
    // ...
}

fn validate_query(&mut self, _query_context: &mut QueryContext, ast: &QueryAstContext) {
    self.validator.validate_unified()...  // 使用内部 ValidationContext
}
```

**影响**:
- Parser 生成的 AST 信息未被充分利用
- 验证状态无法传递给 Planner
- 数据需要在多个上下文之间同步

### 修改方案

#### 修改方案 3.1-3.3: 重构验证流程

**预估工作量**: 4-6 人天

**修改步骤**:

1. 修改 Validator 结构体：
```rust
pub struct Validator {
    // 接收 QueryAstContext 而非使用内部 ValidationContext
    ast_context: Option<QueryAstContext>,
    // ...
}

impl Validator {
    /// 使用 AST 上下文进行验证
    pub fn validate_with_ast_context(
        &mut self,
        ast: &mut QueryAstContext,
    ) -> Result<(), ValidationError> {
        self.ast_context = Some(ast.clone());
        self.validate_impl()?;
        Ok(())
    }
    
    /// 将验证结果应用到 AST 上下文
    pub fn apply_validation_result(&self, ast: &mut QueryAstContext) {
        // 将验证信息写入 AST 上下文
        ast.set_outputs(self.outputs().to_vec());
        ast.set_inputs(self.inputs().to_vec());
        ast.set_validation_errors(self.context.get_validation_errors());
    }
}
```

2. 修改 validate_query 函数：
```rust
fn validate_query(
    &mut self,
    query_context: &mut QueryContext,
    ast: &mut QueryAstContext,
) -> DBResult<()> {
    // 使用 AST 上下文进行验证
    self.validator.validate_with_ast_context(ast)?;
    
    // 应用验证结果
    self.validator.apply_validation_result(ast);
    
    // 检查验证错误
    if ast.has_validation_errors() {
        let errors = ast.get_validation_errors();
        return Err(DBError::Query(QueryError::InvalidQuery(
            format!("验证失败: {:?}", errors)
        )));
    }
    
    Ok(())
}
```

3. 增强 QueryAstContext：
```rust
impl QueryAstContext {
    /// 设置验证输出
    pub fn set_outputs(&mut self, outputs: Vec<ColumnDef>) {
        self.base.set_outputs(outputs);
    }
    
    /// 设置验证输入
    pub fn set_inputs(&mut self, inputs: Vec<ColumnDef>) {
        self.base.set_inputs(inputs);
    }
    
    /// 设置验证错误
    pub fn set_validation_errors(&mut self, errors: Vec<ValidationError>) {
        self.base.set_validation_errors(errors);
    }
    
    /// 检查是否有验证错误
    pub fn has_validation_errors(&self) -> bool {
        self.base.has_validation_errors()
    }
    
    /// 获取验证错误
    pub fn get_validation_errors(&self) -> &[ValidationError] {
        self.base.get_validation_errors()
    }
}
```

#### 修改方案 3.4: 统一错误类型

**预估工作量**: 2 人天

**修改代码**:
```rust
// 在 validation_interface.rs 中使用 thiserror
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("语义错误: {message} (位置: {location})")]
    SemanticError {
        message: String,
        location: Option<String>,
    },
    
    #[error("类型错误: {message}")]
    TypeError {
        message: String,
        expected: ValueType,
        actual: ValueType,
    },
    
    #[error("语法错误: {message}")]
    SyntaxError {
        message: String,
    },
    
    #[error("权限错误: {message}")]
    PermissionError {
        message: String,
    },
}

// 实现 From<ValidationError> for DBError
impl From<ValidationError> for DBError {
    fn from(e: ValidationError) -> Self {
        DBError::Query(QueryError::InvalidQuery(e.to_string()))
    }
}
```

---

## 四、Planner 模块

**模块路径**: `src/query/planner/`

### 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 4.1 | 使用 AstContext 而非完整的 QueryAstContext | 高 | 数据丢失 | 待修复 |
| 4.2 | MatchPlanner 实现不完整 | 高 | 功能缺失 | 待修复 |
| 4.3 | 规划器注册机制缺乏动态配置 | 中 | 扩展性问题 | 待修复 |
| 4.4 | 计划节点 ID 生成方式不标准 | 低 | 代码质量 | 待修复 |
| 4.5 | 缺乏计划缓存机制 | 低 | 性能问题 | 待修复 |

### 详细问题分析

#### 问题 4.1: 使用子集上下文

**问题代码**:
```rust
fn generate_execution_plan(&mut self, _query_context: &mut QueryContext, ast: &QueryAstContext) {
    let ast_ctx = ast.base_context();  // 只使用 AstContext
    self.planner.transform(ast_ctx)...
}
```

**丢失的信息**:
- `query_variables`: 变量信息
- `expression_contexts`: 表达式上下文
- `dependencies`: 依赖关系

#### 问题 4.2: MatchPlanner 实现不完整

**当前实现**:
```rust
impl Planner for MatchPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let stmt = ast_ctx.sentence().ok_or_else(|| {
            PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
        })?;
        
        let space_id = ast_ctx.space.space_id.unwrap_or(1) as i32;
        let start_node = ScanVerticesNode::new(space_id);
        let mut current_plan = SubPlan::from_root(start_node.into_enum());
        
        if ast_ctx.query_type() == QueryType::ReadQuery {
            // 空实现
        }
        
        Ok(current_plan)
    }
}
```

**问题**:
- MATCH 语句的 pattern、where、return 等子句未被处理
- 生成的计划过于简单

### 修改方案

#### 修改方案 4.1: 使用完整上下文

**预估工作量**: 3-4 人天

**修改步骤**:

1. 修改 Planner trait：
```rust
pub trait Planner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;
    
    /// 使用完整上下文进行规划
    fn transform_with_full_context(
        &mut self,
        query_context: &mut QueryContext,
        ast: &QueryAstContext,
    ) -> Result<ExecutionPlan, PlannerError> {
        // 默认实现：提取 AstContext 进行规划
        let ast_ctx = ast.base_context();
        let sub_plan = self.transform(ast_ctx)?;
        Ok(ExecutionPlan::new(sub_plan.root().clone()))
    }
}
```

2. 修改 MatchPlanner 实现：
```rust
impl MatchPlanner {
    fn transform_with_full_context(
        &mut self,
        query_context: &mut QueryContext,
        ast: &QueryAstContext,
    ) -> Result<ExecutionPlan, PlannerError> {
        let stmt = ast.base_context().sentence().ok_or_else(|| {
            PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
        })?;
        
        // 使用 QueryAstContext 中的信息
        let space_id = ast.base_context().space.space_id.unwrap_or(1) as i32;
        
        // 处理 MATCH 语句的各个部分
        let mut current_plan = self.plan_match_clause(ast, stmt, space_id)?;
        
        // 处理 WHERE 子句
        if let Some(where_condition) = self.extract_where_condition(ast, stmt) {
            current_plan = self.plan_filter(current_plan, where_condition, space_id)?;
        }
        
        // 处理 RETURN 子句
        if let Some(return_columns) = self.extract_return_columns(ast, stmt) {
            current_plan = self.plan_project(current_plan, return_columns, space_id)?;
        }
        
        Ok(ExecutionPlan::from_sub_plan(current_plan))
    }
    
    fn plan_match_clause(
        &self,
        ast: &QueryAstContext,
        stmt: &Stmt,
        space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        // 解析 MATCH pattern，生成扫描或索引查找节点
        let pattern = self.extract_match_pattern(ast, stmt)?;
        let start_node = self.plan_pattern(&pattern, space_id)?;
        Ok(SubPlan::from_root(start_node))
    }
    
    fn plan_filter(
        &self,
        input_plan: SubPlan,
        condition: Expression,
        space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        let filter_node = FilterNode::new(input_plan.root().clone(), condition)?;
        Ok(input_plan.with_root(filter_node.into_enum()))
    }
    
    fn plan_project(
        &self,
        input_plan: SubPlan,
        columns: Vec<YieldColumn>,
        space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        let project_node = ProjectNode::new(input_plan.root().clone(), columns)?;
        Ok(input_plan.with_root(project_node.into_enum()))
    }
}
```

#### 修改方案 4.2: 完善 MatchPlanner

**预估工作量**: 5-7 人天

**修改步骤**:

1. 实现 pattern 解析：
```rust
impl MatchPlanner {
    fn extract_match_pattern(&self, ast: &QueryAstContext, stmt: &Stmt) -> Result<MatchPattern, PlannerError> {
        // 从 AST 中提取 MATCH pattern
        // 支持 (n:Tag)、(n:Tag1:Tag2)、(n)-[e:Edge]->(m) 等模式
        match stmt {
            Stmt::Match(match_stmt) => {
                // 解析 pattern
                Ok(match_stmt.pattern.clone())
            }
            _ => Err(PlannerError::InvalidOperation(
                "Expected MATCH statement".to_string()
            ))
        }
    }
    
    fn plan_pattern(&self, pattern: &MatchPattern, space_id: i32) -> Result<PlanNodeEnum, PlannerError> {
        match pattern {
            MatchPattern::Node { name, labels, properties } => {
                self.plan_node_pattern(name, labels, properties, space_id)
            }
            MatchPattern::Relationship { from, edge, to } => {
                self.plan_relationship_pattern(from, edge, to, space_id)
            }
            MatchPattern::Path { elements } => {
                self.plan_path_pattern(elements, space_id)
            }
        }
    }
    
    fn plan_node_pattern(
        &self,
        name: &str,
        labels: &[String],
        properties: &Option<HashMap<String, Expression>>,
        space_id: i32,
    ) -> Result<PlanNodeEnum, PlannerError> {
        // 根据是否有标签和属性选择合适的扫描策略
        if labels.is_empty() && properties.is_none() {
            // 全表扫描
            Ok(ScanVerticesNode::new(space_id).into_enum())
        } else if !labels.is_empty() {
            // 标签过滤扫描
            let node = GetVerticesNode::new(space_id);
            node.set_tag_filter(labels.clone());
            Ok(node.into_enum())
        } else {
            // 属性过滤扫描
            let node = GetVerticesNode::new(space_id);
            node.set_expression(properties.clone());
            Ok(node.into_enum())
        }
    }
}
```

2. 实现 where 子句处理：
```rust
impl MatchPlanner {
    fn extract_where_condition(&self, ast: &QueryAstContext, stmt: &Stmt) -> Option<Expression> {
        match stmt {
            Stmt::Match(match_stmt) => match_stmt.where_clause.clone(),
            _ => None,
        }
    }
}
```

3. 实现 return 子句处理：
```rust
impl MatchPlanner {
    fn extract_return_columns(&self, ast: &QueryAstContext, stmt: &Stmt) -> Option<Vec<YieldColumn>> {
        match stmt {
            Stmt::Match(match_stmt) => match_stmt.return_clause.clone(),
            _ => None,
        }
    }
}
```

#### 修改方案 4.3: 动态规划器配置

**预估工作量**: 2 人天

**修改代码**:
```rust
pub struct PlannerConfig {
    pub enable_caching: bool,
    pub max_plan_depth: usize,
    pub enable_parallel_planning: bool,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            max_plan_depth: 100,
            enable_parallel_planning: false,
        }
    }
}

pub struct PlannerRegistry {
    planners: HashMap<SentenceKind, Vec<MatchAndInstantiate>>,
    config: PlannerConfig,
}

impl PlannerRegistry {
    pub fn with_config(config: PlannerConfig) -> Self {
        Self {
            planners: HashMap::new(),
            config,
        }
    }
    
    pub fn create_plan(&self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 使用配置进行规划
        if self.config.enable_caching {
            if let Some(cached_plan) = self.get_cached_plan(ast_ctx) {
                return Ok(cached_plan);
            }
        }
        
        // 正常规划逻辑
        let plan = self.create_plan_internal(ast_ctx)?;
        
        // 缓存计划
        if self.config.enable_caching {
            self.cache_plan(ast_ctx, &plan);
        }
        
        Ok(plan)
    }
}
```

---

## 五、Optimizer 模块

**模块路径**: `src/query/optimizer/`

### 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 5.1 | 优化规则硬编码，无法动态配置 | 高 | 扩展性问题 | 待修复 |
| 5.2 | MAX_EXPLORATION_ROUNDS 硬编码为 128 | 中 | 灵活性问题 | 待修复 |
| 5.3 | 优化阶段规则分配使用字符串匹配 | 中 | 代码质量问题 | 待修复 |
| 5.4 | property_pruning 和 rewrite_arguments 为空实现 | 中 | 功能缺失 | 待修复 |
| 5.5 | 成本模型过于简单 | 低 | 功能不完整 | 待修复 |

### 详细问题分析

#### 问题 5.1: 优化规则硬编码

**问题代码**:
```rust
pub fn default() -> Self {
    let mut logical_rules = RuleSet::new("logical");
    logical_rules.add_rule(Box::new(FilterPushDownRule));
    logical_rules.add_rule(Box::new(PredicatePushDownRule));
    // ... 20+ 个规则硬编码
}
```

#### 问题 5.3: 规则分配使用字符串匹配

**问题代码**:
```rust
fn get_rules_for_phase(&self, phase: &OptimizationPhase) -> Vec<&dyn OptRule> {
    for rule_set in &self.rule_sets {
        for rule in &rule_set.rules {
            let rule_name = rule.name();  // 返回字符串
            let matches_phase = match phase {
                OptimizationPhase::LogicalOptimization => {
                    matches!(rule_name,
                        "FilterPushDownRule" | "PredicatePushDownRule" | ...
                    )
                }
                // ...
            };
        }
    }
}
```

### 修改方案

#### 修改方案 5.1-5.2: 配置化优化规则

**预估工作量**: 4-5 人天

**修改步骤**:

1. 修改 OptimizationConfig：
```rust
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    pub logical_rules: Vec<String>,
    pub physical_rules: Vec<String>,
    pub post_rules: Vec<String>,
    pub max_iteration_rounds: usize,
    pub max_exploration_rounds: usize,
    pub enable_cost_based_optimization: bool,
    pub default_row_count: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            logical_rules: vec![
                "FilterPushDownRule".to_string(),
                "PredicatePushDownRule".to_string(),
                "ProjectionPushDownRule".to_string(),
                "CombineFilterRule".to_string(),
                "CollapseProjectRule".to_string(),
                "DedupEliminationRule".to_string(),
                "TopNRule".to_string(),
            ],
            physical_rules: vec![
                "JoinOptimizationRule".to_string(),
                "PushLimitDownRule".to_string(),
                "IndexScanRule".to_string(),
            ],
            post_rules: vec![
                "TopNRule".to_string(),
            ],
            max_iteration_rounds: 10,
            max_exploration_rounds: 128,
            enable_cost_based_optimization: true,
            default_row_count: 1000,
        }
    }
}

impl OptimizationConfig {
    /// 从配置文件加载
    pub fn from_config_file(config_path: &Path) -> Result<Self, OptimizerError> {
        // 解析配置文件
        let config_content = std::fs::read_to_string(config_path)?;
        toml::from_str(&config_content).map_err(|e| {
            OptimizerError::ConfigError(format!("Failed to parse config: {}", e))
        })
    }
    
    /// 环境变量覆盖
    pub fn apply_env_overrides(&mut self) {
        if let Ok(max_rounds) = std::env::var("OPTIMIZER_MAX_ROUNDS") {
            if let Ok(rounds) = max_rounds.parse() {
                self.max_iteration_rounds = rounds;
            }
        }
    }
}
```

2. 修改 Optimizer 实现：
```rust
impl Optimizer {
    pub fn with_config(config: OptimizationConfig) -> Self {
        let mut logical_rules = RuleSet::new("logical");
        for rule_name in &config.logical_rules {
            if let Some(rule) = self.create_rule(rule_name) {
                logical_rules.add_rule(rule);
            }
        }
        
        let mut physical_rules = RuleSet::new("physical");
        for rule_name in &config.physical_rules {
            if let Some(rule) = self.create_rule(rule_name) {
                physical_rules.add_rule(rule);
            }
        }
        
        let mut post_rules = RuleSet::new("post");
        for rule_name in &config.post_rules {
            if let Some(rule) = self.create_rule(rule_name) {
                post_rules.add_rule(rule);
            }
        }
        
        Self {
            rule_sets: vec![logical_rules, physical_rules, post_rules],
            config,
        }
    }
    
    fn create_rule(&self, rule_name: &str) -> Option<Box<dyn OptRule>> {
        match rule_name {
            "FilterPushDownRule" => Some(Box::new(FilterPushDownRule)),
            "PredicatePushDownRule" => Some(Box::new(PredicatePushDownRule)),
            // ... 其他规则
            _ => {
                log::warn!("Unknown optimization rule: {}", rule_name);
                None
            }
        }
    }
}
```

#### 修改方案 5.3: 使用枚举匹配替代字符串

**预估工作量**: 1-2 人天

**修改代码**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationPhase {
    LogicalOptimization,
    PhysicalOptimization,
    PostOptimization,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OptimizationRule {
    FilterPushDown,
    PredicatePushDown,
    ProjectionPushDown,
    CombineFilter,
    CollapseProject,
    DedupElimination,
    TopN,
    JoinOptimization,
    PushLimitDown,
    IndexScan,
    // ... 更多规则
}

impl OptimizationRule {
    pub fn belongs_to(&self, phase: OptimizationPhase) -> bool {
        match (self, phase) {
            (OptimizationRule::FilterPushDown, OptimizationPhase::LogicalOptimization) => true,
            (OptimizationRule::PredicatePushDown, OptimizationPhase::LogicalOptimization) => true,
            (OptimizationRule::JoinOptimization, OptimizationPhase::PhysicalOptimization) => true,
            // ... 完整映射
            _ => false,
        }
    }
}

fn get_rules_for_phase(&self, phase: OptimizationPhase) -> Vec<&dyn OptRule> {
    let mut rules = Vec::new();
    
    for rule_set in &self.rule_sets {
        for rule in &rule_set.rules {
            if let Some(rule_enum) = rule.as_rule_enum() {
                if rule_enum.belongs_to(phase) {
                    rules.push(rule.as_ref());
                }
            }
        }
    }
    
    rules
}
```

#### 修改方案 5.4: 实现属性裁剪和参数重写

**预估工作量**: 3-4 人天

**修改代码**:
```rust
impl Optimizer {
    fn prune_properties(
        &self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        let mut property_tracker = PropertyTracker::new();
        self.collect_required_properties(ctx, root_group, &mut property_tracker)?;
        self.apply_property_pruning(ctx, root_group, &property_tracker)?;
        Ok(())
    }
    
    fn apply_property_pruning(
        &self,
        ctx: &mut OptContext,
        group: &mut OptGroup,
        property_tracker: &PropertyTracker,
    ) -> Result<(), OptimizerError> {
        for node in &mut group.nodes {
            self.prune_node_properties(node, property_tracker)?;
            
            // 递归处理依赖节点
            for &dep_id in &node.dependencies {
                if let Some(dep_group) = ctx.group_map.get_mut(&dep_id) {
                    self.apply_property_pruning(ctx, dep_group, property_tracker)?;
                }
            }
        }
        Ok(())
    }
    
    fn prune_node_properties(
        &self,
        node: &mut OptGroupNode,
        property_tracker: &PropertyTracker,
    ) -> Result<(), OptimizerError> {
        match node.plan_node.name() {
            "Project" => {
                if let Some(project_node) = node.plan_node.as_project_mut() {
                    let required_props = property_tracker.get_required_properties_for_node(node.id);
                    project_node.prune_columns(&required_props);
                }
            }
            "GetNeighbors" => {
                if let Some(get_nbrs_node) = node.plan_node.as_get_neighbors_mut() {
                    let required_props = property_tracker.get_required_properties_for_node(node.id);
                    get_nbrs_node.prune_properties(&required_props);
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn rewrite_arguments(
        &self,
        ctx: &mut OptContext,
        group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        // 构建参数映射
        let arg_map = self.build_argument_mapping(ctx, group)?;
        
        // 重写节点参数
        for node in &mut group.nodes {
            self.rewrite_node_arguments(node, &arg_map)?;
            
            // 递归处理依赖节点
            for &dep_id in &node.dependencies {
                if let Some(dep_group) = ctx.group_map.get_mut(&dep_id) {
                    self.rewrite_arguments(ctx, dep_group)?;
                }
            }
        }
        Ok(())
    }
    
    fn build_argument_mapping(
        &self,
        ctx: &OptContext,
        group: &OptGroup,
    ) -> Result<HashMap<usize, Vec<String>>, OptimizerError> {
        let mut mapping = HashMap::new();
        
        for node in &group.nodes {
            if let Some(outputs) = self.get_node_outputs(node) {
                mapping.insert(node.id, outputs);
            }
        }
        
        Ok(mapping)
    }
    
    fn rewrite_node_arguments(
        &self,
        node: &mut OptGroupNode,
        arg_map: &HashMap<usize, Vec<String>>,
    ) -> Result<(), OptimizerError> {
        match node.plan_node.name() {
            "Project" => {
                if let Some(project_node) = node.plan_node.as_project_mut() {
                    for column in project_node.columns_mut() {
                        if let Some(input_var) = arg_map.get(&node.id).and_then(|vars| vars.first()) {
                            column.expression = self.rewrite_expression_args(&column.expression, input_var);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

#### 修改方案 5.5: 增强成本模型

**预估工作量**: 5-7 人天

**修改代码**:
```rust
#[derive(Debug, Clone)]
pub struct Cost {
    pub cpu_cost: f64,
    pub memory_cost: f64,
    pub io_cost: f64,
    pub network_cost: f64,
    pub row_count: usize,
}

impl Cost {
    pub fn new() -> Self {
        Self {
            cpu_cost: 0.0,
            memory_cost: 0.0,
            io_cost: 0.0,
            network_cost: 0.0,
            row_count: 0,
        }
    }
    
    pub fn total(&self) -> f64 {
        // 加权总成本
        self.cpu_cost * 1.0 + self.memory_cost * 0.5 + self.io_cost * 2.0 + self.network_cost * 3.0
    }
    
    pub fn add(&mut self, other: &Cost) {
        self.cpu_cost += other.cpu_cost;
        self.memory_cost += other.memory_cost;
        self.io_cost += other.io_cost;
        self.network_cost += other.network_cost;
        self.row_count = self.row_count.max(other.row_count);
    }
}

pub trait CostEstimator {
    fn estimate_cost(&self, node: &PlanNodeEnum, input_cardinality: usize) -> Cost;
}

pub struct DefaultCostEstimator {
    config: OptimizationConfig,
}

impl DefaultCostEstimator {
    pub fn new(config: OptimizationConfig) -> Self {
        Self { config }
    }
}

impl CostEstimator for DefaultCostEstimator {
    fn estimate_cost(&self, node: &PlanNodeEnum, input_cardinality: usize) -> Cost {
        let mut cost = Cost::new();
        
        match node {
            PlanNodeEnum::ScanVertices(_) => {
                cost.io_cost = self.config.default_row_count as f64;
                cost.row_count = self.config.default_row_count;
            }
            PlanNodeEnum::GetNeighbors(n) => {
                cost.io_cost = input_cardinality as f64 * n.step_limit().unwrap_or(10) as f64;
                cost.row_count = input_cardinality * 10;
            }
            PlanNodeEnum::Filter(_) => {
                cost.cpu_cost = input_cardinality as f64 * 0.1;
                cost.row_count = input_cardinality / 2; // 假设过滤掉一半
            }
            PlanNodeEnum::Project(_) => {
                cost.cpu_cost = input_cardinality as f64 * 0.05;
                cost.row_count = input_cardinality;
            }
            PlanNodeEnum::Aggregate(_) => {
                cost.cpu_cost = input_cardinality as f64 * 0.5;
                cost.row_count = input_cardinality / 10; // 假设聚合后减少
            }
            PlanNodeEnum::Join(n) => {
                let join_type = n.join_type();
                cost.cpu_cost = input_cardinality as f64 * input_cardinality as f64 * 0.01;
                cost.row_count = input_cardinality; // 简化估算
            }
            _ => {
                cost.cpu_cost = input_cardinality as f64 * 0.01;
                cost.row_count = input_cardinality;
            }
        }
        
        cost
    }
}
```

---

## 六、Executor 模块

**模块路径**: `src/query/executor/`

### 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 6.1 | GraphQueryExecutor 中大量语句执行未实现 | 高 | 功能缺失 | 待修复 |
| 6.2 | ExecutorFactory 的 create_executor 方法过长 | 中 | 代码质量问题 | 待修复 |
| 6.3 | 缺乏执行器注册机制 | 中 | 扩展性问题 | 待修复 |
| 6.4 | 错误处理不统一 | 低 | 一致性问题 | 待修复 |
| 6.5 | 缺乏查询超时机制 | 低 | 功能缺失 | 待修复 |

### 详细问题分析

#### 问题 6.1: 大量语句执行未实现

**问题代码**:
```rust
async fn execute_create(&mut self, _clause: CreateStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "CREATE语句执行未实现".to_string()
    )))
}

async fn execute_delete(&mut self, _clause: DeleteStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "DELETE语句执行未实现".to_string()
    )))
}
// ... 更多未实现的语句
```

#### 问题 6.2: create_executor 方法过长

**当前实现**: `create_executor` 方法包含 300+ 行代码，使用大型 match 语句处理所有节点类型。

### 修改方案

#### 修改方案 6.1: 实现缺失的执行器

**预估工作量**: 15-20 人天（按功能模块）

**修改策略**:

1. 按优先级实现功能：
```
优先级 1（高频使用）:
- MATCH 执行器
- CREATE (INSERT) 执行器
- DELETE 执行器
- UPDATE 执行器

优先级 2（中频使用）:
- MERGE 执行器
- SET 执行器
- REMOVE 执行器

优先级 3（低频使用）:
- 批量操作执行器
- 事务控制执行器
```

2. 实现 CREATE 执行器示例：
```rust
pub struct InsertVertexExecutor {
    id: i64,
    storage: Arc<Mutex<S>>,
    space_name: String,
    tag_name: String,
    properties: Vec<(String, Value)>,
}

impl<S: StorageEngine> InsertVertexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        space_name: String,
        tag_name: String,
        properties: Vec<(String, Value)>,
    ) -> Self {
        Self {
            id,
            storage,
            space_name,
            tag_name,
            properties,
        }
    }
}

#[async_trait]
impl<S: StorageEngine> Executor<S> for InsertVertexExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let mut storage = self.storage.lock().map_err(|e| {
            DBError::Execution(e.to_string())
        })?;
        
        // 生成顶点 ID
        let vertex_id = storage.generate_vertex_id(&self.space_name)?;
        
        // 创建顶点
        let vertex = Vertex::new(
            vertex_id,
            self.tag_name.clone(),
            self.properties.clone(),
        );
        
        // 插入顶点
        storage.insert_vertex(&self.space_name, vertex)?;
        
        // 返回结果
        Ok(ExecutionResult::success_with_count(1))
    }
}
```

#### 修改方案 6.2: 重构 ExecutorFactory

**预估工作量**: 3-4 人天

**修改代码**:
```rust
pub struct ExecutorFactory<S: StorageEngine + 'static> {
    storage: Option<Arc<Mutex<S>>>,
    recursion_detector: RecursionDetector,
    safety_validator: ExecutorSafetyValidator,
    // 使用注册表模式
    executors: HashMap<&'static str, Box<dyn ExecutorCreator<S>>>,
}

pub trait ExecutorCreator<S: StorageEngine>: Send {
    fn create(&self, node: &PlanNodeEnum, storage: Arc<Mutex<S>>) -> Result<Box<dyn Executor<S>>, QueryError>;
    fn can_handle(&self, node: &PlanNodeEnum) -> bool;
}

struct StartExecutorCreator;
struct ScanVerticesExecutorCreator;
struct GetVerticesExecutorCreator;
// ... 更多执行器创建器

impl<S: StorageEngine + 'static> ExecutorFactory<S> {
    pub fn new() -> Self {
        let mut factory = Self {
            storage: None,
            recursion_detector: RecursionDetector::new(100),
            safety_validator: ExecutorSafetyValidator::new(ExecutorSafetyConfig::default()),
            executors: HashMap::new(),
        };
        
        // 注册执行器
        factory.register_executor("Start", Box::new(StartExecutorCreator));
        factory.register_executor("ScanVertices", Box::new(ScanVerticesExecutorCreator));
        factory.register_executor("GetVertices", Box::new(GetVerticesExecutorCreator));
        // ... 注册更多执行器
        
        factory
    }
    
    pub fn register_executor(
        &mut self,
        node_type: &'static str,
        creator: Box<dyn ExecutorCreator<S>>,
    ) {
        self.executors.insert(node_type, creator);
    }
    
    pub fn create_executor(
        &self,
        plan_node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        let node_type = plan_node.name();
        
        if let Some(creator) = self.executors.get(node_type) {
            creator.create(plan_node, storage)
        } else {
            Err(QueryError::ExecutionError(format!(
                "No executor registered for node type: {}", node_type
            )))
        }
    }
}

impl<S: StorageEngine> ExecutorCreator<S> for GetVerticesExecutorCreator {
    fn create(&self, node: &PlanNodeEnum, storage: Arc<Mutex<S>>) -> Result<Box<dyn Executor<S>>, QueryError> {
        if let PlanNodeEnum::GetVertices(n) = node {
            let executor = GetVerticesExecutor::new(
                n.id(),
                storage,
                Some(vec![crate::core::Value::String(n.src_vids().to_string())]),
                None,
                n.expression().and_then(|e| parse_expression_safe(e)),
                n.limit().map(|l| l as usize),
            );
            Ok(Box::new(executor))
        } else {
            Err(QueryError::ExecutionError(
                "GetVerticesExecutorCreator can only create for GetVertices node".to_string()
            ))
        }
    }
    
    fn can_handle(&self, node: &PlanNodeEnum) -> bool {
        matches!(node, PlanNodeEnum::GetVertices(_))
    }
}
```

#### 修改方案 6.3: 添加执行器注册机制

**预估工作量**: 2 人天

**修改代码**:
```rust
pub struct ExecutorRegistry<S: StorageEngine + 'static> {
    creators: HashMap<&'static str, Box<dyn ExecutorCreator<S>>>,
    default_creator: Option<Box<dyn ExecutorCreator<S>>>,
}

impl<S: StorageEngine + 'static> ExecutorRegistry<S> {
    pub fn new() -> Self {
        Self {
            creators: HashMap::new(),
            default_creator: None,
        }
    }
    
    pub fn register<F>(&mut self, node_type: &'static str, creator: F)
    where
        F: ExecutorCreator<S> + 'static,
    {
        self.creators.insert(node_type, Box::new(creator));
    }
    
    pub fn set_default<F>(&mut self, creator: F)
    where
        F: ExecutorCreator<S> + 'static,
    {
        self.default_creator = Some(Box::new(creator));
    }
    
    pub fn create(&self, node: &PlanNodeEnum, storage: Arc<Mutex<S>>) -> Result<Box<dyn Executor<S>>, QueryError> {
        let node_type = node.name();
        
        if let Some(creator) = self.creators.get(node_type) {
            creator.create(node, storage)
        } else if let Some(default_creator) = &self.default_creator {
            default_creator.create(node, storage)
        } else {
            Err(QueryError::ExecutionError(format!(
                "No executor found for node type: {}", node_type
            )))
        }
    }
}

// 使用示例
let mut registry = ExecutorRegistry::<StorageEngine>::new();
registry.register("Start", StartExecutorCreator);
registry.register("Filter", FilterExecutorCreator);
// ...
```

#### 修改方案 6.5: 添加查询超时机制

**预估工作量**: 2 人天

**修改代码**:
```rust
use std::time::{Duration, Instant};
use tokio::time::timeout;

pub struct ExecutionConfig {
    pub timeout: Duration,
    pub max_rows: usize,
    pub enable_profiling: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_rows: 1000000,
            enable_profiling: false,
        }
    }
}

pub struct ExecutionContext {
    pub config: ExecutionConfig,
    pub start_time: Instant,
    pub row_count: usize,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            config: ExecutionConfig::default(),
            start_time: Instant::now(),
            row_count: 0,
        }
    }
    
    pub fn check_timeout(&self) -> Result<(), QueryError> {
        if self.start_time.elapsed() > self.config.timeout {
            Err(QueryError::ExecutionError("Query timeout".to_string()))
        } else {
            Ok(())
        }
    }
    
    pub fn check_row_limit(&self) -> Result<(), QueryError> {
        if self.row_count > self.config.max_rows {
            Err(QueryError::ExecutionError(
                format!("Row limit exceeded: {}", self.config.max_rows)
            ))
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl<S: StorageEngine> Executor<S> for FilterExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = self.input.execute().await?;
        
        let mut output_rows = Vec::new();
        
        for row in input_result.rows() {
            // 检查超时和行数限制
            self.context.check_timeout()?;
            self.context.check_row_limit()?;
            
            if self.evaluate_condition(&row)? {
                output_rows.push(row.clone());
                self.context.row_count += 1;
            }
        }
        
        Ok(ExecutionResult::new(output_rows, input_result.columns()))
    }
}
```

---

## 七、Context 模块

**模块路径**: `src/query/context/`

### 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 7.1 | 存在多个上下文对象，职责重叠 | 高 | 架构问题 | 待修复 |
| 7.2 | AstContext 与 QueryAstContext 职责不清 | 高 | 设计缺陷 | 待修复 |
| 7.3 | ValidationContext 独立于 QueryAstContext | 高 | 数据不一致 | 待修复 |
| 7.4 | 上下文字段未使用 | 中 | 代码冗余 | 待修复 |
| 7.5 | 缺乏统一的上下文接口 | 低 | 设计问题 | 待修复 |

### 修改方案

#### 修改方案 7.1-7.3: 统一上下文层次

**预估工作量**: 6-8 人天

**修改步骤**:

1. 定义统一的查询上下文接口：
```rust
pub trait QueryContextTrait {
    fn query_text(&self) -> &str;
    fn statement(&self) -> Option<&Stmt>;
    fn space(&self) -> Option<&SpaceInfo>;
    fn outputs(&self) -> &[ColumnDef];
    fn inputs(&self) -> &[ColumnDef];
    fn validation_errors(&self) -> &[ValidationError];
}
```

2. 合并 QueryAstContext 和 AstContext：
```rust
#[derive(Debug, Clone)]
pub struct UnifiedQueryContext {
    query_text: String,
    statement: Option<Stmt>,
    space: SpaceInfo,
    query_type: QueryType,
    
    // 变量和表达式信息
    variables: HashMap<String, VariableInfo>,
    expression_contexts: Vec<ExpressionContext>,
    
    // 验证结果
    outputs: Vec<ColumnDef>,
    inputs: Vec<ColumnDef>,
    validation_errors: Vec<ValidationError>,
    
    // 依赖关系
    dependencies: HashMap<String, Vec<String>>,
}

impl UnifiedQueryContext {
    pub fn new(query_text: &str) -> Self {
        Self {
            query_text: query_text.to_string(),
            statement: None,
            space: SpaceInfo::default(),
            query_type: QueryType::Unknown,
            variables: HashMap::new(),
            expression_contexts: Vec::new(),
            outputs: Vec::new(),
            inputs: Vec::new(),
            validation_errors: Vec::new(),
            dependencies: HashMap::new(),
        }
    }
    
    pub fn set_statement(&mut self, stmt: Stmt) {
        self.statement = Some(stmt);
        self.query_type = self.deduce_query_type();
    }
    
    pub fn add_variable(&mut self, name: String, info: VariableInfo) {
        self.variables.insert(name, info);
    }
    
    pub fn add_validation_error(&mut self, error: ValidationError) {
        self.validation_errors.push(error);
    }
    
    pub fn has_validation_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }
    
    fn deduce_query_type(&self) -> QueryType {
        match &self.statement {
            Some(Stmt::Match(_)) => QueryType::ReadQuery,
            Some(Stmt::Go(_)) => QueryType::ReadQuery,
            Some(Stmt::Create(_)) => QueryType::WriteQuery,
            Some(Stmt::Delete(_)) => QueryType::WriteQuery,
            Some(Stmt::Update(_)) => QueryType::WriteQuery,
            _ => QueryType::Unknown,
        }
    }
}
```

3. 移除独立的 ValidationContext，将验证信息直接存储到 UnifiedQueryContext：
```rust
// 移除 ValidationContext 或将其降级为内部使用
struct InternalValidationContext {
    // 内部验证状态
    current_space: Option<String>,
    input_columns: Vec<ColumnDef>,
    output_columns: Vec<ColumnDef>,
}

impl Validator {
    pub fn validate_with_context(
        &mut self,
        query_context: &mut UnifiedQueryContext,
    ) -> Result<(), ValidationError> {
        // 直接操作 query_context
        if query_context.space().is_none() {
            return Err(ValidationError::SemanticError {
                message: "No space selected".to_string(),
                location: None,
            });
        }
        
        // 执行验证
        self.validate_impl(query_context)?;
        
        // 验证结果已写入 query_context
        if query_context.has_validation_errors() {
            let errors = query_context.validation_errors().to_vec();
            return Err(ValidationError::CompoundError(errors));
        }
        
        Ok(())
    }
}
```

---

## 八、Scheduler 模块

**模块路径**: `src/query/scheduler/`

### 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 8.1 | 调度器与执行器耦合紧密 | 中 | 架构问题 | 待修复 |
| 8.2 | 缺乏查询队列管理 | 中 | 功能缺失 | 待修复 |
| 8.3 | 不支持查询优先级 | 低 | 功能缺失 | 待修复 |
| 8.4 | 资源监控功能不足 | 低 | 功能缺失 | 待修复 |

### 修改方案

#### 修改方案 8.1: 解耦调度器与执行器

**预估工作量**: 3-4 人天

**修改代码**:
```rust
pub trait QueryScheduler: Send {
    async fn submit(&self, query: QueryTask) -> Result<QueryHandle, SchedulerError>;
    async fn cancel(&self, query_id: &str) -> Result<(), SchedulerError>;
    async fn get_status(&self, query_id: &str) -> Result<QueryStatus, SchedulerError>;
    fn get_queue_length(&self) -> usize;
}

pub struct Scheduler<S: StorageEngine + 'static> {
    queue: Arc<Mutex<QueryQueue>>,
    executor: Arc<ThreadPool>,
    storage: Arc<Mutex<S>>,
    config: SchedulerConfig,
}

struct QueryQueue {
    high_priority: Vec<QueryTask>,
    normal_priority: Vec<QueryTask>,
    low_priority: Vec<QueryTask>,
}

impl<S: StorageEngine + 'static> Scheduler<S> {
    pub async fn submit(&self, task: QueryTask) -> Result<QueryHandle, SchedulerError> {
        let query_id = task.id.clone();
        let handle = QueryHandle::new(query_id.clone());
        
        {
            let mut queue = self.queue.lock().unwrap();
            match task.priority {
                QueryPriority::High => queue.high_priority.push(task),
                QueryPriority::Normal => queue.normal_priority.push(task),
                QueryPriority::Low => queue.low_priority.push(task),
            }
        }
        
        // 异步执行
        let executor = self.executor.clone();
        let storage = self.storage.clone();
        
        tokio::spawn(async move {
            let result = Self::execute_task(executor, storage, task).await;
            handle.set_result(result);
        });
        
        Ok(handle)
    }
    
    async fn execute_task(
        executor: Arc<ThreadPool>,
        storage: Arc<Mutex<S>>,
        task: QueryTask,
    ) -> ExecutionResult {
        // 执行查询任务
        let mut pipeline = QueryPipelineManager::new(storage);
        pipeline.execute_query(&task.query_text).await
    }
}
```

---

## 九、Visitor 模块

**模块路径**: `src/query/visitor/`

### 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 9.1 | 与 Expression Evaluator 功能重叠 | 高 | 架构问题 | 待修复 |
| 9.2 | FoldConstantExprVisitor 实现不完整 | 中 | 功能缺失 | 待修复 |
| 9.3 | 访问者之间缺乏代码共享 | 中 | 代码重复 | 待修复 |
| 9.4 | 错误处理不统一 | 低 | 一致性问题 | 待修复 |

### 修改方案

#### 修改方案 9.1: 明确 Visitor 和 Evaluator 的职责边界

**预估工作量**: 2-3 人天

**修改步骤**:

1. 明确职责划分：
```rust
// Visitor: 用于静态分析，不修改 AST，返回分析结果
pub trait QueryVisitor {
    type Result;
    fn get_result(&self) -> Self::Result;
    fn reset(&mut self);
    fn is_success(&self) -> bool;
}

// Evaluator: 用于运行时求值，需要上下文
#[async_trait]
pub trait ExpressionEvaluator {
    type Output;
    type Error;
    async fn evaluate(&self, context: &dyn ExpressionContext) -> Result<Self::Output, Self::Error>;
}
```

2. 移除重复功能：
```rust
// 移除 Evaluator 中的 can_evaluate 方法，统一使用 Visitor
pub struct ExpressionEvaluator {
    // 只保留求值逻辑
}

impl ExpressionEvaluator {
    pub async fn evaluate(&self, expr: &Expression, ctx: &mut dyn ExpressionContext) -> Result<Value, EvalError> {
        match expr {
            Expression::Literal(v) => Ok(v.clone()),
            Expression::Variable(name) => self.evaluate_variable(name, ctx),
            Expression::Property { object, property } => self.evaluate_property(object, property, ctx),
            // ... 其他表达式类型
        }
    }
}

// 可求值性检查统一使用 EvaluableExprVisitor
pub fn can_evaluate_statically(expr: &Expression) -> bool {
    let mut visitor = EvaluableExprVisitor::new();
    visitor.visit_expression(expr);
    visitor.is_evaluable()
}
```

#### 修改方案 9.2: 完善常量折叠

**预估工作量**: 2-3 人天

**修改代码**:
```rust
impl FoldConstantExprVisitor {
    pub fn fold(&mut self, expr: &Expression) -> Expression {
        self.reset();
        self.visit_expression(expr);
        self.result.take().unwrap_or_else(|| expr.clone())
    }
    
    fn fold_binary_expr(&mut self, left: &Expression, op: &BinaryOperator, right: &Expression) -> Expression {
        // 递归折叠操作数
        let folded_left = self.visit_expression(left);
        let folded_right = self.visit_expression(right);
        
        // 如果两个操作数都是常量，进行计算
        if let (Expression::Literal(lit_l), Expression::Literal(lit_r)) = (&folded_left, &folded_right) {
            self.evaluate_binary(lit_l, op, lit_r)
                .map(Expression::Literal)
                .unwrap_or_else(|| {
                    Expression::Binary {
                        left: Box::new(folded_left),
                        op: op.clone(),
                        right: Box::new(folded_right),
                    }
                })
        } else {
            Expression::Binary {
                left: Box::new(folded_left),
                op: op.clone(),
                right: Box::new(folded_right),
            }
        }
    }
    
    fn evaluate_binary(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Option<Value> {
        match op {
            BinaryOperator::Add => Some(left.add(right)),
            BinaryOperator::Subtract => Some(left.subtract(right)),
            BinaryOperator::Multiply => Some(left.multiply(right)),
            BinaryOperator::Divide => right.as_float().and_then(|r| {
                if r == 0.0 { None } else { Some(left.divide(right)) }
            }),
            // ... 更多操作
        }
    }
}
```

---

## 十、修改优先级汇总

### 高优先级（立即处理）

| 模块 | 问题 | 预估工作量 | 修改方案编号 |
|------|------|------------|--------------|
| QueryPipelineManager | 未使用参数 | 2-3 人天 | 1.1-1.2 |
| Validator | 上下文分离 | 4-6 人天 | 3.1-3.3 |
| Planner | 使用子集上下文 | 3-4 人天 | 4.1 |
| Executor | 未实现语句 | 15-20 人天 | 6.1 |
| Context | 上下文冗余 | 6-8 人天 | 7.1-7.3 |
| Visitor | 功能重叠 | 2-3 人天 | 9.1 |

### 中优先级（短期内处理）

| 模块 | 问题 | 预估工作量 | 修改方案编号 |
|------|------|------------|--------------|
| Optimizer | 规则硬编码 | 4-5 人天 | 5.1-5.2 |
| Executor | Factory 重构 | 3-4 人天 | 6.2 |
| Parser | 错误信息 | 1 人天 | 2.1 |

### 低优先级（长期优化）

| 模块 | 问题 | 预估工作量 | 修改方案编号 |
|------|------|------------|--------------|
| Optimizer | 成本模型 | 5-7 人天 | 5.5 |
| Scheduler | 解耦 | 3-4 人天 | 8.1 |
| Optimizer | 属性裁剪 | 3-4 人天 | 5.4 |

---

## 十一、实施建议

### 实施顺序

1. **第一阶段（1-2 周）**: 修复高优先级的架构问题
   - 统一上下文模块
   - 修复数据传递问题

2. **第二阶段（2-4 周）**: 完善核心功能
   - 实现缺失的执行器
   - 配置化优化规则

3. **第三阶段（4-6 周）**: 优化和重构
   - 重构 ExecutorFactory
   - 完善成本模型
   - 添加监控功能

### 风险控制

1. **回滚计划**: 每个修改都应能够独立回滚
2. **测试覆盖**: 修改前后必须有完整的测试覆盖
3. **渐进式发布**: 先在开发环境测试，再逐步发布

### 验收标准

1. 所有高优先级问题已修复
2. 测试覆盖率不低于 80%
3. 文档已更新
4. 代码审查通过
