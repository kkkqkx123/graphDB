//! AST 构建器
//!
//! 提供方便的 API 来构建复杂的 AST 结构，支持链式调用和类型安全。

use super::node::*;
use super::pattern::*;
use super::statement::*;
use super::types::*;
use super::{AstNode, Expression, Pattern, Span, Statement, ExpressionType, StatementType};

// 明确导入以避免歧义
use super::statement::EdgeDirection;
use crate::core::Value;

/// AST 构建器 - 提供流畅的 API 来构建 AST
pub struct AstBuilder {
    span: Span,
}

impl AstBuilder {
    pub fn new(span: Span) -> Self {
        Self { span }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    // 表达式构建方法

    pub fn constant(&self, value: Value) -> Expr {
        Box::new(ConstantExpr::new(value, self.span))
    }

    pub fn variable(&self, name: impl Into<String>) -> Expr {
        Box::new(VariableExpr::new(name.into(), self.span))
    }

    pub fn binary(
        &self,
        left: Expr,
        op: BinaryOp,
        right: Expr,
    ) -> Expr {
        Box::new(BinaryExpr::new(left, op, right, self.span))
    }

    pub fn unary(&self, op: UnaryOp, operand: Expr) -> Expr {
        Box::new(UnaryExpr::new(op, operand, self.span))
    }

    pub fn function_call(
        &self,
        name: impl Into<String>,
        args: Vec<Expr>,
        distinct: bool,
    ) -> Expr {
        Box::new(FunctionCallExpr::new(
            name.into(),
            args,
            distinct,
            self.span,
        ))
    }

    pub fn property_access(
        &self,
        object: Expr,
        property: impl Into<String>,
    ) -> Expr {
        Box::new(PropertyAccessExpr::new(object, property.into(), self.span))
    }

    pub fn list(&self, elements: Vec<Expr>) -> Expr {
        Box::new(ListExpr::new(elements, self.span))
    }

    pub fn map(&self, pairs: Vec<(String, Expr)>) -> Expr {
        Box::new(MapExpr::new(pairs, self.span))
    }

    pub fn case(
        &self,
        match_expr: Option<Expr>,
        when_then_pairs: Vec<(Expr, Expr)>,
        default: Option<Expr>,
    ) -> Expr {
        Box::new(CaseExpr::new(
            match_expr,
            when_then_pairs,
            default,
            self.span,
        ))
    }

    pub fn subscript(
        &self,
        collection: Expr,
        index: Expr,
    ) -> Expr {
        Box::new(SubscriptExpr::new(collection, index, self.span))
    }

    pub fn predicate(
        &self,
        predicate: PredicateType,
        list: Expr,
        condition: Expr,
    ) -> Expr {
        Box::new(PredicateExpr::new(predicate, list, condition, self.span))
    }

    // 语句构建方法

    pub fn query(&self, statements: Vec<Stmt>) -> Stmt {
        Box::new(QueryStatement::new(statements, self.span))
    }

    pub fn create_node(
        &self,
        identifier: Option<String>,
        labels: Vec<String>,
        if_not_exists: bool,
    ) -> Stmt {
        let target = CreateTarget::Node {
            identifier,
            labels,
            properties: None,
        };
        Box::new(CreateStmt::new(target, if_not_exists, self.span))
    }

    pub fn create_edge(
        &self,
        identifier: Option<String>,
        edge_type: String,
        src: Expr,
        dst: Expr,
        direction: EdgeDirection,
        if_not_exists: bool,
    ) -> Stmt {
        let target = CreateTarget::Edge {
            identifier,
            edge_type,
            src,
            dst,
            direction,
            properties: None,
        };
        Box::new(CreateStmt::new(target, if_not_exists, self.span))
    }

    pub fn match_(&self, patterns: Vec<Pattern>) -> Stmt {
        // 将 Pattern 转换为 MatchClause
        let match_paths: Vec<Pattern> = patterns
            .into_iter()
            .map(|pattern| {
                // 将 Pattern 转换为 PatternSegment
                // 这里需要更复杂的转换逻辑，暂时使用空路径
                Pattern { path: vec![] }
            })
            .collect();

        let match_detail = MatchClause {
            patterns: match_paths,
            where_clause: None,
            with_clause: None,
        };

        Box::new(MatchStmt::new(
            vec![MatchClause::Match(match_detail)],
            self.span,
        ))
    }

    pub fn delete_vertices(&self, vertices: Vec<Expr>) -> Stmt {
        let target = DeleteTarget::Vertices(vertices);
        Box::new(DeleteStmt::new(target, self.span))
    }

    pub fn delete_edge(
        &self,
        edge_type: String,
        src: Expr,
        dst: Expr,
        rank: Option<Expr>,
    ) -> Stmt {
        let target = DeleteTarget::Edges {
            edge_type,
            src,
            dst,
            rank,
        };
        Box::new(DeleteStmt::new(target, self.span))
    }

    pub fn update_vertex(
        &self,
        vertex: Expr,
        assignments: Vec<Assignment>,
    ) -> Stmt {
        let target = UpdateTarget::Vertex(vertex);
        let set_clause = SetClause { assignments };
        Box::new(UpdateStmt::new(target, set_clause, self.span))
    }

    pub fn go(&self, steps: Steps, from: FromClause, over: OverClause) -> Stmt {
        Box::new(GoStatement::new(steps, from, over, self.span))
    }

    pub fn fetch_vertices(
        &self,
        ids: Vec<Expr>,
        properties: Vec<String>,
    ) -> Stmt {
        let target = FetchTarget::Vertices { ids, properties };
        Box::new(FetchStatement::new(target, self.span))
    }

    pub fn use_(&self, space: String) -> Stmt {
        Box::new(UseStmt::new(space, self.span))
    }

    pub fn show(&self, target: ShowTarget) -> Stmt {
        Box::new(ShowStmt::new(target, self.span))
    }

    pub fn explain(&self, statement: Stmt) -> Stmt {
        Box::new(ExplainStmt::new(statement, self.span))
    }

    // 模式构建方法

    pub fn node_pattern(
        &self,
        identifier: Option<String>,
        labels: Vec<String>,
    ) -> Pattern {
        Box::new(NodePattern::new(identifier, labels, self.span))
    }

    pub fn edge_pattern(
        &self,
        identifier: Option<String>,
        edge_type: Option<String>,
        direction: super::pattern::EdgeDirection,
    ) -> Pattern {
        Box::new(EdgePattern::new(
            identifier, edge_type, direction, self.span,
        ))
    }

    pub fn path_pattern(&self, elements: Vec<PathElement>) -> Pattern {
        Box::new(PathPattern::new(elements, self.span))
    }

    pub fn variable_pattern(&self, name: String) -> Pattern {
        Box::new(VariablePattern::new(name, self.span))
    }
}

/// 专门的表达式构建器
pub struct ExpressionBuilder {
    builder: AstBuilder,
}

impl ExpressionBuilder {
    pub fn new(span: Span) -> Self {
        Self {
            builder: AstBuilder::new(span),
        }
    }

