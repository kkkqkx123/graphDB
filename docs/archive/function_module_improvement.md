# 函数模块改进方案

## 1. 背景与问题分析

### 1.1 当前实现状态

当前 GraphDB 的函数模块包含以下核心组件：

- **signature.rs**: 定义 `ValueType` 枚举和 `FunctionSignature` 结构体
- **registry.rs**: 实现 `FunctionRegistry` 函数注册表
- **mod.rs**: 定义 `BuiltinFunction`、`CustomFunction` 等函数类型枚举

### 1.2 与 Nebula-Graph 的主要差距

| 特性 | Nebula-Graph | GraphDB (当前) | 差距等级 |
|------|--------------|----------------|----------|
| 类型签名精度 | 精确到具体类型，支持多签名 | 使用 `Any` 类型 | 严重 |
| 函数重载 | 支持同名多签名 | 不支持 | 严重 |
| 聚合函数管理 | 独立 AggFunctionManager | 与普通函数耦合 | 中等 |
| 返回类型 | 每种签名有精确返回类型 | 多为 `Any` | 中等 |
| UDF 支持 | 动态加载框架 | 功能有限 | 低 |

### 1.3 核心问题

#### 1.3.1 类型签名过于宽泛

当前实现中，大多数函数签名使用 `ValueType::Any` 作为参数类型：

```rust
// 当前实现
FunctionSignature::new("abs", vec![ValueType::Any], ValueType::Any, ...)

// 应改为
FunctionSignature::new("abs", vec![ValueType::Int], ValueType::Int, ...)
FunctionSignature::new("abs", vec![ValueType::Float], ValueType::Float, ...)
```

**影响**：
- 运行时才进行类型检查，错误信息不够精确
- 缺少函数重载支持
- 无法在编译期进行类型推导优化

#### 1.3.2 缺少函数重载机制

`check_types()` 方法只检查是否兼容 `Any` 类型，没有实现真正的函数重载解析。

#### 1.3.3 聚合函数与普通函数耦合

`BuiltinFunction` 枚举将聚合函数与其他函数混在一起，不利于上下文管理和状态追踪。

---

## 2. 修改目标

### 2.1 总体目标

1. **提高类型安全性**: 使用精确的类型签名替代 `Any`
2. **支持函数重载**: 同一函数名支持多个签名
3. **优化架构**: 分离聚合函数管理
4. **保持兼容性**: 不破坏现有 API

### 2.2 具体目标

1. 为所有内置函数定义精确的类型签名
2. 实现签名选择算法，根据参数类型选择最匹配的函数
3. 改进错误信息，提供更精确的类型不匹配提示
4. 预留 UDF 扩展接口

---

## 3. 修改方案

### 3.1 修改 signature.rs

#### 3.1.1 增强 ValueType 枚举

保持现有枚举不变，确保包含所有必要类型。

#### 3.1.2 改进 FunctionSignature

在 `FunctionSignature` 中添加以下方法：

```rust
// 精确类型检查（替代现有的 check_types）
pub fn check_exact_types(&self, args: &[Value]) -> bool {
    if args.len() != self.arg_types.len() {
        return false;
    }
    args.iter().zip(&self.arg_types).all(|(arg, expected)| {
        let actual = ValueType::from_value(arg);
        actual == *expected
    })
}

// 兼容类型检查（用于 Any 类型回退）
pub fn check_compatible_types(&self, args: &[Value]) -> bool {
    args.iter().zip(&self.arg_types).all(|(arg, expected)| {
        if expected == &ValueType::Any {
            return true;
        }
        let actual = ValueType::from_value(arg);
        actual.compatible_with(expected)
    })
}
```

#### 3.1.3 添加函数重载解析器

新增 `FunctionOverloadResolver` 结构体：

```rust
pub struct FunctionOverloadResolver {
    overloads: Vec<RegisteredFunction>,
}

impl FunctionOverloadResolver {
    // 根据参数类型选择最匹配的签名
    pub fn select(&self, args: &[Value]) -> Option<&RegisteredFunction> {
        // 优先级：
        // 1. 精确匹配
        // 2. 兼容匹配（考虑 Any）
        // 3. 返回 None
    }
}
```

### 3.2 修改 registry.rs

#### 3.2.1 改进 register 方法

支持注册多个重载版本：

```rust
// 现有接口保持兼容
pub fn register<F>(&mut self, name: &str, signature: FunctionSignature, func: F)
where
    F: Fn(&[Value]) -> Result<Value, ExpressionError> + 'static + Send + Sync,
{
    // 内部调用 register_overload
}

// 新增：注册重载版本
pub fn register_overload(&mut self, name: &str, signature: FunctionSignature, func: Box<FunctionBody>);
```

#### 3.2.2 改进 execute 方法

实现签名选择逻辑：

```rust
pub fn execute(&self, name: &str, args: &[Value]) -> Result<Value, ExpressionError> {
    let funcs = self.functions.get(name)
        .ok_or_else(|| ExpressionError::undefined_function(name))?;

    // 首先尝试精确类型匹配
    for registered in funcs {
        if registered.signature.check_exact_types(args) {
            return (registered.body)(args);
        }
    }

    // 如果没有精确匹配，尝试兼容匹配
    for registered in funcs {
        if registered.signature.check_compatible_types(args) {
            return (registered.body)(args);
        }
    }

    // 返回类型错误
    Err(ExpressionError::type_mismatch(name, args, funcs))
}
```

