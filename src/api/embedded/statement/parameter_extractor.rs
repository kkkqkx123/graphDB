//! 参数提取器模块
//!
//! 负责从查询语句中提取参数信息

use crate::api::core::CoreError;
use crate::core::types::expression::{ContextualExpression, Expression};
use crate::core::DataType;
use crate::query::parser::ast::pattern::{PathElement, Pattern};
use crate::query::parser::ast::stmt::{
    CreateStmt, DeleteStmt, FetchStmt, FindPathStmt, GoStmt, GroupByStmt,
    InsertStmt, LookupStmt, MatchStmt, MergeStmt, PipeStmt, QueryStmt, RemoveStmt, ReturnStmt,
    SetStmt, Stmt, SubgraphStmt, UnwindStmt, UpdateStmt, WithStmt, YieldStmt,
};
use crate::query::parser::parser::Parser;
use std::collections::HashMap;

/// 参数提取器
///
/// 负责从查询语句中提取所有参数信息
pub struct ParameterExtractor;

impl ParameterExtractor {
    /// 从查询中提取参数
    ///
    /// 使用查询解析器解析查询语句，从 AST 中提取所有参数（$name 格式）
    /// 这是正确的实现方式，能够准确识别查询中的参数位置
    ///
    /// # 返回
    /// - 成功时返回参数映射
    /// - 失败时返回解析错误
    pub fn extract_parameters(query: &str) -> Result<HashMap<String, DataType>, CoreError> {
        let mut params = HashMap::new();

        let mut parser = Parser::new(query);
        match parser.parse() {
            Ok(parser_result) => {
                Self::extract_params_from_stmt(&parser_result.ast.stmt, &mut params);
                Ok(params)
            }
            Err(e) => {
                Err(CoreError::QueryExecutionFailed(format!(
                    "查询解析失败: {:?}",
                    e
                )))
            }
        }
    }

    /// 从语句中提取参数
    fn extract_params_from_stmt(stmt: &Stmt, params: &mut HashMap<String, DataType>) {
        match stmt {
            Stmt::Match(match_stmt) => {
                Self::extract_params_from_match(match_stmt, params);
            }
            Stmt::Go(go_stmt) => {
                Self::extract_params_from_go(go_stmt, params);
            }
            Stmt::Insert(insert_stmt) => {
                Self::extract_params_from_insert(insert_stmt, params);
            }
            Stmt::Update(update_stmt) => {
                Self::extract_params_from_update(update_stmt, params);
            }
            Stmt::Delete(delete_stmt) => {
                Self::extract_params_from_delete(delete_stmt, params);
            }
            Stmt::Fetch(fetch_stmt) => {
                Self::extract_params_from_fetch(fetch_stmt, params);
            }
            Stmt::Lookup(lookup_stmt) => {
                Self::extract_params_from_lookup(lookup_stmt, params);
            }
            Stmt::FindPath(find_path_stmt) => {
                Self::extract_params_from_find_path(find_path_stmt, params);
            }
            Stmt::Merge(merge_stmt) => {
                Self::extract_params_from_merge(merge_stmt, params);
            }
            Stmt::Unwind(unwind_stmt) => {
                Self::extract_params_from_unwind(unwind_stmt, params);
            }
            Stmt::With(with_stmt) => {
                Self::extract_params_from_with(with_stmt, params);
            }
            Stmt::Yield(yield_stmt) => {
                Self::extract_params_from_yield(yield_stmt, params);
            }
            Stmt::Set(set_stmt) => {
                Self::extract_params_from_set(set_stmt, params);
            }
            Stmt::Remove(remove_stmt) => {
                Self::extract_params_from_remove(remove_stmt, params);
            }
            Stmt::Create(create_stmt) => {
                Self::extract_params_from_create(create_stmt, params);
            }
            Stmt::Query(query_stmt) => {
                Self::extract_params_from_query(query_stmt, params);
            }
            Stmt::Return(return_stmt) => {
                Self::extract_params_from_return(return_stmt, params);
            }
            Stmt::GroupBy(group_by_stmt) => {
                Self::extract_params_from_group_by(group_by_stmt, params);
            }
            Stmt::Subgraph(subgraph_stmt) => {
                Self::extract_params_from_subgraph(subgraph_stmt, params);
            }
            Stmt::Pipe(pipe_stmt) => {
                Self::extract_params_from_pipe(pipe_stmt, params);
            }
            Stmt::Show(_) | Stmt::Use(_) | Stmt::Explain(_) | Stmt::Profile(_)
            | Stmt::Drop(_) | Stmt::Desc(_) | Stmt::Alter(_)
            | Stmt::CreateUser(_) | Stmt::AlterUser(_) | Stmt::DropUser(_)
            | Stmt::ChangePassword(_) | Stmt::Grant(_) | Stmt::Revoke(_)
            | Stmt::DescribeUser(_) | Stmt::ShowUsers(_) | Stmt::ShowRoles(_)
            | Stmt::ShowCreate(_) | Stmt::ShowSessions(_) | Stmt::ShowQueries(_)
            | Stmt::KillQuery(_) | Stmt::ShowConfigs(_) | Stmt::UpdateConfigs(_)
            | Stmt::Assignment(_) | Stmt::SetOperation(_) => {}
        }
    }

