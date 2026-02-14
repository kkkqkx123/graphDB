//! 执行器工厂模块
//!
//! 负责根据执行计划创建对应的执行器实例
//! 采用直接匹配模式，简单高效，易于维护

use crate::core::{EdgeDirection, Value};
use crate::core::error::QueryError;
use crate::query::context::execution::QueryContext;
use crate::query::executor::traits::Executor;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::{
    JoinNode, MultipleInputNode, SingleInputNode,
};

use crate::storage::StorageClient;
use std::sync::Arc;
use parking_lot::Mutex;

// 导入已实现的执行器
use crate::query::executor::base::{ExecutionContext, StartExecutor};
use crate::query::executor::data_access::{AllPathsExecutor, GetNeighborsExecutor, GetVerticesExecutor, ScanEdgesExecutor};
use crate::query::executor::data_processing::{
    graph_traversal::{ExpandAllExecutor, TraverseExecutor},
    CrossJoinExecutor, ExpandExecutor, InnerJoinExecutor, LeftJoinExecutor,
    UnionExecutor,
};
use crate::query::executor::data_processing::set_operations::{IntersectExecutor, MinusExecutor};
use crate::query::executor::logic::LoopExecutor;
use crate::query::executor::logic::SelectExecutor;
use crate::query::executor::recursion_detector::{
    ExecutorSafetyConfig, ExecutorSafetyValidator, RecursionDetector,
};
use crate::query::executor::result_processing::{
    AggregateExecutor, AppendVerticesExecutor, AssignExecutor, DedupExecutor, FilterExecutor, LimitExecutor,
    PatternApplyExecutor, ProjectExecutor, RollUpApplyExecutor, SampleExecutor, SampleMethod, SortExecutor,
    TopNExecutor, UnwindExecutor,
};
use crate::query::executor::search_executors::{BFSShortestExecutor, IndexScanExecutor};
use crate::query::executor::special_executors::{ArgumentExecutor, DataCollectExecutor, PassThroughExecutor};

use crate::query::executor::admin::{
    CreateSpaceExecutor, DropSpaceExecutor, DescSpaceExecutor, ShowSpacesExecutor,
    CreateTagExecutor, AlterTagExecutor, DescTagExecutor, DropTagExecutor, ShowTagsExecutor,
    CreateEdgeExecutor, AlterEdgeExecutor, DescEdgeExecutor, DropEdgeExecutor, ShowEdgesExecutor,
    CreateTagIndexExecutor, DropTagIndexExecutor, DescTagIndexExecutor, ShowTagIndexesExecutor,
    CreateEdgeIndexExecutor, DropEdgeIndexExecutor, DescEdgeIndexExecutor, ShowEdgeIndexesExecutor,
    RebuildTagIndexExecutor, RebuildEdgeIndexExecutor,
    CreateUserExecutor, AlterUserExecutor, DropUserExecutor, ChangePasswordExecutor,
};

/// 从 PlanNode 提取顶点 ID 列表
/// 用于多源最短路径等算法获取起始和目标顶点
fn extract_vertex_ids_from_node(node: &PlanNodeEnum) -> Vec<Value> {
    match node {
        PlanNodeEnum::GetVertices(n) => {
            vec![Value::from(format!("vertex_{}", n.id()))]
        }
        PlanNodeEnum::ScanVertices(n) => {
            vec![Value::from(format!("scan_{}", n.id()))]
        }
        PlanNodeEnum::Project(n) => {
            vec![Value::from(format!("project_{}", n.id()))]
        }
        PlanNodeEnum::Start(_) => {
            vec![Value::from("__start__")]
        }
        _ => {
            vec![Value::from(format!("node_{}", node.id()))]
        }
    }
}

/// 解析表达式字符串为 Graph 表达式
/// 安全版本：解析失败时记录日志并返回 None
fn parse_expression_safe(expr_str: &str) -> Option<crate::core::Expression> {
    crate::query::parser::parser::parse_expression_meta_from_string(expr_str)
        .map(|meta| meta.into())
        .inspect_err(|e| {
            log::warn!("Failed to parse expression: {}, error: {:?}", expr_str, e);
        })
        .ok()
}

/// 解析顶点ID字符串为 Value 列表
/// 支持逗号分隔的多个ID
fn parse_vertex_ids(src_vids: &str) -> Vec<Value> {
    src_vids
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| Value::String(s.to_string()))
        .collect()
}

/// 解析排序项字符串为排序键和方向
/// 支持格式: "column" 或 "column ASC" 或 "column DESC"
fn parse_sort_item(sort_item: &str) -> (String, crate::query::executor::result_processing::SortOrder) {
    let parts: Vec<&str> = sort_item
        .split_whitespace()
        .collect();

    match parts.as_slice() {
        [column] => (column.to_string(), crate::query::executor::result_processing::SortOrder::Asc),
        [column, direction] => {
            let order = match direction.to_uppercase().as_str() {
                "DESC" => crate::query::executor::result_processing::SortOrder::Desc,
                _ => crate::query::executor::result_processing::SortOrder::Asc,
            };
            (column.to_string(), order)
        }
        _ => (sort_item.to_string(), crate::query::executor::result_processing::SortOrder::Asc),
    }
}

/// 解析边方向字符串为 EdgeDirection 枚举
fn parse_edge_direction(direction_str: &str) -> crate::core::EdgeDirection {
    match direction_str.to_uppercase().as_str() {
        "OUT" => crate::core::EdgeDirection::Out,
        "IN" => crate::core::EdgeDirection::In,
        _ => crate::core::EdgeDirection::Both,
    }
}

/// 执行器工厂
///
/// 负责根据计划节点类型创建对应的执行器实例
/// 采用直接匹配模式，避免过度抽象
/// 包含递归检测和安全验证机制
pub struct ExecutorFactory<S: StorageClient + 'static> {
    storage: Option<Arc<Mutex<S>>>,
    config: ExecutorSafetyConfig,
    recursion_detector: RecursionDetector,
    safety_validator: ExecutorSafetyValidator,
}