### 3.3 修改 mod.rs

#### 3.3.1 分离聚合函数

将聚合函数独立管理：

```rust
// 新增聚合函数模块
pub mod agg;

// 保留 BuiltinFunction 用于表达式引用
pub enum BuiltinFunction {
    Math(MathFunction),
    String(StringFunction),
    Regex(RegexFunction),
    Conversion(ConversionFunction),
    DateTime(DateTimeFunction),
    // 移除 Aggregate
}
```

#### 3.3.2 保留 AggregationFunction 特征

为聚合函数定义独立接口：

```rust
pub trait AggregationFunction: Send + Sync {
    fn name(&self) -> &str;
    fn accumulate(&self, data: &mut AggData, value: &Value);
    fn finalize(&self, data: &AggData) -> Value;
    fn is_distinct(&self) -> bool;
}
```

### 3.4 更新函数签名

#### 3.4.1 数学函数

| 函数 | 签名 | 返回类型 |
|------|------|----------|
| abs | (INT) -> INT | INT |
| abs | (FLOAT) -> FLOAT | FLOAT |
| floor | (INT) -> FLOAT | FLOAT |
| floor | (FLOAT) -> FLOAT | FLOAT |
| ceil | (INT) -> FLOAT | FLOAT |
| ceil | (FLOAT) -> FLOAT | FLOAT |
| round | (INT) -> FLOAT | FLOAT |
| round | (INT, INT) -> FLOAT | FLOAT |
| sqrt | (INT) -> FLOAT | FLOAT |
| sqrt | (FLOAT) -> FLOAT | FLOAT |
| pow | (INT, INT) -> FLOAT | FLOAT |
| pow | (FLOAT, FLOAT) -> FLOAT | FLOAT |
| pow | (INT, FLOAT) -> FLOAT | FLOAT |
| pow | (FLOAT, INT) -> FLOAT | FLOAT |

#### 3.4.2 字符串函数

| 函数 | 签名 | 返回类型 |
|------|------|----------|
| length | (STRING) -> INT | INT |
| upper | (STRING) -> STRING | STRING |
| lower | (STRING) -> STRING | STRING |
| trim | (STRING) -> STRING | STRING |
| concat | (STRING...) -> STRING | STRING |
| substring | (STRING, INT) -> STRING | STRING |
| substring | (STRING, INT, INT) -> STRING | STRING |
| replace | (STRING, STRING, STRING) -> STRING | STRING |

#### 3.4.3 类型转换函数

| 函数 | 签名 | 返回类型 |
|------|------|----------|
| to_string | (INT) -> STRING | STRING |
| to_string | (FLOAT) -> STRING | STRING |
| to_string | (BOOL) -> STRING | STRING |
| to_int | (STRING) -> INT | INT |
| to_int | (FLOAT) -> INT | INT |
| to_float | (STRING) -> FLOAT | FLOAT |
| to_float | (INT) -> FLOAT | FLOAT |

---

## 4. 实施步骤

### 阶段 1: 核心修改

1. 修改 `signature.rs`
   - 添加 `check_exact_types` 和 `check_compatible_types` 方法
   - 添加 `FunctionOverloadResolver` 结构体

2. 修改 `registry.rs`
   - 改进 `execute` 方法实现签名选择
   - 添加错误信息改进

### 阶段 2: 签名更新

3. 更新数学函数签名
   - abs, floor, ceil, round, sqrt, pow 等

4. 更新字符串函数签名
   - length, upper, lower, trim, concat, substring 等

5. 更新转换函数签名
   - to_string, to_int, to_float, to_bool

### 阶段 3: 架构优化

6. 分离聚合函数到独立模块
7. 添加聚合函数特征定义

### 阶段 4: 验证

8. 运行测试确保兼容性
9. 运行 cargo check 确保编译通过
10. 更新文档

---

## 5. 兼容性保证

### 5.1 API 兼容性

- 保持 `FunctionRegistry` 现有公共方法不变
- 保持 `FunctionSignature` 现有字段不变
- 保持 `BuiltinFunction` 枚举现有变体不变

### 5.2 行为兼容性

- 现有函数调用行为保持一致
- 错误信息更加精确但不改变错误类型
- 向后兼容现有测试用例

---

## 6. 预期收益

1. **类型安全提升**: 编译期类型检查更精确
2. **错误信息优化**: 类型不匹配时提供更准确的提示
3. **函数重载**: 支持同名函数多签名
4. **性能优化**: 减少不必要的运行时类型检查
5. **架构清晰**: 聚合函数独立管理

---

## 7. 风险与应对

| 风险 | 影响 | 应对措施 |
|------|------|----------|
| 修改范围过大 | 可能引入回归 | 分阶段实施，每阶段验证 |
| 性能影响 | 签名选择增加开销 | 优化选择算法，使用缓存 |
| API 兼容性 | 破坏现有调用 | 保持公共接口不变 |
