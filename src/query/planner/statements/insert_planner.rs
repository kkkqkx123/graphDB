//! 插入操作规划器
//!
//! 处理 INSERT VERTEX 和 INSERT EDGE 语句的查询规划

use crate::query::context::ast::AstContext;
use crate::query::parser::ast::{InsertStmt, InsertTarget, Stmt, VertexRow};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{
        insert_nodes::{EdgeInsertInfo, InsertEdgesNode, InsertVerticesNode, VertexInsertInfo},
        ArgumentNode, ProjectNode,
    },
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::validator::YieldColumn;
use crate::core::Expression;

/// 插入操作规划器
/// 负责将 INSERT 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct InsertPlanner;

impl InsertPlanner {
    /// 创建新的插入规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查 AST 上下文是否匹配插入操作
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        matches!(ast_ctx.sentence(), Some(Stmt::Insert(_)))
    }

    /// 获取匹配和实例化函数（静态注册版本）
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiateEnum {
        crate::query::planner::planner::MatchAndInstantiateEnum::Insert(Self::new())
    }

    /// 从 AstContext 提取 InsertStmt
    fn extract_insert_stmt(&self, ast_ctx: &AstContext) -> Result<InsertStmt, PlannerError> {
        match ast_ctx.sentence() {
            Some(Stmt::Insert(insert_stmt)) => Ok(insert_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "AST 上下文中不包含 INSERT 语句".to_string(),
            )),
        }
    }

    /// 构建顶点插入信息
    fn build_vertex_insert_info(
        &self,
        space_name: String,
        tag_name: String,
        prop_names: Vec<String>,
        values: Vec<VertexRow>,
    ) -> Result<VertexInsertInfo, PlannerError> {
        // 将 VertexRow 转换为 (Expression, Vec<Expression>) 格式
        // 暂时只支持单 Tag，所以只取第一个 tag 的值
        let converted_values: Vec<(Expression, Vec<Expression>)> = values
            .into_iter()
            .map(|row| {
                let props = row.tag_values.into_iter().next().unwrap_or_default();
                (row.vid, props)
            })
            .collect();
        
        Ok(VertexInsertInfo {
            space_name,
            tag_name,
            prop_names,
            values: converted_values,
        })
    }

    /// 构建边插入信息
    fn build_edge_insert_info(
        &self,
        space_name: String,
        edge_name: String,
        prop_names: Vec<String>,
        edges: Vec<(Expression, Expression, Option<Expression>, Vec<Expression>)>,
    ) -> EdgeInsertInfo {
        EdgeInsertInfo {
            space_name,
            edge_name,
            prop_names,
            edges,
        }
    }

    /// 创建插入结果投影列
    fn create_yield_columns(&self, count: usize) -> Vec<YieldColumn> {
        vec![YieldColumn {
            expression: Expression::literal(crate::core::Value::Int(count as i64)),
            alias: "inserted_count".to_string(),
            is_matched: false,
        }]
    }
}

