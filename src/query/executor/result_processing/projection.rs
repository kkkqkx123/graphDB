//! 列投影执行器
//!
//! ProjectExecutor - 选择和投影输出列

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::Expression;
use crate::core::Value;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::{DefaultExpressionContext, ExpressionContext};
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::base::InputExecutor;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::traits::{ExecutionResult, Executor};
use crate::storage::StorageEngine;

/// 投影列定义
#[derive(Debug, Clone)]
pub struct ProjectionColumn {
    pub name: String,           // 输出列名
    pub expression: Expression, // 投影表达式
}

impl ProjectionColumn {
    pub fn new(name: String, expression: Expression) -> Self {
        Self { name, expression }
    }
}

/// ProjectExecutor - 投影执行器
///
/// 执行列投影操作，支持表达式求值和列重命名
pub struct ProjectExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    columns: Vec<ProjectionColumn>, // 投影列定义
    input_executor: Option<Box<ExecutorEnum<S>>>,
}

impl<S: StorageEngine> ProjectExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, columns: Vec<ProjectionColumn>) -> Self {
        Self {
            base: BaseExecutor::new(id, "ProjectExecutor".to_string(), storage),
            columns,
            input_executor: None,
        }
    }

    /// 处理单行数据的投影
    fn project_row(&self, row: &[Value], col_names: &[String]) -> DBResult<Vec<Value>> {
        let mut projected_row = Vec::new();

        // 为当前行创建评估上下文
        let mut context = DefaultExpressionContext::new();

        // 将当前行的值设置为上下文变量
        for (i, col_name) in col_names.iter().enumerate() {
            if i < row.len() {
                context.set_variable(col_name.clone(), row[i].clone());
            }
        }

        // 对每个投影列进行求值
        for column in &self.columns {
            match ExpressionEvaluator::evaluate(&column.expression, &mut context) {
                Ok(value) => projected_row.push(value),
                Err(e) => {
                    return Err(DBError::Expression(
                        crate::core::error::ExpressionError::function_error(format!(
                            "Failed to evaluate projection expression '{}': {}",
                            column.name, e
                        )),
                    ));
                }
            }
        }

        Ok(projected_row)
    }

    /// 处理数据集投影
    fn project_dataset(
        &self,
        dataset: crate::core::value::DataSet,
    ) -> DBResult<crate::core::value::DataSet> {
        let mut result_dataset = crate::core::value::DataSet::new();

        // 设置新的列名
        result_dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();

        // 对每一行进行投影
        for row in dataset.rows {
            let projected_row = self.project_row(&row, &dataset.col_names)?;
            result_dataset.rows.push(projected_row);
        }

        Ok(result_dataset)
    }

    /// 处理顶点列表投影
    fn project_vertices(
        &self,
        vertices: Vec<crate::core::Vertex>,
    ) -> DBResult<crate::core::value::DataSet> {
        let mut result_dataset = crate::core::value::DataSet::new();

        // 设置列名
        result_dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();

        // 对每个顶点进行投影
        for vertex in vertices {
            let mut context = DefaultExpressionContext::new();
            // 设置顶点信息
            context.set_variable(
                "_vertex".to_string(),
                Value::Vertex(Box::new(vertex.clone())),
            );

            // 设置顶点ID作为变量
            context.set_variable("id".to_string(), *vertex.vid.clone());

            // 将顶点属性也设置为变量，以便InputProperty可以访问
            for (prop_name, prop_value) in &vertex.properties {
                context.set_variable(prop_name.clone(), prop_value.clone());
            }

            let mut projected_row = Vec::new();
            for column in &self.columns {
                match ExpressionEvaluator::evaluate(&column.expression, &mut context) {
                    Ok(value) => projected_row.push(value),
                    Err(e) => {
                        return Err(DBError::Expression(
                            crate::core::error::ExpressionError::function_error(format!(
                                "Failed to evaluate projection expression '{}': {}",
                                column.name, e
                            )),
                        ));
                    }
                }
            }
            result_dataset.rows.push(projected_row);
        }

        Ok(result_dataset)
    }

    /// 处理边列表投影
    fn project_edges(
        &self,
        edges: Vec<crate::core::Edge>,
    ) -> DBResult<crate::core::value::DataSet> {
        let mut result_dataset = crate::core::value::DataSet::new();

        // 设置列名
        result_dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();

        // 对每个边进行投影
        for edge in edges {
            let mut context = DefaultExpressionContext::new();
            // 设置边信息
            context.set_variable("_edge".to_string(), Value::Edge(edge.clone()));

            // 设置边属性作为变量
            context.set_variable("src".to_string(), *edge.src.clone());
            context.set_variable("dst".to_string(), *edge.dst.clone());
            context.set_variable(
                "edge_type".to_string(),
                Value::String(edge.edge_type.clone()),
            );
            context.set_variable("ranking".to_string(), Value::Int(edge.ranking as i64));

            let mut projected_row = Vec::new();
            for column in &self.columns {
                match ExpressionEvaluator::evaluate(&column.expression, &mut context) {
                    Ok(value) => projected_row.push(value),
                    Err(e) => {
                        return Err(DBError::Expression(
                            crate::core::error::ExpressionError::function_error(format!(
                                "Failed to evaluate projection expression '{}': {}",
                                column.name, e
                            )),
                        ));
                    }
                }
            }
            result_dataset.rows.push(projected_row);
        }

        Ok(result_dataset)
    }
}

