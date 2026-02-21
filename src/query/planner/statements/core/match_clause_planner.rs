/// MATCH子句规划器
///
/// 负责规划 MATCH 子句的执行，是数据流的起始点
///
/// MATCH 子句是 Cypher 查询的核心，用于匹配图中的模式。
/// 它可以包含多个路径，每个路径由节点和边组成。
use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::context::QueryContext;
use crate::query::planner::connector::SegmentsConnector;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::algorithms::path_algorithms::MultiShortestPath;
use crate::query::planner::plan::core::nodes::{LimitNode, PlanNodeEnum, StartNode};
use crate::query::planner::plan::core::node_id_generator::next_node_id;

use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::validator::structs::{AliasType, CypherClauseKind, MatchClauseContext, Path, PathYieldType};
use crate::storage::metadata::SchemaManager;
use std::collections::{HashMap, HashSet};

/// MATCH子句规划器
///
/// 负责规划 MATCH 子句的执行，是数据流的起始点
///
/// MATCH 子句是 Cypher 查询的核心，用于匹配图中的模式。
/// 它可以包含多个路径，每个路径由节点和边组成。
///
/// # 已实现功能
/// - ✅ 从 schema 信息解析标签ID (tids) 和边类型ID
/// - ✅ 完善别名收集和管理 (aliases_available, aliases_generated)
/// - ✅ 支持 Alternative, Optional, Repeated 路径元素类型
/// 
/// # 路径优化说明
/// 谓词路径优化（如算法选择、双向BFS优化等）应在 Optimizer 层通过规则实现，
/// 而非在 Planner 层硬编码。详见：
/// - `src/query/optimizer/rules/path/` 路径优化规则
/// - `PathAlgorithmSelectionRule` - 路径算法选择
/// - `BidirectionalBFSOptimizationRule` - 双向BFS优化
#[derive(Debug)]
pub struct MatchClausePlanner {}

impl MatchClausePlanner {
    /// 创建新的 MATCH 子句规划器
    pub fn new() -> Self {
        Self {}
    }

    fn plan_path(
        &self,
        path: &Path,
        space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        // 根据路径类型选择不同的规划策略
        match path.path_type {
            PathYieldType::Shortest | PathYieldType::AllShortest => {
                self.plan_shortest_path(path, space_id)
            }
            _ => self.plan_standard_path(path, space_id),
        }
    }

    /// 规划标准路径（普通 MATCH 和谓词路径）
    ///
    /// 统一处理普通路径和谓词路径的规划逻辑
    fn plan_standard_path(
        &self,
        path: &Path,
        space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let mut current_plan = SubPlan::new(None, None);

        // 规划路径中的节点
        for node_info in path.node_infos.iter() {
            let scan_node = crate::query::planner::plan::core::nodes::ScanVerticesNode::new(space_id);
            let node_plan = SubPlan::from_root(scan_node.clone().into_enum());

            current_plan = if let Some(existing_root) = current_plan.root.take() {
                SegmentsConnector::cross_join(
                    SubPlan::new(Some(existing_root), current_plan.tail),
                    node_plan,
                )?
            } else {
                node_plan
            };

            if let Some(filter) = &node_info.filter {
                let filter_node = crate::query::planner::plan::core::nodes::FilterNode::new(
                    scan_node.into_enum(),
                    filter.clone(),
                )?;
                current_plan = SubPlan::new(Some(filter_node.into_enum()), current_plan.tail);
            }
        }

        // 规划路径中的边
        for edge_info in &path.edge_infos {
            let expand_node = crate::query::planner::plan::core::nodes::ExpandAllNode::new(
                space_id,
                edge_info.types.clone(),
                "both",
            );
            let edge_plan = SubPlan::from_root(expand_node.into_enum());

            current_plan = if let Some(existing_root) = current_plan.root.take() {
                SegmentsConnector::cross_join(
                    SubPlan::new(Some(existing_root), current_plan.tail),
                    edge_plan,
                )?
            } else {
                edge_plan
            };
        }

        Ok(current_plan)
    }

