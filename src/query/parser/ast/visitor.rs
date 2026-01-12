//! 简化的访问者模式 (v2)
//!
//! 基于枚举的简化访问者模式，减少样板代码和类型转换复杂性。

use super::expr::*;
use super::pattern::*;
use super::stmt::*;
use super::types::*;

/// 表达式访问者 trait
pub trait ExprVisitor {
    type Result;

    /// 访问表达式 - 主入口点
    fn visit_expr(&mut self, expr: &Expr) -> Self::Result {
        match expr {
            Expr::Constant(e) => self.visit_constant(e),
            Expr::Variable(e) => self.visit_variable(e),
            Expr::Binary(e) => self.visit_binary(e),
            Expr::Unary(e) => self.visit_unary(e),
            Expr::FunctionCall(e) => self.visit_function_call(e),
            Expr::PropertyAccess(e) => self.visit_property_access(e),
            Expr::List(e) => self.visit_list(e),
            Expr::Map(e) => self.visit_map(e),
            Expr::Case(e) => self.visit_case(e),
            Expr::Subscript(e) => self.visit_subscript(e),
            Expr::Predicate(e) => self.visit_predicate(e),
            Expr::TagProperty(e) => self.visit_tag_property(e),
            Expr::EdgeProperty(e) => self.visit_edge_property(e),
            Expr::InputProperty(e) => self.visit_input_property(e),
            Expr::VariableProperty(e) => self.visit_variable_property(e),
            Expr::SourceProperty(e) => self.visit_source_property(e),
            Expr::DestinationProperty(e) => self.visit_destination_property(e),
            Expr::TypeCast(e) => self.visit_type_cast(e),
            Expr::Range(e) => self.visit_range(e),
            Expr::Path(e) => self.visit_path(e),
            Expr::Label(e) => self.visit_label(e),
            Expr::Reduce(e) => self.visit_reduce(e),
            Expr::ListComprehension(e) => self.visit_list_comprehension(e),
        }
    }

    /// 访问常量表达式
    fn visit_constant(&mut self, expr: &ConstantExpr) -> Self::Result;

    /// 访问变量表达式
    fn visit_variable(&mut self, expr: &VariableExpr) -> Self::Result;

    /// 访问二元表达式
    fn visit_binary(&mut self, expr: &BinaryExpr) -> Self::Result;

    /// 访问一元表达式
    fn visit_unary(&mut self, expr: &UnaryExpr) -> Self::Result;

    /// 访问函数调用表达式
    fn visit_function_call(&mut self, expr: &FunctionCallExpr) -> Self::Result;

    /// 访问属性访问表达式
    fn visit_property_access(&mut self, expr: &PropertyAccessExpr) -> Self::Result;

    /// 访问列表表达式
    fn visit_list(&mut self, expr: &ListExpr) -> Self::Result;

    /// 访问映射表达式
    fn visit_map(&mut self, expr: &MapExpr) -> Self::Result;

    /// 访问 CASE 表达式
    fn visit_case(&mut self, expr: &CaseExpr) -> Self::Result;

    /// 访问下标表达式
    fn visit_subscript(&mut self, expr: &SubscriptExpr) -> Self::Result;

    /// 访问谓词表达式
    fn visit_predicate(&mut self, expr: &PredicateExpr) -> Self::Result;

    /// 访问标签属性表达式
    fn visit_tag_property(&mut self, expr: &TagPropertyExpr) -> Self::Result;

    /// 访问边属性表达式
    fn visit_edge_property(&mut self, expr: &EdgePropertyExpr) -> Self::Result;

    /// 访问输入属性表达式
    fn visit_input_property(&mut self, expr: &InputPropertyExpr) -> Self::Result;

    /// 访问变量属性表达式
    fn visit_variable_property(&mut self, expr: &VariablePropertyExpr) -> Self::Result;

    /// 访问源属性表达式
    fn visit_source_property(&mut self, expr: &SourcePropertyExpr) -> Self::Result;

    /// 访问目标属性表达式
    fn visit_destination_property(&mut self, expr: &DestinationPropertyExpr) -> Self::Result;

    /// 访问类型转换表达式
    fn visit_type_cast(&mut self, expr: &TypeCastExpr) -> Self::Result;

    /// 访问范围表达式
    fn visit_range(&mut self, expr: &RangeExpr) -> Self::Result;

    /// 访问路径表达式
    fn visit_path(&mut self, expr: &PathExpr) -> Self::Result;

    /// 访问标签表达式
    fn visit_label(&mut self, expr: &LabelExpr) -> Self::Result;

