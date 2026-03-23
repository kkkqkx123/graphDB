//! 执行器工厂主模块
//!
//! 协调各个构建器、解析器和验证器
//! 负责根据执行计划创建对应的执行器实例

use crate::core::error::QueryError;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::factory::builders::Builders;
use crate::query::executor::factory::validators::{RecursionDetector, SafetyValidator};
use crate::query::planner::plan::core::nodes::base::plan_node_enum::PlanNodeEnum;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

// 导入安全配置类型
use crate::query::executor::factory::validators::safety_validator::ExecutorSafetyConfig;

/// 执行器工厂
///
/// 负责协调各个子模块创建执行器
pub struct ExecutorFactory<S: StorageClient + Send + 'static> {
    pub(crate) storage: Option<Arc<Mutex<S>>>,
    pub(crate) config: ExecutorSafetyConfig,
    pub(crate) recursion_detector: RecursionDetector,
    #[allow(dead_code)]
    pub(crate) safety_validator: SafetyValidator<S>,
    pub(crate) builders: Builders<S>,
}

impl<S: StorageClient + Send + 'static> ExecutorFactory<S> {
    /// 创建新的执行器工厂
    pub fn new() -> Self {
        let config = ExecutorSafetyConfig::default();
        let recursion_detector = RecursionDetector::new(config.max_recursion_depth);
        let safety_validator = SafetyValidator::new(config.clone());
        let builders = Builders::new();

        Self {
            storage: None,
            config,
            recursion_detector,
            safety_validator,
            builders,
        }
    }

    /// 设置存储引擎
    pub fn with_storage(storage: Arc<Mutex<S>>) -> Self {
        let mut factory = Self::new();
        factory.storage = Some(storage);
        factory
    }

    /// 分析执行计划的生命周期和安全性
    ///
    /// 使用DFS遍历执行计划树，检测循环引用并验证安全性
    pub fn analyze_plan_lifecycle(&mut self, root: &PlanNodeEnum) -> Result<(), QueryError> {
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

        // 使用 dependencies() 方法获取所有依赖，统一处理
        for dep in node.dependencies() {
            self.analyze_plan_node(&dep, loop_layers)?;
        }

        // 离开当前节点
        self.recursion_detector.leave_executor();

        Ok(())
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
            PlanNodeEnum::Start(node) => {
                use crate::query::executor::base::StartExecutor;
                Ok(ExecutorEnum::Start(StartExecutor::new(
                    node.id(),
                    context.expression_context().clone(),
                )))
            }

            // 数据访问执行器
            PlanNodeEnum::ScanVertices(node) => self
                .builders
                .data_access()
                .build_scan_vertices(node, storage, context),
            PlanNodeEnum::ScanEdges(node) => self
                .builders
                .data_access()
                .build_scan_edges(node, storage, context),
            PlanNodeEnum::GetVertices(node) => self
                .builders
                .data_access()
                .build_get_vertices(node, storage, context),
            PlanNodeEnum::GetNeighbors(node) => self
                .builders
                .data_access()
                .build_get_neighbors(node, storage, context),
            PlanNodeEnum::EdgeIndexScan(node) => self
                .builders
                .data_access()
                .build_edge_index_scan(node, storage, context),
            PlanNodeEnum::GetEdges(node) => self
                .builders
                .data_access()
                .build_get_edges(node, storage, context),
            PlanNodeEnum::IndexScan(node) => self
                .builders
                .data_access()
                .build_index_scan(node, storage, context),

            // 数据修改执行器
            PlanNodeEnum::InsertVertices(node) => self
                .builders
                .data_modification()
                .build_insert_vertices(node, storage, context),
            PlanNodeEnum::InsertEdges(node) => self
                .builders
                .data_modification()
                .build_insert_edges(node, storage, context),
            PlanNodeEnum::Remove(node) => self
                .builders
                .data_modification()
                .build_remove(node, storage, context),

            // 数据处理执行器
            PlanNodeEnum::Filter(node) => self
                .builders
                .data_processing()
                .build_filter(node, storage, context),
            PlanNodeEnum::Project(node) => self
                .builders
                .data_processing()
                .build_project(node, storage, context),
            PlanNodeEnum::Limit(node) => self
                .builders
                .data_processing()
                .build_limit(node, storage, context),
            PlanNodeEnum::Sort(node) => self
                .builders
                .data_processing()
                .build_sort(node, storage, context),
            PlanNodeEnum::TopN(node) => self
                .builders
                .data_processing()
                .build_topn(node, storage, context),
            PlanNodeEnum::Sample(node) => self
                .builders
                .data_processing()
                .build_sample(node, storage, context),
            PlanNodeEnum::Aggregate(node) => self
                .builders
                .data_processing()
                .build_aggregate(node, storage, context),
            PlanNodeEnum::Dedup(node) => self
                .builders
                .data_processing()
                .build_dedup(node, storage, context),

            // 连接执行器
            PlanNodeEnum::InnerJoin(node) => self
                .builders
                .join()
                .build_inner_join(node, storage, context),
            PlanNodeEnum::HashInnerJoin(node) => self
                .builders
                .join()
                .build_hash_inner_join(node, storage, context),
            PlanNodeEnum::LeftJoin(node) => {
                self.builders.join().build_left_join(node, storage, context)
            }
            PlanNodeEnum::HashLeftJoin(node) => self
                .builders
                .join()
                .build_hash_left_join(node, storage, context),
            PlanNodeEnum::FullOuterJoin(node) => self
                .builders
                .join()
                .build_full_outer_join(node, storage, context),
            PlanNodeEnum::CrossJoin(node) => self
                .builders
                .join()
                .build_cross_join(node, storage, context),

            // 集合操作执行器
            PlanNodeEnum::Union(node) => self
                .builders
                .set_operation()
                .build_union(node, storage, context),
            PlanNodeEnum::Minus(node) => self
                .builders
                .set_operation()
                .build_minus(node, storage, context),
            PlanNodeEnum::Intersect(node) => self
                .builders
                .set_operation()
                .build_intersect(node, storage, context),

            // 图遍历执行器
            PlanNodeEnum::Expand(node) => self
                .builders
                .traversal()
                .build_expand(node, storage, context),
            PlanNodeEnum::ExpandAll(node) => self
                .builders
                .traversal()
                .build_expand_all(node, storage, context),
            PlanNodeEnum::Traverse(node) => self
                .builders
                .traversal()
                .build_traverse(node, storage, context),
            PlanNodeEnum::AllPaths(node) => self
                .builders
                .traversal()
                .build_all_paths(node, storage, context),
            PlanNodeEnum::ShortestPath(node) => self
                .builders
                .traversal()
                .build_shortest_path(node, storage, context),
            PlanNodeEnum::BFSShortest(node) => self
                .builders
                .traversal()
                .build_bfs_shortest(node, storage, context),
            PlanNodeEnum::MultiShortestPath(node) => self
                .builders
                .traversal()
                .build_multi_shortest_path(node, storage, context),

            // 数据转换执行器
            PlanNodeEnum::Unwind(node) => self
                .builders
                .transformation()
                .build_unwind(node, storage, context),
            PlanNodeEnum::Assign(node) => self
                .builders
                .transformation()
                .build_assign(node, storage, context),
            PlanNodeEnum::Materialize(node) => self
                .builders
                .transformation()
                .build_materialize(node, storage, context),
            PlanNodeEnum::AppendVertices(node) => self
                .builders
                .transformation()
                .build_append_vertices(node, storage, context),
            PlanNodeEnum::RollUpApply(node) => self
                .builders
                .transformation()
                .build_rollup_apply(node, storage, context),
            PlanNodeEnum::PatternApply(node) => self
                .builders
                .transformation()
                .build_pattern_apply(node, storage, context),

            // 控制流执行器
            PlanNodeEnum::Loop(node) => self.build_loop_executor(node, storage, context),
            PlanNodeEnum::Select(node) => self.build_select_executor(node, storage, context),
            PlanNodeEnum::Argument(node) => self
                .builders
                .control_flow()
                .build_argument(node, storage, context),
            PlanNodeEnum::PassThrough(node) => self
                .builders
                .control_flow()
                .build_pass_through(node, storage, context),
            PlanNodeEnum::DataCollect(node) => self
                .builders
                .control_flow()
                .build_data_collect(node, storage, context),

            // 管理执行器 - 空间管理
            PlanNodeEnum::CreateSpace(node) => self
                .builders
                .admin()
                .build_create_space(node, storage, context),
            PlanNodeEnum::DropSpace(node) => self
                .builders
                .admin()
                .build_drop_space(node, storage, context),
            PlanNodeEnum::DescSpace(node) => self
                .builders
                .admin()
                .build_desc_space(node, storage, context),
            PlanNodeEnum::ShowSpaces(node) => self
                .builders
                .admin()
                .build_show_spaces(node, storage, context),

            // 管理执行器 - 标签管理
            PlanNodeEnum::CreateTag(node) => self
                .builders
                .admin()
                .build_create_tag(node, storage, context),
            PlanNodeEnum::AlterTag(node) => self
                .builders
                .admin()
                .build_alter_tag(node, storage, context),
            PlanNodeEnum::DescTag(node) => {
                self.builders.admin().build_desc_tag(node, storage, context)
            }
            PlanNodeEnum::DropTag(node) => {
                self.builders.admin().build_drop_tag(node, storage, context)
            }
            PlanNodeEnum::ShowTags(node) => self
                .builders
                .admin()
                .build_show_tags(node, storage, context),

            // 管理执行器 - 边管理
            PlanNodeEnum::CreateEdge(node) => self
                .builders
                .admin()
                .build_create_edge(node, storage, context),
            PlanNodeEnum::AlterEdge(node) => self
                .builders
                .admin()
                .build_alter_edge(node, storage, context),
            PlanNodeEnum::DescEdge(node) => self
                .builders
                .admin()
                .build_desc_edge(node, storage, context),
            PlanNodeEnum::DropEdge(node) => self
                .builders
                .admin()
                .build_drop_edge(node, storage, context),
            PlanNodeEnum::ShowEdges(node) => self
                .builders
                .admin()
                .build_show_edges(node, storage, context),

            // 管理执行器 - 标签索引管理
            PlanNodeEnum::CreateTagIndex(node) => self
                .builders
                .admin()
                .build_create_tag_index(node, storage, context),
            PlanNodeEnum::DropTagIndex(node) => self
                .builders
                .admin()
                .build_drop_tag_index(node, storage, context),
            PlanNodeEnum::DescTagIndex(node) => self
                .builders
                .admin()
                .build_desc_tag_index(node, storage, context),
            PlanNodeEnum::ShowTagIndexes(node) => self
                .builders
                .admin()
                .build_show_tag_indexes(node, storage, context),
            PlanNodeEnum::RebuildTagIndex(node) => self
                .builders
                .admin()
                .build_rebuild_tag_index(node, storage, context),

            // 管理执行器 - 边索引管理
            PlanNodeEnum::CreateEdgeIndex(node) => self
                .builders
                .admin()
                .build_create_edge_index(node, storage, context),
            PlanNodeEnum::DropEdgeIndex(node) => self
                .builders
                .admin()
                .build_drop_edge_index(node, storage, context),
            PlanNodeEnum::DescEdgeIndex(node) => self
                .builders
                .admin()
                .build_desc_edge_index(node, storage, context),
            PlanNodeEnum::ShowEdgeIndexes(node) => self
                .builders
                .admin()
                .build_show_edge_indexes(node, storage, context),
            PlanNodeEnum::RebuildEdgeIndex(node) => self
                .builders
                .admin()
                .build_rebuild_edge_index(node, storage, context),

            // 管理执行器 - 用户管理
            PlanNodeEnum::CreateUser(node) => self
                .builders
                .admin()
                .build_create_user(node, storage, context),
            PlanNodeEnum::DropUser(node) => self
                .builders
                .admin()
                .build_drop_user(node, storage, context),
            PlanNodeEnum::AlterUser(node) => self
                .builders
                .admin()
                .build_alter_user(node, storage, context),
            PlanNodeEnum::ChangePassword(node) => self
                .builders
                .admin()
                .build_change_password(node, storage, context),
            PlanNodeEnum::GrantRole(node) => self
                .builders
                .admin()
                .build_grant_role(node, storage, context),
            PlanNodeEnum::RevokeRole(node) => self
                .builders
                .admin()
                .build_revoke_role(node, storage, context),

            // 管理执行器 - 空间管理（补充）
            PlanNodeEnum::SwitchSpace(node) => self
                .builders
                .admin()
                .build_switch_space(node, storage, context),
            PlanNodeEnum::AlterSpace(node) => self
                .builders
                .admin()
                .build_alter_space(node, storage, context),
            PlanNodeEnum::ClearSpace(node) => self
                .builders
                .admin()
                .build_clear_space(node, storage, context),

            // 管理执行器 - 查询管理
            PlanNodeEnum::ShowStats(node) => self
                .builders
                .admin()
                .build_show_stats(node, storage, context),
        }
    }

    /// 构建 Loop 执行器（辅助方法，解决借用检查问题）
    fn build_loop_executor(
        &mut self,
        node: &crate::query::planner::plan::core::nodes::LoopNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // 先验证和检查递归
        if self.config.enable_recursion_detection {
            self.recursion_detector
                .validate_executor(node.id(), "LoopExecutor")
                .map_err(|e| QueryError::ExecutionError(e.to_string()))?;
        }

        let body = node
            .body()
            .as_ref()
            .ok_or_else(|| QueryError::ExecutionError("Loop节点缺少body".to_string()))?;

        // 临时释放 self 的借用，构建 body_executor
        let body_executor = {
            // 重新获取可变引用
            let config = self.config.clone();
            let max_recursion_depth = config.max_recursion_depth;
            let safety_validator = SafetyValidator::new(config.clone());
            let mut temp_factory = ExecutorFactory {
                storage: self.storage.clone(),
                config,
                recursion_detector: RecursionDetector::new(max_recursion_depth),
                safety_validator,
                builders: Builders::new(),
            };

            temp_factory.create_executor(body, storage.clone(), context)?
        };

        let condition = node
            .condition()
            .expression()
            .map(|meta| meta.inner().clone());

        use crate::query::executor::logic::LoopExecutor;
        let executor = LoopExecutor::new(
            node.id(),
            storage,
            condition,
            body_executor,
            None,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Loop(executor))
    }

    /// 构建 Select 执行器（辅助方法，解决借用检查问题）
    fn build_select_executor(
        &mut self,
        node: &crate::query::planner::plan::core::nodes::SelectNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // 先验证和检查递归
        if self.config.enable_recursion_detection {
            self.recursion_detector
                .validate_executor(node.id(), "SelectExecutor")
                .map_err(|e| QueryError::ExecutionError(e.to_string()))?;
        }

        let condition = node
            .condition()
            .expression()
            .map(|meta| meta.inner().clone())
            .unwrap_or_else(|| crate::core::Expression::Literal(crate::core::Value::Bool(true)));

        // 构建 if_branch
        let if_branch = {
            let if_node = node
                .if_branch()
                .as_ref()
                .ok_or_else(|| QueryError::ExecutionError("Select节点缺少if_branch".to_string()))?;

            let config = self.config.clone();
            let max_recursion_depth = config.max_recursion_depth;
            let safety_validator = SafetyValidator::new(config.clone());
            let mut temp_factory = ExecutorFactory {
                storage: self.storage.clone(),
                config,
                recursion_detector: RecursionDetector::new(max_recursion_depth),
                safety_validator,
                builders: Builders::new(),
            };

            temp_factory.create_executor(if_node, storage.clone(), context)?
        };

        // 构建 else_branch
        let else_branch = {
            if let Some(else_node) = node.else_branch().as_ref() {
                let config = self.config.clone();
                let max_recursion_depth = config.max_recursion_depth;
                let safety_validator = SafetyValidator::new(config.clone());
                let mut temp_factory = ExecutorFactory {
                    storage: self.storage.clone(),
                    config,
                    recursion_detector: RecursionDetector::new(max_recursion_depth),
                    safety_validator,
                    builders: Builders::new(),
                };

                Some(temp_factory.create_executor(else_node, storage.clone(), context)?)
            } else {
                None
            }
        };

        use crate::query::executor::logic::SelectExecutor;
        let executor = SelectExecutor::new(
            node.id(),
            storage,
            condition,
            if_branch,
            else_branch,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Select(executor))
    }
}

impl<S: StorageClient + 'static> Clone for ExecutorFactory<S> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            config: self.config.clone(),
            recursion_detector: RecursionDetector::new(self.config.max_recursion_depth),
            safety_validator: SafetyValidator::new(self.config.clone()),
            builders: Builders::new(),
        }
    }
}

impl<S: StorageClient + 'static> Default for ExecutorFactory<S> {
    fn default() -> Self {
        Self::new()
    }
}
