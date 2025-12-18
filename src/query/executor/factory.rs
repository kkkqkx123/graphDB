//! 执行器工厂
//!
//! 负责根据执行计划创建对应的执行器实例
//! 基于nebula-graph的工厂模式设计

use crate::query::executor::traits::Executor;
use crate::query::planner::plan::core::{PlanNode, PlanNodeKind};
use crate::query::planner::plan::core::nodes::traits::PlanNodeProperties;
use crate::query::types::QueryError;
use crate::storage::StorageEngine;
use crate::query::parser::expressions::parse_expression_from_string;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

/// 执行器创建器特征 - 对象安全的设计
pub trait ExecutorCreator<S: StorageEngine>: std::fmt::Debug + Send + Sync {
    /// 创建执行器实例 - 返回具体的执行器类型
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError>;
}

/// 基础执行器工厂
///
/// 负责根据计划节点类型创建对应的执行器
#[derive(Debug)]
pub struct BaseExecutorFactory<S: StorageEngine + 'static> {
    /// 执行器创建器映射
    creators: HashMap<PlanNodeKind, Box<dyn ExecutorCreator<S>>>,
    /// 执行器ID计数器
    next_id: usize,
}

impl<S: StorageEngine + 'static + std::fmt::Debug> BaseExecutorFactory<S> {
    /// 创建新的执行器工厂
    pub fn new() -> Self {
        let mut factory = Self {
            creators: HashMap::new(),
            next_id: 1,
        };

        // 注册默认的执行器创建器
        factory.register_default_creators();
        factory
    }

    /// 注册默认的执行器创建器
    fn register_default_creators(&mut self) {
        // 注册各种计划节点类型的执行器创建器
        self.register_creator(
            PlanNodeKind::ScanVertices,
            Box::new(ScanVerticesCreator::<S> {
                _phantom: PhantomData,
            }),
        );
        self.register_creator(
            PlanNodeKind::ScanEdges,
            Box::new(ScanEdgesCreator::<S> {
                _phantom: PhantomData,
            }),
        );
        self.register_creator(
            PlanNodeKind::Filter,
            Box::new(FilterCreator::<S> {
                _phantom: PhantomData,
            }),
        );
        self.register_creator(
            PlanNodeKind::Project,
            Box::new(ProjectCreator::<S> {
                _phantom: PhantomData,
            }),
        );
        self.register_creator(
            PlanNodeKind::Limit,
            Box::new(LimitCreator::<S> {
                _phantom: PhantomData,
            }),
        );
        self.register_creator(
            PlanNodeKind::Sort,
            Box::new(SortCreator::<S> {
                _phantom: PhantomData,
            }),
        );
        self.register_creator(
            PlanNodeKind::Aggregate,
            Box::new(AggregateCreator::<S> {
                _phantom: PhantomData,
            }),
        );

        // 注册所有连接类型的执行器创建器
        self.register_creator(
            PlanNodeKind::HashInnerJoin,
            Box::new(JoinCreator::<S> {
                _phantom: PhantomData,
            }),
        );
        self.register_creator(
            PlanNodeKind::HashLeftJoin,
            Box::new(JoinCreator::<S> {
                _phantom: PhantomData,
            }),
        );
        self.register_creator(
            PlanNodeKind::CartesianProduct,
            Box::new(JoinCreator::<S> {
                _phantom: PhantomData,
            }),
        );

        self.register_creator(
            PlanNodeKind::Expand,
            Box::new(ExpandCreator::<S> {
                _phantom: PhantomData,
            }),
        );
        self.register_creator(
            PlanNodeKind::Start,
            Box::new(StartCreator::<S> {
                _phantom: PhantomData,
            }),
        );
        self.register_creator(
            PlanNodeKind::Unknown,
            Box::new(DefaultCreator::<S> {
                _phantom: PhantomData,
            }),
        );
    }

    /// 注册执行器创建器
    pub fn register_creator(&mut self, kind: PlanNodeKind, creator: Box<dyn ExecutorCreator<S>>) {
        self.creators.insert(kind, creator);
    }

    /// 根据计划节点创建执行器
    pub fn create_executor(
        &mut self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        let kind = plan_node.kind();

        let creator = self.creators.get(&kind).ok_or_else(|| {
            QueryError::ExecutionError(format!("未找到类型 {:?} 的执行器创建器", kind))
        })?;

        creator.create_executor(plan_node, storage)
    }

    /// 获取下一个执行器ID
    pub fn next_id(&self) -> usize {
        self.next_id
    }

    /// 生成并获取下一个执行器ID
    pub fn generate_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl<S: StorageEngine + 'static + std::fmt::Debug> Default for BaseExecutorFactory<S> {
    fn default() -> Self {
        Self::new()
    }
}