    /// 访问归约表达式
    fn visit_reduce(&mut self, expr: &ReduceExpr) -> Self::Result;

    /// 访问列表推导表达式
    fn visit_list_comprehension(&mut self, expr: &ListComprehensionExpr) -> Self::Result;
}

/// 语句访问者 trait
pub trait StmtVisitor {
    type Result;

    /// 访问语句 - 主入口点
    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::Result {
        match stmt {
            Stmt::Query(s) => self.visit_query(s),
            Stmt::Create(s) => self.visit_create(s),
            Stmt::Match(s) => self.visit_match(s),
            Stmt::Delete(s) => self.visit_delete(s),
            Stmt::Update(s) => self.visit_update(s),
            Stmt::Go(s) => self.visit_go(s),
            Stmt::Fetch(s) => self.visit_fetch(s),
            Stmt::Use(s) => self.visit_use(s),
            Stmt::Show(s) => self.visit_show(s),
            Stmt::Explain(s) => self.visit_explain(s),
            Stmt::Lookup(s) => self.visit_lookup(s),
            Stmt::Subgraph(s) => self.visit_subgraph(s),
            Stmt::FindPath(s) => self.visit_find_path(s),
        }
    }

    /// 访问查询语句
    fn visit_query(&mut self, stmt: &QueryStmt) -> Self::Result;

    /// 访问 CREATE 语句
    fn visit_create(&mut self, stmt: &CreateStmt) -> Self::Result;

    /// 访问 MATCH 语句
    fn visit_match(&mut self, stmt: &MatchStmt) -> Self::Result;

    /// 访问 DELETE 语句
    fn visit_delete(&mut self, stmt: &DeleteStmt) -> Self::Result;

    /// 访问 UPDATE 语句
    fn visit_update(&mut self, stmt: &UpdateStmt) -> Self::Result;

    /// 访问 GO 语句
    fn visit_go(&mut self, stmt: &GoStmt) -> Self::Result;

    /// 访问 FETCH 语句
    fn visit_fetch(&mut self, stmt: &FetchStmt) -> Self::Result;

    /// 访问 USE 语句
    fn visit_use(&mut self, stmt: &UseStmt) -> Self::Result;

    /// 访问 SHOW 语句
    fn visit_show(&mut self, stmt: &ShowStmt) -> Self::Result;

    /// 访问 EXPLAIN 语句
    fn visit_explain(&mut self, stmt: &ExplainStmt) -> Self::Result;

    /// 访问 LOOKUP 语句
    fn visit_lookup(&mut self, stmt: &LookupStmt) -> Self::Result;

    /// 访问 SUBGRAPH 语句
    fn visit_subgraph(&mut self, stmt: &SubgraphStmt) -> Self::Result;

    /// 访问 FIND PATH 语句
    fn visit_find_path(&mut self, stmt: &FindPathStmt) -> Self::Result;
}

/// 模式访问者 trait
pub trait PatternVisitor {
    type Result;

    /// 访问模式 - 主入口点
    fn visit_pattern(&mut self, pattern: &Pattern) -> Self::Result {
        match pattern {
            Pattern::Node(p) => self.visit_node_pattern(p),
            Pattern::Edge(p) => self.visit_edge_pattern(p),
            Pattern::Path(p) => self.visit_path_pattern(p),
            Pattern::Variable(p) => self.visit_variable_pattern(p),
        }
    }

    /// 访问节点模式
    fn visit_node_pattern(&mut self, pattern: &NodePattern) -> Self::Result;

    /// 访问边模式
    fn visit_edge_pattern(&mut self, pattern: &EdgePattern) -> Self::Result;

    /// 访问路径模式
    fn visit_path_pattern(&mut self, pattern: &PathPattern) -> Self::Result;

    /// 访问变量模式
    fn visit_variable_pattern(&mut self, pattern: &VariablePattern) -> Self::Result;
}

/// 默认访问者实现 - 提供基础遍历功能
pub struct DefaultVisitor;

impl ExprVisitor for DefaultVisitor {
    type Result = ();

    fn visit_constant(&mut self, _expr: &ConstantExpr) -> Self::Result {
        // 常量表达式没有子节点
    }

    fn visit_variable(&mut self, _expr: &VariableExpr) -> Self::Result {
        // 变量表达式没有子节点
    }

    fn visit_binary(&mut self, expr: &BinaryExpr) -> Self::Result {
        // 访问左右操作数
        self.visit_expr(&expr.left);
        self.visit_expr(&expr.right);
    }

