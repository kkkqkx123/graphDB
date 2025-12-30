//! 表达式解析器 (v2)

use super::*;
use crate::core::Value;
use crate::query::parser::lexer::{Lexer, TokenKind as LexerToken};

/// 表达式解析器
pub struct ExprParser {
    lexer: Lexer,
}

impl ExprParser {
    /// 创建表达式解析器
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    /// 解析表达式
    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_or_expression()
    }

    /// 解析 OR 表达式
    fn parse_or_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expression()?;

        while self.match_token(LexerToken::Or) {
            let op = BinaryOp::Or;
            let right = self.parse_and_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析 AND 表达式
    fn parse_and_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not_expression()?;

        while self.match_token(LexerToken::And) {
            let op = BinaryOp::And;
            let right = self.parse_not_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析 NOT 表达式
    fn parse_not_expression(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(LexerToken::Not) {
            let op = UnaryOp::Not;
            let operand = self.parse_not_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else {
            self.parse_comparison_expression()
        }
    }

    /// 解析比较表达式
    fn parse_comparison_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive_expression()?;

        // 检查比较操作符
        if let Some(op) = self.parse_comparison_op() {
            let right = self.parse_additive_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析加法表达式
    fn parse_additive_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative_expression()?;

        while let Some(op) = self.parse_additive_op() {
            let right = self.parse_multiplicative_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析乘法表达式
    fn parse_multiplicative_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expression()?;

        while let Some(op) = self.parse_multiplicative_op() {
            let right = self.parse_unary_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析一元表达式
    fn parse_unary_expression(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(LexerToken::Minus) {
            let op = UnaryOp::Minus;
            let operand = self.parse_unary_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else if self.match_token(LexerToken::Plus) {
            let op = UnaryOp::Plus;
            let operand = self.parse_unary_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else {
            self.parse_postfix_expression()
        }
    }

    /// 解析后缀表达式
    fn parse_postfix_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            if self.match_token(LexerToken::LBracket) {
                // 下标访问
                let index = self.parse_expression()?;
                self.expect_token(LexerToken::RBracket)?;
                let span = Span::new(expr.span().start, self.lexer.current_position());
                expr = Expr::Subscript(SubscriptExpr::new(expr, index, span));
            } else if self.match_token(LexerToken::Dot) {
                // 属性访问
                let property = self.expect_identifier()?;
                let span = Span::new(expr.span().start, self.lexer.current_position());
                expr = Expr::PropertyAccess(PropertyAccessExpr::new(expr, property, span));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// 解析基本表达式
    fn parse_primary_expression(&mut self) -> Result<Expr, ParseError> {
        let token = self.lexer.peek()?;

        match token.kind {
            LexerToken::IntegerLiteral(_) => {
                let value = self.parse_integer()?;
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::Int(value), span)))
            }
            LexerToken::Float => {
                let value = self.parse_float()?;
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::Float(value), span)))
            }
            LexerToken::String => {
                let value = self.parse_string()?;
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(
                    Value::String(value),
                    span,
                )))
            }
            LexerToken::BooleanLiteral(_) => {
                let value = self.parse_boolean()?;
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::Bool(value), span)))
            }
            LexerToken::Identifier(_) => {
                let name = self.expect_identifier()?;
                let span = self.current_span();

                // 检查是否是函数调用
                if self.match_token(LexerToken::LParen) {
                    self.parse_function_call(name, span)
                } else {
                    Ok(Expr::Variable(VariableExpr::new(name, span)))
                }
            }
            LexerToken::LParen => {
                // 括号表达式
                self.lexer.advance();
                let expr = self.parse_expression()?;
                self.expect_token(LexerToken::RParen)?;
                Ok(expr)
            }
            LexerToken::LBracket => {
                // 列表表达式
                self.parse_list_expression()
            }
            LexerToken::LBrace => {
                // 映射表达式
                self.parse_map_expression()
            }
            _ => Err(self.parse_error(format!("Unexpected token: {:?}", token.kind))),
        }
    }

    /// 解析函数调用
    fn parse_function_call(&mut self, name: String, span: Span) -> Result<Expr, ParseError> {
        let mut args = Vec::new();
        let mut distinct = false;

        // 检查 DISTINCT 关键字
        if self.match_token(LexerToken::Distinct) {
            distinct = true;
        }

        // 解析参数列表
        if !self.check_token(LexerToken::RParen) {
            loop {
                let arg = self.parse_expression()?;
                args.push(arg);

                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }

        self.expect_token(LexerToken::RParen)?;
        let end_span = self.current_span();
        let full_span = Span::new(span.start, end_span.end);

        Ok(Expr::FunctionCall(FunctionCallExpr::new(
            name, args, distinct, full_span,
        )))
    }

    /// 解析列表表达式
    fn parse_list_expression(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::LBracket)?;

        let mut elements = Vec::new();

        if !self.check_token(LexerToken::RBracket) {
            loop {
                let elem = self.parse_expression()?;
                elements.push(elem);

                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }

        self.expect_token(LexerToken::RBracket)?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Expr::List(ListExpr::new(elements, span)))
    }

    /// 解析映射表达式
    fn parse_map_expression(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::LBrace)?;

        let mut pairs = Vec::new();

        if !self.check_token(LexerToken::RBrace) {
            loop {
                let key = self.expect_identifier()?;
                self.expect_token(LexerToken::Colon)?;
                let value = self.parse_expression()?;
                pairs.push((key, value));

                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }

        self.expect_token(LexerToken::RBrace)?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Expr::Map(MapExpr::new(pairs, span)))
    }

    /// 辅助方法

    fn match_token(&mut self, expected: LexerToken) -> bool {
        if self.lexer.check(expected.clone()) {
            let _ = self.lexer.advance();
            true
        } else {
            false
        }
    }

    fn check_token(&mut self, expected: LexerToken) -> bool {
        self.lexer.check(expected.clone())
    }

    fn expect_token(&mut self, expected: LexerToken) -> Result<(), ParseError> {
        let token = self.lexer.peek()?;
        if token.kind == expected {
            self.lexer.advance();
            Ok(())
        } else {
            Err(self.parse_error(format!("Expected {:?}, found {:?}", expected, token.kind)))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::Identifier(_) = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            Ok(text)
        } else {
            let span = self.current_span();
            Err(ParseError::new(
                format!("Expected identifier, found {:?}", token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    fn parse_integer(&mut self) -> Result<i64, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::IntegerLiteral(_) = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            text.parse().map_err(|_| {
                let span = self.current_span();
                ParseError::new(
                    format!("Invalid integer: {}", text),
                    span.start.line,
                    span.start.column,
                )
            })
        } else {
            let span = self.current_span();
            Err(ParseError::new(
                format!("Expected integer, found {:?}", token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    fn parse_float(&mut self) -> Result<f64, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::Float = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            text.parse().map_err(|_| {
                let span = self.current_span();
                ParseError::new(
                    format!("Invalid float: {}", text),
                    span.start.line,
                    span.start.column,
                )
            })
        } else {
            let span = self.current_span();
            Err(ParseError::new(
                format!("Expected float, found {:?}", token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::String = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            Ok(text.trim_matches('"').to_string())
        } else {
            let span = self.current_span();
            Err(ParseError::new(
                format!("Expected string, found {:?}", token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    fn parse_boolean(&mut self) -> Result<bool, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::BooleanLiteral(_) = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            text.parse().map_err(|_| {
                let span = self.current_span();
                ParseError::new(
                    format!("Invalid boolean: {}", text),
                    span.start.line,
                    span.start.column,
                )
            })
        } else {
            let span = self.current_span();
            Err(ParseError::new(
                format!("Expected boolean, found {:?}", token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    fn parse_comparison_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Eq) {
            Some(BinaryOp::Equal)
        } else if self.match_token(LexerToken::Ne) {
            Some(BinaryOp::NotEqual)
        } else if self.match_token(LexerToken::Lt) {
            Some(BinaryOp::LessThan)
        } else if self.match_token(LexerToken::Le) {
            Some(BinaryOp::LessThanOrEqual)
        } else if self.match_token(LexerToken::Gt) {
            Some(BinaryOp::GreaterThan)
        } else if self.match_token(LexerToken::Ge) {
            Some(BinaryOp::GreaterThanOrEqual)
        } else {
            None
        }
    }

    fn parse_additive_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Plus) {
            Some(BinaryOp::Add)
        } else if self.match_token(LexerToken::Minus) {
            Some(BinaryOp::Subtract)
        } else {
            None
        }
    }

    fn parse_multiplicative_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Star) {
            Some(BinaryOp::Multiply)
        } else if self.match_token(LexerToken::Div) {
            Some(BinaryOp::Divide)
        } else if self.match_token(LexerToken::Mod) {
            Some(BinaryOp::Modulo)
        } else {
            None
        }
    }

    fn current_span(&self) -> Span {
        let pos = self.lexer.current_position();
        Span::new(
            Position::new(pos.line, pos.column),
            Position::new(pos.line, pos.column),
        )
    }

    fn current_position(&self) -> (usize, usize) {
        let pos = self.lexer.current_position();
        (pos.line, pos.column)
    }

    fn parse_error(&self, message: String) -> ParseError {
        let (line, column) = self.current_position();
        ParseError::new(message, line, column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_constant() {
        let mut parser = ExprParser::new("42");
        let result = parser.parse_expression();
        assert!(result.is_ok());

        if let Ok(Expr::Constant(e)) = result {
            assert_eq!(e.value, Value::Int(42));
        } else {
            panic!("Expected constant expression");
        }
    }

    #[test]
    fn test_parse_variable() {
        let mut parser = ExprParser::new("x");
        let result = parser.parse_expression();
        assert!(result.is_ok());

        if let Ok(Expr::Variable(e)) = result {
            assert_eq!(e.name, "x");
        } else {
            panic!("Expected variable expression");
        }
    }

    #[test]
    fn test_parse_binary() {
        let mut parser = ExprParser::new("5 + 3");
        let result = parser.parse_expression();
        assert!(result.is_ok());

        if let Ok(Expr::Binary(e)) = result {
            assert_eq!(e.op, BinaryOp::Add);
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_parse_function_call() {
        let mut parser = ExprParser::new("COUNT(x)");
        let result = parser.parse_expression();
        assert!(result.is_ok());

        if let Ok(Expr::FunctionCall(e)) = result {
            assert_eq!(e.name, "COUNT");
            assert_eq!(e.args.len(), 1);
        } else {
            panic!("Expected function call expression");
        }
    }

    #[test]
    fn test_parse_list() {
        let mut parser = ExprParser::new("[1, 2, 3]");
        let result = parser.parse_expression();
        assert!(result.is_ok());

        if let Ok(Expr::List(e)) = result {
            assert_eq!(e.elements.len(), 3);
        } else {
            panic!("Expected list expression");
        }
    }

    #[test]
    fn test_parse_map() {
        let mut parser = ExprParser::new("{name: \"John\", age: 30}");
        let result = parser.parse_expression();
        assert!(result.is_ok());

        if let Ok(Expr::Map(e)) = result {
            assert_eq!(e.pairs.len(), 2);
        } else {
            panic!("Expected map expression");
        }
    }

    #[test]
    fn test_parse_property_access() {
        let mut parser = ExprParser::new("node.name");
        let result = parser.parse_expression();
        assert!(result.is_ok());

        if let Ok(Expr::PropertyAccess(e)) = result {
            assert_eq!(e.property, "name");
        } else {
            panic!("Expected property access expression");
        }
    }

    #[test]
    fn test_parse_subscript() {
        let mut parser = ExprParser::new("list[0]");
        let result = parser.parse_expression();
        assert!(result.is_ok());

        if let Ok(Expr::Subscript(_e)) = result {
            // 验证下标访问
        } else {
            panic!("Expected subscript expression");
        }
    }
}
