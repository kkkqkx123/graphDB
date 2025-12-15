# Context迁移计划

## 概述

本文档描述了从nebula-graph C++版本到Rust版本的context相关模块的迁移计划。Context模块是查询处理系统的核心组件，负责管理查询执行过程中的各种上下文信息。

## 分析总结

### nebula-graph C++版中的Context定义

1. **ExpressionContext** (`nebula-3.8.0/src/common/context/ExpressionContext.h`)
   - 基础表达式求值上下文接口
   - 提供变量、属性访问的抽象方法
   - 非线程安全设计

2. **QueryContext** (`nebula-3.8.0/src/graph/context/QueryContext.h`)
   - 查询级别的上下文，管理整个查询生命周期
   - 包含请求上下文、验证上下文、执行上下文
   - 管理Schema管理器、索引管理器、存储客户端等资源

3. **ExecutionContext** (`nebula-3.8.0/src/graph/context/ExecutionContext.h`)
   - 查询执行期间的上下文
   - 管理变量值的多版本历史
   - 支持变量的版本控制

4. **RequestContext** (`nebula-3.8.0/src/graph/service/RequestContext.h`)
   - 请求级别的上下文
   - 管理会话信息、查询参数、响应对象
   - 处理请求生命周期

5. **RuntimeContext** (`nebula-3.8.0/src/storage/CommonUtils.h`)
   - 存储层运行时上下文
   - 包含计划上下文引用和运行时可变信息
   - 用于存储层执行节点

6. **StorageExpressionContext** (`nebula-3.8.0/src/storage/context/StorageExpressionContext.h`)
   - 存储层表达式求值上下文
   - 继承自ExpressionContext
   - 支持从RowReader读取值和用户设置值

7. **QueryAstContext** (`nebula-3.8.0/src/graph/context/ast/QueryAstContext.h`)
   - AST级别的上下文
   - 包含各种查询语句的特定上下文信息

### 新Rust架构中已有的Context实现

1. **RequestContext** (`src/query/context/request_context.rs`)
   - 已实现，功能完整
   - 管理会话信息、请求参数、响应对象

2. **QueryContext** (`src/query/context/query_context.rs`)
   - 已实现，功能完整
   - 管理查询级别的资源和状态

3. **QueryExecutionContext** (`src/query/context/execution_context.rs`)
   - 已实现，功能完整
   - 管理查询执行期间的变量和结果

4. **QueryExpressionContext** (`src/query/context/expression_context.rs`)
   - 已实现，功能完整
   - 为表达式求值提供上下文

5. **AstContext** (`src/query/context/ast_context.rs`)
   - 已实现，包含各种查询语句的上下文

6. **ValidateContext** (`src/query/context/validate/context.rs`)
   - 已实现，功能完整
   - 管理验证阶段的上下文信息

7. **EvalContext** (`src/graph/expression/context.rs`)
   - 已实现，但位置不在query/context目录
   - 用于表达式求值

## 需要迁移的模块

### 1. EvalContext迁移

**源位置**: `src/graph/expression/context.rs`
**目标位置**: `src/query/context/expression_eval_context.rs`

**原因**:
- EvalContext是表达式求值的核心上下文
- 应该与其他query context模块放在同一目录下
- 便于统一管理和维护

**迁移内容**:
- EvalContext结构体
- SerializableEvalContext结构体
- 相关的实现方法

### 2. 存储层Context相关模块

**需要创建的新模块**:

1. **RuntimeContext** (`src/query/context/runtime_context.rs`)
   - 对应C++版的RuntimeContext
   - 用于存储层执行节点
   - 包含计划上下文引用和运行时可变信息

2. **StorageExpressionContext** (`src/query/context/storage_expression_context.rs`)
   - 对应C++版的StorageExpressionContext
   - 存储层表达式求值上下文
   - 支持从RowReader读取值和用户设置值

## 迁移计划

### 阶段1: EvalContext迁移

