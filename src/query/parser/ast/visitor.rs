//! 访问者模式实现
//!
//! 提供 AST 节点的访问者模式，支持遍历、转换和分析操作。

use super::{AstNode, Expression, Statement, Pattern, Query, PatternType, ExpressionType, VisitorResult};
use super::node::*;
use super::statement::*;
use super::pattern::*;
use crate::core::Value;

/// 访问者 trait - 定义访问 AST 节点的接口
pub trait Visitor {
    // 基础节点访问方法
    fn visit_node(&mut self, node: &dyn AstNode) -> VisitorResult;
    
    // 表达式访问方法
    fn visit_expression(&mut self, expr: &dyn Expression) -> VisitorResult;
    fn visit_constant_expr(&mut self, expr: &ConstantExpr) -> VisitorResult;
    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> VisitorResult;
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> VisitorResult;
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> VisitorResult;
    fn visit_function_call_expr(&mut self, expr: &FunctionCallExpr) -> VisitorResult;
    fn visit_property_access_expr(&mut self, expr: &PropertyAccessExpr) -> VisitorResult;
    fn visit_list_expr(&mut self, expr: &ListExpr) -> VisitorResult;
    fn visit_map_expr(&mut self, expr: &MapExpr) -> VisitorResult;
    fn visit_case_expr(&mut self, expr: &CaseExpr) -> VisitorResult;
    fn visit_subscript_expr(&mut self, expr: &SubscriptExpr) -> VisitorResult;
    fn visit_predicate_expr(&mut self, expr: &PredicateExpr) -> VisitorResult;
    
    // 语句访问方法
    fn visit_statement(&mut self, stmt: &dyn Statement) -> VisitorResult;
    fn visit_query_statement(&mut self, stmt: &QueryStatement) -> VisitorResult;
    fn visit_create_statement(&mut self, stmt: &CreateStatement) -> VisitorResult;
    fn visit_match_statement(&mut self, stmt: &MatchStatement) -> VisitorResult;
    fn visit_delete_statement(&mut self, stmt: &DeleteStatement) -> VisitorResult;
    fn visit_update_statement(&mut self, stmt: &UpdateStatement) -> VisitorResult;
    fn visit_go_statement(&mut self, stmt: &GoStatement) -> VisitorResult;
    fn visit_fetch_statement(&mut self, stmt: &FetchStatement) -> VisitorResult;
    fn visit_use_statement(&mut self, stmt: &UseStatement) -> VisitorResult;
    fn visit_show_statement(&mut self, stmt: &ShowStatement) -> VisitorResult;
    fn visit_explain_statement(&mut self, stmt: &ExplainStatement) -> VisitorResult;
    
    // 模式访问方法
    fn visit_pattern(&mut self, pattern: &dyn Pattern) -> VisitorResult;
    fn visit_node_pattern(&mut self, pattern: &NodePattern) -> VisitorResult;
    fn visit_edge_pattern(&mut self, pattern: &EdgePattern) -> VisitorResult;
    fn visit_path_pattern(&mut self, pattern: &PathPattern) -> VisitorResult;
    fn visit_variable_pattern(&mut self, pattern: &VariablePattern) -> VisitorResult;
    
    // 查询访问方法
    fn visit_query(&mut self, query: &Query) -> VisitorResult;
}

/// 默认访问者实现 - 提供基础遍历功能
pub struct DefaultVisitor;

impl Visitor for DefaultVisitor {
    fn visit_node(&mut self, _node: &dyn AstNode) -> VisitorResult {
        // 默认实现：什么都不做
        Ok(())
    }
    
    fn visit_expression(&mut self, expr: &dyn Expression) -> VisitorResult {
        // 递归访问子表达式
        for child in expr.children() {
            child.accept(self)?;
        }
        Ok(())
    }
    
    fn visit_constant_expr(&mut self, _expr: &ConstantExpr) -> VisitorResult {
        // 常量表达式没有子节点
        Ok(())
    }
    
    fn visit_variable_expr(&mut self, _expr: &VariableExpr) -> VisitorResult {
        // 变量表达式没有子节点
        Ok(())
    }
    
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> VisitorResult {
        // 访问左右操作数
        expr.left.accept(self)?;
        expr.right.accept(self)?;
        Ok(())
    }
    
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> VisitorResult {
        // 访问操作数
        expr.operand.accept(self)?;
        Ok(())
    }
    
    fn visit_function_call_expr(&mut self, expr: &FunctionCallExpr) -> VisitorResult {
        // 访问所有参数
        for arg in &expr.args {
            arg.accept(self)?;
        }
        Ok(())
    }
    
