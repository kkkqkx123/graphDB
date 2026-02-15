//! 日期时间函数实现

use crate::core::error::ExpressionError;
use crate::core::Value;
use crate::expression::functions::registry::FunctionRegistry;
use crate::expression::functions::signature::FunctionSignature;
use crate::expression::functions::signature::ValueType;
use chrono::{Datelike, NaiveDate, Utc};

/// 注册所有日期时间函数
pub fn register_all(registry: &mut FunctionRegistry) {
    register_now(registry);
    register_date(registry);
    register_year(registry);
    register_month(registry);
    register_day(registry);
    register_hour(registry);
    register_minute(registry);
    register_second(registry);
}

fn register_now(registry: &mut FunctionRegistry) {
    registry.register(
        "now",
        FunctionSignature::new(
            "now",
            vec![],
            ValueType::Int,
            0,
            0,
            false,
            "获取当前时间戳",
        ),
        |_args| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Failed to create function registry")
                .as_millis();
            Ok(Value::Int(now as i64))
        },
    );
}

fn register_date(registry: &mut FunctionRegistry) {
    registry.register(
        "date",
        FunctionSignature::new(
            "date",
            vec![ValueType::String],
            ValueType::Date,
            0,
            1,
            true,
            "创建日期",
        ),
        |args| {
            if args.is_empty() {
                let now = Utc::now();
                Ok(Value::Date(crate::core::value::DateValue {
                    year: now.year(),
                    month: now.month() as u32,
                    day: now.day() as u32,
                }))
            } else {
                match &args[0] {
                    Value::String(s) => {
                        let naivedate = NaiveDate::parse_from_str(s, "%Y-%m-%d")
                            .map_err(|_| ExpressionError::type_error("无法解析日期字符串"))?;
                        Ok(Value::Date(crate::core::value::DateValue {
                            year: naivedate.year(),
                            month: naivedate.month() as u32,
                            day: naivedate.day() as u32,
                        }))
                    }
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("date函数需要字符串类型")),
                }
            }
        },
    );
}

fn register_year(registry: &mut FunctionRegistry) {
    registry.register(
        "year",
        FunctionSignature::new(
            "year",
            vec![ValueType::Any],
            ValueType::Int,
            1,
            1,
            true,
            "提取年份",
        ),
        |args| {
            match &args[0] {
                Value::Date(d) => Ok(Value::Int(d.year as i64)),
                Value::DateTime(dt) => Ok(Value::Int(dt.year as i64)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("year函数需要日期或日期时间类型")),
            }
        },
    );
}

fn register_month(registry: &mut FunctionRegistry) {
    registry.register(
        "month",
        FunctionSignature::new(
            "month",
            vec![ValueType::Any],
            ValueType::Int,
            1,
            1,
            true,
            "提取月份",
        ),
        |args| {
            match &args[0] {
                Value::Date(d) => Ok(Value::Int(d.month as i64)),
                Value::DateTime(dt) => Ok(Value::Int(dt.month as i64)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("month函数需要日期或日期时间类型")),
            }
        },
    );
}

fn register_day(registry: &mut FunctionRegistry) {
    registry.register(
        "day",
        FunctionSignature::new(
            "day",
            vec![ValueType::Any],
            ValueType::Int,
            1,
            1,
            true,
            "提取日",
        ),
        |args| {
            match &args[0] {
                Value::Date(d) => Ok(Value::Int(d.day as i64)),
                Value::DateTime(dt) => Ok(Value::Int(dt.day as i64)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("day函数需要日期或日期时间类型")),
            }
        },
    );
}

fn register_hour(registry: &mut FunctionRegistry) {
    registry.register(
        "hour",
        FunctionSignature::new(
            "hour",
            vec![ValueType::Any],
            ValueType::Int,
            1,
            1,
            true,
            "提取小时",
        ),
        |args| {
            match &args[0] {
                Value::Time(t) => Ok(Value::Int(t.hour as i64)),
                Value::DateTime(dt) => Ok(Value::Int(dt.hour as i64)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("hour函数需要时间或日期时间类型")),
            }
        },
    );
}

fn register_minute(registry: &mut FunctionRegistry) {
    registry.register(
        "minute",
        FunctionSignature::new(
            "minute",
            vec![ValueType::Any],
            ValueType::Int,
            1,
            1,
            true,
            "提取分钟",
        ),
        |args| {
            match &args[0] {
                Value::Time(t) => Ok(Value::Int(t.minute as i64)),
                Value::DateTime(dt) => Ok(Value::Int(dt.minute as i64)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("minute函数需要时间或日期时间类型")),
            }
        },
    );
}

fn register_second(registry: &mut FunctionRegistry) {
    registry.register(
        "second",
        FunctionSignature::new(
            "second",
            vec![ValueType::Any],
            ValueType::Int,
            1,
            1,
            true,
            "提取秒",
        ),
        |args| {
            match &args[0] {
                Value::Time(t) => Ok(Value::Int(t.sec as i64)),
                Value::DateTime(dt) => Ok(Value::Int(dt.sec as i64)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("second函数需要时间或日期时间类型")),
            }
        },
    );
}
