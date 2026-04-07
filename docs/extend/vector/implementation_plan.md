# 向量检索查询流程集成实施方案

> 创建日期：2026-04-07  
> 基于文档：[集成状态分析](integration_status_analysis.md)  
> 预计工作量：约 31 小时（4 个工作日）

---

## 一、总体方案

### 1.1 实施策略

采用**分层实现、逐步集成**的策略，按照查询处理流程依次实现：

```
Parser (解析器)
    ↓
Validator (验证器)
    ↓
Planner (规划器)
    ↓
PlanNode (计划节点)
    ↓
Executor (执行器)
    ↓
Factory (工厂)
```

### 1.2 核心设计原则

1. **复用现有模式** - 完全遵循全文检索模块的设计模式
2. **静态分发** - 使用枚举而非动态分发（trait object）
3. **类型安全** - 编译时检查，避免运行时错误
4. **向后兼容** - 不影响现有查询功能

---

## 二、详细实施方案

### Phase 1: AST 扩展 (预计 4 小时)

#### 1.1 创建向量检索 AST 定义

**文件**: `src/query/parser/ast/vector.rs` (新建)

```rust
//! Vector Search AST Definitions

use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::types::graph_schema::OrderDirection;
use crate::core::types::span::Span;
use serde::{Deserialize, Serialize};

/// 向量查询表达式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorQueryExpr {
    pub span: Span,
    pub query_type: VectorQueryType,
    pub fields: Vec<String>,
    pub query_text: String,
}

/// 向量查询类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorQueryType {
    Vector,      // 直接提供向量
    Text,        // 文本（需要嵌入服务）
    Parameter,   // 参数引用
}

/// 距离度量方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    Dot,
}

/// 创建向量索引语句
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVectorIndexStatement {
    pub span: Span,
    pub if_not_exists: bool,
    pub index_name: String,
    pub tag_name: String,
    pub field_name: String,
    pub config: VectorIndexConfig,
}

/// 向量索引配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndexConfig {
    pub vector_size: usize,
    pub distance: DistanceMetric,
    pub hnsw_m: Option<usize>,
    pub hnsw_ef_construct: Option<usize>,
}

/// 删除向量索引语句
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropVectorIndexStatement {
    pub span: Span,
    pub if_exists: bool,
    pub index_name: String,
}

/// 向量搜索语句
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchVectorStatement {
    pub span: Span,
    pub index_name: String,
    pub query: VectorQueryExpr,
    pub threshold: Option<f32>,
    pub where_clause: Option<ContextualExpression>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub yield_clause: Option<YieldClause>,
}

/// Yield 子句
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldClause {
    pub columns: Vec<YieldColumn>,
}

/// Yield 列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldColumn {
    pub expr: ContextualExpression,
    pub alias: Option<String>,
}

/// 向量查找语句（用于 LOOKUP）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LookupVector {
    pub span: Span,
    pub schema_name: String,
    pub index_name: String,
    pub query: VectorQueryExpr,
    pub yield_clause: Option<YieldClause>,
    pub limit: Option<usize>,
}

/// MATCH 中的向量条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchVector {
    pub span: Span,
    pub pattern: Vec<Pattern>,
    pub vector_condition: VectorCondition,
    pub yield_clause: Option<YieldClause>,
}

/// 向量条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorCondition {
    pub field_name: String,
    pub query: VectorQueryExpr,
    pub threshold: Option<f32>,
}

/// 模式（用于 MATCH）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    // 简化表示，实际使用现有 Pattern 定义
    pub description: String,
}
```

#### 1.2 更新 AST 模块导出

**文件**: `src/query/parser/ast/mod.rs`

**修改**:
```rust
pub mod fulltext;
pub mod vector;  // ← 新增
pub mod pattern;
pub mod types;

// 重新导出
pub use vector::*;  // ← 新增
```

#### 1.3 更新 Stmt 枚举

**文件**: `src/query/parser/ast/stmt.rs` (第 80-100 行附近)

