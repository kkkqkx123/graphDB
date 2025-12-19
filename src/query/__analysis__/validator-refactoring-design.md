# 第二阶段：验证器重构设计方案

## 目标概述

重构验证器，实现验证+规划的一体化设计，参考nebula-graph的Validator模式，将语义验证和初始规划紧密结合。

## 当前验证器状态分析

### 现有验证器结构
```
src/query/validator/
├── base_validator.rs
├── match_validator.rs
├── validate_context.rs
├── validation_factory.rs
├── validation_interface.rs
├── strategies/
└── structs/
```

### 存在的问题
1. **职责分离过度**: 验证和规划完全分离
2. **接口复杂**: 过多的策略和接口定义
3. **上下文分散**: 验证上下文与执行上下文分离
4. **重复代码**: 多个验证器有重复的逻辑

## 新的验证器架构设计

### 1. 核心接口设计

#### Validator trait
```rust
use async_trait::async_trait;
use crate::core::error::{DBError, DBResult};
use crate::query::context::{QueryContext, AstContext, ExecutionContext};
use crate::query::parser::ast::Sentence;
use crate::query::engine::plan::ExecutionPlan;

/// 验证器核心trait
#[async_trait]
pub trait Validator: Send + Sync {
    /// 验证语句的语义正确性
    async fn validate(&mut self) -> DBResult<()>;
    
    /// 将验证后的AST转换为执行计划
    async fn to_plan(&mut self) -> DBResult<ExecutionPlan>;
    
    /// 获取AST上下文
    fn ast_context(&self) -> &AstContext;
    
    /// 获取验证器名称
    fn name(&self) -> &'static str;
    
    /// 获取输入变量名
    fn input_var_name(&self) -> Option<&str>;
    
    /// 设置输入变量名
    fn set_input_var_name(&mut self, name: String);
    
    /// 获取输出列定义
    fn output_columns(&self) -> &[ColumnDefinition];
    
    /// 获取输入列定义
    fn input_columns(&self) -> &[ColumnDefinition];
}

/// 验证器错误类型
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("语法错误: {0}")]
    SyntaxError(String),
    
    #[error("语义错误: {0}")]
    SemanticError(String),
    
    #[error("类型错误: {0}")]
    TypeError(String),
    
    #[error("模式错误: {0}")]
    PatternError(String),
    
    #[error("权限错误: {0}")]
    PermissionError(String),
    
    #[error("上下文错误: {0}")]
    ContextError(String),
    
    #[error("规划错误: {0}")]
    PlanError(String),
    
    #[error("存储错误: {0}")]
    StorageError(#[from] DBError),
}
```

#### 验证器工厂
```rust
use std::collections::HashMap;
use std::sync::Arc;

/// 验证器工厂
pub struct ValidatorFactory {
    creators: HashMap<SentenceType, Box<dyn ValidatorCreator>>,
}

impl ValidatorFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            creators: HashMap::new(),
        };
        
        // 注册Cypher验证器
        factory.register_cypher_validators();
        
        factory
    }
    
    /// 创建验证器
    pub fn create_validator(
        &self,
        sentence: &Sentence,
        qctx: Arc<QueryContext>,
    ) -> DBResult<Box<dyn Validator>> {
        let sentence_type = sentence.sentence_type();
        
        match self.creators.get(&sentence_type) {
            Some(creator) => creator.create(sentence, qctx),
            None => Err(DBError::Validation(
                crate::core::error::ValidationError::UnsupportedStatement(
                    format!("Unsupported sentence type: {:?}", sentence_type)
                )
            )),
        }
    }
    
    /// 注册验证器创建器
    pub fn register_creator(&mut self, sentence_type: SentenceType, creator: Box<dyn ValidatorCreator>) {
        self.creators.insert(sentence_type, creator);
    }
}

/// 验证器创建器trait
pub trait ValidatorCreator: Send + Sync {
    fn create(&self, sentence: &Sentence, qctx: Arc<QueryContext>) -> DBResult<Box<dyn Validator>>;
}
```