impl<S: StorageClient + 'static> ExecutorFactory<S> {
    /// 创建新的执行器工厂
    pub fn new() -> Self {
        let config = ExecutorSafetyConfig::default();
        let recursion_detector = RecursionDetector::new(config.max_recursion_depth);
        let safety_validator = ExecutorSafetyValidator::new(config.clone());

        Self {
            storage: None,
            config,
            recursion_detector,
            safety_validator,
        }
    }

    /// 设置存储引擎
    pub fn with_storage(storage: Arc<Mutex<S>>) -> Self {
        let config = ExecutorSafetyConfig::default();
        let recursion_detector = RecursionDetector::new(config.max_recursion_depth);
        let safety_validator = ExecutorSafetyValidator::new(config.clone());

        Self {
            storage: Some(storage),
            config,
            recursion_detector,
            safety_validator,
        }
    }

    /// 提取连接操作的变量名
    fn extract_join_vars<N: JoinNode>(node: &N) -> (String, String) {
        let left_var = node
            .left_input()
            .output_var()
            .map(|v| v.name.clone())
            .unwrap_or_else(|| format!("left_{}", node.id()));
        let right_var = node
            .right_input()
            .output_var()
            .map(|v| v.name.clone())
            .unwrap_or_else(|| format!("right_{}", node.id()));
        (left_var, right_var)
    }

    /// 分析执行计划的生命周期和安全性
    ///
    /// 使用DFS遍历执行计划树，检测循环引用并验证安全性
    pub fn analyze_plan_lifecycle(
        &mut self,
        root: &PlanNodeEnum,
    ) -> Result<(), QueryError> {
        self.recursion_detector.reset();
        self.analyze_plan_node(root, 0)?;
        Ok(())
    }

    /// 递归分析单个计划节点
    fn analyze_plan_node(
        &mut self,
        node: &PlanNodeEnum,
        loop_layers: usize,
    ) -> Result<(), QueryError> {
        let node_id = node.id();
        let node_name = node.name();

        // 验证执行器是否会导致递归
        self.recursion_detector
            .validate_executor(node_id, node_name)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        // 验证计划节点的安全性
        self.validate_plan_node(node)?;

        // 根据节点类型处理依赖关系
        match node {
            // 单输入节点
            PlanNodeEnum::Filter(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }
            PlanNodeEnum::Project(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }
            PlanNodeEnum::Limit(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }
            PlanNodeEnum::Sort(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }
            PlanNodeEnum::TopN(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }
            PlanNodeEnum::Sample(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }
            PlanNodeEnum::Aggregate(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }
            PlanNodeEnum::Dedup(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }
            PlanNodeEnum::Expand(n) => {
                if let Some(input) = n.inputs().first() {
                    self.analyze_plan_node(input, loop_layers)?;
                }
            }
            PlanNodeEnum::AppendVertices(n) => {
                if let Some(input) = n.inputs().first() {
                    self.analyze_plan_node(input, loop_layers)?;
                }
            }
            PlanNodeEnum::Unwind(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }
            PlanNodeEnum::Assign(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }

            // 双输入节点（连接操作）
            PlanNodeEnum::InnerJoin(n) => {
                self.analyze_plan_node(n.left_input(), loop_layers)?;
                self.analyze_plan_node(n.right_input(), loop_layers)?;
            }
            PlanNodeEnum::HashInnerJoin(n) => {
                self.analyze_plan_node(n.left_input(), loop_layers)?;
                self.analyze_plan_node(n.right_input(), loop_layers)?;
            }
            PlanNodeEnum::LeftJoin(n) => {
                self.analyze_plan_node(n.left_input(), loop_layers)?;
                self.analyze_plan_node(n.right_input(), loop_layers)?;
            }
            PlanNodeEnum::HashLeftJoin(n) => {
                self.analyze_plan_node(n.left_input(), loop_layers)?;
                self.analyze_plan_node(n.right_input(), loop_layers)?;
            }
            PlanNodeEnum::CrossJoin(n) => {
                self.analyze_plan_node(n.left_input(), loop_layers)?;
                self.analyze_plan_node(n.right_input(), loop_layers)?;
            }
            
            // 并集节点
            PlanNodeEnum::Union(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }

            // 差集节点
            PlanNodeEnum::Minus(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }

            // 交集节点
            PlanNodeEnum::Intersect(n) => {
                self.analyze_plan_node(n.input(), loop_layers)?;
            }

            // 循环节点 - 递增循环层级
            PlanNodeEnum::Loop(n) => {
                if let Some(body) = n.body() {
                    self.analyze_plan_node(body, loop_layers)?;
                }
            }

            // 特殊节点
            PlanNodeEnum::Argument(_) => {}
            PlanNodeEnum::PassThrough(_) => {}
            PlanNodeEnum::DataCollect(_) => {}

            // 搜索节点
            PlanNodeEnum::BFSShortest(n) => {
                for dep in n.deps.iter() {
                    self.analyze_plan_node(dep, loop_layers)?;
                }
            }

            // 无输入节点
            PlanNodeEnum::Start(_) => {}

            // 数据访问节点
            PlanNodeEnum::ScanVertices(_) | PlanNodeEnum::GetVertices(_) => {}

            // 暂不支持的节点
            PlanNodeEnum::ScanEdges(_) => {}
            PlanNodeEnum::GetEdges(_) => {}
            PlanNodeEnum::IndexScan(_) => {}
            PlanNodeEnum::Select(_) => {}

            _ => {
                log::warn!("未处理的计划节点类型: {:?}", node.type_name());
            }
        }

        // 离开当前节点
        self.recursion_detector.leave_executor();

        Ok(())
    }

    /// 创建内连接执行器（通用方法）
    fn create_inner_join_executor<N>(
        &self,
        node: &N,
        storage: Arc<Mutex<S>>,
    ) -> Result<ExecutorEnum<S>, QueryError>
    where
        N: JoinNode,
    {
        let (left_var, right_var) = Self::extract_join_vars(node);
        let executor = InnerJoinExecutor::new(
            node.id(),
            storage,
            left_var,
            right_var,
            node.hash_keys().to_vec(),
            node.probe_keys().to_vec(),
            node.col_names().to_vec(),
        );
        Ok(ExecutorEnum::InnerJoin(executor))
    }

    /// 创建左连接执行器（通用方法）
    fn create_left_join_executor<N>(
        &self,
        node: &N,
        storage: Arc<Mutex<S>>,
    ) -> Result<ExecutorEnum<S>, QueryError>
    where
        N: JoinNode,
    {
        let (left_var, right_var) = Self::extract_join_vars(node);
        let executor = LeftJoinExecutor::new(
            node.id(),
            storage,
            left_var,
            right_var,
            node.hash_keys().to_vec(),
            node.probe_keys().to_vec(),
            node.col_names().to_vec(),
        );
        Ok(ExecutorEnum::LeftJoin(executor))
    }

    /// 验证计划节点的安全性
    fn validate_plan_node(&self, plan_node: &PlanNodeEnum) -> Result<(), QueryError> {
        match plan_node {
            PlanNodeEnum::Expand(node) => {
                let step_limit = node
                    .step_limit()
                    .and_then(|s| usize::try_from(s).ok())
                    .unwrap_or(10);
                if step_limit > 1000 {
                    return Err(QueryError::ExecutionError(format!(
                        "Expand执行器的步数限制{}超过安全阈值1000",
                        step_limit
                    )));
                }
            }
            PlanNodeEnum::Loop(_) => {
                return Err(QueryError::ExecutionError(
                    "循环执行器需要手动构建，不支持通过工厂自动创建".to_string(),
                ));
            }
            _ => {}
        }
        Ok(())
    }

    /// 根据计划节点创建执行器
    pub fn create_executor(
        &mut self,
        plan_node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        self.validate_plan_node(plan_node)?;

        if self.config.enable_recursion_detection {
            self.recursion_detector
                .validate_executor(plan_node.id(), plan_node.name())
                .map_err(|e| QueryError::ExecutionError(e.to_string()))?;
        }

        match plan_node {
            PlanNodeEnum::Start(node) => Ok(ExecutorEnum::Start(StartExecutor::new(node.id()))),

            // 数据访问执行器
            PlanNodeEnum::ScanVertices(node) => {
                let executor = GetVerticesExecutor::new(
                    node.id(),
                    storage,
                    None,
                    node.tag_filter().as_ref().and_then(|f| {
                        parse_expression_safe(f)
                    }),
                    node.vertex_filter().as_ref().and_then(|f| {
                        parse_expression_safe(f)
                    }),
                    node.limit().map(|l| l as usize),
                );
                Ok(ExecutorEnum::GetVertices(executor))
            }
            PlanNodeEnum::ScanEdges(node) => {
                let executor = ScanEdgesExecutor::new(
                    node.id(),
                    storage,
                    node.edge_type(),
                    node.filter().and_then(|f| parse_expression_safe(f)),
                    node.limit().map(|l| l as usize),
                );
                Ok(ExecutorEnum::ScanEdges(executor))
            }
            PlanNodeEnum::GetVertices(node) => {
                let vertex_ids = parse_vertex_ids(node.src_vids());
                let executor = GetVerticesExecutor::new(
                    node.id(),
                    storage,
                    if vertex_ids.is_empty() { None } else { Some(vertex_ids) },
                    None,
                    node.expression().and_then(|e| {
                        parse_expression_safe(e)
                    }),
                    node.limit().map(|l| l as usize),
                );
                Ok(ExecutorEnum::GetVertices(executor))
            }
            PlanNodeEnum::GetNeighbors(node) => {
                let vertex_ids = parse_vertex_ids(node.src_vids());
                let edge_direction = parse_edge_direction(node.direction());
                let edge_types = if node.edge_types().is_empty() {
                    None
                } else {
                    Some(node.edge_types().to_vec())
                };
                let executor = GetNeighborsExecutor::new(
                    node.id(),
                    storage,
                    vertex_ids,
                    edge_direction,
                    edge_types,
                );
                Ok(ExecutorEnum::GetNeighbors(executor))
            }

            PlanNodeEnum::Filter(node) => {
                let mut executor = FilterExecutor::new(node.id(), storage, node.condition().clone());
                executor = executor.with_parallel_config(self.config.parallel_config.clone());
                Ok(ExecutorEnum::Filter(executor))
            }
            PlanNodeEnum::Project(node) => {
                let columns = node
                    .columns()
                    .iter()
                    .map(|col| {
                        crate::query::executor::result_processing::ProjectionColumn::new(
                            col.alias.clone(),
                            col.expression.clone(),
                        )
                    })
                    .collect();
                let executor = ProjectExecutor::new(node.id(), storage, columns);
                Ok(ExecutorEnum::Project(executor))
            }
            PlanNodeEnum::Limit(node) => {
                let executor = LimitExecutor::new(
                    node.id(),
                    storage,
                    Some(node.count() as usize),
                    node.offset() as usize,
                );
                Ok(ExecutorEnum::Limit(executor))
            }
            PlanNodeEnum::Sort(node) => {
                let sort_keys = node
                    .sort_items()
                    .iter()
                    .map(|item| {
                        let (column, order) = parse_sort_item(item);
                        crate::query::executor::result_processing::SortKey::new(
                            crate::core::Expression::Variable(column),
                            order,
                        )
                    })
                    .collect();
                let config = crate::query::executor::result_processing::SortConfig::default();
                let executor = SortExecutor::new(
                    node.id(),
                    storage,
                    sort_keys,
                    node.limit().map(|l| l as usize),
                    config,
                )
                .map_err(|e| QueryError::ExecutionError(e.to_string()))?;
                Ok(ExecutorEnum::Sort(executor))
            }
            PlanNodeEnum::TopN(node) => {
                let executor = TopNExecutor::new(
                    node.id(),
                    storage,
                    node.limit() as usize,
                    node.sort_items().to_vec(),
                    true,
                );
                Ok(ExecutorEnum::TopN(executor))
            }
            PlanNodeEnum::Sample(node) => {
                let executor = SampleExecutor::new(
                    node.id(),
                    storage,
                    SampleMethod::Random,
                    node.count() as usize,
                    None,
                );
                Ok(ExecutorEnum::Sample(executor))
            }
            PlanNodeEnum::Aggregate(node) => {
                let aggregate_functions = node
                    .agg_exprs()
                    .iter()
                    .map(|agg_func| {
                        crate::query::executor::result_processing::AggregateFunctionSpec::from_agg_function(
                            agg_func.clone()
                        )
                    })
                    .collect();
                let group_by_expressions = node
                    .group_keys()
                    .iter()
                    .map(|key| crate::core::Expression::Variable(key.clone()))
                    .collect();
                let executor = AggregateExecutor::new(
                    node.id(),
                    storage,
                    aggregate_functions,
                    group_by_expressions,
                );
                Ok(ExecutorEnum::Aggregate(executor))
            }
            PlanNodeEnum::Dedup(node) => {
                let executor = DedupExecutor::new(
                    node.id(),
                    storage,
                    crate::query::executor::result_processing::DedupStrategy::Full,
                    None,
                );
                Ok(ExecutorEnum::Dedup(executor))
            }

            // 数据处理执行器
            PlanNodeEnum::InnerJoin(node) => self.create_inner_join_executor(&node, storage),
            PlanNodeEnum::HashInnerJoin(node) => self.create_inner_join_executor(&node, storage),
            PlanNodeEnum::LeftJoin(node) => self.create_left_join_executor(&node, storage),
            PlanNodeEnum::HashLeftJoin(node) => self.create_left_join_executor(&node, storage),
            PlanNodeEnum::CrossJoin(node) => {
                let left_var = node
                    .left_input()
                    .output_var()
                    .map(|v| v.name.to_string())
                    .unwrap_or_else(|| format!("left_{}", node.id()));
                let right_var = node
                    .right_input()
                    .output_var()
                    .map(|v| v.name.to_string())
                    .unwrap_or_else(|| format!("right_{}", node.id()));
                let executor = CrossJoinExecutor::new(
                    node.id(),
                    storage,
                    vec![left_var, right_var],
                    node.col_names().to_vec(),
                );
                Ok(ExecutorEnum::CrossJoin(executor))
            }
            
            // 并集执行器
            PlanNodeEnum::Union(node) => {
                let input_var = node
                    .input()
                    .output_var()
                    .map(|v| v.name.clone())
                    .unwrap_or_else(|| format!("union_{}", node.id()));
                let executor = UnionExecutor::new(
                    node.id(),
                    storage,
                    input_var.clone(),
                    input_var,
                );
                Ok(ExecutorEnum::Union(executor))
            }

            PlanNodeEnum::Minus(node) => {
                let left_var = node
                    .input()
                    .output_var()
                    .map(|v| v.name.clone())
                    .unwrap_or_else(|| format!("left_{}", node.id()));
                let right_var = node
                    .minus_input()
                    .output_var()
                    .map(|v| v.name.clone())
                    .unwrap_or_else(|| format!("right_{}", node.id()));
                let executor = MinusExecutor::new(
                    node.id(),
                    storage,
                    left_var,
                    right_var,
                );
                Ok(ExecutorEnum::Minus(executor))
            }

            PlanNodeEnum::Intersect(node) => {
                let left_var = node
                    .input()
                    .output_var()
                    .map(|v| v.name.clone())
                    .unwrap_or_else(|| format!("left_{}", node.id()));
                let right_var = node
                    .intersect_input()
                    .output_var()
                    .map(|v| v.name.clone())
                    .unwrap_or_else(|| format!("right_{}", node.id()));
                let executor = IntersectExecutor::new(
                    node.id(),
                    storage,
                    left_var,
                    right_var,
                );
                Ok(ExecutorEnum::Intersect(executor))
            }

            // 图遍历执行器
            PlanNodeEnum::Expand(node) => {
                // 验证Expand执行器的安全配置
                self.safety_validator
                    .validate_expand_config(node.step_limit().and_then(|s| usize::try_from(s).ok()))
                    .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

                let executor = ExpandExecutor::new(
                    node.id(),
                    storage,
                    node.direction(),
                    if node.edge_types().is_empty() {
                        None
                    } else {
                        Some(node.edge_types().to_vec())
                    },
                    node.step_limit().and_then(|s| usize::try_from(s).ok()),
                );
                Ok(ExecutorEnum::Expand(executor))
            }

            PlanNodeEnum::ExpandAll(node) => {
                self.safety_validator
                    .validate_expand_config(node.step_limit().and_then(|s| usize::try_from(s).ok()))
                    .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

                let executor = ExpandAllExecutor::new(
                    node.id(),
                    storage,
                    node.direction().into(),
                    if node.edge_types().is_empty() {
                        None
                    } else {
                        Some(node.edge_types().to_vec())
                    },
                    node.step_limit().and_then(|s| usize::try_from(s).ok()),
                );
                Ok(ExecutorEnum::ExpandAll(executor))
            }

            PlanNodeEnum::Traverse(node) => {
                let executor = TraverseExecutor::new(
                    node.id(),
                    storage,
                    node.direction().into(),
                    if node.edge_types().is_empty() {
                        None
                    } else {
                        Some(node.edge_types().to_vec())
                    },
                    node.step_limit().and_then(|s| usize::try_from(s).ok()),
                    node.filter().cloned(),
                );
                Ok(ExecutorEnum::Traverse(executor))
            }

            // AllPaths执行器 - 查找所有路径
            PlanNodeEnum::AllPaths(node) => {
                let start_vertex = if let Some(first_dep) = node.deps.first() {
                    extract_vertex_ids_from_node(first_dep)
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| Value::from("start"))
                } else {
                    Value::from("start")
                };

                let executor = AllPathsExecutor::new(
                    node.id(),
                    storage,
                    start_vertex,
                    None,
                    node.max_hop(),
                    if node.edge_types.is_empty() {
                        None
                    } else {
                        Some(node.edge_types.clone())
                    },
                    EdgeDirection::Both,
                );
                Ok(ExecutorEnum::AllPaths(executor))
            }

            // 最短路径执行器 - 单对单最短路径
            PlanNodeEnum::ShortestPath(node) => {
                let start_vertex_ids = if let Some(left) = node.deps.first() {
                    extract_vertex_ids_from_node(left)
                } else {
                    vec![Value::from("start")]
                };

                let end_vertex_ids = if let Some(right) = node.deps.get(1) {
                    extract_vertex_ids_from_node(right)
                } else {
                    vec![Value::from("end")]
                };

                let executor = crate::query::executor::data_processing::graph_traversal::ShortestPathExecutor::new(
                    node.id(),
                    storage,
                    start_vertex_ids,
                    end_vertex_ids,
                    EdgeDirection::Both,
                    if node.edge_types.is_empty() {
                        None
                    } else {
                        Some(node.edge_types.clone())
                    },
                    Some(node.max_step()),
                    crate::query::executor::data_processing::graph_traversal::ShortestPathAlgorithm::BFS,
                );
                Ok(ExecutorEnum::ShortestPath(executor))
            }

            // MultiShortestPath执行器 - 多源最短路径
            PlanNodeEnum::MultiShortestPath(node) => {
                let left_vids = if node.left_vid_var.is_empty() {
                    if let Some(left) = node.deps.first() {
                        extract_vertex_ids_from_node(left)
                    } else {
                        vec![Value::from("left_start")]
                    }
                } else {
                    vec![Value::from(node.left_vid_var.as_str())]
                };

                let right_vids = if node.right_vid_var.is_empty() {
                    if let Some(right) = node.deps.get(1) {
                        extract_vertex_ids_from_node(right)
                    } else {
                        vec![Value::from("right_target")]
                    }
                } else {
                    vec![Value::from(node.right_vid_var.as_str())]
                };

                let executor = crate::query::executor::data_processing::graph_traversal::MultiShortestPathExecutor::new(
                    node.id(),
                    storage,
                    left_vids,
                    right_vids,
                    node.steps(),
                    None,
                    node.single_shortest(),
                );
                Ok(ExecutorEnum::MultiShortestPath(executor))
            }

            // 数据转换执行器
            PlanNodeEnum::Unwind(node) => {
                let unwind_expression = crate::query::parser::parser::parse_expression_meta_from_string(
                    node.list_expression(),
                )
                .map(|meta| meta.into())
                .map_err(|e| QueryError::ExecutionError(format!("解析表达式失败: {}", e)))?;
                let executor = UnwindExecutor::new(
                    node.id(),
                    storage,
                    node.alias().to_string(),
                    unwind_expression,
                    node.col_names().to_vec(),
                    false,
                );
                Ok(ExecutorEnum::Unwind(executor))
            }
            PlanNodeEnum::Assign(node) => {
                let mut parsed_assignments = Vec::new();
                for (var_name, expr_str) in node.assignments() {
                    let expression =
                        crate::query::parser::parser::parse_expression_meta_from_string(expr_str)
                            .map(|meta| meta.into())
                            .map_err(|e| {
                            QueryError::ExecutionError(format!("解析表达式失败: {}", e))
                        })?;
                    parsed_assignments.push((var_name.clone(), expression));
                }
                let executor = AssignExecutor::new(node.id(), storage, parsed_assignments);
                Ok(ExecutorEnum::Assign(executor))
            }

            // AppendVertices执行器 - 追加顶点到路径结果
            PlanNodeEnum::AppendVertices(node) => {
                let input_var = node.input_var()
                    .map(|v| v.name.clone())
                    .unwrap_or_else(|| format!("input_{}", node.id()));

                let src_expression = node.src_expression()
                    .cloned()
                    .unwrap_or_else(|| crate::core::Expression::Variable("_".to_string()));

                let executor = AppendVerticesExecutor::new(
                    node.id(),
                    storage,
                    input_var,
                    src_expression,
                    None,
                    node.col_names().to_vec(),
                    node.dedup(),
                    node.track_prev_path(),
                    node.need_fetch_prop(),
                );
                Ok(ExecutorEnum::AppendVertices(executor))
            }

            // RollUpApply执行器 - 分组聚合收集
            PlanNodeEnum::RollUpApply(node) => {
                let left_input_var = node.left_input_var()
                    .cloned()
                    .unwrap_or_else(|| format!("left_{}", node.id()));
                let right_input_var = node.right_input_var()
                    .cloned()
                    .unwrap_or_else(|| format!("right_{}", node.id()));

                let compare_cols: Vec<crate::core::Expression> = node.compare_cols()
                    .iter()
                    .map(|col| {
                        crate::query::parser::parser::parse_expression_meta_from_string(col)
                            .map(|meta| meta.into())
                            .unwrap_or_else(|_| crate::core::Expression::Variable(col.clone()))
                    })
                    .collect();

                let collect_col = node.collect_col()
                    .and_then(|col| parse_expression_safe(col))
                    .unwrap_or_else(|| crate::core::Expression::Variable("_".to_string()));

                let executor = RollUpApplyExecutor::new(
                    node.id(),
                    storage,
                    left_input_var,
                    right_input_var,
                    compare_cols,
                    collect_col,
                    node.col_names().to_vec(),
                );
                Ok(ExecutorEnum::RollUpApply(executor))
            }

            // PatternApply执行器 - 模式匹配应用
            PlanNodeEnum::PatternApply(node) => {
                let left_input_var = node.left_input_var()
                    .cloned()
                    .unwrap_or_else(|| format!("left_{}", node.id()));
                let right_input_var = node.right_input_var()
                    .cloned()
                    .unwrap_or_else(|| format!("right_{}", node.id()));

                let key_cols: Vec<crate::core::Expression> = node.key_cols()
                    .iter()
                    .map(|col| {
                        crate::query::parser::parser::parse_expression_meta_from_string(col)
                            .map(|meta| meta.into())
                            .unwrap_or_else(|_| crate::core::Expression::Variable(col.clone()))
                    })
                    .collect();

                let executor = PatternApplyExecutor::new(
                    node.id(),
                    storage,
                    left_input_var,
                    right_input_var,
                    key_cols,
                    node.col_names().to_vec(),
                    node.is_anti_predicate(),
                );
                Ok(ExecutorEnum::PatternApply(executor))
            }

            // 循环执行器
            PlanNodeEnum::Loop(node) => {
                let body = node.body()
                    .as_ref()
                    .ok_or_else(|| QueryError::ExecutionError(
                        "Loop节点缺少body".to_string(),
                    ))?;
                
                let body_executor = self.create_executor(body, storage.clone(), context)?;
                
                let condition = node.condition()
                    .is_empty()
                    .then_some(node.condition().to_string())
                    .filter(|c| !c.is_empty())
                    .and_then(|c| parse_expression_safe(&c));
                
                let executor = LoopExecutor::new(
                    node.id(),
                    storage,
                    condition,
                    body_executor,
                    None,
                );
                Ok(ExecutorEnum::Loop(executor))
            }

            // 特殊执行器
            PlanNodeEnum::Argument(node) => {
                let executor = ArgumentExecutor::new(node.id(), storage, node.var());
                Ok(ExecutorEnum::Argument(executor))
            }

            PlanNodeEnum::PassThrough(_) => {
                let executor = PassThroughExecutor::new(plan_node.id(), storage);
                Ok(ExecutorEnum::PassThrough(executor))
            }

            PlanNodeEnum::DataCollect(_) => {
                let executor = DataCollectExecutor::new(plan_node.id(), storage);
                Ok(ExecutorEnum::DataCollect(executor))
            }

            // 搜索执行器
            PlanNodeEnum::BFSShortest(node) => {
                let start_vertex = if let Some(first_dep) = node.deps.first() {
                    extract_vertex_ids_from_node(first_dep)
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| Value::from("start"))
                } else {
                    Value::from("start")
                };

                let end_vertex = if let Some(second_dep) = node.deps.get(1) {
                    extract_vertex_ids_from_node(second_dep)
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| Value::from("end"))
                } else {
                    Value::from("end")
                };

                let executor = BFSShortestExecutor::new(
                    node.id(),
                    storage,
                    node.steps,
                    node.edge_types.clone(),
                    node.no_loop,
                    Some(node.steps),
                    false,
                    usize::MAX,
                    start_vertex,
                    end_vertex,
                );
                Ok(ExecutorEnum::BFSShortest(executor))
            }

            PlanNodeEnum::IndexScan(node) => {
                let executor = IndexScanExecutor::new(
                    node.id(),
                    storage,
                    node.space_id,
                    node.tag_id,
                    node.index_id,
                    &node.scan_type,
                    node.scan_limits.clone(),
                    node.filter.as_ref().and_then(|f| parse_expression_safe(f)),
                    node.return_columns.clone(),
                    node.limit.map(|l| l as usize),
                    node.is_edge_scan(),
                );
                Ok(ExecutorEnum::IndexScan(executor))
            }

            PlanNodeEnum::EdgeIndexScan(node) => {
                let executor = IndexScanExecutor::new(
                    node.id(),
                    storage,
                    node.space_id(),
                    node.edge_type().chars().fold(0, |acc, c| acc.wrapping_mul(31).wrapping_add(c as i32)), // 将 edge_type 转换为 tag_id
                    node.index_name().chars().fold(0, |acc, c| acc.wrapping_mul(31).wrapping_add(c as i32)), // 将 index_name 转换为 index_id
                    "EDGE_INDEX",
                    vec![], // EdgeIndexScanNode 没有 scan_limits 字段
                    node.filter().and_then(|f| parse_expression_safe(f)),
                    vec![], // EdgeIndexScanNode 没有 return_columns 字段
                    node.limit().map(|l| l as usize),
                    true, // is_edge - 边索引扫描
                );
                Ok(ExecutorEnum::IndexScan(executor))
            }

            PlanNodeEnum::Select(node) => {
                let condition = node.condition()
                    .is_empty()
                    .then_some(node.condition().to_string())
                    .filter(|c| !c.is_empty())
                    .and_then(|c| parse_expression_safe(&c))
                    .unwrap_or_else(|| crate::core::Expression::Literal(crate::core::Value::Bool(true)));

                let if_branch = node.if_branch()
                    .as_ref()
                    .ok_or_else(|| QueryError::ExecutionError(
                        "Select节点缺少if_branch".to_string(),
                    ))?;

                let if_executor = self.create_executor(if_branch, storage.clone(), context)?;

                let else_executor = node.else_branch()
                    .as_ref()
                    .map(|branch| self.create_executor(branch, storage.clone(), context))
                    .transpose()?;

                let executor = SelectExecutor::new(
                    node.id(),
                    storage,
                    condition,
                    if_executor,
                    else_executor,
                );
                Ok(ExecutorEnum::Select(executor))
            }

            // ========== 管理执行器 ==========

            // 空间管理执行器
            PlanNodeEnum::CreateSpace(node) => {
                use crate::query::executor::admin::space::create_space::ExecutorSpaceInfo;
                let space_info = ExecutorSpaceInfo::new(node.info().space_name.clone())
                    .with_partition_num(node.info().partition_num)
                    .with_replica_factor(node.info().replica_factor)
                    .with_vid_type(node.info().vid_type.clone());
                let executor = CreateSpaceExecutor::new(node.id(), storage, space_info);
                Ok(ExecutorEnum::CreateSpace(executor))
            }

            PlanNodeEnum::DropSpace(node) => {
                let executor = DropSpaceExecutor::new(node.id(), storage, node.space_name().to_string());
                Ok(ExecutorEnum::DropSpace(executor))
            }

            PlanNodeEnum::DescSpace(node) => {
                let executor = DescSpaceExecutor::new(node.id(), storage, node.space_name().to_string());
                Ok(ExecutorEnum::DescSpace(executor))
            }

            PlanNodeEnum::ShowSpaces(node) => {
                let executor = ShowSpacesExecutor::new(node.id(), storage);
                Ok(ExecutorEnum::ShowSpaces(executor))
            }

            // 标签管理执行器
            PlanNodeEnum::CreateTag(node) => {
                use crate::query::executor::admin::tag::create_tag::ExecutorTagInfo;
                let tag_info = ExecutorTagInfo {
                    space_name: node.info().space_name.clone(),
                    tag_name: node.info().tag_name.clone(),
                    properties: node.info().properties.clone(),
                    comment: None,
                };
                let executor = CreateTagExecutor::new(node.id(), storage, tag_info);
                Ok(ExecutorEnum::CreateTag(executor))
            }

            PlanNodeEnum::AlterTag(node) => {
                use crate::query::executor::admin::tag::alter_tag::{AlterTagInfo, AlterTagItem};
                let mut alter_info = AlterTagInfo::new(
                    node.info().space_name.clone(),
                    node.info().tag_name.clone(),
                );
                for prop in node.info().additions.iter() {
                    let item = AlterTagItem::add_property(prop.clone());
                    alter_info = alter_info.with_items(vec![item]);
                }
                for prop_name in node.info().deletions.iter() {
                    let item = AlterTagItem::drop_property(prop_name.clone());
                    alter_info = alter_info.with_items(vec![item]);
                }
                let executor = AlterTagExecutor::new(node.id(), storage, alter_info);
                Ok(ExecutorEnum::AlterTag(executor))
            }

            PlanNodeEnum::DescTag(node) => {
                let executor = DescTagExecutor::new(
                    node.id(),
                    storage,
                    node.space_name().to_string(),
                    node.tag_name().to_string(),
                );
                Ok(ExecutorEnum::DescTag(executor))
            }

            PlanNodeEnum::DropTag(node) => {
                let executor = DropTagExecutor::new(
                    node.id(),
                    storage,
                    node.space_name().to_string(),
                    node.tag_name().to_string(),
                );
                Ok(ExecutorEnum::DropTag(executor))
            }

            PlanNodeEnum::ShowTags(node) => {
                let executor = ShowTagsExecutor::new(node.id(), storage, "".to_string());
                Ok(ExecutorEnum::ShowTags(executor))
            }

            // 边类型管理执行器
            PlanNodeEnum::CreateEdge(node) => {
                use crate::query::executor::admin::edge::create_edge::ExecutorEdgeInfo;
                let edge_info = ExecutorEdgeInfo {
                    space_name: node.info().space_name.clone(),
                    edge_name: node.info().edge_name.clone(),
                    properties: node.info().properties.clone(),
                    comment: None,
                };
                let executor = CreateEdgeExecutor::new(node.id(), storage, edge_info);
                Ok(ExecutorEnum::CreateEdge(executor))
            }

            PlanNodeEnum::DescEdge(node) => {
                let executor = DescEdgeExecutor::new(
                    node.id(),
                    storage,
                    node.space_name().to_string(),
                    node.edge_name().to_string(),
                );
                Ok(ExecutorEnum::DescEdge(executor))
            }

            PlanNodeEnum::DropEdge(node) => {
                let executor = DropEdgeExecutor::new(
                    node.id(),
                    storage,
                    node.space_name().to_string(),
                    node.edge_name().to_string(),
                );
                Ok(ExecutorEnum::DropEdge(executor))
            }

            PlanNodeEnum::ShowEdges(node) => {
                let executor = ShowEdgesExecutor::new(node.id(), storage, "".to_string());
                Ok(ExecutorEnum::ShowEdges(executor))
            }

            PlanNodeEnum::AlterEdge(node) => {
                use crate::query::executor::admin::edge::alter_edge::{AlterEdgeInfo, AlterEdgeItem};
                let mut alter_info = AlterEdgeInfo::new(
                    node.info().space_name.clone(),
                    node.info().edge_name.clone(),
                );
                for prop in node.info().additions.iter() {
                    let item = AlterEdgeItem::add_property(prop.clone());
                    alter_info = alter_info.with_items(vec![item]);
                }
                for prop_name in node.info().deletions.iter() {
                    let item = AlterEdgeItem::drop_property(prop_name.clone());
                    alter_info = alter_info.with_items(vec![item]);
                }
                let executor = AlterEdgeExecutor::new(node.id(), storage, alter_info);
                Ok(ExecutorEnum::AlterEdge(executor))
            }

            // 标签索引管理执行器
            PlanNodeEnum::CreateTagIndex(node) => {
                use crate::index::{Index, IndexType};
                let index = Index::new(
                    0,
                    node.info().index_name.clone(),
                    0,
                    node.info().target_name.clone(),
                    Vec::new(),
                    node.info().properties.clone(),
                    IndexType::TagIndex,
                    false,
                );
                let executor = CreateTagIndexExecutor::new(node.id(), storage, index);
                Ok(ExecutorEnum::CreateTagIndex(executor))
            }

            PlanNodeEnum::DropTagIndex(node) => {
                let executor = DropTagIndexExecutor::new(
                    node.id(),
                    storage,
                    node.space_name().to_string(),
                    node.index_name().to_string(),
                );
                Ok(ExecutorEnum::DropTagIndex(executor))
            }

            PlanNodeEnum::DescTagIndex(node) => {
                let executor = DescTagIndexExecutor::new(
                    node.id(),
                    storage,
                    node.space_name().to_string(),
                    node.index_name().to_string(),
                );
                Ok(ExecutorEnum::DescTagIndex(executor))
            }

            PlanNodeEnum::ShowTagIndexes(node) => {
                let executor = ShowTagIndexesExecutor::new(node.id(), storage, "".to_string());
                Ok(ExecutorEnum::ShowTagIndexes(executor))
            }

            PlanNodeEnum::RebuildTagIndex(node) => {
                let executor = RebuildTagIndexExecutor::new(
                    node.id(),
                    storage,
                    node.space_name().to_string(),
                    node.index_name().to_string(),
                );
                Ok(ExecutorEnum::RebuildTagIndex(executor))
            }

            // 边索引管理执行器
            PlanNodeEnum::CreateEdgeIndex(node) => {
                use crate::index::{Index, IndexType};
                let index = Index::new(
                    0,
                    node.info().index_name.clone(),
                    0,
                    node.info().target_name.clone(),
                    Vec::new(),
                    node.info().properties.clone(),
                    IndexType::EdgeIndex,
                    false,
                );
                let executor = CreateEdgeIndexExecutor::new(node.id(), storage, index);
                Ok(ExecutorEnum::CreateEdgeIndex(executor))
            }

            PlanNodeEnum::DropEdgeIndex(node) => {
                let executor = DropEdgeIndexExecutor::new(
                    node.id(),
                    storage,
                    node.space_name().to_string(),
                    node.index_name().to_string(),
                );
                Ok(ExecutorEnum::DropEdgeIndex(executor))
            }

            PlanNodeEnum::DescEdgeIndex(node) => {
                let executor = DescEdgeIndexExecutor::new(
                    node.id(),
                    storage,
                    node.space_name().to_string(),
                    node.index_name().to_string(),
                );
                Ok(ExecutorEnum::DescEdgeIndex(executor))
            }

            PlanNodeEnum::ShowEdgeIndexes(node) => {
                let executor = ShowEdgeIndexesExecutor::new(node.id(), storage, "".to_string());
                Ok(ExecutorEnum::ShowEdgeIndexes(executor))
            }

            PlanNodeEnum::RebuildEdgeIndex(node) => {
                let executor = RebuildEdgeIndexExecutor::new(
                    node.id(),
                    storage,
                    node.space_name().to_string(),
                    node.index_name().to_string(),
                );
                Ok(ExecutorEnum::RebuildEdgeIndex(executor))
            }

            // 用户管理执行器
            PlanNodeEnum::CreateUser(node) => {
                use crate::core::types::metadata::UserInfo;
                let user_info = UserInfo::new(
                    node.username().to_string(),
                    node.password().to_string(),
                ).with_role(node.role().to_string());
                let executor = CreateUserExecutor::new(node.id(), storage, user_info);
                Ok(ExecutorEnum::CreateUser(executor))
            }

            PlanNodeEnum::AlterUser(node) => {
                use crate::core::types::metadata::UserAlterInfo;
                let mut alter_info = UserAlterInfo::new(node.username().to_string());
                if let Some(role) = node.new_role() {
                    alter_info = alter_info.with_role(role.clone());
                }
                if let Some(locked) = node.is_locked() {
                    alter_info = alter_info.with_locked(locked);
                }
                let executor = AlterUserExecutor::new(node.id(), storage, alter_info);
                Ok(ExecutorEnum::AlterUser(executor))
            }

            PlanNodeEnum::DropUser(node) => {
                let executor = DropUserExecutor::new(
                    node.id(),
                    storage,
                    node.username().to_string(),
                );
                Ok(ExecutorEnum::DropUser(executor))
            }

            PlanNodeEnum::ChangePassword(node) => {
                let password_info = node.password_info().clone();
                let executor = ChangePasswordExecutor::new(
                    node.id(),
                    storage,
                    password_info.username.clone(),
                    password_info.old_password.clone(),
                    password_info.new_password.clone(),
                );
                Ok(ExecutorEnum::ChangePassword(executor))
            }

            _ => Err(QueryError::ExecutionError(format!(
                "暂不支持执行器类型: {:?}",
                plan_node.type_name()
            ))),
        }
    }

    /// 执行执行计划
    pub async fn execute_plan(
        &mut self,
        query_context: &mut QueryContext,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> Result<crate::query::executor::traits::ExecutionResult, QueryError> {
        // 获取存储引擎
        let storage = match &self.storage {
            Some(storage) => storage.clone(),
            None => return Err(QueryError::ExecutionError("存储引擎未设置".to_string())),
        };

        // 获取根节点
        let root_node = match plan.root() {
            Some(node) => node,
            None => return Err(QueryError::ExecutionError("执行计划没有根节点".to_string())),
        };

        // 分析执行计划的生命周期和安全性
        self.analyze_plan_lifecycle(root_node)?;

        // 检查查询是否被终止
        if query_context.is_killed() {
            return Err(QueryError::ExecutionError(
                "查询已被终止".to_string()
            ));
        }

        // 创建执行上下文
        let execution_context = ExecutionContext::new();

        // 递归构建执行树并执行
        let mut executor = self.build_and_create_executor(root_node, storage, &execution_context)?;

        // 执行根执行器
        let result = executor
            .execute()
            .map_err(|e| QueryError::ExecutionError(format!("执行器执行失败: {}", e)))?;

        // 返回执行结果
        Ok(result)
    }

    /// 递归构建执行器树
    fn build_and_create_executor(
        &mut self,
        plan_node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // 先递归构建子节点
        let executor = self.create_executor(plan_node, storage, context)?;
        Ok(executor)
    }
}

impl<S: StorageClient + 'static> Default for ExecutorFactory<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;

    #[test]
    fn test_factory_creation() {
        let _factory = ExecutorFactory::<MockStorage>::new();
    }

    #[test]
    fn test_recursion_detector_basic() {
        let mut factory = ExecutorFactory::<MockStorage>::new();
        let storage = Arc::new(Mutex::new(MockStorage));
        let context = ExecutionContext::new();

        let start_node = PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        );

        let result = factory.create_executor(&start_node, storage, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_plan_lifecycle() {
        let mut factory = ExecutorFactory::<MockStorage>::new();
        let start_node = PlanNodeEnum::Start(
            crate::query::planner::plan::core::nodes::StartNode::new(),
        );

        let result = factory.analyze_plan_lifecycle(&start_node);
        assert!(result.is_ok());
    }
}
