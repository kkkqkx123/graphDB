# ExpressionContext 迁移要求文档

## 概述

本文档记录了从 `query::context::EvalContext` 到 `expression::ExpressionContext` 的迁移要求和修改指南。

## 当前状态

### 已完成的修改

1. **Expression模块重构**
   - 将 `src/graph/expression` 移动到 `src/expression`
   - 创建了新的 `ExpressionContext` 枚举，使用零成本抽象替代 `dyn` trait
   - 更新了所有 expression 模块内部的函数签名

2. **核心文件修改**
   - `src/expression/evaluator.rs` - 更新为使用 `ExpressionContext`
   - `src/expression/binary.rs` - 更新函数签名
   - `src/expression/unary.rs` - 更新函数签名
   - `src/expression/function.rs` - 重写，移除字段访问依赖
   - `src/expression/property.rs` - 重写，使用方法调用替代字段访问
   - `src/expression/container.rs` - 更新函数签名
   - `src/expression/aggregate.rs` - 更新函数签名
   - `src/expression/cypher/cypher_evaluator.rs` - 更新所有函数签名
   - `src/expression/aggregate_functions.rs` - 更新函数签名
   - `src/expression/cypher/mod.rs` - 更新测试和函数签名

## 待完成的修改

### Query模块中的文件需要更新

以下文件仍在使用旧的 `EvalContext`，需要更新为 `ExpressionContext`：

1. **执行器模块**
   - `src/query/executor/data_processing/transformations/append_vertices.rs`
   - `src/query/executor/data_processing/loops.rs`
   - `src/query/executor/result_processing/aggregation.rs`
   - `src/query/executor/result_processing/projection.rs`
   - `src/query/executor/result_processing/sort.rs`
   - `src/query/executor/result_processing/filter.rs`
   - `src/query/executor/result_processing/topn.rs`
   - `src/query/executor/tag_filter.rs`
   - `src/query/executor/data_access.rs`
   - `src/query/executor/data_processing/transformations/assign.rs`
   - `src/query/executor/data_processing/transformations/unwind.rs`
   - `src/query/executor/data_processing/transformations/pattern_apply.rs`
   - `src/query/executor/data_processing/transformations/rollup_apply.rs`
   - `src/query/executor/cypher/clauses/match_path/expression_evaluator.rs`

### 修改模式

对于每个文件，需要进行以下类型的修改：

#### 1. 导入语句更新
```rust
// 旧的导入
use crate::query::context::EvalContext;

// 新的导入
use crate::expression::ExpressionContext;
```

#### 2. 函数参数类型更新
```rust
// 旧的函数签名
fn some_function(context: &EvalContext) -> Result<Value, Error>

// 新的函数签名
fn some_function(context: &ExpressionContext) -> Result<Value, Error>
```

#### 3. 字段访问更新
```rust
// 旧的字段访问
context.vars.get("variable_name")
context.vertex
context.edge

// 新的方法调用
context.get_variable("variable_name")
context.get_vertex()
context.get_edge()
```

#### 4. 变量设置更新
```rust
// 旧的变量设置
context.vars.insert("name".to_string(), value);

// 新的变量设置
context.set_variable("name".to_string(), value);
```

### 批量修改脚本

可以使用以下Python脚本进行批量修改：

```python
import re
import os

def fix_eval_context_usage(file_path):
    """修复文件中的EvalContext使用"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 替换导入
    content = re.sub(
        r'use crate::query::context::EvalContext',
        'use crate::expression::ExpressionContext',
        content
    )
    
    # 替换参数类型
    content = re.sub(
        r'&EvalContext',
        '&ExpressionContext',
        content
    )
    
    # 替换字段访问
    content = re.sub(r'\bcontext\.vars\.get\(([^)]+)\)', r'context.get_variable(\1)', content)
    content = re.sub(r'\bcontext\.vertex\b', 'context.get_vertex()', content)
    content = re.sub(r'\bcontext\.edge\b', 'context.get_edge()', content)
    
    # 替换变量设置
    content = re.sub(
        r'context\.vars\.insert\(([^,]+),\s*([^)]+)\)',
        r'context.set_variable(\1, \2)',
        content
    )
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)

# 应用到所有相关文件
files_to_fix = [
    "src/query/executor/data_processing/transformations/append_vertices.rs",
    # ... 其他文件列表
]

for file_path in files_to_fix:
    if os.path.exists(file_path):
        fix_eval_context_usage(file_path)
        print(f"Fixed: {file_path}")
```

## 架构改进

### 新的ExpressionContext设计

```rust
pub enum ExpressionContext {
    Simple(SimpleExpressionContext),
    Query(QueryContextAdapter),
}

impl ExpressionContext {
    pub fn get_variable(&self, name: &str) -> Option<Value>;
    pub fn set_variable(&mut self, name: String, value: Value);
    pub fn get_vertex(&self) -> Option<&Vertex>;
    pub fn get_edge(&self) -> Option<&Edge>;
    pub fn get_path(&self, name: &str) -> Option<&Path>;
    
    pub fn simple() -> Self;
    pub fn query() -> Self;
}
```

### 优势

1. **零成本抽象**：使用枚举替代trait对象，避免动态分发开销
2. **类型安全**：编译时确定上下文类型，减少运行时错误
3. **扩展性**：易于添加新的上下文类型
4. **性能优化**：消除虚函数调用开销

## 迁移优先级

### 高优先级
1. 核心执行器文件（projection.rs, filter.rs, aggregation.rs等）
2. 数据处理转换文件
3. Cypher执行器相关文件

### 中优先级
1. 辅助执行器文件
2. 工具函数文件

### 低优先级
1. 测试文件
2. 示例代码文件

## 验证方法

1. **编译检查**：确保所有文件编译通过
2. **单元测试**：运行相关单元测试
3. **集成测试**：验证查询执行流程
4. **性能测试**：确保性能没有回归

## 注意事项

1. **向后兼容性**：本次重构不保持向后兼容性，直接采用新设计
2. **测试覆盖**：确保所有修改的代码都有相应的测试覆盖
3. **文档更新**：及时更新相关API文档
4. **代码审查**：所有修改都需要经过代码审查

## 完成标准

当以下条件满足时，认为迁移完成：

1. 所有编译错误已解决
2. 所有单元测试通过
3. 集成测试通过
4. 性能测试显示无显著回归
5. 代码审查通过

## 相关文档

- [Expression模块设计文档](./expression_design.md)
- [架构重构总结](../CONTEXT_REFACTORING_SUMMARY.md)
- [性能优化指南](../performance_optimization.md)