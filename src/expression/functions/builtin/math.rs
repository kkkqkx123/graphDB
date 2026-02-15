//! 数学函数实现

use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::Value;
use crate::expression::functions::registry::FunctionRegistry;
use crate::expression::functions::signature::FunctionSignature;
use crate::expression::functions::signature::ValueType;

/// 注册所有数学函数
pub fn register_all(registry: &mut FunctionRegistry) {
    register_abs(registry);
    register_ceil(registry);
    register_floor(registry);
    register_round(registry);
    register_sqrt(registry);
    register_pow(registry);
    register_exp(registry);
    register_log(registry);
    register_log10(registry);
    register_sin(registry);
    register_cos(registry);
    register_tan(registry);
}

fn register_abs(registry: &mut FunctionRegistry) {
    // abs - INT 版本
    registry.register(
        "abs",
        FunctionSignature::new(
            "abs",
            vec![ValueType::Int],
            ValueType::Int,
            1,
            1,
            true,
            "计算绝对值",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Int(i.abs())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("abs函数需要整数类型")),
            }
        },
    );

    // abs - FLOAT 版本
    registry.register(
        "abs",
        FunctionSignature::new(
            "abs",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算绝对值",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.abs())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("abs函数需要浮点数类型")),
            }
        },
    );
}

fn register_ceil(registry: &mut FunctionRegistry) {
    // ceil - INT 版本
    registry.register(
        "ceil",
        FunctionSignature::new(
            "ceil",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "向上取整",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(*i as f64)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("ceil函数需要整数类型")),
            }
        },
    );

    // ceil - FLOAT 版本
    registry.register(
        "ceil",
        FunctionSignature::new(
            "ceil",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "向上取整",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.ceil())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("ceil函数需要浮点数类型")),
            }
        },
    );
}

fn register_floor(registry: &mut FunctionRegistry) {
    // floor - INT 版本
    registry.register(
        "floor",
        FunctionSignature::new(
            "floor",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "向下取整",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(*i as f64)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("floor函数需要整数类型")),
            }
        },
    );

    // floor - FLOAT 版本
    registry.register(
        "floor",
        FunctionSignature::new(
            "floor",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "向下取整",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.floor())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("floor函数需要浮点数类型")),
            }
        },
    );
}

fn register_round(registry: &mut FunctionRegistry) {
    // round - INT 版本
    registry.register(
        "round",
        FunctionSignature::new(
            "round",
            vec![ValueType::Int],
            ValueType::Int,
            1,
            1,
            true,
            "四舍五入",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Int(*i)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("round函数需要整数类型")),
            }
        },
    );

    // round - FLOAT 版本
    registry.register(
        "round",
        FunctionSignature::new(
            "round",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "四舍五入",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.round())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("round函数需要浮点数类型")),
            }
        },
    );
}

fn register_sqrt(registry: &mut FunctionRegistry) {
    // sqrt - INT 版本
    registry.register(
        "sqrt",
        FunctionSignature::new(
            "sqrt",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算平方根",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) if *i >= 0 => Ok(Value::Float(((*i) as f64).sqrt())),
                Value::Int(i) if *i < 0 => Err(ExpressionError::new(
                    ExpressionErrorType::InvalidOperation,
                    "sqrt of negative number".to_string(),
                )),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("sqrt函数需要整数类型")),
            }
        },
    );

    // sqrt - FLOAT 版本
    registry.register(
        "sqrt",
        FunctionSignature::new(
            "sqrt",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算平方根",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) if *f >= 0.0 => Ok(Value::Float(f.sqrt())),
                Value::Float(f) if *f < 0.0 => Err(ExpressionError::new(
                    ExpressionErrorType::InvalidOperation,
                    "sqrt of negative number".to_string(),
                )),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("sqrt函数需要浮点数类型")),
            }
        },
    );
}

fn register_pow(registry: &mut FunctionRegistry) {
    // pow - INT, INT 版本
    registry.register(
        "pow",
        FunctionSignature::new(
            "pow",
            vec![ValueType::Int, ValueType::Int],
            ValueType::Float,
            2,
            2,
            true,
            "计算幂",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Int(base), Value::Int(exp)) => {
                    Ok(Value::Float(((*base) as f64).powf(*exp as f64)))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("pow函数需要整数类型")),
            }
        },
    );

    // pow - FLOAT, FLOAT 版本
    registry.register(
        "pow",
        FunctionSignature::new(
            "pow",
            vec![ValueType::Float, ValueType::Float],
            ValueType::Float,
            2,
            2,
            true,
            "计算幂",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Float(base), Value::Float(exp)) => {
                    Ok(Value::Float(base.powf(*exp)))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("pow函数需要浮点数类型")),
            }
        },
    );

    // pow - INT, FLOAT 版本
    registry.register(
        "pow",
        FunctionSignature::new(
            "pow",
            vec![ValueType::Int, ValueType::Float],
            ValueType::Float,
            2,
            2,
            true,
            "计算幂",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Int(base), Value::Float(exp)) => {
                    Ok(Value::Float(((*base) as f64).powf(*exp)))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("pow函数需要整数和浮点数类型")),
            }
        },
    );

    // pow - FLOAT, INT 版本
    registry.register(
        "pow",
        FunctionSignature::new(
            "pow",
            vec![ValueType::Float, ValueType::Int],
            ValueType::Float,
            2,
            2,
            true,
            "计算幂",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Float(base), Value::Int(exp)) => {
                    Ok(Value::Float(base.powf(*exp as f64)))
                }
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("pow函数需要浮点数和整数类型")),
            }
        },
    );
}

