//! 日期时间函数实现

use crate::core::error::ExpressionError;
use crate::core::value::{DateTimeValue, DateValue, NullType, TimeValue};
use crate::core::Value;
use crate::expression::context::CacheManager;
use chrono::{Datelike, Timelike};
use crate::define_function_enum;
use crate::define_datetime_extractor;

define_function_enum! {
    /// 日期时间函数枚举
    pub enum DateTimeFunction {
        Now => {
            name: "now",
            arity: 0,
            variadic: false,
            description: "当前时间戳",
            handler: execute_now
        },
        Date => {
            name: "date",
            arity: 1,
            variadic: false,
            description: "创建日期",
            handler: execute_date
        },
        Time => {
            name: "time",
            arity: 1,
            variadic: false,
            description: "创建时间",
            handler: execute_time
        },
        DateTime => {
            name: "datetime",
            arity: 0,
            variadic: true,
            description: "创建日期时间",
            handler: execute_datetime
        },
        Year => {
            name: "year",
            arity: 1,
            variadic: false,
            description: "提取年份",
            handler: execute_year
        },
        Month => {
            name: "month",
            arity: 1,
            variadic: false,
            description: "提取月份",
            handler: execute_month
        },
        Day => {
            name: "day",
            arity: 1,
            variadic: false,
            description: "提取日期",
            handler: execute_day
        },
        Hour => {
            name: "hour",
            arity: 1,
            variadic: false,
            description: "提取小时",
            handler: execute_hour
        },
        Minute => {
            name: "minute",
            arity: 1,
            variadic: false,
            description: "提取分钟",
            handler: execute_minute
        },
        Second => {
            name: "second",
            arity: 1,
            variadic: false,
            description: "提取秒",
            handler: execute_second
        },
        TimeStamp => {
            name: "timestamp",
            arity: 0,
            variadic: true,
            description: "获取当前时间戳或转换日期时间为时间戳",
            handler: execute_timestamp
        },
    }
}

impl DateTimeFunction {
    /// 执行函数（带缓存）
    pub fn execute_with_cache(
        &self,
        args: &[Value],
        cache: &mut CacheManager,
    ) -> Result<Value, ExpressionError> {
        match self {
            DateTimeFunction::Now => execute_now(args),
            DateTimeFunction::Date => execute_date_with_cache(args, cache),
            DateTimeFunction::Time => execute_time_with_cache(args, cache),
            DateTimeFunction::DateTime => execute_datetime_with_cache(args, cache),
            DateTimeFunction::Year => execute_year(args),
            DateTimeFunction::Month => execute_month(args),
            DateTimeFunction::Day => execute_day(args),
            DateTimeFunction::Hour => execute_hour(args),
            DateTimeFunction::Minute => execute_minute(args),
            DateTimeFunction::Second => execute_second(args),
            DateTimeFunction::TimeStamp => execute_timestamp(args),
        }
    }
}

fn execute_now(_args: &[Value]) -> Result<Value, ExpressionError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("系统时间错误")
        .as_millis();
    Ok(Value::Int(now as i64))
}

fn execute_date(_args: &[Value]) -> Result<Value, ExpressionError> {
    // 无缓存版本，创建临时缓存
    let mut cache = CacheManager::new();
    execute_date_with_cache(_args, &mut cache)
}

fn execute_date_with_cache(
    args: &[Value],
    cache: &mut CacheManager,
) -> Result<Value, ExpressionError> {
    if args.is_empty() {
        // 返回当前日期
        let now = chrono::Utc::now();
        Ok(Value::Date(DateValue {
            year: now.year(),
            month: now.month(),
            day: now.day(),
        }))
    } else {
        match &args[0] {
            Value::String(s) => {
                // 尝试从缓存获取
                if let Some(cached) = cache.get_date(s) {
                    return Ok(Value::Date(cached.clone()));
                }
                // 解析日期
                let naivedate = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
                    ExpressionError::type_error("无法解析日期字符串，期望格式: YYYY-MM-DD")
                })?;
                let date = DateValue {
                    year: naivedate.year(),
                    month: naivedate.month(),
                    day: naivedate.day(),
                };
                // 存入缓存
                cache.set_date(s.clone(), date.clone());
                Ok(Value::Date(date))
            }
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("date函数需要字符串类型")),
        }
    }
}

fn execute_time(_args: &[Value]) -> Result<Value, ExpressionError> {
    // 无缓存版本，创建临时缓存
    let mut cache = CacheManager::new();
    execute_time_with_cache(_args, &mut cache)
}

