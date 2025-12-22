# 表达式求值器改进方案

## 概述

基于对 Nebula-Graph 表达式系统的深入分析，本文档提出了针对当前 Rust 实现的全面改进方案。改进重点在于完善核心功能、优化性能、增强可维护性。

## 1. 核心问题分析

### 1.1 当前架构问题

1. **功能不完整**：大量表达式类型返回"尚未实现"错误
2. **架构过度复杂**：多层抽象导致代码冗余
3. **缺少关键优化**：没有表达式编译和常量折叠
4. **性能监控不足**：缺少详细的性能指标

### 1.2 与 Nebula-Graph 的差距

| 功能 | Nebula-Graph | 当前实现 | 差距 |
|------|-------------|----------|------|
| 基础运算 | ✅ 完整 | ❌ 缺失 | 高 |
| 类型转换 | ✅ 完整 | ❌ 缺失 | 高 |
| 属性访问 | ✅ 完整 | ❌ 缺失 | 高 |
| 函数调用 | ✅ 完整 | ❌ 缺失 | 中 |
| 表达式优化 | ✅ 支持 | ❌ 缺失 | 中 |
| 性能监控 | ✅ 详细 | ⚠️ 基础 | 中 |

## 2. 改进方案

### 2.1 阶段一：核心功能实现（优先级：高）

#### 2.1.1 完善二元运算

**目标**：实现所有基础算术和逻辑运算

**实现方案**：
```rust
impl ExpressionEvaluator {
    fn eval_binary_operation(
        &self,
        left: &Value,
        op: &BinaryOperator,
        right: &Value,
    ) -> Result<Value, ExpressionError> {
        match op {
            BinaryOperator::Add => left.add(right)
                .map_err(|e| ExpressionError::runtime_error(e)),
            BinaryOperator::Subtract => left.sub(right)
                .map_err(|e| ExpressionError::runtime_error(e)),
            BinaryOperator::Multiply => left.mul(right)
                .map_err(|e| ExpressionError::runtime_error(e)),
            BinaryOperator::Divide => {
                if right.is_zero() {
                    Err(ExpressionError::division_by_zero())
                } else {
                    left.div(right)
                        .map_err(|e| ExpressionError::runtime_error(e))
                }
            }
            BinaryOperator::Mod => left.modulo(right)
                .map_err(|e| ExpressionError::runtime_error(e)),
            // 比较运算
            BinaryOperator::Equal => Ok(Value::Bool(left.equals(right))),
            BinaryOperator::NotEqual => Ok(Value::Bool(!left.equals(right))),
            BinaryOperator::LessThan => Ok(Value::Bool(left.less_than(right))),
            BinaryOperator::LessThanOrEqual => Ok(Value::Bool(left.less_than_equal(right))),
            BinaryOperator::GreaterThan => Ok(Value::Bool(left.greater_than(right))),
            BinaryOperator::GreaterThanOrEqual => Ok(Value::Bool(left.greater_than_equal(right))),
            // 逻辑运算
            BinaryOperator::And => {
                match (left, right) {
                    (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(*l && *r)),
                    _ => Err(ExpressionError::type_error("逻辑运算需要布尔值")),
                }
            }
            BinaryOperator::Or => {
                match (left, right) {
                    (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(*l || *r)),
                    _ => Err(ExpressionError::type_error("逻辑运算需要布尔值")),
                }
            }
        }
    }
}
```

#### 2.1.2 实现类型转换系统

**目标**：支持所有必要的类型转换

**实现方案**：
```rust
impl ExpressionEvaluator {
    fn eval_type_cast(
        &self,
        value: &Value,
        target_type: &DataType,
    ) -> Result<Value, ExpressionError> {
        match target_type {
            DataType::Bool => value.cast_to_bool()
                .map_err(|e| ExpressionError::type_error(e)),
            DataType::Int => value.cast_to_int()
                .map_err(|e| ExpressionError::type_error(e)),
            DataType::Float => value.cast_to_float()
                .map_err(|e| ExpressionError::type_error(e)),
            DataType::String => value.cast_to_string()
                .map_err(|e| ExpressionError::type_error(e)),
            DataType::List => value.cast_to_list()
                .map_err(|e| ExpressionError::type_error(e)),
            DataType::Map => value.cast_to_map()
                .map_err(|e| ExpressionError::type_error(e)),
            _ => Err(ExpressionError::type_error(
                format!("不支持的类型转换: {:?}", target_type)
            )),
        }
    }
}
```

#### 2.1.3 实现属性访问机制

**目标**：支持顶点、边、映射的属性访问

