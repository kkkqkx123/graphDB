//! 数学函数实现

use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::Value;
use crate::expression::functions::registry::FunctionRegistry;
use crate::expression::functions::signature::FunctionSignature;
use crate::expression::functions::signature::ValueType;
use rand::Rng;
use std::cell::RefCell;

thread_local! {
    static RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
}

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
    register_bit_and(registry);
    register_bit_or(registry);
    register_bit_xor(registry);
    register_asin(registry);
    register_acos(registry);
    register_atan(registry);
    register_cbrt(registry);
    register_hypot(registry);
    register_sign(registry);
    register_rand(registry);
    register_rand32(registry);
    register_rand64(registry);
    register_e(registry);
    register_pi(registry);
    register_exp2(registry);
    register_log2(registry);
    register_radians(registry);
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

fn register_bit_and(registry: &mut FunctionRegistry) {
    registry.register(
        "bit_and",
        FunctionSignature::new(
            "bit_and",
            vec![ValueType::Int, ValueType::Int],
            ValueType::Int,
            2,
            2,
            true,
            "按位与",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a & b)),
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("bit_and函数需要整数参数")),
            }
        },
    );
}

fn register_bit_or(registry: &mut FunctionRegistry) {
    registry.register(
        "bit_or",
        FunctionSignature::new(
            "bit_or",
            vec![ValueType::Int, ValueType::Int],
            ValueType::Int,
            2,
            2,
            true,
            "按位或",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a | b)),
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("bit_or函数需要整数参数")),
            }
        },
    );
}

fn register_bit_xor(registry: &mut FunctionRegistry) {
    registry.register(
        "bit_xor",
        FunctionSignature::new(
            "bit_xor",
            vec![ValueType::Int, ValueType::Int],
            ValueType::Int,
            2,
            2,
            true,
            "按位异或",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a ^ b)),
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("bit_xor函数需要整数参数")),
            }
        },
    );
}

fn register_asin(registry: &mut FunctionRegistry) {
    registry.register(
        "asin",
        FunctionSignature::new(
            "asin",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算反正弦",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => {
                    let f = *i as f64;
                    if f >= -1.0 && f <= 1.0 {
                        Ok(Value::Float(f.asin()))
                    } else {
                        Ok(Value::Null(crate::core::value::NullType::NaN))
                    }
                }
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("asin函数需要整数类型")),
            }
        },
    );

    registry.register(
        "asin",
        FunctionSignature::new(
            "asin",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算反正弦",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => {
                    if *f >= -1.0 && *f <= 1.0 {
                        Ok(Value::Float(f.asin()))
                    } else {
                        Ok(Value::Null(crate::core::value::NullType::NaN))
                    }
                }
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("asin函数需要浮点数类型")),
            }
        },
    );
}

fn register_acos(registry: &mut FunctionRegistry) {
    registry.register(
        "acos",
        FunctionSignature::new(
            "acos",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算反余弦",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => {
                    let f = *i as f64;
                    if f >= -1.0 && f <= 1.0 {
                        Ok(Value::Float(f.acos()))
                    } else {
                        Ok(Value::Null(crate::core::value::NullType::NaN))
                    }
                }
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("acos函数需要整数类型")),
            }
        },
    );

    registry.register(
        "acos",
        FunctionSignature::new(
            "acos",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算反余弦",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => {
                    if *f >= -1.0 && *f <= 1.0 {
                        Ok(Value::Float(f.acos()))
                    } else {
                        Ok(Value::Null(crate::core::value::NullType::NaN))
                    }
                }
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("acos函数需要浮点数类型")),
            }
        },
    );
}

fn register_atan(registry: &mut FunctionRegistry) {
    registry.register(
        "atan",
        FunctionSignature::new(
            "atan",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算反正切",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(((*i) as f64).atan())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("atan函数需要整数类型")),
            }
        },
    );

    registry.register(
        "atan",
        FunctionSignature::new(
            "atan",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算反正切",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.atan())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("atan函数需要浮点数类型")),
            }
        },
    );
}