### 2. 基础验证器实现

#### BaseValidator
```rust
/// 基础验证器实现
pub struct BaseValidator {
    sentence: Sentence,
    qctx: Arc<QueryContext>,
    ast_ctx: AstContext,
    input_var_name: Option<String>,
    output_columns: Vec<ColumnDefinition>,
    input_columns: Vec<ColumnDefinition>,
}

impl BaseValidator {
    pub fn new(sentence: Sentence, qctx: Arc<QueryContext>) -> Self {
        let ast_ctx = AstContext::new(sentence.clone(), qctx.clone());
        
        Self {
            sentence,
            qctx,
            ast_ctx,
            input_var_name: None,
            output_columns: Vec::new(),
            input_columns: Vec::new(),
        }
    }
    
    /// 检查空间是否已选择
    pub fn check_space_chosen(&self) -> DBResult<()> {
        if self.qctx.space_id().is_none() {
            return Err(DBError::Validation(
                crate::core::error::ValidationError::ContextError(
                    "No space selected".to_string()
                )
            ));
        }
        Ok(())
    }
    
    /// 检查权限
    pub async fn check_permission(&self) -> DBResult<()> {
        // 实现权限检查逻辑
        // 这里可以调用权限管理器
        Ok(())
    }
    
    /// 推断表达式类型
    pub fn deduce_expression_type(&self, expr: &Expression) -> DBResult<ValueType> {
        // 实现表达式类型推断
        Ok(ValueType::Unknown)
    }
    
    /// 收集表达式属性
    pub fn collect_expression_properties(&self, expr: &Expression) -> DBResult<ExpressionProperties> {
        // 实现表达式属性收集
        Ok(ExpressionProperties::new())
    }
    
    /// 验证列名不重复
    pub fn validate_no_duplicate_columns(&self) -> DBResult<()> {
        let mut column_names = std::collections::HashSet::new();
        
        for col in &self.output_columns {
            if column_names.contains(&col.name) {
                return Err(DBError::Validation(
                    crate::core::error::ValidationError::SemanticError(
                        format!("Duplicate column name: {}", col.name)
                    )
                ));
            }
            column_names.insert(col.name.clone());
        }
        
        Ok(())
    }
    
    /// 设置输出列
    pub fn set_output_columns(&mut self, columns: Vec<ColumnDefinition>) {
        self.output_columns = columns;
    }
    
    /// 设置输入列
    pub fn set_input_columns(&mut self, columns: Vec<ColumnDefinition>) {
        self.input_columns = columns;
    }
}

impl Validator for BaseValidator {
    async fn validate(&mut self) -> DBResult<()> {
        // 基础验证流程
        self.check_space_chosen()?;
        self.check_permission().await?;
        self.validate_no_duplicate_columns()?;
        
        // 调用具体验证逻辑
        self.validate_impl().await
    }
    
    async fn to_plan(&mut self) -> DBResult<ExecutionPlan> {
        // 调用具体规划逻辑
        self.to_plan_impl().await
    }
    
    fn ast_context(&self) -> &AstContext {
        &self.ast_ctx
    }
    
    fn name(&self) -> &'static str {
        "BaseValidator"
    }
    
    fn input_var_name(&self) -> Option<&str> {
        self.input_var_name.as_deref()
    }
    
    fn set_input_var_name(&mut self, name: String) {
        self.input_var_name = Some(name);
    }
    
    fn output_columns(&self) -> &[ColumnDefinition] {
        &self.output_columns
    }
    
    fn input_columns(&self) -> &[ColumnDefinition] {
        &self.input_columns
    }
}

/// BaseValidator的扩展trait，供具体验证器实现
pub trait ValidatorExt {
    /// 具体验证逻辑
    async fn validate_impl(&mut self) -> DBResult<()>;
    
    /// 具体规划逻辑
    async fn to_plan_impl(&mut self) -> DBResult<ExecutionPlan>;
}
```

