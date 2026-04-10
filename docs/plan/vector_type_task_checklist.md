# 向量类型实施任务清单

## 阶段 1：Parser 扩展与字面值语法（3-5 天）

### 任务 1.1：词法分析器扩展（1 天）

- [ ] **1.1.1** 添加 VECTOR 关键字到 Token 枚举
  - 文件：`src/query/parser/lexer.rs`
  - 修改：`enum Token { ..., VectorKeyword }`
  
- [ ] **1.1.2** 实现 VECTOR 关键字识别
  - 在 `next_token()` 中添加 VECTOR 识别逻辑
  - 测试：`VECTOR` → `Token::VectorKeyword`
  
- [ ] **1.1.3** 添加向量字面值 Token
  - 添加 `VectorLiteral(Vec<f32>)` 变体
  - 支持 `VECTOR[...]` 解析
  
- [ ] **1.1.4** 单元测试
  ```rust
  #[test]
  fn test_vector_keyword_token() {
      let tokens = tokenize("VECTOR[0.1, 0.2]");
      assert_eq!(tokens[0], Token::VectorKeyword);
  }
  ```

### 任务 1.2：语法分析器扩展（2 天）

- [ ] **1.2.1** 扩展 AST 表达式类型
  - 文件：`src/query/parser/ast.rs`
  - 添加：`Expression::VectorLiteral(Vec<f32>)`
  - 添加：`Expression::VectorCast { expr, dimension }`
  
- [ ] **1.2.2** 实现向量字面值解析函数
  - 文件：`src/query/parser/expression_parser.rs`
  - 函数：`parse_vector_literal() -> Result<Expression>`
  - 支持：`VECTOR[...]` 语法
  
- [ ] **1.2.3** 实现类型转换语法解析
  - 函数：`parse_cast_to_vector() -> Result<Expression>`
  - 支持：`[...]::VECTOR` 语法
  
- [ ] **1.2.4** 修改 INSERT 语句解析
  - 文件：`src/query/parser/statement_parser.rs`
  - 支持在 VALUES 子句中使用向量字面值
  
- [ ] **1.2.5** 单元测试
  ```rust
  #[test]
  fn test_parse_vector_insert() {
      let sql = r#"INSERT VERTEX docs(embedding) 
                   VALUES "v1":(VECTOR[0.1, 0.2])"#;
      let ast = parse(sql).unwrap();
      assert!(matches!(ast, Statement::Insert { .. }));
  }
  ```

### 任务 1.3：语义分析器（1 天）

- [ ] **1.3.1** 实现向量类型推断
  - 文件：`src/query/semantic_analyzer.rs`
  - 函数：`infer_vector_type(value: &Value) -> Result<DataType>`
  
- [ ] **1.3.2** 添加维度验证
  - 从索引元数据获取期望维度
  - 验证实际维度是否匹配
  
- [ ] **1.3.3** 集成到表达式分析流程
  - 在 `analyze_expression()` 中添加向量分支
  
- [ ] **1.3.4** 单元测试
  ```rust
  #[test]
  fn test_vector_dimension_check() {
      let vec = Value::vector(vec![0.1; 1536]);
      let result = analyze_vector(&vec, 1536).unwrap();
      assert_eq!(result, DataType::VectorDense(1536));
  }
  ```

### 任务 1.4：向后兼容支持（1 天）

- [ ] **1.4.1** 保持 List 语法有效
  - 确保 `[0.1, 0.2]` 仍然可以解析
  
- [ ] **1.4.2** 实现自动类型推断
  - 如果字段有向量索引，自动将 List 转为 Vector
  
- [ ] **1.4.3** 集成测试
  ```rust
  #[test]
  fn test_backward_compatibility() {
      // 旧语法仍然有效
      let sql = r#"INSERT VERTEX docs(embedding) 
                   VALUES "v1":([0.1, 0.2])"#;
      let result = parse_and_analyze(sql).unwrap();
      // 应该自动推断为 Vector 类型
  }
  ```

### 交付物检查清单

- [ ] 词法分析器支持 VECTOR 关键字
- [ ] 语法分析器支持 3 种向量语法
- [ ] 语义分析器支持维度验证
- [ ] 向后兼容测试通过
- [ ] 文档更新（语法部分）

---

## 阶段 2：向量类型验证器（4-6 天）

### 任务 2.1：错误类型定义（0.5 天）

- [ ] **2.1.1** 创建向量错误枚举
  - 文件：`src/core/error/vector_error.rs`
  - 定义：`VectorValidationError`
  