// 各种执行器创建器的实现

#[derive(Debug)]
struct ScanVerticesCreator<S: StorageEngine + std::fmt::Debug> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for ScanVerticesCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::data_access::GetVerticesExecutor;
        use crate::query::executor::tag_filter::TagFilterProcessor;
        use crate::query::planner::plan::core::nodes::ScanVerticesNode;

        let id = plan_node.id() as usize;

        // 验证计划节点类型
        if plan_node.kind() != PlanNodeKind::ScanVertices {
            return Err(QueryError::ExecutionError(format!(
                "期望ScanVertices计划节点，但得到{:?}",
                plan_node.kind()
            )));
        }

        // 尝试从具体的ScanVertices计划节点中提取参数
        let (vertex_ids, tag_filter, limit) =
            if let Some(scan_node) = plan_node.as_any().downcast_ref::<ScanVerticesNode>() {
                // ScanVertices是全表扫描操作，vertex_ids应为None
                let vertex_ids = None;

                // 使用标签过滤器处理器处理标签过滤条件
                let tag_filter = scan_node.tag_filter().as_ref().and_then(|filter_str| {
                    let processor = TagFilterProcessor::new();
                    match processor.parse_tag_filter(filter_str) {
                        Ok(expr) => Some(expr),
                        Err(e) => {
                            eprintln!("标签过滤表达式解析失败: {}, 使用无过滤", e);
                            None
                        }
                    }
                });

                // 处理limit参数，确保为正数
                let limit = scan_node.limit().and_then(|l| {
                    if l > 0 {
                        Some(l as usize)
                    } else {
                        None // 忽略非正数的limit
                    }
                });

                (vertex_ids, tag_filter, limit)
            } else {
                // 类型转换失败，返回错误
                return Err(QueryError::ExecutionError(
                    "无法将计划节点转换为ScanVertices类型".to_string(),
                ));
            };

        // 解析顶点过滤表达式
        let vertex_filter = if let Some(scan_node) =
            plan_node.as_any().downcast_ref::<ScanVerticesNode>()
        {
            scan_node.vertex_filter().as_ref().and_then(|filter_str| {
                match crate::query::parser::expressions::parse_expression_from_string(filter_str) {
                    Ok(expr) => Some(expr),
                    Err(e) => {
                        eprintln!("顶点过滤表达式解析失败: {}, 使用无过滤", e);
                        None
                    }
                }
            })
        } else {
            None
        };

        let executor =
            GetVerticesExecutor::new(id, storage, vertex_ids, tag_filter, vertex_filter, limit);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct ScanEdgesCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for ScanEdgesCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::data_access::GetEdgesExecutor;
        use crate::query::planner::plan::core::nodes::ScanEdgesNode;

        let id = plan_node.id() as usize;

        // 尝试从具体的ScanEdges计划节点中提取参数
        let edge_type = if let Some(scan_node) = plan_node.as_any().downcast_ref::<ScanEdgesNode>() {
            // 解析边类型
            Some(scan_node.edge_type().to_string())
        } else {
            // 如果不是具体的ScanEdges节点，使用默认值
            None
        };

        let executor = GetEdgesExecutor::new(id, storage, edge_type);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct FilterCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for FilterCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::graph::expression::Expression;
        use crate::query::executor::data_processing::filter::FilterExecutor;
        use crate::query::planner::plan::core::nodes::FilterNode;

        let id = plan_node.id() as usize;

        // 尝试从具体的Filter计划节点中提取条件
        let condition = if let Some(filter_node) = plan_node.as_any().downcast_ref::<FilterNode>() {
            // 使用getter方法获取过滤条件
            filter_node.condition().clone()
        } else {
            // 如果不是具体的Filter节点，使用默认条件
            Expression::literal(true)
        };

        let executor = FilterExecutor::new(id, storage, condition);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct ProjectCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for ProjectCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::graph::expression::Expression;
        use crate::query::executor::result_processing::projection::ProjectExecutor;
        use crate::query::executor::result_processing::projection::ProjectionColumn;
        use crate::query::planner::plan::core::nodes::ProjectNode;

        let id = plan_node.id() as usize;

        // 尝试从具体的Project计划节点中提取投影表达式
        let columns = if let Some(project_node) = plan_node.as_any().downcast_ref::<ProjectNode>() {
            // 使用节点中的列定义
            project_node.columns()
                .iter()
                .map(|yield_col| {
                    ProjectionColumn::new(
                        yield_col.alias.clone(),
                        yield_col.expr.clone(),
                    )
                })
                .collect()
        } else {
            // 如果不是具体的Project节点，使用默认投影
            vec![ProjectionColumn::new(
                "*".to_string(),
                Expression::literal("*"),
            )]
        };

        let executor = ProjectExecutor::new(id, storage, columns);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct LimitCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for LimitCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::data_processing::pagination::LimitExecutor;
        use crate::query::planner::plan::core::nodes::LimitNode;

        let id = plan_node.id() as usize;

        // 尝试从具体的Limit计划节点中提取参数
        let (limit, offset) = if let Some(limit_node) = plan_node.as_any().downcast_ref::<LimitNode>() {
            (
                limit_node.count().try_into().ok(),
                limit_node.offset().try_into().unwrap_or(0),
            )
        } else {
            // 如果不是具体的Limit节点，使用默认值
            (None, 0)
        };

        let executor = LimitExecutor::new(id, storage, limit, offset);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct SortCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for SortCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::graph::expression::Expression;
        use crate::query::executor::data_processing::sort::{SortExecutor, SortKey, SortOrder};
        use crate::query::planner::plan::core::nodes::SortNode;

        let id = plan_node.id() as usize;

        // 尝试从具体的Sort计划节点中提取参数
        let (sort_keys, limit) = if let Some(sort_node) = plan_node.as_any().downcast_ref::<SortNode>()
        {
            // 解析排序字段
            let keys: Vec<SortKey> = sort_node
                .sort_items()
                .iter()
                .map(|item| {
                    // 解析排序方向和表达式
                    let (expr_str, order) = if let Some(asc_pos) = item.find(" ASC") {
                        (item[..asc_pos].trim(), SortOrder::Asc)
                    } else if let Some(desc_pos) = item.find(" DESC") {
                        (item[..desc_pos].trim(), SortOrder::Desc)
                    } else {
                        // 默认为升序
                        (item.as_str(), SortOrder::Asc)
                    };

                    // 尝试解析表达式
                    let expr = match parse_expression_from_string(expr_str) {
                        Ok(parsed_expr) => parsed_expr,
                        Err(e) => {
                            eprintln!("解析排序表达式失败: {}, 使用变量表达式", e);
                            Expression::variable(expr_str.to_string())
                        }
                    };

                    SortKey::new(expr, order)
                })
                .collect();

            (keys, sort_node.limit().and_then(|l| l.try_into().ok()))
        } else {
            // 如果不是具体的Sort节点，使用默认值
            let default_keys = vec![SortKey::new(
                Expression::variable("default".to_string()),
                SortOrder::Asc,
            )];
            (default_keys, None)
        };

        let executor = SortExecutor::new(id, storage, sort_keys, limit);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct AggregateCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for AggregateCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::data_processing::aggregation::AggregateExecutor;
        use crate::query::planner::plan::core::nodes::AggregateNode;

        let id = plan_node.id() as usize;

        // 尝试从具体的Aggregate计划节点中提取参数
        let (_group_keys, _agg_funcs): (Vec<String>, Vec<String>) =
            if let Some(agg_node) = plan_node.as_any().downcast_ref::<AggregateNode>() {
                // 解析分组键和聚合函数
                (agg_node.group_keys().to_vec(), agg_node.agg_exprs().to_vec())
            } else {
                // 如果不是具体的Aggregate节点，使用默认值
                (vec![], vec![])
            };

        let executor = AggregateExecutor::new(id, storage);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct JoinCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for JoinCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::data_processing::join::cross_join::CrossJoinExecutor;
        use crate::query::executor::data_processing::join::inner_join::InnerJoinExecutor;
        use crate::query::executor::data_processing::join::left_join::LeftJoinExecutor;
        use crate::query::planner::plan::core::nodes::{InnerJoinNode, LeftJoinNode, CrossJoinNode};

        let id = plan_node.id() as usize;

        // 根据计划节点类型创建不同的执行器
        match plan_node.kind() {
            PlanNodeKind::HashInnerJoin => {
                // 从HashInnerJoin计划节点中提取参数
                if let Some(join_node) = plan_node.as_any().downcast_ref::<InnerJoinNode>() {
                    // 使用节点自身的字段
                    let left_var = "left_input".to_string();
                    let right_var = "right_input".to_string();
                    let left_keys = join_node.hash_keys().iter().map(|expr| format!("{:?}", expr)).collect::<Vec<String>>();
                    let right_keys = join_node.probe_keys().iter().map(|expr| format!("{:?}", expr)).collect::<Vec<String>>();
                    let output_cols = join_node.col_names().to_vec();

                    let executor = InnerJoinExecutor::new(
                        id,
                        storage,
                        left_var,
                        right_var,
                        left_keys,
                        right_keys,
                        output_cols,
                    );
                    Ok(Box::new(executor))
                } else {
                    return Err(QueryError::ExecutionError(
                        "无法将计划节点转换为 HashInnerJoin 类型".to_string(),
                    ));
                }
            }
            PlanNodeKind::HashLeftJoin => {
                // 处理左连接
                if let Some(join_node) = plan_node.as_any().downcast_ref::<LeftJoinNode>() {
                    // 使用节点自身的字段
                    let left_var = "left_input".to_string();
                    let right_var = "right_input".to_string();
                    let left_keys = join_node.hash_keys().iter().map(|expr| format!("{:?}", expr)).collect::<Vec<String>>();
                    let right_keys = join_node.probe_keys().iter().map(|expr| format!("{:?}", expr)).collect::<Vec<String>>();
                    let output_cols = join_node.col_names().to_vec();

                    let executor = LeftJoinExecutor::new(
                        id,
                        storage,
                        left_var,
                        right_var,
                        left_keys,
                        right_keys,
                        output_cols,
                    );
                    Ok(Box::new(executor))
                } else {
                    return Err(QueryError::ExecutionError(
                        "无法将计划节点转换为 HashLeftJoin 类型".to_string(),
                    ));
                }
            }
            PlanNodeKind::CartesianProduct => {
                // 处理笛卡尔积
                if let Some(join_node) = plan_node.as_any().downcast_ref::<CrossJoinNode>() {
                    // 使用节点自身的字段
                    let input_vars = vec!["left_input".to_string(), "right_input".to_string()];
                    let output_cols = join_node.col_names().to_vec();

                    let executor = CrossJoinExecutor::new(id, storage, input_vars, output_cols);
                    Ok(Box::new(executor))
                } else {
                    return Err(QueryError::ExecutionError(
                        "无法将计划节点转换为 CrossJoin 类型".to_string(),
                    ));
                }
            }
            _ => Err(QueryError::ExecutionError(format!(
                "不支持的连接类型: {:?}",
                plan_node.kind()
            ))),
        }
    }
}