### 3. Cypher验证器实现

#### MatchValidator
```rust
use crate::query::parser::cypher::ast::{MatchClause, WhereClause, ReturnClause};
use crate::query::engine::plan::nodes::*;

/// MATCH语句验证器
pub struct MatchValidator {
    base: BaseValidator,
    match_clause: MatchClause,
}

impl MatchValidator {
    pub fn new(sentence: Sentence, qctx: Arc<QueryContext>) -> DBResult<Self> {
        let match_clause = extract_match_clause(&sentence)?;
        
        Ok(Self {
            base: BaseValidator::new(sentence, qctx),
            match_clause,
        })
    }
    
    /// 验证模式
    async fn validate_pattern(&mut self) -> DBResult<()> {
        for pattern_part in &self.match_clause.patterns {
            self.validate_pattern_part(pattern_part).await?;
        }
        Ok(())
    }
    
    /// 验证模式部分
    async fn validate_pattern_part(&mut self, pattern_part: &PatternPart) -> DBResult<()> {
        // 验证节点模式
        self.validate_node_pattern(&pattern_part.node).await?;
        
        // 验证关系模式
        for rel in &pattern_part.relationships {
            self.validate_relationship_pattern(rel).await?;
        }
        
        Ok(())
    }
    
    /// 验证节点模式
    async fn validate_node_pattern(&mut self, node: &NodePattern) -> DBResult<()> {
        // 验证标签是否存在
        if let Some(labels) = &node.labels {
            for label in labels {
                if !self.base.qctx.schema_manager().tag_exists(label)? {
                    return Err(DBError::Validation(
                        crate::core::error::ValidationError::SemanticError(
                            format!("Tag not found: {}", label)
                        )
                    ));
                }
            }
        }
        
        // 验证属性
        if let Some(properties) = &node.properties {
            self.validate_properties(properties).await?;
        }
        
        Ok(())
    }
    
    /// 验证关系模式
    async fn validate_relationship_pattern(&mut self, rel: &RelationshipPattern) -> DBResult<()> {
        // 验证边类型是否存在
        for edge_type in &rel.types {
            if !self.base.qctx.schema_manager().edge_type_exists(edge_type)? {
                return Err(DBError::Validation(
                    crate::core::error::ValidationError::SemanticError(
                        format!("Edge type not found: {}", edge_type)
                    )
                ));
            }
        }
        
        // 验证属性
        if let Some(properties) = &rel.properties {
            self.validate_properties(properties).await?;
        }
        
        Ok(())
    }
    
    /// 验证WHERE子句
    async fn validate_where_clause(&mut self) -> DBResult<()> {
        if let Some(where_clause) = &self.match_clause.where_clause {
            self.validate_expression(&where_clause.expression).await?;
        }
        Ok(())
    }
    
    /// 验证RETURN子句
    async fn validate_return_clause(&mut self) -> DBResult<()> {
        // 这里需要从sentence中提取RETURN子句
        // 暂时跳过具体实现
        Ok(())
    }
    
    /// 验证表达式
    async fn validate_expression(&mut self, expr: &Expression) -> DBResult<()> {
        match expr {
            Expression::Property(prop) => {
                self.validate_property_expression(prop).await?;
            }
            Expression::FunctionCall(func) => {
                self.validate_function_call(func).await?;
            }
            Expression::Binary(bin) => {
                self.validate_binary_expression(bin).await?;
            }
            _ => {} // 其他表达式类型的验证
        }
        Ok(())
    }
    
    /// 验证属性表达式
    async fn validate_property_expression(&mut self, prop: &PropertyExpression) -> DBResult<()> {
        // 验证属性是否存在
        // 这里需要根据变量名和属性名检查schema
        Ok(())
    }
    
    /// 验证函数调用
    async fn validate_function_call(&mut self, func: &FunctionCall) -> DBResult<()> {
        // 验证函数是否存在
        if !self.base.qctx.function_registry().contains(&func.function_name) {
            return Err(DBError::Validation(
                crate::core::error::ValidationError::SemanticError(
                    format!("Function not found: {}", func.function_name)
                )
            ));
        }
        
        // 验证参数数量
        // 验证参数类型
        Ok(())
    }
    
    /// 验证二元表达式
    async fn validate_binary_expression(&mut self, bin: &BinaryExpression) -> DBResult<()> {
        self.validate_expression(&bin.left).await?;
        self.validate_expression(&bin.right).await?;
        Ok(())
    }
    
    /// 验证属性
    async fn validate_properties(&mut self, properties: &HashMap<String, Expression>) -> DBResult<()> {
        for (name, expr) in properties {
            self.validate_expression(expr).await?;
        }
        Ok(())
    }
    
    /// 创建扫描节点
    fn create_scan_node(&self, pattern: &Pattern) -> DBResult<Box<dyn PlanNode>> {
        // 根据模式创建适当的扫描节点
        if pattern.has_labels() {
            // 创建标签扫描
            Ok(Box::new(IndexScanNode::new(pattern.clone())))
        } else {
            // 创建全图扫描
            Ok(Box::new(ScanVerticesNode::new()))
        }
    }
    
    /// 创建过滤节点
    fn create_filter_node(&self, where_clause: &Option<WhereClause>, input: Box<dyn PlanNode>) -> DBResult<Box<dyn PlanNode>> {
        if let Some(where_clause) = where_clause {
            Ok(Box::new(FilterNode::new(where_clause.expression.clone(), input)))
        } else {
            Ok(input)
        }
    }
    
    /// 创建投影节点
    fn create_project_node(&self, return_clause: &Option<ReturnClause>, input: Box<dyn PlanNode>) -> DBResult<Box<dyn PlanNode>> {
        // 根据RETURN子句创建投影节点
        Ok(Box::new(ProjectNode::new(vec![], input)))
    }
}

#[async_trait]
impl ValidatorExt for MatchValidator {
    async fn validate_impl(&mut self) -> DBResult<()> {
        // 验证模式
        self.validate_pattern().await?;
        
        // 验证WHERE子句
        self.validate_where_clause().await?;
        
        // 验证RETURN子句
        self.validate_return_clause().await?;
        
        Ok(())
    }
    
    async fn to_plan_impl(&mut self) -> DBResult<ExecutionPlan> {
        // 创建扫描节点
        let scan_node = self.create_scan_node(&self.match_clause.patterns[0])?;
        
        // 创建过滤节点
        let filter_node = self.create_filter_node(&self.match_clause.where_clause, scan_node)?;
        
        // 创建投影节点
        let project_node = self.create_project_node(&None, filter_node)?; // 暂时没有RETURN子句
        
        Ok(ExecutionPlan {
            plan_id: PlanId::new(),
            root: project_node.clone(),
            tail: scan_node,
        })
    }
}

impl Validator for MatchValidator {
    async fn validate(&mut self) -> DBResult<()> {
        self.base.validate().await?;
        self.validate_impl().await
    }
    
    async fn to_plan(&mut self) -> DBResult<ExecutionPlan> {
        self.to_plan_impl().await
    }
    
    fn ast_context(&self) -> &AstContext {
        self.base.ast_context()
    }
    
    fn name(&self) -> &'static str {
        "MatchValidator"
    }
    
    fn input_var_name(&self) -> Option<&str> {
        self.base.input_var_name()
    }
    
    fn set_input_var_name(&mut self, name: String) {
        self.base.set_input_var_name(name);
    }
    
    fn output_columns(&self) -> &[ColumnDefinition] {
        self.base.output_columns()
    }
    
    fn input_columns(&self) -> &[ColumnDefinition] {
        self.base.input_columns()
    }
}
```