1. 创建新文件 `src/query/context/expression_eval_context.rs`
2. 将 `src/graph/expression/context.rs` 的内容复制到新文件
3. 更新模块导入和引用
4. 更新所有使用EvalContext的文件中的导入路径
5. 删除原文件 `src/graph/expression/context.rs`
6. 运行测试确保功能正常

### 阶段2: 存储层Context实现

1. 实现 `RuntimeContext`
   ```rust
   // src/query/context/runtime_context.rs
   pub struct RuntimeContext {
       pub plan_context: Arc<PlanContext>,
       pub tag_id: TagID,
       pub tag_name: String,
       pub tag_schema: Option<NebulaSchemaProvider>,
       pub edge_type: EdgeType,
       pub edge_name: String,
       pub edge_schema: Option<NebulaSchemaProvider>,
       pub column_idx: usize,
       pub props: Option<Vec<PropContext>>,
       pub insert: bool,
       pub filter_invalid_result_out: bool,
       pub result_stat: ResultStatus,
   }
   ```

2. 实现 `StorageExpressionContext`
   ```rust
   // src/query/context/storage_expression_context.rs
   pub struct StorageExpressionContext {
       pub v_id_len: usize,
       pub is_int_id: bool,
       pub reader: Option<RowReaderWrapper>,
       pub key: String,
       pub name: String,
       pub schema: Option<NebulaSchemaProvider>,
       pub is_edge: bool,
       pub is_index: bool,
       pub has_nullable_col: bool,
       pub fields: Vec<ColumnDef>,
       pub tag_filters: HashMap<(String, String), Value>,
       pub edge_filters: HashMap<(String, String), Value>,
       pub value_map: HashMap<String, Vec<Value>>,
       pub expr_value_map: HashMap<String, Value>,
   }
   ```

3. 更新 `src/query/context/mod.rs` 以包含新模块

### 阶段3: 集成和测试

1. 更新所有使用这些context的模块
2. 确保与新架构的其他组件兼容
3. 编写单元测试和集成测试
4. 性能测试和优化

## 影响分析

### 需要更新的文件

1. **EvalContext迁移影响**:
   - `src/graph/expression/mod.rs`
   - `src/graph/expression/evaluator.rs`
   - `src/graph/expression/aggregate.rs`
   - `src/graph/expression/binary.rs`
   - `src/graph/expression/container.rs`
   - `src/graph/expression/function.rs`
   - `src/graph/expression/property.rs`
   - `src/graph/expression/unary.rs`
   - `src/query/executor/result_processing/projection.rs`
   - `src/query/executor/data_processing/filter.rs`
   - `src/query/executor/data_processing/loops.rs`
   - `src/query/executor/data_processing/sample.rs`
   - `src/query/executor/data_processing/sort.rs`

2. **新增存储层Context影响**:
   - 需要在存储层实现中使用这些新的context
   - 可能需要创建存储层的执行节点和处理器的Rust版本

## 风险评估

### 低风险
- EvalContext迁移: 主要是文件位置移动，功能不变
- 模块导入更新: 简单的路径替换

### 中等风险
- 存储层Context实现: 需要仔细设计以匹配C++版功能
- 与现有代码的集成: 可能需要调整接口

### 缓解措施
1. 分阶段实施，每个阶段完成后进行充分测试
2. 保留原有接口的兼容性，逐步迁移
3. 编写全面的测试用例
4. 代码审查确保设计合理性

## 时间表

- **阶段1 (EvalContext迁移)**: 2-3天
- **阶段2 (存储层Context实现)**: 5-7天
- **阶段3 (集成和测试)**: 3-5天

总计: 10-15天

## 成功标准

1. 所有context模块统一位于 `src/query/context/` 目录
2. 功能与C++版本保持一致
3. 所有测试通过
4. 性能不低于原实现
5. 代码结构清晰，易于维护

## 后续优化

1. 考虑使用trait来统一不同context的接口
2. 优化内存使用和性能
3. 添加更多文档和示例
4. 考虑异步context支持