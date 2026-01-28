//! 语句转换器（StmtTransformer）
//! 用于转换和修改语句（Stmt）及其子节点
//! 支持深度优先遍历和转换，返回转换后的语句

use crate::core::Expression;
use crate::query::parser::ast::stmt::{
    Stmt, MatchStmt, DeleteStmt, UpdateStmt, GoStmt, FetchStmt,
    InsertStmt, UseStmt, ShowStmt, CreateStmt, DropStmt, AlterStmt,
    SetStmt, LookupStmt, Assignment, QueryStmt, CreateTarget, DropTarget,
    AlterTarget, ShowTarget, LookupTarget, MergeStmt, UnwindStmt,
    ReturnStmt, WithStmt, RemoveStmt, PipeStmt, DescStmt,
    ExplainStmt, SubgraphStmt, FindPathStmt, ChangePasswordStmt,
};

pub trait StmtTransformer {
    type Result;

    fn transform_stmt(&mut self, stmt: &Stmt) -> Self::Result;

    fn transform_match_stmt(&mut self, stmt: &MatchStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Match(stmt.clone()))
    }

    fn transform_delete_stmt(&mut self, stmt: &DeleteStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Delete(stmt.clone()))
    }

    fn transform_update_stmt(&mut self, stmt: &UpdateStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Update(stmt.clone()))
    }

    fn transform_go_stmt(&mut self, stmt: &GoStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Go(stmt.clone()))
    }

    fn transform_fetch_stmt(&mut self, stmt: &FetchStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Fetch(stmt.clone()))
    }

    fn transform_insert_stmt(&mut self, stmt: &InsertStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Insert(stmt.clone()))
    }

    fn transform_use_stmt(&mut self, stmt: &UseStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Use(stmt.clone()))
    }

    fn transform_show_stmt(&mut self, stmt: &ShowStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Show(stmt.clone()))
    }

    fn transform_create_stmt(&mut self, stmt: &CreateStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Create(stmt.clone()))
    }

    fn transform_drop_stmt(&mut self, stmt: &DropStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Drop(stmt.clone()))
    }

    fn transform_alter_stmt(&mut self, stmt: &AlterStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Alter(stmt.clone()))
    }

    fn transform_set_stmt(&mut self, stmt: &SetStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Set(stmt.clone()))
    }

    fn transform_lookup_stmt(&mut self, stmt: &LookupStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Lookup(stmt.clone()))
    }

    fn transform_query_stmt(&mut self, stmt: &QueryStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Query(stmt.clone()))
    }

    fn transform_merge_stmt(&mut self, stmt: &MergeStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Merge(stmt.clone()))
    }

    fn transform_unwind_stmt(&mut self, stmt: &UnwindStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Unwind(stmt.clone()))
    }

    fn transform_return_stmt(&mut self, stmt: &ReturnStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Return(stmt.clone()))
    }

    fn transform_with_stmt(&mut self, stmt: &WithStmt) -> Self::Result {
        self.transform_stmt(&Stmt::With(stmt.clone()))
    }

    fn transform_remove_stmt(&mut self, stmt: &RemoveStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Remove(stmt.clone()))
    }

    fn transform_pipe_stmt(&mut self, stmt: &PipeStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Pipe(stmt.clone()))
    }

    fn transform_desc_stmt(&mut self, stmt: &DescStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Desc(stmt.clone()))
    }

    fn transform_explain_stmt(&mut self, stmt: &ExplainStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Explain(stmt.clone()))
    }

    fn transform_subgraph_stmt(&mut self, stmt: &SubgraphStmt) -> Self::Result {
        self.transform_stmt(&Stmt::Subgraph(stmt.clone()))
    }

    fn transform_find_path_stmt(&mut self, stmt: &FindPathStmt) -> Self::Result {
        self.transform_stmt(&Stmt::FindPath(stmt.clone()))
    }

    fn transform_change_password_stmt(&mut self, stmt: &ChangePasswordStmt) -> Self::Result {
        self.transform_stmt(&Stmt::ChangePassword(stmt.clone()))
    }

    fn transform_assignment(&mut self, assignment: &Assignment) -> Self::Result;
}
