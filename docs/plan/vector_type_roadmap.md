# 向量类型后续任务执行方案

## 概述

本文档基于 `vector_type_design_recommendation.md` 和已完成的向量类型核心实现，制定后续任务的分阶段执行方案。

### 已完成的核心功能 ✅

- [x] VectorValue 类型定义（稠密/稀疏向量）
- [x] Value 枚举扩展
- [x] DataType 扩展
- [x] 序列化/反序列化支持
- [x] 类型系统集成（内存估算、比较、哈希）
- [x] 单元测试（9/9 通过）
- [x] 向后兼容支持

### 待完成任务

根据设计文档和实际使用需求，后续任务分为 5 个阶段，预计总工作量 **15-25 天**。

---

## 阶段 1：Parser 扩展与字面值语法（优先级：高）

**目标**：支持向量字面值语法，提升用户体验

**预计工作量**：3-5 天

### 1.1 SQL 语法设计

#### 方案 A：VECTOR 关键字（推荐）

```sql
-- 插入语句
INSERT VERTEX documents(title, embedding) 
VALUES "doc1":("标题", VECTOR[0.1, 0.2, 0.3]);

-- 查询语句
SEARCH VECTOR doc_embedding 
WITH VECTOR[0.1, 0.2, 0.3] 
LIMIT 10;
```

#### 方案 B：类型转换语法

```sql
-- 显式类型转换
INSERT VERTEX documents(title, embedding) 
VALUES "doc1":("标题", [0.1, 0.2, 0.3]::VECTOR);

-- 带维度的类型转换
INSERT VERTEX documents(title, embedding) 
VALUES "doc1":("标题", [0.1, 0.2, 0.3]::VECTOR_DENSE(1536));
```

#### 方案 C：保持向后兼容

```sql
-- 继续使用 List 语法，自动推断为 Vector 类型
INSERT VERTEX documents(title, embedding) 
VALUES "doc1":("标题", [0.1, 0.2, 0.3]);
-- 如果字段有向量索引，自动推断为 Vector 类型
```

**推荐方案**：方案 A + 方案 C（显式语法 + 自动推断）

### 1.2 实施步骤

#### Step 1.1: 词法分析器扩展（1 天）

**文件**：`src/query/parser/lexer.rs`

```rust
// 添加 VECTOR 关键字
pub enum Token {
    // ... 现有 token
    VectorKeyword,  // VECTOR
    LBracket,       // [
    RBracket,       // ]
    Comma,          // ,
    DoubleColon,    // ::
}
```

**测试**：
- 词法分析测试：`VECTOR[0.1, 0.2]` → 正确的 Token 序列
- 向后兼容测试：`[0.1, 0.2]` → 保持原有行为

#### Step 1.2: 语法分析器扩展（2 天）

**文件**：`src/query/parser/statement_parser.rs`

```rust
// 添加向量字面值解析
fn parse_vector_literal(&mut self) -> ParseResult<Expression> {
    // 解析 VECTOR[...] 语法
    // 解析 [...]::VECTOR 语法
    // 解析 [...] 自动推断语法
}
```

**AST 扩展**：
```rust
pub enum Expression {
    // ... 现有表达式
    VectorLiteral(Vec<f32>),
    VectorCast {
        expr: Box<Expression>,
        dimension: Option<usize>,
    },
}
```

#### Step 1.3: 语义分析器（1 天）

**文件**：`src/query/semantic_analyzer.rs`

```rust
impl SemanticAnalyzer {
    fn analyze_vector_literal(
        &self,
        vector: &[f32],
        expected_dim: Option<usize>
    ) -> Result<()> {
        // 维度验证
        // 类型推断
    }
}
```

### 1.3 测试计划

```rust
#[cfg(test)]
mod vector_parser_tests {
    #[test]
    fn test_vector_keyword_syntax() {
        let sql = r#"INSERT VERTEX documents(title, embedding) 
                     VALUES "doc1":("标题", VECTOR[0.1, 0.2, 0.3])"#;
        let result = parse(sql).unwrap();
        assert!(matches!(result, Statement::Insert(_)));
    }

    #[test]
    fn test_vector_cast_syntax() {
        let sql = r#"INSERT VERTEX documents(title, embedding) 
                     VALUES "doc1":("标题", [0.1, 0.2]::VECTOR)"#;
        let result = parse(sql).unwrap();
        // 验证类型转换
    }

    #[test]
    fn test_vector_auto_inference() {
        let sql = r#"INSERT VERTEX documents(title, embedding) 
                     VALUES "doc1":("标题", [0.1, 0.2])"#;
        // 如果有向量索引，应该推断为 Vector 类型
    }
}
```

