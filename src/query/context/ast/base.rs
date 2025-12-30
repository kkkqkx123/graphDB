//! 基础AST上下文定义

use std::sync::Arc;
use crate::core::context::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::context::validate::types::SpaceInfo;

/// 基础AST上下文
///
/// 提供查询执行过程中的AST节点上下文信息，包括：
/// - QueryContext: 访问运行时资源（元数据管理器、存储客户端等）
/// - Sentence: 关联原始语法树节点，用于错误定位和调试
/// - SpaceInfo: 支持多空间场景
#[derive(Debug, Clone)]
pub struct AstContext {
    /// 查询上下文引用，提供运行时资源访问
    pub qctx: Option<Arc<QueryContext>>,
    
    /// 语法树节点引用，用于错误定位和调试
    pub sentence: Option<Stmt>,
    
    /// 空间信息，支持多空间查询
    pub space: SpaceInfo,
}

impl AstContext {
    /// 创建新的AST上下文
    pub fn new(qctx: Option<Arc<QueryContext>>, sentence: Option<Stmt>) -> Self {
        Self {
            qctx,
            sentence,
            space: SpaceInfo::default(),
        }
    }

    /// 创建新的AST上下文（便捷方法）
    pub fn from_strings(query_type: &str, query_text: &str) -> Self {
        Self {
            qctx: None,
            sentence: None,
            space: SpaceInfo::default(),
        }
    }

    /// 创建带有空间信息的AST上下文
    pub fn with_space(qctx: Option<Arc<QueryContext>>, sentence: Option<Stmt>, space: SpaceInfo) -> Self {
        Self {
            qctx,
            sentence,
            space,
        }
    }

    /// 获取查询上下文
    pub fn query_context(&self) -> Option<&Arc<QueryContext>> {
        self.qctx.as_ref()
    }

    /// 获取语句引用
    pub fn sentence(&self) -> Option<&Stmt> {
        self.sentence.as_ref()
    }

    /// 获取空间信息
    pub fn space(&self) -> &SpaceInfo {
        &self.space
    }

    /// 设置空间信息
    pub fn set_space(&mut self, space: SpaceInfo) {
        self.space = space;
    }

    /// 获取语句类型
    pub fn statement_type(&self) -> &str {
        match &self.sentence {
            Some(stmt) => match stmt {
                Stmt::Query(_) => "QUERY",
                Stmt::Create(_) => "CREATE",
                Stmt::Match(_) => "MATCH",
                Stmt::Delete(_) => "DELETE",
                Stmt::Update(_) => "UPDATE",
                Stmt::Go(_) => "GO",
                Stmt::Fetch(_) => "FETCH",
                Stmt::Use(_) => "USE",
                Stmt::Show(_) => "SHOW",
                Stmt::Explain(_) => "EXPLAIN",
                Stmt::Lookup(_) => "LOOKUP",
                Stmt::Subgraph(_) => "SUBGRAPH",
                Stmt::FindPath(_) => "FIND_PATH",
            },
            None => "UNKNOWN",
        }
    }

    /// 检查是否为路径查询
    pub fn contains_path_query(&self) -> bool {
        matches!(
            &self.sentence,
            Some(Stmt::FindPath(_)) | Some(Stmt::Subgraph(_))
        )
    }

    /// 获取查询文本
    pub fn query_text(&self) -> &str {
        self.qctx
            .as_ref()
            .map(|ctx| ctx.query_text.as_str())
            .unwrap_or("")
    }
}

impl Default for AstContext {
    fn default() -> Self {
        Self {
            qctx: None,
            sentence: None,
            space: SpaceInfo::default(),
        }
    }
}

impl From<(&str, &str)> for AstContext {
    fn from((query_type, query_text): (&str, &str)) -> Self {
        Self {
            qctx: None,
            sentence: None,
            space: SpaceInfo::default(),
        }
    }
}

impl Default for SpaceInfo {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            vid_type: String::new(),
        }
    }
}