- [ ] **2.1.2** 实现错误显示
  - 实现 `Display` trait
  - 提供友好的错误信息
  
- [ ] **2.1.3** 单元测试
  ```rust
  #[test]
  fn test_vector_error_display() {
      let err = VectorValidationError::DimensionMismatch {
          expected: 1536,
          actual: 768,
          field_name: "embedding".to_string(),
      };
      assert!(err.to_string().contains("1536"));
  }
  ```

### 任务 2.2：VectorValidator 核心实现（2 天）

- [ ] **2.2.1** 创建验证器结构
  - 文件：`src/query/validator/vector_validator.rs`
  - 结构：`VectorValidator { metadata: Arc<MetadataContext> }`
  
- [ ] **2.2.2** 实现维度验证方法
  ```rust
  pub fn validate_dimension(
      &self,
      vec: &VectorValue,
      expected: usize
  ) -> Result<(), VectorValidationError>
  ```
  
- [ ] **2.2.3** 实现类型推断方法
  ```rust
  pub fn infer_vector_type(
      &self,
      value: &Value,
      space_id: u64,
      tag_name: &str,
      field_name: &str
  ) -> Result<DataType>
  ```
  
- [ ] **2.2.4** 实现批量验证方法
  ```rust
  pub fn validate_batch(
      &self,
      vectors: &[Value],
      expected_dim: usize
  ) -> Vec<Result<(), VectorValidationError>>
  ```
  
- [ ] **2.2.5** 单元测试
  ```rust
  #[test]
  fn test_validate_dimension() {
      let validator = VectorValidator::new(metadata);
      let vec = Value::vector(vec![0.1; 1536]);
      assert!(validator.validate(&vec, 1536).is_ok());
      assert!(validator.validate(&vec, 768).is_err());
  }
  ```

### 任务 2.3：集成到验证流程（1.5 天）

- [ ] **2.3.1** 修改 INSERT 验证
  - 文件：`src/query/validator/insert_validator.rs`
  - 在验证 VALUES 时调用 `VectorValidator`
  
- [ ] **2.3.2** 修改 UPDATE 验证
  - 文件：`src/query/validator/update_validator.rs`
  - 验证 SET 子句中的向量字段
  
- [ ] **2.3.3** 集成到表达式验证
  - 文件：`src/query/validator/expression_validator.rs`
  - 在验证表达式时处理向量类型
  
- [ ] **2.3.4** 集成测试
  ```rust
  #[test]
  fn test_insert_vector_validation() {
      let sql = r#"INSERT VERTEX docs(embedding) 
                   VALUES "v1":(VECTOR[0.1])"#;
      // 如果期望维度是 1536，应该报错
      let result = execute(sql).unwrap_err();
      assert!(matches!(result, Error::Vector(DimensionMismatch { .. })));
  }
  ```

### 任务 2.4：元数据集成（1 天）

- [ ] **2.4.1** 扩展索引元数据
  - 文件：`src/core/types/index.rs`
  - 添加：`vector_dimension: Option<usize>` 字段
  
- [ ] **2.4.2** 修改 CREATE VECTOR INDEX 解析
  - 文件：`src/query/parser/ddl_parser.rs`
  - 解析 `WITH (dimension=1536)` 选项
  
- [ ] **2.4.3** 存储维度信息
  - 在创建索引时保存维度到元数据
  
- [ ] **2.4.4** 查询维度信息
  - 实现 `get_vector_dimension()` 方法
  
- [ ] **2.4.5** 集成测试
  ```rust
  #[test]
  fn test_vector_index_metadata() {
      run_sql("CREATE VECTOR INDEX idx ON docs(embedding) WITH (dimension=1536)");
      let dim = get_dimension(space_id, "docs", "embedding").unwrap();
      assert_eq!(dim, 1536);
  }
  ```

### 交付物检查清单

- [ ] VectorValidator 实现完成
- [ ] 错误处理完善
- [ ] 集成到 INSERT/UPDATE 流程
- [ ] 元数据支持维度信息
- [ ] 测试覆盖率 > 90%

---

## 阶段 3：向量运算函数库（5-8 天）

### 任务 3.1：标量函数 - 相似度计算（2 天）

- [ ] **3.1.1** 余弦相似度函数
  - 文件：`src/query/executor/expression/functions/vector_functions.rs`
  - SQL: `cosine_similarity(vec1, vec2)`
  - 实现：调用 `VectorValue::cosine_similarity()`
  