### 1.4 交付物

- [ ] 词法分析器支持 VECTOR 关键字
- [ ] 语法分析器支持向量字面值
- [ ] 语义分析器支持维度验证
- [ ] 完整的测试覆盖
- [ ] 语法文档更新

---

## 阶段 2：向量类型验证器（优先级：高）

**目标**：在编译期和运行时提供完整的类型安全检查

**预计工作量**：4-6 天

### 2.1 VectorValidator 设计

**文件**：`src/query/validator/vector_validator.rs`

```rust
/// 向量类型验证器
pub struct VectorValidator {
    metadata: Arc<MetadataContext>,
}

impl VectorValidator {
    /// 推断向量类型
    pub fn infer_vector_type(
        &self,
        value: &Value,
        space_id: u64,
        tag_name: &str,
        field_name: &str
    ) -> Result<DataType> {
        // 从索引元数据获取维度
        let expected_dim = self.get_field_dimension(space_id, tag_name, field_name)?;
        
        match value {
            Value::Vector(vec) => {
                vec.validate_dimension(expected_dim)?;
                Ok(DataType::VectorDense(expected_dim))
            }
            Value::List(list) => {
                // 检查元素类型
                // 检查维度
                // 推断为 Vector 类型
            }
            _ => Err("不是有效的向量类型"),
        }
    }
    
    /// 验证向量维度
    pub fn validate_dimension(
        &self,
        vec: &VectorValue,
        expected: usize
    ) -> Result<(), ValidationError> {
        // 维度验证逻辑
    }
    
    /// 批量验证向量（优化性能）
    pub fn validate_batch(
        &self,
        vectors: &[Value],
        expected_dim: usize
    ) -> Vec<Result<(), ValidationError>> {
        // 批量验证逻辑
    }
}
```

### 2.2 集成到验证流程

**文件**：`src/query/validator/expression_validator.rs`

```rust
impl ExpressionValidator {
    fn validate_insert_values(&mut self, values: &Values) -> Result<()> {
        for (field_name, value) in values {
            let field_def = self.get_field_def(field_name)?;
            
            match field_def.data_type {
                DataType::Vector | DataType::VectorDense(dim) => {
                    self.vector_validator.validate_dimension(value, dim)?;
                }
                _ => {}
            }
        }
    }
}
```

### 2.3 错误类型定义

**文件**：`src/core/error/vector_error.rs`

```rust
#[derive(Debug, Clone)]
pub enum VectorValidationError {
    /// 维度不匹配
    DimensionMismatch {
        expected: usize,
        actual: usize,
        field_name: String,
    },
    /// 元素类型错误
    InvalidElementType {
        field_name: String,
        element_type: DataType,
    },
    /// 向量引擎同步失败
    SyncFailed {
        field_name: String,
        error: String,
    },
}

impl std::fmt::Display for VectorValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VectorValidationError::DimensionMismatch { expected, actual, field_name } => {
                write!(f, "字段 '{}' 的向量维度不匹配：期望 {}, 实际 {}", 
                       field_name, expected, actual)
            }
            VectorValidationError::InvalidElementType { field_name, element_type } => {
                write!(f, "字段 '{}' 的向量元素类型错误：{}", field_name, element_type)
            }
            VectorValidationError::SyncFailed { field_name, error } => {
                write!(f, "字段 '{}' 的向量引擎同步失败：{}", field_name, error)
            }
        }
    }
}
```

### 2.4 测试计划