**修改**:
```rust
pub enum Stmt {
    // ... 现有语句 ...
    
    // Full-text search statements
    CreateFulltextIndex(CreateFulltextIndex),
    DropFulltextIndex(DropFulltextIndex),
    AlterFulltextIndex(AlterFulltextIndex),
    ShowFulltextIndex(ShowFulltextIndex),
    DescribeFulltextIndex(DescribeFulltextIndex),
    Search(SearchStatement),
    LookupFulltext(LookupFulltext),
    MatchFulltext(MatchFulltext),
    
    // Vector search statements (新增)
    CreateVectorIndex(CreateVectorIndexStatement),
    DropVectorIndex(DropVectorIndexStatement),
    SearchVector(SearchVectorStatement),
    LookupVector(LookupVector),
    MatchVector(MatchVector),
}
```

#### 1.4 更新 Stmt 的 span() 方法

**文件**: `src/query/parser/ast/stmt.rs` (第 120-170 行附近)

**修改**:
```rust
pub fn span(&self) -> Span {
    match self {
        // ... 现有分支 ...
        
        // Full-text search statements
        Stmt::CreateFulltextIndex(s) => s.span,
        Stmt::DropFulltextIndex(s) => s.span,
        Stmt::AlterFulltextIndex(s) => s.span,
        Stmt::ShowFulltextIndex(s) => s.span,
        Stmt::DescribeFulltextIndex(s) => s.span,
        Stmt::Search(s) => s.span,
        Stmt::LookupFulltext(s) => s.span,
        Stmt::MatchFulltext(s) => s.span,
        
        // Vector search statements (新增)
        Stmt::CreateVectorIndex(s) => s.span,
        Stmt::DropVectorIndex(s) => s.span,
        Stmt::SearchVector(s) => s.span,
        Stmt::LookupVector(s) => s.span,
        Stmt::MatchVector(s) => s.span,
    }
}
```

#### 1.5 更新 Stmt 的 kind() 方法

**文件**: `src/query/parser/ast/stmt.rs` (第 180-230 行附近)

**修改**:
```rust
pub fn kind(&self) -> &'static str {
    match self {
        // ... 现有分支 ...
        
        // Full-text search
        Stmt::CreateFulltextIndex(_) => "CREATE FULLTEXT INDEX",
        Stmt::DropFulltextIndex(_) => "DROP FULLTEXT INDEX",
        Stmt::Search(_) => "SEARCH",
        Stmt::LookupFulltext(_) => "LOOKUP FULLTEXT",
        Stmt::MatchFulltext(_) => "MATCH FULLTEXT",
        
        // Vector search (新增)
        Stmt::CreateVectorIndex(_) => "CREATE VECTOR INDEX",
        Stmt::DropVectorIndex(_) => "DROP VECTOR INDEX",
        Stmt::SearchVector(_) => "SEARCH VECTOR",
        Stmt::LookupVector(_) => "LOOKUP VECTOR",
        Stmt::MatchVector(_) => "MATCH VECTOR",
    }
}
```

---

### Phase 2: 解析器扩展 (预计 3 小时)

#### 2.1 创建向量检索解析器

**文件**: `src/query/parser/parser/vector.rs` (新建)

