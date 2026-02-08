//! 工具函数和辅助功能

use super::pattern::*;
use super::stmt::*;
use super::types::*;
use crate::core::Value;
use crate::core::types::expression::Expression;

/// 表达式工厂 - 用于创建表达式节点
pub struct ExprFactory;

impl ExprFactory {
    /// 创建常量表达式
    pub fn constant(value: Value) -> Expression {
        Expression::Literal(value)
    }

    /// 创建变量表达式
    pub fn variable(name: String) -> Expression {
        Expression::Variable(name)
    }

    /// 创建二元表达式
    pub fn binary(left: Expression, op: crate::core::types::operators::BinaryOperator, right: Expression) -> Expression {
        Expression::Binary { left: Box::new(left), op, right: Box::new(right) }
    }

    /// 创建一元表达式
    pub fn unary(op: crate::core::types::operators::UnaryOperator, operand: Expression) -> Expression {
        Expression::Unary { op, operand: Box::new(operand) }
    }

    /// 创建函数调用表达式
    pub fn function_call(name: String, args: Vec<Expression>, _distinct: bool) -> Expression {
        Expression::Function { name, args }
    }

    /// 创建属性访问表达式
    pub fn property_access(object: Expression, property: String) -> Expression {
        Expression::Property { object: Box::new(object), property }
    }

    /// 创建列表表达式
    pub fn list(elements: Vec<Expression>) -> Expression {
        Expression::List(elements)
    }

    /// 创建映射表达式
    pub fn map(pairs: Vec<(String, Expression)>) -> Expression {
        Expression::Map(pairs)
    }

    /// 创建 CASE 表达式
    pub fn case(
        match_expression: Option<Expression>,
        when_then_pairs: Vec<(Expression, Expression)>,
        default: Option<Expression>,
    ) -> Expression {
        let conditions = when_then_pairs;
        let default = default.map(Box::new);
        Expression::Case { test_expr: match_expression.map(Box::new), conditions, default }
    }

    /// 创建下标表达式
    pub fn subscript(collection: Expression, index: Expression) -> Expression {
        Expression::Subscript { collection: Box::new(collection), index: Box::new(index) }
    }

    /// 创建比较表达式
    pub fn compare(left: Expression, op: crate::core::types::operators::BinaryOperator, right: Expression) -> Expression {
        Self::binary(left, op, right)
    }

    /// 创建逻辑表达式
    pub fn logical(left: Expression, op: crate::core::types::operators::BinaryOperator, right: Expression) -> Expression {
        Self::binary(left, op, right)
    }

    /// 创建算术表达式
    pub fn arithmetic(left: Expression, op: crate::core::types::operators::BinaryOperator, right: Expression) -> Expression {
        Self::binary(left, op, right)
    }
}

/// 语句工厂 - 用于创建语句节点
pub struct StmtFactory;

impl StmtFactory {
    /// 创建查询语句
    pub fn query(statements: Vec<Stmt>, span: Span) -> Stmt {
        Stmt::Query(QueryStmt::new(statements, span))
    }

    /// 创建 CREATE 节点语句
    pub fn create_node(
        variable: Option<String>,
        labels: Vec<String>,
        properties: Option<Expression>,
        span: Span,
    ) -> Stmt {
        Stmt::Create(CreateStmt {
            span,
            target: CreateTarget::Node {
                variable,
                labels,
                properties,
            },
        })
    }

    /// 创建 CREATE 边语句
    pub fn create_edge(
        variable: Option<String>,
        edge_type: String,
        src: Expression,
        dst: Expression,
        properties: Option<Expression>,
        direction: EdgeDirection,
        span: Span,
    ) -> Stmt {
        Stmt::Create(CreateStmt {
            span,
            target: CreateTarget::Edge {
                variable,
                edge_type,
                src,
                dst,
                properties,
                direction,
            },
        })
    }