fn register_cbrt(registry: &mut FunctionRegistry) {
    registry.register(
        "cbrt",
        FunctionSignature::new(
            "cbrt",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算立方根",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(((*i) as f64).cbrt())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("cbrt函数需要整数类型")),
            }
        },
    );

    registry.register(
        "cbrt",
        FunctionSignature::new(
            "cbrt",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算立方根",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.cbrt())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("cbrt函数需要浮点数类型")),
            }
        },
    );
}

fn register_hypot(registry: &mut FunctionRegistry) {
    registry.register(
        "hypot",
        FunctionSignature::new(
            "hypot",
            vec![ValueType::Int, ValueType::Int],
            ValueType::Float,
            2,
            2,
            true,
            "计算直角三角形斜边长度",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Float((*a as f64).hypot(*b as f64))),
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("hypot函数需要整数参数")),
            }
        },
    );

    registry.register(
        "hypot",
        FunctionSignature::new(
            "hypot",
            vec![ValueType::Float, ValueType::Float],
            ValueType::Float,
            2,
            2,
            true,
            "计算直角三角形斜边长度",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.hypot(*b))),
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("hypot函数需要浮点数参数")),
            }
        },
    );

    registry.register(
        "hypot",
        FunctionSignature::new(
            "hypot",
            vec![ValueType::Int, ValueType::Float],
            ValueType::Float,
            2,
            2,
            true,
            "计算直角三角形斜边长度",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).hypot(*b))),
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("hypot函数需要数值参数")),
            }
        },
    );

    registry.register(
        "hypot",
        FunctionSignature::new(
            "hypot",
            vec![ValueType::Float, ValueType::Int],
            ValueType::Float,
            2,
            2,
            true,
            "计算直角三角形斜边长度",
        ),
        |args| {
            match (&args[0], &args[1]) {
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.hypot(*b as f64))),
                (Value::Null(_), _) | (_, Value::Null(_)) => {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
                _ => Err(ExpressionError::type_error("hypot函数需要数值参数")),
            }
        },
    );
}

fn register_sign(registry: &mut FunctionRegistry) {
    registry.register(
        "sign",
        FunctionSignature::new(
            "sign",
            vec![ValueType::Int],
            ValueType::Int,
            1,
            1,
            true,
            "返回数值的符号（-1, 0, 1）",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Int(i.signum())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("sign函数需要整数类型")),
            }
        },
    );

    registry.register(
        "sign",
        FunctionSignature::new(
            "sign",
            vec![ValueType::Float],
            ValueType::Int,
            1,
            1,
            true,
            "返回数值的符号（-1, 0, 1）",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Int(f.signum() as i64)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("sign函数需要浮点数类型")),
            }
        },
    );
}

fn register_rand(registry: &mut FunctionRegistry) {
    registry.register(
        "rand",
        FunctionSignature::new(
            "rand",
            vec![],
            ValueType::Float,
            0,
            0,
            false,
            "返回0到1之间的随机浮点数",
        ),
        |_args| {
            let value = RNG.with(|rng| rng.borrow_mut().gen::<f64>());
            Ok(Value::Float(value))
        },
    );
}

fn register_rand32(registry: &mut FunctionRegistry) {
    registry.register(
        "rand32",
        FunctionSignature::new(
            "rand32",
            vec![],
            ValueType::Int,
            0,
            2,
            true,
            "返回32位随机整数，可选指定范围",
        ),
        |args| {
            let (min, max) = match args.len() {
                0 => (0i64, i32::MAX as i64),
                1 => match &args[0] {
                    Value::Int(max) => (0i64, *max),
                    Value::Null(_) => return Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => return Err(ExpressionError::type_error("rand32函数参数需要整数类型")),
                },
                2 => match (&args[0], &args[1]) {
                    (Value::Int(min), Value::Int(max)) => (*min, *max),
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        return Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => return Err(ExpressionError::type_error("rand32函数参数需要整数类型")),
                },
                _ => return Err(ExpressionError::type_error("rand32函数参数数量错误")),
            };

            if min >= max {
                return Err(ExpressionError::type_error("rand32函数最小值必须小于最大值"));
            }

            let value = RNG.with(|rng| rng.borrow_mut().gen_range(min..max));
            Ok(Value::Int(value))
        },
    );
}

