//! 工具函数和辅助功能

use crate::core::Value;
use super::expr::*;
use super::stmt::*;
use super::pattern::*;
use super::types::*;

/// 表达式工厂 - 用于创建表达式节点
pub struct ExprFactory;

impl ExprFactory {
    /// 创建常量表达式
    pub fn constant(value: Value, span: Span) -> Expr {
        Expr::Constant(ConstantExpr::new(value, span))
    }
    
    /// 创建变量表达式
    pub fn variable(name: String, span: Span) -> Expr {
        Expr::Variable(VariableExpr::new(name, span))
    }
    
    /// 创建二元表达式
    pub fn binary(left: Expr, op: BinaryOp, right: Expr, span: Span) -> Expr {
        Expr::Binary(BinaryExpr::new(left, op, right, span))
    }
    
    /// 创建一元表达式
    pub fn unary(op: UnaryOp, operand: Expr, span: Span) -> Expr {
        Expr::Unary(UnaryExpr::new(op, operand, span))
    }
    
    /// 创建函数调用表达式
    pub fn function_call(name: String, args: Vec<Expr>, distinct: bool, span: Span) -> Expr {
        Expr::FunctionCall(FunctionCallExpr::new(name, args, distinct, span))
    }
    
    /// 创建属性访问表达式
    pub fn property_access(object: Expr, property: String, span: Span) -> Expr {
        Expr::PropertyAccess(PropertyAccessExpr::new(object, property, span))
    }
    
    /// 创建列表表达式
    pub fn list(elements: Vec<Expr>, span: Span) -> Expr {
        Expr::List(ListExpr::new(elements, span))
    }
    
    /// 创建映射表达式
    pub fn map(pairs: Vec<(String, Expr)>, span: Span) -> Expr {
        Expr::Map(MapExpr::new(pairs, span))
    }
    
    /// 创建 CASE 表达式
    pub fn case(
        match_expr: Option<Expr>,
        when_then_pairs: Vec<(Expr, Expr)>,
        default: Option<Expr>,
        span: Span,
    ) -> Expr {
        Expr::Case(CaseExpr::new(match_expr, when_then_pairs, default, span))
    }
    
    /// 创建下标表达式
    pub fn subscript(collection: Expr, index: Expr, span: Span) -> Expr {
        Expr::Subscript(SubscriptExpr::new(collection, index, span))
    }
    
    /// 创建谓词表达式
    pub fn predicate(predicate: PredicateType, list: Expr, condition: Expr, span: Span) -> Expr {
        Expr::Predicate(PredicateExpr::new(predicate, list, condition, span))
    }
    
    /// 创建比较表达式
    pub fn compare(left: Expr, op: BinaryOp, right: Expr, span: Span) -> Expr {
        Self::binary(left, op, right, span)
    }
    
    /// 创建逻辑表达式
    pub fn logical(left: Expr, op: BinaryOp, right: Expr, span: Span) -> Expr {
        Self::binary(left, op, right, span)
    }
    
    /// 创建算术表达式
    pub fn arithmetic(left: Expr, op: BinaryOp, right: Expr, span: Span) -> Expr {
        Self::binary(left, op, right, span)
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
        properties: Option<Expr>,
        span: Span,
    ) -> Stmt {
        Stmt::Create(CreateStmt {
            span,
            target: CreateTarget::Node { variable, labels, properties },
        })
    }
    
    /// 创建 CREATE 边语句
    pub fn create_edge(
        variable: Option<String>,
        edge_type: String,
        src: Expr,
        dst: Expr,
        properties: Option<Expr>,
        direction: EdgeDirection,
        span: Span,
    ) -> Stmt {
        Stmt::Create(CreateStmt {
            span,
            target: CreateTarget::Edge { variable, edge_type, src, dst, properties, direction },
        })
    }
    
