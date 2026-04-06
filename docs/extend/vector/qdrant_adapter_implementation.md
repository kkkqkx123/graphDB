# Qdrant适配器实现细节

> 分析日期: 2026-04-06
> 依赖: qdrant-client v1.7+

---

## 目录

- [1. 客户端初始化](#1-客户端初始化)
- [2. 集合管理](#2-集合管理)
- [3. 向量操作](#3-向量操作)
- [4. 搜索功能](#4-搜索功能)
- [5. 过滤器转换](#5-过滤器转换)
- [6. 错误处理](#6-错误处理)
- [7. 完整实现示例](#7-完整实现示例)

---

## 1. 客户端初始化

### 1.1 配置选项

```rust
use qdrant_client::Qdrant;
use qdrant_client::config::CompressionEncoding;
use std::time::Duration;

pub struct QdrantConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub compression: Option<CompressionEncoding>,
    pub keep_alive: bool,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6334".to_string(),
            api_key: None,
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            compression: None,
            keep_alive: true,
        }
    }
}

impl QdrantConfig {
    pub fn from_url(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }
    
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    pub fn with_compression(mut self, compression: CompressionEncoding) -> Self {
        self.compression = Some(compression);
        self
    }
    
    pub fn build(&self) -> Result<Qdrant, VectorError> {
        let mut builder = Qdrant::from_url(&self.url)
            .timeout(self.timeout)
            .connect_timeout(self.connect_timeout);
        
        if let Some(ref api_key) = self.api_key {
            builder = builder.api_key(Some(api_key.clone()));
        }
        
        if let Some(compression) = self.compression {
            builder = builder.compression(Some(compression));
        }
        
        if self.keep_alive {
            builder = builder.keep_alive_while_idle();
        }
        
        builder.build().map_err(|e| VectorError::Connection(e.to_string()))
    }
}
```

### 1.2 连接验证

```rust
impl QdrantAdapter {
    pub async fn health_check(&self) -> Result<HealthStatus, VectorError> {
        let health = self.client.health_check().await
            .map_err(|e| VectorError::Connection(e.to_string()))?;
        
        Ok(HealthStatus {
            version: health.version,
            title: health.title,
            is_healthy: true,
        })
    }
}
```

---

## 2. 集合管理

### 2.1 创建集合

```rust
use qdrant_client::qdrant::{
    CreateCollectionBuilder,
    Distance,
    VectorParamsBuilder,
    ScalarQuantizationBuilder,
    HnswConfigDiffBuilder,
    OptimizersConfigDiffBuilder,
};

pub struct CollectionConfig {
    pub vector_size: usize,
    pub distance: DistanceMetric,
    pub hnsw: Option<HnswConfig>,
    pub quantization: Option<QuantizationConfig>,
    pub optimizers: Option<OptimizersConfig>,
}

pub struct HnswConfig {
    pub m: usize,
    pub ef_construct: usize,
    pub full_scan_threshold: Option<usize>,
}

pub enum QuantizationConfig {
    Scalar {
        quantile: Option<f32>,
        always_ram: Option<bool>,
    },
    Product {
        compression: ProductQuantizationCompression,
        always_ram: Option<bool>,
    },
}

impl QdrantAdapter {
    pub async fn create_collection(
        &self,
        collection_name: &str,
        config: CollectionConfig,
    ) -> Result<(), VectorError> {
        let distance = match config.distance {
            DistanceMetric::Cosine => Distance::Cosine,
            DistanceMetric::Euclidean => Distance::Euclid,
            DistanceMetric::Dot => Distance::Dot,
        };
        
        let mut builder = CreateCollectionBuilder::new(collection_name)
            .vectors_config(VectorParamsBuilder::new(
                config.vector_size as u64,
                distance,
            ));
        
        if let Some(hnsw) = config.hnsw {
            let mut hnsw_builder = HnswConfigDiffBuilder::default()
                .m(hnsw.m as u64)
                .ef_construct(hnsw.ef_construct as u64);
            
            if let Some(threshold) = hnsw.full_scan_threshold {
                hnsw_builder = hnsw_builder.full_scan_threshold(threshold as u64);
            }
            
            builder = builder.hnsw_config(hnsw_builder);
        }
        
        if let Some(quant) = config.quantization {
            match quant {
                QuantizationConfig::Scalar { quantile, always_ram } => {
                    let mut sq_builder = ScalarQuantizationBuilder::default();
                    if let Some(q) = quantile {
                        sq_builder = sq_builder.quantile(q);
                    }
                    if let Some(ram) = always_ram {
                        sq_builder = sq_builder.always_ram(ram);
                    }
                    builder = builder.quantization_config(sq_builder);
                }
                QuantizationConfig::Product { compression, always_ram } => {
                    let pq_builder = ProductQuantizationBuilder::default()
                        .compression(compression.into());
                    builder = builder.quantization_config(pq_builder);
                }
            }
        }
        
        if let Some(opts) = config.optimizers {
            builder = builder.optimizers_config(
                OptimizersConfigDiffBuilder::default()
                    .indexing_threshold(opts.indexing_threshold as u64)
            );
        }
        
        self.client.create_collection(builder).await
            .map_err(|e| VectorError::CollectionError(e.to_string()))?;
        
        Ok(())
    }
}
```

### 2.2 集合信息查询

```rust
impl QdrantAdapter {
    pub async fn collection_exists(&self, collection_name: &str) -> Result<bool, VectorError> {
        self.client.collection_exists(collection_name).await
            .map_err(|e| VectorError::QueryError(e.to_string()))
    }
    
    pub async fn get_collection_info(
        &self,
        collection_name: &str,
    ) -> Result<CollectionInfo, VectorError> {
        let info = self.client.collection_info(collection_name).await
            .map_err(|e| VectorError::QueryError(e.to_string()))?;
        
        Ok(CollectionInfo {
            vector_count: info.result.unwrap().points_count.unwrap_or(0),
            indexed_vector_count: info.result.unwrap().indexed_vectors_count.unwrap_or(0),
            segments_count: info.result.unwrap().segments_count.unwrap_or(0),
            status: info.result.unwrap().status,
        })
    }
    
    pub async fn delete_collection(&self, collection_name: &str) -> Result<(), VectorError> {
        self.client.delete_collection(collection_name).await
            .map_err(|e| VectorError::CollectionError(e.to_string()))?;
        Ok(())
    }
}
```

---

## 3. 向量操作

### 3.1 单点插入/更新

```rust
use qdrant_client::{Payload, PointStruct};
use qdrant_client::qdrant::UpsertPointsBuilder;

impl QdrantAdapter {
    pub async fn upsert(
        &self,
        collection_name: &str,
        point_id: &str,
        vector: Vec<f32>,
        payload: HashMap<String, Value>,
    ) -> Result<(), VectorError> {
        let qdrant_payload = self.convert_payload(payload)?;
        
        let point = PointStruct::new(
            point_id,
            vector,
            qdrant_payload,
        );
        
        self.client
            .upsert_points(
                UpsertPointsBuilder::new(collection_name, vec![point])
                    .wait(true)
            )
            .await
            .map_err(|e| VectorError::UpsertError(e.to_string()))?;
        
        Ok(())
    }
    
    fn convert_payload(&self, payload: HashMap<String, Value>) -> Result<Payload, VectorError> {
        let json_value = serde_json::to_value(payload)
            .map_err(|e| VectorError::Serialization(e.to_string()))?;
        
        Payload::try_from(json_value)
            .map_err(|e| VectorError::Serialization(e.to_string()))
    }
}
```

### 3.2 批量插入/更新

```rust
impl QdrantAdapter {
    pub async fn upsert_batch(
        &self,
        collection_name: &str,
        points: Vec<VectorPoint>,
        chunk_size: Option<usize>,
    ) -> Result<UpsertResult, VectorError> {
        let qdrant_points: Vec<PointStruct> = points
            .into_iter()
            .map(|p| -> Result<PointStruct, VectorError> {
                let payload = self.convert_payload(p.payload)?;
                Ok(PointStruct::new(p.id, p.vector, payload))
            })
            .collect::<Result<Vec<_>, _>>()?;
        
        let chunk_size = chunk_size.unwrap_or(100);
        
        if qdrant_points.len() > chunk_size {
            self.client
                .upsert_points_chunked(
                    UpsertPointsBuilder::new(collection_name, qdrant_points)
                        .wait(true),
                    chunk_size,
                )
                .await
                .map_err(|e| VectorError::UpsertError(e.to_string()))?;
        } else {
            self.client
                .upsert_points(
                    UpsertPointsBuilder::new(collection_name, qdrant_points)
                        .wait(true)
                )
                .await
                .map_err(|e| VectorError::UpsertError(e.to_string()))?;
        }
        
        Ok(UpsertResult {
            count: qdrant_points.len(),
        })
    }
}
```

### 3.3 删除操作

```rust
use qdrant_client::qdrant::{
    DeletePointsBuilder,
    Filter,
    PointsIdsList,
    PointId,
};

impl QdrantAdapter {
    pub async fn delete(
        &self,
        collection_name: &str,
        point_id: &str,
    ) -> Result<(), VectorError> {
        self.client
            .delete_points(
                DeletePointsBuilder::new(collection_name)
                    .points(vec![PointId::from(point_id)])
                    .wait(true)
            )
            .await
            .map_err(|e| VectorError::DeleteError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn delete_batch(
        &self,
        collection_name: &str,
        point_ids: Vec<&str>,
    ) -> Result<(), VectorError> {
        let ids: Vec<PointId> = point_ids
            .into_iter()
            .map(|id| PointId::from(id.to_string()))
            .collect();
        
        self.client
            .delete_points(
                DeletePointsBuilder::new(collection_name)
                    .points(ids)
                    .wait(true)
            )
            .await
            .map_err(|e| VectorError::DeleteError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn delete_by_filter(
        &self,
        collection_name: &str,
        filter: VectorFilter,
    ) -> Result<u64, VectorError> {
        let qdrant_filter = self.convert_filter(filter)?;
        
        let result = self.client
            .delete_points(
                DeletePointsBuilder::new(collection_name)
                    .filter(qdrant_filter)
                    .wait(true)
            )
            .await
            .map_err(|e| VectorError::DeleteError(e.to_string()))?;
        
        Ok(result.result.unwrap().deleted_count.unwrap_or(0))
    }
}
```

---

## 4. 搜索功能

### 4.1 基本搜索

```rust
use qdrant_client::qdrant::{
    SearchPointsBuilder,
    SearchParamsBuilder,
    WithPayloadSelector,
    WithVectorsSelector,
};

impl QdrantAdapter {
    pub async fn search(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        options: SearchOptions,
    ) -> Result<Vec<VectorSearchResult>, VectorError> {
        let mut builder = SearchPointsBuilder::new(
            collection_name,
            query_vector,
            limit as u64,
        );
        
        if options.with_payload {
            builder = builder.with_payload(true);
        }
        
        if options.with_vectors {
            builder = builder.with_vectors(true);
        }
        
        if let Some(offset) = options.offset {
            builder = builder.offset(offset as u64);
        }
        
        if let Some(threshold) = options.score_threshold {
            builder = builder.score_threshold(threshold);
        }
        
        if let Some(params) = options.search_params {
            let mut params_builder = SearchParamsBuilder::default();
            if let Some(ef) = params.hnsw_ef {
                params_builder = params_builder.hnsw_ef(ef as u64);
            }
            if let Some(exact) = params.exact {
                params_builder = params_builder.exact(exact);
            }
            builder = builder.params(params_builder);
        }
        
        let result = self.client.search_points(builder).await
            .map_err(|e| VectorError::SearchError(e.to_string()))?;
        
        let results: Vec<VectorSearchResult> = result.result
            .into_iter()
            .map(|r| VectorSearchResult {
                id: r.id.unwrap().to_string(),
                score: r.score,
                payload: r.payload.into_iter()
                    .map(|(k, v)| (k, self.convert_payload_value(v)))
                    .collect(),
                vector: r.vectors.map(|v| match v {
                    qdrant_client::qdrant::Vectors::Vector(v) => v.data,
                    qdrant_client::qdrant::Vectors::Vectors(vs) => {
                        vs.vectors.into_iter().next()
                            .map(|v| v.data)
                            .unwrap_or_default()
                    }
                }),
            })
            .collect();
        
        Ok(results)
    }
}

pub struct SearchOptions {
    pub with_payload: bool,
    pub with_vectors: bool,
    pub offset: Option<usize>,
    pub score_threshold: Option<f32>,
    pub search_params: Option<SearchParams>,
    pub filter: Option<VectorFilter>,
}

pub struct SearchParams {
    pub hnsw_ef: Option<usize>,
    pub exact: Option<bool>,
}
```

### 4.2 带过滤的搜索

```rust
impl QdrantAdapter {
    pub async fn search_with_filter(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        filter: VectorFilter,
        options: SearchOptions,
    ) -> Result<Vec<VectorSearchResult>, VectorError> {
        let qdrant_filter = self.convert_filter(filter)?;
        
        let mut builder = SearchPointsBuilder::new(
            collection_name,
            query_vector,
            limit as u64,
        )
        .filter(qdrant_filter);
        
        if options.with_payload {
            builder = builder.with_payload(true);
        }
        
        if options.with_vectors {
            builder = builder.with_vectors(true);
        }
        
        let result = self.client.search_points(builder).await
            .map_err(|e| VectorError::SearchError(e.to_string()))?;
        
        // 转换结果...
    }
}
```

### 4.3 批量搜索

```rust
use qdrant_client::qdrant::SearchBatchPointsBuilder;

impl QdrantAdapter {
    pub async fn search_batch(
        &self,
        collection_name: &str,
        queries: Vec<SearchQuery>,
    ) -> Result<Vec<Vec<VectorSearchResult>>, VectorError> {
        let search_queries: Vec<_> = queries
            .into_iter()
            .map(|q| {
                let mut builder = SearchPointsBuilder::new(
                    collection_name,
                    q.vector,
                    q.limit as u64,
                );
                
                if let Some(filter) = q.filter {
                    builder = builder.filter(self.convert_filter(filter).unwrap());
                }
                
                builder
            })
            .collect();
        
        let result = self.client
            .search_batch_points(
                SearchBatchPointsBuilder::new(collection_name, search_queries)
            )
            .await
            .map_err(|e| VectorError::SearchError(e.to_string()))?;
        
        // 转换结果...
    }
}
```

---

## 5. 过滤器转换

### 5.1 过滤器定义

```rust
#[derive(Debug, Clone)]
pub enum VectorFilter {
    Must(Vec<VectorFilter>),
    Should(Vec<VectorFilter>),
    MustNot(Vec<VectorFilter>),
    Field(FieldCondition),
}

#[derive(Debug, Clone)]
pub struct FieldCondition {
    pub field: String,
    pub condition: Condition,
}

#[derive(Debug, Clone)]
pub enum Condition {
    Match(Value),
    MatchAny(Vec<Value>),
    Range(RangeCondition),
    IsEmpty,
    IsNull,
    HasId(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct RangeCondition {
    pub gt: Option<f64>,
    pub gte: Option<f64>,
    pub lt: Option<f64>,
    pub lte: Option<f64>,
}
```

### 5.2 过滤器转换实现

```rust
use qdrant_client::qdrant::{Condition, Filter, Range};

impl QdrantAdapter {
    fn convert_filter(&self, filter: VectorFilter) -> Result<Filter, VectorError> {
        match filter {
            VectorFilter::Must(filters) => {
                let conditions: Vec<Condition> = filters
                    .into_iter()
                    .map(|f| self.convert_condition(f))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Filter::must(conditions))
            }
            VectorFilter::Should(filters) => {
                let conditions: Vec<Condition> = filters
                    .into_iter()
                    .map(|f| self.convert_condition(f))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Filter::should(conditions))
            }
            VectorFilter::MustNot(filters) => {
                let conditions: Vec<Condition> = filters
                    .into_iter()
                    .map(|f| self.convert_condition(f))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Filter::must_not(conditions))
            }
            VectorFilter::Field(field_cond) => {
                Ok(Filter::must([self.convert_field_condition(field_cond)?]))
            }
        }
    }
    
    fn convert_condition(&self, filter: VectorFilter) -> Result<Condition, VectorError> {
        match filter {
            VectorFilter::Field(field_cond) => self.convert_field_condition(field_cond),
            _ => Err(VectorError::FilterError("Nested filters not supported as Condition".into())),
        }
    }
    
    fn convert_field_condition(&self, cond: FieldCondition) -> Result<Condition, VectorError> {
        match cond.condition {
            Condition::Match(value) => {
                Ok(Condition::matches(cond.field, self.value_to_string(value)))
            }
            Condition::MatchAny(values) => {
                let strings: Vec<String> = values
                    .into_iter()
                    .map(|v| self.value_to_string(v))
                    .collect();
                Ok(Condition::matches(cond.field, strings))
            }
            Condition::Range(range) => {
                Ok(Condition::range(
                    cond.field,
                    Range {
                        gt: range.gt,
                        gte: range.gte,
                        lt: range.lt,
                        lte: range.lte,
                    }
                ))
            }
            Condition::IsEmpty => {
                Ok(Condition::is_empty(cond.field))
            }
            Condition::IsNull => {
                Ok(Condition::is_null(cond.field))
            }
            Condition::HasId(ids) => {
                let point_ids: Vec<qdrant_client::qdrant::PointId> = ids
                    .into_iter()
                    .map(|id| qdrant_client::qdrant::PointId::from(id))
                    .collect();
                Ok(Condition::has_id(point_ids))
            }
        }
    }
    
    fn value_to_string(&self, value: Value) -> String {
        match value {
            Value::String(s) => s,
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Double(d) => d.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => format!("{:?}", value),
        }
    }
}
```

---

## 6. 错误处理

### 6.1 错误类型定义

```rust
#[derive(Debug, thiserror::Error)]
pub enum VectorError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Collection error: {0}")]
    CollectionError(String),
    
    #[error("Upsert error: {0}")]
    UpsertError(String),
    
    #[error("Delete error: {0}")]
    DeleteError(String),
    
    #[error("Search error: {0}")]
    SearchError(String),
    
    #[error("Query error: {0}")]
    QueryError(String),
    
    #[error("Filter error: {0}")]
    FilterError(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Invalid vector size: expected {expected}, got {actual}")]
    InvalidVectorSize { expected: usize, actual: usize },
    
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),
    
    #[error("Point not found: {0}")]
    PointNotFound(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
}

pub type VectorResult<T> = Result<T, VectorError>;
```

### 6.2 错误转换

```rust
impl From<qdrant_client::QdrantError> for VectorError {
    fn from(error: qdrant_client::QdrantError) -> Self {
        match error {
            qdrant_client::QdrantError::InvalidUrl(url) => {
                VectorError::Connection(format!("Invalid URL: {}", url))
            }
            qdrant_client::QdrantError::ResponseStatus { status, message } => {
                if status == 404 {
                    VectorError::CollectionNotFound(message)
                } else if status == 408 {
                    VectorError::Timeout(message)
                } else {
                    VectorError::QueryError(format!("Status {}: {}", status, message))
                }
            }
            _ => VectorError::QueryError(error.to_string()),
        }
    }
}
```

---

## 7. 完整实现示例

### 7.1 QdrantAdapter完整实现

```rust
use async_trait::async_trait;
use qdrant_client::Qdrant;
use std::sync::Arc;

pub struct QdrantAdapter {
    client: Arc<Qdrant>,
    config: QdrantConfig,
}

impl QdrantAdapter {
    pub fn new(config: QdrantConfig) -> Result<Self, VectorError> {
        let client = config.build()?;
        Ok(Self {
            client: Arc::new(client),
            config,
        })
    }
    
    pub fn client(&self) -> &Qdrant {
        &self.client
    }
}

#[async_trait]
impl VectorEngine for QdrantAdapter {
    fn name(&self) -> &str {
        "qdrant"
    }
    
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    
    async fn upsert(
        &self,
        collection_name: &str,
        point_id: &str,
        vector: Vec<f32>,
        payload: HashMap<String, Value>,
    ) -> Result<(), VectorError> {
        let qdrant_payload = self.convert_payload(payload)?;
        let point = PointStruct::new(point_id, vector, qdrant_payload);
        
        self.client
            .upsert_points(
                UpsertPointsBuilder::new(collection_name, vec![point])
                    .wait(true)
            )
            .await?;
        
        Ok(())
    }
    
    async fn upsert_batch(
        &self,
        collection_name: &str,
        points: Vec<VectorPoint>,
    ) -> Result<(), VectorError> {
        let qdrant_points: Vec<PointStruct> = points
            .into_iter()
            .map(|p| {
                let payload = self.convert_payload(p.payload)?;
                Ok(PointStruct::new(p.id, p.vector, payload))
            })
            .collect::<Result<Vec<_>, VectorError>>()?;
        
        self.client
            .upsert_points_chunked(
                UpsertPointsBuilder::new(collection_name, qdrant_points)
                    .wait(true),
                100,
            )
            .await?;
        
        Ok(())
    }
    
    async fn search(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        filter: Option<VectorFilter>,
    ) -> Result<Vec<VectorSearchResult>, VectorError> {
        let mut builder = SearchPointsBuilder::new(
            collection_name,
            query_vector,
            limit as u64,
        )
        .with_payload(true);
        
        if let Some(f) = filter {
            builder = builder.filter(self.convert_filter(f)?);
        }
        
        let result = self.client.search_points(builder).await?;
        
        Ok(result.result.into_iter().map(|r| {
            VectorSearchResult {
                id: r.id.unwrap().to_string(),
                score: r.score,
                payload: r.payload.into_iter()
                    .map(|(k, v)| (k, self.convert_payload_value(v)))
                    .collect(),
                vector: r.vectors.and_then(|v| match v {
                    qdrant_client::qdrant::Vectors::Vector(v) => Some(v.data),
                    _ => None,
                }),
            }
        }).collect())
    }
    
    async fn delete(
        &self,
        collection_name: &str,
        point_id: &str,
    ) -> Result<(), VectorError> {
        self.client
            .delete_points(
                DeletePointsBuilder::new(collection_name)
                    .points(vec![PointId::from(point_id)])
                    .wait(true)
            )
            .await?;
        Ok(())
    }
    
    async fn delete_batch(
        &self,
        collection_name: &str,
        point_ids: Vec<&str>,
    ) -> Result<(), VectorError> {
        let ids: Vec<PointId> = point_ids
            .into_iter()
            .map(|id| PointId::from(id.to_string()))
            .collect();
        
        self.client
            .delete_points(
                DeletePointsBuilder::new(collection_name)
                    .points(ids)
                    .wait(true)
            )
            .await?;
        Ok(())
    }
    
    async fn get(
        &self,
        collection_name: &str,
        point_id: &str,
    ) -> Result<Option<VectorPoint>, VectorError> {
        use qdrant_client::qdrant::GetPointBuilder;
        
        let result = self.client
            .get_point(
                GetPointBuilder::new(collection_name, PointId::from(point_id))
                    .with_payload(true)
                    .with_vectors(true)
            )
            .await;
        
        match result {
            Ok(point) => {
                let result = point.result.ok_or_else(|| {
                    VectorError::PointNotFound(point_id.to_string())
                })?;
                
                Ok(Some(VectorPoint {
                    id: result.id.unwrap().to_string(),
                    vector: result.vectors.and_then(|v| match v {
                        qdrant_client::qdrant::Vectors::Vector(v) => Some(v.data),
                        _ => None,
                    }).unwrap_or_default(),
                    payload: result.payload.into_iter()
                        .map(|(k, v)| (k, self.convert_payload_value(v)))
                        .collect(),
                }))
            }
            Err(e) => {
                if e.to_string().contains("Not found") {
                    Ok(None)
                } else {
                    Err(e.into())
                }
            }
        }
    }
    
    async fn count(&self, collection_name: &str) -> Result<u64, VectorError> {
        use qdrant_client::qdrant::CountPointsBuilder;
        
        let result = self.client
            .count_points(CountPointsBuilder::new(collection_name))
            .await?;
        
        Ok(result.result.unwrap().count)
    }
    
    async fn create_collection(
        &self,
        collection_name: &str,
        config: CollectionConfig,
    ) -> Result<(), VectorError> {
        let distance = match config.distance {
            DistanceMetric::Cosine => Distance::Cosine,
            DistanceMetric::Euclidean => Distance::Euclid,
            DistanceMetric::Dot => Distance::Dot,
        };
        
        self.client
            .create_collection(
                CreateCollectionBuilder::new(collection_name)
                    .vectors_config(VectorParamsBuilder::new(
                        config.vector_size as u64,
                        distance,
                    ))
            )
            .await?;
        
        Ok(())
    }
    
    async fn delete_collection(&self, collection_name: &str) -> Result<(), VectorError> {
        self.client.delete_collection(collection_name).await?;
        Ok(())
    }
    
    async fn collection_exists(&self, collection_name: &str) -> Result<bool, VectorError> {
        self.client.collection_exists(collection_name).await
            .map_err(Into::into)
    }
}
```

---

## 附录: 使用示例

```rust
use graphdb::vector::{QdrantAdapter, QdrantConfig, VectorEngine, CollectionConfig, DistanceMetric};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建适配器
    let config = QdrantConfig::from_url("http://localhost:6334");
    let adapter = QdrantAdapter::new(config)?;
    
    // 创建集合
    adapter.create_collection(
        "documents",
        CollectionConfig {
            vector_size: 768,
            distance: DistanceMetric::Cosine,
            hnsw: None,
            quantization: None,
            optimizers: None,
        }
    ).await?;
    
    // 插入向量
    let mut payload = HashMap::new();
    payload.insert("title".to_string(), Value::String("Test Document".to_string()));
    payload.insert("category".to_string(), Value::String("tech".to_string()));
    
    adapter.upsert(
        "documents",
        "doc1",
        vec![0.1; 768],
        payload,
    ).await?;
    
    // 搜索
    let results = adapter.search(
        "documents",
        vec![0.1; 768],
        10,
        None,
    ).await?;
    
    for result in results {
        println!("ID: {}, Score: {}", result.id, result.score);
    }
    
    Ok(())
}
```