fn register_rand64(registry: &mut FunctionRegistry) {
    registry.register(
        "rand64",
        FunctionSignature::new(
            "rand64",
            vec![],
            ValueType::Int,
            0,
            2,
            true,
            "返回64位随机整数，可选指定范围",
        ),
        |args| {
            let (min, max) = match args.len() {
                0 => (i64::MIN, i64::MAX),
                1 => match &args[0] {
                    Value::Int(max) => (0i64, *max),
                    Value::Null(_) => return Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => return Err(ExpressionError::type_error("rand64函数参数需要整数类型")),
                },
                2 => match (&args[0], &args[1]) {
                    (Value::Int(min), Value::Int(max)) => (*min, *max),
                    (Value::Null(_), _) | (_, Value::Null(_)) => {
                        return Ok(Value::Null(crate::core::value::NullType::Null))
                    }
                    _ => return Err(ExpressionError::type_error("rand64函数参数需要整数类型")),
                },
                _ => return Err(ExpressionError::type_error("rand64函数参数数量错误")),
            };

            if min >= max {
                return Err(ExpressionError::type_error("rand64函数最小值必须小于最大值"));
            }

            let value = RNG.with(|rng| rng.borrow_mut().gen_range(min..max));
            Ok(Value::Int(value))
        },
    );
}

fn register_e(registry: &mut FunctionRegistry) {
    registry.register(
        "e",
        FunctionSignature::new(
            "e",
            vec![],
            ValueType::Float,
            0,
            0,
            true,
            "自然常数 e",
        ),
        |_args| Ok(Value::Float(std::f64::consts::E)),
    );
}

fn register_pi(registry: &mut FunctionRegistry) {
    registry.register(
        "pi",
        FunctionSignature::new(
            "pi",
            vec![],
            ValueType::Float,
            0,
            0,
            true,
            "圆周率 π",
        ),
        |_args| Ok(Value::Float(std::f64::consts::PI)),
    );
}

fn register_exp2(registry: &mut FunctionRegistry) {
    registry.register(
        "exp2",
        FunctionSignature::new(
            "exp2",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算2的幂",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(((*i) as f64).exp2())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("exp2函数需要整数类型")),
            }
        },
    );

    registry.register(
        "exp2",
        FunctionSignature::new(
            "exp2",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算2的幂",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.exp2())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("exp2函数需要浮点数类型")),
            }
        },
    );
}

fn register_log2(registry: &mut FunctionRegistry) {
    registry.register(
        "log2",
        FunctionSignature::new(
            "log2",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "计算以2为底的对数",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) if *i > 0 => Ok(Value::Float(((*i) as f64).log2())),
                Value::Int(_) => Ok(Value::Null(crate::core::value::NullType::NaN)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("log2函数需要整数类型")),
            }
        },
    );

    registry.register(
        "log2",
        FunctionSignature::new(
            "log2",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "计算以2为底的对数",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) if *f > 0.0 => Ok(Value::Float(f.log2())),
                Value::Float(_) => Ok(Value::Null(crate::core::value::NullType::NaN)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("log2函数需要浮点数类型")),
            }
        },
    );
}

fn register_radians(registry: &mut FunctionRegistry) {
    registry.register(
        "radians",
        FunctionSignature::new(
            "radians",
            vec![ValueType::Int],
            ValueType::Float,
            1,
            1,
            true,
            "将角度转换为弧度",
        ),
        |args| {
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(((*i) as f64).to_radians())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("radians函数需要整数类型")),
            }
        },
    );

    registry.register(
        "radians",
        FunctionSignature::new(
            "radians",
            vec![ValueType::Float],
            ValueType::Float,
            1,
            1,
            true,
            "将角度转换为弧度",
        ),
        |args| {
            match &args[0] {
                Value::Float(f) => Ok(Value::Float(f.to_radians())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("radians函数需要浮点数类型")),
            }
        },
    );
}
