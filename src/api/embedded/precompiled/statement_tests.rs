//! 预编译语句模块测试

use crate::api::embedded::statement::config::{ExecutionStats, StatementConfig};
use crate::api::embedded::statement::parameter_extractor::{extract_parameters, type_matches};
use crate::core::{DataType, Value};
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_parameters_simple() {
        let query = "MATCH (n) WHERE n.age > $min_age RETURN n";
        let params = extract_parameters(query).expect("解析查询失败");

        assert!(params.contains_key("min_age"), "应该包含 min_age 参数");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_extract_parameters_multiple() {
        let query = "MATCH (n) WHERE n.age > $min_age AND n.age < $max_age RETURN n";
        let params = extract_parameters(query).expect("解析查询失败");

        assert!(params.contains_key("min_age"), "应该包含 min_age 参数");
        assert!(params.contains_key("max_age"), "应该包含 max_age 参数");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_extract_parameters_invalid_query() {
        let query = "INVALID SYNTAX !!!";
        let result = extract_parameters(query);

        assert!(result.is_err(), "无效查询应该返回错误");
    }

    #[test]
    fn test_extract_parameters_none() {
        let query = "MATCH (n) WHERE n.age > 18 RETURN n";
        let params = extract_parameters(query).expect("解析查询失败");

        assert!(params.is_empty(), "没有参数的查询应该返回空映射");
    }

    #[test]
    fn test_extract_parameters_variables_not_extracted() {
        let query = "MATCH (n) WHERE n.age > min_age RETURN n";
        let params = extract_parameters(query).expect("解析查询失败");

        assert!(params.is_empty(), "变量不应该被当作参数提取");
    }

    #[test]
    fn test_statement_config() {
        let config = StatementConfig::new()
            .disable_type_check()
            .with_max_history(50);

        assert!(!config.enable_type_check);
        assert_eq!(config.max_history_size, 50);
    }

    #[test]
    fn test_execution_stats() {
        let mut stats = ExecutionStats::new();

        stats.record_execution(Duration::from_millis(10));
        stats.record_execution(Duration::from_millis(20));
        stats.record_execution(Duration::from_millis(30));

        assert_eq!(stats.execution_count, 3);
        assert_eq!(stats.total_execution_time_ms, 60);
        assert_eq!(stats.avg_execution_time_ms, 20.0);
        assert_eq!(stats.min_execution_time_ms, 10);
        assert_eq!(stats.max_execution_time_ms, 30);
    }

    #[test]
    fn test_type_matches() {
        assert!(type_matches(&Value::Int(1), &DataType::Int));
        assert!(type_matches(&Value::Int(1), &DataType::Float));
        assert!(type_matches(
            &Value::String("test".to_string()),
            &DataType::String
        ));
        assert!(type_matches(&Value::Float(1.0), &DataType::Float));
        assert!(!type_matches(&Value::Int(1), &DataType::String));
        assert!(type_matches(
            &Value::Null(crate::core::value::NullType::Null),
            &DataType::Int
        ));
    }

    #[test]
    fn test_type_matches_compatibility() {
        assert!(
            type_matches(&Value::Int(42), &DataType::Float),
            "Int 应该可以匹配 Float"
        );
        assert!(
            !type_matches(&Value::Float(42.0), &DataType::Int),
            "Float 不应该匹配 Int"
        );
    }

    #[test]
    fn test_extract_parameters_complex_query() {
        let query = r#"
            MATCH (u) WHERE u.age >= $min_age AND u.age <= $max_age
            RETURN u.name
        "#;
        let params = extract_parameters(query).expect("解析查询失败");

        assert!(params.contains_key("min_age"));
        assert!(params.contains_key("max_age"));
        assert_eq!(params.len(), 2);
    }
}