    /// 构建常量表达式
    pub fn constant(&self, value: Value) -> Expr {
        self.builder.constant(value)
    }

    /// 构建变量表达式
    pub fn variable(&self, name: impl Into<String>) -> Expr {
        self.builder.variable(name)
    }

    /// 构建算术表达式
    pub fn add(
        &self,
        left: Expr,
        right: Expr,
    ) -> Expr {
        self.builder.binary(left, BinaryOp::Add, right)
    }

    pub fn sub(
        &self,
        left: Expr,
        right: Expr,
    ) -> Expr {
        self.builder.binary(left, BinaryOp::Sub, right)
    }

    pub fn mul(
        &self,
        left: Expr,
        right: Expr,
    ) -> Expr {
        self.builder.binary(left, BinaryOp::Mul, right)
    }

    pub fn div(
        &self,
        left: Expr,
        right: Expr,
    ) -> Expr {
        self.builder.binary(left, BinaryOp::Div, right)
    }

    /// 构建逻辑表达式
    pub fn and(
        &self,
        left: Expr,
        right: Expr,
    ) -> Expr {
        self.builder.binary(left, BinaryOp::And, right)
    }

    pub fn or(&self, left: Expr, right: Expr) -> Expr {
        self.builder.binary(left, BinaryOp::Or, right)
    }