    fn visit_unary(&mut self, expr: &UnaryExpr) -> Self::Result {
        // 访问操作数
        self.visit_expr(&expr.operand);
    }

    fn visit_function_call(&mut self, expr: &FunctionCallExpr) -> Self::Result {
        // 访问所有参数
        for arg in &expr.args {
            self.visit_expr(arg);
        }
    }

    fn visit_property_access(&mut self, expr: &PropertyAccessExpr) -> Self::Result {
        // 访问对象表达式
        self.visit_expr(&expr.object);
    }

    fn visit_list(&mut self, expr: &ListExpr) -> Self::Result {
        // 访问所有元素
        for elem in &expr.elements {
            self.visit_expr(elem);
        }
    }

    fn visit_map(&mut self, expr: &MapExpr) -> Self::Result {
        // 访问所有值
        for (_, value) in &expr.pairs {
            self.visit_expr(value);
        }
    }

    fn visit_case(&mut self, expr: &CaseExpr) -> Self::Result {
        // 访问匹配表达式
        if let Some(ref match_expr) = expr.match_expr {
            self.visit_expr(match_expr);
        }

        // 访问所有 WHEN-THEN 对
        for (when, then) in &expr.when_then_pairs {
            self.visit_expr(when);
            self.visit_expr(then);
        }

        // 访问默认表达式
        if let Some(ref default) = expr.default {
            self.visit_expr(default);
        }
    }

    fn visit_subscript(&mut self, expr: &SubscriptExpr) -> Self::Result {
        // 访问集合和索引表达式
        self.visit_expr(&expr.collection);
        self.visit_expr(&expr.index);
    }

    fn visit_predicate(&mut self, expr: &PredicateExpr) -> Self::Result {
        // 访问列表和条件表达式
        self.visit_expr(&expr.list);
        self.visit_expr(&expr.condition);
    }

    fn visit_tag_property(&mut self, _expr: &TagPropertyExpr) -> Self::Result {
        // 标签属性表达式没有子节点
    }

    fn visit_edge_property(&mut self, _expr: &EdgePropertyExpr) -> Self::Result {
        // 边属性表达式没有子节点
    }

    fn visit_input_property(&mut self, _expr: &InputPropertyExpr) -> Self::Result {
        // 输入属性表达式没有子节点
    }

    fn visit_variable_property(&mut self, _expr: &VariablePropertyExpr) -> Self::Result {
        // 变量属性表达式没有子节点
    }

    fn visit_source_property(&mut self, _expr: &SourcePropertyExpr) -> Self::Result {
        // 源属性表达式没有子节点
    }

    fn visit_destination_property(&mut self, _expr: &DestinationPropertyExpr) -> Self::Result {
        // 目标属性表达式没有子节点
    }

    fn visit_type_cast(&mut self, expr: &TypeCastExpr) -> Self::Result {
        // 访问表达式
        self.visit_expr(&expr.expr);
    }

    fn visit_range(&mut self, expr: &RangeExpr) -> Self::Result {
        // 访问集合表达式
        self.visit_expr(&expr.collection);
        // 访问起始和结束表达式
        if let Some(ref start) = expr.start {
            self.visit_expr(start);
        }
        if let Some(ref end) = expr.end {
            self.visit_expr(end);
        }
    }

    fn visit_path(&mut self, expr: &PathExpr) -> Self::Result {
        // 访问所有路径元素
        for elem in &expr.elements {
            self.visit_expr(elem);
        }
    }

    fn visit_label(&mut self, _expr: &LabelExpr) -> Self::Result {
        // 标签表达式没有子节点
    }

    fn visit_reduce(&mut self, expr: &ReduceExpr) -> Self::Result {
        // 访问列表表达式
        self.visit_expr(&expr.list);
        // 访问初始表达式
        self.visit_expr(&expr.initial);
        // 访问归约表达式
        self.visit_expr(&expr.expr);
    }

    fn visit_list_comprehension(&mut self, expr: &ListComprehensionExpr) -> Self::Result {
        // 访问生成器表达式
        self.visit_expr(&expr.generator);
        // 访问条件表达式
        if let Some(ref condition) = expr.condition {
            self.visit_expr(condition);
        }
    }
}

impl StmtVisitor for DefaultVisitor {
    type Result = ();

    fn visit_query(&mut self, stmt: &QueryStmt) -> Self::Result {
        // 访问所有语句
        for statement in &stmt.statements {
            self.visit_stmt(statement);
        }
    }

