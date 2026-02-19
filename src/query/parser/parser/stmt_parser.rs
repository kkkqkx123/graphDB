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
use crate::core::types::expression::Expression;

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
            TokenKind::Update => self.parse_update_statement_extended(ctx),
            TokenKind::Upsert => DmlParser::new().parse_update_statement(ctx),
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
            TokenKind::Grant => UserParser::new().parse_grant_statement(ctx),
            TokenKind::Revoke => UserParser::new().parse_revoke_statement(ctx),

            // 工具语句
            TokenKind::Use => UtilStmtParser::new().parse_use_statement(ctx),
            TokenKind::Show => self.parse_show_statement_extended(ctx),
            TokenKind::Explain => self.parse_explain_statement(ctx),
            TokenKind::Profile => self.parse_profile_statement(ctx),
            TokenKind::Group => self.parse_group_by_statement(ctx),
            TokenKind::Kill => self.parse_kill_statement(ctx),
            TokenKind::Fetch => UtilStmtParser::new().parse_fetch_statement(ctx),
            TokenKind::Lookup => UtilStmtParser::new().parse_lookup_statement(ctx),
            TokenKind::Unwind => UtilStmtParser::new().parse_unwind_statement(ctx),
            TokenKind::Return => UtilStmtParser::new().parse_return_statement(ctx),
            TokenKind::With => UtilStmtParser::new().parse_with_statement(ctx),
            TokenKind::Yield => UtilStmtParser::new().parse_yield_statement(ctx),
            TokenKind::Set => UtilStmtParser::new().parse_set_statement(ctx),
            TokenKind::Remove => UtilStmtParser::new().parse_remove_statement(ctx),

            // 变量赋值语句 ($var = statement)
            TokenKind::Dollar => self.parse_assignment_statement(ctx),

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
            // 检查是否是集合操作
            self.parse_set_operation_suffix(ctx, left)
        }
    }

    /// 解析 EXPLAIN 语句（需要特殊处理，因为包含子语句）
    fn parse_explain_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Explain)?;

        // 解析可选的 FORMAT 子句
        let format = if ctx.match_token(TokenKind::Format) {
            ctx.expect_token(TokenKind::Assign)?;
            let format_name = ctx.expect_identifier()?;
            match format_name.to_uppercase().as_str() {
                "DOT" => ExplainFormat::Dot,
                "TABLE" => ExplainFormat::Table,
                _ => {
                    return Err(ParseError::new(
                        ParseErrorKind::SyntaxError,
                        format!("未知的 EXPLAIN 格式: {}, 期望 DOT 或 TABLE", format_name),
                        ctx.current_position(),
                    ));
                }
            }
        } else {
            ExplainFormat::default()
        };

        let statement = Box::new(self.parse_statement(ctx)?);

        Ok(Stmt::Explain(ExplainStmt {
            span: start_span,
            statement,
            format,
        }))
    }

    /// 解析 PROFILE 语句
    fn parse_profile_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Profile)?;

        // 解析可选的 FORMAT 子句
        let format = if ctx.match_token(TokenKind::Format) {
            ctx.expect_token(TokenKind::Assign)?;
            let format_name = ctx.expect_identifier()?;
            match format_name.to_uppercase().as_str() {
                "DOT" => ExplainFormat::Dot,
                "TABLE" => ExplainFormat::Table,
                _ => {
                    return Err(ParseError::new(
                        ParseErrorKind::SyntaxError,
                        format!("未知的 PROFILE 格式: {}, 期望 DOT 或 TABLE", format_name),
                        ctx.current_position(),
                    ));
                }
            }
        } else {
            ExplainFormat::default()
        };

        let statement = Box::new(self.parse_statement(ctx)?);

        Ok(Stmt::Profile(ProfileStmt {
            span: start_span,
            statement,
            format,
        }))
    }

    /// 解析 GROUP BY 语句
    fn parse_group_by_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        use crate::query::parser::ast::stmt::{GroupByStmt, YieldItem};
        use crate::query::parser::parser::clause_parser::ClauseParser;
        use crate::core::types::expression::Expression;

        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Group)?;
        ctx.expect_token(TokenKind::By)?;

        // 解析分组项列表（只解析标识符）
        let mut group_items = Vec::new();
        loop {
            let ident = ctx.expect_identifier()?;
            group_items.push(Expression::Variable(ident));
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }

        // 解析 YIELD 子句
        let yield_clause = if ctx.match_token(TokenKind::Yield) {
            ClauseParser::new().parse_yield_clause(ctx)?
        } else {
            // 如果没有 YIELD，创建一个默认的返回所有分组项的 YIELD
            let items: Vec<YieldItem> = group_items.iter().enumerate().map(|(i, expr)| {
                YieldItem {
                    expression: expr.clone(),
                    alias: Some(format!("group_{}", i)),
                }
            }).collect();
            crate::query::parser::ast::stmt::YieldClause {
                span: start_span,
                items,
                where_clause: None,
                limit: None,
                skip: None,
                sample: None,
            }
        };

        // 解析可选的 HAVING 子句
        let having_clause = if ctx.match_token(TokenKind::Having) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::GroupBy(GroupByStmt {
            span,
            group_items,
            yield_clause,
            having_clause,
        }))
    }

    /// 解析表达式（辅助方法）
    fn parse_expression(&mut self, ctx: &mut ParseContext) -> Result<Expression, ParseError> {
        let mut expr_parser = crate::query::parser::parser::ExprParser::new(ctx);
        let result = expr_parser.parse_expression(ctx)?;
        Ok(result.expr)
    }

    /// 解析扩展的 SHOW 语句（包括 SESSIONS、QUERIES 和 CONFIGS）
    fn parse_show_statement_extended(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        use crate::query::parser::ast::stmt::{ShowSessionsStmt, ShowQueriesStmt, ShowConfigsStmt};

        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Show)?;

        // 检查下一个 token
        if ctx.check_token(TokenKind::Sessions) {
            ctx.expect_token(TokenKind::Sessions)?;
            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);
            Ok(Stmt::ShowSessions(ShowSessionsStmt { span }))
        } else if ctx.check_token(TokenKind::Queries) {
            ctx.expect_token(TokenKind::Queries)?;
            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);
            Ok(Stmt::ShowQueries(ShowQueriesStmt { span }))
        } else if ctx.check_token(TokenKind::Configs) {
            ctx.expect_token(TokenKind::Configs)?;
            // 解析可选的模块名
            let module = if ctx.is_identifier_or_in_token() {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);
            Ok(Stmt::ShowConfigs(ShowConfigsStmt { span, module }))
        } else if ctx.check_token(TokenKind::Spaces) {
            ctx.expect_token(TokenKind::Spaces)?;
            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);
            Ok(Stmt::Show(crate::query::parser::ast::stmt::ShowStmt {
                span,
                target: crate::query::parser::ast::stmt::ShowTarget::Spaces,
            }))
        } else if ctx.check_token(TokenKind::Tags) {
            ctx.expect_token(TokenKind::Tags)?;
            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);
            Ok(Stmt::Show(crate::query::parser::ast::stmt::ShowStmt {
                span,
                target: crate::query::parser::ast::stmt::ShowTarget::Tags,
            }))
        } else if ctx.check_token(TokenKind::Edges) {
            ctx.expect_token(TokenKind::Edges)?;
            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);
            Ok(Stmt::Show(crate::query::parser::ast::stmt::ShowStmt {
                span,
                target: crate::query::parser::ast::stmt::ShowTarget::Edges,
            }))
        } else if ctx.check_token(TokenKind::Hosts) {
            ctx.expect_token(TokenKind::Hosts)?;
            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);
            // HOSTS 暂时映射到 Spaces，因为这是一个单节点实现
            Ok(Stmt::Show(crate::query::parser::ast::stmt::ShowStmt {
                span,
                target: crate::query::parser::ast::stmt::ShowTarget::Spaces,
            }))
        } else if ctx.check_token(TokenKind::Parts) {
            ctx.expect_token(TokenKind::Parts)?;
            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);
            // PARTS 暂时映射到 Spaces，因为这是一个单节点实现
            Ok(Stmt::Show(crate::query::parser::ast::stmt::ShowStmt {
                span,
                target: crate::query::parser::ast::stmt::ShowTarget::Spaces,
            }))
        } else if ctx.check_token(TokenKind::Users) {
            ctx.expect_token(TokenKind::Users)?;
            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);
            Ok(Stmt::ShowUsers(crate::query::parser::ast::stmt::ShowUsersStmt { span }))
        } else if ctx.check_token(TokenKind::Roles) {
            ctx.expect_token(TokenKind::Roles)?;
            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);
            Ok(Stmt::ShowRoles(crate::query::parser::ast::stmt::ShowRolesStmt { 
                span,
                space_name: None,
            }))
        } else if ctx.check_token(TokenKind::Create) {
            // SHOW CREATE 需要特殊处理 - 简化实现
            ctx.expect_token(TokenKind::Create)?;
            // 解析 TAG 或 EDGE
            let target = if ctx.match_token(TokenKind::Tag) {
                let name = ctx.expect_identifier()?;
                crate::query::parser::ast::stmt::ShowCreateTarget::Tag(name)
            } else if ctx.match_token(TokenKind::Edge) {
                let name = ctx.expect_identifier()?;
                crate::query::parser::ast::stmt::ShowCreateTarget::Edge(name)
            } else {
                return Err(ParseError::new(
                    ParseErrorKind::SyntaxError,
                    "SHOW CREATE 期望 TAG 或 EDGE".to_string(),
                    ctx.current_position(),
                ));
            };
            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);
            Ok(Stmt::ShowCreate(crate::query::parser::ast::stmt::ShowCreateStmt { 
                span,
                target,
            }))
        } else {
            Err(ParseError::new(
                ParseErrorKind::SyntaxError,
                format!("未知的 SHOW 目标: {:?}", ctx.peek_token().kind),
                ctx.current_position(),
            ))
        }
    }

    /// 解析 KILL 语句
    fn parse_kill_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        use crate::query::parser::ast::stmt::KillQueryStmt;

        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Kill)?;
        ctx.expect_token(TokenKind::Query)?;

        // 解析 session_id
        let session_id = ctx.expect_integer_literal()?;

        // 解析逗号
        ctx.expect_token(TokenKind::Comma)?;

        // 解析 plan_id
        let plan_id = ctx.expect_integer_literal()?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::KillQuery(KillQueryStmt {
            span,
            session_id,
            plan_id,
        }))
    }

    /// 解析扩展的 UPDATE 语句（包括 UPDATE CONFIGS）
    fn parse_update_statement_extended(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        use crate::query::parser::ast::stmt::UpdateConfigsStmt;
        use crate::query::parser::parser::dml_parser::DmlParser;

        // 检查是否是 UPDATE CONFIGS
        // 先消费 UPDATE token
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Update)?;

        if ctx.check_token(TokenKind::Configs) {
            // 解析 UPDATE CONFIGS
            ctx.expect_token(TokenKind::Configs)?;

            // 先解析第一个标识符
            let first_ident = ctx.expect_identifier()?;

            // 检查下一个 token 是否是 '='，如果是，则第一个标识符是配置名
            // 否则，第一个标识符是模块名，还需要解析配置名
            let (module, config_name) = if ctx.check_token(TokenKind::Assign) {
                (None, first_ident)
            } else {
                (Some(first_ident), ctx.expect_identifier()?)
            };

            // 解析等号和值
            ctx.expect_token(TokenKind::Assign)?;
            let config_value = self.parse_expression(ctx)?;

            let end_span = ctx.current_span();
            let span = ctx.merge_span(start_span.start, end_span.end);

            Ok(Stmt::UpdateConfigs(UpdateConfigsStmt {
                span,
                module,
                config_name,
                config_value,
            }))
        } else {
            // 不是 UPDATE CONFIGS，回退到普通的 UPDATE 解析
            // 由于我们已经消费了 UPDATE token，需要调用 DML 解析器的其他方法
            // 这里我们直接调用 parse_update_statement 并处理错误
            // 实际上应该重构，但这里先这样处理
            DmlParser::new().parse_update_after_token(ctx, start_span)
        }
    }

    /// 解析变量赋值语句 ($var = statement)
    fn parse_assignment_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        use crate::query::parser::ast::stmt::AssignmentStmt;

        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Dollar)?;

        // 解析变量名
        let var_name = ctx.expect_identifier()?;

        // 解析等号
        ctx.expect_token(TokenKind::Assign)?;

        // 解析右侧语句
        let statement = Box::new(self.parse_statement(ctx)?);

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Assignment(AssignmentStmt {
            span,
            variable: var_name,
            statement,
        }))
    }

    /// 解析集合操作语句后的管道或结束
    fn parse_set_operation_suffix(&mut self, ctx: &mut ParseContext, left: Stmt) -> Result<Stmt, ParseError> {
        use crate::query::parser::ast::stmt::{SetOperationStmt, SetOperationType};

        // 检查是否是集合操作符
        let op_type = if ctx.match_token(TokenKind::Union) {
            if ctx.match_token(TokenKind::All) {
                SetOperationType::UnionAll
            } else {
                SetOperationType::Union
            }
        } else if ctx.match_token(TokenKind::Intersect) {
            SetOperationType::Intersect
        } else if ctx.match_token(TokenKind::SetMinus) {
            SetOperationType::Minus
        } else {
            // 不是集合操作符，返回左侧语句
            return Ok(left);
        };

        let start_span = left.span();
        let right = self.parse_single_statement(ctx)?;
        let end_span = right.span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        let set_op_stmt = Stmt::SetOperation(SetOperationStmt {
            span,
            op_type,
            left: Box::new(left),
            right: Box::new(right),
        });

        // 继续检查是否有更多的集合操作
        self.parse_set_operation_suffix(ctx, set_op_stmt)
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

    #[test]
    fn test_parse_explain_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("EXPLAIN MATCH (n) RETURN n");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "EXPLAIN 解析失败: {:?}", result.err());

        if let Ok(Stmt::Explain(stmt)) = result {
            assert!(matches!(stmt.format, ExplainFormat::Table));
        } else {
            panic!("期望 Explain 语句");
        }
    }

    #[test]
    fn test_parse_explain_with_format() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("EXPLAIN FORMAT = DOT MATCH (n) RETURN n");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "EXPLAIN FORMAT 解析失败: {:?}", result.err());

        if let Ok(Stmt::Explain(stmt)) = result {
            assert!(matches!(stmt.format, ExplainFormat::Dot));
        } else {
            panic!("期望 Explain 语句");
        }
    }

    #[test]
    fn test_parse_profile_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("PROFILE GO FROM \"player100\" OVER follow");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "PROFILE 解析失败: {:?}", result.err());

        if let Ok(Stmt::Profile(stmt)) = result {
            assert!(matches!(stmt.format, ExplainFormat::Table));
        } else {
            panic!("期望 Profile 语句");
        }
    }

    #[test]
    fn test_parse_profile_with_format() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("PROFILE FORMAT = TABLE MATCH (n) RETURN n");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "PROFILE FORMAT 解析失败: {:?}", result.err());

        if let Ok(Stmt::Profile(stmt)) = result {
            assert!(matches!(stmt.format, ExplainFormat::Table));
        } else {
            panic!("期望 Profile 语句");
        }
    }

    #[test]
    fn test_parse_group_by_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("GROUP BY category YIELD category");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "GROUP BY 解析失败: {:?}", result.err());

        if let Ok(Stmt::GroupBy(stmt)) = result {
            assert_eq!(stmt.group_items.len(), 1);
            assert_eq!(stmt.yield_clause.items.len(), 1);
            assert!(stmt.having_clause.is_none());
        } else {
            panic!("期望 GroupBy 语句");
        }
    }

    #[test]
    fn test_parse_group_by_multiple_items() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("GROUP BY category, type YIELD category, type");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "GROUP BY 多字段解析失败: {:?}", result.err());

        if let Ok(Stmt::GroupBy(stmt)) = result {
            assert_eq!(stmt.group_items.len(), 2);
            assert_eq!(stmt.yield_clause.items.len(), 2);
        } else {
            panic!("期望 GroupBy 语句");
        }
    }

    #[test]
    fn test_parse_show_sessions() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("SHOW SESSIONS");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "SHOW SESSIONS 解析失败: {:?}", result.err());

        if let Ok(Stmt::ShowSessions(_)) = result {
            // 成功
        } else {
            panic!("期望 ShowSessions 语句");
        }
    }

    #[test]
    fn test_parse_show_queries() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("SHOW QUERIES");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "SHOW QUERIES 解析失败: {:?}", result.err());

        if let Ok(Stmt::ShowQueries(_)) = result {
            // 成功
        } else {
            panic!("期望 ShowQueries 语句");
        }
    }

    #[test]
    fn test_parse_kill_query() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("KILL QUERY 123, 456");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "KILL QUERY 解析失败: {:?}", result.err());

        if let Ok(Stmt::KillQuery(stmt)) = result {
            assert_eq!(stmt.session_id, 123);
            assert_eq!(stmt.plan_id, 456);
        } else {
            panic!("期望 KillQuery 语句");
        }
    }

    #[test]
    fn test_parse_show_configs() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("SHOW CONFIGS");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "SHOW CONFIGS 解析失败: {:?}", result.err());

        if let Ok(Stmt::ShowConfigs(stmt)) = result {
            assert!(stmt.module.is_none());
        } else {
            panic!("期望 ShowConfigs 语句");
        }
    }

    #[test]
    fn test_parse_show_configs_with_module() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("SHOW CONFIGS storage");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "SHOW CONFIGS storage 解析失败: {:?}", result.err());

        if let Ok(Stmt::ShowConfigs(stmt)) = result {
            assert_eq!(stmt.module, Some("storage".to_string()));
        } else {
            panic!("期望 ShowConfigs 语句");
        }
    }

    #[test]
    fn test_parse_update_configs() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("UPDATE CONFIGS max_connections = 100");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "UPDATE CONFIGS 解析失败: {:?}", result.err());

        if let Ok(Stmt::UpdateConfigs(stmt)) = result {
            assert!(stmt.module.is_none());
            assert_eq!(stmt.config_name, "max_connections");
        } else {
            panic!("期望 UpdateConfigs 语句");
        }
    }

    #[test]
    fn test_parse_update_configs_with_module() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("UPDATE CONFIGS storage cache_size = 1024");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "UPDATE CONFIGS storage 解析失败: {:?}", result.err());

        if let Ok(Stmt::UpdateConfigs(stmt)) = result {
            assert_eq!(stmt.module, Some("storage".to_string()));
            assert_eq!(stmt.config_name, "cache_size");
        } else {
            panic!("期望 UpdateConfigs 语句");
        }
    }

    #[test]
    fn test_parse_assignment_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("$result = GO FROM \"player100\" OVER follow");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "变量赋值解析失败: {:?}", result.err());

        if let Ok(Stmt::Assignment(stmt)) = result {
            assert_eq!(stmt.variable, "result");
        } else {
            panic!("期望 Assignment 语句，实际得到 {:?}", result);
        }
    }

    #[test]
    fn test_parse_union_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("GO FROM \"player100\" OVER follow UNION GO FROM \"player101\" OVER follow");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "UNION 解析失败: {:?}", result.err());

        if let Ok(Stmt::SetOperation(stmt)) = result {
            assert!(matches!(stmt.op_type, crate::query::parser::ast::stmt::SetOperationType::Union));
        } else {
            panic!("期望 SetOperation 语句，实际得到 {:?}", result);
        }
    }

    #[test]
    fn test_parse_intersect_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("GO FROM \"player100\" OVER follow INTERSECT GO FROM \"player101\" OVER follow");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "INTERSECT 解析失败: {:?}", result.err());

        if let Ok(Stmt::SetOperation(stmt)) = result {
            assert!(matches!(stmt.op_type, crate::query::parser::ast::stmt::SetOperationType::Intersect));
        } else {
            panic!("期望 SetOperation 语句，实际得到 {:?}", result);
        }
    }

    #[test]
    fn test_parse_minus_statement() {
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("GO FROM \"player100\" OVER follow MINUS GO FROM \"player101\" OVER follow");
        let result = parser.parse_statement(&mut ctx);
        assert!(result.is_ok(), "MINUS 解析失败: {:?}", result.err());

        if let Ok(Stmt::SetOperation(stmt)) = result {
            assert!(matches!(stmt.op_type, crate::query::parser::ast::stmt::SetOperationType::Minus));
        } else {
            panic!("期望 SetOperation 语句，实际得到 {:?}", result);
        }
    }
}