    pub fn not(&self, expr: Expr) -> Expr {
        self.builder.unary(UnaryOp::Not, expr)
    }

    /// 构建关系表达式
    pub fn eq(&self, left: Expr, right: Expr) -> Expr {
        self.builder.binary(left, BinaryOp::Eq, right)
    }

    pub fn ne(&self, left: Expr, right: Expr) -> Expr {
        self.builder.binary(left, BinaryOp::Ne, right)
    }

    pub fn lt(&self, left: Expr, right: Expr) -> Expr {
        self.builder.binary(left, BinaryOp::Lt, right)
    }

    pub fn le(&self, left: Expr, right: Expr) -> Expr {
        self.builder.binary(left, BinaryOp::Le, right)
    }

    pub fn gt(&self, left: Expr, right: Expr) -> Expr {
        self.builder.binary(left, BinaryOp::Gt, right)
    }

    pub fn ge(&self, left: Expr, right: Expr) -> Expr {
        self.builder.binary(left, BinaryOp::Ge, right)
    }

    /// 构建聚合函数
    pub fn count(&self, expr: Expr) -> Expr {
        self.builder.function_call("COUNT", vec![expr], false)
    }

    pub fn sum(&self, expr: Expr) -> Expr {
        self.builder.function_call("SUM", vec![expr], false)
    }

    pub fn avg(&self, expr: Expr) -> Expr {
        self.builder.function_call("AVG", vec![expr], false)
    }

    pub fn min(&self, expr: Expr) -> Expr {
        self.builder.function_call("MIN", vec![expr], false)
    }

    pub fn max(&self, expr: Expr) -> Expr {
        self.builder.function_call("MAX", vec![expr], false)
    }

    /// 构建谓词表达式
    pub fn all(
        &self,
        list: Expr,
        condition: Expr,
    ) -> Expr {
        self.builder.predicate(PredicateType::All, list, condition)
    }

    pub fn any(
        &self,
        list: Expr,
        condition: Expr,
    ) -> Expr {
        self.builder.predicate(PredicateType::Any, list, condition)
    }

    pub fn exists(&self, expr: Expr) -> Expr {
        self.builder.predicate(
            PredicateType::Exists,
            expr,
            self.builder.constant(Value::Bool(true)),
        )
    }
}

/// 专门的语句构建器
pub struct StatementBuilder {
    builder: AstBuilder,
}

impl StatementBuilder {
    pub fn new(span: Span) -> Self {
        Self {
            builder: AstBuilder::new(span),
        }
    }

    /// 构建节点模式
    pub fn node_pattern(
        &self,
        identifier: Option<String>,
        labels: Vec<String>,
    ) -> Pattern {
        self.builder.node_pattern(identifier, labels)
    }

    /// 构建 MATCH 语句
    pub fn match_pattern(&self, pattern: Pattern) -> MatchStmt {
        let match_paths = vec![Pattern { path: vec![] }]; // 简化处理
        let match_detail = MatchClause {
            patterns: match_paths,
            where_clause: None,
            with_clause: None,
        };
        MatchStmt::new(vec![MatchClause::Match(match_detail)], self.builder.span)
    }

    pub fn match_patterns(&self, patterns: Vec<Pattern>) -> MatchStmt {
        let match_paths: Vec<Pattern> = patterns
            .into_iter()
            .map(|pattern| Pattern { path: vec![] }) // 简化处理
            .collect();
        let match_detail = MatchClause {
            patterns: match_paths,
            where_clause: None,
            with_clause: None,
        };
        MatchStmt::new(vec![MatchClause::Match(match_detail)], self.builder.span)
    }

    /// 构建 CREATE 语句
    pub fn create_node(&self, identifier: Option<String>, labels: Vec<String>) -> CreateStmt {
        CreateStmt::new(
            CreateTarget::Node {
                identifier,
                labels,
                properties: None,
            },
            false,
            self.builder.span,
        )
    }