```rust
#[cfg(test)]
mod vector_validator_tests {
    #[test]
    fn test_dimension_validation() {
        let validator = VectorValidator::new(metadata);
        let vec = Value::vector(vec![0.1; 1536]);
        
        // 维度匹配
        assert!(validator.validate(&vec, 1536).is_ok());
        
        // 维度不匹配
        let result = validator.validate(&vec, 768).unwrap_err();
        assert!(matches!(result, VectorValidationError::DimensionMismatch { .. }));
    }

    #[test]
    fn test_auto_type_inference() {
        // 从 List<Float> 自动推断为 Vector
        let list = Value::List(List::from_vec(vec![
            Value::Float(0.1),
            Value::Float(0.2),
        ]));
        let inferred_type = validator.infer_vector_type(&list, ...).unwrap();
        assert_eq!(inferred_type, DataType::VectorDense(2));
    }
}
```

### 2.5 交付物

- [ ] VectorValidator 实现
- [ ] 集成到 INSERT/UPDATE 验证流程
- [ ] 完整的错误处理
- [ ] 批量验证优化
- [ ] 测试覆盖

---

## 阶段 3：向量运算函数库（优先级：中）

**目标**：提供 SQL 级别的向量运算能力

**预计工作量**：5-8 天

### 3.1 标量函数

#### 3.1.1 相似度计算

```sql
-- 余弦相似度
SELECT cosine_similarity(embedding1, embedding2) 
FROM documents;

-- 点积
SELECT dot_product(embedding, query_vector) 
FROM documents;

-- 欧几里得距离
SELECT euclidean_distance(embedding, query_vector) 
FROM documents;

-- 曼哈顿距离
SELECT manhattan_distance(embedding, query_vector) 
FROM documents;
```

**实现**：`src/query/executor/expression/functions/vector_functions.rs`

```rust
pub fn cosine_similarity(vec1: &VectorValue, vec2: &VectorValue) -> Result<f32> {
    vec1.cosine_similarity(vec2)
        .map_err(|e| ExecutionError::VectorError(e))
}

pub fn dot_product(vec1: &VectorValue, vec2: &VectorValue) -> Result<f32> {
    vec1.dot(vec2)
        .map_err(|e| ExecutionError::VectorError(e))
}

pub fn euclidean_distance(vec1: &VectorValue, vec2: &VectorValue) -> Result<f32> {
    let diff = vec1.to_dense().iter()
        .zip(vec2.to_dense().iter())
        .map(|(&a, &b)| (a - b).powi(2))
        .sum::<f32>();
    Ok(diff.sqrt())
}
```

#### 3.1.2 向量属性

```sql
-- 维度
SELECT dimension(embedding) FROM documents;

-- L2 范数
SELECT l2_norm(embedding) FROM documents;

-- 非零元素数量（稀疏向量）
SELECT nnz(embedding) FROM documents;

-- 归一化
SELECT normalize(embedding) FROM documents;
```

### 3.2 聚合函数

#### 3.2.1 向量平均

```sql
-- 按类别计算平均向量
SELECT category, AVG(embedding) as avg_embedding
FROM documents
GROUP BY category;
```

**实现**：`src/query/executor/expression/functions/vector_aggregates.rs`

```rust
pub struct VectorAvgAccumulator {
    sum: Vec<f32>,
    count: usize,
    dimension: usize,
}

impl VectorAvgAccumulator {
    pub fn new(dimension: usize) -> Self {
        Self {
            sum: vec![0.0; dimension],
            count: 0,
            dimension,
        }
    }
    
    pub fn accumulate(&mut self, vector: &VectorValue) -> Result<()> {
        let vec = vector.to_dense();
        if vec.len() != self.dimension {
            return Err("维度不匹配");
        }
        
        for (i, &val) in vec.iter().enumerate() {
            self.sum[i] += val;
        }
        self.count += 1;
        Ok(())
    }
    
    pub fn finish(self) -> Option<VectorValue> {
        if self.count == 0 {
            None
        } else {
            let avg = self.sum.iter()
                .map(|&s| s / self.count as f32)
                .collect();
            Some(VectorValue::dense(avg))
        }
    }
}
```

#### 3.2.2 其他聚合

```sql
-- 向量求和
SELECT SUM(embedding) FROM documents;

-- 向量最大值（按某个维度）
SELECT MAX(embedding[0]) FROM documents;

-- 向量最小值
SELECT MIN(embedding[0]) FROM documents;
```

### 3.3 向量索引访问

