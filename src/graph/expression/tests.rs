#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::core::{Value, Vertex, Tag, NullType};
    use crate::graph::expression::{Expression, ExpressionEvaluator, EvalContext, expr_type::BinaryOperator};

    #[test]
    fn test_constant_evaluation() {
        let evaluator = ExpressionEvaluator;
        let expr = Expression::Constant(Value::Int(42));
        let context = EvalContext::new();

        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_binary_operation() {
        let evaluator = ExpressionEvaluator;
        let expr = Expression::BinaryOp(
            Box::new(Expression::Constant(Value::Int(10))),
            BinaryOperator::Add,
            Box::new(Expression::Constant(Value::Int(5))),
        );
        let context = EvalContext::new();

        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(15));
    }

    #[test]
    fn test_property_access() {
        let evaluator = ExpressionEvaluator;
        let mut props = HashMap::new();
        props.insert("age".to_string(), Value::Int(25));
        let tag = Tag::new("person".to_string(), props);
        let vertex = Vertex::new(Value::Int(1), vec![tag]);
        let context = EvalContext::with_vertex(&vertex);

        let expr = Expression::Property("age".to_string());
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(25));
    }

    #[test]
    fn test_list_container() {
        let evaluator = ExpressionEvaluator;
        let expr = Expression::List(vec![
            Expression::Constant(Value::Int(1)),
            Expression::Constant(Value::Int(2)),
            Expression::Constant(Value::Int(3)),
        ]);
        let context = EvalContext::new();

        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    }
}