    pub fn create_edge(
        &self,
        identifier: Option<String>,
        edge_type: String,
        src: Expr,
        dst: Expr,
        direction: EdgeDirection,
    ) -> CreateStmt {
        CreateStmt::new(
            CreateTarget::Edge {
                identifier,
                edge_type,
                src,
                dst,
                direction,
                properties: None,
            },
            false,
            self.builder.span,
        )
    }

    /// 构建 GO 语句
    pub fn go_steps(
        &self,
        steps: u32,
        from: Vec<Expr>,
        over: Vec<String>,
    ) -> GoStatement {
        let steps_enum = Steps::Fixed(steps);
        let from_clause = FromClause { vertices: from };
        let over_clause = OverClause {
            edge_types: over,
            direction: EdgeDirection::Outbound,
            reversely: false,
        };

        GoStatement::new(steps_enum, from_clause, over_clause, self.builder.span)
    }
}

/// 辅助构建器宏
#[macro_export]
macro_rules! ast_builder {
    ($span:expr) => {
        AstBuilder::new($span)
    };
}

#[macro_export]
macro_rules! expr_builder {
    ($span:expr) => {
        ExpressionBuilder::new($span)
    };
}

#[macro_export]
macro_rules! stmt_builder {
    ($span:expr) => {
        StatementBuilder::new($span)
    };
}

/// 便捷的表达式构建宏
#[macro_export]
macro_rules! binary_expr {
    ($builder:expr, $left:expr, $op:expr, $right:expr) => {
        $builder.binary($left, $op, $right)
    };
}

#[macro_export]
macro_rules! and_expr {
    ($builder:expr, $left:expr, $right:expr) => {
        $builder.binary($left, BinaryOp::And, $right)
    };
}

#[macro_export]
macro_rules! or_expr {
    ($builder:expr, $left:expr, $right:expr) => {
        $builder.binary($left, BinaryOp::Or, $right)
    };
}

#[macro_export]
macro_rules! eq_expr {
    ($builder:expr, $left:expr, $right:expr) => {
        $builder.binary($left, BinaryOp::Eq, $right)
    };
}

/// 测试模块
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_builder() {
        let span = Span::default();
        let builder = AstBuilder::new(span);

        // 构建简单的常量表达式
        let expr = builder.constant(Value::Int(42));
        assert_eq!(expr.expr_type(), Expr::Constant);

        // 构建变量表达式
        let var_expr = builder.variable("x");
        assert_eq!(var_expr.expr_type(), Expr::Variable);
    }

    #[test]
    fn test_expression_builder() {
        let span = Span::default();
        let builder = ExpressionBuilder::new(span);

        let left = builder.constant(Value::Int(5));
        let right = builder.constant(Value::Int(3));
        let add_expr = builder.add(left, right);

        assert_eq!(add_expr.expr_type(), Expr::Binary);
        assert!(add_expr.is_constant());
    }

    #[test]
    fn test_statement_builder() {
        let span = Span::default();
        let builder = StatementBuilder::new(span);

        let pattern = builder.node_pattern(Some("n".to_string()), vec!["Person".to_string()]);
        let match_stmt = builder.match_pattern(pattern);

        assert_eq!(match_stmt.stmt_type(), Stmt::Match);
    }

    #[test]
    fn test_macros() {
        let span = Span::default();
        let builder = ast_builder!(span);

        let left = builder.constant(Value::Int(5));
        let right = builder.constant(Value::Int(3));
        let expr = binary_expr!(builder, left, BinaryOp::Add, right);

        assert_eq!(expr.expr_type(), Expr::Binary);
    }

    #[test]
    fn test_complex_expression() {
        let span = Span::default();
        let builder = ExpressionBuilder::new(span);

        // 构建: (x > 10) AND (y < 20)
        let x = builder.variable("x");
        let ten = builder.constant(Value::Int(10));
        let y = builder.variable("y");
        let twenty = builder.constant(Value::Int(20));

        let cond1 = builder.gt(x, ten);
        let cond2 = builder.lt(y, twenty);
        let combined = builder.and(cond1, cond2);

        assert_eq!(combined.expr_type(), Expr::Binary);
        assert!(!combined.is_constant()); // 包含变量，不是常量
    }
}
