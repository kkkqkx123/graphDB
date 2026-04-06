# 向量检索查询集成

> 分析日期: 2026-04-06
> 依赖: 现有查询引擎架构

---

## 目录

- [1. 语法设计](#1-语法设计)
- [2. AST扩展](#2-ast扩展)
- [3. 解析器扩展](#3-解析器扩展)
- [4. 验证器扩展](#4-验证器扩展)
- [5. 计划节点](#5-计划节点)
- [6. 执行器实现](#6-执行器实现)
- [7. 与图查询结合](#7-与图查询结合)

---

## 1. 语法设计

### 1.1 创建向量索引

```sql
-- 基本语法
CREATE VECTOR INDEX [IF NOT EXISTS] <index_name>
ON <tag_name>(<field_name>)
WITH (
    vector_size = <dimension>,
    distance = 'cosine' | 'euclidean' | 'dot',
    engine = 'qdrant',
    hnsw_m = 16,
    hnsw_ef_construct = 100
);

-- 示例
CREATE VECTOR INDEX IF NOT EXISTS idx_doc_embedding
ON Document(embedding)
WITH (
    vector_size = 768,
    distance = 'cosine',
    engine = 'qdrant'
);
```

### 1.2 删除向量索引

```sql
-- 基本语法
DROP VECTOR INDEX [IF EXISTS] <index_name>;

-- 示例
DROP VECTOR INDEX IF EXISTS idx_doc_embedding;
```

### 1.3 向量搜索

```sql
-- 基本搜索
SEARCH VECTOR <index_name>
WITH vector = [<float_array>]
LIMIT <n>
RETURN <fields>, score;

-- 文本搜索（需要嵌入服务）
SEARCH VECTOR <index_name>
WITH text = '<query_text>'
LIMIT <n>
RETURN <fields>, score;

-- 带过滤的搜索
SEARCH VECTOR <index_name>
WITH vector = [<float_array>]
WHERE <filter_conditions>
LIMIT <n>
RETURN <fields>, score;

-- 带分数阈值
SEARCH VECTOR <index_name>
WITH vector = [<float_array>]
WITH threshold = 0.8
LIMIT <n>
RETURN <fields>, score;
```

### 1.4 与MATCH结合

```sql
-- 向量搜索 + 图遍历
MATCH (d:Document)
WHERE d.embedding SIMILAR TO [<vector>] WITH threshold = 0.8
RETURN d
LIMIT 10;

-- 向量搜索 + 关系遍历
MATCH (d:Document)-[:REFERENCES]->(r:Document)
WHERE d.embedding SIMILAR TO [<vector>] WITH threshold = 0.8
RETURN d, r
LIMIT 10;
```

---

## 2. AST扩展

### 2.1 语句定义

```rust
// src/query/ast/vector.rs

use crate::core::types::span::Span;
use crate::core::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorStatement {
    CreateVectorIndex(CreateVectorIndexStatement),
    DropVectorIndex(DropVectorIndexStatement),
    SearchVector(SearchVectorStatement),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVectorIndexStatement {
    pub span: Span,
    pub if_not_exists: bool,
    pub index_name: String,
    pub tag_name: String,
    pub field_name: String,
    pub config: VectorIndexConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndexConfig {
    pub vector_size: usize,
    pub distance: DistanceMetric,
    pub engine: Option<String>,
    pub hnsw_m: Option<usize>,
    pub hnsw_ef_construct: Option<usize>,
    pub quantization: Option<QuantizationType>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    Dot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QuantizationType {
    Scalar,
    Product,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropVectorIndexStatement {
    pub span: Span,
    pub if_exists: bool,
    pub index_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchVectorStatement {
    pub span: Span,
    pub index_name: String,
    pub query: VectorQueryExpr,
    pub threshold: Option<f32>,
    pub where_clause: Option<Expression>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub yield_clause: Option<YieldClause>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorQueryExpr {
    Vector {
        span: Span,
        values: Vec<f32>,
    },
    Text {
        span: Span,
        text: String,
    },
    Parameter {
        span: Span,
        name: String,
    },
    Field {
        span: Span,
        field_name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSimilarityExpr {
    pub span: Span,
    pub field_name: String,
    pub query: VectorQueryExpr,
    pub threshold: Option<f32>,
}
```

### 2.2 表达式扩展

```rust
// 在 src/core/types/expr/expression.rs 中添加

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    // 现有表达式类型...
    
    // 向量相似度表达式
    VectorSimilarity {
        span: Span,
        field: Box<Expression>,
        query: VectorQueryExpr,
        threshold: Option<f32>,
    },
}
```

---

## 3. 解析器扩展

### 3.1 向量索引解析

```rust
// src/query/parser/parsing/vector_parser.rs

use crate::query::ast::vector::*;
use crate::query::parser::{Parser, Token};

impl Parser {
    pub fn parse_create_vector_index(&mut self) -> Result<CreateVectorIndexStatement, ParseError> {
        let span = self.current_span();
        
        // 解析 IF NOT EXISTS
        let if_not_exists = self.parse_if_not_exists()?;
        
        // 解析索引名称
        let index_name = self.parse_identifier()?;
        
        // 解析 ON tag(field)
        self.expect_keyword(Keyword::ON)?;
        let tag_name = self.parse_identifier()?;
        self.expect_token(Token::LParen)?;
        let field_name = self.parse_identifier()?;
        self.expect_token(Token::RParen)?;
        
        // 解析 WITH (...)
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
    
    fn parse_vector_index_config(&mut self) -> Result<VectorIndexConfig, ParseError> {
        let mut config = VectorIndexConfig {
            vector_size: 768,
            distance: DistanceMetric::Cosine,
            engine: None,
            hnsw_m: None,
            hnsw_ef_construct: None,
            quantization: None,
        };
        
        self.expect_keyword(Keyword::WITH)?;
        self.expect_token(Token::LParen)?;
        
        while !self.check_token(Token::RParen) {
            let key = self.parse_identifier()?;
            self.expect_token(Token::Eq)?;
            
            match key.to_lowercase().as_str() {
                "vector_size" => {
                    config.vector_size = self.parse_integer()? as usize;
                }
                "distance" => {
                    let dist = self.parse_identifier()?;
                    config.distance = match dist.to_lowercase().as_str() {
                        "cosine" => DistanceMetric::Cosine,
                        "euclidean" => DistanceMetric::Euclidean,
                        "dot" => DistanceMetric::Dot,
                        _ => return Err(ParseError::InvalidDistance(dist)),
                    };
                }
                "engine" => {
                    config.engine = Some(self.parse_string_literal()?);
                }
                "hnsw_m" => {
                    config.hnsw_m = Some(self.parse_integer()? as usize);
                }
                "hnsw_ef_construct" => {
                    config.hnsw_ef_construct = Some(self.parse_integer()? as usize);
                }
                "quantization" => {
                    let q = self.parse_identifier()?;
                    config.quantization = Some(match q.to_lowercase().as_str() {
                        "scalar" => QuantizationType::Scalar,
                        "product" => QuantizationType::Product,
                        _ => return Err(ParseError::InvalidQuantization(q)),
                    });
                }
                _ => return Err(ParseError::UnknownConfig(key)),
            }
            
            if !self.check_token(Token::RParen) {
                self.expect_token(Token::Comma)?;
            }
        }
        
        self.expect_token(Token::RParen)?;
        Ok(config)
    }
}
```

### 3.2 向量搜索解析

```rust
impl Parser {
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
        
        // 解析 RETURN
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
    
    fn parse_vector_query(&mut self) -> Result<VectorQueryExpr, ParseError> {
        self.expect_keyword(Keyword::WITH)?;
        
        let span = self.current_span();
        let keyword = self.parse_identifier()?;
        
        self.expect_token(Token::Eq)?;
        
        match keyword.to_lowercase().as_str() {
            "vector" => {
                let values = self.parse_float_array()?;
                Ok(VectorQueryExpr::Vector { span, values })
            }
            "text" => {
                let text = self.parse_string_literal()?;
                Ok(VectorQueryExpr::Text { span, text })
            }
            "param" => {
                let name = self.parse_parameter_name()?;
                Ok(VectorQueryExpr::Parameter { span, name })
            }
            _ => Err(ParseError::InvalidVectorQuery(keyword)),
        }
    }
    
    fn parse_float_array(&mut self) -> Result<Vec<f32>, ParseError> {
        self.expect_token(Token::LBracket)?;
        
        let mut values = Vec::new();
        while !self.check_token(Token::RBracket) {
            let value = self.parse_float()?;
            values.push(value);
            
            if !self.check_token(Token::RBracket) {
                self.expect_token(Token::Comma)?;
            }
        }
        
        self.expect_token(Token::RBracket)?;
        Ok(values)
    }
    
    fn parse_optional_threshold(&mut self) -> Result<Option<f32>, ParseError> {
        if self.check_keyword(Keyword::WITH) {
            self.advance();
            self.expect_identifier("threshold")?;
            self.expect_token(Token::Eq)?;
            Ok(Some(self.parse_float()?))
        } else {
            Ok(None)
        }
    }
}
```

---

## 4. 验证器扩展

### 4.1 向量索引验证

```rust
// src/query/validator/vector_validator.rs

use crate::query::ast::vector::*;
use crate::query::validator::{ValidationContext, ValidationError};

pub struct VectorValidator;

impl VectorValidator {
    pub fn validate_create_vector_index(
        ctx: &mut ValidationContext,
        stmt: &CreateVectorIndexStatement,
    ) -> Result<(), ValidationError> {
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
    
    pub fn validate_search_vector(
        ctx: &mut ValidationContext,
        stmt: &SearchVectorStatement,
    ) -> Result<(), ValidationError> {
        // 验证索引是否存在
        if !ctx.vector_index_exists(&stmt.index_name)? {
            return Err(ValidationError::IndexNotFound(stmt.index_name.clone()));
        }
        
        // 获取索引元数据
        let metadata = ctx.get_vector_index_metadata(&stmt.index_name)?;
        
        // 验证查询向量维度
        match &stmt.query {
            VectorQueryExpr::Vector { values, .. } => {
                if values.len() != metadata.vector_size {
                    return Err(ValidationError::VectorSizeMismatch {
                        expected: metadata.vector_size,
                        actual: values.len(),
                    });
                }
            }
            _ => {}
        }
        
        // 验证阈值范围
        if let Some(threshold) = stmt.threshold {
            if threshold < 0.0 || threshold > 1.0 {
                return Err(ValidationError::InvalidThreshold(threshold));
            }
        }
        
        // 验证 WHERE 子句
        if let Some(where_clause) = &stmt.where_clause {
            ctx.validate_expression(where_clause)?;
        }
        
        Ok(())
    }
}
```

---

## 5. 计划节点

### 5.1 节点定义

```rust
// src/query/planning/plan/core/nodes/data_access/vector_search.rs

use crate::core::types::span::Span;
use crate::query::ast::vector::{VectorQueryExpr, DistanceMetric};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchNode {
    pub id: NodeId,
    pub span: Span,
    pub index_name: String,
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub query: VectorQueryExpr,
    pub threshold: Option<f32>,
    pub filter: Option<Expression>,
    pub limit: usize,
    pub offset: usize,
    pub output_fields: Vec<OutputField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputField {
    pub name: String,
    pub alias: Option<String>,
    pub expr: Expression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVectorIndexNode {
    pub id: NodeId,
    pub span: Span,
    pub space_id: u64,
    pub index_name: String,
    pub tag_name: String,
    pub field_name: String,
    pub config: VectorIndexConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropVectorIndexNode {
    pub id: NodeId,
    pub span: Span,
    pub space_id: u64,
    pub index_name: String,
    pub if_exists: bool,
}
```

### 5.2 计划构建

```rust
// src/query/planning/plan/builder/vector_plan_builder.rs

use crate::query::ast::vector::*;
use crate::query::planning::plan::core::nodes::data_access::vector_search::*;

pub struct VectorPlanBuilder;

impl VectorPlanBuilder {
    pub fn build_create_vector_index(
        ctx: &mut PlanContext,
        stmt: CreateVectorIndexStatement,
    ) -> Result<Plan, PlanError> {
        let space_id = ctx.current_space_id()?;
        
        let node = CreateVectorIndexNode {
            id: ctx.next_node_id(),
            span: stmt.span,
            space_id,
            index_name: stmt.index_name,
            tag_name: stmt.tag_name,
            field_name: stmt.field_name,
            config: stmt.config,
        };
        
        Ok(Plan::from_node(node))
    }
    
    pub fn build_search_vector(
        ctx: &mut PlanContext,
        stmt: SearchVectorStatement,
    ) -> Result<Plan, PlanError> {
        let space_id = ctx.current_space_id()?;
        
        // 解析索引名称，获取 tag_name 和 field_name
        let (tag_name, field_name) = ctx.parse_vector_index_name(&stmt.index_name)?;
        
        // 构建输出字段
        let output_fields = Self::build_output_fields(&stmt.yield_clause)?;
        
        let node = VectorSearchNode {
            id: ctx.next_node_id(),
            span: stmt.span,
            index_name: stmt.index_name,
            space_id,
            tag_name,
            field_name,
            query: stmt.query,
            threshold: stmt.threshold,
            filter: stmt.where_clause,
            limit: stmt.limit.unwrap_or(10),
            offset: stmt.offset.unwrap_or(0),
            output_fields,
        };
        
        Ok(Plan::from_node(node))
    }
}
```

---

## 6. 执行器实现

### 6.1 向量搜索执行器

```rust
// src/query/executor/data_access/vector_search.rs

use crate::coordinator::VectorCoordinator;
use crate::query::executor::{Executor, ExecutionContext, ExecutionResult, ExecutorError};
use crate::query::planning::plan::core::nodes::data_access::vector_search::VectorSearchNode;
use async_trait::async_trait;

pub struct VectorSearchExecutor {
    node: VectorSearchNode,
}

impl VectorSearchExecutor {
    pub fn new(node: VectorSearchNode) -> Self {
        Self { node }
    }
}

#[async_trait]
impl Executor for VectorSearchExecutor {
    async fn execute(&self, ctx: &mut ExecutionContext) -> Result<ExecutionResult, ExecutorError> {
        let coordinator = ctx.vector_coordinator()
            .ok_or(ExecutorError::VectorCoordinatorNotAvailable)?;
        
        // 1. 获取查询向量
        let query_vector = self.resolve_query_vector(ctx, &self.node.query).await?;
        
        // 2. 构建过滤器
        let filter = self.node.filter.as_ref()
            .map(|f| self.build_vector_filter(ctx, f))
            .transpose()?;
        
        // 3. 执行向量搜索
        let search_options = SearchOptions {
            with_payload: true,
            with_vectors: false,
            offset: Some(self.node.offset),
            score_threshold: self.node.threshold,
            filter,
        };
        
        let results = coordinator.search(
            self.node.space_id,
            &self.node.tag_name,
            &self.node.field_name,
            query_vector,
            self.node.limit,
            search_options,
        ).await.map_err(|e| ExecutorError::VectorSearchError(e.to_string()))?;
        
        // 4. 根据 point_ids 获取完整顶点数据
        let vertex_ids: Vec<Value> = results.iter()
            .map(|r| self.parse_vertex_id(&r.id))
            .collect::<Result<Vec<_>, _>>()?;
        
        let vertices = if !vertex_ids.is_empty() {
            ctx.storage()
                .get_vertices(self.node.space_id, &vertex_ids)
                .await
                .map_err(|e| ExecutorError::StorageError(e.to_string()))?
        } else {
            vec![]
        };
        
        // 5. 构建结果行
        let rows = self.build_result_rows(&results, &vertices)?;
        
        Ok(ExecutionResult::Rows(rows))
    }
}

impl VectorSearchExecutor {
    async fn resolve_query_vector(
        &self,
        ctx: &ExecutionContext,
        query: &VectorQueryExpr,
    ) -> Result<Vec<f32>, ExecutorError> {
        match query {
            VectorQueryExpr::Vector { values, .. } => Ok(values.clone()),
            VectorQueryExpr::Text { text, .. } => {
                let coordinator = ctx.vector_coordinator()
                    .ok_or(ExecutorError::VectorCoordinatorNotAvailable)?;
                
                coordinator.embed_text(text)
                    .await
                    .map_err(|e| ExecutorError::EmbeddingError(e.to_string()))
            }
            VectorQueryExpr::Parameter { name, .. } => {
                ctx.get_parameter(name)?
                    .as_vector()
                    .ok_or_else(|| ExecutorError::InvalidParameterType(name.clone()))
            }
            VectorQueryExpr::Field { field_name, .. } => {
                ctx.get_field_value(field_name)?
                    .as_vector()
                    .ok_or_else(|| ExecutorError::InvalidFieldType(field_name.clone()))
            }
        }
    }
    
    fn build_result_rows(
        &self,
        results: &[VectorSearchResult],
        vertices: &[Vertex],
    ) -> Result<Vec<Row>, ExecutorError> {
        let mut rows = Vec::new();
        
        // 创建 vertex_id -> vertex 的映射
        let vertex_map: HashMap<_, _> = vertices.iter()
            .map(|v| (v.vid.to_string(), v))
            .collect();
        
        for result in results {
            let vertex = vertex_map.get(&result.id);
            
            let mut row = Row::new();
            
            // 添加输出字段
            for field in &self.node.output_fields {
                let value = match field.name.as_str() {
                    "score" => Value::Double(result.score as f64),
                    "id" | "_id" => Value::String(result.id.clone()),
                    name => {
                        if let Some(v) = vertex {
                            v.get_property(name)
                                .cloned()
                                .unwrap_or(Value::Null)
                        } else if let Some(payload_val) = result.payload.get(name) {
                            payload_val.clone()
                        } else {
                            Value::Null
                        }
                    }
                };
                
                let col_name = field.alias.as_ref().unwrap_or(&field.name);
                row.set(col_name.clone(), value);
            }
            
            rows.push(row);
        }
        
        Ok(rows)
    }
}
```

### 6.2 创建向量索引执行器

```rust
// src/query/executor/admin/index/vector_index.rs

pub struct CreateVectorIndexExecutor {
    node: CreateVectorIndexNode,
}

#[async_trait]
impl Executor for CreateVectorIndexExecutor {
    async fn execute(&self, ctx: &mut ExecutionContext) -> Result<ExecutionResult, ExecutorError> {
        let coordinator = ctx.vector_coordinator()
            .ok_or(ExecutorError::VectorCoordinatorNotAvailable)?;
        
        // 检查索引是否已存在
        if coordinator.index_exists(
            self.node.space_id,
            &self.node.tag_name,
            &self.node.field_name,
        ).await? {
            if !self.node.if_not_exists {
                return Err(ExecutorError::IndexAlreadyExists(self.node.index_name.clone()));
            }
            return Ok(ExecutionResult::empty());
        }
        
        // 创建向量索引
        coordinator.create_vector_index(
            self.node.space_id,
            &self.node.tag_name,
            &self.node.field_name,
            self.node.config.vector_size,
            self.node.config.distance,
            self.node.config.hnsw_m,
            self.node.config.hnsw_ef_construct,
        ).await.map_err(|e| ExecutorError::VectorIndexError(e.to_string()))?;
        
        // 持久化索引元数据
        ctx.metadata_manager()
            .save_vector_index_metadata(&VectorIndexMetadata {
                index_name: self.node.index_name.clone(),
                space_id: self.node.space_id,
                tag_name: self.node.tag_name.clone(),
                field_name: self.node.field_name.clone(),
                vector_size: self.node.config.vector_size,
                distance: self.node.config.distance,
                created_at: chrono::Utc::now(),
            }).await?;
        
        Ok(ExecutionResult::empty())
    }
}
```

---

## 7. 与图查询结合

### 7.1 向量相似度谓词

```rust
// 在 MATCH 语句中支持向量相似度条件

// SQL:
// MATCH (d:Document)
// WHERE d.embedding SIMILAR TO [0.1, 0.2, ...] WITH threshold = 0.8
// RETURN d

// 解析器扩展
impl Parser {
    fn parse_similarity_expression(&mut self) -> Result<Expression, ParseError> {
        let span = self.current_span();
        
        let field = self.parse_expression()?;
        
        self.expect_keyword(Keyword::SIMILAR)?;
        self.expect_keyword(Keyword::TO)?;
        
        let query = self.parse_vector_query()?;
        
        let threshold = self.parse_optional_threshold()?;
        
        Ok(Expression::VectorSimilarity {
            span,
            field: Box::new(field),
            query,
            threshold,
        })
    }
}
```

### 7.2 向量过滤执行器

```rust
// src/query/executor/data_access/vector_filter.rs

pub struct VectorFilterExecutor {
    tag_name: String,
    field_name: String,
    query: VectorQueryExpr,
    threshold: Option<f32>,
}

impl VectorFilterExecutor {
    pub async fn filter_vertices(
        &self,
        ctx: &ExecutionContext,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Vertex>, ExecutorError> {
        let coordinator = ctx.vector_coordinator()
            .ok_or(ExecutorError::VectorCoordinatorNotAvailable)?;
        
        // 获取查询向量
        let query_vector = self.resolve_query_vector(ctx, &self.query).await?;
        
        // 获取每个顶点的向量并计算相似度
        let mut filtered = Vec::new();
        
        for vertex in vertices {
            if let Some(vector) = vertex.get_property(&self.field_name)
                .and_then(|v| v.as_vector()) 
            {
                let similarity = self.compute_similarity(&query_vector, &vector)?;
                
                if let Some(threshold) = self.threshold {
                    if similarity >= threshold {
                        filtered.push(vertex);
                    }
                } else {
                    filtered.push(vertex);
                }
            }
        }
        
        Ok(filtered)
    }
    
    fn compute_similarity(&self, a: &[f32], b: &[f32]) -> Result<f32, ExecutorError> {
        if a.len() != b.len() {
            return Err(ExecutorError::VectorSizeMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }
        
        let similarity = match self.distance_metric {
            DistanceMetric::Cosine => {
                let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
                dot / (norm_a * norm_b)
            }
            DistanceMetric::Euclidean => {
                let dist: f32 = a.iter().zip(b.iter())
                    .map(|(x, y)| (x - y).powi(2))
                    .sum::<f32>()
                    .sqrt();
                1.0 / (1.0 + dist)
            }
            DistanceMetric::Dot => {
                a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
            }
        };
        
        Ok(similarity)
    }
}
```

### 7.3 混合检索示例

```sql
-- 混合检索：全文检索 + 向量检索
MATCH (d:Document)
WHERE d.content MATCH 'graph database'
  AND d.embedding SIMILAR TO [0.1, 0.2, ...] WITH threshold = 0.8
RETURN d
ORDER BY d.score_fulltext * 0.3 + d.score_vector * 0.7 DESC
LIMIT 10;

-- 向量搜索后图遍历
SEARCH VECTOR idx_doc_embedding
WITH vector = [0.1, 0.2, ...]
LIMIT 10
RETURN id, content, score
|> MATCH (d:Document {id: id})-[:REFERENCES]->(r:Document)
   RETURN d, r;
```

---

## 附录: 执行器工厂注册

```rust
// src/query/executor/factory/executor_factory.rs

impl ExecutorFactory {
    pub fn create_executor(&self, node: &PlanNode) -> Result<Box<dyn Executor>, ExecutorError> {
        match node {
            // 现有节点类型...
            
            PlanNode::VectorSearch(n) => {
                Ok(Box::new(VectorSearchExecutor::new(n.clone())))
            }
            PlanNode::CreateVectorIndex(n) => {
                Ok(Box::new(CreateVectorIndexExecutor::new(n.clone())))
            }
            PlanNode::DropVectorIndex(n) => {
                Ok(Box::new(DropVectorIndexExecutor::new(n.clone())))
            }
            
            // ...
        }
    }
}
```
