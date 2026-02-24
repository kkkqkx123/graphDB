//! 数学函数实现

use crate::core::error::ExpressionError;
use crate::core::value::NullType;
use crate::core::Value;
use crate::define_function_enum;
use crate::define_unary_float_fn;
use crate::define_unary_numeric_fn;
use crate::define_binary_numeric_fn;

define_function_enum! {
    /// 数学函数枚举
    pub enum MathFunction {
        Abs => {
            name: "abs",
            arity: 1,
            variadic: false,
            description: "计算绝对值",
            handler: execute_abs
        },
        Sqrt => {
            name: "sqrt",
            arity: 1,
            variadic: false,
            description: "计算平方根",
            handler: execute_sqrt
        },
        Pow => {
            name: "pow",
            arity: 2,
            variadic: false,
            description: "计算幂",
            handler: execute_pow
        },
        Log => {
            name: "log",
            arity: 2,
            variadic: false,
            description: "计算对数",
            handler: execute_log
        },
        Log10 => {
            name: "log10",
            arity: 1,
            variadic: false,
            description: "计算以10为底的对数",
            handler: execute_log10
        },
        Sin => {
            name: "sin",
            arity: 1,
            variadic: false,
            description: "计算正弦",
            handler: execute_sin
        },
        Cos => {
            name: "cos",
            arity: 1,
            variadic: false,
            description: "计算余弦",
            handler: execute_cos
        },
        Tan => {
            name: "tan",
            arity: 1,
            variadic: false,
            description: "计算正切",
            handler: execute_tan
        },
        Round => {
            name: "round",
            arity: 1,
            variadic: false,
            description: "四舍五入",
            handler: execute_round
        },
        Ceil => {
            name: "ceil",
            arity: 1,
            variadic: false,
            description: "向上取整",
            handler: execute_ceil
        },
        Floor => {
            name: "floor",
            arity: 1,
            variadic: false,
            description: "向下取整",
            handler: execute_floor
        },
        Asin => {
            name: "asin",
            arity: 1,
            variadic: false,
            description: "计算反正弦",
            handler: execute_asin
        },
        Acos => {
            name: "acos",
            arity: 1,
            variadic: false,
            description: "计算反余弦",
            handler: execute_acos
        },
        Atan => {
            name: "atan",
            arity: 1,
            variadic: false,
            description: "计算反正切",
            handler: execute_atan
        },
        Cbrt => {
            name: "cbrt",
            arity: 1,
            variadic: false,
            description: "计算立方根",
            handler: execute_cbrt
        },
        Hypot => {
            name: "hypot",
            arity: 2,
            variadic: false,
            description: "计算直角三角形斜边",
            handler: execute_hypot
        },
        Sign => {
            name: "sign",
            arity: 1,
            variadic: false,
            description: "返回数值符号",
            handler: execute_sign
        },
        Rand => {
            name: "rand",
            arity: 0,
            variadic: false,
            description: "生成随机浮点数",
            handler: execute_rand
        },
        Rand32 => {
            name: "rand32",
            arity: 0,
            variadic: true,
            description: "生成32位随机整数",
            handler: execute_rand32
        },
        Rand64 => {
            name: "rand64",
            arity: 0,
            variadic: false,
            description: "生成64位随机整数",
            handler: execute_rand64
        },
        E => {
            name: "e",
            arity: 0,
            variadic: false,
            description: "返回自然常数e",
            handler: execute_e
        },
        Pi => {
            name: "pi",
            arity: 0,
            variadic: false,
            description: "返回圆周率pi",
            handler: execute_pi
        },
        Exp2 => {
            name: "exp2",
            arity: 1,
            variadic: false,
            description: "计算2的幂",
            handler: execute_exp2
        },
        Log2 => {
            name: "log2",
            arity: 1,
            variadic: false,
            description: "计算以2为底的对数",
            handler: execute_log2
        },
        Radians => {
            name: "radians",
            arity: 1,
            variadic: false,
            description: "角度转弧度",
            handler: execute_radians
        },
        BitAnd => {
            name: "bit_and",
            arity: 2,
            variadic: false,
            description: "按位与",
            handler: execute_bit_and
        },
        BitOr => {
            name: "bit_or",
            arity: 2,
            variadic: false,
            description: "按位或",
            handler: execute_bit_or
        },
        BitXor => {
            name: "bit_xor",
            arity: 2,
            variadic: false,
            description: "按位异或",
            handler: execute_bit_xor
        },
    }
}

define_unary_numeric_fn!(
    execute_abs,
    int: |i: i64| Ok(Value::Int(i.abs())),
    float: |f: f64| Ok(Value::Float(f.abs())),
    "abs"
);

define_unary_float_fn!(execute_sqrt, |v: f64| v.sqrt(), "sqrt");
define_unary_float_fn!(execute_sin, |v: f64| v.sin(), "sin");
define_unary_float_fn!(execute_cos, |v: f64| v.cos(), "cos");
define_unary_float_fn!(execute_tan, |v: f64| v.tan(), "tan");
define_unary_float_fn!(execute_log10, |v: f64| v.log10(), "log10");

define_unary_numeric_fn!(
    execute_round,
    int: |i: i64| Ok(Value::Int(i)),
    float: |f: f64| Ok(Value::Float(f.round())),
    "round"
);

define_unary_numeric_fn!(
    execute_ceil,
    int: |i: i64| Ok(Value::Float(i as f64)),
    float: |f: f64| Ok(Value::Float(f.ceil())),
    "ceil"
);