    /// 创建 MATCH 语句
    pub fn match_stmt(
        patterns: Vec<Pattern>,
        where_clause: Option<Expr>,
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
    pub fn delete(target: DeleteTarget, where_clause: Option<Expr>, span: Span) -> Stmt {
        Stmt::Delete(DeleteStmt { span, target, where_clause })
    }
    
    /// 创建 UPDATE 语句
    pub fn update(
        target: UpdateTarget,
        set_clause: SetClause,
        where_clause: Option<Expr>,
        span: Span,
    ) -> Stmt {
        Stmt::Update(UpdateStmt { span, target, set_clause, where_clause })
    }
    
    /// 创建 GO 语句
    pub fn go(
        steps: Steps,
        from: FromClause,
        over: Option<OverClause>,
        where_clause: Option<Expr>,
        yield_clause: Option<YieldClause>,
        span: Span,
    ) -> Stmt {
        Stmt::Go(GoStmt { span, steps, from, over, where_clause, yield_clause })
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
    pub fn lookup(target: LookupTarget, where_clause: Option<Expr>, yield_clause: Option<YieldClause>, span: Span) -> Stmt {
        Stmt::Lookup(LookupStmt { span, target, where_clause, yield_clause })
    }
    
    /// 创建 SUBGRAPH 语句
    pub fn subgraph(
        steps: Steps,
        from: FromClause,
        over: Option<OverClause>,
        where_clause: Option<Expr>,
        yield_clause: Option<YieldClause>,
        span: Span,
    ) -> Stmt {
        Stmt::Subgraph(SubgraphStmt { span, steps, from, over, where_clause, yield_clause })
    }
    
    /// 创建 FIND PATH 语句
    pub fn find_path(
        from: FromClause,
        to: Expr,
        over: Option<OverClause>,
        where_clause: Option<Expr>,
        shortest: bool,
        yield_clause: Option<YieldClause>,
        span: Span,
    ) -> Stmt {
        Stmt::FindPath(FindPathStmt { span, from, to, over, where_clause, shortest, yield_clause })
    }
}

/// 模式工厂 - 用于创建模式节点
pub struct PatternFactory;

impl PatternFactory {
    /// 创建节点模式
    pub fn node(
        variable: Option<String>,
        labels: Vec<String>,
        properties: Option<Expr>,
        predicates: Vec<Expr>,
        span: Span,
    ) -> Pattern {
        Pattern::Node(NodePattern::new(variable, labels, properties, predicates, span))
    }
    
    /// 创建边模式
    pub fn edge(
        variable: Option<String>,
        edge_types: Vec<String>,
        properties: Option<Expr>,
        predicates: Vec<Expr>,
        direction: EdgeDirection,
        range: Option<EdgeRange>,
        span: Span,
    ) -> Pattern {
        Pattern::Edge(EdgePattern::new(variable, edge_types, properties, predicates, direction, range, span))
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
    pub fn simple_edge(variable: Option<String>, edge_types: Vec<String>, direction: EdgeDirection, span: Span) -> Pattern {
        Self::edge(variable, edge_types, None, vec![], direction, None, span)
    }
    
    /// 创建有向边模式
    pub fn directed_edge(variable: Option<String>, edge_type: String, direction: EdgeDirection, span: Span) -> Pattern {
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
    pub fn build_simple_match(&self, pattern: Pattern, return_expr: Expr) -> Stmt {
        let return_clause = ReturnClause {
            span: self.span,
            items: vec![ReturnItem::Expression {
                expr: return_expr,
                alias: None,
            }],
            distinct: false,
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
        src: Expr,
        dst: Expr,
        direction: EdgeDirection,
    ) -> Stmt {
        StmtFactory::create_edge(variable, edge_type, src, dst, None, direction, self.span)
    }
    
    /// 构建简单的 DELETE 查询
    pub fn build_delete_vertices(&self, vertices: Vec<Expr>) -> Stmt {
        StmtFactory::delete(DeleteTarget::Vertices(vertices), None, self.span)
    }
    
    /// 构建简单的 UPDATE 查询
    pub fn build_update_vertex(&self, vertex: Expr, assignments: Vec<Assignment>) -> Stmt {
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
    pub fn constant_folding(expr: Expr) -> Expr {
        match expr {
            Expr::Binary(mut e) => {
                e.left = Box::new(Self::constant_folding(*e.left));
                e.right = Box::new(Self::constant_folding(*e.right));
                
                // 如果左右操作数都是常量，尝试计算结果
                if e.left.is_constant() && e.right.is_constant() {
                    if let (Expr::Constant(left), Expr::Constant(right)) = (&*e.left, &*e.right) {
                        if let Some(result) = Self::evaluate_binary_op(&left.value, e.op, &right.value) {
                            return Expr::Constant(ConstantExpr::new(result, e.span));
                        }
                    }
                }
                
                Expr::Binary(e)
            }
            Expr::Unary(mut e) => {
                e.operand = Box::new(Self::constant_folding(*e.operand));
                
                // 如果操作数是常量，尝试计算结果
                if e.operand.is_constant() {
                    if let Expr::Constant(operand) = &*e.operand {
                        if let Some(result) = Self::evaluate_unary_op(e.op, &operand.value) {
                            return Expr::Constant(ConstantExpr::new(result, e.span));
                        }
                    }
                }
                
                Expr::Unary(e)
            }
            Expr::List(mut e) => {
                e.elements = e.elements.into_iter()
                    .map(Self::constant_folding)
                    .collect();
                Expr::List(e)
            }
            Expr::Map(mut e) => {
                e.pairs = e.pairs.into_iter()
                    .map(|(key, value)| (key, Self::constant_folding(value)))
                    .collect();
                Expr::Map(e)
            }
            Expr::Case(mut e) => {
                if let Some(ref mut match_expr) = e.match_expr {
                    *match_expr = Box::new(Self::constant_folding(*match_expr.clone()));
                }
                
                e.when_then_pairs = e.when_then_pairs.into_iter()
                    .map(|(when, then)| {
                        (
                            Box::new(Self::constant_folding(*when)),
                            Box::new(Self::constant_folding(*then))
                        )
                    })
                    .collect();
                
                if let Some(ref mut default) = e.default {
                    *default = Box::new(Self::constant_folding(*default.clone()));
                }
                
                Expr::Case(e)
            }
            Expr::Subscript(mut e) => {
                e.collection = Box::new(Self::constant_folding(*e.collection));
                e.index = Box::new(Self::constant_folding(*e.index));
                Expr::Subscript(e)
            }
            Expr::Predicate(mut e) => {
                e.list = Box::new(Self::constant_folding(*e.list));
                e.condition = Box::new(Self::constant_folding(*e.condition));
                Expr::Predicate(e)
            }
            _ => expr,
        }
    }
    
    /// 评估二元操作符
    fn evaluate_binary_op(left: &Value, op: BinaryOp, right: &Value) -> Option<Value> {
        use crate::core::Value;
        
        match (left, op, right) {
            (Value::Int(l), BinaryOp::Add, Value::Int(r)) => Some(Value::Int(l + r)),
            (Value::Int(l), BinaryOp::Sub, Value::Int(r)) => Some(Value::Int(l - r)),
            (Value::Int(l), BinaryOp::Mul, Value::Int(r)) => Some(Value::Int(l * r)),
            (Value::Int(l), BinaryOp::Div, Value::Int(r)) => {
                if *r != 0 {
                    Some(Value::Int(l / r))
                } else {
                    None
                }
            }
            (Value::Int(l), BinaryOp::Mod, Value::Int(r)) => {
                if *r != 0 {
                    Some(Value::Int(l % r))
                } else {
                    None
                }
            }
            (Value::Float(l), BinaryOp::Add, Value::Float(r)) => Some(Value::Float(l + r)),
            (Value::Float(l), BinaryOp::Sub, Value::Float(r)) => Some(Value::Float(l - r)),
            (Value::Float(l), BinaryOp::Mul, Value::Float(r)) => Some(Value::Float(l * r)),
            (Value::Float(l), BinaryOp::Div, Value::Float(r)) => {
                if *r != 0.0 {
                    Some(Value::Float(l / r))
                } else {
                    None
                }
            }
            (Value::String(l), BinaryOp::Add, Value::String(r)) => Some(Value::String(format!("{}{}", l, r))),
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
    pub fn simplify(expr: Expr) -> Expr {
        let folded = Self::constant_folding(expr);
        Self::remove_redundant_operations(folded)
    }
    
    /// 移除冗余操作
    fn remove_redundant_operations(expr: Expr) -> Expr {
        match expr {
            Expr::Binary(e) => {
                let left = Self::remove_redundant_operations(*e.left);
                let right = Self::remove_redundant_operations(*e.right);
                
                // 简化：x + 0 -> x
                if e.op == BinaryOp::Add {
                    if let Expr::Constant(constant) = &right {
                        if matches!(constant.value, Value::Int(0) | Value::Float(0.0)) {
                            return left;
                        }
                    }
                    if let Expr::Constant(constant) = &left {
                        if matches!(constant.value, Value::Int(0) | Value::Float(0.0)) {
                            return right;
                        }
                    }
                }
                
                // 简化：x * 1 -> x
                if e.op == BinaryOp::Mul {
                    if let Expr::Constant(constant) = &right {
                        if matches!(constant.value, Value::Int(1) | Value::Float(1.0)) {
                            return left;
                        }
                    }
                    if let Expr::Constant(constant) = &left {
                        if matches!(constant.value, Value::Int(1) | Value::Float(1.0)) {
                            return right;
                        }
                    }
                }
                
                // 简化：x * 0 -> 0
                if e.op == BinaryOp::Mul {
                    if let Expr::Constant(constant) = &right {
                        if matches!(constant.value, Value::Int(0) | Value::Float(0.0)) {
                            return right;
                        }
                    }
                    if let Expr::Constant(constant) = &left {
                        if matches!(constant.value, Value::Int(0) | Value::Float(0.0)) {
                            return left;
                        }
                    }
                }
                
                Expr::Binary(BinaryExpr::new(left, e.op, right, e.span))
            }
            Expr::Unary(e) => {
                let operand = Self::remove_redundant_operations(*e.operand);
                
                // 简化：+x -> x
                if e.op == UnaryOp::Plus {
                    return operand;
                }
                
                // 简化：!!x -> x
                if e.op == UnaryOp::Not {
                    if let Expr::Unary(inner) = &operand {
                        if inner.op == UnaryOp::Not {
                            return *inner.operand.clone();
                        }
                    }
                }
                
                Expr::Unary(UnaryExpr::new(e.op, operand, e.span))
            }
            _ => expr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    
    #[test]
    fn test_expr_factory() {
        let span = Span::default();
        
        // 测试常量表达式
        let const_expr = ExprFactory::constant(Value::Int(42), span);
        assert!(matches!(const_expr, Expr::Constant(_)));
        
        // 测试变量表达式
        let var_expr = ExprFactory::variable("x".to_string(), span);
        assert!(matches!(var_expr, Expr::Variable(_)));
        
        // 测试二元表达式
        let left = ExprFactory::constant(Value::Int(5), span);
        let right = ExprFactory::constant(Value::Int(3), span);
        let binary_expr = ExprFactory::binary(left, BinaryOp::Add, right, span);
        assert!(matches!(binary_expr, Expr::Binary(_)));
    }
    
    #[test]
    fn test_constant_folding() {
        let span = Span::default();
        
        // 测试 5 + 3 -> 8
        let left = ExprFactory::constant(Value::Int(5), span);
        let right = ExprFactory::constant(Value::Int(3), span);
        let expr = ExprFactory::binary(left, BinaryOp::Add, right, span);
        
        let optimized = ExprOptimizer::constant_folding(expr);
        assert!(matches!(optimized, Expr::Constant(_)));
        if let Expr::Constant(e) = optimized {
            assert_eq!(e.value, Value::Int(8));
        }
        
        // 测试 -5 -> -5
        let operand = ExprFactory::constant(Value::Int(5), span);
        let expr = ExprFactory::unary(UnaryOp::Minus, operand, span);
        
        let optimized = ExprOptimizer::constant_folding(expr);
        assert!(matches!(optimized, Expr::Constant(_)));
        if let Expr::Constant(e) = optimized {
            assert_eq!(e.value, Value::Int(-5));
        }
    }
    
    #[test]
    fn test_expression_simplification() {
        let span = Span::default();
        
        // 测试 x + 0 -> x
        let x = ExprFactory::variable("x".to_string(), span);
        let zero = ExprFactory::constant(Value::Int(0), span);
        let expr = ExprFactory::binary(x.clone(), BinaryOp::Add, zero, span);
        
        let simplified = ExprOptimizer::simplify(expr);
        assert_eq!(simplified, x);
        
        // 测试 x * 1 -> x
        let x = ExprFactory::variable("x".to_string(), span);
        let one = ExprFactory::constant(Value::Int(1), span);
        let expr = ExprFactory::binary(x.clone(), BinaryOp::Mul, one, span);
        
        let simplified = ExprOptimizer::simplify(expr);
        assert_eq!(simplified, x);
        
        // 测试 !!x -> x
        let x = ExprFactory::variable("x".to_string(), span);
        let not_expr = ExprFactory::unary(UnaryOp::Not, x.clone(), span);
        let expr = ExprFactory::unary(UnaryOp::Not, not_expr, span);
        
        let simplified = ExprOptimizer::simplify(expr);
        assert_eq!(simplified, x);
    }
}