- [ ] **3.1.2** 点积函数
  - SQL: `dot_product(vec1, vec2)`
  - 实现：调用 `VectorValue::dot()`
  
- [ ] **3.1.3** 欧几里得距离函数
  - SQL: `euclidean_distance(vec1, vec2)`
  - 实现：计算 L2 范数
  
- [ ] **3.1.4** 曼哈顿距离函数
  - SQL: `manhattan_distance(vec1, vec2)`
  - 实现：计算 L1 范数
  
- [ ] **3.1.5** 函数注册
  - 文件：`src/query/executor/expression/functions/mod.rs`
  - 注册所有向量函数到函数表
  
- [ ] **3.1.6** 单元测试
  ```rust
  #[test]
  fn test_cosine_similarity_function() {
      let vec1 = VectorValue::dense(vec![1.0, 0.0]);
      let vec2 = VectorValue::dense(vec![0.0, 1.0]);
      let result = cosine_similarity(&vec1, &vec2).unwrap();
      assert!((result - 0.0).abs() < 1e-6);
  }
  ```

### 任务 3.2：标量函数 - 向量属性（1 天）

- [ ] **3.2.1** dimension() 函数
  - SQL: `dimension(vector)`
  - 返回：向量维度
  
- [ ] **3.2.2** l2_norm() 函数
  - SQL: `l2_norm(vector)`
  - 实现：调用 `VectorValue::l2_norm()`
  
- [ ] **3.2.3** nnz() 函数
  - SQL: `nnz(vector)`
  - 返回：非零元素数量
  
- [ ] **3.2.4** normalize() 函数
  - SQL: `normalize(vector)`
  - 返回：归一化后的向量
  
- [ ] **3.2.5** 单元测试
  ```rust
  #[test]
  fn test_dimension_function() {
      let vec = VectorValue::dense(vec![0.1; 1536]);
      assert_eq!(dimension(&vec).unwrap(), 1536);
  }
  ```

### 任务 3.3：聚合函数（2 天）

- [ ] **3.3.1** AVG 聚合函数
  - 文件：`src/query/executor/expression/functions/vector_aggregates.rs`
  - SQL: `AVG(embedding)`
  - 实现：`VectorAvgAccumulator`
  
- [ ] **3.3.2** SUM 聚合函数
  - SQL: `SUM(embedding)`
  - 实现：向量求和
  
- [ ] **3.3.3** 聚合函数注册
  - 文件：`src/query/executor/expression/functions/mod.rs`
  - 注册到聚合函数表
  
- [ ] **3.3.4** 集成测试
  ```rust
  #[test]
  fn test_vector_avg_aggregate() {
      let sql = r#"SELECT AVG(embedding) FROM documents GROUP BY category"#;
      let result = execute(sql).unwrap();
      assert!(result.rows.len() > 0);
  }
  ```

### 任务 3.4：向量索引访问（1 天）

- [ ] **3.4.1** 下标访问函数
  - SQL: `vector[i]`
  - 实现：`vector_subscript(vector, index)`
  
- [ ] **3.4.2** 切片访问函数
  - SQL: `vector[start:end]`
  - 实现：返回子向量
  
- [ ] **3.4.3** 单元测试
  ```rust
  #[test]
  fn test_vector_subscript() {
      let vec = VectorValue::dense(vec![1.0, 2.0, 3.0]);
      assert_eq!(vector_subscript(&vec, 0).unwrap(), 1.0);
  }
  ```

### 任务 3.5：文档和示例（1 天）

- [ ] **3.5.1** SQL 参考文档更新
  - 文件：`docs/release/01_dql_query_syntax.md`
  - 添加向量函数章节
  
- [ ] **3.5.2** 使用示例
  - 文件：`docs/extend/vector/vector_type_usage_examples.md`
  - 添加函数使用示例
  
- [ ] **3.5.3** 基准测试
  - 创建性能基准
  - 记录函数执行时间

### 交付物检查清单

- [ ] 10+ 个标量函数
- [ ] 2+ 个聚合函数
- [ ] 向量索引访问
- [ ] 完整的 SQL 文档
- [ ] 性能基准报告

---

## 阶段 4：存储优化（3-6 天）

### 任务 4.1：向量压缩（2 天）

- [ ] **4.1.1** 定义压缩枚举
  - 文件：`src/core/value/vector_compression.rs`
  - `enum VectorCompression { None, Scalar8, Scalar4, PQ }`
  