    fn visit_property_access_expr(&mut self, expr: &PropertyAccessExpr) -> VisitorResult {
        // 访问对象表达式
        expr.object.accept(self)?;
        Ok(())
    }
    
    fn visit_list_expr(&mut self, expr: &ListExpr) -> VisitorResult {
        // 访问所有元素
        for elem in &expr.elements {
            elem.accept(self)?;
        }
        Ok(())
    }
    
    fn visit_map_expr(&mut self, expr: &MapExpr) -> VisitorResult {
        // 访问所有值
        for (_, value) in &expr.pairs {
            value.accept(self)?;
        }
        Ok(())
    }
    
    fn visit_case_expr(&mut self, expr: &CaseExpr) -> VisitorResult {
        // 访问匹配表达式
        if let Some(ref match_expr) = expr.match_expr {
            match_expr.accept(self)?;
        }
        
        // 访问所有 WHEN-THEN 对
        for (when, then) in &expr.when_then_pairs {
            when.accept(self)?;
            then.accept(self)?;
        }
        
        // 访问默认表达式
        if let Some(ref default) = expr.default {
            default.accept(self)?;
        }
        Ok(())
    }
    
    fn visit_subscript_expr(&mut self, expr: &SubscriptExpr) -> VisitorResult {
        // 访问集合和索引表达式
        expr.collection.accept(self)?;
        expr.index.accept(self)?;
        Ok(())
    }
    
    fn visit_predicate_expr(&mut self, expr: &PredicateExpr) -> VisitorResult {
        // 访问列表和条件表达式
        expr.list.accept(self)?;
        expr.condition.accept(self)?;
        Ok(())
    }
    
    fn visit_statement(&mut self, stmt: &dyn Statement) -> VisitorResult {
        // 递归访问子节点
        for child in stmt.children() {
            child.accept(self)?;
        }
        Ok(())
    }
    
    fn visit_query_statement(&mut self, stmt: &QueryStatement) -> VisitorResult {
        // 访问所有语句
        for statement in &stmt.statements {
            statement.accept(self)?;
        }
        Ok(())
    }
    