    fn visit_create(&mut self, stmt: &CreateStmt) -> Self::Result {
        // 根据创建目标访问相关表达式
        match &stmt.target {
            CreateTarget::Node { properties, .. } => {
                if let Some(props) = properties {
                    self.visit_expr(props);
                }
            }
            CreateTarget::Edge {
                src,
                dst,
                properties,
                ..
            } => {
                self.visit_expr(src);
                self.visit_expr(dst);
                if let Some(props) = properties {
                    self.visit_expr(props);
                }
            }
            _ => {}
        }
    }

    fn visit_match(&mut self, stmt: &MatchStmt) -> Self::Result {
        // 访问所有模式
        for pattern in &stmt.patterns {
            self.visit_pattern(pattern);
        }

        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expr(where_clause);
        }

        // 访问 RETURN 子句
        if let Some(ref return_clause) = stmt.return_clause {
            for item in &return_clause.items {
                match item {
                    ReturnItem::Expression { expr, .. } => {
                        self.visit_expr(expr);
                    }
                    _ => {}
                }
            }
        }
    }

    fn visit_delete(&mut self, stmt: &DeleteStmt) -> Self::Result {
        // 根据删除目标访问相关表达式
        match &stmt.target {
            DeleteTarget::Vertices(vertices) => {
                for vertex in vertices {
                    self.visit_expr(vertex);
                }
            }
            DeleteTarget::Edges { src, dst, rank, .. } => {
                self.visit_expr(src);
                self.visit_expr(dst);
                if let Some(ref rank) = rank {
                    self.visit_expr(rank);
                }
            }
            _ => {}
        }

        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expr(where_clause);
        }
    }

    fn visit_update(&mut self, stmt: &UpdateStmt) -> Self::Result {
        // 根据更新目标访问相关表达式
        match &stmt.target {
            UpdateTarget::Vertex(vertex) => {
                self.visit_expr(vertex);
            }
            UpdateTarget::Edge { src, dst, rank, .. } => {
                self.visit_expr(src);
                self.visit_expr(dst);
                if let Some(ref rank) = rank {
                    self.visit_expr(rank);
                }
            }
            _ => {}
        }

        // 访问 SET 子句中的表达式
        for assignment in &stmt.set_clause.assignments {
            self.visit_expr(&assignment.value);
        }

        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expr(where_clause);
        }
    }

    fn visit_go(&mut self, stmt: &GoStmt) -> Self::Result {
        // 访问 FROM 子句中的顶点
        for vertex in &stmt.from.vertices {
            self.visit_expr(vertex);
        }

        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expr(where_clause);
        }

        // 访问 YIELD 子句
        if let Some(ref yield_clause) = stmt.yield_clause {
            for item in &yield_clause.items {
                self.visit_expr(&item.expr);
            }
        }
    }

    fn visit_fetch(&mut self, stmt: &FetchStmt) -> Self::Result {
        // 根据获取目标访问相关表达式
        match &stmt.target {
            FetchTarget::Vertices { ids, .. } => {
                for id in ids {
                    self.visit_expr(id);
                }
            }
            FetchTarget::Edges { src, dst, rank, .. } => {
                self.visit_expr(src);
                self.visit_expr(dst);
                if let Some(ref rank) = rank {
                    self.visit_expr(rank);
                }
            }
        }
    }

    fn visit_use(&mut self, _stmt: &UseStmt) -> Self::Result {
        // USE 语句没有子表达式
    }

    fn visit_show(&mut self, _stmt: &ShowStmt) -> Self::Result {
        // SHOW 语句没有子表达式
    }

    fn visit_explain(&mut self, stmt: &ExplainStmt) -> Self::Result {
        // 访问被解释的语句
        self.visit_stmt(&stmt.statement);
    }

    fn visit_lookup(&mut self, stmt: &LookupStmt) -> Self::Result {
        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expr(where_clause);
        }

        // 访问 YIELD 子句
        if let Some(ref yield_clause) = stmt.yield_clause {
            for item in &yield_clause.items {
                self.visit_expr(&item.expr);
            }
        }
    }

    fn visit_subgraph(&mut self, stmt: &SubgraphStmt) -> Self::Result {
        // 访问 FROM 子句中的顶点
        for vertex in &stmt.from.vertices {
            self.visit_expr(vertex);
        }

        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expr(where_clause);
        }

        // 访问 YIELD 子句
        if let Some(ref yield_clause) = stmt.yield_clause {
            for item in &yield_clause.items {
                self.visit_expr(&item.expr);
            }
        }
    }

    fn visit_find_path(&mut self, stmt: &FindPathStmt) -> Self::Result {
        // 访问 FROM 子句中的顶点
        for vertex in &stmt.from.vertices {
            self.visit_expr(vertex);
        }

        // 访问目标表达式
        self.visit_expr(&stmt.to);

        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expr(where_clause);
        }

        // 访问 YIELD 子句
        if let Some(ref yield_clause) = stmt.yield_clause {
            for item in &yield_clause.items {
                self.visit_expr(&item.expr);
            }
        }
    }
}

