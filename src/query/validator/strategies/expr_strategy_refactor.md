### 1. Trait 定义与实现 (行14-85)
```
trait ExpressionValidationContext 
{ ... }
impl ExpressionValidationContext 
for WhereClauseContext { ... }
impl ExpressionValidationContext 
for MatchClauseContext { ... }
// ... 共 6 个 Context 的实现
```
### 2. 验证方法 (行93-350)
- validate_filter / validate_path / validate_return
- validate_with / validate_unwind / validate_yield
- validate_node_pattern / validate_edge_pattern
### 3. 类型推导系统 (行373-930)
```
fn 
validate_expression_type_full<C: ...
>(...) -> Result<...>
fn 
deduce_expression_type_full<C: ...>
(...) -> ValueTypeDef
fn deduce_binary_expr_type(...) -> 
ValueTypeDef
fn deduce_function_return_type(...) 
-> ValueTypeDef
// ... 10+ 个类型推导方法
```
### 4. 聚合验证 (行967-1122)
```
fn validate_aggregate_expression
(...) -> Result<...>
fn validate_group_key_expression
(...) -> Result<...>
fn validate_aggregate_arguments
(...) -> Result<...>
```
### 5. 变量验证 (行1133-1460)
```
fn validate_variable_scope<C: ...>
(...) -> Result<...>
fn validate_variable_usage(...) -> 
Result<...>
fn validate_variable_name_format
(...) -> Result<...>
```
### 6. 表达式操作验证 (行1464-1670)
```
fn validate_expression_operations
(...) -> Result<...>
fn validate_expression_cycles(...) 
-> Result<...>
fn calculate_expression_depth(...) 
-> usize
```
### 7. 测试函数 (行1853-2226)
373 行测试代码 ，与业务逻辑混在一起

## ⚠️ 问题总结
1. 违反单一职责原则 - 一个文件处理了：路径验证、类型推导、聚合验证、变量验证、表达式操作验证
2. 文件过大 - 2226 行远超推荐的单文件上限（300-500行）
3. 类型推导系统膨胀 - 包含完整的类型推导逻辑，应该独立
4. 测试代码混在一起 - 373行测试代码应分离到单独文件
5. 部分方法实现粗糙 - 如 validate_edge_pattern 的简化实现注释
## ✅ 重构建议
```
strategies/
├── expression_strategy.rs      # 精
简到 500 行以内
│   ├── 保留核心验证方法
│   └── 保留公共 trait 实现
├── type_inference.rs           # 新
建：类型推导 (600行)
├── variable_validator.rs       # 新
建：变量验证 (400行)
└── expression_operations.rs    # 新
建：表达式操作验证 (300行)
```
### 提取优先级
1. 高优先级 ：类型推导系统 → type_inference.rs
2. 中优先级 ：变量验证 → variable_validator.rs
3. 中优先级 ：测试代码 → expression_strategy_test.rs
4. 低优先级 ：表达式操作验证 → expression_operations.rs
