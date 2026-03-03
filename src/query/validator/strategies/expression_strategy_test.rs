//! 表达式验证策略测试
//! 测试表达式验证策略的各种功能

#[cfg(test)]
mod expression_strategy_tests {
    use crate::core::types::expression::utils::test_helpers::create_test_contextual_expression;
    use crate::core::types::YieldColumn;
    use crate::core::Expression;
    use crate::core::Value;
    use crate::query::validator::strategies::expression_strategy::ExpressionValidationStrategy;
    use crate::query::validator::structs::*;
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
        let bool_expr = Expression::Literal(Value::Bool(true));
        let bool_expression = create_test_contextual_expression(bool_expr);
        let result = strategy.validate_filter(&bool_expression, &context);
        assert!(result.is_ok());

        // 无效的非布尔表达式
        let int_expr = Expression::Literal(Value::Int(42));
        let int_expression = create_test_contextual_expression(int_expr);
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
        let path_expr = Expression::Path(vec![
            Expression::Label("Person".to_string()),
            Expression::Label("KNOWS".to_string()),
            Expression::Label("Person".to_string()),
        ]);
        let path_expression = create_test_contextual_expression(path_expr);
        let result = strategy.validate_path(&path_expression, &context);
        assert!(result.is_ok());

        // 测试标签表达式（应该返回 Empty 类型，也被接受）
        let label_expr = Expression::Label("Person".to_string());
        let label_expression = create_test_contextual_expression(label_expr);
        let result = strategy.validate_path(&label_expression, &context);
        assert!(result.is_ok());

        // 测试无效的类型（非路径类型）
        let int_expr = Expression::Literal(Value::Int(42));
        let int_expression = create_test_contextual_expression(int_expr);
        let result = strategy.validate_path(&int_expression, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_return() {
        let strategy = ExpressionValidationStrategy::new();
        let mut aliases = HashMap::new();
        aliases.insert("n".to_string(), AliasType::Node);

        let yield_columns = vec![YieldColumn {
            expression: create_test_contextual_expression(Expression::Variable("n".to_string())),
            alias: "n".to_string(),
            is_matched: false,
        }];

        let yield_clause = YieldClauseContext {
            yield_columns,
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

        let yield_columns_clone = yield_clause.yield_columns.clone();

        let context = ReturnClauseContext {
            yield_clause,
            aliases_available: aliases.clone(),
            aliases_generated: HashMap::new(),
            pagination: None,
            order_by: None,
            distinct: false,
            query_parts: vec![],
            errors: vec![],
        };

        // 测试有效的变量引用
        let var_expr = Expression::Variable("n".to_string());
        let var_expression = create_test_contextual_expression(var_expr);
        let result = strategy.validate_return(&var_expression, &yield_columns_clone, &context);
        assert!(result.is_ok());

        // 测试有效的属性访问
        let prop_expr = Expression::Property {
            object: Box::new(Expression::Variable("n".to_string())),
            property: "name".to_string(),
        };
        let prop_expression = create_test_contextual_expression(prop_expr);
        let result = strategy.validate_return(&prop_expression, &yield_columns_clone, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_where() {
        let strategy = ExpressionValidationStrategy::new();
        let mut aliases = HashMap::new();
        aliases.insert("n".to_string(), AliasType::Node);

        let context = WhereClauseContext {
            filter: None,
            aliases_available: aliases.clone(),
            aliases_generated: HashMap::new(),
            paths: vec![],
            query_parts: vec![],
            errors: vec![],
        };

        // 测试有效的布尔表达式
        let bool_expr = Expression::Binary {
            left: Box::new(Expression::Variable("n".to_string())),
            op: crate::core::types::operators::BinaryOperator::Equal,
            right: Box::new(Expression::Literal(Value::String("test".to_string()))),
        };
        let bool_expression = create_test_contextual_expression(bool_expr);
        let result = strategy.validate_filter(&bool_expression, &context);
        assert!(result.is_ok());
    }
}
