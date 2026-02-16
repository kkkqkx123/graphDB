//! 日期时间函数实现

use crate::core::error::ExpressionError;
use crate::core::Value;
use crate::expression::functions::registry::FunctionRegistry;
use crate::expression::functions::signature::FunctionSignature;
use crate::expression::functions::signature::ValueType;
use chrono::{Datelike, NaiveDate, NaiveTime, NaiveDateTime, Utc, Timelike};

/// 注册所有日期时间函数
pub fn register_all(registry: &mut FunctionRegistry) {
    register_now(registry);
    register_date(registry);
    register_time(registry);
    register_datetime(registry);
    register_timestamp(registry);
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

fn register_time(registry: &mut FunctionRegistry) {
    registry.register(
        "time",
        FunctionSignature::new(
            "time",
            vec![ValueType::String],
            ValueType::Time,
            0,
            1,
            true,
            "创建时间或获取当前时间",
        ),
        |args| {
            if args.is_empty() {
                let now = Utc::now();
                Ok(Value::Time(crate::core::value::TimeValue {
                    hour: now.hour(),
                    minute: now.minute(),
                    sec: now.second(),
                    microsec: now.timestamp_subsec_micros(),
                }))
            } else {
                match &args[0] {
                    Value::String(s) => {
                        let time = NaiveTime::parse_from_str(s, "%H:%M:%S%.f")
                            .or_else(|_| NaiveTime::parse_from_str(s, "%H:%M:%S"))
                            .map_err(|_| ExpressionError::type_error("无法解析时间字符串，期望格式: HH:MM:SS 或 HH:MM:SS.microseconds"))?;
                        Ok(Value::Time(crate::core::value::TimeValue {
                            hour: time.hour(),
                            minute: time.minute(),
                            sec: time.second(),
                            microsec: time.nanosecond() / 1000,
                        }))
                    }
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("time函数需要字符串类型")),
                }
            }
        },
    );
}

fn register_datetime(registry: &mut FunctionRegistry) {
    registry.register(
        "datetime",
        FunctionSignature::new(
            "datetime",
            vec![ValueType::String],
            ValueType::DateTime,
            0,
            1,
            true,
            "创建日期时间或获取当前日期时间",
        ),
        |args| {
            if args.is_empty() {
                let now = Utc::now();
                Ok(Value::DateTime(crate::core::value::DateTimeValue {
                    year: now.year(),
                    month: now.month(),
                    day: now.day(),
                    hour: now.hour(),
                    minute: now.minute(),
                    sec: now.second(),
                    microsec: now.timestamp_subsec_micros(),
                }))
            } else {
                match &args[0] {
                    Value::String(s) => {
                        let datetime = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
                            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
                            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f"))
                            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
                            .map_err(|_| ExpressionError::type_error("无法解析日期时间字符串，期望格式: YYYY-MM-DD HH:MM:SS 或 ISO 8601格式"))?;
                        Ok(Value::DateTime(crate::core::value::DateTimeValue {
                            year: datetime.year(),
                            month: datetime.month(),
                            day: datetime.day(),
                            hour: datetime.hour(),
                            minute: datetime.minute(),
                            sec: datetime.second(),
                            microsec: datetime.nanosecond() / 1000,
                        }))
                    }
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("datetime函数需要字符串类型")),
                }
            }
        },
    );
}

fn register_timestamp(registry: &mut FunctionRegistry) {
    registry.register(
        "timestamp",
        FunctionSignature::new(
            "timestamp",
            vec![ValueType::Any],
            ValueType::Int,
            0,
            1,
            true,
            "获取当前时间戳或将日期时间转换为时间戳",
        ),
        |args| {
            if args.is_empty() {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("系统时间错误")
                    .as_secs();
                Ok(Value::Int(now as i64))
            } else {
                match &args[0] {
                    Value::DateTime(dt) => {
                        let naive_dt = NaiveDateTime::new(
                            NaiveDate::from_ymd_opt(dt.year, dt.month, dt.day)
                                .ok_or_else(|| ExpressionError::type_error("无效的日期"))?,
                            NaiveTime::from_hms_micro_opt(dt.hour, dt.minute, dt.sec, dt.microsec)
                                .ok_or_else(|| ExpressionError::type_error("无效的时间"))?,
                        );
                        let timestamp = naive_dt.and_utc().timestamp();
                        Ok(Value::Int(timestamp))
                    }
                    Value::Date(d) => {
                        let naive_date = NaiveDate::from_ymd_opt(d.year, d.month, d.day)
                            .ok_or_else(|| ExpressionError::type_error("无效的日期"))?;
                        let timestamp = naive_date.and_hms_opt(0, 0, 0)
                            .ok_or_else(|| ExpressionError::type_error("无效的时间"))?
                            .and_utc()
                            .timestamp();
                        Ok(Value::Int(timestamp))
                    }
                    Value::String(s) => {
                        let datetime = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d"))
                            .map_err(|_| ExpressionError::type_error("无法解析日期时间字符串"))?;
                        Ok(Value::Int(datetime.and_utc().timestamp()))
                    }
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("timestamp函数需要日期时间、日期或字符串类型")),
                }
            }
        },
    );
}