    /// 创建 MATCH 语句
    pub fn match_stmt(
        patterns: Vec<Pattern>,
        where_clause: Option<Expression>,
        return_clause: Option<ReturnClause>,
        order_by: Option<OrderByClause>,
        limit: Option<usize>,
        skip: Option<usize>,
        span: Span,
    ) -> Stmt {
        Stmt::Match(MatchStmt {
            span,
            patterns,
            where_clause,
            return_clause,
            order_by,
            limit,
            skip,
        })
    }

    /// 创建 DELETE 语句
    pub fn delete(target: DeleteTarget, where_clause: Option<Expression>, span: Span) -> Stmt {
        Stmt::Delete(DeleteStmt {
            span,
            target,
            where_clause,
        })
    }

    /// 创建 UPDATE 语句
    pub fn update(
        target: UpdateTarget,
        set_clause: SetClause,
        where_clause: Option<Expression>,
        span: Span,
    ) -> Stmt {
        Stmt::Update(UpdateStmt {
            span,
            target,
            set_clause,
            where_clause,
        })
    }

    /// 创建 GO 语句
    pub fn go(
        steps: Steps,
        from: FromClause,
        over: Option<OverClause>,
        where_clause: Option<Expression>,
        yield_clause: Option<YieldClause>,
        span: Span,
    ) -> Stmt {
        Stmt::Go(GoStmt {
            span,
            steps,
            from,
            over,
            where_clause,
            yield_clause,
        })
    }

    /// 创建 FETCH 语句
    pub fn fetch(target: FetchTarget, span: Span) -> Stmt {
        Stmt::Fetch(FetchStmt { span, target })
    }

    /// 创建 USE 语句
    pub fn r#use(space: String, span: Span) -> Stmt {
        Stmt::Use(UseStmt { span, space })
    }

    /// 创建 SHOW 语句
    pub fn show(target: ShowTarget, span: Span) -> Stmt {
        Stmt::Show(ShowStmt { span, target })
    }

    /// 创建 EXPLAIN 语句
    pub fn explain(statement: Box<Stmt>, span: Span) -> Stmt {
        Stmt::Explain(ExplainStmt { span, statement })
    }

    /// 创建 LOOKUP 语句
    pub fn lookup(
        target: LookupTarget,
        where_clause: Option<Expression>,
        yield_clause: Option<YieldClause>,
        span: Span,
    ) -> Stmt {
        Stmt::Lookup(LookupStmt {
            span,
            target,
            where_clause,
            yield_clause,
        })
    }

    /// 创建 SUBGRAPH 语句
    pub fn subgraph(
        steps: Steps,
        from: FromClause,
        over: Option<OverClause>,
        where_clause: Option<Expression>,
        yield_clause: Option<YieldClause>,
        span: Span,
    ) -> Stmt {
        Stmt::Subgraph(SubgraphStmt {
            span,
            steps,
            from,
            over,
            where_clause,
            yield_clause,
        })
    }

    /// 创建 FIND PATH 语句
    pub fn find_path(
        from: FromClause,
        to: Expression,
        over: Option<OverClause>,
        where_clause: Option<Expression>,
        shortest: bool,
        yield_clause: Option<YieldClause>,
        span: Span,
    ) -> Stmt {
        Stmt::FindPath(FindPathStmt {
            span,
            from,
            to,
            over,
            where_clause,
            shortest,
            yield_clause,
        })
    }
}

/// 模式工厂 - 用于创建模式节点
pub struct PatternFactory;

impl PatternFactory {
    /// 创建节点模式
    pub fn node(
        variable: Option<String>,
        labels: Vec<String>,
        properties: Option<Expression>,
        predicates: Vec<Expression>,
        span: Span,
    ) -> Pattern {
        Pattern::Node(NodePattern::new(
            variable, labels, properties, predicates, span,
        ))
    }

    /// 创建边模式
    pub fn edge(
        variable: Option<String>,
        edge_types: Vec<String>,
        properties: Option<Expression>,
        predicates: Vec<Expression>,
        direction: EdgeDirection,
        range: Option<EdgeRange>,
        span: Span,
    ) -> Pattern {
        Pattern::Edge(EdgePattern::new(
            variable, edge_types, properties, predicates, direction, range, span,
        ))
    }

