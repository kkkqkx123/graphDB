//! 语句解析模块
//!
//! 负责解析各种语句，包括 MATCH、GO、CREATE、DELETE、UPDATE 等。
//! 本模块作为入口，将具体解析逻辑委托给各个子模块。

use crate::query::parser::ast::stmt::*;
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::query::parser::parser::parse_context::ParseContext;
use crate::query::parser::parser::{
    ddl_parser::DdlParser,
    dml_parser::DmlParser,
    traversal_parser::TraversalParser,
    user_parser::UserParser,
    util_stmt_parser::UtilStmtParser,
};
use crate::query::parser::TokenKind;

/// 语句解析器
pub struct StmtParser;

impl StmtParser {
    pub fn new() -> Self {
        Self
    }

    /// 解析语句（支持管道操作符）
    pub fn parse_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let stmt = self.parse_single_statement(ctx)?;
        self.parse_pipe_suffix(ctx, stmt)
    }

    /// 解析单个语句（不分发管道）
    fn parse_single_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let token = ctx.current_token().clone();
        match token.kind {
            // 图遍历语句
            TokenKind::Match | TokenKind::Optional => TraversalParser::new().parse_match_statement(ctx),
            TokenKind::Go => TraversalParser::new().parse_go_statement(ctx),
            TokenKind::Find => TraversalParser::new().parse_find_path_statement(ctx),
            TokenKind::Get => TraversalParser::new().parse_subgraph_statement(ctx),

            // 数据修改语句
            TokenKind::Insert => DmlParser::new().parse_insert_statement(ctx),
            TokenKind::Delete => DmlParser::new().parse_delete_statement(ctx),
            TokenKind::Update => DmlParser::new().parse_update_statement(ctx),
            TokenKind::Merge => DmlParser::new().parse_merge_statement(ctx),

            // DDL 语句
            TokenKind::Create => DdlParser::new().parse_create_statement(ctx),
            TokenKind::Drop => DdlParser::new().parse_drop_statement(ctx),
            TokenKind::Desc => DdlParser::new().parse_desc_statement(ctx),
            TokenKind::Alter => DdlParser::new().parse_alter_statement(ctx),

            // 用户管理语句
            TokenKind::CreateUser => UserParser::new().parse_create_user_statement(ctx),
            TokenKind::AlterUser => UserParser::new().parse_alter_user_statement(ctx),
            TokenKind::DropUser => UserParser::new().parse_drop_user_statement(ctx),
            TokenKind::ChangePassword => UserParser::new().parse_change_password_statement(ctx),
            TokenKind::Change => UserParser::new().parse_change_statement(ctx),

            // 工具语句
            TokenKind::Use => UtilStmtParser::new().parse_use_statement(ctx),
            TokenKind::Show => UtilStmtParser::new().parse_show_statement(ctx),
            TokenKind::Explain => self.parse_explain_statement(ctx),
            TokenKind::Fetch => UtilStmtParser::new().parse_fetch_statement(ctx),
            TokenKind::Lookup => UtilStmtParser::new().parse_lookup_statement(ctx),
            TokenKind::Unwind => UtilStmtParser::new().parse_unwind_statement(ctx),
            TokenKind::Return => UtilStmtParser::new().parse_return_statement(ctx),
            TokenKind::With => UtilStmtParser::new().parse_with_statement(ctx),
            TokenKind::Yield => UtilStmtParser::new().parse_yield_statement(ctx),
            TokenKind::Set => UtilStmtParser::new().parse_set_statement(ctx),
            TokenKind::Remove => UtilStmtParser::new().parse_remove_statement(ctx),

            _ => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!("Unexpected token: {:?}", token.kind),
                ctx.current_position(),
            )),
        }
    }

    /// 解析管道后缀（| 操作符）
    fn parse_pipe_suffix(&mut self, ctx: &mut ParseContext, left: Stmt) -> Result<Stmt, ParseError> {
        if ctx.match_token(TokenKind::Pipe) {
            let start_span = left.span();
            let right = self.parse_single_statement(ctx)?;
            let end_span = right.span();
            let span = ctx.merge_span(start_span.start, end_span.end);

            let pipe_stmt = Stmt::Pipe(PipeStmt {
                span,
                left: Box::new(left),
                right: Box::new(right),
            });

            self.parse_pipe_suffix(ctx, pipe_stmt)
        } else {
            Ok(left)
        }
    }

    /// 解析 EXPLAIN 语句（需要特殊处理，因为包含子语句）
    fn parse_explain_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Explain)?;

        let statement = Box::new(self.parse_statement(ctx)?);

        Ok(Stmt::Explain(ExplainStmt { span: start_span, statement }))
    }
}