    fn visit_create_statement(&mut self, stmt: &CreateStatement) -> VisitorResult {
        // 根据创建目标访问相关表达式
        match &stmt.target {
            CreateTarget::Node { properties, .. } => {
                if let Some(props) = properties {
                    props.accept(self)?;
                }
            }
            CreateTarget::Edge { src, dst, properties, .. } => {
                src.accept(self)?;
                dst.accept(self)?;
                if let Some(props) = properties {
                    props.accept(self)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    fn visit_match_statement(&mut self, stmt: &MatchStatement) -> VisitorResult {
        // 简化实现：暂时不访问子句
        Ok(())
    }
    
    fn visit_delete_statement(&mut self, stmt: &DeleteStatement) -> VisitorResult {
        // 根据删除目标访问相关表达式
        match &stmt.target {
            DeleteTarget::Vertices(vertices) => {
                for vertex in vertices {
                    vertex.accept(self)?;
                }
            }
            DeleteTarget::Edges { src, dst, rank, .. } => {
                src.accept(self)?;
                dst.accept(self)?;
                if let Some(ref rank) = rank {
                    rank.accept(self)?;
                }
            }
        }
        
        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            where_clause.accept(self)?;
        }
        Ok(())
    }
    
    fn visit_update_statement(&mut self, stmt: &UpdateStatement) -> VisitorResult {
        // 根据更新目标访问相关表达式
        match &stmt.target {
            UpdateTarget::Vertex(vertex) => {
                vertex.accept(self)?;
            }
            UpdateTarget::Edge { src, dst, rank, .. } => {
                src.accept(self)?;
                dst.accept(self)?;
                if let Some(ref rank) = rank {
                    rank.accept(self)?;
                }
            }
        }
        
        // 访问 SET 子句中的表达式
        for assignment in &stmt.set_clause.assignments {
            assignment.value.accept(self)?;
        }
        
        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            where_clause.accept(self)?;
        }
        Ok(())
    }
    
    fn visit_go_statement(&mut self, stmt: &GoStatement) -> VisitorResult {
        // 访问 FROM 子句中的顶点
        for vertex in &stmt.from.vertices {
            vertex.accept(self)?;
        }
        
        // 访问 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            where_clause.accept(self)?;
        }
        Ok(())
    }
    
    fn visit_fetch_statement(&mut self, stmt: &FetchStatement) -> VisitorResult {
        // 根据获取目标访问相关表达式
        match &stmt.target {
            FetchTarget::Vertices { ids, .. } => {
                for id in ids {
                    id.accept(self)?;
                }
            }
            FetchTarget::Edges { src, dst, rank, .. } => {
                src.accept(self)?;
                dst.accept(self)?;
                if let Some(ref rank) = rank {
                    rank.accept(self)?;
                }
            }
        }
        Ok(())
    }
    
    fn visit_use_statement(&mut self, _stmt: &UseStatement) -> VisitorResult {
        // USE 语句没有子表达式
        Ok(())
    }
    
    fn visit_show_statement(&mut self, _stmt: &ShowStatement) -> VisitorResult {
        // SHOW 语句没有子表达式
        Ok(())
    }
    
    fn visit_explain_statement(&mut self, stmt: &ExplainStatement) -> VisitorResult {
        // 访问被解释的语句
        stmt.statement.accept(self)?;
        Ok(())
    }
    
    fn visit_pattern(&mut self, pattern: &dyn Pattern) -> VisitorResult {
        // 递归访问模式元素
        match pattern.pattern_type() {
            PatternType::Node => {
                if let Some(node_pattern) = pattern.as_any().downcast_ref::<NodePattern>() {
                    self.visit_node_pattern(node_pattern)?;
                }
            }
            PatternType::Edge => {
                if let Some(edge_pattern) = pattern.as_any().downcast_ref::<EdgePattern>() {
                    self.visit_edge_pattern(edge_pattern)?;
                }
            }
            PatternType::Path => {
                if let Some(path_pattern) = pattern.as_any().downcast_ref::<PathPattern>() {
                    self.visit_path_pattern(path_pattern)?;
                }
            }
            PatternType::Variable => {
                if let Some(var_pattern) = pattern.as_any().downcast_ref::<VariablePattern>() {
                    self.visit_variable_pattern(var_pattern)?;
                }
            }
        }
        Ok(())
    }
    
    fn visit_node_pattern(&mut self, pattern: &NodePattern) -> VisitorResult {
        // 访问属性表达式
        if let Some(ref props) = pattern.properties {
            props.accept(self)?;
        }
        
        // 访问谓词表达式
        for predicate in &pattern.predicates {
            predicate.accept(self)?;
        }
        Ok(())
    }
    
    fn visit_edge_pattern(&mut self, pattern: &EdgePattern) -> VisitorResult {
        // 访问属性表达式
        if let Some(ref props) = pattern.properties {
            props.accept(self)?;
        }
        
        // 访问谓词表达式
        for predicate in &pattern.predicates {
            predicate.accept(self)?;
        }
        Ok(())
    }
    
    fn visit_path_pattern(&mut self, pattern: &PathPattern) -> VisitorResult {
        // 访问所有路径元素
        for element in &pattern.elements {
            match element {
                PathElement::Node(node) => node.accept(self)?,
                PathElement::Edge(edge) => edge.accept(self)?,
                PathElement::Alternative(patterns) => {
                    for p in patterns {
                        p.accept(self)?;
                    }
                }
                PathElement::Optional(elem) => elem.accept(self)?,
                PathElement::Repeated(elem, _) => elem.accept(self)?,
            }
        }
        Ok(())
    }
    
    fn visit_variable_pattern(&mut self, _pattern: &VariablePattern) -> VisitorResult {
        // 变量模式没有子表达式
        Ok(())
    }
    
    fn visit_query(&mut self, query: &Query) -> VisitorResult {
        // 访问所有语句
        for statement in &query.statements {
            statement.accept(self)?;
        }
        Ok(())
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

impl Visitor for TypeChecker {
    fn visit_node(&mut self, node: &dyn AstNode) -> VisitorResult {
        DefaultVisitor.visit_node(node)
    }
    
    fn visit_expression(&mut self, expr: &dyn Expression) -> VisitorResult {
        DefaultVisitor.visit_expression(expr)
    }
    
    fn visit_constant_expr(&mut self, expr: &ConstantExpr) -> VisitorResult {
        DefaultVisitor.visit_constant_expr(expr)
    }
    
    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> VisitorResult {
        DefaultVisitor.visit_variable_expr(expr)
    }
    
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> VisitorResult {
        // 检查二元表达式的类型兼容性
        match expr.op {
            super::node::BinaryOp::Add | super::node::BinaryOp::Sub | super::node::BinaryOp::Mul |
            super::node::BinaryOp::Div | super::node::BinaryOp::Mod => {
                // 算术操作符需要数值类型
                if !expr.left.expr_type().is_numeric() || !expr.right.expr_type().is_numeric() {
                    self.errors.push(format!(
                        "Arithmetic operation {} requires numeric operands",
                        expr.op
                    ));
                }
            }
            super::node::BinaryOp::And | super::node::BinaryOp::Or | super::node::BinaryOp::Xor => {
                // 逻辑操作符需要布尔类型
                if expr.left.expr_type() != super::ExpressionType::Constant ||
                   expr.right.expr_type() != super::ExpressionType::Constant {
                    self.warnings.push("Logical operations on non-constant expressions".to_string());
                }
            }
            _ => {}
        }
        
        // 递归访问子表达式
        DefaultVisitor.visit_binary_expr(expr)
    }
    
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> VisitorResult {
        DefaultVisitor.visit_unary_expr(expr)
    }
    
    fn visit_function_call_expr(&mut self, expr: &FunctionCallExpr) -> VisitorResult {
        // 检查函数参数数量
        match expr.name.as_str() {
            "COUNT" => {
                if expr.args.len() > 1 && !expr.distinct {
                    self.errors.push("COUNT function takes at most one argument".to_string());
                }
            }
            "SUM" | "AVG" | "MIN" | "MAX" => {
                if expr.args.len() != 1 {
                    self.errors.push(format!(
                        "{} function takes exactly one argument",
                        expr.name
                    ));
                }
            }
            _ => {}
        }
        
        // 递归访问子表达式
        DefaultVisitor.visit_function_call_expr(expr)
    }
    
    fn visit_property_access_expr(&mut self, expr: &PropertyAccessExpr) -> VisitorResult {
        DefaultVisitor.visit_property_access_expr(expr)
    }
    
    fn visit_list_expr(&mut self, expr: &ListExpr) -> VisitorResult {
        DefaultVisitor.visit_list_expr(expr)
    }
    
    fn visit_map_expr(&mut self, expr: &MapExpr) -> VisitorResult {
        DefaultVisitor.visit_map_expr(expr)
    }
    
    fn visit_case_expr(&mut self, expr: &CaseExpr) -> VisitorResult {
        DefaultVisitor.visit_case_expr(expr)
    }
    
    fn visit_subscript_expr(&mut self, expr: &SubscriptExpr) -> VisitorResult {
        DefaultVisitor.visit_subscript_expr(expr)
    }
    
    fn visit_predicate_expr(&mut self, expr: &PredicateExpr) -> VisitorResult {
        DefaultVisitor.visit_predicate_expr(expr)
    }
    
    fn visit_statement(&mut self, stmt: &dyn Statement) -> VisitorResult {
        DefaultVisitor.visit_statement(stmt)
    }
    
    fn visit_query_statement(&mut self, stmt: &QueryStatement) -> VisitorResult {
        DefaultVisitor.visit_query_statement(stmt)
    }
    
    fn visit_create_statement(&mut self, stmt: &CreateStatement) -> VisitorResult {
        DefaultVisitor.visit_create_statement(stmt)
    }
    
    fn visit_delete_statement(&mut self, stmt: &DeleteStatement) -> VisitorResult {
        DefaultVisitor.visit_delete_statement(stmt)
    }
    
    fn visit_match_statement(&mut self, stmt: &MatchStatement) -> VisitorResult {
        DefaultVisitor.visit_match_statement(stmt)
    }
    
    fn visit_update_statement(&mut self, stmt: &UpdateStatement) -> VisitorResult {
        DefaultVisitor.visit_update_statement(stmt)
    }
    
    fn visit_go_statement(&mut self, stmt: &GoStatement) -> VisitorResult {
        DefaultVisitor.visit_go_statement(stmt)
    }
    
    fn visit_fetch_statement(&mut self, stmt: &FetchStatement) -> VisitorResult {
        DefaultVisitor.visit_fetch_statement(stmt)
    }
    
    fn visit_use_statement(&mut self, stmt: &UseStatement) -> VisitorResult {
        DefaultVisitor.visit_use_statement(stmt)
    }
    
    fn visit_show_statement(&mut self, stmt: &ShowStatement) -> VisitorResult {
        DefaultVisitor.visit_show_statement(stmt)
    }
    
    fn visit_explain_statement(&mut self, stmt: &ExplainStatement) -> VisitorResult {
        DefaultVisitor.visit_explain_statement(stmt)
    }
    
    fn visit_pattern(&mut self, pattern: &dyn Pattern) -> VisitorResult {
        DefaultVisitor.visit_pattern(pattern)
    }
    
    fn visit_node_pattern(&mut self, pattern: &NodePattern) -> VisitorResult {
        DefaultVisitor.visit_node_pattern(pattern)
    }
    
    fn visit_edge_pattern(&mut self, pattern: &EdgePattern) -> VisitorResult {
        DefaultVisitor.visit_edge_pattern(pattern)
    }
    
    fn visit_path_pattern(&mut self, pattern: &PathPattern) -> VisitorResult {
        DefaultVisitor.visit_path_pattern(pattern)
    }
    
    fn visit_variable_pattern(&mut self, pattern: &VariablePattern) -> VisitorResult {
        DefaultVisitor.visit_variable_pattern(pattern)
    }
    
    fn visit_query(&mut self, query: &Query) -> VisitorResult {
        DefaultVisitor.visit_query(query)
    }
}

/// 语义分析访问者
pub struct SemanticAnalyzer {
    pub symbol_table: std::collections::HashMap<String, super::ExpressionType>,
    pub errors: Vec<String>,
    pub current_scope: Vec<String>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: std::collections::HashMap::new(),
            errors: Vec::new(),
            current_scope: Vec::new(),
        }
    }
    
    pub fn enter_scope(&mut self, scope_name: String) {
        self.current_scope.push(scope_name);
    }
    
    pub fn exit_scope(&mut self) {
        self.current_scope.pop();
    }
    
    pub fn add_symbol(&mut self, name: String, expr_type: super::ExpressionType) {
        let full_name = if self.current_scope.is_empty() {
            name
        } else {
            format!("{}::{}", self.current_scope.join("::"), name)
        };
        self.symbol_table.insert(full_name, expr_type);
    }
    
    pub fn lookup_symbol(&self, name: &str) -> Option<&super::ExpressionType> {
        // 先在当前作用域查找
        if !self.current_scope.is_empty() {
            let full_name = format!("{}::{}", self.current_scope.join("::"), name);
            if let Some(expr_type) = self.symbol_table.get(&full_name) {
                return Some(expr_type);
            }
        }
        
        // 然后在全局作用域查找
        self.symbol_table.get(name)
    }
}

impl Visitor for SemanticAnalyzer {
    fn visit_node(&mut self, node: &dyn AstNode) -> VisitorResult {
        DefaultVisitor.visit_node(node)
    }
    