    /// 创建路径模式
    pub fn path(elements: Vec<PathElement>, span: Span) -> Pattern {
        Pattern::Path(PathPattern::new(elements, span))
    }

    /// 创建变量模式
    pub fn variable(name: String, span: Span) -> Pattern {
        Pattern::Variable(VariablePattern::new(name, span))
    }

    /// 创建简单的节点模式
    pub fn simple_node(variable: Option<String>, labels: Vec<String>, span: Span) -> Pattern {
        Self::node(variable, labels, None, vec![], span)
    }

    /// 创建简单的边模式
    pub fn simple_edge(
        variable: Option<String>,
        edge_types: Vec<String>,
        direction: EdgeDirection,
        span: Span,
    ) -> Pattern {
        Self::edge(variable, edge_types, None, vec![], direction, None, span)
    }

    /// 创建有向边模式
    pub fn directed_edge(
        variable: Option<String>,
        edge_type: String,
        direction: EdgeDirection,
        span: Span,
    ) -> Pattern {
        Self::simple_edge(variable, vec![edge_type], direction, span)
    }

    /// 创建无向边模式
    pub fn undirected_edge(variable: Option<String>, edge_type: String, span: Span) -> Pattern {
        Self::simple_edge(variable, vec![edge_type], EdgeDirection::Both, span)
    }
}

/// AST 构建器 - 用于构建复杂的 AST 结构
pub struct AstBuilder {
    span: Span,
}

impl AstBuilder {
    pub fn new(span: Span) -> Self {
        Self { span }
    }

    /// 构建简单的 MATCH 查询
    pub fn build_simple_match(&self, pattern: Pattern, return_expression: Expression) -> Stmt {
        let return_clause = ReturnClause {
            span: self.span,
            items: vec![ReturnItem::Expression {
                expression: return_expression,
                alias: None,
            }],
            distinct: false,
            limit: None,
            skip: None,
            sample: None,
        };

        StmtFactory::match_stmt(
            vec![pattern],
            None,
            Some(return_clause),
            None,
            None,
            None,
            self.span,
        )
    }

    /// 构建简单的 CREATE 节点查询
    pub fn build_create_node(&self, variable: Option<String>, labels: Vec<String>) -> Stmt {
        StmtFactory::create_node(variable, labels, None, self.span)
    }

    /// 构建简单的 CREATE 边查询
    pub fn build_create_edge(
        &self,
        variable: Option<String>,
        edge_type: String,
        src: Expression,
        dst: Expression,
        direction: EdgeDirection,
    ) -> Stmt {
        StmtFactory::create_edge(variable, edge_type, src, dst, None, direction, self.span)
    }

    /// 构建简单的 DELETE 查询
    pub fn build_delete_vertices(&self, vertices: Vec<Expression>) -> Stmt {
        StmtFactory::delete(DeleteTarget::Vertices(vertices), None, self.span)
    }

    /// 构建简单的 UPDATE 查询
    pub fn build_update_vertex(&self, vertex: Expression, assignments: Vec<Assignment>) -> Stmt {
        let set_clause = SetClause {
            span: self.span,
            assignments,
        };

        StmtFactory::update(UpdateTarget::Vertex(vertex), set_clause, None, self.span)
    }
}

/// 表达式优化器 - 用于优化表达式
pub struct ExprOptimizer;