define_unary_numeric_fn!(
    execute_floor,
    int: |i: i64| Ok(Value::Float(i as f64)),
    float: |f: f64| Ok(Value::Float(f.floor())),
    "floor"
);

define_binary_numeric_fn!(
    execute_pow,
    |a: f64, b: f64| Ok(Value::Float(a.powf(b))),
    "pow"
);

define_binary_numeric_fn!(
    execute_log,
    |base: f64, val: f64| Ok(Value::Float(val.log(base))),
    "log"
);

// 新增数学函数实现
define_unary_float_fn!(execute_asin, |v: f64| v.asin(), "asin");
define_unary_float_fn!(execute_acos, |v: f64| v.acos(), "acos");
define_unary_float_fn!(execute_atan, |v: f64| v.atan(), "atan");
define_unary_float_fn!(execute_cbrt, |v: f64| v.cbrt(), "cbrt");
define_unary_float_fn!(execute_exp2, |v: f64| v.exp2(), "exp2");
define_unary_float_fn!(execute_log2, |v: f64| v.log2(), "log2");
define_unary_float_fn!(execute_radians, |v: f64| v.to_radians(), "radians");

define_binary_numeric_fn!(
    execute_hypot,
    |a: f64, b: f64| Ok(Value::Float(a.hypot(b))),
    "hypot"
);

fn execute_sign(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("sign函数需要1个参数"));
    }
    match &args[0] {
        Value::Int(i) => Ok(Value::Int(i.signum())),
        Value::Float(f) => Ok(Value::Int(f.signum() as i64)),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("sign函数需要数值类型")),
    }
}

fn execute_rand(_args: &[Value]) -> Result<Value, ExpressionError> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    Ok(Value::Float(rng.gen::<f64>()))
}

fn execute_rand32(args: &[Value]) -> Result<Value, ExpressionError> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let result = match args.len() {
        0 => rng.gen::<i32>() as i64,
        1 => match &args[0] {
            Value::Int(max) => rng.gen_range(0..*max) as i64,
            Value::Null(_) => return Ok(Value::Null(NullType::Null)),
            _ => return Err(ExpressionError::type_error("rand32函数需要整数参数")),
        },
        2 => match (&args[0], &args[1]) {
            (Value::Int(min), Value::Int(max)) => rng.gen_range(*min..*max),
            (Value::Null(_), _) | (_, Value::Null(_)) => return Ok(Value::Null(NullType::Null)),
            _ => return Err(ExpressionError::type_error("rand32函数需要整数参数")),
        },
        _ => return Err(ExpressionError::type_error("rand32函数需要0-2个参数")),
    };
    Ok(Value::Int(result))
}

fn execute_rand64(_args: &[Value]) -> Result<Value, ExpressionError> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    Ok(Value::Int(rng.gen::<i64>()))
}

fn execute_e(_args: &[Value]) -> Result<Value, ExpressionError> {
    Ok(Value::Float(std::f64::consts::E))
}

fn execute_pi(_args: &[Value]) -> Result<Value, ExpressionError> {
    Ok(Value::Float(std::f64::consts::PI))
}

fn execute_bit_and(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 2 {
        return Err(ExpressionError::type_error("bit_and函数需要2个参数"));
    }
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a & b)),
        (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("bit_and函数需要整数参数")),
    }
}

fn execute_bit_or(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 2 {
        return Err(ExpressionError::type_error("bit_or函数需要2个参数"));
    }
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a | b)),
        (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("bit_or函数需要整数参数")),
    }
}

fn execute_bit_xor(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 2 {
        return Err(ExpressionError::type_error("bit_xor函数需要2个参数"));
    }
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a ^ b)),
        (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("bit_xor函数需要整数参数")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs_int() {
        let func = MathFunction::Abs;
        let result = func.execute(&[Value::Int(-5)]).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_abs_float() {
        let func = MathFunction::Abs;
        let result = func.execute(&[Value::Float(-5.5)]).unwrap();
        assert_eq!(result, Value::Float(5.5));
    }

    #[test]
    fn test_sqrt() {
        let func = MathFunction::Sqrt;
        let result = func.execute(&[Value::Int(16)]).unwrap();
        assert_eq!(result, Value::Float(4.0));
    }

    #[test]
    fn test_pow() {
        let func = MathFunction::Pow;
        let result = func.execute(&[Value::Int(2), Value::Int(3)]).unwrap();
        assert_eq!(result, Value::Float(8.0));
    }

    #[test]
    fn test_sin() {
        let func = MathFunction::Sin;
        let result = func.execute(&[Value::Float(0.0)]).unwrap();
        assert_eq!(result, Value::Float(0.0));
    }

    #[test]
    fn test_cos() {
        let func = MathFunction::Cos;
        let result = func.execute(&[Value::Float(0.0)]).unwrap();
        assert_eq!(result, Value::Float(1.0));
    }

    #[test]
    fn test_round() {
        let func = MathFunction::Round;
        let result = func.execute(&[Value::Float(3.7)]).unwrap();
        assert_eq!(result, Value::Float(4.0));
    }

    #[test]
    fn test_ceil() {
        let func = MathFunction::Ceil;
        let result = func.execute(&[Value::Float(3.2)]).unwrap();
        assert_eq!(result, Value::Float(4.0));
    }

    #[test]
    fn test_floor() {
        let func = MathFunction::Floor;
        let result = func.execute(&[Value::Float(3.9)]).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_null_handling() {
        let func = MathFunction::Abs;
        let result = func.execute(&[Value::Null(NullType::Null)]).unwrap();
        assert_eq!(result, Value::Null(NullType::Null));
    }
}