impl Planner for InsertPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 获取空间名称
        let space_name = ast_ctx.space().space_name.clone();

        // 提取 INSERT 语句
        let insert_stmt = self.extract_insert_stmt(ast_ctx)?;

        // 创建参数节点
        let arg_node = ArgumentNode::new(next_node_id(), "insert_args");

        // 根据 INSERT 目标类型创建相应的插入节点
        let (insert_node, inserted_count) = match &insert_stmt.target {
            InsertTarget::Vertices { tags, values } => {
                let count = values.len();
                // 暂时只支持单 Tag 插入，后续需要扩展支持多 Tag
                let tag_spec = tags.first().ok_or_else(|| {
                    PlannerError::PlanGenerationFailed("INSERT VERTEX must specify at least one tag".to_string())
                })?;
                let info = self.build_vertex_insert_info(
                    space_name,
                    tag_spec.tag_name.clone(),
                    tag_spec.prop_names.clone(),
                    values.clone(),
                )?;
                (
                    PlanNodeEnum::InsertVertices(InsertVerticesNode::new(next_node_id(), info)),
                    count,
                )
            }
            InsertTarget::Edge {
                edge_name,
                prop_names,
                edges,
            } => {
                let count = edges.len();
                let info = self.build_edge_insert_info(space_name, edge_name.clone(), prop_names.clone(), edges.clone());
                (
                    PlanNodeEnum::InsertEdges(InsertEdgesNode::new(next_node_id(), info)),
                    count,
                )
            }
        };

        // 创建投影节点来返回插入结果
        let yield_columns = self.create_yield_columns(inserted_count);

        let project_node = ProjectNode::new(insert_node, yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("创建 ProjectNode 失败: {}", e))
        })?;

        let final_node = PlanNodeEnum::Project(project_node);

        // 创建 SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for InsertPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::query::context::ast::base::AstContext;
    use crate::query::context::validate::types::SpaceInfo;
    use crate::query::parser::ast::{InsertStmt, InsertTarget, Span, TagInsertSpec, VertexRow};

    // 辅助函数：创建常量表达式
    fn lit(val: Value) -> Expression {
        Expression::literal(val)
    }

    fn create_test_span() -> Span {
        use crate::core::types::span::Position;
        Span::new(Position::new(1, 1), Position::new(1, 1))
    }

    fn create_test_ast_ctx_with_insert(target: InsertTarget) -> AstContext {
        let insert_stmt = InsertStmt {
            span: create_test_span(),
            target,
            if_not_exists: false,
        };
        let mut ctx = AstContext::new(None, Some(Stmt::Insert(insert_stmt)));
        ctx.set_space(SpaceInfo {
            space_name: "test_space".to_string(),
            ..Default::default()
        });
        ctx
    }

    #[test]
    fn test_insert_planner_new() {
        let planner = InsertPlanner::new();
        assert!(planner.match_planner(&create_test_ast_ctx_with_insert(InsertTarget::Vertices {
            tags: vec![
                TagInsertSpec {
                    tag_name: "person".to_string(),
                    prop_names: vec!["name".to_string(), "age".to_string()],
                    is_default_props: false,
                },
            ],
            values: vec![
                VertexRow {
                    vid: lit(Value::Int(1)),
                    tag_values: vec![
                        vec![
                            lit(Value::String("Alice".to_string())),
                            lit(Value::Int(30)),
                        ],
                    ],
                },
            ],
        })));
    }

    #[test]
    fn test_match_ast_ctx_with_insert() {
        let ctx = create_test_ast_ctx_with_insert(InsertTarget::Vertices {
            tags: vec![
                TagInsertSpec {
                    tag_name: "person".to_string(),
                    prop_names: vec![],
                    is_default_props: true,
                },
            ],
            values: vec![],
        });
        assert!(InsertPlanner::match_ast_ctx(&ctx));
    }

    #[test]
    fn test_match_ast_ctx_without_insert() {
        let ctx = AstContext::new(None, None);
        assert!(!InsertPlanner::match_ast_ctx(&ctx));
    }

    #[test]
    fn test_extract_insert_stmt_success() {
        let planner = InsertPlanner::new();
        let target = InsertTarget::Vertices {
            tags: vec![
                TagInsertSpec {
                    tag_name: "person".to_string(),
                    prop_names: vec!["name".to_string()],
                    is_default_props: false,
                },
            ],
            values: vec![],
        };
        let ctx = create_test_ast_ctx_with_insert(target.clone());
        let result = planner.extract_insert_stmt(&ctx).expect("Failed to extract insert statement");
        assert_eq!(result.target, target);
    }

    #[test]
    fn test_extract_insert_stmt_failure() {
        let planner = InsertPlanner::new();
        let ctx = AstContext::new(None, None);
        let result = planner.extract_insert_stmt(&ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("不包含 INSERT 语句"));
    }

    #[test]
    fn test_build_vertex_insert_info() {
        let planner = InsertPlanner::new();
        let info = planner.build_vertex_insert_info(
            "test_space".to_string(),
            "person".to_string(),
            vec!["name".to_string(), "age".to_string()],
            vec![
                VertexRow {
                    vid: lit(Value::Int(1)),
                    tag_values: vec![
                        vec![
                            lit(Value::String("Alice".to_string())),
                            lit(Value::Int(30)),
                        ],
                    ],
                },
            ],
        ).expect("Failed to build vertex insert info");
        assert_eq!(info.space_name, "test_space");
        assert_eq!(info.tag_name, "person");
        assert_eq!(info.prop_names.len(), 2);
        assert_eq!(info.values.len(), 1);
    }

    #[test]
    fn test_build_edge_insert_info() {
        let planner = InsertPlanner::new();
        let info = planner.build_edge_insert_info(
            "test_space".to_string(),
            "follow".to_string(),
            vec!["since".to_string()],
            vec![(
                lit(Value::Int(1)),
                lit(Value::Int(2)),
                Some(lit(Value::Int(0))),
                vec![lit(Value::String("2023".to_string()))],
            )],
        );
        assert_eq!(info.space_name, "test_space");
        assert_eq!(info.edge_name, "follow");
        assert_eq!(info.prop_names.len(), 1);
        assert_eq!(info.edges.len(), 1);
    }

    #[test]
    fn test_create_yield_columns() {
        let planner = InsertPlanner::new();
        let columns = planner.create_yield_columns(5);
        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0].alias, "inserted_count");
    }

    #[test]
    fn test_transform_insert_vertices() {
        let mut planner = InsertPlanner::new();
        let target = InsertTarget::Vertices {
            tags: vec![
                TagInsertSpec {
                    tag_name: "person".to_string(),
                    prop_names: vec!["name".to_string()],
                    is_default_props: false,
                },
            ],
            values: vec![
                VertexRow {
                    vid: lit(Value::Int(1)),
                    tag_values: vec![vec![lit(Value::String("Alice".to_string()))]],
                },
                VertexRow {
                    vid: lit(Value::Int(2)),
                    tag_values: vec![vec![lit(Value::String("Bob".to_string()))]],
                },
            ],
        };
        let ctx = create_test_ast_ctx_with_insert(target);
        let result = planner.transform(&ctx);
        assert!(result.is_ok());
        let sub_plan = result.expect("Failed to transform insert statement");
        assert!(sub_plan.root.is_some());
    }

    #[test]
    fn test_transform_insert_edge() {
        let mut planner = InsertPlanner::new();
        let target = InsertTarget::Edge {
            edge_name: "follow".to_string(),
            prop_names: vec!["since".to_string()],
            edges: vec![(
                lit(Value::Int(1)),
                lit(Value::Int(2)),
                Some(lit(Value::Int(0))),
                vec![lit(Value::String("2023".to_string()))],
            )],
        };
        let ctx = create_test_ast_ctx_with_insert(target);
        let result = planner.transform(&ctx);
        assert!(result.is_ok());
        let sub_plan = result.expect("Failed to transform insert statement");
        assert!(sub_plan.root.is_some());
    }

    #[test]
    fn test_transform_without_insert_stmt() {
        let mut planner = InsertPlanner::new();
        let ctx = AstContext::new(None, None);
        let result = planner.transform(&ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_impl() {
        let planner: InsertPlanner = Default::default();
        let ctx = create_test_ast_ctx_with_insert(InsertTarget::Vertices {
            tags: vec![
                TagInsertSpec {
                    tag_name: "test".to_string(),
                    prop_names: vec![],
                    is_default_props: true,
                },
            ],
            values: vec![],
        });
        assert!(planner.match_planner(&ctx));
    }
}