impl ExprOptimizer {
    /// 常量折叠优化
    pub fn constant_folding(expression: Expression) -> Expression {
        match expression {
            Expression::Binary { left, op, right } => {
                let optimized_left = Self::constant_folding(*left);
                let optimized_right = Self::constant_folding(*right);

                // 如果左右操作数都是常量，尝试计算结果
                if let (Expression::Literal(ref left_value), Expression::Literal(ref right_value)) =
                    (&optimized_left, &optimized_right)
                {
                    if let Some(result) =
                        Self::evaluate_binary_op(left_value, op, right_value)
                    {
                        return Expression::Literal(result);
                    }
                }

                Expression::Binary { left: Box::new(optimized_left), op, right: Box::new(optimized_right) }
            }
            Expression::Unary { op, operand } => {
                let optimized_operand = Self::constant_folding(*operand);

                // 如果操作数是常量，尝试计算结果
                if let Expression::Literal(ref operand_value) = optimized_operand {
                    if let Some(result) = Self::evaluate_unary_op(op, operand_value) {
                        return Expression::Literal(result);
                    }
                }

                Expression::Unary { op, operand: Box::new(optimized_operand) }
            }
            Expression::List(elements) => {
                let optimized_elements = elements.into_iter().map(Self::constant_folding).collect();
                Expression::List(optimized_elements)
            }
            Expression::Map(pairs) => {
                let optimized_pairs = pairs
                    .into_iter()
                    .map(|(key, value)| (key, Self::constant_folding(value)))
                    .collect();
                Expression::Map(optimized_pairs)
            }
            Expression::Subscript { collection, index, .. } => {
                let optimized_collection = Self::constant_folding(*collection);
                let optimized_index = Self::constant_folding(*index);
                Expression::Subscript { collection: Box::new(optimized_collection), index: Box::new(optimized_index) }
            }
            _ => expression,
        }
    }

    /// 评估二元操作符
    fn evaluate_binary_op(left: &Value, op: BinaryOp, right: &Value) -> Option<Value> {
        use crate::core::Value;

        match (left, op, right) {
            (Value::Int(l), BinaryOp::Add, Value::Int(r)) => Some(Value::Int(l + r)),
            (Value::Int(l), BinaryOp::Subtract, Value::Int(r)) => Some(Value::Int(l - r)),
            (Value::Int(l), BinaryOp::Multiply, Value::Int(r)) => Some(Value::Int(l * r)),
            (Value::Int(l), BinaryOp::Divide, Value::Int(r)) => {
                if *r != 0 {
                    Some(Value::Int(l / r))
                } else {
                    None
                }
            }
            (Value::Int(l), BinaryOp::Modulo, Value::Int(r)) => {
                if *r != 0 {
                    Some(Value::Int(l % r))
                } else {
                    None
                }
            }
            (Value::Float(l), BinaryOp::Add, Value::Float(r)) => Some(Value::Float(l + r)),
            (Value::Float(l), BinaryOp::Subtract, Value::Float(r)) => Some(Value::Float(l - r)),
            (Value::Float(l), BinaryOp::Multiply, Value::Float(r)) => Some(Value::Float(l * r)),
            (Value::Float(l), BinaryOp::Divide, Value::Float(r)) => {
                if *r != 0.0 {
                    Some(Value::Float(l / r))
                } else {
                    None
                }
            }
            (Value::String(l), BinaryOp::Add, Value::String(r)) => {
                Some(Value::String(format!("{}{}", l, r)))
            }
            _ => None,
        }
    }

    /// 评估一元操作符
    fn evaluate_unary_op(op: UnaryOp, operand: &Value) -> Option<Value> {
        use crate::core::Value;

        match (op, operand) {
            (UnaryOp::Minus, Value::Int(v)) => Some(Value::Int(-v)),
            (UnaryOp::Minus, Value::Float(v)) => Some(Value::Float(-v)),
            (UnaryOp::Not, Value::Bool(v)) => Some(Value::Bool(!v)),
            _ => None,
        }
    }

    /// 表达式简化
    pub fn simplify(expression: Expression) -> Expression {
        let folded = Self::constant_folding(expression);
        Self::remove_redundant_operations(folded)
    }