impl PatternVisitor for DefaultVisitor {
    type Result = ();

    fn visit_node_pattern(&mut self, pattern: &NodePattern) -> Self::Result {
        // 访问属性表达式
        if let Some(ref props) = pattern.properties {
            self.visit_expr(props);
        }

        // 访问谓词表达式
        for predicate in &pattern.predicates {
            self.visit_expr(predicate);
        }
    }

    fn visit_edge_pattern(&mut self, pattern: &EdgePattern) -> Self::Result {
        // 访问属性表达式
        if let Some(ref props) = pattern.properties {
            self.visit_expr(props);
        }

        // 访问谓词表达式
        for predicate in &pattern.predicates {
            self.visit_expr(predicate);
        }
    }

    fn visit_path_pattern(&mut self, pattern: &PathPattern) -> Self::Result {
        // 访问所有路径元素
        for element in &pattern.elements {
            self.visit_path_element(element);
        }
    }

    fn visit_variable_pattern(&mut self, _pattern: &VariablePattern) -> Self::Result {
        // 变量模式没有子表达式
    }
}

impl DefaultVisitor {
    /// 访问路径元素（辅助方法）
    fn visit_path_element(&mut self, element: &PathElement) {
        match element {
            PathElement::Node(node) => self.visit_node_pattern(node),
            PathElement::Edge(edge) => self.visit_edge_pattern(edge),
            PathElement::Alternative(patterns) => {
                for p in patterns {
                    self.visit_pattern(p);
                }
            }
            PathElement::Optional(elem) => self.visit_path_element(elem),
            PathElement::Repeated(elem, _) => self.visit_path_element(elem),
        }
    }
}

/// 类型检查访问者
pub struct TypeChecker {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl ExprVisitor for TypeChecker {
    type Result = ();

    fn visit_constant(&mut self, _expr: &ConstantExpr) -> Self::Result {
        // 常量表达式总是类型安全的
    }

    fn visit_variable(&mut self, _expr: &VariableExpr) -> Self::Result {
        // 变量表达式需要符号表检查（TODO）
    }

    fn visit_binary(&mut self, expr: &BinaryExpr) -> Self::Result {
        // 检查二元表达式的类型兼容性
        match expr.op {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo => {
                // 算术操作符需要数值类型
                // TODO: 实现类型检查逻辑
                self.warnings.push(format!(
                    "Arithmetic operation {} should have numeric operands",
                    expr.op
                ));
            }
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                // 逻辑操作符需要布尔类型
                self.warnings.push(format!(
                    "Logical operation {} should have boolean operands",
                    expr.op
                ));
            }
            _ => {}
        }

        // 递归检查子表达式
        self.visit_expr(&expr.left);
        self.visit_expr(&expr.right);
    }

    fn visit_function_call(&mut self, expr: &FunctionCallExpr) -> Self::Result {
        // 检查函数参数数量
        match expr.name.as_str() {
            "COUNT" => {
                if expr.args.len() > 1 && !expr.distinct {
                    self.errors
                        .push("COUNT function takes at most one argument".to_string());
                }
            }
            "SUM" | "AVG" | "MIN" | "MAX" => {
                if expr.args.len() != 1 {
                    self.errors
                        .push(format!("{} function takes exactly one argument", expr.name));
                }
            }
            _ => {}
        }

        // 递归检查所有参数
        for arg in &expr.args {
            self.visit_expr(arg);
        }
    }

    fn visit_unary(&mut self, expr: &UnaryExpr) -> Self::Result {
        // 递归检查操作数
        self.visit_expr(&expr.operand);
    }

    fn visit_property_access(&mut self, expr: &PropertyAccessExpr) -> Self::Result {
        // 递归检查对象表达式
        self.visit_expr(&expr.object);
    }

    fn visit_list(&mut self, expr: &ListExpr) -> Self::Result {
        // 递归检查所有元素
        for elem in &expr.elements {
            self.visit_expr(elem);
        }
    }

    fn visit_map(&mut self, expr: &MapExpr) -> Self::Result {
        // 递归检查所有值
        for (_, value) in &expr.pairs {
            self.visit_expr(value);
        }
    }

