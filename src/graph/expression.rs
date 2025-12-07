use std::collections::HashMap;
use crate::core::{Value, Vertex, Edge};

/// Represents an expression in a query
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Constant(Value),
    Property(String),  // Property name to access
    Function(String, Vec<Expression>),  // Function name and arguments
    BinaryOp(Box<Expression>, BinaryOperator, Box<Expression>),
    UnaryOp(UnaryOperator, Box<Expression>),
}

/// Binary operators for expressions
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

/// Unary operators for expressions
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Neg,
    Not,
}

/// Context for evaluating expressions, containing values for variables/properties
pub struct EvalContext<'a> {
    pub vertex: Option<&'a Vertex>,
    pub edge: Option<&'a Edge>,
    pub vars: HashMap<String, Value>,
}

impl<'a> EvalContext<'a> {
    pub fn new() -> Self {
        Self {
            vertex: None,
            edge: None,
            vars: HashMap::new(),
        }
    }

    pub fn with_vertex(vertex: &'a Vertex) -> Self {
        Self {
            vertex: Some(vertex),
            edge: None,
            vars: HashMap::new(),
        }
    }

    pub fn with_edge(edge: &'a Edge) -> Self {
        Self {
            vertex: None,
            edge: Some(edge),
            vars: HashMap::new(),
        }
    }
}