    /// 移除冗余操作
    fn remove_redundant_operations(expression: Expression) -> Expression {
        match expression {
            Expression::Binary { left, op, right, .. } => {
                let left = Self::remove_redundant_operations(*left);
                let right = Self::remove_redundant_operations(*right);

                // 简化：x + 0 -> x
                if op == crate::core::types::operators::BinaryOperator::Add {
                    if let Expression::Literal(Value::Int(0) | Value::Float(0.0)) = &right {
                        return left;
                    }
                    if let Expression::Literal(Value::Int(0) | Value::Float(0.0)) = &left {
                        return right;
                    }
                }

                // 简化：x * 1 -> x
                if op == crate::core::types::operators::BinaryOperator::Multiply {
                    if let Expression::Literal(Value::Int(1) | Value::Float(1.0)) = &right {
                        return left;
                    }
                    if let Expression::Literal(Value::Int(1) | Value::Float(1.0)) = &left {
                        return right;
                    }
                }

                // 简化：x * 0 -> 0
                if op == crate::core::types::operators::BinaryOperator::Multiply {
                    if let Expression::Literal(Value::Int(0) | Value::Float(0.0)) = &right {
                        return right;
                    }
                    if let Expression::Literal(Value::Int(0) | Value::Float(0.0)) = &left {
                        return left;
                    }
                }

                Expression::Binary { left: Box::new(left), op, right: Box::new(right) }
            }
            Expression::Unary { op, operand, .. } => {
                let operand = Self::remove_redundant_operations(*operand);

                // 简化：+x -> x
                if op == crate::core::types::operators::UnaryOperator::Plus {
                    return operand;
                }

                // 简化：!!x -> x
                if op == crate::core::types::operators::UnaryOperator::Not {
                    if let Expression::Unary { op: inner_op, operand: inner_operand, .. } = &operand {
                        if *inner_op == crate::core::types::operators::UnaryOperator::Not {
                            return *inner_operand.clone();
                        }
                    }
                }

                Expression::Unary { op, operand: Box::new(operand) }
            }
            _ => expression,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_expr_factory() {
        // 测试常量表达式
        let const_expression = ExprFactory::constant(Value::Int(42));
        assert!(matches!(const_expression, Expression::Literal(_)));

        // 测试变量表达式
        let var_expression = ExprFactory::variable("x".to_string());
        assert!(matches!(var_expression, Expression::Variable(_)));

        // 测试二元表达式
        let left = ExprFactory::constant(Value::Int(5));
        let right = ExprFactory::constant(Value::Int(3));
        let binary_expression = ExprFactory::binary(left, crate::core::types::operators::BinaryOperator::Add, right);
        assert!(matches!(binary_expression, Expression::Binary { left: _, op: _, right: _ }));
    }

    #[test]
    fn test_constant_folding() {
        // 测试 5 + 3 -> 8
        let left = ExprFactory::constant(Value::Int(5));
        let right = ExprFactory::constant(Value::Int(3));
        let expression = ExprFactory::binary(left, crate::core::types::operators::BinaryOperator::Add, right);

        let optimized = ExprOptimizer::constant_folding(expression);
        assert!(matches!(optimized, Expression::Literal(Value::Int(8))));
    }

    #[test]
    fn test_unary_minus() {
        // 测试 -5 -> -5
        let operand = ExprFactory::constant(Value::Int(5));
        let expression = ExprFactory::unary(crate::core::types::operators::UnaryOperator::Minus, operand);

        let optimized = ExprOptimizer::constant_folding(expression);
        assert!(matches!(optimized, Expression::Literal(Value::Int(-5))));
    }

    #[test]
    fn test_expression_simplification() {
        // 测试 x + 0 -> x
        let x = ExprFactory::variable("x".to_string());
        let zero = ExprFactory::constant(Value::Int(0));
        let expression = ExprFactory::binary(x.clone(), crate::core::types::operators::BinaryOperator::Add, zero);

        let simplified = ExprOptimizer::simplify(expression);
        assert_eq!(simplified, x);

        // 测试 x * 1 -> x
        let x = ExprFactory::variable("x".to_string());
        let one = ExprFactory::constant(Value::Int(1));
        let expression = ExprFactory::binary(x.clone(), crate::core::types::operators::BinaryOperator::Multiply, one);

        let simplified = ExprOptimizer::simplify(expression);
        assert_eq!(simplified, x);

        // 测试 !!x -> x
        let x = ExprFactory::variable("x".to_string());
        let not_expression = ExprFactory::unary(crate::core::types::operators::UnaryOperator::Not, x.clone());
        let expression = ExprFactory::unary(crate::core::types::operators::UnaryOperator::Not, not_expression);

        let simplified = ExprOptimizer::simplify(expression);
        assert_eq!(simplified, x);
    }
}