    fn visit_case(&mut self, expr: &CaseExpr) -> Self::Result {
        // 递归检查所有子表达式
        if let Some(ref match_expr) = expr.match_expr {
            self.visit_expr(match_expr);
        }

        for (when, then) in &expr.when_then_pairs {
            self.visit_expr(when);
            self.visit_expr(then);
        }

        if let Some(ref default) = expr.default {
            self.visit_expr(default);
        }
    }

    fn visit_subscript(&mut self, expr: &SubscriptExpr) -> Self::Result {
        // 递归检查集合和索引表达式
        self.visit_expr(&expr.collection);
        self.visit_expr(&expr.index);
    }

    fn visit_predicate(&mut self, expr: &PredicateExpr) -> Self::Result {
        // 递归检查列表和条件表达式
        self.visit_expr(&expr.list);
        self.visit_expr(&expr.condition);
    }

    fn visit_tag_property(&mut self, _expr: &TagPropertyExpr) -> Self::Result {
        // 标签属性表达式总是类型安全的
    }

    fn visit_edge_property(&mut self, _expr: &EdgePropertyExpr) -> Self::Result {
        // 边属性表达式总是类型安全的
    }

    fn visit_input_property(&mut self, _expr: &InputPropertyExpr) -> Self::Result {
        // 输入属性表达式总是类型安全的
    }

    fn visit_variable_property(&mut self, _expr: &VariablePropertyExpr) -> Self::Result {
        // 变量属性表达式总是类型安全的
    }

    fn visit_source_property(&mut self, _expr: &SourcePropertyExpr) -> Self::Result {
        // 源属性表达式总是类型安全的
    }

    fn visit_destination_property(&mut self, _expr: &DestinationPropertyExpr) -> Self::Result {
        // 目标属性表达式总是类型安全的
    }

    fn visit_type_cast(&mut self, expr: &TypeCastExpr) -> Self::Result {
        // 递归检查表达式
        self.visit_expr(&expr.expr);
    }

    fn visit_range(&mut self, expr: &RangeExpr) -> Self::Result {
        // 递归检查集合表达式
        self.visit_expr(&expr.collection);
        // 递归检查起始和结束表达式
        if let Some(ref start) = expr.start {
            self.visit_expr(start);
        }
        if let Some(ref end) = expr.end {
            self.visit_expr(end);
        }
    }

    fn visit_path(&mut self, expr: &PathExpr) -> Self::Result {
        // 递归检查所有路径元素
        for elem in &expr.elements {
            self.visit_expr(elem);
        }
    }

    fn visit_label(&mut self, _expr: &LabelExpr) -> Self::Result {
        // 标签表达式总是类型安全的
    }

    fn visit_reduce(&mut self, expr: &ReduceExpr) -> Self::Result {
        // 递归检查列表表达式
        self.visit_expr(&expr.list);
        // 递归检查初始表达式
        self.visit_expr(&expr.initial);
        // 递归检查归约表达式
        self.visit_expr(&expr.expr);
    }

    fn visit_list_comprehension(&mut self, expr: &ListComprehensionExpr) -> Self::Result {
        // 递归检查生成器表达式
        self.visit_expr(&expr.generator);
        // 递归检查条件表达式
        if let Some(ref condition) = expr.condition {
            self.visit_expr(condition);
        }
    }
}

/// AST 格式化器 - 用于生成格式化的 AST 字符串表示
pub struct AstFormatter {
    pub indent: usize,
    pub result: String,
}

impl AstFormatter {
    pub fn new() -> Self {
        Self {
            indent: 0,
            result: String::new(),
        }
    }

    pub fn format(&mut self, expr: &Expr) -> String {
        self.result.clear();
        self.indent = 0;
        self.visit_expr(expr);
        self.result.clone()
    }

    fn indent_str(&self) -> String {
        "  ".repeat(self.indent)
    }

    fn write_line(&mut self, content: &str) {
        self.result.push_str(&self.indent_str());
        self.result.push_str(content);
        self.result.push('\n');
    }

    fn increase_indent(&mut self) {
        self.indent += 1;
    }

    fn decrease_indent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }
}

impl ExprVisitor for AstFormatter {
    type Result = ();

    fn visit_constant(&mut self, expr: &ConstantExpr) -> Self::Result {
        self.write_line(&format!("Constant: {:?}", expr.value));
    }

    fn visit_variable(&mut self, expr: &VariableExpr) -> Self::Result {
        self.write_line(&format!("Variable: {}", expr.name));
    }

