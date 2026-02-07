//! 语句访问器（StmtVisitor）
//! 用于遍历和访问语句（Stmt）及其子节点
//! 提供统一的语句遍历接口，支持深度优先遍历

use crate::query::parser::ast::stmt::{
    Stmt, MatchStmt, DeleteStmt, UpdateStmt, GoStmt, FetchStmt,
    InsertStmt, UseStmt, ShowStmt, CreateStmt, DropStmt, AlterStmt,
    SetStmt, LookupStmt, Assignment, QueryStmt, MergeStmt, UnwindStmt,
    ReturnStmt, WithStmt, RemoveStmt, PipeStmt, DescStmt,
    ExplainStmt, SubgraphStmt, FindPathStmt, ChangePasswordStmt,
    CreateUserStmt, AlterUserStmt, DropUserStmt,
};

pub trait StmtVisitor {
    type Result;

    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::Result;

    fn visit_match_stmt(&mut self, stmt: &MatchStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Match(stmt.clone()))
    }

    fn visit_delete_stmt(&mut self, stmt: &DeleteStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Delete(stmt.clone()))
    }

    fn visit_update_stmt(&mut self, stmt: &UpdateStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Update(stmt.clone()))
    }

    fn visit_go_stmt(&mut self, stmt: &GoStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Go(stmt.clone()))
    }

    fn visit_fetch_stmt(&mut self, stmt: &FetchStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Fetch(stmt.clone()))
    }

    fn visit_insert_stmt(&mut self, stmt: &InsertStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Insert(stmt.clone()))
    }

    fn visit_use_stmt(&mut self, stmt: &UseStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Use(stmt.clone()))
    }

    fn visit_show_stmt(&mut self, stmt: &ShowStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Show(stmt.clone()))
    }

    fn visit_create_stmt(&mut self, stmt: &CreateStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Create(stmt.clone()))
    }

    fn visit_drop_stmt(&mut self, stmt: &DropStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Drop(stmt.clone()))
    }

    fn visit_alter_stmt(&mut self, stmt: &AlterStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Alter(stmt.clone()))
    }

    fn visit_set_stmt(&mut self, stmt: &SetStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Set(stmt.clone()))
    }

    fn visit_lookup_stmt(&mut self, stmt: &LookupStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Lookup(stmt.clone()))
    }

    fn visit_query_stmt(&mut self, stmt: &QueryStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Query(stmt.clone()))
    }

    fn visit_merge_stmt(&mut self, stmt: &MergeStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Merge(stmt.clone()))
    }

    fn visit_unwind_stmt(&mut self, stmt: &UnwindStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Unwind(stmt.clone()))
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Return(stmt.clone()))
    }

    fn visit_with_stmt(&mut self, stmt: &WithStmt) -> Self::Result {
        self.visit_stmt(&Stmt::With(stmt.clone()))
    }

    fn visit_remove_stmt(&mut self, stmt: &RemoveStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Remove(stmt.clone()))
    }

    fn visit_pipe_stmt(&mut self, stmt: &PipeStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Pipe(stmt.clone()))
    }

    fn visit_desc_stmt(&mut self, stmt: &DescStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Desc(stmt.clone()))
    }

    fn visit_explain_stmt(&mut self, stmt: &ExplainStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Explain(stmt.clone()))
    }

    fn visit_subgraph_stmt(&mut self, stmt: &SubgraphStmt) -> Self::Result {
        self.visit_stmt(&Stmt::Subgraph(stmt.clone()))
    }

    fn visit_find_path_stmt(&mut self, stmt: &FindPathStmt) -> Self::Result {
        self.visit_stmt(&Stmt::FindPath(stmt.clone()))
    }

    fn visit_change_password_stmt(&mut self, stmt: &ChangePasswordStmt) -> Self::Result {
        self.visit_stmt(&Stmt::ChangePassword(stmt.clone()))
    }

    fn visit_create_user_stmt(&mut self, stmt: &CreateUserStmt) -> Self::Result {
        self.visit_stmt(&Stmt::CreateUser(stmt.clone()))
    }

    fn visit_alter_user_stmt(&mut self, stmt: &AlterUserStmt) -> Self::Result {
        self.visit_stmt(&Stmt::AlterUser(stmt.clone()))
    }

    fn visit_drop_user_stmt(&mut self, stmt: &DropUserStmt) -> Self::Result {
        self.visit_stmt(&Stmt::DropUser(stmt.clone()))
    }

    fn visit_assignment(&mut self, assignment: &Assignment) -> Self::Result;
}