#[derive(Debug)]
struct ExpandCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for ExpandCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::base::EdgeDirection;
        use crate::query::executor::data_processing::graph_traversal::expand::ExpandExecutor;
        use crate::query::planner::plan::core::nodes::ExpandNode;

        let id = plan_node.id() as usize;

        // 尝试从具体的Expand计划节点中提取参数
        let (direction, edge_types, max_depth) =
            if let Some(expand_node) = plan_node.as_any().downcast_ref::<ExpandNode>() {
                // 解析展开参数
                let direction = match expand_node.direction() {
                    "IN" => EdgeDirection::In,
                    "OUT" => EdgeDirection::Out,
                    "BOTH" => EdgeDirection::Both,
                    _ => {
                        eprintln!("未知的方向: {}, 使用默认值Both", expand_node.direction());
                        EdgeDirection::Both
                    }
                };

                let edge_types = if expand_node.edge_types().is_empty() {
                    None
                } else {
                    Some(expand_node.edge_types().to_vec())
                };

                let max_depth = expand_node.step_limit().map(|d| d as usize);

                (direction, edge_types, max_depth)
            } else {
                // 如果不是具体的Expand节点，使用默认值
                (EdgeDirection::Both, None, None)
            };

        let executor = ExpandExecutor::new(id, storage, direction, edge_types, max_depth);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct StartCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for StartCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::base::StartExecutor;

        let id = plan_node.id() as usize;

        // 创建基础的Start执行器
        let executor = StartExecutor::new(id, storage);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct DefaultCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for DefaultCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        _storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        Err(QueryError::ExecutionError(format!(
            "未知类型的计划节点: {:?}",
            plan_node.kind()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageEngine;

    // 模拟存储引擎用于测试
    #[derive(Debug)]
    struct MockStorage;

    impl StorageEngine for MockStorage {
        fn insert_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<crate::core::Value, crate::storage::StorageError> {
            unimplemented!()
        }

        fn get_node(
            &self,
            _id: &crate::core::Value,
        ) -> Result<Option<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            unimplemented!()
        }

        fn update_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<(), crate::storage::StorageError> {
            unimplemented!()
        }

        fn delete_node(
            &mut self,
            _id: &crate::core::Value,
        ) -> Result<(), crate::storage::StorageError> {
            unimplemented!()
        }

        fn scan_all_vertices(
            &self,
        ) -> Result<Vec<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            Ok(Vec::new())
        }

        fn scan_vertices_by_tag(
            &self,
            _tag: &str,
        ) -> Result<Vec<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            Ok(Vec::new())
        }

        fn insert_edge(
            &mut self,
            _edge: crate::core::vertex_edge_path::Edge,
        ) -> Result<(), crate::storage::StorageError> {
            unimplemented!()
        }

        fn get_edge(
            &self,
            _src: &crate::core::Value,
            _dst: &crate::core::Value,
            _edge_type: &str,
        ) -> Result<Option<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
        {
            unimplemented!()
        }

        fn get_node_edges(
            &self,
            _node_id: &crate::core::Value,
            _direction: crate::core::vertex_edge_path::Direction,
        ) -> Result<Vec<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
        {
            unimplemented!()
        }

        fn delete_edge(
            &mut self,
            _src: &crate::core::Value,
            _dst: &crate::core::Value,
            _edge_type: &str,
        ) -> Result<(), crate::storage::StorageError> {
            unimplemented!()
        }

        fn begin_transaction(&mut self) -> Result<u64, crate::storage::StorageError> {
            unimplemented!()
        }

        fn commit_transaction(&mut self, _tx_id: u64) -> Result<(), crate::storage::StorageError> {
            unimplemented!()
        }

        fn rollback_transaction(
            &mut self,
            _tx_id: u64,
        ) -> Result<(), crate::storage::StorageError> {
            unimplemented!()
        }
    }

    #[test]
    fn test_factory_creation() {
        let factory = BaseExecutorFactory::<MockStorage>::new();
        assert_eq!(factory.next_id(), 1);
    }

    #[test]
    fn test_generate_id() {
        let mut factory = BaseExecutorFactory::<MockStorage>::new();
        assert_eq!(factory.generate_id(), 1);
        assert_eq!(factory.generate_id(), 2);
        assert_eq!(factory.next_id(), 3);
    }
}