#### CreateValidator
```rust
/// CREATE语句验证器
pub struct CreateValidator {
    base: BaseValidator,
    create_clause: CreateClause,
}

impl CreateValidator {
    pub fn new(sentence: Sentence, qctx: Arc<QueryContext>) -> DBResult<Self> {
        let create_clause = extract_create_clause(&sentence)?;
        
        Ok(Self {
            base: BaseValidator::new(sentence, qctx),
            create_clause,
        })
    }
    
    /// 验证创建模式
    async fn validate_create_pattern(&mut self) -> DBResult<()> {
        for pattern in &self.create_clause.patterns {
            self.validate_pattern(pattern).await?;
        }
        Ok(())
    }
    
    /// 验证模式
    async fn validate_pattern(&mut self, pattern: &Pattern) -> DBResult<()> {
        for pattern_part in &pattern.parts {
            self.validate_pattern_part(pattern_part).await?;
        }
        Ok(())
    }
    
    /// 验证模式部分
    async fn validate_pattern_part(&mut self, pattern_part: &PatternPart) -> DBResult<()> {
        // 验证节点创建
        self.validate_node_creation(&pattern_part.node).await?;
        
        // 验证关系创建
        for rel in &pattern_part.relationships {
            self.validate_relationship_creation(rel).await?;
        }
        
        Ok(())
    }
    
    /// 验证节点创建
    async fn validate_node_creation(&mut self, node: &NodePattern) -> DBResult<()> {
        // 验证标签是否存在
        if let Some(labels) = &node.labels {
            for label in labels {
                if !self.base.qctx.schema_manager().tag_exists(label)? {
                    return Err(DBError::Validation(
                        crate::core::error::ValidationError::SemanticError(
                            format!("Tag not found: {}", label)
                        )
                    ));
                }
            }
        }
        
        // 验证属性
        if let Some(properties) = &node.properties {
            self.validate_properties(properties).await?;
        }
        
        Ok(())
    }
    
    /// 验证关系创建
    async fn validate_relationship_creation(&mut self, rel: &RelationshipPattern) -> DBResult<()> {
        // 验证边类型是否存在
        for edge_type in &rel.types {
            if !self.base.qctx.schema_manager().edge_type_exists(edge_type)? {
                return Err(DBError::Validation(
                    crate::core::error::ValidationError::SemanticError(
                        format!("Edge type not found: {}", edge_type)
                    )
                ));
            }
        }
        
        // 验证属性
        if let Some(properties) = &rel.properties {
            self.validate_properties(properties).await?;
        }
        
        Ok(())
    }
    
    /// 验证属性
    async fn validate_properties(&mut self, properties: &HashMap<String, Expression>) -> DBResult<()> {
        for (name, expr) in properties {
            self.validate_expression(expr).await?;
        }
        Ok(())
    }
    
    /// 验证表达式
    async fn validate_expression(&mut self, expr: &Expression) -> DBResult<()> {
        match expr {
            Expression::FunctionCall(func) => {
                self.validate_function_call(func).await?;
            }
            Expression::Binary(bin) => {
                self.validate_binary_expression(bin).await?;
            }
            _ => {} // 其他表达式类型的验证
        }
        Ok(())
    }
    
    /// 验证函数调用
    async fn validate_function_call(&mut self, func: &FunctionCall) -> DBResult<()> {
        // 验证函数是否存在
        if !self.base.qctx.function_registry().contains(&func.function_name) {
            return Err(DBError::Validation(
                crate::core::error::ValidationError::SemanticError(
                    format!("Function not found: {}", func.function_name)
                )
            ));
        }
        Ok(())
    }
    
    /// 验证二元表达式
    async fn validate_binary_expression(&mut self, bin: &BinaryExpression) -> DBResult<()> {
        self.validate_expression(&bin.left).await?;
        self.validate_expression(&bin.right).await?;
        Ok(())
    }
    
    /// 创建插入节点
    fn create_insert_vertices_node(&self, patterns: &[Pattern]) -> DBResult<Box<dyn PlanNode>> {
        let vertices = self.extract_vertices_from_patterns(patterns)?;
        Ok(Box::new(InsertVerticesNode::new(vertices)))
    }
    
    /// 创建插入边节点
    fn create_insert_edges_node(&self, patterns: &[Pattern]) -> DBResult<Box<dyn PlanNode>> {
        let edges = self.extract_edges_from_patterns(patterns)?;
        Ok(Box::new(InsertEdgesNode::new(edges)))
    }
    
    /// 从模式中提取顶点
    fn extract_vertices_from_patterns(&self, patterns: &[Pattern]) -> DBResult<Vec<Vertex>> {
        // 实现顶点提取逻辑
        Ok(vec![])
    }
    
    /// 从模式中提取边
    fn extract_edges_from_patterns(&self, patterns: &[Pattern]) -> DBResult<Vec<Edge>> {
        // 实现边提取逻辑
        Ok(vec![])
    }
}

#[async_trait]
impl ValidatorExt for CreateValidator {
    async fn validate_impl(&mut self) -> DBResult<()> {
        // 验证创建模式
        self.validate_create_pattern().await?;
        
        Ok(())
    }
    
    async fn to_plan_impl(&mut self) -> DBResult<ExecutionPlan> {
        // 创建插入顶点节点
        let insert_vertices_node = self.create_insert_vertices_node(&self.create_clause.patterns)?;
        
        // 创建插入边节点
        let insert_edges_node = self.create_insert_edges_node(&self.create_clause.patterns)?;
        
        // 创建输出节点
        let output_node = Box::new(OutputNode::new(insert_edges_node));
        
        Ok(ExecutionPlan {
            plan_id: PlanId::new(),
            root: output_node.clone(),
            tail: insert_vertices_node,
        })
    }
}

impl Validator for CreateValidator {
    async fn validate(&mut self) -> DBResult<()> {
        self.base.validate().await?;
        self.validate_impl().await
    }
    
    async fn to_plan(&mut self) -> DBResult<ExecutionPlan> {
        self.to_plan_impl().await
    }
    
    fn ast_context(&self) -> &AstContext {
        self.base.ast_context()
    }
    
    fn name(&self) -> &'static str {
        "CreateValidator"
    }
    
    fn input_var_name(&self) -> Option<&str> {
        self.base.input_var_name()
    }
    
    fn set_input_var_name(&mut self, name: String) {
        self.base.set_input_var_name(name);
    }
    
    fn output_columns(&self) -> &[ColumnDefinition] {
        self.base.output_columns()
    }
    
    fn input_columns(&self) -> &[ColumnDefinition] {
        self.base.input_columns()
    }
}
```

