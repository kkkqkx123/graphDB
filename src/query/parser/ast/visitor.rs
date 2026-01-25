//! 简化的访问者模式 (v2)
//!
//! 基于枚举的简化访问者模式，减少样板代码和类型转换复杂性。

use super::expression::*;
use super::pattern::*;
use super::stmt::*;
use super::types::*;

/// 表达式访问者 trait
pub trait ExprVisitor {
    type Result;

    /// 访问表达式 - 主入口点
    fn visit_expression(&mut self, expression: &Expression) -> Self::Result {
        match expression {
            Expression::Constant(e) => self.visit_constant(e),
            Expression::Variable(e) => self.visit_variable(e),
            Expression::Binary(e) => self.visit_binary(e),
            Expression::Unary(e) => self.visit_unary(e),
            Expression::FunctionCall(e) => self.visit_function_call(e),
            Expression::PropertyAccess(e) => self.visit_property_access(e),
            Expression::List(e) => self.visit_list(e),
            Expression::Map(e) => self.visit_map(e),
            Expression::Case(e) => self.visit_case(e),
            Expression::Subscript(e) => self.visit_subscript(e),
            Expression::TypeCast(e) => self.visit_type_cast(e),
            Expression::Range(e) => self.visit_range(e),
            Expression::Path(e) => self.visit_path(e),
            Expression::Label(e) => self.visit_label(e),
        }
    }

    /// 访问常量表达式
    fn visit_constant(&mut self, expression: &ConstantExpression) -> Self::Result;

    /// 访问变量表达式
    fn visit_variable(&mut self, expression: &VariableExpression) -> Self::Result;

    /// 访问二元表达式
    fn visit_binary(&mut self, expression: &BinaryExpression) -> Self::Result;

    /// 访问一元表达式
    fn visit_unary(&mut self, expression: &UnaryExpression) -> Self::Result;

    /// 访问函数调用表达式
    fn visit_function_call(&mut self, expression: &FunctionCallExpression) -> Self::Result;

    /// 访问属性访问表达式
    fn visit_property_access(&mut self, expression: &PropertyAccessExpression) -> Self::Result;

    /// 访问列表表达式
    fn visit_list(&mut self, expression: &ListExpression) -> Self::Result;

    /// 访问映射表达式
    fn visit_map(&mut self, expression: &MapExpression) -> Self::Result;

    /// 访问 CASE 表达式
    fn visit_case(&mut self, expression: &CaseExpression) -> Self::Result;

    /// 访问下标表达式
    fn visit_subscript(&mut self, expression: &SubscriptExpression) -> Self::Result;

    /// 访问类型转换表达式
    fn visit_type_cast(&mut self, expression: &TypeCastExpression) -> Self::Result;

    /// 访问范围表达式
    fn visit_range(&mut self, expression: &RangeExpression) -> Self::Result;

    /// 访问路径表达式
    fn visit_path(&mut self, expression: &PathExpression) -> Self::Result;

    /// 访问标签表达式
    fn visit_label(&mut self, expression: &LabelExpression) -> Self::Result;
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
            Stmt::Insert(s) => self.visit_insert(s),
            Stmt::Merge(s) => self.visit_merge(s),
            Stmt::Unwind(s) => self.visit_unwind(s),
            Stmt::Return(s) => self.visit_return(s),
            Stmt::With(s) => self.visit_with(s),
            Stmt::Set(s) => self.visit_set(s),
            Stmt::Remove(s) => self.visit_remove(s),
            Stmt::Pipe(s) => self.visit_pipe(s),
            Stmt::Drop(s) => self.visit_drop(s),
            Stmt::Desc(s) => self.visit_desc(s),
            Stmt::Alter(s) => self.visit_alter(s),
            Stmt::ChangePassword(s) => self.visit_change_password(s),
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

    /// 访问 INSERT 语句
    fn visit_insert(&mut self, stmt: &InsertStmt) -> Self::Result;

    /// 访问 MERGE 语句
    fn visit_merge(&mut self, stmt: &MergeStmt) -> Self::Result;

    /// 访问 UNWIND 语句
    fn visit_unwind(&mut self, stmt: &UnwindStmt) -> Self::Result;

    /// 访问 RETURN 语句
    fn visit_return(&mut self, stmt: &ReturnStmt) -> Self::Result;

    /// 访问 WITH 语句
    fn visit_with(&mut self, stmt: &WithStmt) -> Self::Result;