    /// 从 MATCH 语句中提取参数
    fn extract_params_from_match(match_stmt: &MatchStmt, params: &mut HashMap<String, DataType>) {
        for pattern in &match_stmt.patterns {
            Self::extract_params_from_pattern(pattern, params);
        }

        if let Some(where_clause) = &match_stmt.where_clause {
            Self::extract_params_from_expr(where_clause, params);
        }
    }

    /// 从 GO 语句中提取参数
    fn extract_params_from_go(go_stmt: &GoStmt, params: &mut HashMap<String, DataType>) {
        for expr in &go_stmt.from.vertices {
            Self::extract_params_from_expr(expr, params);
        }

        if let Some(where_clause) = &go_stmt.where_clause {
            Self::extract_params_from_expr(where_clause, params);
        }
    }

    /// 从 INSERT 语句中提取参数
    fn extract_params_from_insert(
        insert_stmt: &InsertStmt,
        params: &mut HashMap<String, DataType>,
    ) {
        use crate::query::parser::ast::stmt::InsertTarget;

        match &insert_stmt.target {
            InsertTarget::Vertices { values, .. } => {
                for vertex_row in values {
                    Self::extract_params_from_expr(&vertex_row.vid, params);
                    for tag_values in &vertex_row.tag_values {
                        for expr in tag_values {
                            Self::extract_params_from_expr(expr, params);
                        }
                    }
                }
            }
            InsertTarget::Edge { edges, .. } => {
                for (src, dst, rank, props) in edges {
                    Self::extract_params_from_expr(src, params);
                    Self::extract_params_from_expr(dst, params);
                    if let Some(rank_expr) = rank {
                        Self::extract_params_from_expr(rank_expr, params);
                    }
                    for prop in props {
                        Self::extract_params_from_expr(prop, params);
                    }
                }
            }
        }
    }

    /// 从 UPDATE 语句中提取参数
    fn extract_params_from_update(
        update_stmt: &UpdateStmt,
        params: &mut HashMap<String, DataType>,
    ) {
        use crate::query::parser::ast::stmt::UpdateTarget;

        match &update_stmt.target {
            UpdateTarget::Vertex(expr) => {
                Self::extract_params_from_expr(expr, params);
            }
            UpdateTarget::Edge { src, dst, rank, .. } => {
                Self::extract_params_from_expr(src, params);
                Self::extract_params_from_expr(dst, params);
                if let Some(rank_expr) = rank {
                    Self::extract_params_from_expr(rank_expr, params);
                }
            }
            UpdateTarget::Tag(_) => {}
            UpdateTarget::TagOnVertex { vid, .. } => {
                Self::extract_params_from_expr(vid, params);
            }
        }

        for assignment in &update_stmt.set_clause.assignments {
            Self::extract_params_from_expr(&assignment.value, params);
        }

        if let Some(where_clause) = &update_stmt.where_clause {
            Self::extract_params_from_expr(where_clause, params);
        }
    }