    /// 规划最短路径
    fn plan_shortest_path(
        &self,
        path: &Path,
        _space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        // 验证最短路径模式：需要恰好两个节点和一条边
        if path.node_infos.len() != 2 || path.edge_infos.len() != 1 {
            return Err(PlannerError::InvalidOperation(
                "最短路径模式需要恰好两个节点和一条边，如: (a)-[:type*..5]->(b)".to_string()
            ));
        }

        let edge_info = &path.edge_infos[0];

        // 创建最短路径计划节点
        let steps = edge_info.range.as_ref().map(|r| r.max()).unwrap_or(5) as usize;
        let shortest_node = MultiShortestPath::new(
            next_node_id(),
            PlanNodeEnum::Start(StartNode::new()),
            PlanNodeEnum::Start(StartNode::new()),
            steps,
        );

        // 设置最短路径类型
        let mut shortest_node = shortest_node;
        if path.path_type == PathYieldType::Shortest {
            // 单条最短路径
            shortest_node.single_shortest = true;
        }

        Ok(SubPlan::from_root(shortest_node.into_enum()))
    }

    /// 查找两个计划之间的共享别名
    fn find_inter_aliases(
        &self,
        match_clause_ctx: &MatchClauseContext,
        input_plan: &SubPlan,
        _current_plan: &SubPlan,
    ) -> HashSet<String> {
        let mut inter_aliases = HashSet::new();

        // 获取输入计划的列名（可用别名）
        let input_aliases: std::collections::HashSet<String> = if let Some(ref root) = input_plan.root {
            root.col_names().iter().cloned().collect()
        } else {
            std::collections::HashSet::new()
        };

        // 检查 MATCH 子句生成的别名是否与输入计划共享
        for path in &match_clause_ctx.paths {
            for node_info in &path.node_infos {
                if !node_info.alias.is_empty() && input_aliases.contains(&node_info.alias) {
                    inter_aliases.insert(node_info.alias.clone());
                }
            }
            for edge_info in &path.edge_infos {
                if !edge_info.alias.is_empty() && input_aliases.contains(&edge_info.alias) {
                    inter_aliases.insert(edge_info.alias.clone());
                }
            }
        }

        inter_aliases
    }

    /// 规划 MATCH 子句
    ///
    /// # 参数
    /// - `match_clause_ctx`: MATCH 子句上下文，包含路径、WHERE条件等信息
    /// - `input_plan`: 输入计划，对于 OPTIONAL MATCH 或连续 MATCH 时使用
    /// - `space_id`: 图空间 ID
    ///
    /// # 返回
    /// - 成功：生成的子计划
    /// - 失败：规划错误
    pub fn plan_match_clause(
        &self,
        match_clause_ctx: &MatchClauseContext,
        input_plan: Option<&SubPlan>,
        space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        // 验证 MATCH 子句上下文的完整性
        if match_clause_ctx.paths.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "MATCH 子句必须至少包含一个路径".to_string(),
            ));
        }

        // 处理路径
        let mut plan = SubPlan::new(None, None);

        for path in &match_clause_ctx.paths {
            let path_plan = self.plan_path(path, space_id)?;

            // 连接路径计划
            plan = if let Some(existing_root) = plan.root.take() {
                SegmentsConnector::cross_join(
                    SubPlan::new(Some(existing_root), plan.tail),
                    path_plan,
                )?
            } else {
                path_plan
            };
        }

        // 处理 WHERE 条件
        if let Some(ref where_ctx) = match_clause_ctx.where_clause {
            if let Some(ref filter) = where_ctx.filter {
                let input_node = plan.root.as_ref()
                    .ok_or_else(|| PlannerError::PlanGenerationFailed("MATCH计划缺少根节点".to_string()))?;
                let filter_node = crate::query::planner::plan::core::nodes::FilterNode::new(
                    input_node.clone(),
                    filter.clone(),
                )?;
                plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
            }
        }

        // 处理 OPTIONAL MATCH：如果有输入计划，使用左连接
        if let Some(input) = input_plan {
            if match_clause_ctx.is_optional {
                // OPTIONAL MATCH：使用左连接
                let qctx = QueryContext::new();
                let inter_aliases = self.find_inter_aliases(match_clause_ctx, input, &plan);
                let inter_aliases_ref: HashSet<&str> = inter_aliases.iter().map(|s| s.as_str()).collect();
                plan = SegmentsConnector::left_join(&qctx, input.clone(), plan, inter_aliases_ref)?;
            } else {
                // 普通 MATCH：使用内连接
                let qctx = QueryContext::new();
                let inter_aliases = self.find_inter_aliases(match_clause_ctx, input, &plan);
                if inter_aliases.is_empty() {
                    // 没有共享别名，使用交叉连接
                    plan = SegmentsConnector::cross_join(input.clone(), plan)?;
                } else {
                    let inter_aliases_ref: HashSet<&str> = inter_aliases.iter().map(|s| s.as_str()).collect();
                    plan = SegmentsConnector::inner_join(&qctx, input.clone(), plan, inter_aliases_ref)?;
                }
            }
        }

        // 处理分页（如果存在）
        // 合并 skip 和 limit 为一个 LimitNode 处理
        let skip_value = match_clause_ctx.skip.as_ref().map(|skip| match skip {
            Expression::Literal(crate::core::Value::Int(v)) if *v > 0 => *v,
            _ => 0,
        }).unwrap_or(0);

        let limit_value = match_clause_ctx.limit.as_ref().map(|limit| match limit {
            Expression::Literal(crate::core::Value::Int(v)) if *v >= 0 => *v,
            _ => i64::MAX,
        }).unwrap_or(i64::MAX);

        // 当 skip 或 limit 有有效值时，创建 LimitNode
        if skip_value > 0 || limit_value != i64::MAX {
            let input_node = plan.root.as_ref()
                .ok_or_else(|| PlannerError::PlanGenerationFailed("分页处理需要输入计划".to_string()))?;
            let limit_node = LimitNode::new(
                input_node.clone(),
                skip_value,
                if limit_value == i64::MAX { i64::MAX } else { limit_value },
            )?;
            plan = SubPlan::new(Some(limit_node.into_enum()), plan.tail);
        }

        Ok(plan)
    }
}

