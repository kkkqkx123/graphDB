//! Module for defining built-in function macros
//!
//! Provide macros for reducing the amount of样板 code, used for defining function enumerations and for executing functions.

/// Macro for defining an enumeration of built-in functions
///
/// 自动生成 name(), arity(), is_variadic(), description(), execute() 方法
#[macro_export]
macro_rules! define_function_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => {
                    name: $func_name:literal,
                    arity: $arity:expr,
                    variadic: $variadic:expr,
                    description: $desc:literal,
                    handler: $handler:expr
                }
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl $name {
            /// Obtain the function name
            $vis fn name(&self) -> &str {
                match self {
                    $(Self::$variant => $func_name,)*
                }
            }

            /// Determine the number of parameters
            $vis fn arity(&self) -> usize {
                match self {
                    $(Self::$variant => $arity,)*
                }
            }

            /// Is it a function with variable parameters?
            $vis fn is_variadic(&self) -> bool {
                match self {
                    $(Self::$variant => $variadic,)*
                }
            }

            /// Obtain the function description
            $vis fn description(&self) -> &str {
                match self {
                    $(Self::$variant => $desc,)*
                }
            }

            /// Execute the function
            $vis fn execute(&self, args: &[$crate::core::Value]) -> Result<$crate::core::Value, $crate::core::error::ExpressionError> {
                let handler: fn(&[$crate::core::Value]) -> Result<$crate::core::Value, $crate::core::error::ExpressionError> = match self {
                    $(Self::$variant => $handler,)*
                };
                handler(args)
            }
        }
    };
}

/// Define a single-parameter numerical function that returns a Float value.
#[macro_export]
macro_rules! define_unary_float_fn {
    ($name:ident, $op:expr, $desc:literal) => {
        fn $name(
            args: &[$crate::core::Value],
        ) -> Result<$crate::core::Value, $crate::core::error::ExpressionError> {
            if args.is_empty() {
                return Err($crate::core::error::ExpressionError::new(
                    $crate::core::error::ExpressionErrorType::InvalidArgumentCount,
                    concat!($desc, "函数需要1个参数"),
                ));
            }

            let op = $op;
            match &args[0] {
                Value::Int(i) => Ok(Value::Float(op(*i as f64))),
                Value::Float(f) => Ok(Value::Float(op(*f))),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err($crate::core::error::ExpressionError::type_error(concat!(
                    $desc,
                    "函数需要数值类型"
                ))),
            }
        }
    };
}

/// Define a single-parameter integer/float function (while preserving the data type)
#[macro_export]
macro_rules! define_unary_numeric_fn {
    ($name:ident, int: $int_op:expr, float: $float_op:expr, $desc:literal) => {
        fn $name(
            args: &[$crate::core::Value],
        ) -> Result<$crate::core::Value, $crate::core::error::ExpressionError> {
            if args.is_empty() {
                return Err($crate::core::error::ExpressionError::new(
                    $crate::core::error::ExpressionErrorType::InvalidArgumentCount,
                    concat!($desc, "函数需要1个参数"),
                ));
            }

            match &args[0] {
                Value::Int(i) => $int_op(*i),
                Value::Float(f) => $float_op(*f),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err($crate::core::error::ExpressionError::type_error(concat!(
                    $desc,
                    "函数需要数值类型"
                ))),
            }
        }
    };
}

/// Define a single-parameter string function
#[macro_export]
macro_rules! define_unary_string_fn {
    ($name:ident, $op:expr, $desc:literal) => {
        fn $name(
            args: &[$crate::core::Value],
        ) -> Result<$crate::core::Value, $crate::core::error::ExpressionError> {
            if args.is_empty() {
                return Err($crate::core::error::ExpressionError::new(
                    $crate::core::error::ExpressionErrorType::InvalidArgumentCount,
                    concat!($desc, "函数需要1个参数"),
                ));
            }

            let op = $op;
            match &args[0] {
                Value::String(s) => Ok(Value::String(op(s))),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err($crate::core::error::ExpressionError::type_error(concat!(
                    $desc,
                    "函数需要字符串类型"
                ))),
            }
        }
    };
}