```sql
-- 访问向量的第 i 个元素
SELECT embedding[0] FROM documents;
SELECT embedding[1:10] FROM documents;  -- 切片
```

**实现**：

```rust
pub fn vector_subscript(vector: &VectorValue, index: usize) -> Result<f32> {
    match vector {
        VectorValue::Dense(data) => {
            data.get(index)
                .copied()
                .ok_or_else(|| ExecutionError::OutOfBounds { index, dimension: data.len() })
        }
        VectorValue::Sparse { indices, values } => {
            // 稀疏向量查找
            indices.iter()
                .position(|&i| i == index as u32)
                .and_then(|i| values.get(i).copied())
                .ok_or_else(|| ExecutionError::OutOfBounds { index, dimension: vector.dimension() })
        }
    }
}
```

### 3.4 测试计划

```rust
#[cfg(test)]
mod vector_function_tests {
    #[test]
    fn test_cosine_similarity() {
        let vec1 = VectorValue::dense(vec![1.0, 0.0, 0.0]);
        let vec2 = VectorValue::dense(vec![0.0, 1.0, 0.0]);
        let result = cosine_similarity(&vec1, &vec2).unwrap();
        assert!((result - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_avg_aggregate() {
        let mut acc = VectorAvgAccumulator::new(3);
        acc.accumulate(&VectorValue::dense(vec![1.0, 2.0, 3.0])).unwrap();
        acc.accumulate(&VectorValue::dense(vec![4.0, 5.0, 6.0])).unwrap();
        let avg = acc.finish().unwrap();
        assert_eq!(avg.to_dense(), vec![2.5, 3.5, 4.5]);
    }
}
```

### 3.5 交付物

- [ ] 标量函数库（相似度、距离、属性）
- [ ] 聚合函数库（AVG, SUM）
- [ ] 向量索引访问
- [ ] 完整的 SQL 文档
- [ ] 性能基准测试

---

## 阶段 4：存储优化（优先级：中）

**目标**：进一步优化向量存储效率和访问性能

**预计工作量**：3-6 天

### 4.1 向量压缩

#### 4.1.1 标量量化

```rust
pub enum VectorCompression {
    /// 无压缩
    None,
    /// 8-bit 标量量化
    ScalarQuantization8,
    /// 4-bit 标量量化
    ScalarQuantization4,
    /// 产品量化（需要训练）
    ProductQuantization {
        m: usize,  // 子向量数量
        nbits: usize,  // 每个子向量的比特数
        centroids: Vec<Vec<f32>>,
    },
}

impl VectorValue {
    pub fn compress(&self, compression: &VectorCompression) -> CompressedVector {
        match compression {
            VectorCompression::ScalarQuantization8 => {
                // 将 f32 转换为 u8
                self.quantize_to_u8()
            }
            VectorCompression::ProductQuantization { .. } => {
                // 产品量化
                self.pq_compress()
            }
            _ => CompressedVector::Uncompressed(self.clone()),
        }
    }
}
```

#### 4.1.2 存储格式

```rust
pub struct CompressedVector {
    compression: VectorCompression,
    data: Vec<u8>,
    original_dimension: usize,
    metadata: HashMap<String, String>,
}

impl CompressedVector {
    pub fn decompress(&self) -> VectorValue {
        // 解压缩逻辑
    }
    
    pub fn estimated_size(&self) -> usize {
        // 压缩后的内存估算
    }
}
```

### 4.2 向量化存储

```rust
/// 批量存储向量，减少内存碎片
pub struct VectorBatch {
    vectors: Vec<f32>,  // 连续存储
    dimensions: Vec<usize>,  // 每个向量的维度
    offsets: Vec<usize>,  // 偏移量
}

impl VectorBatch {
    pub fn new(capacity: usize) -> Self {
        Self {
            vectors: Vec::with_capacity(capacity * 1536),  // 预分配
            dimensions: Vec::with_capacity(capacity),
            offsets: Vec::with_capacity(capacity),
        }
    }
    
    pub fn push(&mut self, vector: &VectorValue) {
        let data = vector.to_dense();
        self.offsets.push(self.vectors.len());
        self.dimensions.push(data.len());
        self.vectors.extend(data);
    }
    
    pub fn get(&self, index: usize) -> Option<VectorValue> {
        let offset = self.offsets.get(index)?;
        let dim = self.dimensions.get(index)?;
        let data = self.vectors[*offset..*offset + *dim].to_vec();
        Some(VectorValue::dense(data))
    }
}
```

