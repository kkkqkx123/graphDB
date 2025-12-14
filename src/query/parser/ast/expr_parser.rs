//! 表达式解析器 (v2)

use super::*;
use crate::query::parser::lexer::{Lexer, Token as LexerToken};

impl ParserV2 {
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
            left = ExprFactory::binary(left, op, right, span);
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
            left = ExprFactory::binary(left, op, right, span);
        }
        
        Ok(left)
    }
    
    /// 解析 NOT 表达式
    fn parse_not_expression(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(LexerToken::Not) {
            let op = UnaryOp::Not;
            let operand = self.parse_not_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(ExprFactory::unary(op, operand, span))
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
            left = ExprFactory::binary(left, op, right, span);
        }
        
        Ok(left)
    }
    
    /// 解析加法表达式
    fn parse_additive_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative_expression()?;
        
        while let Some(op) = self.parse_additive_op() {
            let right = self.parse_multiplicative_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = ExprFactory::binary(left, op, right, span);
        }
        
        Ok(left)
    }
    
    /// 解析乘法表达式
    fn parse_multiplicative_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expression()?;
        
        while let Some(op) = self.parse_multiplicative_op() {
            let right = self.parse_unary_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = ExprFactory::binary(left, op, right, span);
        }
        
        Ok(left)
    }
    
    /// 解析一元表达式
    fn parse_unary_expression(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(LexerToken::Minus) {
            let op = UnaryOp::Minus;
            let operand = self.parse_unary_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(ExprFactory::unary(op, operand, span))
        } else if self.match_token(LexerToken::Plus) {
            let op = UnaryOp::Plus;
            let operand = self.parse_unary_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(ExprFactory::unary(op, operand, span))
        } else {
            self.parse_postfix_expression()
        }
    }
    
    /// 解析后缀表达式
    fn parse_postfix_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary_expression()?;
        
        loop {
            if self.match_token(LexerToken::LeftBracket) {
                // 下标访问
                let index = self.parse_expression()?;
                self.expect_token(LexerToken::RightBracket)?;
                let span = Span::new(expr.span().start, self.current_span().end);
                expr = ExprFactory::subscript(expr, index, span);
            } else if self.match_token(LexerToken::Dot) {
                // 属性访问
                let property = self.expect_identifier()?;
                let span = Span::new(expr.span().start, self.current_span().end);
                expr = ExprFactory::property_access(expr, property, span);
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
            LexerToken::Integer => {
                let value = self.parse_integer()?;
                let span = self.current_span();
                Ok(ExprFactory::constant(Value::Int(value), span))
            }
            LexerToken::Float => {
                let value = self.parse_float()?;
                let span = self.current_span();
                Ok(ExprFactory::constant(Value::Float(value), span))
            }
            LexerToken::String => {
                let value = self.parse_string()?;
                let span = self.current_span();
                Ok(ExprFactory::constant(Value::String(value), span))
            }
            LexerToken::Boolean => {
                let value = self.parse_boolean()?;
                let span = self.current_span();
                Ok(ExprFactory::constant(Value::Bool(value), span))
            }
            LexerToken::Identifier => {
                let name = self.expect_identifier()?;
                let span = self.current_span();
                
                // 检查是否是函数调用
                if self.match_token(LexerToken::LeftParen) {
                    self.parse_function_call(name, span)
                } else {
                    Ok(ExprFactory::variable(name, span))
                }
            }
            LexerToken::LeftParen => {
                // 括号表达式
                self.lexer.advance()?;
                let expr = self.parse_expression()?;
                self.expect_token(LexerToken::RightParen)?;
                Ok(expr)
            }
            LexerToken::LeftBracket => {
                // 列表表达式
                self.parse_list_expression()
            }
            LexerToken::LeftBrace => {
                // 映射表达式
                self.parse_map_expression()
            }
            _ => {
                Err(ParseError::new(
                    format!("Unexpected token: {:?}", token.kind),
                    self.current_span(),
                ))
            }
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
        if !self.check_token(LexerToken::RightParen) {
            loop {
                let arg = self.parse_expression()?;
                args.push(arg);
                
                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }
        
        self.expect_token(LexerToken::RightParen)?;
        let end_span = self.current_span();
        let full_span = Span::new(span.start, end_span.end);
        
        Ok(ExprFactory::function_call(name, args, distinct, full_span))
    }
    
    /// 解析列表表达式
    fn parse_list_expression(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::LeftBracket)?;
        
        let mut elements = Vec::new();
        
        if !self.check_token(LexerToken::RightBracket) {
            loop {
                let elem = self.parse_expression()?;
                elements.push(elem);
                
                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }
        
        self.expect_token(LexerToken::RightBracket)?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);
        
        Ok(ExprFactory::list(elements, span))
    }
    
    /// 解析映射表达式
    fn parse_map_expression(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::LeftBrace)?;
        
        let mut pairs = Vec::new();
        
        if !self.check_token(LexerToken::RightBrace) {
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
        
        self.expect_token(LexerToken::RightBrace)?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);
        
        Ok(ExprFactory::map(pairs, span))
    }
    
    /// 辅助方法
    
    fn match_token(&mut self, expected: LexerToken) -> bool {
        if self.lexer.check(&expected) {
            self.lexer.advance().ok();
            true
        } else {
            false
        }
    }
    
    fn check_token(&mut self, expected: LexerToken) -> bool {
        self.lexer.check(&expected)
    }
    
    fn expect_token(&mut self, expected: LexerToken) -> Result<(), ParseError> {
        let token = self.lexer.peek()?;
        if token.kind == expected {
            self.lexer.advance()?;
            Ok(())
        } else {
            Err(ParseError::new(
                format!("Expected {:?}, found {:?}", expected, token.kind),
                self.current_span(),
            ))
        }
    }
    
    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::Identifier = token.kind {
            let text = token.text.clone();
            self.lexer.advance()?;
            Ok(text)
        } else {
            Err(ParseError::new(
                format!("Expected identifier, found {:?}", token.kind),
                self.current_span(),
            ))
        }
    }
    
    fn parse_integer(&mut self) -> Result<i64, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::Integer = token.kind {
            let text = token.text.clone();
            self.lexer.advance()?;
            text.parse().map_err(|_| {
                ParseError::new(
                    format!("Invalid integer: {}", text),
                    self.current_span(),
                )
            })
        } else {
            Err(ParseError::new(
                format!("Expected integer, found {:?}", token.kind),
                self.current_span(),
            ))
        }
    }
    
    fn parse_float(&mut self) -> Result<f64, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::Float = token.kind {
            let text = token.text.clone();
            self.lexer.advance()?;
            text.parse().map_err(|_| {
                ParseError::new(
                    format!("Invalid float: {}", text),
                    self.current_span(),
                )
            })
        } else {
            Err(ParseError::new(
                format!("Expected float, found {:?}", token.kind),
                self.current_span(),
            ))
        }
    }
    
    fn parse_string(&mut self) -> Result<String, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::String = token.kind {
            let text = token.text.clone();
            self.lexer.advance()?;
            // 移除引号
            Ok(text.trim_matches('"').to_string())
        } else {
            Err(ParseError::new(
                format!("Expected string, found {:?}", token.kind),
                self.current_span(),
            ))
        }
    }
    
    fn parse_boolean(&mut self) -> Result<bool, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::Boolean = token.kind {
            let text = token.text.clone();
            self.lexer.advance()?;
            text.parse().map_err(|_| {
                ParseError::new(
                    format!("Invalid boolean: {}", text),
                    self.current_span(),
                )
            })
        } else {
            Err(ParseError::new(
                format!("Expected boolean, found {:?}", token.kind),
                self.current_span(),
            ))
        }
    }
    
    fn parse_comparison_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Equal) {
            Some(BinaryOp::Eq)
        } else if self.match_token(LexerToken::NotEqual) {
            Some(BinaryOp::Ne)
        } else if self.match_token(LexerToken::Less) {
            Some(BinaryOp::Lt)
        } else if self.match_token(LexerToken::LessEqual) {
            Some(BinaryOp::Le)
        } else if self.match_token(LexerToken::Greater) {
            Some(BinaryOp::Gt)
        } else if self.match_token(LexerToken::GreaterEqual) {
            Some(BinaryOp::Ge)
        } else {
            None
        }
    }
    
    fn parse_additive_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Plus) {
            Some(BinaryOp::Add)
        } else if self.match_token(LexerToken::Minus) {
            Some(BinaryOp::Sub)
        } else {
            None
        }
    }
    
    fn parse_multiplicative_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Star) {
            Some(BinaryOp::Mul)
        } else if self.match_token(LexerToken::Slash) {
            Some(BinaryOp::Div)
        } else if self.match_token(LexerToken::Percent) {
            Some(BinaryOp::Mod)
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
}