    /// 访问 SET 语句
    fn visit_set(&mut self, stmt: &SetStmt) -> Self::Result;

    /// 访问 REMOVE 语句
    fn visit_remove(&mut self, stmt: &RemoveStmt) -> Self::Result;

    /// 访问 PIPE 语句
    fn visit_pipe(&mut self, stmt: &PipeStmt) -> Self::Result;

    /// 访问 DROP 语句
    fn visit_drop(&mut self, stmt: &DropStmt) -> Self::Result;

    /// 访问 DESCRIBE 语句
    fn visit_desc(&mut self, stmt: &DescStmt) -> Self::Result;

    /// 访问 ALTER 语句
    fn visit_alter(&mut self, stmt: &AlterStmt) -> Self::Result;

    /// 访问 CHANGE PASSWORD 语句
    fn visit_change_password(&mut self, stmt: &ChangePasswordStmt) -> Self::Result;
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

    fn visit_constant(&mut self, _expression: &ConstantExpression) -> Self::Result {
        // 常量表达式没有子节点
    }

    fn visit_variable(&mut self, _expression: &VariableExpression) -> Self::Result {
        // 变量表达式没有子节点
    }

    fn visit_binary(&mut self, expression: &BinaryExpression) -> Self::Result {
        // 访问左右操作数
        self.visit_expression(&expression.left);
        self.visit_expression(&expression.right);
    }

    fn visit_unary(&mut self, expression: &UnaryExpression) -> Self::Result {
        // 访问操作数
        self.visit_expression(&expression.operand);
    }

    fn visit_function_call(&mut self, expression: &FunctionCallExpression) -> Self::Result {
        // 访问所有参数
        for arg in &expression.args {
            self.visit_expression(arg);
        }
    }

    fn visit_property_access(&mut self, expression: &PropertyAccessExpression) -> Self::Result {
        // 访问对象表达式
        self.visit_expression(&expression.object);
    }

    fn visit_list(&mut self, expression: &ListExpression) -> Self::Result {
        // 访问所有元素
        for elem in &expression.elements {
            self.visit_expression(elem);
        }
    }

    fn visit_map(&mut self, expression: &MapExpression) -> Self::Result {
        // 访问所有值
        for (_, value) in &expression.pairs {
            self.visit_expression(value);
        }
    }

    fn visit_case(&mut self, expression: &CaseExpression) -> Self::Result {
        // 访问匹配表达式
        if let Some(ref match_expression) = expression.match_expression {
            self.visit_expression(match_expression);
        }

        // 访问所有 WHEN-THEN 对
        for (when, then) in &expression.when_then_pairs {
            self.visit_expression(when);
            self.visit_expression(then);
        }

        // 访问默认表达式
        if let Some(ref default) = expression.default {
            self.visit_expression(default);
        }
    }

    fn visit_subscript(&mut self, expression: &SubscriptExpression) -> Self::Result {
        // 访问集合和索引表达式
        self.visit_expression(&expression.collection);
        self.visit_expression(&expression.index);
    }

    fn visit_type_cast(&mut self, expression: &TypeCastExpression) -> Self::Result {
        // 访问表达式
        self.visit_expression(&expression.expression);
    }

    fn visit_range(&mut self, expression: &RangeExpression) -> Self::Result {
        // 访问集合表达式
        self.visit_expression(&expression.collection);
        // 访问起始和结束表达式
        if let Some(ref start) = expression.start {
            self.visit_expression(start);
        }
        if let Some(ref end) = expression.end {
            self.visit_expression(end);
        }
    }

    fn visit_path(&mut self, expression: &PathExpression) -> Self::Result {
        // 访问所有路径元素
        for elem in &expression.elements {
            self.visit_expression(elem);
        }
    }