- [ ] **4.1.2** 实现标量量化
  - `quantize_to_u8()` - f32 转 u8
  - `dequantize_from_u8()` - u8 转 f32
  
- [ ] **4.1.3** 实现压缩向量类型
  - `struct CompressedVector { ... }`
  - 实现 `compress()` 和 `decompress()`
  
- [ ] **4.1.4** 单元测试
  ```rust
  #[test]
  fn test_scalar_quantization() {
      let vec = VectorValue::dense(vec![0.0, 0.5, 1.0]);
      let compressed = vec.compress(&VectorCompression::Scalar8);
      let decompressed = compressed.decompress();
      // 验证精度损失在可接受范围内
  }
  ```

### 任务 4.2：批量存储（1.5 天）

- [ ] **4.2.1** 实现 VectorBatch
  - 文件：`src/core/value/vector_batch.rs`
  - 连续存储多个向量
  
- [ ] **4.2.2** 实现批量访问方法
  - `push()`, `get()`, `iter()`
  
- [ ] **4.2.3** 性能测试
  - 对比单个存储 vs 批量存储
  - 验证内存效率提升

### 任务 4.3：缓存机制（1.5 天）

- [ ] **4.3.1** 实现 VectorCache
  - 文件：`src/vector/cache.rs`
  - 使用 DashMap + LRU 策略
  
- [ ] **4.3.2** 集成到 VectorCoordinator
  - 在查询时先检查缓存
  
- [ ] **4.3.3** 缓存命中率统计
  - 添加指标收集
  
- [ ] **4.3.4** 性能测试
  - 对比有缓存 vs 无缓存

### 交付物检查清单

- [ ] 向量压缩支持
- [ ] 批量存储优化
- [ ] 向量缓存机制
- [ ] 压缩率基准测试

---

## 阶段 5：测试与优化（3-5 天）

### 任务 5.1：集成测试（1.5 天）

- [ ] **5.1.1** 端到端测试
  - 文件：`tests/integration/vector_operations.rs`
  - 测试完整工作流
  
- [ ] **5.1.2** 边界条件测试
  - 空向量、超大维度、稀疏向量
  
- [ ] **5.1.3** 错误处理测试
  - 维度不匹配、类型错误
  
- [ ] **5.1.4** 并发测试
  - 多线程并发访问向量

### 任务 5.2：性能基准（1 天）

- [ ] **5.2.1** 创建基准测试文件
  - 文件：`benches/vector_benchmarks.rs`
  
- [ ] **5.2.2** 向量创建基准
  ```rust
  #[bench]
  fn bench_vector_creation(b: &mut Bencher) { ... }
  ```
  
- [ ] **5.2.3** 相似度计算基准
  ```rust
  #[bench]
  fn bench_vector_similarity(b: &mut Bencher) { ... }
  ```
  
- [ ] **5.2.4** 序列化基准
  ```rust
  #[bench]
  fn bench_vector_serialization(b: &mut Bencher) { ... }
  ```
  
- [ ] **5.2.5** 生成基准报告
  - 运行 `cargo bench`
  - 保存结果到文档

### 任务 5.3：性能优化（1.5 天）

- [ ] **5.3.1** SIMD 优化（可选）
  - 实现 AVX2 加速版本
  - 提供纯 Rust 回退
  
- [ ] **5.3.2** 并行计算
  - 使用 Rayon 并行化批量操作
  
- [ ] **5.3.3** 内存优化
  - 减少不必要的克隆
  - 使用引用和 Cow
  
- [ ] **5.3.4** 验证优化效果
  - 对比优化前后性能

### 任务 5.4：文档完善（1 天）

- [ ] **5.4.1** API 文档
  - 运行 `cargo doc`
  - 确保所有公共 API 有文档
  
- [ ] **5.4.2** 性能调优指南
  - 文件：`docs/extend/vector/performance_tuning.md`
  
- [ ] **5.4.3** 最佳实践
  - 文件：`docs/extend/vector/best_practices.md`
  
- [ ] **5.4.4** 迁移指南
  - 文件：`docs/extend/vector/migration_guide.md`

### 交付物检查清单

- [ ] 完整的集成测试套件
- [ ] 性能基准报告
- [ ] SIMD/并行优化
- [ ] 完整的文档体系

---

## 总体检查清单

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

**版本**：v1.0  
**创建日期**：2026-04-10  
**维护者**：GraphDB 开发团队