/// Define a function for extracting date and time fields
#[macro_export]
macro_rules! define_datetime_extractor {
    ($name:ident, Date => $date_field:ident, DateTime => $datetime_field:ident) => {
        fn $name(
            args: &[$crate::core::Value],
        ) -> Result<$crate::core::Value, $crate::core::error::ExpressionError> {
            if args.is_empty() {
                return Err($crate::core::error::ExpressionError::new(
                    $crate::core::error::ExpressionErrorType::InvalidArgumentCount,
                    concat!(stringify!($name), "函数需要1个参数"),
                ));
            }

            match &args[0] {
                Value::Date(d) => Ok(Value::Int(d.$date_field as i64)),
                Value::DateTime(dt) => Ok(Value::Int(dt.$datetime_field as i64)),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err($crate::core::error::ExpressionError::type_error(concat!(
                    stringify!($name),
                    "函数需要日期或日期时间类型"
                ))),
            }
        }
    };
    ($name:ident, Time => $time_field:ident, DateTime => $datetime_field:ident) => {
        fn $name(
            args: &[$crate::core::Value],
        ) -> Result<$crate::core::Value, $crate::core::error::ExpressionError> {
            if args.is_empty() {
                return Err($crate::core::error::ExpressionError::new(
                    $crate::core::error::ExpressionErrorType::InvalidArgumentCount,
                    concat!(stringify!($name), "函数需要1个参数"),
                ));
            }

            match &args[0] {
                Value::Time(t) => Ok(Value::Int(t.$time_field as i64)),
                Value::DateTime(dt) => Ok(Value::Int(dt.$datetime_field as i64)),
                Value::Null(_) => Ok(Value::Null(NullType::Null)),
                _ => Err($crate::core::error::ExpressionError::type_error(concat!(
                    stringify!($name),
                    "函数需要时间或日期时间类型"
                ))),
            }
        }
    };
}

/// Define a wrapper function that performs a check on the number of parameters
#[macro_export]
macro_rules! define_arg_checked_fn {
    ($name:ident, $arity:expr, $handler:expr, $type_desc:literal) => {
        fn $name(
            args: &[$crate::core::Value],
        ) -> Result<$crate::core::Value, $crate::core::error::ExpressionError> {
            if args.len() != $arity {
                return Err($crate::core::error::ExpressionError::type_error(concat!(
                    stringify!($name),
                    "函数需要",
                    stringify!($arity),
                    "个参数"
                )));
            }
            $handler(args)
        }
    };
}

/// Define a binary numeric operation function
#[macro_export]
macro_rules! define_binary_numeric_fn {
    ($name:ident, $op:expr, $desc:literal) => {
        fn $name(
            args: &[$crate::core::Value],
        ) -> Result<$crate::core::Value, $crate::core::error::ExpressionError> {
            if args.len() != 2 {
                return Err($crate::core::error::ExpressionError::new(
                    $crate::core::error::ExpressionErrorType::InvalidArgumentCount,
                    concat!($desc, "函数需要2个参数"),
                ));
            }

            let op = $op;
            match (&args[0], &args[1]) {
                (Value::Int(a), Value::Int(b)) => op(*a as f64, *b as f64),
                (Value::Int(a), Value::Float(b)) => op(*a as f64, *b),
                (Value::Float(a), Value::Int(b)) => op(*a, *b as f64),
                (Value::Float(a), Value::Float(b)) => op(*a, *b),
                (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
                _ => Err($crate::core::error::ExpressionError::type_error(concat!(
                    $desc,
                    "函数需要数值类型"
                ))),
            }
        }
    };
}

/// Define a binary string comparison function
#[macro_export]
macro_rules! define_binary_string_bool_fn {
    ($name:ident, $op:expr, $desc:literal) => {
        fn $name(
            args: &[$crate::core::Value],
        ) -> Result<$crate::core::Value, $crate::core::error::ExpressionError> {
            if args.len() != 2 {
                return Err($crate::core::error::ExpressionError::new(
                    $crate::core::error::ExpressionErrorType::InvalidArgumentCount,
                    concat!($desc, "函数需要2个参数"),
                ));
            }

            let op = $op;
            match (&args[0], &args[1]) {
                (Value::String(a), Value::String(b)) => Ok(Value::Bool(op(a, b))),
                (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
                _ => Err($crate::core::error::ExpressionError::type_error(concat!(
                    $desc,
                    "函数需要字符串类型"
                ))),
            }
        }
    };
}

/// Define a geospatial binary function
#[macro_export]
macro_rules! define_binary_geography_fn {
    ($name:ident, $op:expr, $desc:literal) => {
        fn $name(
            args: &[$crate::core::Value],
        ) -> Result<$crate::core::Value, $crate::core::error::ExpressionError> {
            if args.len() != 2 {
                return Err($crate::core::error::ExpressionError::new(
                    $crate::core::error::ExpressionErrorType::InvalidArgumentCount,
                    concat!($desc, "函数需要2个参数"),
                ));
            }

            let op = $op;
            match (&args[0], &args[1]) {
                (Value::Geography(geo1), Value::Geography(geo2)) => op(geo1, geo2),
                (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
                _ => Err($crate::core::error::ExpressionError::type_error(concat!(
                    $desc,
                    "函数需要地理类型参数"
                ))),
            }
        }
    };
}