fn execute_time_with_cache(
    args: &[Value],
    cache: &mut CacheManager,
) -> Result<Value, ExpressionError> {
    if args.is_empty() {
        // 返回当前时间
        let now = chrono::Utc::now();
        Ok(Value::Time(TimeValue {
            hour: now.hour(),
            minute: now.minute(),
            sec: now.second(),
            microsec: now.timestamp_subsec_micros(),
        }))
    } else {
        match &args[0] {
            Value::String(s) => {
                // 尝试从缓存获取
                if let Some(cached) = cache.get_time(s) {
                    return Ok(Value::Time(cached.clone()));
                }
                // 解析时间
                let time = chrono::NaiveTime::parse_from_str(s, "%H:%M:%S%.f")
                    .or_else(|_| chrono::NaiveTime::parse_from_str(s, "%H:%M:%S"))
                    .map_err(|_| {
                        ExpressionError::type_error("无法解析时间字符串，期望格式: HH:MM:SS")
                    })?;
                let time_val = TimeValue {
                    hour: time.hour(),
                    minute: time.minute(),
                    sec: time.second(),
                    microsec: time.nanosecond() / 1000,
                };
                // 存入缓存
                cache.set_time(s.clone(), time_val.clone());
                Ok(Value::Time(time_val))
            }
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("time函数需要字符串类型")),
        }
    }
}

define_datetime_extractor!(execute_year, Date => year, DateTime => year);
define_datetime_extractor!(execute_month, Date => month, DateTime => month);
define_datetime_extractor!(execute_day, Date => day, DateTime => day);
define_datetime_extractor!(execute_hour, Time => hour, DateTime => hour);
define_datetime_extractor!(execute_minute, Time => minute, DateTime => minute);
define_datetime_extractor!(execute_second, Time => sec, DateTime => sec);

fn execute_datetime(_args: &[Value]) -> Result<Value, ExpressionError> {
    let mut cache = CacheManager::new();
    execute_datetime_with_cache(_args, &mut cache)
}

fn execute_datetime_with_cache(
    args: &[Value],
    cache: &mut CacheManager,
) -> Result<Value, ExpressionError> {
    if args.is_empty() {
        let now = chrono::Utc::now();
        Ok(Value::DateTime(DateTimeValue {
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
                if let Some(cached) = cache.get_datetime(s) {
                    return Ok(Value::DateTime(cached.clone()));
                }
                let datetime = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                    .map_err(|_| {
                        ExpressionError::type_error("无法解析日期时间字符串，期望格式: YYYY-MM-DD HH:MM:SS")
                    })?;
                let dt_val = DateTimeValue {
                    year: datetime.year(),
                    month: datetime.month(),
                    day: datetime.day(),
                    hour: datetime.hour(),
                    minute: datetime.minute(),
                    sec: datetime.second(),
                    microsec: datetime.nanosecond() / 1000,
                };
                cache.set_datetime(s.clone(), dt_val.clone());
                Ok(Value::DateTime(dt_val))
            }
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("datetime函数需要字符串类型")),
        }
    }
}

fn execute_timestamp(args: &[Value]) -> Result<Value, ExpressionError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    if args.is_empty() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("系统时间错误")
            .as_millis();
        Ok(Value::Int(now as i64))
    } else {
        match &args[0] {
            Value::DateTime(dt) => {
                let naive_dt = chrono::NaiveDateTime::new(
                    chrono::NaiveDate::from_ymd_opt(dt.year, dt.month, dt.day)
                        .ok_or_else(|| ExpressionError::type_error("无效的日期"))?,
                    chrono::NaiveTime::from_hms_micro_opt(dt.hour, dt.minute, dt.sec, dt.microsec)
                        .ok_or_else(|| ExpressionError::type_error("无效的时间"))?,
                );
                let timestamp = naive_dt.and_utc().timestamp_millis();
                Ok(Value::Int(timestamp))
            }
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("timestamp函数需要日期时间类型或无参数")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now() {
        let func = DateTimeFunction::Now;
        let result = func.execute(&[]).unwrap();
        assert!(matches!(result, Value::Int(_)));
    }

    #[test]
    fn test_year() {
        let func = DateTimeFunction::Year;
        let date = DateValue {
            year: 2024,
            month: 1,
            day: 15,
        };
        let result = func.execute(&[Value::Date(date)]).unwrap();
        assert_eq!(result, Value::Int(2024));
    }

    #[test]
    fn test_month() {
        let func = DateTimeFunction::Month;
        let date = DateValue {
            year: 2024,
            month: 6,
            day: 15,
        };
        let result = func.execute(&[Value::Date(date)]).unwrap();
        assert_eq!(result, Value::Int(6));
    }

    #[test]
    fn test_day() {
        let func = DateTimeFunction::Day;
        let date = DateValue {
            year: 2024,
            month: 6,
            day: 25,
        };
        let result = func.execute(&[Value::Date(date)]).unwrap();
        assert_eq!(result, Value::Int(25));
    }

    #[test]
    fn test_null_handling() {
        let func = DateTimeFunction::Year;
        let result = func.execute(&[Value::Null(NullType::Null)]).unwrap();
        assert_eq!(result, Value::Null(NullType::Null));
    }
}