impl<S: StorageEngine + Send + 'static> InputExecutor<S> for ProjectExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for ProjectExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            ExecutionResult::DataSet(crate::core::value::DataSet::new())
        };

        let projected_result = match input_result {
            ExecutionResult::DataSet(dataset) => {
                let projected_dataset = self.project_dataset(dataset)?;
                ExecutionResult::DataSet(projected_dataset)
            }
            ExecutionResult::Vertices(vertices) => {
                let projected_dataset = self.project_vertices(vertices)?;
                ExecutionResult::DataSet(projected_dataset)
            }
            ExecutionResult::Edges(edges) => {
                let projected_dataset = self.project_edges(edges)?;
                ExecutionResult::DataSet(projected_dataset)
            }
            ExecutionResult::Values(values) => {
                let mut dataset = crate::core::value::DataSet::new();
                dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();

                for value in values {
                    dataset.rows.push(vec![value]);
                }
                ExecutionResult::DataSet(dataset)
            }
            ExecutionResult::Paths(paths) => {
                let mut dataset = crate::core::value::DataSet::new();
                dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();

                for path in paths {
                    let mut context = DefaultExpressionContext::new();
                    context.set_variable("path_length".to_string(), Value::Int(path.len() as i64));
                    context
                        .set_variable("src".to_string(), Value::String(path.src.vid.to_string()));

                    let mut projected_row = Vec::new();
                    for column in &self.columns {
                        match ExpressionEvaluator::evaluate(&column.expression, &mut context) {
                            Ok(value) => projected_row.push(value),
                            Err(e) => {
                                return Err(DBError::Expression(
                                    crate::core::error::ExpressionError::function_error(format!(
                                        "Failed to evaluate projection expression '{}': {}",
                                        column.name, e
                                    )),
                                ));
                            }
                        }
                    }
                    dataset.rows.push(projected_row);
                }
                ExecutionResult::DataSet(dataset)
            }
            ExecutionResult::Count(count) => {
                let mut dataset = crate::core::value::DataSet::new();
                dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();
                dataset.rows.push(vec![Value::Int(count as i64)]);
                ExecutionResult::DataSet(dataset)
            }
            ExecutionResult::Success => ExecutionResult::Success,
            ExecutionResult::Error(_) => input_result,
            ExecutionResult::Result(_) => input_result,
        };

        Ok(projected_result)
    }

    fn open(&mut self) -> DBResult<()> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::{DataSet, Value};
    use crate::core::{BinaryOperator, Expression};
    use crate::query::executor::base::BaseExecutor;
    use crate::query::executor::executor_enum::ExecutorEnum;
    use crate::query::executor::HasStorage;
    use crate::query::executor::traits::{ExecutionResult, Executor, ExecutorStats};
    use crate::storage::test_mock::MockStorage;

    #[tokio::test]
    async fn test_simple_projection() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建简单的投影：选择第一列
        let columns = vec![ProjectionColumn::new(
            "projected_col1".to_string(),
            Expression::Variable("col1".to_string()),
        )];

        let mut executor = ProjectExecutor::new(1, storage, columns);

        // 创建测试数据集
        let mut input_dataset = DataSet::new();
        input_dataset.col_names = vec!["col1".to_string(), "col2".to_string()];
        input_dataset.rows = vec![
            vec![Value::Int(1), Value::String("Alice".to_string())],
            vec![Value::Int(2), Value::String("Bob".to_string())],
            vec![Value::Int(3), Value::String("Charlie".to_string())],
        ];

        // 创建模拟输入执行器
        let input_executor = ExecutorEnum::Base(BaseExecutor::new(0, "MockInputExecutor".to_string(), Arc::new(Mutex::new(MockStorage))));
        executor.set_input(input_executor);

        // 执行投影
        let result = executor
            .execute()
            .await
            .expect("Projection executor should execute successfully");

        // 验证结果
        match result {
            ExecutionResult::DataSet(dataset) => {
                assert_eq!(dataset.col_names, vec!["projected_col1"]);
                assert_eq!(dataset.rows.len(), 3);
                assert_eq!(dataset.rows[0], vec![Value::Int(1)]);
                assert_eq!(dataset.rows[1], vec![Value::Int(2)]);
                assert_eq!(dataset.rows[2], vec![Value::Int(3)]);
            }
            _ => panic!("期望DataSet结果"),
        }
    }

    #[tokio::test]
    async fn test_expression_projection() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建表达式投影：计算两列之和
        let columns = vec![ProjectionColumn::new(
            "sum".to_string(),
            Expression::Binary {
                left: Box::new(Expression::Variable("col1".to_string())),
                op: BinaryOperator::Add,
                right: Box::new(Expression::Variable("col2".to_string())),
            },
        )];

        let mut executor = ProjectExecutor::new(1, storage, columns);

        // 创建测试数据集
        let mut input_dataset = DataSet::new();
        input_dataset.col_names = vec!["col1".to_string(), "col2".to_string()];
        input_dataset.rows = vec![
            vec![Value::Int(1), Value::Int(10)],
            vec![Value::Int(2), Value::Int(20)],
            vec![Value::Int(3), Value::Int(30)],
        ];

        // 创建模拟输入执行器
        let input_executor = ExecutorEnum::Base(BaseExecutor::new(0, "MockInputExecutor".to_string(), Arc::new(Mutex::new(MockStorage))));
        executor.set_input(input_executor);

        // 执行投影
        let result = executor
            .execute()
            .await
            .expect("Projection executor should execute successfully");

        // 验证结果
        match result {
            ExecutionResult::DataSet(dataset) => {
                assert_eq!(dataset.col_names, vec!["sum"]);
                assert_eq!(dataset.rows.len(), 3);
                assert_eq!(dataset.rows[0], vec![Value::Int(11)]);
                assert_eq!(dataset.rows[1], vec![Value::Int(22)]);
                assert_eq!(dataset.rows[2], vec![Value::Int(33)]);
            }
            _ => panic!("期望DataSet结果"),
        }
    }

    #[tokio::test]
    async fn test_vertex_projection() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建顶点投影
        let columns = vec![
            ProjectionColumn::new(
                "vertex_id".to_string(),
                Expression::Variable("id".to_string()),
            ),
            ProjectionColumn::new(
                "name".to_string(),
                Expression::Variable("name".to_string()),
            ),
        ];

        let mut executor = ProjectExecutor::new(1, storage, columns);

        // 创建测试顶点
        let vertex1 = crate::core::Vertex {
            vid: Box::new(Value::Int(1)),
            id: 1,
            tags: vec![crate::core::vertex_edge_path::Tag {
                name: "person".to_string(),
                properties: std::collections::HashMap::new(),
            }],
            properties: std::collections::HashMap::from([
                ("name".to_string(), Value::String("Alice".to_string())),
                ("age".to_string(), Value::Int(25)),
            ]),
        };

        let vertex2 = crate::core::Vertex {
            vid: Box::new(Value::Int(2)),
            id: 2,
            tags: vec![crate::core::vertex_edge_path::Tag {
                name: "person".to_string(),
                properties: std::collections::HashMap::new(),
            }],
            properties: std::collections::HashMap::from([
                ("name".to_string(), Value::String("Bob".to_string())),
                ("age".to_string(), Value::Int(30)),
            ]),
        };

        // 创建模拟输入执行器
        let input_executor = ExecutorEnum::Base(BaseExecutor::new(0, "MockInputExecutor".to_string(), Arc::new(Mutex::new(MockStorage))));
        executor.set_input(input_executor);

        // 执行投影
        let result = executor
            .execute()
            .await
            .expect("Projection executor should execute successfully");

        // 验证结果
        match result {
            ExecutionResult::DataSet(dataset) => {
                assert_eq!(dataset.col_names, vec!["vertex_id", "name"]);
                assert_eq!(dataset.rows.len(), 2);
                assert_eq!(
                    dataset.rows[0],
                    vec![Value::Int(1), Value::String("Alice".to_string())]
                );
                assert_eq!(
                    dataset.rows[1],
                    vec![Value::Int(2), Value::String("Bob".to_string())]
                );
            }
            _ => panic!("期望DataSet结果"),
        }
    }

    #[tokio::test]
    async fn test_edge_projection() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建边投影
        let columns = vec![
            ProjectionColumn::new(
                "src_id".to_string(),
                Expression::Variable("src".to_string()),
            ),
            ProjectionColumn::new(
                "dst_id".to_string(),
                Expression::Variable("dst".to_string()),
            ),
            ProjectionColumn::new(
                "edge_type".to_string(),
                Expression::Variable("edge_type".to_string()),
            ),
        ];

        let mut executor = ProjectExecutor::new(1, storage, columns);

        // 创建测试边
        let edge1 = crate::core::Edge {
            src: Box::new(Value::Int(1)),
            dst: Box::new(Value::Int(2)),
            edge_type: "knows".to_string(),
            ranking: 0,
            id: 1,
            props: std::collections::HashMap::from([("since".to_string(), Value::Int(2020))]),
        };

        let edge2 = crate::core::Edge {
            src: Box::new(Value::Int(2)),
            dst: Box::new(Value::Int(3)),
            edge_type: "works_with".to_string(),
            ranking: 0,
            id: 2,
            props: std::collections::HashMap::from([(
                "project".to_string(),
                Value::String("GraphDB".to_string()),
            )]),
        };

        // 创建模拟输入执行器
        let input_executor = ExecutorEnum::Base(BaseExecutor::new(0, "MockInputExecutor".to_string(), Arc::new(Mutex::new(MockStorage))));
        executor.set_input(input_executor);

        // 执行投影
        let result = executor
            .execute()
            .await
            .expect("Projection executor should execute successfully");

        // 验证结果
        match result {
            ExecutionResult::DataSet(dataset) => {
                assert_eq!(dataset.col_names, vec!["src_id", "dst_id", "edge_type"]);
                assert_eq!(dataset.rows.len(), 2);
                assert_eq!(
                    dataset.rows[0],
                    vec![
                        Value::Int(1),
                        Value::Int(2),
                        Value::String("knows".to_string())
                    ]
                );
                assert_eq!(
                    dataset.rows[1],
                    vec![
                        Value::Int(2),
                        Value::Int(3),
                        Value::String("works_with".to_string())
                    ]
                );
            }
            _ => panic!("期望DataSet结果"),
        }
    }
}