    fn visit_expression(&mut self, expr: &dyn Expression) -> VisitorResult {
        DefaultVisitor.visit_expression(expr)
    }
    
    fn visit_constant_expr(&mut self, expr: &ConstantExpr) -> VisitorResult {
        DefaultVisitor.visit_constant_expr(expr)
    }
    
    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> VisitorResult {
        // 检查变量是否已定义
        if self.lookup_symbol(&expr.name).is_none() {
            self.errors.push(format!("Undefined variable: {}", expr.name));
        }
        Ok(())
    }
    
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> VisitorResult {
        DefaultVisitor.visit_binary_expr(expr)
    }
    
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> VisitorResult {
        DefaultVisitor.visit_unary_expr(expr)
    }
    
    fn visit_function_call_expr(&mut self, expr: &FunctionCallExpr) -> VisitorResult {
        DefaultVisitor.visit_function_call_expr(expr)
    }
    
    fn visit_property_access_expr(&mut self, expr: &PropertyAccessExpr) -> VisitorResult {
        DefaultVisitor.visit_property_access_expr(expr)
    }
    
    fn visit_list_expr(&mut self, expr: &ListExpr) -> VisitorResult {
        DefaultVisitor.visit_list_expr(expr)
    }
    
    fn visit_map_expr(&mut self, expr: &MapExpr) -> VisitorResult {
        DefaultVisitor.visit_map_expr(expr)
    }
    