    /// 从 DELETE 语句中提取参数
    fn extract_params_from_delete(
        delete_stmt: &DeleteStmt,
        params: &mut HashMap<String, DataType>,
    ) {
        use crate::query::parser::ast::stmt::DeleteTarget;

        match &delete_stmt.target {
            DeleteTarget::Vertices(vertices) => {
                for expr in vertices {
                    Self::extract_params_from_expr(expr, params);
                }
            }
            DeleteTarget::Edges { edges, .. } => {
                for (src, dst, rank) in edges {
                    Self::extract_params_from_expr(src, params);
                    Self::extract_params_from_expr(dst, params);
                    if let Some(rank_expr) = rank {
                        Self::extract_params_from_expr(rank_expr, params);
                    }
                }
            }
            DeleteTarget::Tags { vertex_ids, .. } => {
                for expr in vertex_ids {
                    Self::extract_params_from_expr(expr, params);
                }
            }
            _ => {}
        }

        if let Some(where_clause) = &delete_stmt.where_clause {
            Self::extract_params_from_expr(where_clause, params);
        }
    }

    /// 从 FETCH 语句中提取参数
    fn extract_params_from_fetch(fetch_stmt: &FetchStmt, params: &mut HashMap<String, DataType>) {
        use crate::query::parser::ast::stmt::FetchTarget;

        match &fetch_stmt.target {
            FetchTarget::Vertices { ids, properties: _ } => {
                for id in ids {
                    Self::extract_params_from_expr(id, params);
                }
            }
            FetchTarget::Edges {
                src,
                dst,
                edge_type: _,
                rank,
                properties: _,
            } => {
                Self::extract_params_from_expr(src, params);
                Self::extract_params_from_expr(dst, params);
                if let Some(rank_expr) = rank {
                    Self::extract_params_from_expr(rank_expr, params);
                }
            }
        }
    }

    /// 从 LOOKUP 语句中提取参数
    fn extract_params_from_lookup(
        lookup_stmt: &LookupStmt,
        params: &mut HashMap<String, DataType>,
    ) {
        if let Some(where_clause) = &lookup_stmt.where_clause {
            Self::extract_params_from_expr(where_clause, params);
        }
        if let Some(yield_clause) = &lookup_stmt.yield_clause {
            for item in &yield_clause.items {
                Self::extract_params_from_expr(&item.expression, params);
            }
        }
    }

    /// 从 FIND PATH 语句中提取参数
    fn extract_params_from_find_path(
        find_path_stmt: &FindPathStmt,
        params: &mut HashMap<String, DataType>,
    ) {
        Self::extract_params_from_expr(&find_path_stmt.to, params);
        if let Some(where_clause) = &find_path_stmt.where_clause {
            Self::extract_params_from_expr(where_clause, params);
        }
        if let Some(yield_clause) = &find_path_stmt.yield_clause {
            for item in &yield_clause.items {
                Self::extract_params_from_expr(&item.expression, params);
            }
        }
    }

    /// 从 MERGE 语句中提取参数
    fn extract_params_from_merge(merge_stmt: &MergeStmt, params: &mut HashMap<String, DataType>) {
        Self::extract_params_from_pattern(&merge_stmt.pattern, params);
        if let Some(on_create) = &merge_stmt.on_create {
            for assignment in &on_create.assignments {
                Self::extract_params_from_expr(&assignment.value, params);
            }
        }
        if let Some(on_match) = &merge_stmt.on_match {
            for assignment in &on_match.assignments {
                Self::extract_params_from_expr(&assignment.value, params);
            }
        }
    }

    /// 从 UNWIND 语句中提取参数
    fn extract_params_from_unwind(
        unwind_stmt: &UnwindStmt,
        params: &mut HashMap<String, DataType>,
    ) {
        Self::extract_params_from_expr(&unwind_stmt.expression, params);
    }