### 4.3 缓存优化

```rust
/// 向量缓存（使用 LRU 策略）
pub struct VectorCache {
    cache: DashMap<VectorIndexLocation, VectorValue>,
    max_size: usize,
    current_size: AtomicUsize,
}

impl VectorCache {
    pub fn get_or_load<F>(&self, key: &VectorIndexLocation, loader: F) 
    -> Result<VectorValue>
    where
        F: FnOnce() -> Result<VectorValue>,
    {
        if let Some(vec) = self.cache.get(key) {
            return Ok(vec.clone());
        }
        
        let vector = loader()?;
        self.cache.insert(key.clone(), vector.clone());
        Ok(vector)
    }
}
```

### 4.4 交付物

- [ ] 向量压缩支持（标量量化、产品量化）
- [ ] 批量存储优化
- [ ] 向量缓存机制
- [ ] 压缩率基准测试

---

## 阶段 5：测试与优化（优先级：高）

**目标**：确保功能完整性和性能

**预计工作量**：3-5 天

### 5.1 集成测试

#### 5.1.1 端到端测试

```rust
#[cfg(test)]
mod vector_integration_tests {
    #[test]
    fn test_full_vector_workflow() {
        // 1. 创建向量索引
        run_sql("CREATE VECTOR INDEX doc_embedding ON documents(embedding) WITH (dimension=1536)");
        
        // 2. 插入带向量的顶点
        run_sql(r#"INSERT VERTEX documents(title, embedding) 
                   VALUES "doc1":("标题", VECTOR[0.1, 0.2, ...])"#);
        
        // 3. 向量搜索
        let result = run_sql(r#"SEARCH VECTOR doc_embedding 
                              WITH VECTOR[0.1, 0.2, ...] 
                              LIMIT 10"#);
        
        // 4. 验证结果
        assert_eq!(result.rows.len(), 10);
        
        // 5. 更新向量
        run_sql(r#"UPDATE VERTEX "doc1" SET embedding = VECTOR[0.5, 0.6, ...]"#);
        
        // 6. 删除向量
        run_sql(r#"DELETE VERTEX "doc1""#);
    }
}
```

#### 5.1.2 边界条件测试

```rust
#[test]
fn test_edge_cases() {
    // 空向量
    test_vector(VectorValue::dense(vec![]));
    
    // 超大维度
    test_vector(VectorValue::dense(vec![0.1; 10000]));
    
    // 稀疏向量
    test_vector(VectorValue::sparse(vec![0, 999], vec![1.0, 2.0]));
    
    // 维度验证错误
    assert_error_on_dimension_mismatch();
}
```

### 5.2 性能基准测试

```rust
#[bench]
fn bench_vector_creation(b: &mut Bencher) {
    b.iter(|| Value::vector(vec![0.1; 1536]));
}

#[bench]
fn bench_vector_similarity(b: &mut Bencher) {
    let vec1 = VectorValue::dense(vec![0.1; 1536]);
    let vec2 = VectorValue::dense(vec![0.2; 1536]);
    b.iter(|| vec1.cosine_similarity(&vec2));
}

#[bench]
fn bench_vector_serialization(b: &mut Bencher) {
    let vec = Value::vector(vec![0.1; 1536]);
    b.iter(|| bincode::serialize(&vec));
}

#[bench]
fn bench_vector_memory(b: &mut Bencher) {
    b.iter(|| {
        let vec = Value::vector(vec![0.1; 1536]);
        vec.estimated_size()
    });
}
```

### 5.3 性能优化

#### 5.3.1 SIMD 优化

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// 使用 AVX2 加速点积运算
#[target_feature(enable = "avx2")]
unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    // SIMD 实现
}
```

#### 5.3.2 并行计算

```rust
use rayon::prelude::*;