fn register_exp(registry: &mut FunctionRegistry) {
    // exp - INT 版本
    registry.register(
        "exp",
        FunctionSignature::new(
            "exp",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算指数",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(((*i) as f64).exp())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("exp函数需要整数类型")),
            }
        },
    );

    // exp - FLOAT 版本
    registry.register(
        "exp",
        FunctionSignature::new(
            "exp",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算指数",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.exp())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("exp函数需要浮点数类型")),
            }
        },
    );
}

fn register_log(registry: &mut FunctionRegistry) {
    // log - INT 版本
    registry.register(
        "log",
        FunctionSignature::new(
            "log",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算自然对数",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) if *i > 0 => Ok(Value::Float(((*i) as f64).ln())),
                Value::Int(i) if *i <= 0 => Err(ExpressionError::new(
                    ExpressionErrorType::InvalidOperation,
                    "log of non-positive number".to_string(),
                )),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("log函数需要整数类型")),
            }
        },
    );

    // log - FLOAT 版本
    registry.register(
        "log",
        FunctionSignature::new(
            "log",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算自然对数",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) if *f > 0.0 => Ok(Value::Float(f.ln())),
                Value::Float(f) if *f <= 0.0 => Err(ExpressionError::new(
                    ExpressionErrorType::InvalidOperation,
                    "log of non-positive number".to_string(),
                )),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("log函数需要浮点数类型")),
            }
        },
    );
}

fn register_log10(registry: &mut FunctionRegistry) {
    // log10 - INT 版本
    registry.register(
        "log10",
        FunctionSignature::new(
            "log10",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算以10为底的对数",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) if *i > 0 => Ok(Value::Float(((*i) as f64).log10())),
                Value::Int(i) if *i <= 0 => Err(ExpressionError::new(
                    ExpressionErrorType::InvalidOperation,
                    "log10 of non-positive number".to_string(),
                )),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("log10函数需要整数类型")),
            }
        },
    );

    // log10 - FLOAT 版本
    registry.register(
        "log10",
        FunctionSignature::new(
            "log10",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算以10为底的对数",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) if *f > 0.0 => Ok(Value::Float(f.log10())),
                Value::Float(f) if *f <= 0.0 => Err(ExpressionError::new(
                    ExpressionErrorType::InvalidOperation,
                    "log10 of non-positive number".to_string(),
                )),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("log10函数需要浮点数类型")),
            }
        },
    );
}

fn register_sin(registry: &mut FunctionRegistry) {
    // sin - INT 版本
    registry.register(
        "sin",
        FunctionSignature::new(
            "sin",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算正弦",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(((*i) as f64).sin())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("sin函数需要整数类型")),
            }
        },
    );

    // sin - FLOAT 版本
    registry.register(
        "sin",
        FunctionSignature::new(
            "sin",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算正弦",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.sin())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("sin函数需要浮点数类型")),
            }
        },
    );
}

fn register_cos(registry: &mut FunctionRegistry) {
    // cos - INT 版本
    registry.register(
        "cos",
        FunctionSignature::new(
            "cos",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算余弦",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(((*i) as f64).cos())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("cos函数需要整数类型")),
            }
        },
    );

    // cos - FLOAT 版本
    registry.register(
        "cos",
        FunctionSignature::new(
            "cos",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算余弦",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.cos())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("cos函数需要浮点数类型")),
            }
        },
    );
}

fn register_tan(registry: &mut FunctionRegistry) {
    // tan - INT 版本
    registry.register(
        "tan",
        FunctionSignature::new(
            "tan",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算正切",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(((*i) as f64).tan())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("tan函数需要整数类型")),
            }
        },
    );

    // tan - FLOAT 版本
    registry.register(
        "tan",
        FunctionSignature::new(
            "tan",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算正切",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.tan())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("tan函数需要浮点数类型")),
            }
        },
    );
}