```rust
//! Vector Search Parser

use crate::core::types::expr::contextual::ContextualExpression;
use crate::query::parser::ast::vector::*;
use crate::query::parser::parser::{ParseError, Parser, ParserResult};
use crate::core::types::span::Span;

impl Parser {
    /// 解析 SEARCH VECTOR 语句
    pub fn parse_search_vector(&mut self) -> Result<SearchVectorStatement, ParseError> {
        let span = self.current_span();
        
        // 解析 SEARCH VECTOR index_name
        self.expect_keyword(Keyword::SEARCH)?;
        self.expect_keyword(Keyword::VECTOR)?;
        let index_name = self.parse_identifier()?;
        
        // 解析 WITH vector = [...] 或 WITH text = '...'
        let query = self.parse_vector_query()?;
        
        // 解析可选的 threshold
        let threshold = self.parse_optional_threshold()?;
        
        // 解析可选的 WHERE
        let where_clause = self.parse_optional_where()?;
        
        // 解析可选的 LIMIT
        let limit = self.parse_optional_limit()?;
        
        // 解析可选的 OFFSET
        let offset = self.parse_optional_offset()?;
        
        // 解析 RETURN/YIELD
        let yield_clause = self.parse_optional_yield()?;
        
        Ok(SearchVectorStatement {
            span,
            index_name,
            query,
            threshold,
            where_clause,
            limit,
            offset,
            yield_clause,
        })
    }
    
    /// 解析 CREATE VECTOR INDEX 语句
    pub fn parse_create_vector_index(&mut self) -> Result<CreateVectorIndexStatement, ParseError> {
        let span = self.current_span();
        
        self.expect_keyword(Keyword::CREATE)?;
        self.expect_keyword(Keyword::VECTOR)?;
        self.expect_keyword(Keyword::INDEX)?;
        
        let if_not_exists = self.parse_if_not_exists()?;
        let index_name = self.parse_identifier()?;
        
        self.expect_keyword(Keyword::ON)?;
        let tag_name = self.parse_identifier()?;
        self.expect_token(Token::LParen)?;
        let field_name = self.parse_identifier()?;
        self.expect_token(Token::RParen)?;
        
        let config = self.parse_vector_index_config()?;
        
        Ok(CreateVectorIndexStatement {
            span,
            if_not_exists,
            index_name,
            tag_name,
            field_name,
            config,
        })
    }
    
    /// 解析 DROP VECTOR INDEX 语句
    pub fn parse_drop_vector_index(&mut self) -> Result<DropVectorIndexStatement, ParseError> {
        let span = self.current_span();
        
        self.expect_keyword(Keyword::DROP)?;
        self.expect_keyword(Keyword::VECTOR)?;
        self.expect_keyword(Keyword::INDEX)?;
        
        let if_exists = self.parse_if_exists()?;
        let index_name = self.parse_identifier()?;
        
        Ok(DropVectorIndexStatement {
            span,
            if_exists,
            index_name,
        })
    }
    
    /// 解析向量查询表达式
    fn parse_vector_query(&mut self) -> Result<VectorQueryExpr, ParseError> {
        let span = self.current_span();
        self.expect_keyword(Keyword::WITH)?;
        
        let keyword = self.parse_identifier()?;
        self.expect_token(Token::Eq)?;
        
        let (query_type, fields, query_text) = if keyword == "vector" {
            // vector = [0.1, 0.2, ...]
            let vector = self.parse_vector_literal()?;
            (VectorQueryType::Vector, vec![], vector)
        } else if keyword == "text" {
            // text = 'search query'
            let text = self.parse_string_literal()?;
            (VectorQueryType::Text, vec![], text)
        } else if keyword == "param" || keyword == "parameter" {
            // param = $param_name
            let param = self.parse_parameter()?;
            (VectorQueryType::Parameter, vec![], param)
        } else {
            return Err(ParseError::new(
                span,
                format!("Expected 'vector', 'text', or 'param', found '{}'", keyword),
            ));
        };
        
        Ok(VectorQueryExpr {
            span,
            query_type,
            fields,
            query_text,
        })
    }
    
    /// 解析向量索引配置
    fn parse_vector_index_config(&mut self) -> Result<VectorIndexConfig, ParseError> {
        self.expect_keyword(Keyword::WITH)?;
        self.expect_token(Token::LParen)?;
        
        let mut vector_size = None;
        let mut distance = DistanceMetric::Cosine;
        let mut hnsw_m = None;
        let mut hnsw_ef_construct = None;
        
        loop {
            let key = self.parse_identifier()?;
            self.expect_token(Token::Eq)?;
            
            match key.as_str() {
                "vector_size" => {
                    vector_size = Some(self.parse_number()?);
                }
                "distance" => {
                    let dist = self.parse_string_literal()?;
                    distance = match dist.as_str() {
                        "cosine" => DistanceMetric::Cosine,
                        "euclidean" => DistanceMetric::Euclidean,
                        "dot" => DistanceMetric::Dot,
                        _ => {
                            return Err(ParseError::new(
                                self.current_span(),
                                format!("Unknown distance metric '{}'", dist),
                            ))
                        }
                    };
                }
                "hnsw_m" => {
                    hnsw_m = Some(self.parse_number()?);
                }
                "hnsw_ef_construct" => {
                    hnsw_ef_construct = Some(self.parse_number()?);
                }
                _ => {
                    return Err(ParseError::new(
                        self.current_span(),
                        format!("Unknown config option '{}'", key),
                    ))
                }
            }
            
            if self.consume_token(Token::Comma).is_none() {
                break;
            }
        }
        
        self.expect_token(Token::RParen)?;
        
        let vector_size = vector_size.ok_or_else(|| {
            ParseError::new(
                self.current_span(),
                "vector_size is required".to_string(),
            )
        })?;
        
        Ok(VectorIndexConfig {
            vector_size,
            distance,
            hnsw_m,
            hnsw_ef_construct,
        })
    }
    
    /// 解析向量字面量
    fn parse_vector_literal(&mut self) -> Result<String, ParseError> {
        self.expect_token(Token::LBracket)?;
        let mut elements = Vec::new();
        
        loop {
            let num = self.parse_number::<f32>()?;
            elements.push(format!("{}", num));
            
            if self.consume_token(Token::Comma).is_none() {
                break;
            }
        }
        
        self.expect_token(Token::RBracket)?;
        Ok(format!("[{}]", elements.join(", ")))
    }
}
```