    fn visit_label(&mut self, _expression: &LabelExpression) -> Self::Result {
        // 标签表达式没有子节点
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
                    self.visit_expression(props);
                }
            }
            CreateTarget::Edge {
                src,
                dst,
                properties,
                ..
            } => {
                self.visit_expression(src);
                self.visit_expression(dst);
                if let Some(props) = properties {
                    self.visit_expression(props);
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
            self.visit_expression(where_clause);
        }

        // 访问 RETURN 子句
        if let Some(ref return_clause) = stmt.return_clause {
            for item in &return_clause.items {
                match item {
                    ReturnItem::Expression { expression, .. } => {
                        self.visit_expression(expression);
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
                    self.visit_expression(vertex);
                }
            }
            DeleteTarget::Edges { src, dst, rank, .. } => {
                self.visit_expression(src);
                self.visit_expression(dst);
                if let Some(ref rank) = rank {
                    self.visit_expression(rank);
                }
            }
            _ => {}
        }

        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expression(where_clause);
        }
    }

    fn visit_update(&mut self, stmt: &UpdateStmt) -> Self::Result {
        // 根据更新目标访问相关表达式
        match &stmt.target {
            UpdateTarget::Vertex(vertex) => {
                self.visit_expression(vertex);
            }
            UpdateTarget::Edge { src, dst, rank, .. } => {
                self.visit_expression(src);
                self.visit_expression(dst);
                if let Some(ref rank) = rank {
                    self.visit_expression(rank);
                }
            }
            _ => {}
        }

        // 访问 SET 子句中的表达式
        for assignment in &stmt.set_clause.assignments {
            self.visit_expression(&assignment.value);
        }

        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expression(where_clause);
        }
    }

    fn visit_go(&mut self, stmt: &GoStmt) -> Self::Result {
        // 访问 FROM 子句中的顶点
        for vertex in &stmt.from.vertices {
            self.visit_expression(vertex);
        }

        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expression(where_clause);
        }

        // 访问 YIELD 子句
        if let Some(ref yield_clause) = stmt.yield_clause {
            for item in &yield_clause.items {
                self.visit_expression(&item.expression);
            }
        }
    }

    fn visit_fetch(&mut self, stmt: &FetchStmt) -> Self::Result {
        // 根据获取目标访问相关表达式
        match &stmt.target {
            FetchTarget::Vertices { ids, .. } => {
                for id in ids {
                    self.visit_expression(id);
                }
            }
            FetchTarget::Edges { src, dst, rank, .. } => {
                self.visit_expression(src);
                self.visit_expression(dst);
                if let Some(ref rank) = rank {
                    self.visit_expression(rank);
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
            self.visit_expression(where_clause);
        }

        // 访问 YIELD 子句
        if let Some(ref yield_clause) = stmt.yield_clause {
            for item in &yield_clause.items {
                self.visit_expression(&item.expression);
            }
        }
    }

    fn visit_subgraph(&mut self, stmt: &SubgraphStmt) -> Self::Result {
        // 访问 FROM 子句中的顶点
        for vertex in &stmt.from.vertices {
            self.visit_expression(vertex);
        }

        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expression(where_clause);
        }

        // 访问 YIELD 子句
        if let Some(ref yield_clause) = stmt.yield_clause {
            for item in &yield_clause.items {
                self.visit_expression(&item.expression);
            }
        }
    }

    fn visit_find_path(&mut self, stmt: &FindPathStmt) -> Self::Result {
        // 访问 FROM 子句中的顶点
        for vertex in &stmt.from.vertices {
            self.visit_expression(vertex);
        }

        // 访问目标表达式
        self.visit_expression(&stmt.to);

        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expression(where_clause);
        }

        // 访问 YIELD 子句
        if let Some(ref yield_clause) = stmt.yield_clause {
            for item in &yield_clause.items {
                self.visit_expression(&item.expression);
            }
        }
    }

    fn visit_insert(&mut self, stmt: &InsertStmt) -> Self::Result {
        match &stmt.target {
            InsertTarget::Vertices { tag_name: _, prop_names: _, values } => {
                for (_, prop_values) in values {
                    for prop_value in prop_values {
                        self.visit_expression(prop_value);
                    }
                }
            }
            InsertTarget::Edge { edge_name: _, prop_names: _, src, dst, rank: _, values } => {
                self.visit_expression(src);
                self.visit_expression(dst);
                for value in values {
                    self.visit_expression(value);
                }
            }
        }
    }

    fn visit_merge(&mut self, stmt: &MergeStmt) -> Self::Result {
        // 访问模式
        self.visit_pattern(&stmt.pattern);
    }

    fn visit_unwind(&mut self, stmt: &UnwindStmt) -> Self::Result {
        // 访问表达式
        self.visit_expression(&stmt.expression);
    }

    fn visit_return(&mut self, stmt: &ReturnStmt) -> Self::Result {
        // 访问所有返回项
        for item in &stmt.items {
            match item {
                ReturnItem::Expression { expression, .. } => {
                    self.visit_expression(expression);
                }
                _ => {}
            }
        }
    }

    fn visit_with(&mut self, stmt: &WithStmt) -> Self::Result {
        // 访问所有 WITH 项
        for item in &stmt.items {
            match item {
                ReturnItem::Expression { expression, .. } => {
                    self.visit_expression(expression);
                }
                _ => {}
            }
        }

        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.visit_expression(where_clause);
        }
    }

    fn visit_set(&mut self, stmt: &SetStmt) -> Self::Result {
        // 访问所有赋值
        for assignment in &stmt.assignments {
            self.visit_expression(&assignment.value);
        }
    }

    fn visit_remove(&mut self, stmt: &RemoveStmt) -> Self::Result {
        // 访问所有删除项
        for item in &stmt.items {
            self.visit_expression(item);
        }
    }

    fn visit_pipe(&mut self, stmt: &PipeStmt) -> Self::Result {
        // 访问表达式
        self.visit_expression(&stmt.expression);
    }

    fn visit_drop(&mut self, _stmt: &DropStmt) -> Self::Result {
        // DROP 语句没有子表达式
    }

    fn visit_desc(&mut self, _stmt: &DescStmt) -> Self::Result {
        // DESCRIBE 语句没有子表达式
    }

    fn visit_alter(&mut self, _stmt: &AlterStmt) -> Self::Result {
        // ALTER 语句没有子表达式
    }

    fn visit_change_password(&mut self, _stmt: &ChangePasswordStmt) -> Self::Result {
        // CHANGE PASSWORD 语句没有子表达式
    }
}