    fn visit_binary(&mut self, expr: &BinaryExpr) -> Self::Result {
        self.write_line(&format!("Binary: {}", expr.op));
        self.increase_indent();
        self.write_line("Left:");
        self.increase_indent();
        self.visit_expr(&expr.left);
        self.decrease_indent();
        self.write_line("Right:");
        self.increase_indent();
        self.visit_expr(&expr.right);
        self.decrease_indent();
        self.decrease_indent();
    }

    fn visit_function_call(&mut self, expr: &FunctionCallExpr) -> Self::Result {
        self.write_line(&format!(
            "FunctionCall: {} ({} args)",
            expr.name,
            expr.args.len()
        ));
        if !expr.args.is_empty() {
            self.increase_indent();
            for (i, arg) in expr.args.iter().enumerate() {
                self.write_line(&format!("Arg {}:", i));
                self.increase_indent();
                self.visit_expr(arg);
                self.decrease_indent();
            }
            self.decrease_indent();
        }
    }

    fn visit_unary(&mut self, expr: &UnaryExpr) -> Self::Result {
        self.write_line(&format!("Unary: {}", expr.op));
        self.increase_indent();
        self.visit_expr(&expr.operand);
        self.decrease_indent();
    }

    fn visit_property_access(&mut self, expr: &PropertyAccessExpr) -> Self::Result {
        self.write_line(&format!("PropertyAccess: {}", expr.property));
        self.increase_indent();
        self.visit_expr(&expr.object);
        self.decrease_indent();
    }

    fn visit_list(&mut self, expr: &ListExpr) -> Self::Result {
        self.write_line(&format!("List: {} elements", expr.elements.len()));
        if !expr.elements.is_empty() {
            self.increase_indent();
            for (i, elem) in expr.elements.iter().enumerate() {
                self.write_line(&format!("Element {}:", i));
                self.increase_indent();
                self.visit_expr(elem);
                self.decrease_indent();
            }
            self.decrease_indent();
        }
    }

    fn visit_map(&mut self, expr: &MapExpr) -> Self::Result {
        self.write_line(&format!("Map: {} pairs", expr.pairs.len()));
        if !expr.pairs.is_empty() {
            self.increase_indent();
            for (key, value) in &expr.pairs {
                self.write_line(&format!("Key: {}", key));
                self.increase_indent();
                self.visit_expr(value);
                self.decrease_indent();
            }
            self.decrease_indent();
        }
    }

    fn visit_case(&mut self, expr: &CaseExpr) -> Self::Result {
        self.write_line("Case:");
        self.increase_indent();

        if let Some(ref match_expr) = expr.match_expr {
            self.write_line("Match:");
            self.increase_indent();
            self.visit_expr(match_expr);
            self.decrease_indent();
        }

        for (i, (when, then)) in expr.when_then_pairs.iter().enumerate() {
            self.write_line(&format!("When-Then {}:", i));
            self.increase_indent();
            self.write_line("When:");
            self.increase_indent();
            self.visit_expr(when);
            self.decrease_indent();
            self.write_line("Then:");
            self.increase_indent();
            self.visit_expr(then);
            self.decrease_indent();
            self.decrease_indent();
        }

        if let Some(ref default) = expr.default {
            self.write_line("Default:");
            self.increase_indent();
            self.visit_expr(default);
            self.decrease_indent();
        }

        self.decrease_indent();
    }

    fn visit_subscript(&mut self, expr: &SubscriptExpr) -> Self::Result {
        self.write_line("Subscript:");
        self.increase_indent();
        self.write_line("Collection:");
        self.increase_indent();
        self.visit_expr(&expr.collection);
        self.decrease_indent();
        self.write_line("Index:");
        self.increase_indent();
        self.visit_expr(&expr.index);
        self.decrease_indent();
        self.decrease_indent();
    }

    fn visit_predicate(&mut self, expr: &PredicateExpr) -> Self::Result {
        self.write_line(&format!("Predicate: {}", expr.predicate));
        self.increase_indent();
        self.write_line("List:");
        self.increase_indent();
        self.visit_expr(&expr.list);
        self.decrease_indent();
        self.write_line("Condition:");
        self.increase_indent();
        self.visit_expr(&expr.condition);
        self.decrease_indent();
        self.decrease_indent();
    }

    fn visit_tag_property(&mut self, expr: &TagPropertyExpr) -> Self::Result {
        self.write_line(&format!("TagProperty: {}.{}", expr.tag, expr.prop));
    }

    fn visit_edge_property(&mut self, expr: &EdgePropertyExpr) -> Self::Result {
        self.write_line(&format!("EdgeProperty: {}.{}", expr.edge, expr.prop));
    }