#### 2.2 更新主解析器

**文件**: `src/query/parser/parser.rs`

**修改**: 在语句解析入口添加向量检索语句的识别

```rust
pub fn parse_statement(&mut self) -> Result<Ast, ParseError> {
    let stmt = if self.check_keyword(Keyword::SEARCH) {
        // 判断是 SEARCH 还是 SEARCH VECTOR
        if self.check_next_keyword(Keyword::VECTOR) {
            Stmt::SearchVector(self.parse_search_vector()?)
        } else {
            Stmt::Search(self.parse_search()?)
        }
    } else if self.check_keyword(Keyword::CREATE) {
        if self.check_next_keywords(&[Keyword::VECTOR, Keyword::INDEX]) {
            Stmt::CreateVectorIndex(self.parse_create_vector_index()?)
        } else if self.check_next_keywords(&[Keyword::FULLTEXT, Keyword::INDEX]) {
            Stmt::CreateFulltextIndex(self.parse_create_fulltext_index()?)
        } else {
            // ... 其他 CREATE 语句
        }
    } else if self.check_keyword(Keyword::DROP) {
        if self.check_next_keywords(&[Keyword::VECTOR, Keyword::INDEX]) {
            Stmt::DropVectorIndex(self.parse_drop_vector_index()?)
        } else if self.check_next_keywords(&[Keyword::FULLTEXT, Keyword::INDEX]) {
            Stmt::DropFulltextIndex(self.parse_drop_fulltext_index()?)
        } else {
            // ... 其他 DROP 语句
        }
    } else {
        // ... 其他语句
    };
    
    Ok(Ast::new(stmt, self.expr_context.clone()))
}
```

---

### Phase 3: 验证器扩展 (预计 3 小时)

#### 3.1 创建向量检索验证器

**文件**: `src/query/validator/vector_validator.rs` (新建)

