//! CREATE 数据语句规划器
//!
//! 处理 Cypher 风格 CREATE 语句的查询规划
//! 支持 CREATE (n:Label {props}) 和 CREATE (a)-[:Type]->(b) 语法

use crate::query::context::QueryContext;
use crate::query::parser::ast::{CreateStmt, CreateTarget, Stmt};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{
        insert_nodes::{EdgeInsertInfo, InsertEdgesNode, InsertVerticesNode, VertexInsertInfo, TagInsertSpec},
        ArgumentNode, ProjectNode,
    },
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::core::YieldColumn;
use crate::core::{Expression, Value};
use std::sync::Arc;

/// CREATE 数据语句规划器
/// 负责将 Cypher 风格的 CREATE 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct CreatePlanner;

impl CreatePlanner {
    /// 创建新的 CREATE 规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 判断是否为数据创建语句（而非 Schema 创建）
    fn is_data_create(stmt: &CreateStmt) -> bool {
        matches!(&stmt.target,
            CreateTarget::Node { .. } |
            CreateTarget::Edge { .. } |
            CreateTarget::Path { .. }
        )
    }

    /// 从 Stmt 提取 CreateStmt
    fn extract_create_stmt(&self, stmt: &Stmt) -> Result<CreateStmt, PlannerError> {
        match stmt {
            Stmt::Create(create_stmt) => Ok(create_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不包含 CREATE".to_string(),
            )),
        }
    }

    /// 构建顶点插入信息
    fn build_vertex_insert_info(
        &self,
        space_name: String,
        labels: &[String],
        properties: &[(String, Expression)],
    ) -> Result<VertexInsertInfo, PlannerError> {
        if labels.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "CREATE 节点必须指定至少一个 Label".to_string()
            ));
        }

        // 转换标签规范
        let tag_specs: Vec<TagInsertSpec> = labels
            .iter()
            .map(|label| TagInsertSpec {
                tag_name: label.clone(),
                prop_names: properties.iter().map(|(k, _)| k.clone()).collect(),
            })
            .collect();

        // 属性值
        let prop_values: Vec<Expression> = properties
            .iter()
            .map(|(_, v)| v.clone())
            .collect();

        // 注意：这里需要 VID，但在 Cypher 语法中通常不直接指定
        // 我们需要生成一个 VID 或使用变量引用
        // 暂时使用一个占位符，实际执行时会处理
        let vid_expr = Expression::literal(Value::Null(crate::core::NullType::default()));

        Ok(VertexInsertInfo {
            space_name,
            tags: tag_specs,
            values: vec![(vid_expr, vec![prop_values])],
        })
    }

    /// 构建边插入信息
    fn build_edge_insert_info(
        &self,
        space_name: String,
        edge_type: String,
        src_vid: Expression,
        dst_vid: Expression,
        properties: &[(String, Expression)],
    ) -> EdgeInsertInfo {
        let prop_names: Vec<String> = properties.iter().map(|(k, _)| k.clone()).collect();
        let prop_values: Vec<Expression> = properties.iter().map(|(_, v)| v.clone()).collect();

        EdgeInsertInfo {
            space_name,
            edge_name: edge_type,
            prop_names,
            edges: vec![(src_vid, dst_vid, None, prop_values)],
        }
    }

    /// 创建结果投影列
    fn create_yield_columns(&self, count: usize) -> Vec<YieldColumn> {
        vec![YieldColumn {
            expression: Expression::literal(Value::Int(count as i64)),
            alias: "created_count".to_string(),
            is_matched: false,
        }]
    }
}

impl Planner for CreatePlanner {
    fn transform(
        &mut self,
        stmt: &Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        // 获取空间名称
        let space_name = qctx.rctx()
            .and_then(|rctx| rctx.space_name())
            .unwrap_or_else(|| "default".to_string());

        // 提取 CREATE 语句
        let create_stmt = self.extract_create_stmt(stmt)?;

        // 创建参数节点
        let arg_node = ArgumentNode::new(next_node_id(), "create_args");

        // 根据 CREATE 目标类型创建相应的插入节点
        let (insert_node, created_count) = match &create_stmt.target {
            CreateTarget::Node { variable: _, labels, properties } => {
                // 解析属性
                let props = if let Some(expr) = properties {
                    Self::extract_properties(expr)?
                } else {
                    vec![]
                };

                let info = self.build_vertex_insert_info(
                    space_name,
                    labels,
                    &props,
                )?;

                (
                    PlanNodeEnum::InsertVertices(InsertVerticesNode::new(next_node_id(), info)),
                    1,
                )
            }
            CreateTarget::Edge { variable: _, edge_type, src, dst, properties, direction: _ } => {
                // 解析属性
                let props = if let Some(expr) = properties {
                    Self::extract_properties(expr)?
                } else {
                    vec![]
                };

                let info = self.build_edge_insert_info(
                    space_name,
                    edge_type.clone(),
                    src.clone(),
                    dst.clone(),
                    &props,
                );

                (
                    PlanNodeEnum::InsertEdges(InsertEdgesNode::new(next_node_id(), info)),
                    1,
                )
            }
            CreateTarget::Path { patterns: _ } => {
                // 路径创建需要更复杂的处理
                // 这里简化处理，返回错误提示
                return Err(PlannerError::PlanGenerationFailed(
                    "路径创建尚未完全实现".to_string()
                ));
            }
            _ => {
                return Err(PlannerError::PlanGenerationFailed(
                    "不支持的 CREATE 目标类型".to_string()
                ));
            }
        };

        // 创建投影节点来返回创建结果
        let yield_columns = self.create_yield_columns(created_count);

        let project_node = ProjectNode::new(insert_node, yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("创建 ProjectNode 失败: {}", e))
        })?;

        let final_node = PlanNodeEnum::Project(project_node);

        // 创建 SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Create(create_stmt) => Self::is_data_create(create_stmt),
            _ => false,
        }
    }
}

impl CreatePlanner {
    /// 从表达式中提取属性键值对
    fn extract_properties(expr: &Expression) -> Result<Vec<(String, Expression)>, PlannerError> {
        match expr {
            Expression::Map(map) => {
                let mut result = Vec::new();
                for (key, value) in map {
                    result.push((key.clone(), value.clone()));
                }
                Ok(result)
            }
            _ => Err(PlannerError::PlanGenerationFailed(
                "属性必须是 Map 表达式".to_string()
            )),
        }
    }
}

impl Default for CreatePlanner {
    fn default() -> Self {
        Self::new()
    }
}