    /// 从 WITH 语句中提取参数
    fn extract_params_from_with(with_stmt: &WithStmt, params: &mut HashMap<String, DataType>) {
        for item in &with_stmt.items {
            match item {
                crate::query::parser::ast::stmt::ReturnItem::Expression { expression, .. } => {
                    Self::extract_params_from_expr(expression, params);
                }
            }
        }
        if let Some(where_clause) = &with_stmt.where_clause {
            Self::extract_params_from_expr(where_clause, params);
        }
    }

    /// 从 YIELD 语句中提取参数
    fn extract_params_from_yield(yield_stmt: &YieldStmt, params: &mut HashMap<String, DataType>) {
        for item in &yield_stmt.items {
            Self::extract_params_from_expr(&item.expression, params);
        }
    }

    /// 从 SET 语句中提取参数
    fn extract_params_from_set(set_stmt: &SetStmt, params: &mut HashMap<String, DataType>) {
        for assignment in &set_stmt.assignments {
            Self::extract_params_from_expr(&assignment.value, params);
        }
    }

    /// 从 REMOVE 语句中提取参数
    fn extract_params_from_remove(
        remove_stmt: &RemoveStmt,
        params: &mut HashMap<String, DataType>,
    ) {
        for item in &remove_stmt.items {
            Self::extract_params_from_expr(item, params);
        }
    }

    /// 从 CREATE 语句中提取参数
    fn extract_params_from_create(create_stmt: &CreateStmt, params: &mut HashMap<String, DataType>) {
        use crate::query::parser::ast::stmt::CreateTarget;

        match &create_stmt.target {
            CreateTarget::Node {
                variable: _,
                labels: _,
                properties,
            } => {
                if let Some(props) = properties {
                    Self::extract_params_from_expr(props, params);
                }
            }
            CreateTarget::Edge {
                variable: _,
                edge_type: _,
                properties,
                src,
                dst,
                direction: _,
            } => {
                if let Some(props) = properties {
                    Self::extract_params_from_expr(props, params);
                }
                Self::extract_params_from_expr(src, params);
                Self::extract_params_from_expr(dst, params);
            }
            CreateTarget::Path { patterns } => {
                for pattern in patterns {
                    Self::extract_params_from_pattern(pattern, params);
                }
            }
            CreateTarget::Tag { .. }
            | CreateTarget::EdgeType { .. }
            | CreateTarget::Space { .. }
            | CreateTarget::Index { .. } => {}
        }
    }

    /// 从 QUERY 语句中提取参数
    fn extract_params_from_query(query_stmt: &QueryStmt, params: &mut HashMap<String, DataType>) {
        for stmt in &query_stmt.statements {
            Self::extract_params_from_stmt(stmt, params);
        }
    }

    /// 从 RETURN 语句中提取参数
    fn extract_params_from_return(
        return_stmt: &ReturnStmt,
        params: &mut HashMap<String, DataType>,
    ) {
        for item in &return_stmt.items {
            match item {
                crate::query::parser::ast::stmt::ReturnItem::Expression { expression, .. } => {
                    Self::extract_params_from_expr(expression, params);
                }
            }
        }
    }

    /// 从 GROUP BY 语句中提取参数
    fn extract_params_from_group_by(
        group_by_stmt: &GroupByStmt,
        params: &mut HashMap<String, DataType>,
    ) {
        for item in &group_by_stmt.group_items {
            Self::extract_params_from_expr(item, params);
        }
        for item in &group_by_stmt.yield_clause.items {
            Self::extract_params_from_expr(&item.expression, params);
        }
        if let Some(having) = &group_by_stmt.having_clause {
            Self::extract_params_from_expr(having, params);
        }
    }

    /// 从 SUBGRAPH 语句中提取参数
    fn extract_params_from_subgraph(
        subgraph_stmt: &SubgraphStmt,
        params: &mut HashMap<String, DataType>,
    ) {
        for expr in &subgraph_stmt.from.vertices {
            Self::extract_params_from_expr(expr, params);
        }
        if let Some(where_clause) = &subgraph_stmt.where_clause {
            Self::extract_params_from_expr(where_clause, params);
        }
        if let Some(yield_clause) = &subgraph_stmt.yield_clause {
            for item in &yield_clause.items {
                Self::extract_params_from_expr(&item.expression, params);
            }
        }
    }