```rust
//! Vector Search Validator

use crate::query::ast::vector::*;
use crate::query::validator::{ValidationContext, ValidationError, ValidationResult};

pub struct VectorValidator;

impl VectorValidator {
    /// 验证 CREATE VECTOR INDEX 语句
    pub fn validate_create_vector_index(
        ctx: &mut ValidationContext,
        stmt: &CreateVectorIndexStatement,
    ) -> ValidationResult {
        // 验证索引名称
        if stmt.index_name.is_empty() {
            return Err(ValidationError::EmptyIndexName);
        }
        
        // 验证 Tag 是否存在
        if !ctx.tag_exists(&stmt.tag_name)? {
            return Err(ValidationError::TagNotFound(stmt.tag_name.clone()));
        }
        
        // 验证字段是否存在
        if !ctx.field_exists(&stmt.tag_name, &stmt.field_name)? {
            return Err(ValidationError::FieldNotFound {
                tag: stmt.tag_name.clone(),
                field: stmt.field_name.clone(),
            });
        }
        
        // 验证向量维度
        if stmt.config.vector_size == 0 || stmt.config.vector_size > 65536 {
            return Err(ValidationError::InvalidVectorSize(stmt.config.vector_size));
        }
        
        // 验证索引是否已存在
        if !stmt.if_not_exists && ctx.vector_index_exists(&stmt.index_name)? {
            return Err(ValidationError::IndexAlreadyExists(stmt.index_name.clone()));
        }
        
        Ok(())
    }
    
    /// 验证 DROP VECTOR INDEX 语句
    pub fn validate_drop_vector_index(
        ctx: &mut ValidationContext,
        stmt: &DropVectorIndexStatement,
    ) -> ValidationResult {
        // 验证索引名称
        if stmt.index_name.is_empty() {
            return Err(ValidationError::EmptyIndexName);
        }
        
        // 验证索引是否存在（除非 IF EXISTS）
        if !stmt.if_exists && !ctx.vector_index_exists(&stmt.index_name)? {
            return Err(ValidationError::IndexNotFound(stmt.index_name.clone()));
        }
        
        Ok(())
    }
    
    /// 验证 SEARCH VECTOR 语句
    pub fn validate_search_vector(
        ctx: &mut ValidationContext,
        stmt: &SearchVectorStatement,
    ) -> ValidationResult {
        // 验证索引是否存在
        if !ctx.vector_index_exists(&stmt.index_name)? {
            return Err(ValidationError::IndexNotFound(stmt.index_name.clone()));
        }
        
        // 验证查询向量
        match &stmt.query.query_type {
            VectorQueryType::Vector => {
                // 验证向量格式
                // TODO: 解析并验证向量
            }
            VectorQueryType::Text => {
                // 需要嵌入服务
                if !ctx.embedding_service_available()? {
                    return Err(ValidationError::EmbeddingServiceNotAvailable);
                }
            }
            VectorQueryType::Parameter => {
                // 验证参数是否存在
                // TODO: 验证参数
            }
        }
        
        // 验证 threshold
        if let Some(threshold) = stmt.threshold {
            if threshold < 0.0 || threshold > 1.0 {
                return Err(ValidationError::InvalidThreshold(threshold));
            }
        }
        
        // 验证 WHERE 子句
        if let Some(where_clause) = &stmt.where_clause {
            Self::validate_where_clause(ctx, where_clause)?;
        }
        
        // 验证 LIMIT
        if let Some(limit) = stmt.limit {
            if limit == 0 || limit > 10000 {
                return Err(ValidationError::InvalidLimit(limit));
            }
        }
        
        Ok(())
    }
    
    /// 验证 WHERE 子句
    fn validate_where_clause(
        ctx: &ValidationContext,
        where_clause: &ContextualExpression,
    ) -> ValidationResult {
        // 验证表达式的有效性
        // TODO: 实现表达式验证
        Ok(())
    }
}
```

#### 3.2 更新验证器枚举

**文件**: `src/query/validator/mod.rs`

**修改**:
```rust
pub mod validator_enum;
pub mod fulltext_validator;
pub mod vector_validator;  // ← 新增

pub use validator_enum::ValidatorEnum;
pub use vector_validator::VectorValidator;  // ← 新增
```

**文件**: `src/query/validator/validator_enum.rs`

**修改**: 添加向量验证器的匹配逻辑