impl Default for StmtParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::parser::parse_context::ParseContext;

    fn create_parser_context(input: &str) -> ParseContext {
        ParseContext::new(input)
    }

    #[test]
    fn test_parse_match_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("MATCH (n:Person) RETURN n");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "MATCH 解析失败: {:?}", result.err());
    }

    #[test]
    fn test_parse_go_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("GO 1 STEP FROM \"player100\" OVER follow");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "GO 解析失败: {:?}", result.err());
    }

    #[test]
    fn test_parse_create_tag_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("CREATE TAG IF NOT EXISTS Person(name: STRING, age: INT)");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "CREATE TAG 解析失败: {:?}", result.err());
    }

    #[test]
    fn test_parse_insert_vertex_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("INSERT VERTEX Person(name, age) VALUES \"player100\":(\"Tom\", 18)");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "INSERT VERTEX 解析失败: {:?}", result.err());
    }

    #[test]
    fn test_parse_delete_vertex_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("DELETE VERTEX \"player100\"");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "DELETE VERTEX 解析失败: {:?}", result.err());
    }

    #[test]
    fn test_parse_use_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("USE test_space");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "USE 解析失败: {:?}", result.err());
        
        if let Ok(Stmt::Use(stmt)) = result {
            assert_eq!(stmt.space, "test_space");
        } else {
            panic!("期望 Use 语句");
        }
    }

    #[test]
    fn test_parse_show_spaces_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("SHOW SPACES");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "SHOW SPACES 解析失败: {:?}", result.err());
    }

    #[test]
    fn test_create_space_statement_parses() {
        let mut parser = StmtParser::new();
        
        // 测试 CREATE SPACE 语句能够解析成功
        let mut ctx = create_parser_context("CREATE SPACE IF NOT EXISTS test_space");
        let result = parser.parse_statement(&mut ctx);
        
        // 验证解析成功
        assert!(result.is_ok(), "CREATE SPACE 解析失败: {:?}", result.err());
        
        // 验证是 Create 语句
        if let Ok(Stmt::Create(stmt)) = result {
            // 验证是 Space 创建目标
            match &stmt.target {
                CreateTarget::Space { name, vid_type, partition_num, replica_factor, .. } => {
                    assert_eq!(name, "test_space");
                    assert_eq!(vid_type, "INT64");
                    assert_eq!(*partition_num, 1);
                    assert_eq!(*replica_factor, 1);
                }
                _ => panic!("期望 Space 创建目标，实际得到 {:?}", stmt.target),
            }
            assert!(stmt.if_not_exists);
        } else {
            panic!("期望 Create 语句");
        }
    }

    #[test]
    fn test_create_space_with_params_parses() {
        let mut parser = StmtParser::new();
        
        // 测试 CREATE SPACE 带参数语句能够解析成功
        let mut ctx = create_parser_context(
            "CREATE SPACE test_space(vid_type=FIXEDSTRING32, partition_num=10, replica_factor=3)"
        );
        let result = parser.parse_statement(&mut ctx);
        
        // 验证解析成功
        assert!(result.is_ok(), "CREATE SPACE with params 解析失败: {:?}", result.err());
        
        // 验证是 Create 语句
        if let Ok(Stmt::Create(stmt)) = result {
            // 验证是 Space 创建目标
            match &stmt.target {
                CreateTarget::Space { name, vid_type, partition_num, replica_factor, .. } => {
                    assert_eq!(name, "test_space");
                    assert_eq!(vid_type, "FIXEDSTRING32");
                    assert_eq!(*partition_num, 10);
                    assert_eq!(*replica_factor, 3);
                }
                _ => panic!("期望 Space 创建目标，实际得到 {:?}", stmt.target),
            }
        } else {
            panic!("期望 Create 语句");
        }
    }
}