    fn visit_case_expr(&mut self, expr: &CaseExpr) -> VisitorResult {
        DefaultVisitor.visit_case_expr(expr)
    }
    
    fn visit_subscript_expr(&mut self, expr: &SubscriptExpr) -> VisitorResult {
        DefaultVisitor.visit_subscript_expr(expr)
    }
    
    fn visit_predicate_expr(&mut self, expr: &PredicateExpr) -> VisitorResult {
        DefaultVisitor.visit_predicate_expr(expr)
    }
    
    fn visit_statement(&mut self, stmt: &dyn Statement) -> VisitorResult {
        DefaultVisitor.visit_statement(stmt)
    }
    
    fn visit_query_statement(&mut self, stmt: &QueryStatement) -> VisitorResult {
        DefaultVisitor.visit_query_statement(stmt)
    }
    
    fn visit_create_statement(&mut self, stmt: &CreateStatement) -> VisitorResult {
        DefaultVisitor.visit_create_statement(stmt)
    }
    
    fn visit_delete_statement(&mut self, stmt: &DeleteStatement) -> VisitorResult {
        DefaultVisitor.visit_delete_statement(stmt)
    }
    
    fn visit_update_statement(&mut self, stmt: &UpdateStatement) -> VisitorResult {
        DefaultVisitor.visit_update_statement(stmt)
    }
    
    fn visit_go_statement(&mut self, stmt: &GoStatement) -> VisitorResult {
        DefaultVisitor.visit_go_statement(stmt)
    }
    