```rust
impl ValidatorEnum {
    pub fn validate(&self, stmt: &Stmt, ctx: &mut ValidationContext) -> ValidationResult {
        match stmt {
            // ... 现有验证 ...
            
            // Vector search statements (新增)
            Stmt::CreateVectorIndex(s) => {
                VectorValidator::validate_create_vector_index(ctx, s)
            }
            Stmt::DropVectorIndex(s) => {
                VectorValidator::validate_drop_vector_index(ctx, s)
            }
            Stmt::SearchVector(s) => {
                VectorValidator::validate_search_vector(ctx, s)
            }
            Stmt::LookupVector(s) => {
                VectorValidator::validate_lookup_vector(ctx, s)
            }
            Stmt::MatchVector(s) => {
                VectorValidator::validate_match_vector(ctx, s)
            }
            
            _ => Ok(()),
        }
    }
}
```

---

### Phase 4: Planner 扩展 (预计 4 小时)

#### 4.1 创建向量检索规划器

**文件**: `src/query/planning/planner/vector_planner.rs` (新建)

```rust
//! Vector Search Planner

use std::sync::Arc;

use crate::query::ast::vector::*;
use crate::query::planning::plan::ExecutionPlan;
use crate::query::planning::plan::SubPlan;
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;

/// 向量搜索规划器
#[derive(Debug, Clone, Default)]
pub struct VectorSearchPlanner;

impl VectorSearchPlanner {
    pub fn new() -> Self {
        Self
    }
}

impl Planner for VectorSearchPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let stmt = validated.stmt();
        let space_name = qctx.space_name().unwrap_or_else(|| "default".to_string());
        
        match stmt {
            Stmt::CreateVectorIndex(create) => {
                self.plan_create_vector_index(create, space_name)
            }
            Stmt::DropVectorIndex(drop) => {
                self.plan_drop_vector_index(drop, space_name)
            }
            Stmt::SearchVector(search) => {
                self.plan_search_vector(search, qctx)
            }
            Stmt::LookupVector(lookup) => {
                self.plan_lookup_vector(lookup, space_name)
            }
            Stmt::MatchVector(match_stmt) => {
                self.plan_match_vector(match_stmt, qctx)
            }
            _ => Err(PlannerError::PlanGenerationFailed(
                "Not a vector search statement".to_string(),
            )),
        }
    }
    
    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(
            stmt,
            Stmt::CreateVectorIndex(_)
                | Stmt::DropVectorIndex(_)
                | Stmt::SearchVector(_)
                | Stmt::LookupVector(_)
                | Stmt::MatchVector(_)
        )
    }
}

impl VectorSearchPlanner {
    /// 为 CREATE VECTOR INDEX 生成执行计划
    fn plan_create_vector_index(
        &self,
        stmt: &CreateVectorIndexStatement,
        space_name: String,
    ) -> Result<SubPlan, PlannerError> {
        use crate::query::planning::plan::core::nodes::management::vector_nodes::CreateVectorIndexNode;
        
        let node = CreateVectorIndexNode::new(
            stmt.index_name.clone(),
            space_name,
            stmt.tag_name.clone(),
            stmt.field_name.clone(),
            stmt.config.clone(),
            stmt.if_not_exists,
        );
        
        Ok(SubPlan::new(Some(node.into_enum()), None))
    }
    
    /// 为 DROP VECTOR INDEX 生成执行计划
    fn plan_drop_vector_index(
        &self,
        stmt: &DropVectorIndexStatement,
        space_name: String,
    ) -> Result<SubPlan, PlannerError> {
        use crate::query::planning::plan::core::nodes::management::vector_nodes::DropVectorIndexNode;
        
        let node = DropVectorIndexNode::new(
            stmt.index_name.clone(),
            space_name,
            stmt.if_exists,
        );
        
        Ok(SubPlan::new(Some(node.into_enum()), None))
    }
    
    /// 为 SEARCH VECTOR 生成执行计划
    fn plan_search_vector(
        &self,
        stmt: &SearchVectorStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        use crate::query::planning::plan::core::nodes::data_access::vector_search::VectorSearchNode;
        
        // 获取 space_id
        let space_id = qctx.space_id().unwrap_or(0);
        
        // 解析索引元数据，获取 tag_name 和 field_name
        let (tag_name, field_name) = self.resolve_index_metadata(&stmt.index_name, qctx.as_ref())?;
        
        let node = VectorSearchNode::new(
            stmt.index_name.clone(),
            space_id,
            tag_name,
            field_name,
            stmt.query.clone(),
            stmt.threshold,
            stmt.where_clause.clone(),
            stmt.limit.unwrap_or(10),
            stmt.offset.unwrap_or(0),
            self.extract_output_fields(&stmt.yield_clause),
        );
        
        Ok(SubPlan::new(Some(node.into_enum()), None))
    }
    
    /// 为 LOOKUP VECTOR 生成执行计划
    fn plan_lookup_vector(
        &self,
        stmt: &LookupVector,
        space_name: String,
    ) -> Result<SubPlan, PlannerError> {
        // 类似 plan_search_vector，但用于 LOOKUP 场景
        todo!()
    }
    
    /// 为 MATCH VECTOR 生成执行计划
    fn plan_match_vector(
        &self,
        stmt: &MatchVector,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        // 将向量条件集成到 MATCH 计划中
        todo!()
    }
    
    /// 解析索引元数据
    fn resolve_index_metadata(
        &self,
        index_name: &str,
        qctx: &QueryContext,
    ) -> Result<(String, String), PlannerError> {
        // TODO: 从元数据中查询索引信息
        // 这里需要根据实际的元数据管理实现
        Ok(("Tag".to_string(), "field".to_string()))
    }
    
    /// 提取输出字段
    fn extract_output_fields(
        &self,
        yield_clause: &Option<YieldClause>,
    ) -> Vec<OutputField> {
        match yield_clause {
            Some(yield_clause) => {
                yield_clause.columns.iter().map(|col| {
                    OutputField {
                        name: col.expr.to_string(),
                        alias: col.alias.clone(),
                        expr: col.expr.clone(),
                    }
                }).collect()
            }
            None => vec![],
        }
    }
}
```

