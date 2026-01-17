//! 模式解析器 (v2)

use super::*;
use crate::query::parser::core::error::ParseErrorKind;
use crate::query::parser::lexer::{Lexer, TokenKind as LexerToken};

/// 模式解析器
pub struct PatternParser {
    lexer: Lexer,
}

impl PatternParser {
    /// 创建模式解析器
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }
    /// 解析模式
    pub fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let token = self.lexer.peek()?;

        match token.kind {
            LexerToken::LParen => {
                // 节点模式: (variable:Label {properties})
                self.parse_node_pattern()
            }
            LexerToken::LBracket => {
                // 边模式: [variable:Type {properties}]-> or <-[variable:Type {properties}]
                self.parse_edge_pattern()
            }
            LexerToken::Identifier(_) => {
                // 变量模式或路径模式
                self.parse_variable_or_path_pattern()
            }
            _ => Err(self.parse_error(format!("Expected pattern, found {:?}", token.kind))),
        }
    }

    /// 解析节点模式
    fn parse_node_pattern(&mut self) -> Result<Pattern, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::LParen)?;

        // 解析变量名（可选）
        let variable = if let LexerToken::Identifier(_) = self.lexer.peek()?.kind {
            Some(self.expect_identifier()?)
        } else {
            None
        };

        // 解析标签（可选）
        let mut labels = Vec::new();
        if self.match_token(LexerToken::Colon) {
            // 解析标签列表
            loop {
                let label = self.expect_identifier()?;
                labels.push(label);

                if !self.match_token(LexerToken::Pipe) {
                    // | 用于多个标签
                    break;
                }
            }
        }

        // 解析属性（可选）
        let properties = if self.match_token(LexerToken::LBrace) {
            Some(self.parse_map_expression()?)
        } else {
            None
        };

        self.expect_token(LexerToken::RParen)?;

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(PatternFactory::node(
            variable,
            labels,
            properties,
            vec![],
            span,
        ))
    }

    /// 解析边模式
    fn parse_edge_pattern(&mut self) -> Result<Pattern, ParseError> {
        let start_span = self.current_span();

        // 检查方向
        let direction = if self.match_token(LexerToken::BackArrow) {
            EdgeDirection::Incoming
        } else {
            EdgeDirection::Outgoing
        };

        self.expect_token(LexerToken::LBracket)?;

        // 解析变量名（可选）
        let variable = if let LexerToken::Identifier(_) = self.lexer.peek()?.kind {
            Some(self.expect_identifier()?)
        } else {
            None
        };

        // 解析边类型（可选）
        let mut edge_types = Vec::new();
        if self.match_token(LexerToken::Colon) {
            // 解析边类型列表
            loop {
                let edge_type = self.expect_identifier()?;
                edge_types.push(edge_type);

                if !self.match_token(LexerToken::Pipe) {
                    // | 用于多个类型
                    break;
                }
            }
        }

        // 解析属性（可选）
        let properties = if self.match_token(LexerToken::LBrace) {
            Some(self.parse_map_expression()?)
        } else {
            None
        };

        // 解析范围（可选）
        let range = if self.match_token(LexerToken::Star) {
            if self.match_token(LexerToken::LParen) {
                // 解析范围: *({min}, {max})
                let min = if self.match_token(LexerToken::IntegerLiteral(0)) {
                    Some(self.parse_integer()? as usize)
                } else {
                    None
                };

                self.expect_token(LexerToken::Comma)?;

                let max = if self.match_token(LexerToken::IntegerLiteral(0)) {
                    Some(self.parse_integer()? as usize)
                } else {
                    None
                };

                self.expect_token(LexerToken::RParen)?;
                Some(EdgeRange::new(min, max))
            } else {
                // 任意长度: *
                Some(EdgeRange::any())
            }
        } else {
            None
        };

        self.expect_token(LexerToken::RBracket)?;

        // 检查方向
        let final_direction = if direction == EdgeDirection::Incoming {
            if self.match_token(LexerToken::Arrow) {
                EdgeDirection::Both // <-[]-> 表示双向
            } else {
                EdgeDirection::Incoming
            }
        } else {
            if self.match_token(LexerToken::Arrow) {
                EdgeDirection::Outgoing
            } else {
                EdgeDirection::Both // -[]- 表示双向
            }
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(PatternFactory::edge(
            variable,
            edge_types,
            properties,
            vec![],
            final_direction,
            range,
            span,
        ))
    }

    /// 解析变量或路径模式
    fn parse_variable_or_path_pattern(&mut self) -> Result<Pattern, ParseError> {
        let name = self.expect_identifier()?;
        let span = self.current_span();

        // 检查是否是路径模式（包含箭头）
        if self.match_token(LexerToken::Arrow) || self.match_token(LexerToken::BackArrow) {
            // 这是一个路径模式，需要重新解析
            // 这里简化处理，返回变量模式
            Ok(PatternFactory::variable(name, span))
        } else {
            // 变量模式
            Ok(PatternFactory::variable(name, span))
        }
    }

    /// 解析路径模式
    pub fn parse_path_pattern(&mut self) -> Result<Pattern, ParseError> {
        let start_span = self.current_span();
        let mut elements = Vec::new();

        // 解析路径元素
        loop {
            let element = self.parse_path_element()?;
            elements.push(element);

            // 检查是否还有更多的路径元素
            if !self.is_path_continuation() {
                break;
            }
        }

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Pattern::Path(PathPattern::new(elements, span)))
    }

    /// 解析路径元素
    fn parse_path_element(&mut self) -> Result<PathElement, ParseError> {
        let token = self.lexer.peek()?;

        match token.kind {
            LexerToken::LParen => {
                // 节点元素
                let node_pattern = self.parse_node_pattern()?;
                if let Pattern::Node(node) = node_pattern {
                    Ok(PathElement::Node(node))
                } else {
                    unreachable!()
                }
            }
            LexerToken::LBracket => {
                // 边元素
                let edge_pattern = self.parse_edge_pattern()?;
                if let Pattern::Edge(edge) = edge_pattern {
                    Ok(PathElement::Edge(edge))
                } else {
                    unreachable!()
                }
            }
            LexerToken::Pipe => {
                // 替代模式: (a|b|c)
                self.parse_alternative_pattern()
            }
            LexerToken::QMark | LexerToken::Question => {
                // 可选模式: ?
                self.lexer.advance();
                let inner = self.parse_path_element()?;
                Ok(PathElement::Optional(Box::new(inner)))
            }
            LexerToken::Star => {
                // 重复模式: *
                self.lexer.advance();
                let inner = self.parse_path_element()?;
                Ok(PathElement::Repeated(
                    Box::new(inner),
                    RepetitionType::ZeroOrMore,
                ))
            }
            LexerToken::Plus => {
                // 重复模式: +
                self.lexer.advance();
                let inner = self.parse_path_element()?;
                Ok(PathElement::Repeated(
                    Box::new(inner),
                    RepetitionType::OneOrMore,
                ))
            }
            _ => Err(self.parse_error(format!("Expected path element, found {:?}", token.kind))),
        }
    }

    /// 解析替代模式
    fn parse_alternative_pattern(&mut self) -> Result<PathElement, ParseError> {
        self.expect_token(LexerToken::Pipe)?;

        let mut alternatives = Vec::new();

        loop {
            let pattern = self.parse_pattern()?;
            alternatives.push(pattern);

            if !self.match_token(LexerToken::Pipe) {
                break;
            }
        }

        Ok(PathElement::Alternative(alternatives))
    }

    /// 检查是否是路径延续
    fn is_path_continuation(&mut self) -> bool {
        let token = self.lexer.peek().ok();

        match token.map(|t| t.kind) {
            Some(LexerToken::LParen) | Some(LexerToken::LBracket) => true,
            Some(LexerToken::Pipe)
            | Some(LexerToken::QMark)
            | Some(LexerToken::Question)
            | Some(LexerToken::Star)
            | Some(LexerToken::Plus) => true,
            _ => false,
        }
    }

    /// 辅助方法

    fn match_token(&mut self, expected: LexerToken) -> bool {
        if self.lexer.check(expected) {
            self.lexer.advance();
            true
        } else {
            false
        }
    }

    fn check_token(&mut self, expected: LexerToken) -> bool {
        self.lexer.check(expected)
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
            Err(self.parse_error(format!("Expected identifier, found {:?}", token.kind)))
        }
    }

    fn parse_integer(&mut self) -> Result<i64, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::IntegerLiteral(_) = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            text.parse()
                .map_err(|_| self.parse_error(format!("Invalid integer: {}", text)))
        } else {
            Err(self.parse_error(format!("Expected integer, found {:?}", token.kind)))
        }
    }

    fn parse_map_expression(&mut self) -> Result<Expr, ParseError> {
        // 由于lexer.input是私有的，这里需要重新设计
        // 暂时返回一个简单的映射表达式
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

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        // 由于lexer.input是私有的，这里需要重新设计
        // 暂时返回一个简单的变量表达式
        let name = self.expect_identifier()?;
        let span = self.current_span();
        Ok(Expr::Variable(VariableExpr::new(name, span)))
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
        ParseError::new(ParseErrorKind::SyntaxError, message, line, column)
    }
}