    fn visit_fetch_statement(&mut self, stmt: &FetchStatement) -> VisitorResult {
        DefaultVisitor.visit_fetch_statement(stmt)
    }
    
    fn visit_use_statement(&mut self, stmt: &UseStatement) -> VisitorResult {
        DefaultVisitor.visit_use_statement(stmt)
    }
    
    fn visit_show_statement(&mut self, stmt: &ShowStatement) -> VisitorResult {
        DefaultVisitor.visit_show_statement(stmt)
    }
    
    fn visit_explain_statement(&mut self, stmt: &ExplainStatement) -> VisitorResult {
        DefaultVisitor.visit_explain_statement(stmt)
    }
    
    fn visit_pattern(&mut self, pattern: &dyn Pattern) -> VisitorResult {
        DefaultVisitor.visit_pattern(pattern)
    }
    
    fn visit_node_pattern(&mut self, pattern: &NodePattern) -> VisitorResult {
        DefaultVisitor.visit_node_pattern(pattern)
    }
    
    fn visit_edge_pattern(&mut self, pattern: &EdgePattern) -> VisitorResult {
        DefaultVisitor.visit_edge_pattern(pattern)
    }
    
    fn visit_path_pattern(&mut self, pattern: &PathPattern) -> VisitorResult {
        DefaultVisitor.visit_path_pattern(pattern)
    }
    
    fn visit_variable_pattern(&mut self, pattern: &VariablePattern) -> VisitorResult {
        DefaultVisitor.visit_variable_pattern(pattern)
    }
    
    fn visit_query(&mut self, query: &Query) -> VisitorResult {
        DefaultVisitor.visit_query(query)
    }
    
    fn visit_match_statement(&mut self, stmt: &MatchStatement) -> VisitorResult {
        DefaultVisitor.visit_match_statement(stmt)
    }
    
}

/// AST 转换器 - 用于转换 AST 结构
pub struct AstTransformer {
    pub transformations: Vec<String>,
}

impl AstTransformer {
    pub fn new() -> Self {
        Self {
            transformations: Vec::new(),
        }
    }
}

impl Visitor for AstTransformer {
    fn visit_node(&mut self, node: &dyn AstNode) -> VisitorResult {
        DefaultVisitor.visit_node(node)
    }
    
    fn visit_expression(&mut self, expr: &dyn Expression) -> VisitorResult {
        DefaultVisitor.visit_expression(expr)
    }
    
    fn visit_constant_expr(&mut self, expr: &ConstantExpr) -> VisitorResult {
        DefaultVisitor.visit_constant_expr(expr)
    }
    
    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> VisitorResult {
        DefaultVisitor.visit_variable_expr(expr)
    }
    
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> VisitorResult {
        // 示例：将 a + 0 转换为 a
        if let (super::node::BinaryOp::Add, Some(constant)) = (&expr.op, expr.right.as_any().downcast_ref::<ConstantExpr>()) {
            if let Value::Int(0) = constant.value {
                self.transformations.push("Optimized a + 0 to a".to_string());
            }
        }
        
        // 递归访问子表达式
        DefaultVisitor.visit_binary_expr(expr)
    }
    
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> VisitorResult {
        DefaultVisitor.visit_unary_expr(expr)
    }
    
    fn visit_function_call_expr(&mut self, expr: &FunctionCallExpr) -> VisitorResult {
        DefaultVisitor.visit_function_call_expr(expr)
    }
    
    fn visit_property_access_expr(&mut self, expr: &PropertyAccessExpr) -> VisitorResult {
        DefaultVisitor.visit_property_access_expr(expr)
    }
    
    fn visit_list_expr(&mut self, expr: &ListExpr) -> VisitorResult {
        DefaultVisitor.visit_list_expr(expr)
    }
    
    fn visit_map_expr(&mut self, expr: &MapExpr) -> VisitorResult {
        DefaultVisitor.visit_map_expr(expr)
    }
    
    fn visit_case_expr(&mut self, expr: &CaseExpr) -> VisitorResult {
        DefaultVisitor.visit_case_expr(expr)
    }
    
    fn visit_subscript_expr(&mut self, expr: &SubscriptExpr) -> VisitorResult {
        DefaultVisitor.visit_subscript_expr(expr)
    }
    
    fn visit_predicate_expr(&mut self, expr: &PredicateExpr) -> VisitorResult {
        DefaultVisitor.visit_predicate_expr(expr)
    }
    
    fn visit_statement(&mut self, stmt: &dyn Statement) -> VisitorResult {
        DefaultVisitor.visit_statement(stmt)
    }
    