#### 4.2 更新 PlannerEnum

**文件**: [`src/query/planning/planner.rs`](file:///d:/项目/database/graphDB/src/query/planning/planner.rs) (第 260-270 行)

**修改**:
```rust
pub enum PlannerEnum {
    Match(MatchStatementPlanner),
    Go(GoPlanner),
    Lookup(LookupPlanner),
    Path(PathPlanner),
    Subgraph(SubgraphPlanner),
    FetchVertices(FetchVerticesPlanner),
    FetchEdges(FetchEdgesPlanner),
    Maintain(MaintainPlanner),
    UserManagement(UserManagementPlanner),
    Insert(InsertPlanner),
    Delete(DeletePlanner),
    Update(UpdatePlanner),
    Remove(RemovePlanner),
    Set(SetPlanner),
    Merge(MergePlanner),
    GroupBy(GroupByPlanner),
    SetOperation(SetOperationPlanner),
    Use(UsePlanner),
    With(WithPlanner),
    Return(ReturnPlanner),
    Yield(YieldPlanner),
    FulltextSearch(FulltextSearchPlanner),
    VectorSearch(VectorSearchPlanner),  // ← 新增
}
```

**修改**: `from_stmt` 方法 (第 290-330 行)

```rust
pub fn from_stmt(stmt: &Arc<Stmt>) -> Option<Self> {
    match stmt.as_ref() {
        // ... 现有分支 ...
        
        // Full-text search statements
        Stmt::CreateFulltextIndex(_)
        | Stmt::DropFulltextIndex(_)
        | Stmt::AlterFulltextIndex(_)
        | Stmt::ShowFulltextIndex(_)
        | Stmt::DescribeFulltextIndex(_)
        | Stmt::Search(_)
        | Stmt::LookupFulltext(_)
        | Stmt::MatchFulltext(_) => {
            Some(PlannerEnum::FulltextSearch(FulltextSearchPlanner::new()))
        }
        
        // Vector search statements (新增)
        Stmt::CreateVectorIndex(_)
        | Stmt::DropVectorIndex(_)
        | Stmt::SearchVector(_)
        | Stmt::LookupVector(_)
        | Stmt::MatchVector(_) => {
            Some(PlannerEnum::VectorSearch(VectorSearchPlanner::new()))
        }
        
        // ... 其他语句 ...
    }
}
```

**修改**: `transform` 方法 (第 360-370 行)

```rust
pub fn transform(
    &mut self,
    validated: &ValidatedStatement,
    qctx: Arc<QueryContext>,
) -> Result<SubPlan, PlannerError> {
    match self {
        PlannerEnum::Match(planner) => planner.transform(validated, qctx),
        PlannerEnum::Go(planner) => planner.transform(validated, qctx),
        // ... 其他规划器 ...
        PlannerEnum::FulltextSearch(planner) => planner.transform(validated, qctx),
        PlannerEnum::VectorSearch(planner) => planner.transform(validated, qctx),  // ← 新增
    }
}
```

---

（由于文档长度限制，后续 Phase 5-8 的详细代码实现将在下一个文档中继续）

---

## 三、测试策略

### 3.1 单元测试

每个模块都需要编写单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_search_vector() {
        // 测试解析 SEARCH VECTOR 语句
    }
    
    #[test]
    fn test_validate_create_vector_index() {
        // 测试验证 CREATE VECTOR INDEX
    }
}
```

### 3.2 集成测试

**文件**: `tests/integration_vector_query.rs` (新建)

```rust
//! Vector Query Integration Tests