### 4. 验证器工厂实现

#### CypherValidatorFactory
```rust
/// Cypher验证器工厂
pub struct CypherValidatorFactory;

impl CypherValidatorFactory {
    pub fn create_validator(sentence: &Sentence, qctx: Arc<QueryContext>) -> DBResult<Box<dyn Validator>> {
        match sentence {
            Sentence::Match(_) => Ok(Box::new(MatchValidator::new(sentence.clone(), qctx)?)),
            Sentence::Create(_) => Ok(Box::new(CreateValidator::new(sentence.clone(), qctx)?)),
            Sentence::Return(_) => Ok(Box::new(ReturnValidator::new(sentence.clone(), qctx)?)),
            Sentence::With(_) => Ok(Box::new(WithValidator::new(sentence.clone(), qctx)?)),
            Sentence::Set(_) => Ok(Box::new(SetValidator::new(sentence.clone(), qctx)?)),
            Sentence::Delete(_) => Ok(Box::new(DeleteValidator::new(sentence.clone(), qctx)?)),
            Sentence::Merge(_) => Ok(Box::new(MergeValidator::new(sentence.clone(), qctx)?)),
            Sentence::Unwind(_) => Ok(Box::new(UnwindValidator::new(sentence.clone(), qctx)?)),
            _ => Err(DBError::Validation(
                crate::core::error::ValidationError::UnsupportedStatement(
                    format!("Unsupported Cypher sentence: {:?}", sentence)
                )
            )),
        }
    }
}

impl ValidatorCreator for CypherValidatorFactory {
    fn create(&self, sentence: &Sentence, qctx: Arc<QueryContext>) -> DBResult<Box<dyn Validator>> {
        Self::create_validator(sentence, qctx)
    }
}
```

