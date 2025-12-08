//! Validator 模块的单元测试

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::{Value, ValueTypeDef};
    use crate::graph::expression::{BinaryOperator, Expression};
    use crate::query::validator::validate_context::{Column, Space, Variable};
    use std::collections::HashMap;

    #[test]
    fn test_validate_context_new() {
        let context = ValidateContext::new();
        assert!(!context.space_chosen());
        assert!(context.get_space().is_none());
        assert!(context.get_errors().is_empty());
        assert!(!context.has_errors());
    }

    #[test]
    fn test_validate_context_space() {
        let mut context = ValidateContext::new();
        let space = Space {
            id: 1,
            name: "test_space".to_string(),
            vid_type: ValueTypeDef::Int,
        };

        context.set_space(space.clone());
        assert!(context.space_chosen());
        assert_eq!(context.which_space().name, space.name);
        assert_eq!(context.which_space().id, space.id);
    }

    #[test]
    fn test_validate_context_variables() {
        let mut context = ValidateContext::new();

        // 测试添加变量
        let var = Variable {
            name: "test_var".to_string(),
            columns: vec![
                Column {
                    name: "col1".to_string(),
                    type_: ValueTypeDef::Int,
                },
                Column {
                    name: "col2".to_string(),
                    type_: ValueTypeDef::String,
                },
            ],
        };

        context.add_variable(var.clone());
        assert!(context.exists_var("test_var"));

        let retrieved_var = context.get_variable("test_var").unwrap();
        assert_eq!(retrieved_var.name, var.name);
        assert_eq!(retrieved_var.columns.len(), var.columns.len());

        let columns = context.get_var("test_var");
        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0].name, "col1");
        assert_eq!(columns[1].name, "col2");
    }

    #[test]
    fn test_validate_context_parameters() {
        let mut context = ValidateContext::new();

        // 测试添加参数
        let param_value = Value::Int(42);
        context.add_parameter("test_param".to_string(), param_value.clone());

        assert!(context.exist_parameter("test_param"));

        let retrieved_param = context.get_parameter("test_param").unwrap();
        assert_eq!(retrieved_param, &param_value);
    }

    #[test]
    fn test_validate_context_aliases() {
        let mut context = ValidateContext::new();

        // 测试添加别名
        context.add_alias("test_alias".to_string(), ValueTypeDef::String);

        let alias_type = context.get_alias_type("test_alias").unwrap();
        assert_eq!(alias_type, &ValueTypeDef::String);
    }

    #[test]
    fn test_validate_context_errors() {
        let mut context = ValidateContext::new();

        // 测试错误处理
        assert!(!context.has_errors());

        context.add_error("Test error".to_string());
        assert!(context.has_errors());

        let errors = context.get_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], "Test error");

        context.clear_errors();
        assert!(!context.has_errors());
        assert!(context.get_errors().is_empty());
    }

    #[test]
    fn test_base_validator_new() {
        let context = ValidateContext::new();
        let validator = Validator::new(context);

        // 测试验证器创建
        assert!(!validator.context().space_chosen());
    }

    #[test]
    fn test_base_validator_context_access() {
        let mut context = ValidateContext::new();
        context.add_error("Test error".to_string());

        let mut validator = Validator::new(context);

        // 测试上下文访问
        assert!(validator.context().has_errors());

        validator
            .context_mut()
            .add_error("Another error".to_string());
        assert_eq!(validator.context().get_errors().len(), 2);
    }

    #[test]
    fn test_base_validator_add_error() {
        let mut context = ValidateContext::new();
        let mut validator = Validator::new(context);

        validator.add_error("Test error".to_string());
        assert!(validator.context().has_errors());
        assert_eq!(validator.context().get_errors().len(), 1);
    }

    #[test]
    fn test_match_validator_new() {
        let context = ValidateContext::new();
        let validator = MatchValidator::new(context);

        // 测试 MatchValidator 创建
        assert!(!validator.context().space_chosen());
    }

    #[test]
    fn test_match_validator_validate() {
        let context = ValidateContext::new();
        let mut validator = MatchValidator::new(context);

        // 测试基本验证功能
        let result = validator.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_validator_validate_with_context() {
        let mut context = ValidateContext::new();
        context.add_error("Initial error".to_string());

        let mut validator = MatchValidator::new(context);

        // 测试带有错误上下文的验证
        let result = validator.validate();
        assert!(result.is_ok());
        assert!(validator.context().has_errors());
    }

    #[test]
    fn test_match_validator_context_access() {
        let mut context = ValidateContext::new();
        context.add_alias("test_alias".to_string(), ValueTypeDef::String);

        let mut validator = MatchValidator::new(context);

        // 测试上下文访问
        assert!(validator.context().get_alias_type("test_alias").is_some());

        validator.context_mut().add_error("Test error".to_string());
        assert!(validator.context().has_errors());
    }
}
