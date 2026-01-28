//! AST 遍历器（AstTraverser）
//! 实现 StmtVisitor trait，用于深度优先遍历语句 AST
//! 收集遍历过程中的信息，支持子类的扩展

use crate::core::Expression;
use crate::query::parser::ast::stmt::{
    Stmt, MatchStmt, DeleteStmt, UpdateStmt, GoStmt, FetchStmt,
    InsertStmt, UseStmt, ShowStmt, CreateStmt, DropStmt, AlterStmt,
    SetStmt, LookupStmt, Assignment, QueryStmt, CreateTarget, DropTarget,
    AlterTarget, ShowTarget, LookupTarget, MergeStmt, UnwindStmt,
    ReturnStmt, WithStmt, RemoveStmt, PipeStmt, DescStmt,
    ExplainStmt, SubgraphStmt, FindPathStmt, ChangePasswordStmt,
};
use crate::query::visitor::stmt_visitor::StmtVisitor;

pub trait AstTraverser: StmtVisitor {
    fn traverse_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Query(s) => self.traverse_query_stmt(s),
            Stmt::Match(s) => self.traverse_match_stmt(s),
            Stmt::Delete(s) => self.traverse_delete_stmt(s),
            Stmt::Update(s) => self.traverse_update_stmt(s),
            Stmt::Go(s) => self.traverse_go_stmt(s),
            Stmt::Fetch(s) => self.traverse_fetch_stmt(s),
            Stmt::Insert(s) => self.traverse_insert_stmt(s),
            Stmt::Use(s) => self.traverse_use_stmt(s),
            Stmt::Show(s) => self.traverse_show_stmt(s),
            Stmt::Create(s) => self.traverse_create_stmt(s),
            Stmt::Drop(s) => self.traverse_drop_stmt(s),
            Stmt::Alter(s) => self.traverse_alter_stmt(s),
            Stmt::Set(s) => self.traverse_set_stmt(s),
            Stmt::Lookup(s) => self.traverse_lookup_stmt(s),
            Stmt::Explain(s) => self.traverse_explain_stmt(s),
            Stmt::Subgraph(s) => self.traverse_subgraph_stmt(s),
            Stmt::FindPath(s) => self.traverse_find_path_stmt(s),
            Stmt::Merge(s) => self.traverse_merge_stmt(s),
            Stmt::Unwind(s) => self.traverse_unwind_stmt(s),
            Stmt::Return(s) => self.traverse_return_stmt(s),
            Stmt::With(s) => self.traverse_with_stmt(s),
            Stmt::Remove(s) => self.traverse_remove_stmt(s),
            Stmt::Pipe(s) => self.traverse_pipe_stmt(s),
            Stmt::Desc(s) => self.traverse_desc_stmt(s),
            Stmt::ChangePassword(s) => self.traverse_change_password_stmt(s),
        }
    }

    fn traverse_match_stmt(&mut self, stmt: &MatchStmt) {
        self.visit_match_stmt(stmt);
    }

    fn traverse_delete_stmt(&mut self, stmt: &DeleteStmt) {
        self.visit_delete_stmt(stmt);
    }

    fn traverse_update_stmt(&mut self, stmt: &UpdateStmt) {
        self.visit_update_stmt(stmt);
    }

    fn traverse_go_stmt(&mut self, stmt: &GoStmt) {
        self.visit_go_stmt(stmt);
    }

    fn traverse_fetch_stmt(&mut self, stmt: &FetchStmt) {
        self.visit_fetch_stmt(stmt);
    }

    fn traverse_insert_stmt(&mut self, stmt: &InsertStmt) {
        self.visit_insert_stmt(stmt);
    }

    fn traverse_use_stmt(&mut self, stmt: &UseStmt) {
        self.visit_use_stmt(stmt);
    }

    fn traverse_show_stmt(&mut self, stmt: &ShowStmt) {
        self.visit_show_stmt(stmt);
    }

    fn traverse_create_stmt(&mut self, stmt: &CreateStmt) {
        self.visit_create_stmt(stmt);
    }

    fn traverse_drop_stmt(&mut self, stmt: &DropStmt) {
        self.visit_drop_stmt(stmt);
    }

    fn traverse_alter_stmt(&mut self, stmt: &AlterStmt) {
        self.visit_alter_stmt(stmt);
    }

    fn traverse_set_stmt(&mut self, stmt: &SetStmt) {
        self.visit_set_stmt(stmt);
    }

    fn traverse_lookup_stmt(&mut self, stmt: &LookupStmt) {
        self.visit_lookup_stmt(stmt);
    }

    fn traverse_query_stmt(&mut self, stmt: &QueryStmt) {
        self.visit_query_stmt(stmt);
    }

    fn traverse_merge_stmt(&mut self, stmt: &MergeStmt) {
        self.visit_merge_stmt(stmt);
    }

    fn traverse_unwind_stmt(&mut self, stmt: &UnwindStmt) {
        self.visit_unwind_stmt(stmt);
    }

    fn traverse_return_stmt(&mut self, stmt: &ReturnStmt) {
        self.visit_return_stmt(stmt);
    }

    fn traverse_with_stmt(&mut self, stmt: &WithStmt) {
        self.visit_with_stmt(stmt);
    }

    fn traverse_remove_stmt(&mut self, stmt: &RemoveStmt) {
        self.visit_remove_stmt(stmt);
    }

    fn traverse_pipe_stmt(&mut self, stmt: &PipeStmt) {
        self.visit_pipe_stmt(stmt);
    }

    fn traverse_desc_stmt(&mut self, stmt: &DescStmt) {
        self.visit_desc_stmt(stmt);
    }

    fn traverse_explain_stmt(&mut self, stmt: &ExplainStmt) {
        self.visit_explain_stmt(stmt);
    }

    fn traverse_subgraph_stmt(&mut self, stmt: &SubgraphStmt) {
        self.visit_subgraph_stmt(stmt);
    }

    fn traverse_find_path_stmt(&mut self, stmt: &FindPathStmt) {
        self.visit_find_path_stmt(stmt);
    }

    fn traverse_change_password_stmt(&mut self, stmt: &ChangePasswordStmt) {
        self.visit_change_password_stmt(stmt);
    }

    fn traverse_assignment(&mut self, assignment: &Assignment) {
        self.visit_assignment(assignment);
    }

    fn collect_expressions(&mut self, exprs: &[Expression]) {
        for expr in exprs {
            self.traverse_expression(expr);
        }
    }

    fn traverse_expression(&mut self, expr: &Expression) {
        use crate::core::types::Expression::*;
        match expr {
            Literal(_) => {}
            Variable(_) => {}
            Property { object, property: _ } => {
                self.traverse_expression(object);
            }
            Binary { left, op: _, right } => {
                self.traverse_expression(left);
                self.traverse_expression(right);
            }
            Unary { op: _, operand } => {
                self.traverse_expression(operand);
            }
            Function { name: _, args } => {
                self.collect_expressions(args);
            }
            Aggregate { func: _, arg, distinct: _ } => {
                self.traverse_expression(arg);
            }
            List(items) => {
                self.collect_expressions(items);
            }
            Map(items) => {
                for (_, value) in items {
                    self.traverse_expression(value);
                }
            }
            Case { conditions, default } => {
                for (when, then) in conditions {
                    self.traverse_expression(when);
                    self.traverse_expression(then);
                }
                if let Some(default) = default {
                    self.traverse_expression(default);
                }
            }
            TypeCast { expression, target_type: _ } => {
                self.traverse_expression(expression);
            }
            Subscript { collection, index } => {
                self.traverse_expression(collection);
                self.traverse_expression(index);
            }
            Range { collection, start, end } => {
                self.traverse_expression(collection);
                if let Some(start) = start {
                    self.traverse_expression(start);
                }
                if let Some(end) = end {
                    self.traverse_expression(end);
                }
            }
            Path(exprs) => {
                self.collect_expressions(exprs);
            }
            Label(_) => {}
            ListComprehension { variable: _, source, filter, map } => {
                self.traverse_expression(source);
                if let Some(filter) = filter {
                    self.traverse_expression(filter);
                }
                if let Some(map) = map {
                    self.traverse_expression(map);
                }
            }
            LabelTagProperty { tag, .. } => {
                self.traverse_expression(tag);
            }
            TagProperty { .. } => {}
            EdgeProperty { .. } => {}
            Predicate { args, .. } => {
                self.collect_expressions(args);
            }
            Reduce { initial, source, mapping, .. } => {
                self.traverse_expression(initial);
                self.traverse_expression(source);
                self.traverse_expression(mapping);
            }
            PathBuild(exprs) => {
                self.collect_expressions(exprs);
            }
        }
    }
}