**实现方案**：
```rust
impl ExpressionEvaluator {
    fn eval_property_access(
        &self,
        object: &Value,
        property: &str,
    ) -> Result<Value, ExpressionError> {
        match object {
            Value::Vertex(vertex) => {
                vertex.properties.get(property)
                    .cloned()
                    .ok_or_else(|| ExpressionError::runtime_error(
                        format!("顶点属性不存在: {}", property)
                    ))
            }
            Value::Edge(edge) => {
                edge.properties.get(property)
                    .cloned()
                    .ok_or_else(|| ExpressionError::runtime_error(
                        format!("边属性不存在: {}", property)
                    ))
            }
            Value::Map(map) => {
                map.get(property)
                    .cloned()
                    .ok_or_else(|| ExpressionError::runtime_error(
                        format!("映射键不存在: {}", property)
                    ))
            }
            Value::List(list) => {
                // 支持数字索引访问
                if let Ok(index) = property.parse::<isize>() {
                    let adjusted_index = if index < 0 {
                        list.len() as isize + index
                    } else {
                        index
                    };
                    
                    if adjusted_index >= 0 && adjusted_index < list.len() as isize {
                        Ok(list[adjusted_index as usize].clone())
                    } else {
                        Err(ExpressionError::index_out_of_bounds(
                            adjusted_index, list.len()
                        ))
                    }
                } else {
                    Err(ExpressionError::type_error(
                        "列表索引必须是整数"
                    ))
                }
            }
            _ => Err(ExpressionError::type_error(
                "不支持属性访问的类型"
            )),
        }
    }
}
```

### 2.2 阶段二：性能优化（优先级：中）

#### 2.2.1 表达式编译器

**目标**：将表达式编译为字节码以提高执行效率

**实现方案**：
```rust
#[derive(Debug, Clone)]
pub struct CompiledExpression {
    pub bytecode: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub variables: Vec<String>,
    pub functions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum OpCode {
    LoadConstant(usize),
    LoadVariable(String),
    BinaryOp(BinaryOperator),
    UnaryOp(UnaryOperator),
    PropertyAccess(String),
    TypeCast(DataType),
    FunctionCall(String, usize),
    JumpIfFalse(usize),
    Jump(usize),
    Return,
}

pub struct ExpressionCompiler {
    bytecode: Vec<OpCode>,
    constants: Vec<Value>,
    variables: Vec<String>,
    functions: Vec<String>,
}

impl ExpressionCompiler {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            constants: Vec::new(),
            variables: Vec::new(),
            functions: Vec::new(),
        }
    }
    
    pub fn compile(&mut self, expr: &Expression) -> Result<CompiledExpression, ExpressionError> {
        self.compile_expression(expr)?;
        Ok(CompiledExpression {
            bytecode: self.bytecode.clone(),
            constants: self.constants.clone(),
            variables: self.variables.clone(),
            functions: self.functions.clone(),
        })
    }
    
    fn compile_expression(&mut self, expr: &Expression) -> Result<(), ExpressionError> {
        match expr {
            Expression::Literal(value) => {
                let index = self.add_constant(value.clone());
                self.bytecode.push(OpCode::LoadConstant(index));
            }
            Expression::Variable(name) => {
                self.add_variable(name.clone());
                self.bytecode.push(OpCode::LoadVariable(name.clone()));
            }
            Expression::Binary { left, op, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                self.bytecode.push(OpCode::BinaryOp(op.clone()));
            }
            // ... 其他表达式类型
            _ => return Err(ExpressionError::runtime_error(
                format!("不支持编译的表达式类型: {:?}", expr)
            )),
        }
        Ok(())
    }
    
    fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
    
    fn add_variable(&mut self, name: String) {
        if !self.variables.contains(&name) {
            self.variables.push(name);
        }
    }
}
```

#### 2.2.2 常量折叠优化

**目标**：在编译时计算常量表达式