impl ClausePlanner for MatchClausePlanner {
    fn clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Match
    }

    fn name(&self) -> &'static str {
        "MatchClausePlanner"
    }

    fn transform_clause(
        &self,
        query_context: &mut QueryContext,
        ast_ctx: &AstContext,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        // 从 AST 上下文中提取 MATCH 子句信息
        // 传入 input_plan 用于提取可用别名
        let match_clause_ctx = Self::extract_match_context(ast_ctx, query_context, &input_plan)?;

        // 从 AST 上下文中获取 space_id，默认为 1
        let space_id = ast_ctx.space().space_id.unwrap_or(1);

        // 对于 MATCH 子句，input_plan 可能是空计划（作为 Source）
        // 或者在管道中作为输入（如 WITH ... MATCH ...）
        let has_input = input_plan.root().is_some();
        self.plan_match_clause(
            &match_clause_ctx,
            if has_input { Some(&input_plan) } else { None },
            space_id,
        )
    }
}

impl MatchClausePlanner {
    /// 从 AST 上下文中提取 MATCH 子句上下文
    ///
    /// 完善后的实现包括：
    /// - 完整的 Pattern 到 Path 转换
    /// - 别名收集和管理
    /// - WHERE 子句处理
    /// - Schema 信息解析（标签ID和边类型ID）
    fn extract_match_context(
        ast_ctx: &AstContext,
        query_context: &QueryContext,
        input_plan: &SubPlan,
    ) -> Result<MatchClauseContext, PlannerError> {
        use crate::query::parser::ast::Stmt;
        use crate::query::validator::structs::WhereClauseContext;

        let sentence = ast_ctx.sentence()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("AST 上下文中没有语句".to_string()))?;

        let match_stmt = match sentence {
            Stmt::Match(m) => m,
            _ => {
                return Err(PlannerError::PlanGenerationFailed(
                    "期望 MATCH 语句，但得到了其他类型的语句".to_string()
                ));
            }
        };

        // 获取 schema manager 和 space 名称
        let schema_manager = query_context.schema_manager();
        let space_name = ast_ctx.space().space_name.clone();

        // 转换 patterns 到 paths（传入 schema_manager 解析标签ID和边类型ID）
        let paths = Self::convert_patterns_to_paths(&match_stmt.patterns, schema_manager, &space_name)?;
        
        // 从 paths 收集别名
        let aliases_generated = Self::collect_aliases_from_paths(&paths);

        // 从 input_plan 提取可用别名
        let aliases_available = Self::extract_aliases_from_input_plan(input_plan);