    fn visit_query_statement(&mut self, stmt: &QueryStatement) -> VisitorResult {
        DefaultVisitor.visit_query_statement(stmt)
    }
    
    fn visit_create_statement(&mut self, stmt: &CreateStatement) -> VisitorResult {
        DefaultVisitor.visit_create_statement(stmt)
    }
    
    fn visit_match_statement(&mut self, stmt: &MatchStatement) -> VisitorResult {
        DefaultVisitor.visit_match_statement(stmt)
    }
    
    fn visit_delete_statement(&mut self, stmt: &DeleteStatement) -> VisitorResult {
        DefaultVisitor.visit_delete_statement(stmt)
    }
    
    fn visit_update_statement(&mut self, stmt: &UpdateStatement) -> VisitorResult {
        DefaultVisitor.visit_update_statement(stmt)
    }
    
    fn visit_go_statement(&mut self, stmt: &GoStatement) -> VisitorResult {
        DefaultVisitor.visit_go_statement(stmt)
    }
    
    fn visit_fetch_statement(&mut self, stmt: &FetchStatement) -> VisitorResult {
        DefaultVisitor.visit_fetch_statement(stmt)
    }
    
    fn visit_use_statement(&mut self, stmt: &UseStatement) -> VisitorResult {
        DefaultVisitor.visit_use_statement(stmt)
    }
    
    fn visit_show_statement(&mut self, stmt: &ShowStatement) -> VisitorResult {
        DefaultVisitor.visit_show_statement(stmt)
    }
    
    fn visit_explain_statement(&mut self, stmt: &ExplainStatement) -> VisitorResult {
        DefaultVisitor.visit_explain_statement(stmt)
    }
    
    fn visit_pattern(&mut self, pattern: &dyn Pattern) -> VisitorResult {
        DefaultVisitor.visit_pattern(pattern)
    }
    
    fn visit_node_pattern(&mut self, pattern: &NodePattern) -> VisitorResult {
        DefaultVisitor.visit_node_pattern(pattern)
    }
    
    fn visit_edge_pattern(&mut self, pattern: &EdgePattern) -> VisitorResult {
        DefaultVisitor.visit_edge_pattern(pattern)
    }
    
    fn visit_path_pattern(&mut self, pattern: &PathPattern) -> VisitorResult {
        DefaultVisitor.visit_path_pattern(pattern)
    }
    
    fn visit_variable_pattern(&mut self, pattern: &VariablePattern) -> VisitorResult {
        DefaultVisitor.visit_variable_pattern(pattern)
    }
    