mod common;

use std::sync::Arc;
use graphdb::query::QueryPipelineManager;
use graphdb::vector::VectorCoordinator;

#[tokio::test]
async fn test_create_vector_index() {
    let mut scenario = common::setup_vector_space().await;
    
    let result = scenario.exec_dql(
        "CREATE VECTOR INDEX idx_embedding ON Document(embedding) \
         WITH (vector_size = 768, distance = 'cosine')"
    );
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_vector() {
    let mut scenario = common::setup_vector_space().await;
    
    // 创建索引
    scenario.exec_dql(
        "CREATE VECTOR INDEX idx_embedding ON Document(embedding) \
         WITH (vector_size = 3, distance = 'cosine')"
    ).unwrap();
    
    // 插入数据
    scenario.exec_dql(
        "INSERT VERTEX Document(content, embedding) VALUES \
         'doc1':('test content', [0.1, 0.2, 0.3])"
    ).unwrap();
    
    // 等待同步
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 执行搜索
    let result = scenario.exec_dql(
        "SEARCH VECTOR idx_embedding WITH vector = [0.1, 0.2, 0.3] \
         LIMIT 10 RETURN id, score"
    );
    
    assert!(result.is_ok());
    let dataset = result.unwrap();
    assert_eq!(dataset.size(), 1);
}
```

---

## 四、验收标准

### 4.1 功能验收

- [ ] 可以解析 `CREATE VECTOR INDEX` 语句
- [ ] 可以解析 `DROP VECTOR INDEX` 语句
- [ ] 可以解析 `SEARCH VECTOR` 语句
- [ ] 可以验证向量索引创建
- [ ] 可以验证向量搜索查询
- [ ] 可以生成向量搜索执行计划
- [ ] 可以执行向量搜索并返回结果
- [ ] 向量搜索可以与图查询结合（MATCH + 向量条件）

### 4.2 性能验收

- [ ] 向量搜索响应时间 < 100ms (1000 条记录)
- [ ] 向量搜索响应时间 < 500ms (100 万条记录)
- [ ] 并发搜索 QPS > 100

### 4.3 质量验收

- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] 代码覆盖率 > 80%
- [ ] 无 clippy 警告
- [ ] 文档完整

---

## 五、风险与缓解

### 5.1 技术风险

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| 解析器冲突 | 高 | 低 | 仔细设计关键词识别逻辑 |
| 性能问题 | 高 | 中 | 早期性能测试，使用 Qdrant 优化 |
| 同步问题 | 中 | 中 | 充分的集成测试 |

### 5.2 进度风险

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| 实现复杂度低估 | 高 | 中 | 分阶段实施，优先核心功能 |
| 测试时间不足 | 中 | 高 | 提前编写测试用例 |

---

*文档生成时间：2026-04-07*  
*版本：v1.0*
