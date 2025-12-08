use std::collections::HashMap;
use crate::core::{Value, Vertex, Edge, NullType};

/// Represents an expression in a query
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Constant(Value),
    Property(String),  // Property name to access
    Function(String, Vec<Expression>),  // Function name and arguments
    BinaryOp(Box<Expression>, BinaryOperator, Box<Expression>),
    UnaryOp(UnaryOperator, Box<Expression>),
}

impl Expression {
    /// Get the kind of this expression
    pub fn kind(&self) -> ExpressionKind {
        match self {
            Expression::Constant(_) => ExpressionKind::Constant,
            Expression::Property(_) => ExpressionKind::Variable,
            Expression::Function(name, _) => {
                // Could be more specific based on function name, but for now we'll use FunctionCall
                ExpressionKind::FunctionCall
            },
            Expression::BinaryOp(_, _, _) => ExpressionKind::Arithmetic, // Could be more specific based on operator
            Expression::UnaryOp(_, _) => ExpressionKind::UnaryPlus, // Could be more specific based on operator
        }
    }

    /// Get child expressions
    pub fn children(&self) -> Vec<&Expression> {
        match self {
            Expression::Constant(_) => vec![],
            Expression::Property(_) => vec![],
            Expression::Function(_, args) => {
                args.iter().collect()
            },
            Expression::BinaryOp(left, _, right) => {
                vec![left.as_ref(), right.as_ref()]
            },
            Expression::UnaryOp(_, operand) => {
                vec![operand.as_ref()]
            },
        }
    }
}

/// A simplified version of the ExpressionKind enum for expression analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpressionKind {
    // 属性表达式类型
    TagProperty,
    EdgeProperty,
    InputProperty,
    VariableProperty,
    DestinationProperty,
    SourceProperty,

    // 二元表达式类型
    Arithmetic,
    Relational,
    Logical,

    // 一元表达式类型
    UnaryPlus,
    UnaryNegate,
    UnaryNot,
    UnaryInvert,

    // 函数调式
    FunctionCall,

    // 常量
    Constant,

    // 变量
    Variable,

    // 参数 (simplified as Variable for now)
    Parameter,

    // 其他类型
    Aggregate,
    TypeCasting,
    Label,

    // 容器类型
    List,
    Set,
    Map,
}

/// Binary operators for expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    // Arithmetic operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Relational operations
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical operations
    And,
    Or,
    Xor,
    // Other operations
    In,
    NotIn,
    Subscript,
    Attribute,
    Contains,
    StartsWith,
    EndsWith,
}