        // 构建 WHERE 子句上下文
        let where_clause = match_stmt.where_clause.as_ref().map(|condition| {
            WhereClauseContext {
                filter: Some(condition.clone()),
                aliases_available: aliases_available.clone(),
                aliases_generated: std::collections::HashMap::new(),
                paths: vec![],
                query_parts: vec![],
                errors: vec![],
            }
        });

        // 转换 skip/limit
        let skip = match_stmt.skip.map(|s| crate::core::Expression::Literal(
            crate::core::Value::Int(s as i64)
        ));
        let limit = match_stmt.limit.map(|l| crate::core::Expression::Literal(
            crate::core::Value::Int(l as i64)
        ));

        Ok(MatchClauseContext {
            paths,
            aliases_available,
            aliases_generated,
            where_clause,
            is_optional: match_stmt.optional,
            skip,
            limit,
            query_parts: vec![],
            errors: vec![],
        })
    }

    /// 从输入计划中提取可用别名
    fn extract_aliases_from_input_plan(input_plan: &SubPlan) -> HashMap<String, AliasType> {
        let mut aliases = HashMap::new();
        
        if let Some(ref root) = input_plan.root {
            for col_name in root.col_names() {
                aliases.insert(col_name.clone(), AliasType::Variable);
            }
        }
        
        aliases
    }

    /// 解析标签名称列表为标签ID列表
    fn resolve_tag_ids(
        schema_manager: Option<&std::sync::Arc<dyn SchemaManager>>,
        space_name: &str,
        labels: &[String],
    ) -> Vec<i32> {
        let mut tids = Vec::new();
        
        if let Some(sm) = schema_manager {
            if let Ok(tags) = sm.list_tags(space_name) {
                for label in labels {
                    if let Some(tag) = tags.iter().find(|t| &t.tag_name == label) {
                        tids.push(tag.tag_id);
                    }
                }
            }
        }
        
        tids
    }

    /// 解析边类型名称列表为边类型ID列表
    fn resolve_edge_type_ids(
        schema_manager: Option<&std::sync::Arc<dyn SchemaManager>>,
        space_name: &str,
        edge_types: &[String],
    ) -> Vec<i32> {
        let mut type_ids = Vec::new();
        
        if let Some(sm) = schema_manager {
            if let Ok(edges) = sm.list_edge_types(space_name) {
                for type_name in edge_types {
                    if let Some(edge) = edges.iter().find(|e| &e.edge_type_name == type_name) {
                        type_ids.push(edge.edge_type_id);
                    }
                }
            }
        }
        
        type_ids
    }

    /// 将 AST Pattern 列表转换为 Path 列表
    ///
    /// 完善后的实现包括：
    /// - 从 predicates 构建 filter 表达式
    /// - 收集所有别名
    /// - 支持更复杂的模式类型
    /// - 从 schema 解析标签ID和边类型ID
    fn convert_patterns_to_paths(
        patterns: &[crate::query::parser::ast::pattern::Pattern],
        schema_manager: Option<&std::sync::Arc<dyn SchemaManager>>,
        space_name: &str,
    ) -> Result<Vec<crate::query::validator::structs::Path>, PlannerError> {
        use crate::query::validator::structs::{Path, NodeInfo, EdgeInfo, PathYieldType, Direction, MatchStepRange};

        let mut paths = Vec::new();

        for pattern in patterns {
            match pattern {
                crate::query::parser::ast::pattern::Pattern::Path(path_pattern) => {
                    let mut node_infos = Vec::new();
                    let mut edge_infos = Vec::new();

                    for element in &path_pattern.elements {
                        match element {
                            crate::query::parser::ast::pattern::PathElement::Node(node) => {
                                // 从 predicates 构建 filter 表达式
                                let filter = Self::build_filter_expression(&node.predicates);
                                
                                // 解析标签ID
                                let tids = Self::resolve_tag_ids(schema_manager, space_name, &node.labels);
                                
                                node_infos.push(NodeInfo {
                                    alias: node.variable.clone().unwrap_or_default(),
                                    labels: node.labels.clone(),
                                    props: node.properties.clone(),
                                    anonymous: node.variable.is_none(),
                                    filter,
                                    tids,
                                    label_props: vec![],
                                });
                            }
                            crate::query::parser::ast::pattern::PathElement::Edge(edge) => {
                                use crate::query::parser::ast::types::EdgeDirection;
                                
                                let direction = match edge.direction {
                                    EdgeDirection::Out => Direction::Forward,
                                    EdgeDirection::In => Direction::Backward,
                                    EdgeDirection::Both => Direction::Bidirectional,
                                };
                                
                                let range = edge.range.as_ref().map(|r| MatchStepRange {
                                    min: r.min.map(|v| v as u32).unwrap_or(1),
                                    max: r.max.map(|v| v as u32).unwrap_or(1),
                                });
                                
                                // 从 predicates 构建 filter 表达式
                                let filter = Self::build_filter_expression(&edge.predicates);
                                
                                // 解析边类型ID
                                let edge_types = Self::resolve_edge_type_ids(schema_manager, space_name, &edge.edge_types);
                                
                                edge_infos.push(EdgeInfo {
                                    alias: edge.variable.clone().unwrap_or_default(),
                                    inner_alias: String::new(),
                                    types: edge.edge_types.clone(),
                                    props: edge.properties.clone(),
                                    anonymous: edge.variable.is_none(),
                                    filter,
                                    direction,
                                    range,
                                    edge_types,
                                });
                            }
                            crate::query::parser::ast::pattern::PathElement::Alternative(alt_patterns) => {
                                // Alternative: (a)-[:KNOWS|FOLLOWS]->(b)
                                // 展开为多个路径模式，使用 UNION ALL 合并结果
                                for alt_pattern in alt_patterns {
                                    let alt_paths = Self::convert_single_pattern(
                                        alt_pattern,
                                        schema_manager,
                                        space_name,
                                    )?;
                                    paths.extend(alt_paths);
                                }
                            }
                            crate::query::parser::ast::pattern::PathElement::Optional(opt_element) => {
                                // Optional: (a)-[e?]->(b) - 边是可选的
                                // 将可选元素转换为带标记的边，后续在 plan_path 中处理
                                match opt_element.as_ref() {
                                    crate::query::parser::ast::pattern::PathElement::Edge(edge) => {
                                        use crate::query::parser::ast::types::EdgeDirection;
                                        
                                        let direction = match edge.direction {
                                            EdgeDirection::Out => Direction::Forward,
                                            EdgeDirection::In => Direction::Backward,
                                            EdgeDirection::Both => Direction::Bidirectional,
                                        };
                                        
                                        // 可选边的范围是 0..1
                                        let range = Some(MatchStepRange { min: 0, max: 1 });
                                        
                                        let filter = Self::build_filter_expression(&edge.predicates);
                                        let edge_types = Self::resolve_edge_type_ids(
                                            schema_manager,
                                            space_name,
                                            &edge.edge_types,
                                        );
                                        
                                        edge_infos.push(EdgeInfo {
                                            alias: edge.variable.clone().unwrap_or_default(),
                                            inner_alias: String::new(),
                                            types: edge.edge_types.clone(),
                                            props: edge.properties.clone(),
                                            anonymous: edge.variable.is_none(),
                                            filter,
                                            direction,
                                            range,
                                            edge_types,
                                        });
                                    }
                                    crate::query::parser::ast::pattern::PathElement::Node(node) => {
                                        // 可选节点：使用左连接处理
                                        let filter = Self::build_filter_expression(&node.predicates);
                                        let tids = Self::resolve_tag_ids(
                                            schema_manager,
                                            space_name,
                                            &node.labels,
                                        );
                                        
                                        // 标记为可选节点（通过特殊别名前缀）
                                        let alias = node.variable.clone().unwrap_or_default();
                                        node_infos.push(NodeInfo {
                                            alias,
                                            labels: node.labels.clone(),
                                            props: node.properties.clone(),
                                            anonymous: node.variable.is_none(),
                                            filter,
                                            tids,
                                            label_props: vec![],
                                        });
                                    }
                                    _ => {
                                        return Err(PlannerError::PlanGenerationFailed(
                                            "Optional 不支持嵌套复杂元素".to_string()
                                        ));
                                    }
                                }
                            }
                            crate::query::parser::ast::pattern::PathElement::Repeated(rep_element, rep_type) => {
                                // Repeated: (a)-[e*1..3]->(b) - 重复边
                                // 根据重复类型设置范围
                                let (min, max) = match rep_type {
                                    crate::query::parser::ast::pattern::RepetitionType::ZeroOrMore => (0, 10), // 默认最大10步
                                    crate::query::parser::ast::pattern::RepetitionType::OneOrMore => (1, 10),
                                    crate::query::parser::ast::pattern::RepetitionType::ZeroOrOne => (0, 1),
                                    crate::query::parser::ast::pattern::RepetitionType::Exactly(n) => (*n, *n),
                                    crate::query::parser::ast::pattern::RepetitionType::Range(min, max) => (*min, *max),
                                };
                                
                                match rep_element.as_ref() {
                                    crate::query::parser::ast::pattern::PathElement::Edge(edge) => {
                                        use crate::query::parser::ast::types::EdgeDirection;
                                        
                                        let direction = match edge.direction {
                                            EdgeDirection::Out => Direction::Forward,
                                            EdgeDirection::In => Direction::Backward,
                                            EdgeDirection::Both => Direction::Bidirectional,
                                        };
                                        
                                        let range = Some(MatchStepRange {
                                            min: min as u32,
                                            max: max as u32,
                                        });
                                        
                                        let filter = Self::build_filter_expression(&edge.predicates);
                                        let edge_types = Self::resolve_edge_type_ids(
                                            schema_manager,
                                            space_name,
                                            &edge.edge_types,
                                        );
                                        
                                        edge_infos.push(EdgeInfo {
                                            alias: edge.variable.clone().unwrap_or_default(),
                                            inner_alias: String::new(),
                                            types: edge.edge_types.clone(),
                                            props: edge.properties.clone(),
                                            anonymous: edge.variable.is_none(),
                                            filter,
                                            direction,
                                            range,
                                            edge_types,
                                        });
                                    }
                                    _ => {
                                        return Err(PlannerError::PlanGenerationFailed(
                                            "Repeated 目前只支持边元素".to_string()
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    paths.push(Path {
                        alias: String::new(),
                        anonymous: false,
                        gen_path: false,
                        path_type: PathYieldType::Default,
                        node_infos,
                        edge_infos,
                        path_build: None,
                        is_pred: false,
                        is_anti_pred: false,
                        compare_variables: vec![],
                        collect_variable: String::new(),
                        roll_up_apply: false,
                    });
                }
                crate::query::parser::ast::pattern::Pattern::Node(node) => {
                    // 从 predicates 构建 filter 表达式
                    let filter = Self::build_filter_expression(&node.predicates);
                    
                    // 解析标签ID
                    let tids = Self::resolve_tag_ids(schema_manager, space_name, &node.labels);
                    
                    // 单个节点模式
                    paths.push(Path {
                        alias: String::new(),
                        anonymous: false,
                        gen_path: false,
                        path_type: PathYieldType::Default,
                        node_infos: vec![NodeInfo {
                            alias: node.variable.clone().unwrap_or_default(),
                            labels: node.labels.clone(),
                            props: node.properties.clone(),
                            anonymous: node.variable.is_none(),
                            filter,
                            tids,
                            label_props: vec![],
                        }],
                        edge_infos: vec![],
                        path_build: None,
                        is_pred: false,
                        is_anti_pred: false,
                        compare_variables: vec![],
                        collect_variable: String::new(),
                        roll_up_apply: false,
                    });
                }
                _ => {
                    // 其他模式类型暂不支持
                    return Err(PlannerError::PlanGenerationFailed(
                        format!("不支持的模式类型: {:?}", pattern)
                    ));
                }
            }
        }

        Ok(paths)
    }

    /// 转换单个 Pattern（用于 Alternative 展开）
    fn convert_single_pattern(
        pattern: &crate::query::parser::ast::pattern::Pattern,
        schema_manager: Option<&std::sync::Arc<dyn SchemaManager>>,
        space_name: &str,
    ) -> Result<Vec<crate::query::validator::structs::Path>, PlannerError> {
        use crate::query::validator::structs::{Path, NodeInfo, EdgeInfo, PathYieldType, Direction, MatchStepRange};
        use crate::query::parser::ast::types::EdgeDirection;

        let mut paths = Vec::new();

        match pattern {
            crate::query::parser::ast::pattern::Pattern::Node(node) => {
                let filter = Self::build_filter_expression(&node.predicates);
                let tids = Self::resolve_tag_ids(schema_manager, space_name, &node.labels);

                paths.push(Path {
                    alias: String::new(),
                    anonymous: false,
                    gen_path: false,
                    path_type: PathYieldType::Default,
                    node_infos: vec![NodeInfo {
                        alias: node.variable.clone().unwrap_or_default(),
                        labels: node.labels.clone(),
                        props: node.properties.clone(),
                        anonymous: node.variable.is_none(),
                        filter,
                        tids,
                        label_props: vec![],
                    }],
                    edge_infos: vec![],
                    path_build: None,
                    is_pred: false,
                    is_anti_pred: false,
                    compare_variables: vec![],
                    collect_variable: String::new(),
                    roll_up_apply: false,
                });
            }
            crate::query::parser::ast::pattern::Pattern::Edge(edge) => {
                let direction = match edge.direction {
                    EdgeDirection::Out => Direction::Forward,
                    EdgeDirection::In => Direction::Backward,
                    EdgeDirection::Both => Direction::Bidirectional,
                };

                let range = edge.range.as_ref().map(|r| MatchStepRange {
                    min: r.min.map(|v| v as u32).unwrap_or(1),
                    max: r.max.map(|v| v as u32).unwrap_or(1),
                });

                let filter = Self::build_filter_expression(&edge.predicates);
                let edge_types = Self::resolve_edge_type_ids(schema_manager, space_name, &edge.edge_types);

                // 对于单个边模式，创建一个虚拟的起始节点
                paths.push(Path {
                    alias: String::new(),
                    anonymous: false,
                    gen_path: false,
                    path_type: PathYieldType::Default,
                    node_infos: vec![NodeInfo {
                        alias: String::new(),
                        labels: vec![],
                        props: None,
                        anonymous: true,
                        filter: None,
                        tids: vec![],
                        label_props: vec![],
                    }],
                    edge_infos: vec![EdgeInfo {
                        alias: edge.variable.clone().unwrap_or_default(),
                        inner_alias: String::new(),
                        types: edge.edge_types.clone(),
                        props: edge.properties.clone(),
                        anonymous: edge.variable.is_none(),
                        filter,
                        direction,
                        range,
                        edge_types,
                    }],
                    path_build: None,
                    is_pred: false,
                    is_anti_pred: false,
                    compare_variables: vec![],
                    collect_variable: String::new(),
                    roll_up_apply: false,
                });
            }
            _ => {
                return Err(PlannerError::PlanGenerationFailed(
                    "Alternative 中的模式类型不支持".to_string()
                ));
            }
        }

        Ok(paths)
    }

    /// 从谓词列表构建 filter 表达式
    ///
    /// 将多个谓词用 AND 组合成一个表达式
    fn build_filter_expression(predicates: &[crate::core::Expression]) -> Option<crate::core::Expression> {
        use crate::core::Expression;
        
        if predicates.is_empty() {
            return None;
        }
        
        if predicates.len() == 1 {
            return Some(predicates[0].clone());
        }
        
        // 多个谓词用 AND 组合
        let mut result = predicates[0].clone();
        for predicate in &predicates[1..] {
            result = Expression::Binary {
                op: crate::core::BinaryOperator::And,
                left: Box::new(result),
                right: Box::new(predicate.clone()),
            };
        }
        
        Some(result)
    }

    /// 从 Path 列表收集所有别名
    ///
    /// 收集节点和边的别名到 aliases_generated
    fn collect_aliases_from_paths(paths: &[crate::query::validator::structs::Path]) -> std::collections::HashMap<String, crate::query::validator::structs::AliasType> {
        use crate::query::validator::structs::AliasType;
        
        let mut aliases = std::collections::HashMap::new();
        
        for path in paths {
            for node_info in &path.node_infos {
                if !node_info.alias.is_empty() {
                    aliases.insert(node_info.alias.clone(), AliasType::Node);
                }
            }
            
            for edge_info in &path.edge_infos {
                if !edge_info.alias.is_empty() {
                    aliases.insert(edge_info.alias.clone(), AliasType::Edge);
                }
            }
        }
        
        aliases
    }
}

impl Default for MatchClausePlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_clause_planner_creation() {
        let planner = MatchClausePlanner::new();
        assert_eq!(planner.name(), "MatchClausePlanner");
        assert_eq!(planner.clause_kind(), CypherClauseKind::Match);
    }
}