    fn visit_input_property(&mut self, expr: &InputPropertyExpr) -> Self::Result {
        self.write_line(&format!("InputProperty: $-.{}", expr.prop));
    }

    fn visit_variable_property(&mut self, expr: &VariablePropertyExpr) -> Self::Result {
        self.write_line(&format!("VariableProperty: ${}.{}", expr.var, expr.prop));
    }

    fn visit_source_property(&mut self, expr: &SourcePropertyExpr) -> Self::Result {
        self.write_line(&format!("SourceProperty: $^.{}.{}", expr.tag, expr.prop));
    }

    fn visit_destination_property(&mut self, expr: &DestinationPropertyExpr) -> Self::Result {
        self.write_line(&format!(
            "DestinationProperty: $$.{}.{}",
            expr.tag, expr.prop
        ));
    }

    fn visit_type_cast(&mut self, expr: &TypeCastExpr) -> Self::Result {
        self.write_line(&format!("TypeCast: {:?}", expr.target_type));
        self.increase_indent();
        self.visit_expr(&expr.expr);
        self.decrease_indent();
    }

    fn visit_range(&mut self, expr: &RangeExpr) -> Self::Result {
        self.write_line("Range:");
        self.increase_indent();
        self.write_line("Collection:");
        self.increase_indent();
        self.visit_expr(&expr.collection);
        self.decrease_indent();
        if let Some(ref start) = expr.start {
            self.write_line("Start:");
            self.increase_indent();
            self.visit_expr(start);
            self.decrease_indent();
        }
        if let Some(ref end) = expr.end {
            self.write_line("End:");
            self.increase_indent();
            self.visit_expr(end);
            self.decrease_indent();
        }
        self.decrease_indent();
    }

    fn visit_path(&mut self, expr: &PathExpr) -> Self::Result {
        self.write_line(&format!("Path: {} elements", expr.elements.len()));
        if !expr.elements.is_empty() {
            self.increase_indent();
            for (i, elem) in expr.elements.iter().enumerate() {
                self.write_line(&format!("Element {}:", i));
                self.increase_indent();
                self.visit_expr(elem);
                self.decrease_indent();
            }
            self.decrease_indent();
        }
    }

    fn visit_label(&mut self, expr: &LabelExpr) -> Self::Result {
        self.write_line(&format!("Label: {}", expr.label));
    }

    fn visit_reduce(&mut self, expr: &ReduceExpr) -> Self::Result {
        self.write_line(&format!("Reduce: var = {}", expr.var));
        self.increase_indent();
        self.write_line("List:");
        self.increase_indent();
        self.visit_expr(&expr.list);
        self.decrease_indent();
        self.write_line("Initial:");
        self.increase_indent();
        self.visit_expr(&expr.initial);
        self.decrease_indent();
        self.write_line("Expression:");
        self.increase_indent();
        self.visit_expr(&expr.expr);
        self.decrease_indent();
        self.decrease_indent();
    }

    fn visit_list_comprehension(&mut self, expr: &ListComprehensionExpr) -> Self::Result {
        self.write_line("ListComprehension:");
        self.increase_indent();
        self.write_line("Generator:");
        self.increase_indent();
        self.visit_expr(&expr.generator);
        self.decrease_indent();
        if let Some(ref condition) = expr.condition {
            self.write_line("Condition:");
            self.increase_indent();
            self.visit_expr(condition);
            self.decrease_indent();
        }
        self.decrease_indent();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_default_visitor() {
        let mut visitor = DefaultVisitor;
        let expr = Expr::Constant(ConstantExpr::new(Value::Int(42), Span::default()));

        // 应该能够访问而不出错
        visitor.visit_expr(&expr);
    }

    #[test]
    fn test_type_checker() {
        let mut checker = TypeChecker::new();
        let left = Expr::Constant(ConstantExpr::new(Value::Int(5), Span::default()));
        let right = Expr::Constant(ConstantExpr::new(
            Value::String("hello".to_string()),
            Span::default(),
        ));
        let expr = Expr::Binary(BinaryExpr::new(left, BinaryOp::Add, right, Span::default()));

        checker.visit_expr(&expr);
        assert!(checker.has_warnings());
    }

    #[test]
    fn test_ast_formatter() {
        let mut formatter = AstFormatter::new();
        let expr = Expr::Constant(ConstantExpr::new(Value::Int(42), Span::default()));

        let result = formatter.format(&expr);
        assert!(result.contains("Constant: Int(42)"));
    }
}
