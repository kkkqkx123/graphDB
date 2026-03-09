use crate::core::error::{DBError, DBResult, QueryError};
use crate::core::types::{Index, IndexType};
use crate::query::executor::admin as admin_executor;
use crate::query::executor::admin::edge::create_edge::{CreateEdgeExecutor, ExecutorEdgeInfo};
use crate::query::executor::admin::index::edge_index::CreateEdgeIndexExecutor;
use crate::query::executor::admin::index::tag_index::CreateTagIndexExecutor;
use crate::query::executor::admin::space::create_space::{CreateSpaceExecutor, ExecutorSpaceInfo};
use crate::query::executor::admin::tag::create_tag::{CreateTagExecutor, ExecutorTagInfo};
use crate::query::executor::base::{ExecutionResult, Executor};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::parser::ast::stmt::{AlterStmt, CreateStmt, DescStmt, DropStmt};
use crate::query::planner::planner::{Planner, ValidatedStatement};
use crate::query::planner::statements::create_planner::CreatePlanner;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::validator::ValidationInfo;
use crate::query::QueryContext;
use crate::query::parser::ast::stmt::{CreateTarget, DropTarget, AlterTarget, DescTarget, PropertyChange};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct DDLExecutor<S: StorageClient> {
    id: i64,
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient> DDLExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self { id, storage }
    }

    pub fn execute_create(&self, clause: CreateStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        match clause.target {
            CreateTarget::Tag {
                name,
                properties,
                ttl_duration: _,
                ttl_col: _,
            } => {
                let tag_info = ExecutorTagInfo::new("default".to_string(), name.clone())
                    .with_properties(properties);
                let expr_context = Arc::new(ExpressionAnalysisContext::new());
                let mut executor = if clause.if_not_exists {
                    CreateTagExecutor::with_if_not_exists(
                        self.id,
                        self.storage.clone(),
                        tag_info,
                        expr_context.clone(),
                    )
                } else {
                    CreateTagExecutor::new(
                        self.id,
                        self.storage.clone(),
                        tag_info,
                        expr_context.clone(),
                    )
                };
                Executor::open(&mut executor)?;
                executor.execute()
            }
            CreateTarget::EdgeType {
                name,
                properties,
                ttl_duration: _,
                ttl_col: _,
            } => {
                let edge_info = ExecutorEdgeInfo::new("default".to_string(), name.clone())
                    .with_properties(properties);
                let expr_context = Arc::new(ExpressionAnalysisContext::new());
                let mut executor = if clause.if_not_exists {
                    CreateEdgeExecutor::with_if_not_exists(
                        self.id,
                        self.storage.clone(),
                        edge_info,
                        expr_context.clone(),
                    )
                } else {
                    CreateEdgeExecutor::new(
                        self.id,
                        self.storage.clone(),
                        edge_info,
                        expr_context.clone(),
                    )
                };
                Executor::open(&mut executor)?;
                executor.execute()
            }
            CreateTarget::Space {
                name,
                vid_type,
                comment: _,
            } => {
                let mut space_info = ExecutorSpaceInfo::new(name);
                space_info.vid_type = vid_type;
                let mut executor = if clause.if_not_exists {
                    CreateSpaceExecutor::with_if_not_exists(
                        self.id,
                        self.storage.clone(),
                        space_info,
                        Arc::new(ExpressionAnalysisContext::new()),
                    )
                } else {
                    CreateSpaceExecutor::new(
                        self.id,
                        self.storage.clone(),
                        space_info,
                        Arc::new(ExpressionAnalysisContext::new()),
                    )
                };
                Executor::open(&mut executor)?;
                executor.execute()
            }
            CreateTarget::Index {
                name,
                on,
                properties,
            } => {
                if on.starts_with("tag:") {
                    let tag_name = on.strip_prefix("tag:").unwrap_or(&on);
                    let index_info = Index::new(
                        0,
                        name.clone(),
                        0,
                        tag_name.to_string(),
                        Vec::new(),
                        properties,
                        IndexType::TagIndex,
                        false,
                    );
                    let expr_context = Arc::new(ExpressionAnalysisContext::new());
                    let mut executor = if clause.if_not_exists {
                        CreateTagIndexExecutor::with_if_not_exists(
                            self.id,
                            self.storage.clone(),
                            index_info,
                            expr_context.clone(),
                        )
                    } else {
                        CreateTagIndexExecutor::new(
                            self.id,
                            self.storage.clone(),
                            index_info,
                            expr_context.clone(),
                        )
                    };
                    Executor::open(&mut executor)?;
                    executor.execute()
                } else if on.starts_with("edge:") {
                    let edge_name = on.strip_prefix("edge:").unwrap_or(&on);
                    let index_info = Index::new(
                        0,
                        name.clone(),
                        0,
                        edge_name.to_string(),
                        Vec::new(),
                        properties,
                        IndexType::EdgeIndex,
                        false,
                    );
                    let expr_context = Arc::new(ExpressionAnalysisContext::new());
                    let mut executor = if clause.if_not_exists {
                        CreateEdgeIndexExecutor::with_if_not_exists(
                            self.id,
                            self.storage.clone(),
                            index_info,
                            expr_context.clone(),
                        )
                    } else {
                        CreateEdgeIndexExecutor::new(
                            self.id,
                            self.storage.clone(),
                            index_info,
                            expr_context.clone(),
                        )
                    };
                    Executor::open(&mut executor)?;
                    executor.execute()
                } else {
                    Err(DBError::Query(QueryError::ExecutionError(format!(
                        "Unsupported index target: {}",
                        on
                    ))))
                }
            }
            CreateTarget::Node { .. } | CreateTarget::Edge { .. } | CreateTarget::Path { .. } => {
                let qctx = Arc::new(QueryContext::default());

                let validation_info = ValidationInfo::new();
                let ctx = Arc::new(ExpressionAnalysisContext::new());
                let ast = Arc::new(crate::query::parser::ast::Ast::new(
                    crate::query::parser::ast::stmt::Stmt::Create(clause),
                    ctx,
                ));
                let validated = ValidatedStatement::new(ast, validation_info);

                let mut planner = CreatePlanner::new();
                let plan = planner
                    .transform(&validated, qctx)
                    .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

                let root_node = plan
                    .root()
                    .as_ref()
                    .ok_or_else(|| {
                        DBError::Query(QueryError::ExecutionError("执行计划为空".to_string()))
                    })?
                    .clone();

                let mut executor_factory = ExecutorFactory::with_storage(self.storage.clone());
                let mut executor = executor_factory
                    .create_executor(&root_node, self.storage.clone(), &Default::default())
                    .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

                Executor::open(&mut executor)?;
                let result = Executor::execute(&mut executor)?;
                Executor::close(&mut executor)?;
                Ok(result)
            }
        }
    }

    pub fn execute_drop(&self, clause: DropStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        match clause.target {
            DropTarget::Space(space_name) => {
                let mut executor = admin_executor::DropSpaceExecutor::new(
                    self.id,
                    self.storage.clone(),
                    space_name,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
            DropTarget::Tags(tag_names) => {
                for tag_name in &tag_names {
                    let mut executor = admin_executor::DropTagExecutor::new(
                        self.id,
                        self.storage.clone(),
                        String::new(),
                        tag_name.clone(),
                        Arc::new(ExpressionAnalysisContext::new()),
                    );
                    if let Err(e) = Executor::open(&mut executor) {
                        return Err(DBError::Query(QueryError::ExecutionError(format!(
                            "删除标签 {} 失败: {}",
                            tag_name, e
                        ))));
                    }
                }

                let mut total_dropped = 0;
                for tag_name in tag_names {
                    let mut executor = admin_executor::DropTagExecutor::new(
                        self.id,
                        self.storage.clone(),
                        String::new(),
                        tag_name.clone(),
                        Arc::new(ExpressionAnalysisContext::new()),
                    );
                    match Executor::execute(&mut executor) {
                        Ok(_) => total_dropped += 1,
                        Err(e) => {
                            return Err(DBError::Query(QueryError::ExecutionError(format!(
                                "删除标签 {} 时发生错误: {}",
                                tag_name, e
                            ))));
                        }
                    }
                }

                Ok(ExecutionResult::Count(total_dropped))
            }
            DropTarget::Edges(edge_names) => {
                for edge_name in &edge_names {
                    let mut executor = admin_executor::DropEdgeExecutor::new(
                        self.id,
                        self.storage.clone(),
                        String::new(),
                        edge_name.clone(),
                        Arc::new(ExpressionAnalysisContext::new()),
                    );
                    if let Err(e) = Executor::open(&mut executor) {
                        return Err(DBError::Query(QueryError::ExecutionError(format!(
                            "删除边类型 {} 失败: {}",
                            edge_name, e
                        ))));
                    }
                }

                let mut total_dropped = 0;
                for edge_name in edge_names {
                    let mut executor = admin_executor::DropEdgeExecutor::new(
                        self.id,
                        self.storage.clone(),
                        String::new(),
                        edge_name.clone(),
                        Arc::new(ExpressionAnalysisContext::new()),
                    );
                    match Executor::execute(&mut executor) {
                        Ok(_) => total_dropped += 1,
                        Err(e) => {
                            return Err(DBError::Query(QueryError::ExecutionError(format!(
                                "删除边类型 {} 时发生错误: {}",
                                edge_name, e
                            ))));
                        }
                    }
                }

                Ok(ExecutionResult::Count(total_dropped))
            }
            DropTarget::TagIndex {
                space_name,
                index_name,
            } => {
                let mut executor = admin_executor::DropTagIndexExecutor::new(
                    self.id,
                    self.storage.clone(),
                    space_name,
                    index_name,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
            DropTarget::EdgeIndex {
                space_name,
                index_name,
            } => {
                let mut executor = admin_executor::DropEdgeIndexExecutor::new(
                    self.id,
                    self.storage.clone(),
                    space_name,
                    index_name,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
        }
    }

    pub fn execute_desc(&self, clause: DescStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        match clause.target {
            DescTarget::Space(space_name) => {
                let mut executor = admin_executor::DescSpaceExecutor::new(
                    self.id,
                    self.storage.clone(),
                    space_name,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
            DescTarget::Tag {
                space_name,
                tag_name,
            } => {
                let mut executor = admin_executor::DescTagExecutor::new(
                    self.id,
                    self.storage.clone(),
                    space_name,
                    tag_name,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
            DescTarget::Edge {
                space_name,
                edge_name,
            } => {
                let mut executor = admin_executor::DescEdgeExecutor::new(
                    self.id,
                    self.storage.clone(),
                    space_name,
                    edge_name,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
        }
    }

    pub fn execute_alter(&self, clause: AlterStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use admin_executor::{
            AlterEdgeExecutor, AlterEdgeInfo, AlterEdgeItem, AlterTagExecutor, AlterTagInfo,
            AlterTagItem,
        };
        
        match clause.target {
            AlterTarget::Tag {
                tag_name,
                additions,
                deletions: _,
                changes: _,
            } => {
                let mut items = Vec::new();
                for prop in additions {
                    items.push(AlterTagItem::add_property(prop));
                }
                let alter_info = AlterTagInfo::new(String::new(), tag_name).with_items(items);
                let mut executor = AlterTagExecutor::new(
                    self.id,
                    self.storage.clone(),
                    alter_info,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
            AlterTarget::Edge {
                edge_name,
                additions,
                deletions: _,
                changes: _,
            } => {
                let mut items = Vec::new();
                for prop in additions {
                    items.push(AlterEdgeItem::add_property(prop));
                }
                let alter_info = AlterEdgeInfo::new(String::new(), edge_name).with_items(items);
                let mut executor = AlterEdgeExecutor::new(
                    self.id,
                    self.storage.clone(),
                    alter_info,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
            AlterTarget::Space {
                space_name,
                comment,
            } => {
                let mut options = Vec::new();
                if let Some(comment_str) = comment {
                    options.push(crate::query::executor::admin::space::alter_space::SpaceAlterOption::Comment(comment_str));
                }
                let mut executor = admin_executor::AlterSpaceExecutor::new(
                    self.id,
                    self.storage.clone(),
                    space_name,
                    options,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
        }
    }
}
