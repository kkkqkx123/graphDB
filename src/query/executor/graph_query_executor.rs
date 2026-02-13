//! 图查询执行器
//!
//! 提供图查询语言（Cypher/NGQL）的执行功能
//! 支持MATCH、CREATE、DELETE等图操作语句

use crate::core::error::{DBError, DBResult, QueryError};
use crate::query::context::ast::AstContext;
use crate::query::executor::admin as admin_executor;
use crate::query::executor::factory::ExecutorFactory;
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::query::parser::ast::stmt::{AlterStmt, ChangePasswordStmt, CreateUserStmt, AlterUserStmt, DropUserStmt, DescStmt, DropStmt, Stmt};
use crate::core::types::metadata::{UserAlterInfo, UserInfo};
use crate::query::planner::planner::Planner;
use crate::query::planner::statements::match_statement_planner::MatchStatementPlanner;
use crate::storage::StorageClient;
use std::sync::{Arc, Mutex};

/// 图查询执行器
///
/// 提供图查询语言（Cypher/NGQL）的执行功能
/// 支持MATCH、CREATE、DELETE等图操作语句
pub struct GraphQueryExecutor<S: StorageClient> {
    /// 执行器ID
    id: i64,
    /// 执行器名称
    name: String,
    /// 执行器描述
    description: String,
    /// 存储引擎引用
    storage: Arc<Mutex<S>>,
    /// 是否已打开
    is_open: bool,
    /// 执行统计信息
    stats: crate::query::executor::traits::ExecutorStats,
}

impl<S: StorageClient> std::fmt::Debug for GraphQueryExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphQueryExecutor")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("is_open", &self.is_open)
            .field("stats", &self.stats)
            .finish()
    }
}