### 5. 验证器注册

#### ValidatorRegistry
```rust
/// 验证器注册表
pub struct ValidatorRegistry {
    factory: ValidatorFactory,
}

impl ValidatorRegistry {
    pub fn new() -> Self {
        let mut factory = ValidatorFactory::new();
        
        // 注册Cypher验证器
        factory.register_creator(
            SentenceType::Match,
            Box::new(CypherValidatorFactory),
        );
        factory.register_creator(
            SentenceType::Create,
            Box::new(CypherValidatorFactory),
        );
        // ... 注册其他类型的验证器
        
        Self { factory }
    }
    
    /// 创建验证器
    pub fn create_validator(
        &self,
        sentence: &Sentence,
        qctx: Arc<QueryContext>,
    ) -> DBResult<Box<dyn Validator>> {
        self.factory.create_validator(sentence, qctx)
    }
}

/// 全局验证器注册表
lazy_static::lazy_static! {
    pub static ref VALIDATOR_REGISTRY: ValidatorRegistry = ValidatorRegistry::new();
}
```

## 实施步骤

### 第一步：设计验证器接口 (3-4天)
1. 定义Validator trait
2. 定义ValidatorCreator trait
3. 定义ValidationError类型
4. 实现BaseValidator

### 第二步：重构MatchValidator (4-5天)
1. 实现MatchValidator结构
2. 实现模式验证逻辑
3. 实现规划逻辑
4. 添加单元测试

