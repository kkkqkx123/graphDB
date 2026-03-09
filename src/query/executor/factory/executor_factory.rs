//! 执行器工厂主模块
//!
//! 协调各个构建器、解析器和验证器
//! 负责根据执行计划创建对应的执行器实例

use crate::core::error::QueryError;
use crate::query::executor::base::{ExecutionContext, Executor};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::factory::builders::Builders;
use crate::query::executor::factory::validators::{
    RecursionDetector, SafetyValidator,
};
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

// 导入安全配置类型
use crate::query::executor::factory::validators::safety_validator::ExecutorSafetyConfig;

/// 执行器工厂
///
/// 负责协调各个子模块创建执行器
/// 采用直接匹配模式，简单高效，易于维护
pub struct ExecutorFactory<S: StorageClient + 'static> {
    pub(crate) storage: Option<Arc<Mutex<S>>>,
    pub(crate) config: ExecutorSafetyConfig,
    pub(crate) recursion_detector: RecursionDetector,
    pub(crate) safety_validator: SafetyValidator<S>,
    pub(crate) builders: Builders<S>,
}

impl<S: StorageClient + 'static> ExecutorFactory<S> {
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
            PlanNodeEnum::FullOuterJoin(n) => {
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
            PlanNodeEnum::ScanVertices(node) => {
                self.builders.data_access().build_scan_vertices(node, storage, context)
            }
            PlanNodeEnum::ScanEdges(node) => {
                self.builders.data_access().build_scan_edges(node, storage, context)
            }
            PlanNodeEnum::GetVertices(node) => {
                self.builders.data_access().build_get_vertices(node, storage, context)
            }
            PlanNodeEnum::GetNeighbors(node) => {
                self.builders.data_access().build_get_neighbors(node, storage, context)
            }
            PlanNodeEnum::IndexScan(node) => {
                self.builders.data_access().build_index_scan(node, storage, context)
            }
            PlanNodeEnum::EdgeIndexScan(node) => {
                self.builders.data_access().build_edge_index_scan(node, storage, context)
            }

            // 数据处理执行器
            PlanNodeEnum::Filter(node) => {
                self.builders.data_processing().build_filter(node, storage, context)
            }
            PlanNodeEnum::Project(node) => {
                self.builders.data_processing().build_project(node, storage, context)
            }
            PlanNodeEnum::Limit(node) => {
                self.builders.data_processing().build_limit(node, storage, context)
            }
            PlanNodeEnum::Sort(node) => {
                self.builders.data_processing().build_sort(node, storage, context)
            }
            PlanNodeEnum::TopN(node) => {
                self.builders.data_processing().build_topn(node, storage, context)
            }
            PlanNodeEnum::Sample(node) => {
                self.builders.data_processing().build_sample(node, storage, context)
            }
            PlanNodeEnum::Aggregate(node) => {
                self.builders.data_processing().build_aggregate(node, storage, context)
            }
            PlanNodeEnum::Dedup(node) => {
                self.builders.data_processing().build_dedup(node, storage, context)
            }

            // 连接执行器
            PlanNodeEnum::InnerJoin(node) => {
                self.builders.join().build_inner_join(node, storage, context)
            }
            PlanNodeEnum::HashInnerJoin(node) => {
                self.builders.join().build_hash_inner_join(node, storage, context)
            }
            PlanNodeEnum::LeftJoin(node) => {
                self.builders.join().build_left_join(node, storage, context)
            }
            PlanNodeEnum::HashLeftJoin(node) => {
                self.builders.join().build_hash_left_join(node, storage, context)
            }
            PlanNodeEnum::FullOuterJoin(node) => {
                self.builders.join().build_full_outer_join(node, storage, context)
            }
            PlanNodeEnum::CrossJoin(node) => {
                self.builders.join().build_cross_join(node, storage, context)
            }

            // 集合操作执行器
            PlanNodeEnum::Union(node) => {
                self.builders.set_operation().build_union(node, storage, context)
            }
            PlanNodeEnum::Minus(node) => {
                self.builders.set_operation().build_minus(node, storage, context)
            }
            PlanNodeEnum::Intersect(node) => {
                self.builders.set_operation().build_intersect(node, storage, context)
            }

            // 图遍历执行器
            PlanNodeEnum::Expand(node) => {
                self.builders.traversal().build_expand(node, storage, context)
            }
            PlanNodeEnum::ExpandAll(node) => {
                self.builders.traversal().build_expand_all(node, storage, context)
            }
            PlanNodeEnum::Traverse(node) => {
                self.builders.traversal().build_traverse(node, storage, context)
            }
            PlanNodeEnum::AllPaths(node) => {
                self.builders.traversal().build_all_paths(node, storage, context)
            }
            PlanNodeEnum::ShortestPath(node) => {
                self.builders.traversal().build_shortest_path(node, storage, context)
            }
            PlanNodeEnum::BFSShortest(node) => {
                self.builders.traversal().build_bfs_shortest(node, storage, context)
            }

            // 数据转换执行器
            PlanNodeEnum::Unwind(node) => {
                self.builders.transformation().build_unwind(node, storage, context)
            }
            PlanNodeEnum::Assign(node) => {
                self.builders.transformation().build_assign(node, storage, context)
            }
            PlanNodeEnum::AppendVertices(node) => {
                self.builders.transformation().build_append_vertices(node, storage, context)
            }
            PlanNodeEnum::RollUpApply(node) => {
                self.builders.transformation().build_rollup_apply(node, storage, context)
            }
            PlanNodeEnum::PatternApply(node) => {
                self.builders.transformation().build_pattern_apply(node, storage, context)
            }

            // 控制流执行器
            PlanNodeEnum::Loop(node) => {
                self.builders.control_flow().build_loop(
                    node,
                    storage,
                    context,
                    &mut |n, s, c| self.create_executor(n, s, c),
                )
            }
            PlanNodeEnum::Select(node) => {
                self.builders.control_flow().build_select(
                    node,
                    storage,
                    context,
                    &mut |n, s, c| self.create_executor(n, s, c),
                )
            }
            PlanNodeEnum::Argument(node) => {
                self.builders.control_flow().build_argument(node, storage, context)
            }
            PlanNodeEnum::PassThrough(node) => {
                self.builders.control_flow().build_pass_through(node, storage, context)
            }
            PlanNodeEnum::DataCollect(node) => {
                self.builders.control_flow().build_data_collect(node, storage, context)
            }

            // 管理执行器 - 空间管理
            PlanNodeEnum::CreateSpace(node) => {
                self.builders.admin().build_create_space(node, storage, context)
            }
            PlanNodeEnum::DropSpace(node) => {
                self.builders.admin().build_drop_space(node, storage, context)
            }
            PlanNodeEnum::DescSpace(node) => {
                self.builders.admin().build_desc_space(node, storage, context)
            }
            PlanNodeEnum::ShowSpaces(node) => {
                self.builders.admin().build_show_spaces(node, storage, context)
            }

            // 管理执行器 - 标签管理
            PlanNodeEnum::CreateTag(node) => {
                self.builders.admin().build_create_tag(node, storage, context)
            }
            PlanNodeEnum::AlterTag(node) => {
                self.builders.admin().build_alter_tag(node, storage, context)
            }
            PlanNodeEnum::DescTag(node) => {
                self.builders.admin().build_desc_tag(node, storage, context)
            }
            PlanNodeEnum::DropTag(node) => {
                self.builders.admin().build_drop_tag(node, storage, context)
            }
            PlanNodeEnum::ShowTags(node) => {
                self.builders.admin().build_show_tags(node, storage, context)
            }

            // 管理执行器 - 边管理
            PlanNodeEnum::CreateEdge(node) => {
                self.builders.admin().build_create_edge(node, storage, context)
            }
            PlanNodeEnum::AlterEdge(node) => {
                self.builders.admin().build_alter_edge(node, storage, context)
            }
            PlanNodeEnum::DescEdge(node) => {
                self.builders.admin().build_desc_edge(node, storage, context)
            }
            PlanNodeEnum::DropEdge(node) => {
                self.builders.admin().build_drop_edge(node, storage, context)
            }
            PlanNodeEnum::ShowEdges(node) => {
                self.builders.admin().build_show_edges(node, storage, context)
            }

            // 管理执行器 - 标签索引管理
            PlanNodeEnum::CreateTagIndex(node) => {
                self.builders.admin().build_create_tag_index(node, storage, context)
            }
            PlanNodeEnum::DropTagIndex(node) => {
                self.builders.admin().build_drop_tag_index(node, storage, context)
            }
            PlanNodeEnum::DescTagIndex(node) => {
                self.builders.admin().build_desc_tag_index(node, storage, context)
            }
            PlanNodeEnum::ShowTagIndexes(node) => {
                self.builders.admin().build_show_tag_indexes(node, storage, context)
            }
            PlanNodeEnum::RebuildTagIndex(node) => {
                self.builders.admin().build_rebuild_tag_index(node, storage, context)
            }

            // 管理执行器 - 边索引管理
            PlanNodeEnum::CreateEdgeIndex(node) => {
                self.builders.admin().build_create_edge_index(node, storage, context)
            }
            PlanNodeEnum::DropEdgeIndex(node) => {
                self.builders.admin().build_drop_edge_index(node, storage, context)
            }
            PlanNodeEnum::DescEdgeIndex(node) => {
                self.builders.admin().build_desc_edge_index(node, storage, context)
            }
            PlanNodeEnum::ShowEdgeIndexes(node) => {
                self.builders.admin().build_show_edge_indexes(node, storage, context)
            }
            PlanNodeEnum::RebuildEdgeIndex(node) => {
                self.builders.admin().build_rebuild_edge_index(node, storage, context)
            }

            // 管理执行器 - 用户管理
            PlanNodeEnum::CreateUser(node) => {
                self.builders.admin().build_create_user(node, storage, context)
            }
            PlanNodeEnum::AlterUser(node) => {
                self.builders.admin().build_alter_user(node, storage, context)
            }
            PlanNodeEnum::DropUser(node) => {
                self.builders.admin().build_drop_user(node, storage, context)
            }
            PlanNodeEnum::ChangePassword(node) => {
                self.builders.admin().build_change_password(node, storage, context)
            }

            _ => Err(QueryError::ExecutionError(format!(
                "不支持的计划节点类型: {:?}",
                plan_node.type_name()
            ))),
        }
    }

    /// 递归构建执行器树
    pub fn build_and_create_executor(
        &mut self,
        plan_node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // 创建当前节点的执行器
        let mut executor = self.create_executor(plan_node, storage.clone(), context)?;

        // 根据节点类型递归构建子执行器
        match plan_node {
            // 单输入节点
            PlanNodeEnum::Filter(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }
            PlanNodeEnum::Project(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }
            PlanNodeEnum::Limit(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }
            PlanNodeEnum::Sort(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }
            PlanNodeEnum::TopN(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }
            PlanNodeEnum::Sample(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }
            PlanNodeEnum::Aggregate(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }
            PlanNodeEnum::Dedup(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }
            PlanNodeEnum::Expand(n) => {
                if let Some(input) = n.inputs().first() {
                    let child = self.build_and_create_executor(input, storage, context)?;
                    executor.add_child(child);
                }
            }
            PlanNodeEnum::AppendVertices(n) => {
                if let Some(input) = n.inputs().first() {
                    let child = self.build_and_create_executor(input, storage, context)?;
                    executor.add_child(child);
                }
            }
            PlanNodeEnum::Unwind(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }
            PlanNodeEnum::Assign(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }

            // 双输入节点（连接操作）
            PlanNodeEnum::InnerJoin(n) => {
                let left = self.build_and_create_executor(n.left_input(), storage.clone(), context)?;
                let right = self.build_and_create_executor(n.right_input(), storage, context)?;
                executor.add_child(left);
                executor.add_child(right);
            }
            PlanNodeEnum::HashInnerJoin(n) => {
                let left = self.build_and_create_executor(n.left_input(), storage.clone(), context)?;
                let right = self.build_and_create_executor(n.right_input(), storage, context)?;
                executor.add_child(left);
                executor.add_child(right);
            }
            PlanNodeEnum::LeftJoin(n) => {
                let left = self.build_and_create_executor(n.left_input(), storage.clone(), context)?;
                let right = self.build_and_create_executor(n.right_input(), storage, context)?;
                executor.add_child(left);
                executor.add_child(right);
            }
            PlanNodeEnum::HashLeftJoin(n) => {
                let left = self.build_and_create_executor(n.left_input(), storage.clone(), context)?;
                let right = self.build_and_create_executor(n.right_input(), storage, context)?;
                executor.add_child(left);
                executor.add_child(right);
            }
            PlanNodeEnum::FullOuterJoin(n) => {
                let left = self.build_and_create_executor(n.left_input(), storage.clone(), context)?;
                let right = self.build_and_create_executor(n.right_input(), storage, context)?;
                executor.add_child(left);
                executor.add_child(right);
            }
            PlanNodeEnum::CrossJoin(n) => {
                let left = self.build_and_create_executor(n.left_input(), storage.clone(), context)?;
                let right = self.build_and_create_executor(n.right_input(), storage, context)?;
                executor.add_child(left);
                executor.add_child(right);
            }

            // 集合操作节点
            PlanNodeEnum::Union(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }
            PlanNodeEnum::Minus(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }
            PlanNodeEnum::Intersect(n) => {
                let child = self.build_and_create_executor(n.input(), storage, context)?;
                executor.add_child(child);
            }

            // 循环节点
            PlanNodeEnum::Loop(_) => {
                // Loop执行器的body已经在创建时处理
            }

            // 选择节点
            PlanNodeEnum::Select(_) => {
                // Select执行器的分支已经在创建时处理
            }

            // 无子节点的执行器
            PlanNodeEnum::Start(_)
            | PlanNodeEnum::ScanVertices(_)
            | PlanNodeEnum::ScanEdges(_)
            | PlanNodeEnum::GetVertices(_)
            | PlanNodeEnum::GetNeighbors(_)
            | PlanNodeEnum::IndexScan(_)
            | PlanNodeEnum::EdgeIndexScan(_)
            | PlanNodeEnum::Argument(_)
            | PlanNodeEnum::PassThrough(_)
            | PlanNodeEnum::DataCollect(_)
            | PlanNodeEnum::BFSShortest(_) => {}

            // 管理执行器（无子节点）
            PlanNodeEnum::CreateSpace(_)
            | PlanNodeEnum::DropSpace(_)
            | PlanNodeEnum::DescSpace(_)
            | PlanNodeEnum::ShowSpaces(_)
            | PlanNodeEnum::CreateTag(_)
            | PlanNodeEnum::AlterTag(_)
            | PlanNodeEnum::DescTag(_)
            | PlanNodeEnum::DropTag(_)
            | PlanNodeEnum::ShowTags(_)
            | PlanNodeEnum::CreateEdge(_)
            | PlanNodeEnum::AlterEdge(_)
            | PlanNodeEnum::DescEdge(_)
            | PlanNodeEnum::DropEdge(_)
            | PlanNodeEnum::ShowEdges(_)
            | PlanNodeEnum::CreateTagIndex(_)
            | PlanNodeEnum::DropTagIndex(_)
            | PlanNodeEnum::DescTagIndex(_)
            | PlanNodeEnum::ShowTagIndexes(_)
            | PlanNodeEnum::RebuildTagIndex(_)
            | PlanNodeEnum::CreateEdgeIndex(_)
            | PlanNodeEnum::DropEdgeIndex(_)
            | PlanNodeEnum::DescEdgeIndex(_)
            | PlanNodeEnum::ShowEdgeIndexes(_)
            | PlanNodeEnum::RebuildEdgeIndex(_)
            | PlanNodeEnum::CreateUser(_)
            | PlanNodeEnum::AlterUser(_)
            | PlanNodeEnum::DropUser(_)
            | PlanNodeEnum::ChangePassword(_) => {}

            _ => {
                log::warn!(
                    "build_and_create_executor: 未处理的计划节点类型: {:?}",
                    plan_node.type_name()
                );
            }
        }

        Ok(executor)
    }
}

impl<S: StorageClient + 'static> Default for ExecutorFactory<S> {
    fn default() -> Self {
        Self::new()
    }
}