impl PatternVisitor for DefaultVisitor {
    type Result = ();

    fn visit_node_pattern(&mut self, pattern: &NodePattern) -> Self::Result {
        // 访问属性表达式
        if let Some(ref props) = pattern.properties {
            self.visit_expression(props);
        }

        // 访问谓词表达式
        for predicate in &pattern.predicates {
            self.visit_expression(predicate);
        }
    }

    fn visit_edge_pattern(&mut self, pattern: &EdgePattern) -> Self::Result {
        // 访问属性表达式
        if let Some(ref props) = pattern.properties {
            self.visit_expression(props);
        }

        // 访问谓词表达式
        for predicate in &pattern.predicates {
            self.visit_expression(predicate);
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

    fn visit_constant(&mut self, _expression: &ConstantExpression) -> Self::Result {
        // 常量表达式总是类型安全的
    }

    fn visit_variable(&mut self, _expression: &VariableExpression) -> Self::Result {
        // 变量表达式需要符号表检查（TODO）
    }

    fn visit_binary(&mut self, expression: &BinaryExpression) -> Self::Result {
        // 检查二元表达式的类型兼容性
        match expression.op {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo => {
                // 算术操作符需要数值类型
                // TODO: 实现类型检查逻辑
                self.warnings.push(format!(
                    "Arithmetic operation {} should have numeric operands",
                    expression.op
                ));
            }
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                // 逻辑操作符需要布尔类型
                self.warnings.push(format!(
                    "Logical operation {} should have boolean operands",
                    expression.op
                ));
            }
            _ => {}
        }

        // 递归检查子表达式
        self.visit_expression(&expression.left);
        self.visit_expression(&expression.right);
    }

    fn visit_function_call(&mut self, expression: &FunctionCallExpression) -> Self::Result {
        // 检查函数参数数量
        match expression.name.as_str() {
            "COUNT" => {
                if expression.args.len() > 1 && !expression.distinct {
                    self.errors
                        .push("COUNT function takes at most one argument".to_string());
                }
            }
            "SUM" | "AVG" | "MIN" | "MAX" => {
                if expression.args.len() != 1 {
                    self.errors
                        .push(format!("{} function takes exactly one argument", expression.name));
                }
            }
            _ => {}
        }

        // 递归检查所有参数
        for arg in &expression.args {
            self.visit_expression(arg);
        }
    }

    fn visit_unary(&mut self, expression: &UnaryExpression) -> Self::Result {
        // 递归检查操作数
        self.visit_expression(&expression.operand);
    }

    fn visit_property_access(&mut self, expression: &PropertyAccessExpression) -> Self::Result {
        // 递归检查对象表达式
        self.visit_expression(&expression.object);
    }

    fn visit_list(&mut self, expression: &ListExpression) -> Self::Result {
        // 递归检查所有元素
        for elem in &expression.elements {
            self.visit_expression(elem);
        }
    }

    fn visit_map(&mut self, expression: &MapExpression) -> Self::Result {
        // 递归检查所有值
        for (_, value) in &expression.pairs {
            self.visit_expression(value);
        }
    }

    fn visit_case(&mut self, expression: &CaseExpression) -> Self::Result {
        // 递归检查所有子表达式
        if let Some(ref match_expression) = expression.match_expression {
            self.visit_expression(match_expression);
        }

        for (when, then) in &expression.when_then_pairs {
            self.visit_expression(when);
            self.visit_expression(then);
        }

        if let Some(ref default) = expression.default {
            self.visit_expression(default);
        }
    }

    fn visit_subscript(&mut self, expression: &SubscriptExpression) -> Self::Result {
        // 递归检查集合和索引表达式
        self.visit_expression(&expression.collection);
        self.visit_expression(&expression.index);
    }

    fn visit_type_cast(&mut self, expression: &TypeCastExpression) -> Self::Result {
        // 递归检查表达式
        self.visit_expression(&expression.expression);
    }

    fn visit_range(&mut self, expression: &RangeExpression) -> Self::Result {
        // 递归检查集合表达式
        self.visit_expression(&expression.collection);
        // 递归检查起始和结束表达式
        if let Some(ref start) = expression.start {
            self.visit_expression(start);
        }
        if let Some(ref end) = expression.end {
            self.visit_expression(end);
        }
    }

    fn visit_path(&mut self, expression: &PathExpression) -> Self::Result {
        // 递归检查所有路径元素
        for elem in &expression.elements {
            self.visit_expression(elem);
        }
    }

    fn visit_label(&mut self, _expression: &LabelExpression) -> Self::Result {
        // 标签表达式总是类型安全的
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

    pub fn format(&mut self, expression: &Expression) -> String {
        self.result.clear();
        self.indent = 0;
        self.visit_expression(expression);
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

    fn visit_constant(&mut self, expression: &ConstantExpression) -> Self::Result {
        self.write_line(&format!("Constant: {:?}", expression.value));
    }

    fn visit_variable(&mut self, expression: &VariableExpression) -> Self::Result {
        self.write_line(&format!("Variable: {}", expression.name));
    }

    fn visit_binary(&mut self, expression: &BinaryExpression) -> Self::Result {
        self.write_line(&format!("Binary: {}", expression.op));
        self.increase_indent();
        self.write_line("Left:");
        self.increase_indent();
        self.visit_expression(&expression.left);
        self.decrease_indent();
        self.write_line("Right:");
        self.increase_indent();
        self.visit_expression(&expression.right);
        self.decrease_indent();
        self.decrease_indent();
    }

    fn visit_function_call(&mut self, expression: &FunctionCallExpression) -> Self::Result {
        self.write_line(&format!(
            "FunctionCall: {} ({} args)",
            expression.name,
            expression.args.len()
        ));
        if !expression.args.is_empty() {
            self.increase_indent();
            for (i, arg) in expression.args.iter().enumerate() {
                self.write_line(&format!("Arg {}:", i));
                self.increase_indent();
                self.visit_expression(arg);
                self.decrease_indent();
            }
            self.decrease_indent();
        }
    }

    fn visit_unary(&mut self, expression: &UnaryExpression) -> Self::Result {
        self.write_line(&format!("Unary: {}", expression.op));
        self.increase_indent();
        self.visit_expression(&expression.operand);
        self.decrease_indent();
    }

    fn visit_property_access(&mut self, expression: &PropertyAccessExpression) -> Self::Result {
        self.write_line(&format!("PropertyAccess: {}", expression.property));
        self.increase_indent();
        self.visit_expression(&expression.object);
        self.decrease_indent();
    }

    fn visit_list(&mut self, expression: &ListExpression) -> Self::Result {
        self.write_line(&format!("List: {} elements", expression.elements.len()));
        if !expression.elements.is_empty() {
            self.increase_indent();
            for (i, elem) in expression.elements.iter().enumerate() {
                self.write_line(&format!("Element {}:", i));
                self.increase_indent();
                self.visit_expression(elem);
                self.decrease_indent();
            }
            self.decrease_indent();
        }
    }

    fn visit_map(&mut self, expression: &MapExpression) -> Self::Result {
        self.write_line(&format!("Map: {} pairs", expression.pairs.len()));
        if !expression.pairs.is_empty() {
            self.increase_indent();
            for (key, value) in &expression.pairs {
                self.write_line(&format!("Key: {}", key));
                self.increase_indent();
                self.visit_expression(value);
                self.decrease_indent();
            }
            self.decrease_indent();
        }
    }

    fn visit_case(&mut self, expression: &CaseExpression) -> Self::Result {
        self.write_line("Case:");
        self.increase_indent();

        if let Some(ref match_expression) = expression.match_expression {
            self.write_line("Match:");
            self.increase_indent();
            self.visit_expression(match_expression);
            self.decrease_indent();
        }

        for (i, (when, then)) in expression.when_then_pairs.iter().enumerate() {
            self.write_line(&format!("When-Then {}:", i));
            self.increase_indent();
            self.write_line("When:");
            self.increase_indent();
            self.visit_expression(when);
            self.decrease_indent();
            self.write_line("Then:");
            self.increase_indent();
            self.visit_expression(then);
            self.decrease_indent();
            self.decrease_indent();
        }

        if let Some(ref default) = expression.default {
            self.write_line("Default:");
            self.increase_indent();
            self.visit_expression(default);
            self.decrease_indent();
        }

        self.decrease_indent();
    }

    fn visit_subscript(&mut self, expression: &SubscriptExpression) -> Self::Result {
        self.write_line("Subscript:");
        self.increase_indent();
        self.write_line("Collection:");
        self.increase_indent();
        self.visit_expression(&expression.collection);
        self.decrease_indent();
        self.write_line("Index:");
        self.increase_indent();
        self.visit_expression(&expression.index);
        self.decrease_indent();
        self.decrease_indent();
    }

    fn visit_type_cast(&mut self, expression: &TypeCastExpression) -> Self::Result {
        self.write_line(&format!("TypeCast: {:?}", expression.target_type));
        self.increase_indent();
        self.visit_expression(&expression.expression);
        self.decrease_indent();
    }

    fn visit_range(&mut self, expression: &RangeExpression) -> Self::Result {
        self.write_line("Range:");
        self.increase_indent();
        self.write_line("Collection:");
        self.increase_indent();
        self.visit_expression(&expression.collection);
        self.decrease_indent();
        if let Some(ref start) = expression.start {
            self.write_line("Start:");
            self.increase_indent();
            self.visit_expression(start);
            self.decrease_indent();
        }
        if let Some(ref end) = expression.end {
            self.write_line("End:");
            self.increase_indent();
            self.visit_expression(end);
            self.decrease_indent();
        }
        self.decrease_indent();
    }

    fn visit_path(&mut self, expression: &PathExpression) -> Self::Result {
        self.write_line(&format!("Path: {} elements", expression.elements.len()));
        if !expression.elements.is_empty() {
            self.increase_indent();
            for (i, elem) in expression.elements.iter().enumerate() {
                self.write_line(&format!("Element {}:", i));
                self.increase_indent();
                self.visit_expression(elem);
                self.decrease_indent();
            }
            self.decrease_indent();
        }
    }

    fn visit_label(&mut self, expression: &LabelExpression) -> Self::Result {
        self.write_line(&format!("Label: {}", expression.label));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_default_visitor() {
        let mut visitor = DefaultVisitor;
        let expression = Expression::Constant(ConstantExpression::new(Value::Int(42), Span::default()));

        // 应该能够访问而不出错
        visitor.visit_expression(&expression);
    }

    #[test]
    fn test_type_checker() {
        let mut checker = TypeChecker::new();
        let left = Expression::Constant(ConstantExpression::new(Value::Int(5), Span::default()));
        let right = Expression::Constant(ConstantExpression::new(
            Value::String("hello".to_string()),
            Span::default(),
        ));
        let expression = Expression::Binary(BinaryExpression::new(left, BinaryOp::Add, right, Span::default()));

        checker.visit_expression(&expression);
        assert!(checker.has_warnings());
    }

    #[test]
    fn test_ast_formatter() {
        let mut formatter = AstFormatter::new();
        let expression = Expression::Constant(ConstantExpression::new(Value::Int(42), Span::default()));

        let result = formatter.format(&expression);
        assert!(result.contains("Constant: Int(42)"));
    }
}