    fn visit_query(&mut self, query: &Query) -> VisitorResult {
        DefaultVisitor.visit_query(query)
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
    
    pub fn format(&mut self, node: &dyn AstNode) -> String {
        self.result.clear();
        self.indent = 0;
        node.accept(self);
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

impl Visitor for AstFormatter {
    fn visit_node(&mut self, node: &dyn AstNode) -> VisitorResult {
        DefaultVisitor.visit_node(node)
    }
    
    fn visit_expression(&mut self, expr: &dyn Expression) -> VisitorResult {
        DefaultVisitor.visit_expression(expr)
    }
    
    fn visit_constant_expr(&mut self, expr: &ConstantExpr) -> VisitorResult {
        self.write_line(&format!("Constant: {:?}", expr.value));
        Ok(())
    }
    
    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> VisitorResult {
        self.write_line(&format!("Variable: {}", expr.name));
        Ok(())
    }
    
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> VisitorResult {
        self.write_line(&format!("Binary: {} ({})", expr.op, expr.node_type()));
        self.increase_indent();
        self.write_line("Left:");
        self.increase_indent();
        expr.left.accept(self)?;
        self.decrease_indent();
        self.write_line("Right:");
        self.increase_indent();
        expr.right.accept(self)?;
        self.decrease_indent();
        self.decrease_indent();
        Ok(())
    }
    
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> VisitorResult {
        DefaultVisitor.visit_unary_expr(expr)
    }
    
    fn visit_function_call_expr(&mut self, expr: &FunctionCallExpr) -> VisitorResult {
        self.write_line(&format!("FunctionCall: {} ({} args)", expr.name, expr.args.len()));
        if !expr.args.is_empty() {
            self.increase_indent();
            for (i, arg) in expr.args.iter().enumerate() {
                self.write_line(&format!("Arg {}:", i));
                self.increase_indent();
                arg.accept(self)?;
                self.decrease_indent();
            }
            self.decrease_indent();
        }
        Ok(())
    }
    
    fn visit_property_access_expr(&mut self, expr: &PropertyAccessExpr) -> VisitorResult {
        DefaultVisitor.visit_property_access_expr(expr)
    }
    
    fn visit_list_expr(&mut self, expr: &ListExpr) -> VisitorResult {
        DefaultVisitor.visit_list_expr(expr)
    }
    
    fn visit_map_expr(&mut self, expr: &MapExpr) -> VisitorResult {
        DefaultVisitor.visit_map_expr(expr)
    }
    
    fn visit_case_expr(&mut self, expr: &CaseExpr) -> VisitorResult {
        DefaultVisitor.visit_case_expr(expr)
    }
    
    fn visit_subscript_expr(&mut self, expr: &SubscriptExpr) -> VisitorResult {
        DefaultVisitor.visit_subscript_expr(expr)
    }
    
    fn visit_predicate_expr(&mut self, expr: &PredicateExpr) -> VisitorResult {
        DefaultVisitor.visit_predicate_expr(expr)
    }
    
    fn visit_statement(&mut self, stmt: &dyn Statement) -> VisitorResult {
        DefaultVisitor.visit_statement(stmt)
    }
    
    fn visit_query_statement(&mut self, stmt: &QueryStatement) -> VisitorResult {
        DefaultVisitor.visit_query_statement(stmt)
    }
    
    fn visit_create_statement(&mut self, stmt: &CreateStatement) -> VisitorResult {
        DefaultVisitor.visit_create_statement(stmt)
    }
    
    fn visit_match_statement(&mut self, stmt: &MatchStatement) -> VisitorResult {
        DefaultVisitor.visit_match_statement(stmt)
    }
    
    fn visit_delete_statement(&mut self, stmt: &DeleteStatement) -> VisitorResult {
        DefaultVisitor.visit_delete_statement(stmt)
    }
    
    fn visit_update_statement(&mut self, stmt: &UpdateStatement) -> VisitorResult {
        DefaultVisitor.visit_update_statement(stmt)
    }
    
    fn visit_go_statement(&mut self, stmt: &GoStatement) -> VisitorResult {
        DefaultVisitor.visit_go_statement(stmt)
    }
    
    fn visit_fetch_statement(&mut self, stmt: &FetchStatement) -> VisitorResult {
        DefaultVisitor.visit_fetch_statement(stmt)
    }
    
    fn visit_use_statement(&mut self, stmt: &UseStatement) -> VisitorResult {
        DefaultVisitor.visit_use_statement(stmt)
    }
    
    fn visit_show_statement(&mut self, stmt: &ShowStatement) -> VisitorResult {
        DefaultVisitor.visit_show_statement(stmt)
    }
    
    fn visit_explain_statement(&mut self, stmt: &ExplainStatement) -> VisitorResult {
        DefaultVisitor.visit_explain_statement(stmt)
    }
    
    fn visit_pattern(&mut self, pattern: &dyn Pattern) -> VisitorResult {
        DefaultVisitor.visit_pattern(pattern)
    }
    
    fn visit_node_pattern(&mut self, pattern: &NodePattern) -> VisitorResult {
        DefaultVisitor.visit_node_pattern(pattern)
    }
    
    fn visit_edge_pattern(&mut self, pattern: &EdgePattern) -> VisitorResult {
        DefaultVisitor.visit_edge_pattern(pattern)
    }
    
    fn visit_path_pattern(&mut self, pattern: &PathPattern) -> VisitorResult {
        DefaultVisitor.visit_path_pattern(pattern)
    }
    
    fn visit_variable_pattern(&mut self, pattern: &VariablePattern) -> VisitorResult {
        DefaultVisitor.visit_variable_pattern(pattern)
    }
    
    fn visit_query(&mut self, query: &Query) -> VisitorResult {
        DefaultVisitor.visit_query(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    
    #[test]
    fn test_default_visitor() {
        let mut visitor = DefaultVisitor;
        let expr = ConstantExpr::new(Value::Int(42), Span::default());
        
        // 应该能够访问而不出错
        expr.accept(&mut visitor).unwrap();
    }
    
    #[test]
    fn test_type_checker() {
        let mut checker = TypeChecker::new();
        let left = Box::new(ConstantExpr::new(Value::Int(5), Span::default()));
        let right = Box::new(ConstantExpr::new(Value::String("hello".to_string()), Span::default()));
        let expr = BinaryExpr::new(left, BinaryOp::Add, right, Span::default());
        
        expr.accept(&mut checker).unwrap();
        assert!(checker.has_errors());
    }
    
    #[test]
    fn test_ast_formatter() {
        let mut formatter = AstFormatter::new();
        let expr = ConstantExpr::new(Value::Int(42), Span::default());
        
        let result = formatter.format(&expr);
        assert!(result.contains("Constant: Int(42)"));
    }
}