**实现方案**：
```rust
pub struct ConstantFoldingOptimizer;

impl ExpressionOptimizer for ConstantFoldingOptimizer {
    fn optimize(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Binary { left, op, right } => {
                let optimized_left = self.optimize(left);
                let optimized_right = self.optimize(right);
                
                if let (Expression::Literal(l1), Expression::Literal(l2)) = (&optimized_left, &optimized_right) {
                    // 常量折叠
                    if let Ok(result) = self.eval_constant_binary(l1, op, l2) {
                        return Expression::Literal(result);
                    }
                }
                
                Expression::Binary {
                    left: Box::new(optimized_left),
                    op: op.clone(),
                    right: Box::new(optimized_right),
                }
            }
            Expression::Unary { op, expr } => {
                let optimized_expr = self.optimize(expr);
                
                if let Expression::Literal(value) = &optimized_expr {
                    // 常量折叠
                    if let Ok(result) = self.eval_constant_unary(op, value) {
                        return Expression::Literal(result);
                    }
                }
                
                Expression::Unary {
                    op: op.clone(),
                    expr: Box::new(optimized_expr),
                }
            }
            _ => expr.clone(),
        }
    }
    
    fn eval_constant_binary(
        &self,
        left: &LiteralValue,
        op: &BinaryOperator,
        right: &LiteralValue,
    ) -> Result<LiteralValue, String> {
        let left_value = Value::from(left.clone());
        let right_value = Value::from(right.clone());
        
        let result = match op {
            BinaryOperator::Add => left_value.add(&right_value),
            BinaryOperator::Subtract => left_value.sub(&right_value),
            BinaryOperator::Multiply => left_value.mul(&right_value),
            BinaryOperator::Divide => left_value.div(&right_value),
            _ => return Err("不支持的常量运算".to_string()),
        };
        
        match result {
            Value::Int(i) => Ok(LiteralValue::Int(i)),
            Value::Float(f) => Ok(LiteralValue::Float(f)),
            Value::Bool(b) => Ok(LiteralValue::Bool(b)),
            Value::String(s) => Ok(LiteralValue::String(s)),
            _ => Err("不支持的常量类型".to_string()),
        }
    }
}
```

### 2.3 阶段三：高级功能（优先级：低）

#### 2.3.1 函数调用系统

**目标**：实现完整的函数注册和调用机制

**实现方案**：
```rust
pub trait ExpressionFunction: Send + Sync {
    fn name(&self) -> &str;
    fn arity(&self) -> usize;
    fn is_variadic(&self) -> bool;
    fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError>;
    fn description(&self) -> &str;
}

pub struct FunctionRegistry {
    functions: HashMap<String, Box<dyn ExpressionFunction>>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }
    
    pub fn register<F>(&mut self, function: F) 
    where 
        F: ExpressionFunction + 'static
    {
        self.functions.insert(function.name().to_string(), Box::new(function));
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn ExpressionFunction> {
        self.functions.get(name).map(|f| f.as_ref())
    }
    
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, ExpressionError> {
        let function = self.get(name)
            .ok_or_else(|| ExpressionError::undefined_function(name))?;
            
        if !function.is_variadic() && args.len() != function.arity() {
            return Err(ExpressionError::argument_count_error(
                function.arity(), 
                args.len()
            ));
        }
        
        function.execute(args)
    }
}

// 内置函数实现
pub struct AbsFunction;

impl ExpressionFunction for AbsFunction {
    fn name(&self) -> &str {
        "abs"
    }
    
    fn arity(&self) -> usize {
        1
    }
    
    fn is_variadic(&self) -> bool {
        false
    }
    
    fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        if args.len() != 1 {
            return Err(ExpressionError::argument_count_error(1, args.len()));
        }
        
        match &args[0] {
            Value::Int(i) => Ok(Value::Int(i64::abs(*i))),
            Value::Float(f) => Ok(Value::Float(f64::abs(*f))),
            _ => Err(ExpressionError::type_error("abs函数需要数值参数")),
        }
    }
    
    fn description(&self) -> &str {
        "计算绝对值"
    }
}
```

#### 2.3.2 性能监控系统

**目标**：提供详细的性能指标和监控

