//! Cypher表达式解析器
//!
//! 提供完整的Cypher表达式解析功能

use super::ast::*;
use super::parser_core::CypherParserCore;

impl CypherParserCore {
    /// 解析表达式（完整实现）
    pub fn parse_expression_full(&mut self) -> Result<Expression, String> {
        self.parse_or_expression()
    }

    /// 解析OR表达式
    fn parse_or_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_and_expression()?;

        self.skip_whitespace();
        while self.is_current_keyword("OR") {
            self.consume_token(); // 消费 OR
            let right = self.parse_and_expression()?;
            left = Expression::Binary(BinaryExpression {
                left: Box::new(left),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            });
            self.skip_whitespace();
        }

        Ok(left)
    }

    /// 解析AND表达式
    fn parse_and_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_not_expression()?;

        self.skip_whitespace();
        while self.is_current_keyword("AND") {
            self.consume_token(); // 消费 AND
            let right = self.parse_not_expression()?;
            left = Expression::Binary(BinaryExpression {
                left: Box::new(left),
                operator: BinaryOperator::And,
                right: Box::new(right),
            });
            self.skip_whitespace();
        }

        Ok(left)
    }

    /// 解析NOT表达式
    fn parse_not_expression(&mut self) -> Result<Expression, String> {
        self.skip_whitespace();

        if self.is_current_keyword("NOT") {
            self.consume_token(); // 消费 NOT
            let expression = self.parse_not_expression()?;
            Ok(Expression::Unary(UnaryExpression {
                operator: UnaryOperator::Not,
                expression: Box::new(expression),
            }))
        } else {
            self.parse_comparison_expression()
        }
    }

    /// 解析比较表达式
    fn parse_comparison_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_additive_expression()?;

        self.skip_whitespace();
        while let Some(operator) = self.parse_comparison_operator() {
            self.consume_token(); // 消费操作符
            let right = self.parse_additive_expression()?;
            left = Expression::Binary(BinaryExpression {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
            self.skip_whitespace();
        }

        Ok(left)
    }

    /// 解析比较操作符
    fn parse_comparison_operator(&self) -> Option<BinaryOperator> {
        match self.current_token().value.as_str() {
            "=" => Some(BinaryOperator::Equal),
            "==" => Some(BinaryOperator::Equal),
            "!=" => Some(BinaryOperator::NotEqual),
            "<>" => Some(BinaryOperator::NotEqual),
            "<" => Some(BinaryOperator::LessThan),
            "<=" => Some(BinaryOperator::LessThanOrEqual),
            ">" => Some(BinaryOperator::GreaterThan),
            ">=" => Some(BinaryOperator::GreaterThanOrEqual),
            _ => None,
        }
    }

    /// 解析加法表达式
    fn parse_additive_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplicative_expression()?;

        self.skip_whitespace();
        while self.is_current_token_value("+") || self.is_current_token_value("-") {
            let operator = if self.is_current_token_value("+") {
                BinaryOperator::Add
            } else {
                BinaryOperator::Subtract
            };
            self.consume_token(); // 消费操作符
            let right = self.parse_multiplicative_expression()?;
            left = Expression::Binary(BinaryExpression {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
            self.skip_whitespace();
        }

        Ok(left)
    }

    /// 解析乘法表达式
    fn parse_multiplicative_expression(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_unary_expression()?;

        self.skip_whitespace();
        while self.is_current_token_value("*")
            || self.is_current_token_value("/")
            || self.is_current_token_value("%")
        {
            let operator = if self.is_current_token_value("*") {
                BinaryOperator::Multiply
            } else if self.is_current_token_value("/") {
                BinaryOperator::Divide
            } else {
                BinaryOperator::Modulo
            };
            self.consume_token(); // 消费操作符
            let right = self.parse_unary_expression()?;
            left = Expression::Binary(BinaryExpression {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
            self.skip_whitespace();
        }

        Ok(left)
    }

    /// 解析一元表达式
    fn parse_unary_expression(&mut self) -> Result<Expression, String> {
        self.skip_whitespace();

        if self.is_current_token_value("+") {
            self.consume_token(); // 消费 '+'
            let expression = self.parse_unary_expression()?;
            Ok(Expression::Unary(UnaryExpression {
                operator: UnaryOperator::Plus,
                expression: Box::new(expression),
            }))
        } else if self.is_current_token_value("-") {
            self.consume_token(); // 消费 '-'
            let expression = self.parse_unary_expression()?;
            Ok(Expression::Unary(UnaryExpression {
                operator: UnaryOperator::Minus,
                expression: Box::new(expression),
            }))
        } else {
            self.parse_primary_expression()
        }
    }

    /// 解析基本表达式
    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        self.skip_whitespace();

        if self.is_current_token_value("(") {
            self.consume_token(); // 消费 '('
            let expression = self.parse_expression_full()?;
            self.skip_whitespace();
            self.expect_token_value(")")?;
            Ok(expression)
        } else if self.is_current_token_type(super::lexer::TokenType::LiteralString) {
            let value = self.parse_string_literal()?;
            Ok(Expression::Literal(Literal::String(value)))
        } else if self.is_current_token_type(super::lexer::TokenType::LiteralNumber) {
            let value = self.parse_number_literal()?;
            Ok(Expression::Literal(Literal::Integer(value)))
        } else if self.is_current_keyword("TRUE") {
            self.consume_token();
            Ok(Expression::Literal(Literal::Boolean(true)))
        } else if self.is_current_keyword("FALSE") {
            self.consume_token();
            Ok(Expression::Literal(Literal::Boolean(false)))
        } else if self.is_current_keyword("NULL") {
            self.consume_token();
            Ok(Expression::Literal(Literal::Null))
        } else if self.is_current_token_type(super::lexer::TokenType::Identifier) {
            self.parse_identifier_or_function_call()
        } else if self.is_current_token_value("[") {
            self.parse_list_expression()
        } else if self.is_current_token_value("{") {
            self.parse_map_expression()
        } else {
            Err(format!(
                "不支持的表达式: '{}' 在位置 {}",
                self.current_token().value,
                self.current_token().position
            ))
        }
    }

    /// 解析标识符或函数调用
    fn parse_identifier_or_function_call(&mut self) -> Result<Expression, String> {
        let identifier = self.parse_identifier()?;

        self.skip_whitespace();
        if self.is_current_token_value("(") {
            // 函数调用
            self.consume_token(); // 消费 '('
            let arguments = self.parse_function_arguments()?;
            self.skip_whitespace();
            self.expect_token_value(")")?;

            Ok(Expression::FunctionCall(FunctionCall {
                function_name: identifier,
                arguments,
            }))
        } else {
            // 检查是否是属性表达式
            self.skip_whitespace();
            if self.is_current_token_value(".") {
                self.consume_token(); // 消费 '.'
                let property_name = self.parse_identifier()?;

                // 检查是否有更多属性访问
                self.skip_whitespace();
                if self.is_current_token_value(".") {
                    self.consume_token(); // 消费 '.'
                    let next_property = self.parse_identifier()?;
                    Ok(Expression::Property(PropertyExpression {
                        expression: Box::new(Expression::Property(PropertyExpression {
                            expression: Box::new(Expression::Variable(identifier)),
                            property_name,
                        })),
                        property_name: next_property,
                    }))
                } else {
                    Ok(Expression::Property(PropertyExpression {
                        expression: Box::new(Expression::Variable(identifier)),
                        property_name,
                    }))
                }
            } else {
                Ok(Expression::Variable(identifier))
            }
        }
    }

    /// 解析函数参数（公共接口）
    pub fn parse_function_arguments_public(&mut self) -> Result<Vec<Expression>, String> {
        self.parse_function_arguments()
    }

    /// 解析函数参数
    fn parse_function_arguments(&mut self) -> Result<Vec<Expression>, String> {
        let mut arguments = Vec::new();

        self.skip_whitespace();
        if !self.is_current_token_value(")") {
            let expression = self.parse_expression_full()?;
            arguments.push(expression);

            self.skip_whitespace();
            while self.is_current_token_value(",") {
                self.consume_token(); // 消费 ','
                self.skip_whitespace();
                let expression = self.parse_expression_full()?;
                arguments.push(expression);
                self.skip_whitespace();
            }
        }

        Ok(arguments)
    }

    /// 解析列表表达式
    fn parse_list_expression(&mut self) -> Result<Expression, String> {
        self.consume_token(); // 消费 '['
        let mut elements = Vec::new();

        self.skip_whitespace();
        if !self.is_current_token_value("]") {
            let element = self.parse_expression_full()?;
            elements.push(element);

            self.skip_whitespace();
            while self.is_current_token_value(",") {
                self.consume_token(); // 消费 ','
                self.skip_whitespace();
                let element = self.parse_expression_full()?;
                elements.push(element);
                self.skip_whitespace();
            }
        }

        self.expect_token_value("]")?;
        Ok(Expression::List(ListExpression { elements }))
    }

    /// 解析映射表达式
    fn parse_map_expression(&mut self) -> Result<Expression, String> {
        self.consume_token(); // 消费 '{'
        let mut properties = std::collections::HashMap::new();

        self.skip_whitespace();
        if !self.is_current_token_value("}") {
            let key = self.parse_identifier()?;
            self.skip_whitespace();
            self.expect_token_value(":")?;
            let value = self.parse_expression_full()?;
            properties.insert(key, value);

            self.skip_whitespace();
            while self.is_current_token_value(",") {
                self.consume_token(); // 消费 ','
                self.skip_whitespace();
                let key = self.parse_identifier()?;
                self.skip_whitespace();
                self.expect_token_value(":")?;
                let value = self.parse_expression_full()?;
                properties.insert(key, value);
                self.skip_whitespace();
            }
        }

        self.expect_token_value("}")?;
        Ok(Expression::Map(MapExpression { properties }))
    }

    /// 解析CASE表达式
    pub fn parse_case_expression(&mut self) -> Result<Expression, String> {
        self.expect_keyword("CASE")?;

        let mut expression = None;
        let mut alternatives = Vec::new();
        let mut default_alternative = None;

        // 检查是否有CASE表达式
        self.skip_whitespace();
        if !self.is_current_keyword("WHEN") {
            expression = Some(Box::new(self.parse_expression_full()?));
            self.skip_whitespace();
        }

        // 解析WHEN-THEN子句
        while self.is_current_keyword("WHEN") {
            self.consume_token(); // 消费 WHEN
            let when_expression = self.parse_expression_full()?;
            self.skip_whitespace();
            self.expect_keyword("THEN")?;
            let then_expression = self.parse_expression_full()?;

            alternatives.push(CaseAlternative {
                when_expression,
                then_expression,
            });

            self.skip_whitespace();
        }

        // 解析ELSE子句
        if self.is_current_keyword("ELSE") {
            self.consume_token(); // 消费 ELSE
            default_alternative = Some(Box::new(self.parse_expression_full()?));
            self.skip_whitespace();
        }

        self.expect_keyword("END")?;

        Ok(Expression::Case(CaseExpression {
            expression,
            alternatives,
            default_alternative,
        }))
    }

    /// 解析模式表达式
    pub fn parse_pattern_expression(&mut self) -> Result<Expression, String> {
        // 这里需要调用模式解析器，暂时简化实现
        Err("模式表达式解析尚未实现".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_expression() {
        let mut parser = CypherParserCore::new("n.name".to_string());
        let expression = parser
            .parse_expression_full()
            .expect("Expression parser should parse valid expressions");

        match expression {
            Expression::Property(prop) => {
                assert_eq!(prop.property_name, "name");
                match *prop.expression {
                    Expression::Variable(var) => assert_eq!(var, "n"),
                    _ => panic!("Expected variable"),
                }
            }
            _ => panic!("Expected property expression"),
        }
    }

    #[test]
    fn test_parse_comparison_expression() {
        let mut parser = CypherParserCore::new("n.age > 30".to_string());
        let expression = parser
            .parse_expression_full()
            .expect("Expression parser should parse valid expressions");

        match expression {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::GreaterThan);
                match *binary.left {
                    Expression::Property(prop) => {
                        assert_eq!(prop.property_name, "age");
                        match *prop.expression {
                            Expression::Variable(var) => assert_eq!(var, "n"),
                            _ => panic!("Expected variable"),
                        }
                    }
                    _ => panic!("Expected property expression"),
                }
                match *binary.right {
                    Expression::Literal(Literal::Integer(value)) => assert_eq!(value, 30),
                    _ => panic!("Expected integer literal"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_parse_logical_expression() {
        let mut parser = CypherParserCore::new("n.age > 30 AND n.name = \"Alice\"".to_string());
        let expression = parser
            .parse_expression_full()
            .expect("Expression parser should parse valid expressions");

        match expression {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::And);
                // 进一步验证左右表达式...
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let mut parser = CypherParserCore::new("count(n)".to_string());
        let expression = parser
            .parse_expression_full()
            .expect("Expression parser should parse valid expressions");

        match expression {
            Expression::FunctionCall(func) => {
                assert_eq!(func.function_name, "count");
                assert_eq!(func.arguments.len(), 1);
                match &func.arguments[0] {
                    Expression::Variable(var) => assert_eq!(var, "n"),
                    _ => panic!("Expected variable argument"),
                }
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_parse_list_expression() {
        let mut parser = CypherParserCore::new("[1, 2, 3]".to_string());
        let expression = parser
            .parse_expression_full()
            .expect("Expression parser should parse valid expressions");

        match expression {
            Expression::List(list) => {
                assert_eq!(list.elements.len(), 3);
                // 进一步验证列表元素...
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_parse_map_expression() {
        let mut parser = CypherParserCore::new("{name: \"Alice\", age: 30}".to_string());
        let expression = parser
            .parse_expression_full()
            .expect("Expression parser should parse valid expressions");

        match expression {
            Expression::Map(map) => {
                assert_eq!(map.properties.len(), 2);
                // 进一步验证映射属性...
            }
            _ => panic!("Expected map expression"),
        }
    }
}
