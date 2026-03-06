//! 预编译语句模块测试

use crate::api::embedded::statement::config::{ExecutionStats, StatementConfig};
use crate::api::embedded::statement::parameter_extractor::ParameterExtractor;
use crate::core::{DataType, Value};
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_parameters_insert() {
        let query = "INSERT VERTEX Person(name, age) VALUES $id:($name, $age)";
        let params = ParameterExtractor::extract_parameters(query).expect("解析查询失败");

        assert!(params.contains_key("id"), "应该包含 id 参数");
        assert!(params.contains_key("name"), "应该包含 name 参数");
        assert!(params.contains_key("age"), "应该包含 age 参数");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_extract_parameters_update() {
        let query = "UPDATE $vid SET age = $new_age";
        let params = ParameterExtractor::extract_parameters(query).expect("解析查询失败");

        assert!(params.contains_key("vid"), "应该包含 vid 参数");
        assert!(params.contains_key("new_age"), "应该包含 new_age 参数");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_extract_parameters_delete() {
        let query = "DELETE VERTEX $vid";
        let params = ParameterExtractor::extract_parameters(query).expect("解析查询失败");

        assert!(params.contains_key("vid"), "应该包含 vid 参数");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_extract_parameters_invalid_query() {
        let query = "INVALID SYNTAX !!!";
        let result = ParameterExtractor::extract_parameters(query);

        assert!(result.is_err(), "无效查询应该返回错误");
    }

    #[test]
    fn test_extract_parameters_none() {
        let query = "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)";
        let params = ParameterExtractor::extract_parameters(query).expect("解析查询失败");

        assert!(params.is_empty(), "没有参数的查询应该返回空映射");
    }

    #[test]
    fn test_statement_config() {
        let config = StatementConfig::new()
            .disable_cache()
            .disable_type_check()
            .with_max_history(50);

        assert!(!config.enable_cache);
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
        assert!(ParameterExtractor::type_matches(&Value::Int(1), &DataType::Int));
        assert!(ParameterExtractor::type_matches(
            &Value::String("test".to_string()),
            &DataType::String
        ));
        assert!(!ParameterExtractor::type_matches(
            &Value::Int(1),
            &DataType::String
        ));
    }
}