**实现方案**：
```rust
#[derive(Debug, Clone)]
pub struct ExpressionPerformanceMetrics {
    pub evaluator_name: String,
    pub total_evaluations: usize,
    pub total_evaluation_time_us: u64,
    pub average_evaluation_time_us: f64,
    pub min_evaluation_time_us: u64,
    pub max_evaluation_time_us: u64,
    pub successful_evaluations: usize,
    pub failed_evaluations: usize,
    pub success_rate: f64,
    pub cache_hit_rate: f64,
    pub memory_usage_bytes: usize,
    pub expression_type_counts: HashMap<String, usize>,
}

impl ExpressionPerformanceMetrics {
    pub fn new(evaluator_name: impl Into<String>) -> Self {
        Self {
            evaluator_name: evaluator_name.into(),
            total_evaluations: 0,
            total_evaluation_time_us: 0,
            average_evaluation_time_us: 0.0,
            min_evaluation_time_us: u64::MAX,
            max_evaluation_time_us: 0,
            successful_evaluations: 0,
            failed_evaluations: 0,
            success_rate: 0.0,
            cache_hit_rate: 0.0,
            memory_usage_bytes: 0,
            expression_type_counts: HashMap::new(),
        }
    }
    
    pub fn record_evaluation(
        &mut self, 
        expression: &Expression, 
        evaluation_time_us: u64, 
        success: bool,
        cache_hit: bool
    ) {
        self.total_evaluations += 1;
        self.total_evaluation_time_us += evaluation_time_us;
        
        if evaluation_time_us < self.min_evaluation_time_us {
            self.min_evaluation_time_us = evaluation_time_us;
        }
        
        if evaluation_time_us > self.max_evaluation_time_us {
            self.max_evaluation_time_us = evaluation_time_us;
        }
        
        if success {
            self.successful_evaluations += 1;
        } else {
            self.failed_evaluations += 1;
        }
        
        // 更新表达式类型计数
        let type_name = format!("{:?}", expression.expression_type());
        *self.expression_type_counts.entry(type_name).or_insert(0) += 1;
        
        // 更新统计信息
        self.average_evaluation_time_us = 
            self.total_evaluation_time_us as f64 / self.total_evaluations as f64;
        
        if self.total_evaluations > 0 {
            self.success_rate = self.successful_evaluations as f64 / self.total_evaluations as f64;
        }
        
        // 更新缓存命中率（简化实现）
        if cache_hit {
            self.cache_hit_rate = (self.cache_hit_rate * (self.total_evaluations - 1) as f64 + 1.0) 
                / self.total_evaluations as f64;
        } else {
            self.cache_hit_rate = (self.cache_hit_rate * (self.total_evaluations - 1) as f64) 
                / self.total_evaluations as f64;
        }
    }
    
    pub fn generate_report(&self) -> String {
        format!(
            "表达式求值器性能报告\n\
             ====================\n\
             求值器名称: {}\n\
             总求值次数: {}\n\
             成功率: {:.2}%\n\
             平均求值时间: {:.2}μs\n\
             最小求值时间: {}μs\n\
             最大求值时间: {}μs\n\
             缓存命中率: {:.2}%\n\
             内存使用量: {}字节\n\
             表达式类型分布:\n\
             {}",
            self.evaluator_name,
            self.total_evaluations,
            self.success_rate * 100.0,
            self.average_evaluation_time_us,
            self.min_evaluation_time_us,
            self.max_evaluation_time_us,
            self.cache_hit_rate * 100.0,
            self.memory_usage_bytes,
            self.expression_type_counts
                .iter()
                .map(|(k, v)| format!("  {}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
```

## 3. 实施计划

### 3.1 第一阶段（1-2周）

1. **完善二元运算**：实现所有基础算术和逻辑运算
2. **实现类型转换**：支持所有必要的类型转换
3. **实现属性访问**：支持顶点、边、映射的属性访问
4. **基础测试**：确保核心功能正确性

### 3.2 第二阶段（2-3周）

1. **表达式编译器**：实现字节码编译和执行
2. **常量折叠优化**：实现编译时优化
3. **性能监控**：添加详细的性能指标
4. **性能测试**：验证优化效果

### 3.3 第三阶段（3-4周）

1. **函数调用系统**：实现完整的函数注册和调用
2. **聚合函数**：实现常用聚合函数
3. **CASE表达式**：实现条件表达式
4. **集成测试**：全面测试所有功能

## 4. 风险评估

### 4.1 技术风险

1. **复杂性增加**：新功能可能增加系统复杂性
2. **性能回归**：某些优化可能影响其他场景的性能
3. **兼容性问题**：API变更可能影响现有代码

### 4.2 缓解措施

1. **渐进式实施**：分阶段实施，每个阶段都有明确的回滚点
2. **全面测试**：每个功能都有对应的单元测试和集成测试
3. **性能基准**：建立性能基准，持续监控性能变化

## 5. 成功指标

### 5.1 功能指标

- [ ] 所有基础运算100%实现
- [ ] 所有类型转换100%实现
- [ ] 属性访问功能100%实现
- [ ] 函数调用系统100%实现

### 5.2 性能指标

- [ ] 表达式求值性能提升50%以上
- [ ] 内存使用量减少30%以上
- [ ] 缓存命中率达到80%以上

### 5.3 质量指标

- [ ] 代码覆盖率达到90%以上
- [ ] 所有公共API都有文档
- [ ] 没有已知的严重bug

## 6. 总结

本改进方案基于对 Nebula-Graph 表达式系统的深入分析，针对当前 Rust 实现的问题提出了全面的解决方案。通过分阶段实施，我们可以在保证系统稳定性的同时，显著提升表达式求值器的功能完整性、性能和可维护性。

改进后的表达式求值器将具备：
1. 完整的核心功能实现
2. 高效的性能优化机制
3. 完善的监控和调试工具
4. 良好的扩展性和可维护性

这将为整个图数据库系统提供坚实的基础，支持更复杂的查询操作和更高的性能要求。