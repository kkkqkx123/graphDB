//! CREATE 语句解析器实现

use crate::query::parser::ast::*;
use crate::query::parser::core::error::ParseError;
use crate::query::parser::TokenKind;

impl crate::query::parser::Parser {
    pub fn parse_create_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Create)?;
        
        match self.current_token().kind {
            TokenKind::Tag => {
                self.next_token();
                self.parse_create_tag_statement()
            }
            TokenKind::Edge => {
                self.next_token();
                self.parse_create_edge_statement()
            }
            TokenKind::Space => {
                self.next_token();
                self.parse_create_space_statement()
            }
            TokenKind::Index => {
                self.next_token();
                self.parse_create_index_statement()
            }
            _ => {
                let variable = if matches!(self.current_token().kind, TokenKind::Identifier(_)) {
                    Some(self.parse_identifier()?)
                } else {
                    None
                };
                
                let pattern = self.parse_pattern()?;
                
                Ok(Stmt::Create(CreateStmt {
                    span: self.current_span(),
                    target: CreateTarget::Node {
                        variable,
                        labels: vec![],
                        properties: None,
                    },
                }))
            }
        }
    }
    
    fn parse_create_tag_statement(&mut self) -> Result<Stmt, ParseError> {
        let name = self.parse_identifier()?;
        
        let mut properties = Vec::new();
        
        if self.current_token().kind == TokenKind::LParen {
            self.next_token();
            
            while self.current_token().kind != TokenKind::RParen {
                let prop_name = self.parse_identifier()?;
                self.expect_token(TokenKind::Colon)?;
                let data_type = self.parse_data_type()?;
                
                properties.push(PropertyDef {
                    name: prop_name,
                    data_type,
                    nullable: false,
                    default: None,
                });
                
                if self.current_token().kind == TokenKind::Comma {
                    self.next_token();
                } else {
                    break;
                }
            }
            
            self.expect_token(TokenKind::RParen)?;
        }
        
        Ok(Stmt::Create(CreateStmt {
            span: self.current_span(),
            target: CreateTarget::Tag {
                name,
                properties,
            },
        }))
    }
    
    fn parse_create_edge_statement(&mut self) -> Result<Stmt, ParseError> {
        let name = self.parse_identifier()?;
        
        let mut properties = Vec::new();
        
        if self.current_token().kind == TokenKind::LParen {
            self.next_token();
            
            while self.current_token().kind != TokenKind::RParen {
                let prop_name = self.parse_identifier()?;
                self.expect_token(TokenKind::Colon)?;
                let data_type = self.parse_data_type()?;
                
                properties.push(PropertyDef {
                    name: prop_name,
                    data_type,
                    nullable: false,
                    default: None,
                });
                
                if self.current_token().kind == TokenKind::Comma {
                    self.next_token();
                } else {
                    break;
                }
            }
            
            self.expect_token(TokenKind::RParen)?;
        }
        
        Ok(Stmt::Create(CreateStmt {
            span: self.current_span(),
            target: CreateTarget::Edge {
                edge_type: name,
                properties,
            },
        }))
    }
    
    fn parse_create_space_statement(&mut self) -> Result<Stmt, ParseError> {
        let name = self.parse_identifier()?;
        
        Ok(Stmt::Create(CreateStmt {
            span: self.current_span(),
            target: CreateTarget::Space { name },
        }))
    }
    
    fn parse_create_index_statement(&mut self) -> Result<Stmt, ParseError> {
        let name = self.parse_identifier()?;
        
        self.expect_token(TokenKind::On)?;
        
        let on = self.parse_identifier()?;
        
        let mut properties = Vec::new();
        
        if self.current_token().kind == TokenKind::LParen {
            self.next_token();
            
            while self.current_token().kind != TokenKind::RParen {
                properties.push(self.parse_identifier()?);
                
                if self.current_token().kind == TokenKind::Comma {
                    self.next_token();
                } else {
                    break;
                }
            }
            
            self.expect_token(TokenKind::RParen)?;
        }
        
        Ok(Stmt::Create(CreateStmt {
            span: self.current_span(),
            target: CreateTarget::Index {
                name,
                on,
                properties,
            },
        }))
    }
    
    fn parse_data_type(&mut self) -> Result<DataType, ParseError> {
        match &self.current_token().kind {
            TokenKind::Bool => {
                self.next_token();
                Ok(DataType::Bool)
            }
            TokenKind::Int | TokenKind::Int64 => {
                self.next_token();
                Ok(DataType::Int64)
            }
            TokenKind::Int32 => {
                self.next_token();
                Ok(DataType::Int32)
            }
            TokenKind::Int16 => {
                self.next_token();
                Ok(DataType::Int16)
            }
            TokenKind::Int8 => {
                self.next_token();
                Ok(DataType::Int8)
            }
            TokenKind::Float | TokenKind::Double => {
                self.next_token();
                Ok(DataType::Double)
            }
            TokenKind::String | TokenKind::FixedString => {
                self.next_token();
                Ok(DataType::String)
            }
            TokenKind::Timestamp => {
                self.next_token();
                Ok(DataType::Timestamp)
            }
            TokenKind::Date => {
                self.next_token();
                Ok(DataType::Date)
            }
            TokenKind::Time => {
                self.next_token();
                Ok(DataType::Time)
            }
            TokenKind::Datetime => {
                self.next_token();
                Ok(DataType::Datetime)
            }
            _ => Err(ParseError::syntax_error(
                format!("Expected data type, got {:?}", self.current_token().kind),
                self.current_token().line,
                self.current_token().column,
            )),
        }
    }
}
