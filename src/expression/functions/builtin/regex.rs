//! 正则表达式函数实现

use crate::core::error::{ExpressionError, ExpressionErrorType};
use crate::core::value::NullType;
use crate::core::Value;
use crate::expression::context::CacheManager;

/// 正则表达式函数枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegexFunction {
    RegexMatch,
    RegexReplace,
    RegexFind,
}

impl RegexFunction {
    pub fn name(&self) -> &str {
        match self {
            RegexFunction::RegexMatch => "regex_match",
            RegexFunction::RegexReplace => "regex_replace",
            RegexFunction::RegexFind => "regex_find",
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            RegexFunction::RegexMatch => 2,
            RegexFunction::RegexReplace => 3,
            RegexFunction::RegexFind => 2,
        }
    }

    pub fn is_variadic(&self) -> bool {
        false
    }

    pub fn description(&self) -> &str {
        match self {
            RegexFunction::RegexMatch => "正则表达式匹配",
            RegexFunction::RegexReplace => "正则表达式替换",
            RegexFunction::RegexFind => "正则表达式查找",
        }
    }

    pub fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        // 无缓存版本，创建临时缓存
        let mut cache = CacheManager::new();
        self.execute_with_cache(args, &mut cache)
    }

    /// 执行函数（带缓存）
    pub fn execute_with_cache(&self, args: &[Value], cache: &mut CacheManager) -> Result<Value, ExpressionError> {
        match self {
            RegexFunction::RegexMatch => {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(pattern)) => {
                        if let Some(regex) = cache.get_regex(pattern) {
                            Ok(Value::Bool(regex.is_match(s)))
                        } else {
                            Err(ExpressionError::new(
                                ExpressionErrorType::InvalidOperation,
                                format!("无效的正则表达式: {}", pattern),
                            ))
                        }
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
                    _ => Err(ExpressionError::type_error("regex_match函数需要字符串类型")),
                }
            }
            RegexFunction::RegexReplace => {
                match (&args[0], &args[1], &args[2]) {
                    (Value::String(s), Value::String(pattern), Value::String(replacement)) => {
                        if let Some(regex) = cache.get_regex(pattern) {
                            Ok(Value::String(regex.replace_all(s, replacement.as_str()).to_string()))
                        } else {
                            Err(ExpressionError::new(
                                ExpressionErrorType::InvalidOperation,
                                format!("无效的正则表达式: {}", pattern),
                            ))
                        }
                    }
                    (Value::Null(_), _, _) | (_, Value::Null(_), _) | (_, _, Value::Null(_)) => {
                        Ok(Value::Null(NullType::Null))
                    }
                    _ => Err(ExpressionError::type_error("regex_replace函数需要字符串类型")),
                }
            }
            RegexFunction::RegexFind => {
                match (&args[0], &args[1]) {
                    (Value::String(s), Value::String(pattern)) => {
                        if let Some(regex) = cache.get_regex(pattern) {
                            if let Some(matched) = regex.find(s) {
                                Ok(Value::String(matched.as_str().to_string()))
                            } else {
                                Ok(Value::Null(NullType::Null))
                            }
                        } else {
                            Err(ExpressionError::new(
                                ExpressionErrorType::InvalidOperation,
                                format!("无效的正则表达式: {}", pattern),
                            ))
                        }
                    }
                    (Value::Null(_), _) | (_, Value::Null(_)) => Ok(Value::Null(NullType::Null)),
                    _ => Err(ExpressionError::type_error("regex_find函数需要字符串类型")),
                }
            }
        }
    }
}
