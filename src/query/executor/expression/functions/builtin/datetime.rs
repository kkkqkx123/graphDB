//! 日期时间函数实现

use crate::core::error::ExpressionError;
use crate::core::value::{DateTimeValue, DateValue, NullType, TimeValue};
use crate::core::Value;
use crate::define_datetime_extractor;
use crate::define_function_enum;
use chrono::{Datelike, Timelike};

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
        _cache: &mut (),
    ) -> Result<Value, ExpressionError> {
        // 缓存功能已移除，直接调用execute
        self.execute(args)
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

fn execute_date(args: &[Value]) -> Result<Value, ExpressionError> {
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
                // 解析日期
                let naivedate = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
                    ExpressionError::type_error("无法解析日期字符串，期望格式: YYYY-MM-DD")
                })?;
                let date = DateValue {
                    year: naivedate.year(),
                    month: naivedate.month(),
                    day: naivedate.day(),
                };
                Ok(Value::Date(date))
            }
            Value::Null(_) => Ok(Value::Null(NullType::Null)),
            _ => Err(ExpressionError::type_error("date函数需要字符串类型")),
        }
    }
}

fn execute_time(args: &[Value]) -> Result<Value, ExpressionError> {
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

fn execute_datetime(args: &[Value]) -> Result<Value, ExpressionError> {
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
                let datetime = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                    .map_err(|_| {
                        ExpressionError::type_error(
                            "无法解析日期时间字符串，期望格式: YYYY-MM-DD HH:MM:SS",
                        )
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
            _ => Err(ExpressionError::type_error(
                "timestamp函数需要日期时间类型或无参数",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now() {
        let func = DateTimeFunction::Now;
        let result = func.execute(&[]).expect("Execution should succeed");
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
        let result = func.execute(&[Value::Date(date)]).expect("Execution should succeed");
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
        let result = func.execute(&[Value::Date(date)]).expect("Execution should succeed");
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
        let result = func.execute(&[Value::Date(date)]).expect("Execution should succeed");
        assert_eq!(result, Value::Int(25));
    }

    #[test]
    fn test_null_handling() {
        let func = DateTimeFunction::Year;
        let result = func.execute(&[Value::Null(NullType::Null)]).expect("Execution should succeed");
        assert_eq!(result, Value::Null(NullType::Null));
    }
}