/// Unary operators for expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Not,
    Increment,
    Decrement,
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
                    BinaryOperator::Mod => self.mod_values(left_val, right_val),
                    BinaryOperator::Eq => Ok(Value::Bool(left_val == right_val)),
                    BinaryOperator::Ne => Ok(Value::Bool(left_val != right_val)),
                    BinaryOperator::Lt => self.cmp_values(left_val, right_val, |a, b| a < b),
                    BinaryOperator::Le => self.cmp_values(left_val, right_val, |a, b| a <= b),
                    BinaryOperator::Gt => self.cmp_values(left_val, right_val, |a, b| a > b),
                    BinaryOperator::Ge => self.cmp_values(left_val, right_val, |a, b| a >= b),
                    BinaryOperator::And => self.and_values(left_val, right_val),
                    BinaryOperator::Or => self.or_values(left_val, right_val),
                    BinaryOperator::Xor => self.xor_values(left_val, right_val),
                    BinaryOperator::In => self.in_values(left_val, right_val),
                    BinaryOperator::NotIn => self.not_in_values(left_val, right_val),
                    BinaryOperator::Subscript => self.subscript_values(left_val, right_val),
                    BinaryOperator::Attribute => self.attribute_values(left_val, right_val),
                    BinaryOperator::Contains => self.contains_values(left_val, right_val),
                    BinaryOperator::StartsWith => self.starts_with_values(left_val, right_val),
                    BinaryOperator::EndsWith => self.ends_with_values(left_val, right_val),
                }
            },
            Expression::UnaryOp(op, operand) => {
                let operand_val = self.evaluate(operand, context)?;
                
                match op {
                    UnaryOperator::Plus => Ok(operand_val),  // Identity operation
                    UnaryOperator::Minus => self.neg_value(operand_val),
                    UnaryOperator::Not => Ok(Value::Bool(!self.value_to_bool(&operand_val))),
                    UnaryOperator::Increment => Err(ExpressionError::InvalidOperation("Increment operation not supported".to_string())),
                    UnaryOperator::Decrement => Err(ExpressionError::InvalidOperation("Decrement operation not supported".to_string())),
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
    
    fn mod_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    return Err(ExpressionError::InvalidOperation("Division by zero".to_string()));
                }
                Ok(Value::Int(a % b))
            },
            (Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    return Err(ExpressionError::InvalidOperation("Division by zero".to_string()));
                }
                Ok(Value::Float(a % b))
            },
            (Value::Int(a), Value::Float(b)) => {
                if b == 0.0 {
                    return Err(ExpressionError::InvalidOperation("Division by zero".to_string()));
                }
                Ok(Value::Float((a as f64) % b))
            },
            (Value::Float(a), Value::Int(b)) => {
                if b == 0 {
                    return Err(ExpressionError::InvalidOperation("Division by zero".to_string()));
                }
                Ok(Value::Float(a % (b as f64)))
            },
            _ => Err(ExpressionError::TypeError("Cannot perform mod operation on these value types".to_string())),
        }
    }

    fn xor_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        let left_bool = self.value_to_bool(&left);
        let right_bool = self.value_to_bool(&right);
        Ok(Value::Bool(left_bool ^ right_bool))  // XOR operation
    }

    fn in_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        match right {
            Value::List(items) => {
                let found = items.iter().any(|item| *item == left);
                Ok(Value::Bool(found))
            },
            Value::Set(items) => {
                Ok(Value::Bool(items.contains(&left)))
            },
            Value::Map(items) => {
                if let Value::String(key) = &left {
                    Ok(Value::Bool(items.contains_key(key)))
                } else {
                    Err(ExpressionError::TypeError("Key for 'in' operation on map must be a string".to_string()))
                }
            },
            _ => Err(ExpressionError::TypeError("Right operand of 'in' must be a list, set, or map".to_string())),
        }
    }

    fn not_in_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        match self.in_values(left, right) {
            Ok(Value::Bool(b)) => Ok(Value::Bool(!b)),
            Ok(_) => Err(ExpressionError::TypeError("in_values should return boolean".to_string())),
            Err(e) => Err(e),
        }
    }

    fn subscript_values(&self, collection: Value, index: Value) -> Result<Value, ExpressionError> {
        match collection {
            Value::List(items) => {
                if let Value::Int(i) = index {
                    if i >= 0 && (i as usize) < items.len() {
                        Ok(items[i as usize].clone())
                    } else {
                        Err(ExpressionError::InvalidOperation("List index out of bounds".to_string()))
                    }
                } else {
                    Err(ExpressionError::TypeError("List index must be an integer".to_string()))
                }
            },
            Value::Map(items) => {
                if let Value::String(key) = index {
                    match items.get(&key) {
                        Some(value) => Ok(value.clone()),
                        None => Ok(Value::Null(NullType::Null)),
                    }
                } else {
                    Err(ExpressionError::TypeError("Map key must be a string".to_string()))
                }
            },
            _ => Err(ExpressionError::TypeError("Subscript operation requires a list or map".to_string())),
        }
    }

    fn attribute_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        // For simplicity, treat this like a subscript operation for now
        // In a real system, this would access object properties
        match (&left, &right) {
            (Value::Map(m), Value::String(key)) => {
                match m.get(key) {
                    Some(value) => Ok(value.clone()),
                    None => Ok(Value::Null(NullType::Null)),
                }
            },
            _ => Err(ExpressionError::TypeError("Attribute access requires a map and string key".to_string())),
        }
    }

    fn contains_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        // Check if 'left' contains 'right'
        match (&left, &right) {
            (Value::List(items), item) => {
                Ok(Value::Bool(items.contains(item)))
            },
            (Value::Set(items), item) => {
                Ok(Value::Bool(items.contains(item)))
            },
            (Value::String(s), Value::String(substring)) => {
                Ok(Value::Bool(s.contains(substring)))
            },
            _ => Err(ExpressionError::TypeError("Contains operation not supported for these types".to_string())),
        }
    }

    fn starts_with_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        match (&left, &right) {
            (Value::String(s), Value::String(prefix)) => {
                Ok(Value::Bool(s.starts_with(prefix)))
            },
            _ => Err(ExpressionError::TypeError("Starts with operation requires string operands".to_string())),
        }
    }

    fn ends_with_values(&self, left: Value, right: Value) -> Result<Value, ExpressionError> {
        match (&left, &right) {
            (Value::String(s), Value::String(suffix)) => {
                Ok(Value::Bool(s.ends_with(suffix)))
            },
            _ => Err(ExpressionError::TypeError("Ends with operation requires string operands".to_string())),
        }
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