/// 批量计算相似度
pub fn batch_cosine_similarity(vectors: &[VectorValue], query: &VectorValue) -> Vec<f32> {
    vectors.par_iter()
        .map(|v| v.cosine_similarity(query).unwrap_or(0.0))
        .collect()
}
```

### 5.4 文档完善

- [ ] SQL 参考文档更新
- [ ] API 文档完善
- [ ] 性能调优指南
- [ ] 最佳实践文档
- [ ] 迁移指南（从 List 到 Vector）

### 5.5 交付物

- [ ] 完整的集成测试套件
- [ ] 性能基准测试报告
- [ ] SIMD/并行优化
- [ ] 完整的文档体系

---

## 时间线总览

| 阶段 | 任务 | 优先级 | 预计天数 | 依赖 |
|------|------|--------|----------|------|
| 阶段 1 | Parser 扩展 | 高 | 3-5 | 无 |
| 阶段 2 | 向量验证器 | 高 | 4-6 | 阶段 1 |
| 阶段 3 | 向量函数库 | 中 | 5-8 | 阶段 2 |
| 阶段 4 | 存储优化 | 中 | 3-6 | 阶段 2 |
| 阶段 5 | 测试与优化 | 高 | 3-5 | 阶段 3, 4 |

**总计**：15-25 天

---

## 风险与缓解

### 风险 1：Parser 改动影响范围大

**缓解措施**：
- 保持向后兼容，支持 List 语法
- 充分的单元测试和集成测试
- 渐进式发布，先在小范围测试

### 风险 2：向量验证性能开销

**缓解措施**：
- 实现批量验证优化
- 缓存维度信息
- 提供配置选项关闭严格验证

### 风险 3：压缩算法实现复杂

**缓解措施**：
- 优先实现简单的标量量化
- 产品量化可以依赖现有库（如 quantization-rs）
- 提供无压缩选项

### 风险 4：SIMD 优化平台兼容性

**缓解措施**：
- 提供纯 Rust 回退实现
- 使用 `target_feature` 条件编译
- 充分的跨平台测试

---

## 成功标准

### 功能完整性

- [ ] 支持 VECTOR 字面值语法
- [ ] 完整的类型验证
- [ ] 10+ 个向量运算函数
- [ ] 2+ 个向量聚合函数
- [ ] 向量压缩支持

### 性能指标

- [ ] 向量创建时间 < 0.1ms（1536 维）
- [ ] 相似度计算 < 0.05ms（1536 维）
- [ ] 序列化/反序列化 < 0.2ms
- [ ] 内存占用 < 8KB（1536 维，压缩后）

### 质量指标

- [ ] 单元测试覆盖率 > 90%
- [ ] 集成测试覆盖率 > 80%
- [ ] 零编译错误和警告
- [ ] 文档完整度 > 95%

---

## 附录

### A. 相关文件清单

**需要创建的文件**：
- `src/query/parser/vector_literal.rs`
- `src/query/validator/vector_validator.rs`
- `src/query/executor/expression/functions/vector_functions.rs`
- `src/query/executor/expression/functions/vector_aggregates.rs`
- `src/core/value/vector_compression.rs`
- `tests/integration/vector_operations.rs`

**需要修改的文件**：
- `src/query/parser/lexer.rs`
- `src/query/parser/statement_parser.rs`
- `src/query/validator/expression_validator.rs`
- `src/query/executor/expression/functions/mod.rs`
- `docs/release/01_dql_query_syntax.md`
- `docs/release/02_dml_data_manipulation.md`

### B. 参考资源

- [pgvector](https://github.com/pgvector/pgvector) - PostgreSQL 向量扩展
- [Qdrant](https://qdrant.tech/) - 向量搜索引擎
- [Faiss](https://github.com/facebookresearch/faiss) - Facebook 向量相似度搜索库
- [quantization-rs](https://github.com/quantization-rs) - Rust 量化库

### C. 团队分工建议

- **Parser 扩展**：1 人（熟悉词法/语法分析）
- **验证器**：1 人（熟悉类型系统）
- **函数库**：2 人（熟悉数学运算和 SQL 函数）
- **存储优化**：1 人（熟悉存储和压缩算法）
- **测试与优化**：1-2 人（熟悉测试和性能分析）

---

**文档版本**：v1.0  
**创建日期**：2026-04-10  
**最后更新**：2026-04-10  
**维护者**：GraphDB 开发团队
