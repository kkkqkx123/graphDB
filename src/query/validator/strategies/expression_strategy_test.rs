//! 表达式验证策略测试
//! 测试表达式验证策略的各种功能

#[cfg(test)]
mod expression_strategy_tests {
    use crate::query::validator::strategies::expression_strategy::ExpressionValidationStrategy;
    use crate::query::validator::structs::*;
    use crate::core::Expression;
    use crate::core::DataType;
    use crate::core::Value;
    use std::collections::HashMap;

    #[test]
    fn test_expression_validation_strategy_creation() {
        let _strategy = ExpressionValidationStrategy::new();
        assert!(true);
    }

    #[test]
    fn test_validate_filter() {
        let strategy = ExpressionValidationStrategy::new();
        let context = WhereClauseContext {
            filter: None,
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: vec![],
            query_parts: vec![],
            errors: vec![],
        };
        
        // 有效的布尔表达式
        let bool_expression = Expression::Literal(Value::Bool(true));
        let result = strategy.validate_filter(&bool_expression, &context);
        assert!(result.is_ok());
        
        // 无效的非布尔表达式
        let int_expression = Expression::Literal(Value::Int(42));
        let result = strategy.validate_filter(&int_expression, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path() {
        let strategy = ExpressionValidationStrategy::new();
        let context = MatchClauseContext {
            paths: vec![],
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
            query_parts: vec![],
            errors: vec![],
        };
        
        // 测试有效的路径表达式
        let path_expression = Expression::Path(vec![
            Expression::Label("Person".to_string()),
            Expression::Label("KNOWS".to_string()),
            Expression::Label("Person".to_string()),
        ]);
        let result = strategy.validate_path(&path_expression, &context);
        assert!(result.is_ok());
        
        // 测试标签表达式（应该返回 Empty 类型，也被接受）
        let label_expression = Expression::Label("Person".to_string());
        let result = strategy.validate_path(&label_expression, &context);
        assert!(result.is_ok());
        
        // 测试无效的类型（非路径类型）
        let int_expression = Expression::Literal(Value::Int(42));
        let result = strategy.validate_path(&int_expression, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_return() {
        let strategy = ExpressionValidationStrategy::new();
        let mut aliases = HashMap::new();
        aliases.insert("n".to_string(), crate::query::validator::AliasType::Node);
        let context = ReturnClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: vec![],
                aliases_available: aliases.clone(),
                aliases_generated: HashMap::new(),
                distinct: false,
                has_agg: false,
                group_keys: vec![],
                group_items: vec![],
                need_gen_project: false,
                agg_output_column_names: vec![],
                proj_output_column_names: vec![],
                proj_cols: vec![],
                paths: vec![],
                query_parts: vec![],
                errors: vec![],
                filter_condition: None,
                skip: None,
                limit: None,
            },
            aliases_available: aliases.clone(),
            aliases_generated: HashMap::new(),
            pagination: None,
            order_by: None,
            distinct: false,
            query_parts: vec![],
            errors: vec![],
        };
        
        // 测试简单的返回表达式
        let var_expression = Expression::Variable("n".to_string());
        let result = strategy.validate_return(&var_expression, &[], &context);
        assert!(result.is_ok());
        
        // 测试包含聚合函数的返回表达式
        let agg_expression = Expression::Aggregate {
            func: crate::core::AggregateFunction::Count(None),
            arg: Box::new(Expression::Variable("n".to_string())),
            distinct: false,
        };
        let result = strategy.validate_return(&agg_expression, &[], &context);
        assert!(result.is_ok());
        
        // 测试包含 GROUP BY 上下文的聚合函数
        let context_with_group = ReturnClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: vec![],
                aliases_available: aliases.clone(),
                aliases_generated: HashMap::new(),
                distinct: false,
                has_agg: true,
                group_keys: vec![Expression::Literal(Value::String("group_key".to_string()))],
                group_items: vec![],
                need_gen_project: false,
                agg_output_column_names: vec![],
                proj_output_column_names: vec![],
                proj_cols: vec![],
                paths: vec![],
                query_parts: vec![],
                errors: vec![],
                filter_condition: None,
                skip: None,
                limit: None,
            },
            aliases_available: aliases,
            aliases_generated: HashMap::new(),
            pagination: None,
            order_by: None,
            distinct: false,
            query_parts: vec![],
            errors: vec![],
        };
        let result = strategy.validate_return(&agg_expression, &[], &context_with_group);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with() {
        let strategy = ExpressionValidationStrategy::new();
        let mut aliases = HashMap::new();
        aliases.insert("n".to_string(), crate::query::validator::AliasType::Node);
        let context = WithClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: vec![],
                aliases_available: aliases.clone(),
                aliases_generated: HashMap::new(),
                distinct: false,
                has_agg: false,
                group_keys: vec![],
                group_items: vec![],
                need_gen_project: false,
                agg_output_column_names: vec![],
                proj_output_column_names: vec![],
                proj_cols: vec![],
                paths: vec![],
                query_parts: vec![],
                errors: vec![],
                filter_condition: None,
                skip: None,
                limit: None,
            },
            aliases_available: aliases.clone(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            pagination: None,
            order_by: None,
            distinct: false,
            query_parts: vec![],
            errors: vec![],
        };
        
        // 测试简单的 With 表达式
        let var_expression = Expression::Variable("n".to_string());
        let result = strategy.validate_with(&var_expression, &[], &context);
        assert!(result.is_ok());
        
        // 测试包含聚合函数的 With 表达式
        let agg_expression = Expression::Aggregate {
            func: crate::core::AggregateFunction::Count(None),
            arg: Box::new(Expression::Variable("n".to_string())),
            distinct: false,
        };
        let result = strategy.validate_with(&agg_expression, &[], &context);
        assert!(result.is_ok());
        
        // 测试包含 GROUP BY 上下文的聚合函数
        let context_with_group = WithClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: vec![],
                aliases_available: aliases.clone(),
                aliases_generated: HashMap::new(),
                distinct: false,
                has_agg: true,
                group_keys: vec![Expression::Literal(Value::String("group_key".to_string()))],
                group_items: vec![],
                need_gen_project: false,
                agg_output_column_names: vec![],
                proj_output_column_names: vec![],
                proj_cols: vec![],
                paths: vec![],
                query_parts: vec![],
                errors: vec![],
                filter_condition: None,
                skip: None,
                limit: None,
            },
            aliases_available: aliases,
            aliases_generated: HashMap::new(),
            where_clause: None,
            pagination: None,
            order_by: None,
            distinct: false,
            query_parts: vec![],
            errors: vec![],
        };
        let result = strategy.validate_with(&agg_expression, &[], &context_with_group);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_unwind() {
        let strategy = ExpressionValidationStrategy::new();
        let mut aliases = HashMap::new();
        aliases.insert("list_var".to_string(), crate::query::validator::AliasType::Variable);
        let context = UnwindClauseContext {
            alias: "item".to_string(),
            unwind_expression: Expression::Literal(Value::Int(0)),
            aliases_available: aliases,
            aliases_generated: HashMap::new(),
            paths: vec![],
            query_parts: vec![],
            errors: vec![],
        };
        
        // 测试有效的列表表达式
        let list_expression = Expression::List(vec![
            Expression::Literal(Value::Int(1)),
            Expression::Literal(Value::Int(2)),
            Expression::Literal(Value::Int(3)),
        ]);
        let result = strategy.validate_unwind(&list_expression, &context);
        assert!(result.is_ok());
        
        // 测试变量表达式（可能返回 Empty 类型，也被接受）
        let var_expression = Expression::Variable("list_var".to_string());
        let result = strategy.validate_unwind(&var_expression, &context);
        assert!(result.is_ok());
        
        // 测试无效的类型（非列表类型）
        let int_expression = Expression::Literal(Value::Int(42));
        let result = strategy.validate_unwind(&int_expression, &context);
        assert!(result.is_err());
        
        // 测试字符串表达式（非列表类型）
        let string_expression = Expression::Literal(Value::String("test".to_string()));
        let result = strategy.validate_unwind(&string_expression, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_yield() {
        let strategy = ExpressionValidationStrategy::new();
        let mut aliases = HashMap::new();
        aliases.insert("n".to_string(), crate::query::validator::AliasType::Node);
        let context = YieldClauseContext {
            yield_columns: vec![],
            aliases_available: aliases.clone(),
            aliases_generated: HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: vec![],
            group_items: vec![],
            need_gen_project: false,
            agg_output_column_names: vec![],
            proj_output_column_names: vec![],
            proj_cols: vec![],
            paths: vec![],
            query_parts: vec![],
            errors: vec![],
            filter_condition: None,
            skip: None,
            limit: None,
        };

        // 测试简单的 YIELD 上下文
        let result = strategy.validate_yield(&context);
        assert!(result.is_ok());

        // 测试包含聚合函数的 YIELD 子句
        let context_with_agg = YieldClauseContext {
            yield_columns: vec![
                crate::query::validator::YieldColumn {
                    expression: Expression::Aggregate {
                        func: crate::core::AggregateFunction::Count(None),
                        arg: Box::new(Expression::Literal(Value::Int(1))),
                        distinct: false,
                    },
                    alias: "count".to_string(),
                    is_matched: false,
                },
            ],
            aliases_available: aliases.clone(),
            aliases_generated: HashMap::new(),
            distinct: false,
            has_agg: true,
            group_keys: vec![],
            group_items: vec![],
            need_gen_project: false,
            agg_output_column_names: vec![],
            proj_output_column_names: vec![],
            proj_cols: vec![],
            paths: vec![],
            query_parts: vec![],
            errors: vec![],
            filter_condition: None,
            skip: None,
            limit: None,
        };
        let result = strategy.validate_yield(&context_with_agg);
        assert!(result.is_ok());

        // 测试包含 GROUP BY 的 YIELD 子句
        let context_with_group = YieldClauseContext {
            yield_columns: vec![
                crate::query::validator::YieldColumn {
                    expression: Expression::Literal(Value::String("group_key".to_string())),
                    alias: "node".to_string(),
                    is_matched: false,
                },
                crate::query::validator::YieldColumn {
                    expression: Expression::Aggregate {
                        func: crate::core::AggregateFunction::Count(None),
                        arg: Box::new(Expression::Literal(Value::Int(1))),
                        distinct: false,
                    },
                    alias: "count".to_string(),
                    is_matched: false,
                },
            ],
            aliases_available: aliases,
            aliases_generated: HashMap::new(),
            distinct: false,
            has_agg: true,
            group_keys: vec![Expression::Literal(Value::String("group_key".to_string()))],
            group_items: vec![],
            need_gen_project: false,
            agg_output_column_names: vec![],
            proj_output_column_names: vec![],
            proj_cols: vec![],
            paths: vec![],
            query_parts: vec![],
            errors: vec![],
            filter_condition: None,
            skip: None,
            limit: None,
        };
        let result = strategy.validate_yield(&context_with_group);
        assert!(result.is_ok());
    }

    #[test]
    fn test_single_path_pattern() {
        let strategy = ExpressionValidationStrategy::new();
        let mut context = MatchClauseContext {
            paths: vec![],
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
            query_parts: vec![],
            errors: vec![],
        };
        
        // 测试有效的路径模式
        let path_expression = Expression::Path(vec![
            Expression::Label("Person".to_string()),
            Expression::Label("KNOWS".to_string()),
            Expression::Label("Person".to_string()),
        ]);
        let result = strategy.validate_single_path_pattern(&path_expression, &mut context);
        assert!(result.is_ok());
        
        // 测试标签表达式（应该返回 Empty 类型，也被接受）
        let label_expression = Expression::Label("Person".to_string());
        let result = strategy.validate_single_path_pattern(&label_expression, &mut context);
        assert!(result.is_ok());
        
        // 测试无效的类型（非路径类型）
        let int_expression = Expression::Literal(Value::Int(42));
        let result = strategy.validate_single_path_pattern(&int_expression, &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_expression_type() {
        let type_validator = crate::query::validator::strategies::type_inference::TypeValidator::new();
        let context = ValidationContextImpl::new();
        
        // 字面量类型验证
        let bool_expression = Expression::Literal(Value::Bool(true));
        let result = type_validator.validate_expression_type(&bool_expression, &context, DataType::Bool);
        assert!(result.is_ok());
        
        let result = type_validator.validate_expression_type(&bool_expression, &context, DataType::Int);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_aggregate_expression() {
        let type_validator = crate::query::validator::strategies::type_inference::TypeValidator::new();
        let context = YieldClauseContext {
            yield_columns: vec![],
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: vec![],
            group_items: vec![],
            need_gen_project: false,
            agg_output_column_names: vec![],
            proj_output_column_names: vec![],
            proj_cols: vec![],
            paths: vec![],
            query_parts: vec![],
            errors: vec![],
            filter_condition: None,
            skip: None,
            limit: None,
        };
        
        // 聚合表达式
        let agg_expression = Expression::Aggregate {
            func: crate::core::AggregateFunction::Count(None),
            arg: Box::new(Expression::Variable("n".to_string())),
            distinct: false,
        };
        
        // 验证聚合表达式类型
        let result = type_validator.deduce_expression_type_full(&agg_expression, &context);
        assert!(matches!(result, DataType::Int));
    }

    #[test]
    fn test_validate_expression_operations() {
        let expr_validator = crate::query::validator::strategies::expression_operations::ExpressionOperationsValidator::new();
        
        // 简单的二元表达式
        let binary_expression = Expression::Binary {
            op: crate::core::BinaryOperator::Add,
            left: Box::new(Expression::Literal(Value::Int(1))),
            right: Box::new(Expression::Literal(Value::Int(2))),
        };
        
        let result = expr_validator.validate_expression_operations(&binary_expression);
        assert!(result.is_ok());
    }

    #[test]
    fn test_has_aggregate_expression() {
        let type_validator = crate::query::validator::strategies::type_inference::TypeValidator::new();
        
        // 包含聚合函数的表达式
        let agg_expression = Expression::Aggregate {
            func: crate::core::AggregateFunction::Count(None),
            arg: Box::new(Expression::Variable("n".to_string())),
            distinct: false,
        };
        
        let result = type_validator.has_aggregate_expression(&agg_expression);
        assert!(result);
        
        // 不包含聚合函数的表达式
        let var_expression = Expression::Variable("n".to_string());
        let result = type_validator.has_aggregate_expression(&var_expression);
        assert!(!result);
    }

    #[test]
    fn test_validate_group_key_type() {
        let type_validator = crate::query::validator::strategies::type_inference::TypeValidator::new();
        let mut aliases = HashMap::new();
        aliases.insert("n".to_string(), crate::query::validator::AliasType::Node);
        let context = YieldClauseContext {
            yield_columns: vec![],
            aliases_available: aliases,
            aliases_generated: HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: vec![],
            group_items: vec![],
            need_gen_project: false,
            agg_output_column_names: vec![],
            proj_output_column_names: vec![],
            proj_cols: vec![],
            paths: vec![],
            query_parts: vec![],
            errors: vec![],
            filter_condition: None,
            skip: None,
            limit: None,
        };
        
        // 分组键表达式 - 使用基本类型（字符串）
        let string_expression = Expression::Literal(Value::String("test".to_string()));
        let result = type_validator.validate_group_key_type(&string_expression, &context);
        assert!(result.is_ok());
        
        // 分组键表达式 - 使用整数类型
        let int_expression = Expression::Literal(Value::Int(42));
        let result = type_validator.validate_group_key_type(&int_expression, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_expression_cycles() {
        let expr_validator = crate::query::validator::strategies::expression_operations::ExpressionOperationsValidator::new();
        
        // 简单的表达式
        let var_expression = Expression::Variable("n".to_string());
        let result = expr_validator.validate_expression_cycles(&var_expression);
        assert!(result.is_ok());
        
        // 复杂的嵌套表达式
        let nested_expression = Expression::Binary {
            op: crate::core::BinaryOperator::Add,
            left: Box::new(Expression::Variable("a".to_string())),
            right: Box::new(Expression::Binary {
                op: crate::core::BinaryOperator::Multiply,
                left: Box::new(Expression::Variable("b".to_string())),
                right: Box::new(Expression::Variable("c".to_string())),
            }),
        };
        let result = expr_validator.validate_expression_cycles(&nested_expression);
        assert!(result.is_ok());
    }
}