    /// 从 PIPE 语句中提取参数
    fn extract_params_from_pipe(pipe_stmt: &PipeStmt, params: &mut HashMap<String, DataType>) {
        Self::extract_params_from_stmt(&pipe_stmt.left, params);
        Self::extract_params_from_stmt(&pipe_stmt.right, params);
    }

    /// 从模式中递归提取参数
    fn extract_params_from_pattern(pattern: &Pattern, params: &mut HashMap<String, DataType>) {
        match pattern {
            Pattern::Node(node) => {
                if let Some(props) = &node.properties {
                    Self::extract_params_from_expr(props, params);
                }
                for predicate in &node.predicates {
                    Self::extract_params_from_expr(predicate, params);
                }
            }
            Pattern::Edge(edge) => {
                if let Some(props) = &edge.properties {
                    Self::extract_params_from_expr(props, params);
                }
                for predicate in &edge.predicates {
                    Self::extract_params_from_expr(predicate, params);
                }
            }
            Pattern::Path(path) => {
                for element in &path.elements {
                    match element {
                        PathElement::Node(node) => {
                            if let Some(props) = &node.properties {
                                Self::extract_params_from_expr(props, params);
                            }
                            for predicate in &node.predicates {
                                Self::extract_params_from_expr(predicate, params);
                            }
                        }
                        PathElement::Edge(edge) => {
                            if let Some(props) = &edge.properties {
                                Self::extract_params_from_expr(props, params);
                            }
                            for predicate in &edge.predicates {
                                Self::extract_params_from_expr(predicate, params);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    /// 从表达式中递归提取参数
    fn extract_params_from_expr(
        expr: &ContextualExpression,
        params: &mut HashMap<String, DataType>,
    ) {
        let expr_meta = match expr.expression() {
            Some(e) => e,
            None => return,
        };
        let inner_expr = expr_meta.inner();
        match inner_expr {
            Expression::Parameter(name) => {
                if !params.contains_key(name.as_str()) {
                    params.insert(name.clone(), DataType::Empty);
                }
            }
            Expression::Variable(name) => {
                let param_name = if name.starts_with('$') {
                    name.trim_start_matches('$').to_string()
                } else {
                    name.clone()
                };
                if !param_name.is_empty()
                    && (param_name
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_lowercase())
                        || param_name.contains('_'))
                {
                    if !params.contains_key(&param_name) {
                        params.insert(param_name, DataType::Empty);
                    }
                }
            }
            Expression::Binary { left, right, .. } => {
                Self::extract_params_from_expression(left, params);
                Self::extract_params_from_expression(right, params);
            }
            Expression::Unary { operand, .. } => {
                Self::extract_params_from_expression(operand, params);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    Self::extract_params_from_expression(arg, params);
                }
            }
            Expression::Aggregate { arg, .. } => {
                Self::extract_params_from_expression(arg, params);
            }
            Expression::List(items) => {
                for item in items {
                    Self::extract_params_from_expression(item, params);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    Self::extract_params_from_expression(value, params);
                }
            }
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                if let Some(test) = test_expr {
                    Self::extract_params_from_expression(test, params);
                }
                for (cond, value) in conditions {
                    Self::extract_params_from_expression(cond, params);
                    Self::extract_params_from_expression(value, params);
                }
                if let Some(def) = default {
                    Self::extract_params_from_expression(def, params);
                }
            }
            Expression::TypeCast { expression, .. } => {
                Self::extract_params_from_expression(expression, params);
            }
            Expression::Subscript { collection, index } => {
                Self::extract_params_from_expression(collection, params);
                Self::extract_params_from_expression(index, params);
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                Self::extract_params_from_expression(collection, params);
                if let Some(s) = start {
                    Self::extract_params_from_expression(s, params);
                }
                if let Some(e) = end {
                    Self::extract_params_from_expression(e, params);
                }
            }
            Expression::Path(items) => {
                for item in items {
                    Self::extract_params_from_expression(item, params);
                }
            }
            Expression::ListComprehension {
                source,
                filter,
                map,
                ..
            } => {
                Self::extract_params_from_expression(source, params);
                if let Some(f) = filter {
                    Self::extract_params_from_expression(f, params);
                }
                if let Some(m) = map {
                    Self::extract_params_from_expression(m, params);
                }
            }
            Expression::Property { object, .. } => {
                Self::extract_params_from_expression(object, params);
            }
            _ => {}
        }
    }

    /// 从 Expression 提取参数（辅助方法）
    fn extract_params_from_expression(expr: &Expression, params: &mut HashMap<String, DataType>) {
        match expr {
            Expression::Parameter(name) => {
                if !params.contains_key(name.as_str()) {
                    params.insert(name.clone(), DataType::Empty);
                }
            }
            Expression::Variable(name) => {
                let param_name = if name.starts_with('$') {
                    name.trim_start_matches('$').to_string()
                } else {
                    name.clone()
                };
                if !param_name.is_empty()
                    && (param_name
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_lowercase())
                        || param_name.contains('_'))
                {
                    if !params.contains_key(&param_name) {
                        params.insert(param_name, DataType::Empty);
                    }
                }
            }
            Expression::Binary { left, right, .. } => {
                Self::extract_params_from_expression(left, params);
                Self::extract_params_from_expression(right, params);
            }
            Expression::Unary { operand, .. } => {
                Self::extract_params_from_expression(operand, params);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    Self::extract_params_from_expression(arg, params);
                }
            }
            Expression::Aggregate { arg, .. } => {
                Self::extract_params_from_expression(arg, params);
            }
            Expression::List(items) => {
                for item in items {
                    Self::extract_params_from_expression(item, params);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    Self::extract_params_from_expression(value, params);
                }
            }
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                if let Some(test) = test_expr {
                    Self::extract_params_from_expression(test, params);
                }
                for (cond, value) in conditions {
                    Self::extract_params_from_expression(cond, params);
                    Self::extract_params_from_expression(value, params);
                }
                if let Some(def) = default {
                    Self::extract_params_from_expression(def, params);
                }
            }
            Expression::TypeCast { expression, .. } => {
                Self::extract_params_from_expression(expression, params);
            }
            Expression::Subscript { collection, index } => {
                Self::extract_params_from_expression(collection, params);
                Self::extract_params_from_expression(index, params);
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                Self::extract_params_from_expression(collection, params);
                if let Some(s) = start {
                    Self::extract_params_from_expression(s, params);
                }
                if let Some(e) = end {
                    Self::extract_params_from_expression(e, params);
                }
            }
            Expression::Path(items) => {
                for item in items {
                    Self::extract_params_from_expression(item, params);
                }
            }
            Expression::ListComprehension {
                source,
                filter,
                map,
                ..
            } => {
                Self::extract_params_from_expression(source, params);
                if let Some(f) = filter {
                    Self::extract_params_from_expression(f, params);
                }
                if let Some(m) = map {
                    Self::extract_params_from_expression(m, params);
                }
            }
            Expression::Property { object, .. } => {
                Self::extract_params_from_expression(object, params);
            }
            _ => {}
        }
    }

    /// 类型匹配检查
    pub fn type_matches(value: &crate::core::Value, expected_type: &DataType) -> bool {
        use crate::core::Value;
        match (value, expected_type) {
            (Value::Int(_), DataType::Int) => true,
            (Value::Float(_), DataType::Float) => true,
            (Value::String(_), DataType::String) => true,
            (Value::Bool(_), DataType::Bool) => true,
            (Value::Date(_), DataType::Date) => true,
            (Value::DateTime(_), DataType::DateTime) => true,
            (Value::Time(_), DataType::Time) => true,
            (Value::Null(_), _) => true,
            _ => false,
        }
    }
}