impl<S: StorageClient + 'static> GraphQueryExecutor<S> {
    /// 创建新的图查询执行器
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name: "GraphQueryExecutor".to_string(),
            description: "图查询语言执行器".to_string(),
            storage,
            is_open: false,
            stats: crate::query::executor::traits::ExecutorStats::new(),
        }
    }

    /// 带名称创建执行器
    pub fn with_name(id: i64, name: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name,
            description: "图查询语言执行器".to_string(),
            storage,
            is_open: false,
            stats: crate::query::executor::traits::ExecutorStats::new(),
        }
    }

    /// 带名称和描述创建执行器
    pub fn with_description(
        id: i64,
        name: String,
        description: String,
        storage: Arc<Mutex<S>>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            storage,
            is_open: false,
            stats: crate::query::executor::traits::ExecutorStats::new(),
        }
    }

    /// 执行具体的语句
    #[allow(dead_code)]
    fn execute_statement(
        &mut self,
        statement: Stmt,
    ) -> Result<ExecutionResult, DBError> {
        match statement {
            Stmt::Match(clause) => self.execute_match(clause),
            Stmt::Create(clause) => self.execute_create(clause),
            Stmt::Delete(clause) => self.execute_delete(clause),
            Stmt::Update(clause) => self.execute_update(clause),
            Stmt::Query(clause) => self.execute_query(clause),
            Stmt::Go(clause) => self.execute_go(clause),
            Stmt::Fetch(clause) => self.execute_fetch(clause),
            Stmt::Lookup(clause) => self.execute_lookup(clause),
            Stmt::FindPath(clause) => self.execute_find_path(clause),
            Stmt::Use(clause) => self.execute_use(clause),
            Stmt::Show(clause) => self.execute_show(clause),
            Stmt::Explain(clause) => self.execute_explain(clause),
            Stmt::Subgraph(clause) => self.execute_subgraph(clause),
            Stmt::Insert(clause) => self.execute_insert(clause),
            Stmt::Merge(clause) => self.execute_merge(clause),
            Stmt::Unwind(clause) => self.execute_unwind(clause),
            Stmt::Return(clause) => self.execute_return(clause),
            Stmt::With(clause) => self.execute_with(clause),
            Stmt::Set(clause) => self.execute_set(clause),
            Stmt::Remove(clause) => self.execute_remove(clause),
            Stmt::Pipe(clause) => self.execute_pipe(clause),
            Stmt::Drop(clause) => self.execute_drop(clause),
            Stmt::Desc(clause) => self.execute_desc(clause),
            Stmt::Alter(clause) => self.execute_alter(clause),
            Stmt::CreateUser(clause) => self.execute_create_user(clause),
            Stmt::AlterUser(clause) => self.execute_alter_user(clause),
            Stmt::DropUser(clause) => self.execute_drop_user(clause),
            Stmt::ChangePassword(clause) => self.execute_change_password(clause),
        }
    }

    fn execute_match(&mut self, clause: crate::query::parser::ast::stmt::MatchStmt) -> Result<ExecutionResult, DBError> {
        let _id = self.id;

        let mut ast_ctx = AstContext::new(None, Some(Stmt::Match(clause)));
        ast_ctx.set_query_type(crate::query::context::ast::QueryType::ReadQuery);

        let mut planner = MatchStatementPlanner::new();
        let plan = planner.transform(&ast_ctx)
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        let root_node = plan.root()
            .as_ref()
            .ok_or_else(|| DBError::Query(QueryError::ExecutionError("执行计划为空".to_string())))?
            .clone();

        let mut executor_factory = ExecutorFactory::with_storage(self.storage.clone());
        let mut executor = executor_factory.create_executor(&root_node, self.storage.clone(), &Default::default())
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        executor.open()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        let result = executor.execute()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        executor.close()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        Ok(result)
    }

    fn execute_create(&mut self, clause: crate::query::parser::ast::stmt::CreateStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::parser::ast::stmt::CreateTarget;
        use crate::query::executor::admin::tag::create_tag::{CreateTagExecutor, ExecutorTagInfo};
        use crate::query::executor::admin::edge::create_edge::{CreateEdgeExecutor, ExecutorEdgeInfo};
        use crate::query::executor::admin::space::create_space::{CreateSpaceExecutor, ExecutorSpaceInfo};
        use crate::query::executor::admin::index::tag_index::CreateTagIndexExecutor;
        use crate::query::executor::admin::index::edge_index::CreateEdgeIndexExecutor;
        use crate::index::{Index, IndexType};

        match clause.target {
            CreateTarget::Tag { name, properties } => {
                let tag_info = ExecutorTagInfo::new("default".to_string(), name.clone())
                    .with_properties(properties);
                let mut executor = if clause.if_not_exists {
                    CreateTagExecutor::with_if_not_exists(self.id, self.storage.clone(), tag_info)
                } else {
                    CreateTagExecutor::new(self.id, self.storage.clone(), tag_info)
                };
                executor.open()?;
                executor.execute()
            }
            CreateTarget::EdgeType { name, properties } => {
                let edge_info = ExecutorEdgeInfo::new("default".to_string(), name.clone())
                    .with_properties(properties);
                let mut executor = if clause.if_not_exists {
                    CreateEdgeExecutor::with_if_not_exists(self.id, self.storage.clone(), edge_info)
                } else {
                    CreateEdgeExecutor::new(self.id, self.storage.clone(), edge_info)
                };
                executor.open()?;
                executor.execute()
            }
            CreateTarget::Space { name } => {
                let space_info = ExecutorSpaceInfo::new(name);
                let mut executor = if clause.if_not_exists {
                    CreateSpaceExecutor::with_if_not_exists(self.id, self.storage.clone(), space_info)
                } else {
                    CreateSpaceExecutor::new(self.id, self.storage.clone(), space_info)
                };
                executor.open()?;
                executor.execute()
            }
            CreateTarget::Index { name, on, properties } => {
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
                    let mut executor = if clause.if_not_exists {
                        CreateTagIndexExecutor::with_if_not_exists(self.id, self.storage.clone(), index_info)
                    } else {
                        CreateTagIndexExecutor::new(self.id, self.storage.clone(), index_info)
                    };
                    executor.open()?;
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
                    let mut executor = if clause.if_not_exists {
                        CreateEdgeIndexExecutor::with_if_not_exists(self.id, self.storage.clone(), index_info)
                    } else {
                        CreateEdgeIndexExecutor::new(self.id, self.storage.clone(), index_info)
                    };
                    executor.open()?;
                    executor.execute()
                } else {
                    Err(DBError::Query(QueryError::ExecutionError(
                        format!("Unsupported index target: {}", on)
                    )))
                }
            }
            CreateTarget::Node { .. } | CreateTarget::Edge { .. } => {
                Err(DBError::Query(QueryError::ExecutionError(
                    "CREATE NODE/EDGE for MATCH pattern is not implemented yet".to_string()
                )))
            }
        }
    }

    fn execute_delete(&mut self, clause: crate::query::parser::ast::stmt::DeleteStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::executor::data_modification::DeleteExecutor;
        use crate::query::parser::ast::stmt::DeleteTarget;
        use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
        use crate::expression::DefaultExpressionContext;

        match clause.target {
            DeleteTarget::Vertices(vertex_exprs) => {
                let mut vertex_ids = Vec::new();
                for expr in vertex_exprs {
                    let mut context = DefaultExpressionContext::new();
                    let vid = ExpressionEvaluator::evaluate(&expr, &mut context)
                        .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("顶点ID求值失败: {}", e))))?;
                    vertex_ids.push(vid);
                }

                let mut executor = DeleteExecutor::new(
                    self.id,
                    self.storage.clone(),
                    Some(vertex_ids),
                    None,
                    None,
                );
                executor.open()?;
                executor.execute()
            }
            DeleteTarget::Edges { edge_type, edges } => {
                let mut edge_ids = Vec::new();
                for (src_expr, dst_expr, rank_expr) in edges {
                    let mut src_context = DefaultExpressionContext::new();
                    let src = ExpressionEvaluator::evaluate(&src_expr, &mut src_context)
                        .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("源顶点ID求值失败: {}", e))))?;

                    let mut dst_context = DefaultExpressionContext::new();
                    let dst = ExpressionEvaluator::evaluate(&dst_expr, &mut dst_context)
                        .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("目标顶点ID求值失败: {}", e))))?;

                    let _rank = match rank_expr {
                        Some(ref r) => {
                            let mut rank_context = DefaultExpressionContext::new();
                            let rank_val = ExpressionEvaluator::evaluate(r, &mut rank_context)
                                .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("rank求值失败: {}", e))))?;
                            match rank_val {
                                crate::core::Value::Int(i) => Some(i),
                                _ => return Err(DBError::Query(QueryError::ExecutionError("rank必须是整数".to_string()))),
                            }
                        }
                        None => None,
                    };
                    let edge_type_str = edge_type.clone().unwrap_or_default();
                    edge_ids.push((src, dst, edge_type_str));
                }

                let mut executor = DeleteExecutor::new(
                    self.id,
                    self.storage.clone(),
                    None,
                    Some(edge_ids),
                    None,
                );
                executor.open()?;
                executor.execute()
            }
            _ => Err(DBError::Query(QueryError::ExecutionError(
                format!("DELETE {:?} 未实现", clause.target)
            )))
        }
    }

    fn execute_update(&mut self, clause: crate::query::parser::ast::stmt::UpdateStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::executor::data_modification::{UpdateExecutor, VertexUpdate, EdgeUpdate};
        use crate::query::parser::ast::stmt::UpdateTarget;
        use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
        use crate::expression::DefaultExpressionContext;

        match clause.target {
            UpdateTarget::Vertex(vid_expr) => {
                let mut context = DefaultExpressionContext::new();
                let vid = ExpressionEvaluator::evaluate(&vid_expr, &mut context)
                    .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("顶点ID求值失败: {}", e))))?;

                let mut properties = std::collections::HashMap::new();
                for assignment in &clause.set_clause.assignments {
                    let mut prop_context = DefaultExpressionContext::new();
                    let value = ExpressionEvaluator::evaluate(&assignment.value, &mut prop_context)
                        .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("属性值求值失败: {}", e))))?;
                    properties.insert(assignment.property.clone(), value);
                }

                let vertex_updates = vec![VertexUpdate {
                    vertex_id: vid,
                    properties,
                    tags_to_add: None,
                    tags_to_remove: None,
                }];

                let mut executor = UpdateExecutor::new(
                    self.id,
                    self.storage.clone(),
                    Some(vertex_updates),
                    None,
                    None,
                )
                .with_insertable(false)
                .with_space("default".to_string());

                executor.open()?;
                executor.execute()
            }
            UpdateTarget::Edge { src, dst, edge_type, rank } => {
                let mut src_context = DefaultExpressionContext::new();
                let src_val = ExpressionEvaluator::evaluate(&src, &mut src_context)
                    .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("源顶点ID求值失败: {}", e))))?;

                let mut dst_context = DefaultExpressionContext::new();
                let dst_val = ExpressionEvaluator::evaluate(&dst, &mut dst_context)
                    .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("目标顶点ID求值失败: {}", e))))?;

                let rank_val = match rank {
                    Some(ref r) => {
                        let mut rank_context = DefaultExpressionContext::new();
                        let rank_val = ExpressionEvaluator::evaluate(r, &mut rank_context)
                            .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("rank求值失败: {}", e))))?;
                        match rank_val {
                            crate::core::Value::Int(i) => Some(i),
                            _ => return Err(DBError::Query(QueryError::ExecutionError("rank必须是整数".to_string()))),
                        }
                    }
                    None => None,
                };

                let mut properties = std::collections::HashMap::new();
                for assignment in &clause.set_clause.assignments {
                    let mut prop_context = DefaultExpressionContext::new();
                    let value = ExpressionEvaluator::evaluate(&assignment.value, &mut prop_context)
                        .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("属性值求值失败: {}", e))))?;
                    properties.insert(assignment.property.clone(), value);
                }

                let edge_updates = vec![EdgeUpdate {
                    src: src_val,
                    dst: dst_val,
                    edge_type: edge_type.unwrap_or_default(),
                    rank: rank_val,
                    properties,
                }];

                let mut executor = UpdateExecutor::new(
                    self.id,
                    self.storage.clone(),
                    None,
                    Some(edge_updates),
                    None,
                )
                .with_insertable(false)
                .with_space("default".to_string());

                executor.open()?;
                executor.execute()
            }
            _ => Err(DBError::Query(QueryError::ExecutionError(
                format!("UPDATE {:?} 未实现", clause.target)
            )))
        }
    }

    #[allow(dead_code)]
    fn execute_query(&mut self, _clause: crate::query::parser::ast::stmt::QueryStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("QUERY语句执行未实现".to_string())))
    }

    #[allow(dead_code)]
    fn execute_go(&mut self, _clause: crate::query::parser::ast::stmt::GoStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("GO语句执行未实现".to_string())))
    }

    fn execute_fetch(&mut self, clause: crate::query::parser::ast::stmt::FetchStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::parser::ast::stmt::FetchTarget;
        use crate::query::executor::data_access::GetVerticesExecutor;
        use crate::query::executor::data_access::GetEdgesExecutor;
        use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
        use crate::expression::DefaultExpressionContext;

        match clause.target {
            FetchTarget::Vertices { ids, properties: _ } => {
                let mut vertex_ids = Vec::new();
                for expr in ids {
                    let mut context = DefaultExpressionContext::new();
                    let vid = ExpressionEvaluator::evaluate(&expr, &mut context)
                        .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("顶点ID求值失败: {}", e))))?;
                    vertex_ids.push(vid);
                }

                let mut executor = GetVerticesExecutor::new(
                    self.id,
                    self.storage.clone(),
                    Some(vertex_ids),
                    None,
                    None,
                    None,
                );
                executor.open()?;
                executor.execute()
            }
            FetchTarget::Edges { src: _, dst: _, edge_type, rank: _, properties: _ } => {
                let mut executor = GetEdgesExecutor::new(
                    self.id,
                    self.storage.clone(),
                    Some(edge_type),
                );
                executor.open()?;
                executor.execute()
            }
        }
    }

    fn execute_lookup(&mut self, clause: crate::query::parser::ast::stmt::LookupStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::parser::ast::stmt::LookupTarget;
        use crate::query::executor::data_access::IndexScanExecutor;

        match clause.target {
            LookupTarget::Tag(tag_name) => {
                let mut executor = IndexScanExecutor::new(
                    self.id,
                    self.storage.clone(),
                    format!("idx_{}", tag_name),
                    None,
                    true,
                    None,
                );
                executor.open()?;
                executor.execute()
            }
            LookupTarget::Edge(edge_name) => {
                Err(DBError::Query(QueryError::ExecutionError(
                    format!("LOOKUP ON EDGE {} 未实现", edge_name)
                )))
            }
        }
    }

    fn execute_find_path(&mut self, clause: crate::query::parser::ast::stmt::FindPathStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::executor::data_processing::graph_traversal::AllPathsExecutor;
        use crate::query::executor::base::EdgeDirection;
        use crate::core::Value;

        let storage = self.storage.clone();

        // 解析起点和终点
        let left_start_ids: Vec<Value> = clause.from.vertices.iter()
            .map(|expr| match expr {
                crate::core::types::expression::Expression::Literal(Value::Int(n)) => Value::Int(*n),
                crate::core::types::expression::Expression::Literal(Value::String(s)) => Value::String(s.clone()),
                _ => Value::Null(crate::core::NullType::default()),
            })
            .collect();

        let right_start_ids: Vec<Value> = vec![match &clause.to {
            crate::core::types::expression::Expression::Literal(Value::Int(n)) => Value::Int(*n),
            crate::core::types::expression::Expression::Literal(Value::String(s)) => Value::String(s.clone()),
            _ => Value::Null(crate::core::NullType::default()),
        }];

        // 解析边方向
        let edge_direction = if let Some(ref over) = clause.over {
            match over.direction {
                crate::query::parser::ast::types::EdgeDirection::Out => EdgeDirection::Out,
                crate::query::parser::ast::types::EdgeDirection::In => EdgeDirection::In,
                crate::query::parser::ast::types::EdgeDirection::Both => EdgeDirection::Both,
            }
        } else {
            EdgeDirection::Both
        };

        // 解析边类型
        let edge_types = clause.over.as_ref().map(|over| over.edge_types.clone());

        // 解析最大步数
        let max_steps = clause.max_steps.unwrap_or(5);

        // 解析 limit 和 offset
        let limit = clause.limit.unwrap_or(std::usize::MAX);
        let offset = clause.offset.unwrap_or(0);

        // 创建执行器
        let mut executor = AllPathsExecutor::new(
            0,
            storage,
            left_start_ids,
            right_start_ids,
            edge_direction,
            edge_types,
            max_steps,
        ).with_config(
            false, // with_prop
            limit,
            offset,
        );

        // 执行查询
        match executor.execute() {
            Ok(_paths) => {
                // 转换为 ExecutionResult
                let core_result = crate::core::result::Result::empty(vec!["path".to_string()]);
                let result = ExecutionResult::from_result(core_result);
                Ok(result)
            }
            Err(e) => Err(DBError::Query(QueryError::ExecutionError(format!("FIND PATH执行失败: {:?}", e)))),
        }
    }

    fn execute_use(&mut self, clause: crate::query::parser::ast::stmt::UseStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::executor::admin::space::switch_space::SwitchSpaceExecutor;

        let mut executor = SwitchSpaceExecutor::new(
            self.id,
            self.storage.clone(),
            clause.space,
        );
        executor.open()?;
        executor.execute()
    }

    fn execute_show(&mut self, clause: crate::query::parser::ast::stmt::ShowStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::parser::ast::stmt::ShowTarget;

        match clause.target {
            ShowTarget::Spaces => {
                use crate::query::executor::admin::space::show_spaces::ShowSpacesExecutor;
                let mut executor = ShowSpacesExecutor::new(self.id, self.storage.clone());
                executor.open()?;
                executor.execute()
            }
            ShowTarget::Tags => {
                use crate::query::executor::admin::tag::show_tags::ShowTagsExecutor;
                let mut executor = ShowTagsExecutor::new(self.id, self.storage.clone(), String::new());
                executor.open()?;
                executor.execute()
            }
            ShowTarget::Edges => {
                use crate::query::executor::admin::edge::show_edges::ShowEdgesExecutor;
                let mut executor = ShowEdgesExecutor::new(self.id, self.storage.clone(), String::new());
                executor.open()?;
                executor.execute()
            }
            ShowTarget::Tag(tag_name) => {
                use crate::query::executor::admin::tag::desc_tag::DescTagExecutor;
                let mut executor = DescTagExecutor::new(self.id, self.storage.clone(), String::new(), tag_name);
                executor.open()?;
                executor.execute()
            }
            ShowTarget::Edge(edge_name) => {
                use crate::query::executor::admin::edge::desc_edge::DescEdgeExecutor;
                let mut executor = DescEdgeExecutor::new(self.id, self.storage.clone(), String::new(), edge_name);
                executor.open()?;
                executor.execute()
            }
            ShowTarget::Indexes => {
                use crate::query::executor::admin::index::ShowTagIndexesExecutor;
                use crate::query::executor::admin::index::ShowEdgeIndexesExecutor;
                let mut tag_executor = ShowTagIndexesExecutor::new(self.id, self.storage.clone(), String::new());
                tag_executor.open()?;
                let tag_result = tag_executor.execute();
                
                let mut edge_executor = ShowEdgeIndexesExecutor::new(self.id, self.storage.clone(), String::new());
                edge_executor.open()?;
                let edge_result = edge_executor.execute();
                
                match (tag_result, edge_result) {
                    (Ok(ExecutionResult::DataSet(mut tag_dataset)), Ok(ExecutionResult::DataSet(edge_dataset))) => {
                        tag_dataset.rows.extend(edge_dataset.rows);
                        Ok(ExecutionResult::DataSet(tag_dataset))
                    }
                    _ => Err(DBError::Query(QueryError::ExecutionError(
                        "SHOW INDEXES 执行失败".to_string()
                    )))
                }
            }
            ShowTarget::Index(index_name) => {
                Err(DBError::Query(QueryError::ExecutionError(
                    format!("SHOW INDEX {} 未实现", index_name)
                )))
            }
            ShowTarget::Users => {
                Err(DBError::Query(QueryError::ExecutionError(
                    "SHOW USERS 未实现".to_string()
                )))
            }
            ShowTarget::Roles => {
                Err(DBError::Query(QueryError::ExecutionError(
                    "SHOW ROLES 未实现".to_string()
                )))
            }
        }
    }

    fn execute_explain(&mut self, clause: crate::query::parser::ast::stmt::ExplainStmt) -> Result<ExecutionResult, DBError> {
        use crate::core::result::Result as CoreResult;
        use crate::core::Value as CoreValue;

        let query_str = format!("{:?}", clause.statement);

        let plan = vec![
            format!("Query: {}", query_str),
            "Execution Plan:".to_string(),
            "  1. Parse Query".to_string(),
            "  2. Validate AST".to_string(),
            "  3. Generate Execution Plan".to_string(),
            "  4. Execute Query".to_string(),
        ];

        let rows = plan.into_iter().map(|s| vec![CoreValue::String(s)]).collect();
        let core_result = CoreResult::from_rows(rows, vec!["plan".to_string()]);
        Ok(ExecutionResult::from_result(core_result))
    }

    #[allow(dead_code)]
    fn execute_subgraph(&mut self, _clause: crate::query::parser::ast::stmt::SubgraphStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("SUBGRAPH语句执行未实现".to_string())))
    }

    fn execute_insert(&mut self, clause: crate::query::parser::ast::stmt::InsertStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::executor::data_modification::InsertExecutor;
        use crate::query::parser::ast::stmt::InsertTarget;
        use crate::core::Vertex;
        use crate::core::Edge;
        use crate::core::vertex_edge_path::Tag;
        use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
        use crate::expression::DefaultExpressionContext;

        match clause.target {
            InsertTarget::Vertices { tag_name, prop_names, values } => {
                let mut vertices = Vec::new();

                for (vid_expr, prop_values) in values {
                    let mut context = DefaultExpressionContext::new();
                    let vid = ExpressionEvaluator::evaluate(&vid_expr, &mut context)
                        .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("表达式求值失败: {}", e))))?;

                    let mut properties = std::collections::HashMap::new();
                    for (i, prop_name) in prop_names.iter().enumerate() {
                        if i < prop_values.len() {
                            let mut prop_context = DefaultExpressionContext::new();
                            let prop_value = ExpressionEvaluator::evaluate(&prop_values[i], &mut prop_context)
                                .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("属性值求值失败: {}", e))))?;
                            properties.insert(prop_name.clone(), prop_value);
                        }
                    }

                    let tag = Tag::new(tag_name.clone(), std::collections::HashMap::new());
                    let vertex = Vertex::new_with_properties(
                        vid,
                        vec![tag],
                        properties,
                    );
                    vertices.push(vertex);
                }

                let mut executor = InsertExecutor::with_vertices(
                    self.id,
                    self.storage.clone(),
                    vertices,
                );
                executor.open()?;
                executor.execute()
            }
            InsertTarget::Edge { edge_name, prop_names, edges } => {
                let mut edge_list = Vec::new();

                for (src_expr, dst_expr, rank_expr, prop_values) in edges {
                    let mut src_context = DefaultExpressionContext::new();
                    let src = ExpressionEvaluator::evaluate(&src_expr, &mut src_context)
                        .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("源顶点ID求值失败: {}", e))))?;

                    let mut dst_context = DefaultExpressionContext::new();
                    let dst = ExpressionEvaluator::evaluate(&dst_expr, &mut dst_context)
                        .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("目标顶点ID求值失败: {}", e))))?;

                    let rank = match rank_expr {
                        Some(ref r) => {
                            let mut rank_context = DefaultExpressionContext::new();
                            let rank_val = ExpressionEvaluator::evaluate(r, &mut rank_context)
                                .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("rank求值失败: {}", e))))?;
                            match rank_val {
                                crate::core::Value::Int(i) => i,
                                _ => return Err(DBError::Query(QueryError::ExecutionError("rank必须是整数".to_string()))),
                            }
                        }
                        None => 0,
                    };

                    let mut properties = std::collections::HashMap::new();
                    for (i, prop_name) in prop_names.iter().enumerate() {
                        if i < prop_values.len() {
                            let mut prop_context = DefaultExpressionContext::new();
                            let prop_value = ExpressionEvaluator::evaluate(&prop_values[i], &mut prop_context)
                                .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("属性值求值失败: {}", e))))?;
                            properties.insert(prop_name.clone(), prop_value);
                        }
                    }

                    let edge = Edge::new(
                        src,
                        dst,
                        edge_name.clone(),
                        rank,
                        properties,
                    );
                    edge_list.push(edge);
                }

                let mut executor = InsertExecutor::with_edges(
                    self.id,
                    self.storage.clone(),
                    edge_list,
                );
                executor.open()?;
                executor.execute()
            }
        }
    }

    #[allow(dead_code)]
    fn execute_merge(&mut self, _clause: crate::query::parser::ast::stmt::MergeStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("MERGE语句执行未实现".to_string())))
    }

    fn execute_unwind(&mut self, clause: crate::query::parser::ast::stmt::UnwindStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::executor::result_processing::transformations::unwind::UnwindExecutor;

        let mut executor = UnwindExecutor::new(
            self.id,
            self.storage.clone(),
            "_input".to_string(),
            clause.expression,
            vec![clause.variable.clone()],
            false,
        );
        executor.open()?;
        executor.execute()
    }

    #[allow(dead_code)]
    fn execute_return(&mut self, _clause: crate::query::parser::ast::stmt::ReturnStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("RETURN语句执行未实现".to_string())))
    }

    #[allow(dead_code)]
    fn execute_with(&mut self, _clause: crate::query::parser::ast::stmt::WithStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("WITH语句执行未实现".to_string())))
    }

    fn execute_set(&mut self, clause: crate::query::parser::ast::stmt::SetStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::executor::result_processing::transformations::assign::AssignExecutor;

        let mut assignments = Vec::new();
        for assignment in clause.assignments {
            assignments.push((assignment.property, assignment.value));
        }

        let mut executor = AssignExecutor::new(
            self.id,
            self.storage.clone(),
            assignments,
        );
        executor.open()?;
        executor.execute()
    }

    #[allow(dead_code)]
    fn execute_remove(&mut self, _clause: crate::query::parser::ast::stmt::RemoveStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("REMOVE语句执行未实现".to_string())))
    }

    #[allow(dead_code)]
    fn execute_pipe(&mut self, _clause: crate::query::parser::ast::stmt::PipeStmt) -> Result<ExecutionResult, DBError> {
        Err(DBError::Query(QueryError::ExecutionError("PIPE语句执行未实现".to_string())))
    }

    fn execute_drop(&mut self, clause: DropStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::parser::ast::stmt::DropTarget;
        let id = self.id;

        match clause.target {
            DropTarget::Space(space_name) => {
                let mut executor = admin_executor::DropSpaceExecutor::new(id, self.storage.clone(), space_name);
                executor.open()?;
                executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            DropTarget::Tags(tag_names) => {
                let mut total_dropped = 0;
                let mut errors = Vec::new();

                for tag_name in tag_names {
                    let mut executor = admin_executor::DropTagExecutor::new(id, self.storage.clone(), String::new(), tag_name.clone());
                    if let Err(e) = executor.open().and_then(|_| executor.execute()) {
                        errors.push(format!("DROP TAG {}: {}", tag_name, e));
                    } else {
                        total_dropped += 1;
                    }
                }

                if !errors.is_empty() {
                    Err(DBError::Query(QueryError::ExecutionError(
                        format!("部分标签删除失败: {}", errors.join("; "))
                    )))
                } else {
                    Ok(ExecutionResult::Count(total_dropped))
                }
            }
            DropTarget::Edges(edge_names) => {
                let mut total_dropped = 0;
                let mut errors = Vec::new();

                for edge_name in edge_names {
                    let mut executor = admin_executor::DropEdgeExecutor::new(id, self.storage.clone(), String::new(), edge_name.clone());
                    if let Err(e) = executor.open().and_then(|_| executor.execute()) {
                        errors.push(format!("DROP EDGE {}: {}", edge_name, e));
                    } else {
                        total_dropped += 1;
                    }
                }

                if !errors.is_empty() {
                    Err(DBError::Query(QueryError::ExecutionError(
                        format!("部分边类型删除失败: {}", errors.join("; "))
                    )))
                } else {
                    Ok(ExecutionResult::Count(total_dropped))
                }
            }
            DropTarget::TagIndex { space_name, index_name } => {
                let mut executor = admin_executor::DropTagIndexExecutor::new(id, self.storage.clone(), space_name, index_name);
                executor.open()?;
                executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            DropTarget::EdgeIndex { space_name, index_name } => {
                let mut executor = admin_executor::DropEdgeIndexExecutor::new(id, self.storage.clone(), space_name, index_name);
                executor.open()?;
                executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
        }
    }

    fn execute_desc(&mut self, clause: DescStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::parser::ast::stmt::DescTarget;
        let id = self.id;

        match clause.target {
            DescTarget::Space(space_name) => {
                let mut executor = admin_executor::DescSpaceExecutor::new(id, self.storage.clone(), space_name);
                executor.open()?;
                executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            DescTarget::Tag { space_name, tag_name } => {
                let mut executor = admin_executor::DescTagExecutor::new(id, self.storage.clone(), space_name, tag_name);
                executor.open()?;
                executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            DescTarget::Edge { space_name, edge_name } => {
                let mut executor = admin_executor::DescEdgeExecutor::new(id, self.storage.clone(), space_name, edge_name);
                executor.open()?;
                executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
        }
    }

    fn execute_alter(&mut self, clause: AlterStmt) -> Result<ExecutionResult, DBError> {
        use crate::query::parser::ast::stmt::AlterTarget;
        use admin_executor::{AlterEdgeExecutor, AlterTagExecutor, AlterEdgeInfo, AlterTagInfo, AlterTagItem, AlterEdgeItem};
        use crate::query::executor::admin::space::alter_space::{AlterSpaceExecutor, SpaceAlterOption};
        let id = self.id;

        match clause.target {
            AlterTarget::Tag { tag_name, additions, deletions: _, changes: _ } => {
                let mut items = Vec::new();
                for prop in additions {
                    items.push(AlterTagItem::add_property(prop));
                }
                let alter_info = AlterTagInfo::new(String::new(), tag_name).with_items(items);
                let mut executor = AlterTagExecutor::new(id, self.storage.clone(), alter_info);
                executor.open()?;
                executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            AlterTarget::Edge { edge_name, additions, deletions: _, changes: _ } => {
                let mut items = Vec::new();
                for prop in additions {
                    items.push(AlterEdgeItem::add_property(prop));
                }
                let alter_info = AlterEdgeInfo::new(String::new(), edge_name).with_items(items);
                let mut executor = AlterEdgeExecutor::new(id, self.storage.clone(), alter_info);
                executor.open()?;
                executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
            AlterTarget::Space { space_name, partition_num, replica_factor, comment } => {
                let mut options = Vec::new();
                if let Some(num) = partition_num {
                    options.push(SpaceAlterOption::PartitionNum(num));
                }
                if let Some(factor) = replica_factor {
                    options.push(SpaceAlterOption::ReplicaFactor(factor));
                }
                if let Some(comment_str) = comment {
                    options.push(SpaceAlterOption::Comment(comment_str));
                }
                let mut executor = AlterSpaceExecutor::new(id, self.storage.clone(), space_name, options);
                executor.open()?;
                executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
            }
        }
    }

    fn execute_create_user(&mut self, clause: CreateUserStmt) -> Result<ExecutionResult, DBError> {
        use admin_executor::CreateUserExecutor;
        let id = self.id;

        let user_info = UserInfo::new(clause.username, clause.password);
        let mut executor = CreateUserExecutor::new(id, self.storage.clone(), user_info);
        executor.open()?;
        executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
    }

    fn execute_alter_user(&mut self, clause: AlterUserStmt) -> Result<ExecutionResult, DBError> {
        use admin_executor::AlterUserExecutor;
        let id = self.id;

        let mut alter_info = UserAlterInfo::new(clause.username);
        if let Some(role) = clause.new_role {
            alter_info = alter_info.with_role(role);
        }
        if let Some(is_locked) = clause.is_locked {
            alter_info = alter_info.with_locked(is_locked);
        }
        let mut executor = AlterUserExecutor::new(id, self.storage.clone(), alter_info);
        executor.open()?;
        executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
    }

    fn execute_drop_user(&mut self, clause: DropUserStmt) -> Result<ExecutionResult, DBError> {
        use admin_executor::DropUserExecutor;
        let id = self.id;

        let mut executor = DropUserExecutor::new(id, self.storage.clone(), clause.username);
        executor.open()?;
        executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
    }

    fn execute_change_password(&mut self, clause: ChangePasswordStmt) -> Result<ExecutionResult, DBError> {
        use admin_executor::ChangePasswordExecutor;
        let id = self.id;

        let mut executor = ChangePasswordExecutor::new(
            id,
            self.storage.clone(),
            clause.username,
            clause.old_password,
            clause.new_password,
        );
        executor.open()?;
        executor.execute().map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))
    }
}

impl<S: StorageClient> Executor<S> for GraphQueryExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        Err(DBError::Query(QueryError::ExecutionError("需要先设置要执行的语句".to_string())))
    }

    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn open(&mut self) -> DBResult<()> {
        self.is_open = true;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.is_open = false;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        &self.stats
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        &mut self.stats
    }
}

impl<S: StorageClient> HasStorage<S> for GraphQueryExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        &self.storage
    }
}
