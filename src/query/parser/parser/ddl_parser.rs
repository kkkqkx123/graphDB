//! DDL 语句解析模块
//!
//! 负责解析数据定义语言语句，包括 CREATE、DROP、ALTER、DESC 等。

use crate::core::types::PropertyDef;
use crate::query::parser::ast::stmt::*;
use crate::query::parser::ast::types::DataType;
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::query::parser::parser::parse_context::ParseContext;
use crate::query::parser::TokenKind;

/// DDL 解析器
pub struct DdlParser;

impl DdlParser {
    pub fn new() -> Self {
        Self
    }

    /// 解析 CREATE 语句
    pub fn parse_create_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Create)?;

        if ctx.match_token(TokenKind::Tag) {
            // 解析 IF NOT EXISTS (在 TAG 之后)
            let mut if_not_exists = false;
            if ctx.match_token(TokenKind::If) {
                ctx.expect_token(TokenKind::Not)?;
                ctx.expect_token(TokenKind::Exists)?;
                if_not_exists = true;
            }
            let name = ctx.expect_identifier()?;
            let properties = self.parse_property_defs(ctx)?;
            Ok(Stmt::Create(CreateStmt {
                span: start_span,
                target: CreateTarget::Tag { name, properties },
                if_not_exists,
            }))
        } else if ctx.match_token(TokenKind::Edge) {
            // 解析 IF NOT EXISTS (在 EDGE 之后)
            let mut if_not_exists = false;
            if ctx.match_token(TokenKind::If) {
                ctx.expect_token(TokenKind::Not)?;
                ctx.expect_token(TokenKind::Exists)?;
                if_not_exists = true;
            }
            let name = ctx.expect_identifier()?;
            let properties = self.parse_property_defs(ctx)?;
            Ok(Stmt::Create(CreateStmt {
                span: start_span,
                target: CreateTarget::EdgeType { name, properties },
                if_not_exists,
            }))
        } else if ctx.match_token(TokenKind::Space) {
            // 解析 CREATE SPACE
            let mut if_not_exists = false;
            if ctx.match_token(TokenKind::If) {
                ctx.expect_token(TokenKind::Not)?;
                ctx.expect_token(TokenKind::Exists)?;
                if_not_exists = true;
            }
            let name = ctx.expect_identifier()?;
            
            // 解析可选参数 (vid_type, partition_num, replica_factor, comment)
            let mut vid_type = "INT64".to_string();
            let mut partition_num = 1i64;
            let mut replica_factor = 1i64;
            let mut comment = None;
            
            // 解析 (vid_type=INT64, partition_num=1, replica_factor=1, comment="xxx") 格式
            if ctx.match_token(TokenKind::LParen) {
                loop {
                    if ctx.check_token(TokenKind::RParen) {
                        ctx.expect_token(TokenKind::RParen)?;
                        break;
                    }
                    
                    if ctx.match_token(TokenKind::VIdType) {
                        ctx.expect_token(TokenKind::Assign)?;
                        vid_type = ctx.expect_identifier()?;
                    } else if ctx.match_token(TokenKind::PartitionNum) {
                        ctx.expect_token(TokenKind::Assign)?;
                        partition_num = ctx.expect_integer_literal()?;
                    } else if ctx.match_token(TokenKind::ReplicaFactor) {
                        ctx.expect_token(TokenKind::Assign)?;
                        replica_factor = ctx.expect_integer_literal()?;
                    } else if ctx.match_token(TokenKind::Comment) {
                        ctx.expect_token(TokenKind::Assign)?;
                        comment = Some(ctx.expect_string_literal()?);
                    }
                    
                    // 检查是否还有更多参数
                    if !ctx.match_token(TokenKind::Comma) {
                        ctx.expect_token(TokenKind::RParen)?;
                        break;
                    }
                }
            }
            
            Ok(Stmt::Create(CreateStmt {
                span: start_span,
                target: CreateTarget::Space { 
                    name, 
                    vid_type, 
                    partition_num, 
                    replica_factor, 
                    comment 
                },
                if_not_exists,
            }))
        } else if ctx.match_token(TokenKind::User) {
            // 解析 CREATE USER
            let mut if_not_exists = false;
            if ctx.match_token(TokenKind::If) {
                ctx.expect_token(TokenKind::Not)?;
                ctx.expect_token(TokenKind::Exists)?;
                if_not_exists = true;
            }
            let username = ctx.expect_identifier()?;
            ctx.expect_token(TokenKind::With)?;
            ctx.expect_token(TokenKind::Password)?;
            let password = ctx.expect_string_literal()?;

            let mut role = None;
            if ctx.match_token(TokenKind::With) {
                ctx.expect_token(TokenKind::Role)?;
                role = Some(ctx.expect_identifier()?);
            }

            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);

            Ok(Stmt::CreateUser(CreateUserStmt {
                span,
                username,
                password,
                role,
                if_not_exists,
            }))
        } else {
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "Expected TAG, EDGE, SPACE, or USER after CREATE".to_string(),
                ctx.current_position(),
            ))
        }
    }

    /// 解析 DROP 语句
    pub fn parse_drop_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Drop)?;

        let target = if ctx.match_token(TokenKind::Space) {
            DropTarget::Space(ctx.expect_identifier()?)
        } else if ctx.match_token(TokenKind::Tag) {
            // 解析 IF EXISTS (在 TAG 之后)
            let mut if_exists = false;
            if ctx.match_token(TokenKind::If) {
                ctx.expect_token(TokenKind::Exists)?;
                if_exists = true;
            }
            let mut tag_names = vec![ctx.expect_identifier()?];
            while ctx.match_token(TokenKind::Comma) {
                tag_names.push(ctx.expect_identifier()?);
            }
            return Ok(Stmt::Drop(DropStmt {
                span: start_span,
                target: DropTarget::Tags(tag_names),
                if_exists,
            }));
        } else if ctx.check_token(TokenKind::Edge) {
            ctx.next_token(); // 消费 EDGE
            if ctx.check_token(TokenKind::Index) {
                ctx.next_token(); // 消费 INDEX
                let index_name = ctx.expect_identifier()?;
                let space_name = if ctx.match_token(TokenKind::On) {
                    Some(ctx.expect_identifier()?)
                } else {
                    None
                };
                DropTarget::EdgeIndex {
                    space_name: space_name.unwrap_or_default(),
                    index_name,
                }
            } else {
                // 解析 IF EXISTS (在 EDGE 之后)
                let mut if_exists = false;
                if ctx.match_token(TokenKind::If) {
                    ctx.expect_token(TokenKind::Exists)?;
                    if_exists = true;
                }
                let mut edge_names = vec![ctx.expect_identifier()?];
                while ctx.match_token(TokenKind::Comma) {
                    edge_names.push(ctx.expect_identifier()?);
                }
                return Ok(Stmt::Drop(DropStmt {
                    span: start_span,
                    target: DropTarget::Edges(edge_names),
                    if_exists,
                }));
            }
        } else if ctx.match_token(TokenKind::Index) {
            let index_name = ctx.expect_identifier()?;
            let space_name = if ctx.match_token(TokenKind::On) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            DropTarget::TagIndex {
                space_name: space_name.unwrap_or_default(),
                index_name,
            }
        } else if ctx.match_token(TokenKind::User) {
            // 解析 DROP USER
            let mut if_exists = false;
            if ctx.match_token(TokenKind::If) {
                ctx.expect_token(TokenKind::Exists)?;
                if_exists = true;
            }
            let username = ctx.expect_identifier()?;

            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);

            return Ok(Stmt::DropUser(DropUserStmt {
                span,
                username,
                if_exists,
            }));
        } else {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "Expected SPACE, TAG, EDGE, INDEX, or USER".to_string(),
                ctx.current_position(),
            ));
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Drop(DropStmt { span, target, if_exists: false }))
    }

    /// 解析 DESC 语句
    pub fn parse_desc_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Desc)?;

        // 检查是否是 DESCRIBE USER
        if ctx.check_token(TokenKind::User) {
            return self.parse_describe_user_internal(ctx, start_span);
        }

        let target = if ctx.match_token(TokenKind::Space) {
            DescTarget::Space(ctx.expect_identifier()?)
        } else if ctx.match_token(TokenKind::Tag) {
            let tag_name = ctx.expect_identifier()?;
            let space_name = if ctx.match_token(TokenKind::In) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            DescTarget::Tag {
                space_name: space_name.unwrap_or_default(),
                tag_name,
            }
        } else if ctx.match_token(TokenKind::Edge) {
            let edge_name = ctx.expect_identifier()?;
            let space_name = if ctx.match_token(TokenKind::In) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            DescTarget::Edge {
                space_name: space_name.unwrap_or_default(),
                edge_name,
            }
        } else {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "Expected SPACE, TAG, EDGE, or USER".to_string(),
                ctx.current_position(),
            ));
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Desc(DescStmt { span, target }))
    }

    /// 解析 DESCRIBE USER 内部方法
    fn parse_describe_user_internal(&mut self, ctx: &mut ParseContext, start_span: crate::query::parser::ast::types::Span) -> Result<Stmt, ParseError> {
        ctx.expect_token(TokenKind::User)?;

        let username = ctx.expect_identifier()?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::DescribeUser(DescribeUserStmt {
            span,
            username,
        }))
    }

    /// 解析 ALTER 语句
    pub fn parse_alter_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Alter)?;

        // 检查是否是 ALTER USER
        if ctx.check_token(TokenKind::User) {
            return self.parse_alter_user_internal(ctx, start_span);
        }

        let (is_tag, name, additions, deletions, changes) = if ctx.match_token(TokenKind::Tag) {
            let tag_name = ctx.expect_identifier()?;
            let (additions, deletions, changes) = self.parse_alter_operations(ctx)?;
            (true, tag_name, additions, deletions, changes)
        } else if ctx.match_token(TokenKind::Edge) {
            let edge_name = ctx.expect_identifier()?;
            let (additions, deletions, changes) = self.parse_alter_operations(ctx)?;
            (false, edge_name, additions, deletions, changes)
        } else {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "Expected TAG, EDGE, or USER".to_string(),
                ctx.current_position(),
            ));
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        if is_tag {
            Ok(Stmt::Alter(AlterStmt {
                span,
                target: AlterTarget::Tag {
                    tag_name: name,
                    additions,
                    deletions,
                    changes,
                },
            }))
        } else {
            Ok(Stmt::Alter(AlterStmt {
                span,
                target: AlterTarget::Edge {
                    edge_name: name,
                    additions,
                    deletions,
                    changes,
                },
            }))
        }
    }

    /// 解析 ALTER 操作（ADD/DROP/CHANGE）
    fn parse_alter_operations(&mut self, ctx: &mut ParseContext) -> Result<(Vec<PropertyDef>, Vec<String>, Vec<PropertyChange>), ParseError> {
        let mut additions = Vec::new();
        let mut deletions = Vec::new();
        let mut changes = Vec::new();

        loop {
            if ctx.match_token(TokenKind::Add) {
                additions.extend(self.parse_property_defs(ctx)?);
            } else if ctx.match_token(TokenKind::Drop) {
                ctx.expect_token(TokenKind::LParen)?;
                loop {
                    deletions.push(ctx.expect_identifier()?);
                    if !ctx.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                ctx.expect_token(TokenKind::RParen)?;
            } else if ctx.match_token(TokenKind::Change) {
                ctx.expect_token(TokenKind::LParen)?;
                loop {
                    let old_name = ctx.expect_identifier()?;
                    let new_name = ctx.expect_identifier()?;
                    ctx.expect_token(TokenKind::Colon)?;
                    let data_type = self.parse_data_type(ctx)?;
                    changes.push(PropertyChange {
                        old_name,
                        new_name,
                        data_type,
                    });
                    if !ctx.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                ctx.expect_token(TokenKind::RParen)?;
            } else {
                break;
            }
        }

        Ok((additions, deletions, changes))
    }

    /// 解析 ALTER USER 内部方法
    fn parse_alter_user_internal(&mut self, ctx: &mut ParseContext, start_span: crate::query::parser::ast::types::Span) -> Result<Stmt, ParseError> {
        ctx.expect_token(TokenKind::User)?;

        let username = ctx.expect_identifier()?;

        let mut password = None;
        let mut new_role = None;
        let mut is_locked = None;

        // 解析 WITH PASSWORD 或 SET 子句
        if ctx.match_token(TokenKind::With) {
            if ctx.match_token(TokenKind::Password) {
                password = Some(ctx.expect_string_literal()?);
            } else if ctx.match_token(TokenKind::Role) {
                new_role = Some(ctx.expect_identifier()?);
            }
        }

        // 也支持 SET ROLE = ... 和 SET LOCKED = ... 语法
        while ctx.match_token(TokenKind::Set) {
            if ctx.match_token(TokenKind::Role) {
                ctx.expect_token(TokenKind::Eq)?;
                new_role = Some(ctx.expect_identifier()?);
            } else if ctx.match_token(TokenKind::Locked) {
                ctx.expect_token(TokenKind::Eq)?;
                let value = ctx.expect_identifier()?;
                is_locked = Some(value.to_lowercase() == "true");
            }
        }

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::AlterUser(AlterUserStmt {
            span,
            username,
            password,
            new_role,
            is_locked,
        }))
    }

    /// 解析属性定义列表
    pub fn parse_property_defs(&mut self, ctx: &mut ParseContext) -> Result<Vec<PropertyDef>, ParseError> {
        let mut defs = Vec::new();
        if ctx.match_token(TokenKind::LParen) {
            while !ctx.match_token(TokenKind::RParen) {
                let name = ctx.expect_identifier()?;
                ctx.expect_token(TokenKind::Colon)?;
                
                // 解析数据类型，支持关键字或标识符
                let dtype = self.parse_data_type(ctx)?;
                
                defs.push(PropertyDef {
                    name,
                    data_type: dtype,
                    nullable: true,
                    default: None,
                    comment: None,
                });
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        Ok(defs)
    }
    
    /// 解析数据类型，支持关键字（如 STRING, INT）或标识符
    pub fn parse_data_type(&mut self, ctx: &mut ParseContext) -> Result<DataType, ParseError> {
        let token = ctx.current_token();
        match token.kind {
            // 支持数据类型关键字
            TokenKind::Int | TokenKind::Int8 | TokenKind::Int16 | TokenKind::Int32 | TokenKind::Int64 => {
                ctx.next_token();
                Ok(DataType::Int)
            }
            TokenKind::Float | TokenKind::Double => {
                ctx.next_token();
                Ok(DataType::Float)
            }
            TokenKind::String | TokenKind::FixedString => {
                ctx.next_token();
                Ok(DataType::String)
            }
            TokenKind::Bool => {
                ctx.next_token();
                Ok(DataType::Bool)
            }
            TokenKind::Date => {
                ctx.next_token();
                Ok(DataType::Date)
            }
            TokenKind::Timestamp => {
                ctx.next_token();
                Ok(DataType::Timestamp)
            }
            TokenKind::Datetime => {
                ctx.next_token();
                Ok(DataType::DateTime)
            }
            // 支持标识符形式的数据类型（如 "INT", "string" 等）
            TokenKind::Identifier(ref s) => {
                let type_name = s.clone();
                ctx.next_token();
                match type_name.to_uppercase().as_str() {
                    "INT" | "INTEGER" | "INT8" | "INT16" | "INT32" | "INT64" => Ok(DataType::Int),
                    "FLOAT" | "DOUBLE" => Ok(DataType::Float),
                    "STRING" | "VARCHAR" | "TEXT" => Ok(DataType::String),
                    "BOOL" | "BOOLEAN" => Ok(DataType::Bool),
                    "DATE" => Ok(DataType::Date),
                    "TIMESTAMP" => Ok(DataType::Timestamp),
                    "DATETIME" => Ok(DataType::DateTime),
                    _ => Err(ParseError::new(
                        ParseErrorKind::SyntaxError,
                        format!("未知数据类型: {}", type_name),
                        ctx.current_position(),
                    )),
                }
            }
            _ => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!("期望数据类型，发现 {:?}", token.kind),
                ctx.current_position(),
            )),
        }
    }
}

impl Default for DdlParser {
    fn default() -> Self {
        Self::new()
    }
}