/// Expression evaluation error
#[derive(Debug, thiserror::Error)]
pub enum ExpressionError {
    #[error("Type error: {0}")]
    TypeError(String),
    #[error("Property not found: {0}")]
    PropertyNotFound(String),
    #[error("Function error: {0}")]
    FunctionError(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Expression evaluator
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// Evaluate an expression in the given context
    pub fn evaluate(&self, expr: &Expression, context: &EvalContext) -> Result<Value, ExpressionError> {
        match expr {
            Expression::Constant(value) => Ok(value.clone()),
            Expression::Property(prop_name) => {
                // Try to find the property in vertex
                if let Some(vertex) = context.vertex {
                    for tag in &vertex.tags {
                        if let Some(value) = tag.properties.get(prop_name) {
                            return Ok(value.clone());
                        }
                    }
                }
                
                // Try to find the property in edge
                if let Some(edge) = context.edge {
                    if let Some(value) = edge.props.get(prop_name) {
                        return Ok(value.clone());
                        }
                }
                
                // Try to find the property in variables
                if let Some(value) = context.vars.get(prop_name) {
                    return Ok(value.clone());
                }
                
                Err(ExpressionError::PropertyNotFound(prop_name.clone()))
            },
            Expression::BinaryOp(left, op, right) => {
                let left_val = self.evaluate(left, context)?;
                let right_val = self.evaluate(right, context)?;
                
                match op {
                    BinaryOperator::Add => self.add_values(left_val, right_val),
                    BinaryOperator::Sub => self.sub_values(left_val, right_val),
                    BinaryOperator::Mul => self.mul_values(left_val, right_val),
                    BinaryOperator::Div => self.div_values(left_val, right_val),
                    BinaryOperator::Eq => Ok(Value::Bool(left_val == right_val)),
                    BinaryOperator::Ne => Ok(Value::Bool(left_val != right_val)),
                    BinaryOperator::Lt => self.cmp_values(left_val, right_val, |a, b| a < b),
                    BinaryOperator::Le => self.cmp_values(left_val, right_val, |a, b| a <= b),
                    BinaryOperator::Gt => self.cmp_values(left_val, right_val, |a, b| a > b),
                    BinaryOperator::Ge => self.cmp_values(left_val, right_val, |a, b| a >= b),
                    BinaryOperator::And => self.and_values(left_val, right_val),
                    BinaryOperator::Or => self.or_values(left_val, right_val),
                }
            },
            Expression::UnaryOp(op, operand) => {
                let operand_val = self.evaluate(operand, context)?;
                
                match op {
                    UnaryOperator::Neg => self.neg_value(operand_val),
                    UnaryOperator::Not => Ok(Value::Bool(!self.value_to_bool(&operand_val))),
                }
            },
            Expression::Function(func_name, args) => {
                self.call_function(func_name, args, context)
            },
        }
    }
    
    fn add_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::String(a), Value::Int(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::Int(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            // Add more combinations as needed
            _ => Err(ExpressionError::TypeError("Cannot add these value types".to_string())),
        }
    }
    
    fn sub_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - b as f64)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
            _ => Err(ExpressionError::TypeError("Cannot subtract these value types".to_string())),
        }
    }
    
    fn mul_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * b as f64)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
            _ => Err(ExpressionError::TypeError("Cannot multiply these value types".to_string())),
        }
    }
    
    fn div_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) if b != 0 => Ok(Value::Int(a / b)),
            (Value::Float(a), Value::Float(b)) if b != 0.0 => Ok(Value::Float(a / b)),
            (Value::Float(a), Value::Int(b)) if b != 0 => Ok(Value::Float(a / b as f64)),
            (Value::Int(a), Value::Float(b)) if b != 0.0 => Ok(Value::Float(a as f64 / b)),
            _ => Err(ExpressionError::TypeError("Cannot divide these value types or division by zero".to_string())),
        }
    }
    
    fn cmp_values<F>(&self, left: Value, right: Value, cmp_fn: F) -> Result<Value, ExpressionError>
    where
        F: Fn(&Value, &Value) -> bool,
    {
        Ok(Value::Bool(cmp_fn(&left, &right)))
    }
    
    fn and_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        let left_bool = self.value_to_bool(&left);
        let right_bool = self.value_to_bool(&right);
        Ok(Value::Bool(left_bool && right_bool))
    }
    
    fn or_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        let left_bool = self.value_to_bool(&left);
        let right_bool = self.value_to_bool(&right);
        Ok(Value::Bool(left_bool || right_bool))
    }
    
    fn neg_value(&self, value: Value) -> Result<Value, ExpressionError> {
        match value {
            Value::Int(n) => Ok(Value::Int(-n)),
            Value::Float(n) => Ok(Value::Float(-n)),
            _ => Err(ExpressionError::TypeError("Cannot negate this value type".to_string())),
        }
    }
    
    fn value_to_bool(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0 && !f.is_nan(),
            Value::String(s) => !s.is_empty(),
            Value::Null(_) => false,
            Value::Empty => false,
            _ => true, // Default to true for other types
        }
    }
    
    fn call_function(&self, func_name: &str, args: &[Expression], context: &EvalContext) -> Result<Value, ExpressionError> {
        match func_name.to_lowercase().as_str() {
            "has_property" => {
                if args.len() != 1 {
                    return Err(ExpressionError::FunctionError("has_property expects 1 argument".to_string()));
                }
                
                let prop_expr = &args[0];
                let prop_name_val = self.evaluate(prop_expr, context)?;
                let prop_name = match prop_name_val {
                    Value::String(name) => name,
                    _ => return Err(ExpressionError::FunctionError("Property name must be a string".to_string())),
                };
                
                // Check if property exists in vertex
                let exists = if let Some(vertex) = context.vertex {
                    vertex.tags.iter().any(|tag| tag.properties.contains_key(&prop_name))
                } else if let Some(edge) = context.edge {
                    edge.props.contains_key(&prop_name)
                } else {
                    false
                };
                
                Ok(Value::Bool(exists))
            },
            "coalesce" => {
                for arg in args {
                    let val = self.evaluate(arg, context)?;
                    if !self.is_null_value(&val) {
                        return Ok(val);
                    }
                }
                Ok(Value::Null(crate::core::NullType::Null))
            },
            _ => Err(ExpressionError::FunctionError(format!("Unknown function: {}", func_name))),
        }
    }
    
    fn is_null_value(&self, value: &Value) -> bool {
        matches!(value, Value::Null(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Tag, NullType};

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
    fn test_has_property_function() {
        let evaluator = ExpressionEvaluator;
        let mut props = HashMap::new();
        props.insert("age".to_string(), Value::Int(25));
        let tag = Tag::new("person".to_string(), props);
        let vertex = Vertex::new(Value::Int(1), vec![tag]);
        let context = EvalContext::with_vertex(&vertex);
        
        let args = vec![Expression::Constant(Value::String("age".to_string()))];
        let expr = Expression::Function("has_property".to_string(), args);
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }
}