### 第三步：重构CreateValidator (3-4天)
1. 实现CreateValidator结构
2. 实现创建验证逻辑
3. 实现规划逻辑
4. 添加单元测试

### 第四步：实现其他验证器 (5-6天)
1. 实现ReturnValidator
2. 实现WithValidator
3. 实现SetValidator
4. 实现DeleteValidator
5. 实现其他验证器

### 第五步：实现验证器工厂 (2-3天)
1. 实现ValidatorFactory
2. 实现CypherValidatorFactory
3. 实现ValidatorRegistry
4. 添加集成测试

### 第六步：测试和优化 (2-3天)
1. 编写单元测试
2. 编写集成测试
3. 性能优化
4. 文档更新

## 预期收益

### 1. 架构简化
- **验证+规划一体化**: 减少验证和规划之间的复杂性
- **统一接口**: 所有验证器使用相同的接口
- **工厂模式**: 统一的验证器创建机制

### 2. 性能提升
- **减少转换**: 验证和规划在同一阶段完成
- **缓存优化**: 验证结果可以缓存
- **并行处理**: 验证器可以并行执行

### 3. 可维护性增强
- **代码复用**: BaseValidator提供通用功能
- **职责明确**: 每个验证器职责单一
- **易于扩展**: 新增验证器只需实现Validator trait

### 4. 可扩展性提升
- **语言支持**: 易于添加新的查询语言支持
- **功能扩展**: 易于添加新的验证功能
- **插件化**: 验证器可以作为插件动态加载

这个设计方案基于nebula-graph的Validator模式，实现了验证+规划的一体化设计，既保持了架构的简洁性，又提供了足够的灵活性。