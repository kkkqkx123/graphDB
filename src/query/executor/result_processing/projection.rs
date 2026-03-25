//! Column Projection Executor
//!
//! ProjectExecutor – Selection and projection of output columns
//!
//! CPU-intensive operations are parallelized using Rayon.

use parking_lot::Mutex;
use rayon::prelude::*;
use std::sync::Arc;

use crate::core::error::{DBError, DBResult};
use crate::core::types::ContextualExpression;
use crate::core::Value;
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::base::Executor;
use crate::query::executor::base::InputExecutor;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::expression::evaluator::traits::ExpressionContext;
use crate::query::executor::expression::DefaultExpressionContext;
use crate::query::executor::recursion_detector::ParallelConfig;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::ExecutionResult;
use crate::storage::StorageClient;

/// Projection column definition
#[derive(Debug, Clone)]
pub struct ProjectionColumn {
    pub name: String,                     // Column names
    pub expression: ContextualExpression, // Projection expression
}

impl ProjectionColumn {
    pub fn new(name: String, expression: ContextualExpression) -> Self {
        Self { name, expression }
    }
}

/// ProjectExecutor – The projection executor
///
/// Performs column projection operations, supports the evaluation of expressions, and allows for the renaming of columns.
///
/// CPU-intensive operations are parallelized using Rayon.
pub struct ProjectExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    columns: Vec<ProjectionColumn>,
    input_executor: Option<Box<ExecutorEnum<S>>>,
    /// Parallel computing configuration
    parallel_config: ParallelConfig,
}

impl<S: StorageClient> ProjectExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        columns: Vec<ProjectionColumn>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ProjectExecutor".to_string(), storage, expr_context),
            columns,
            input_executor: None,
            parallel_config: ParallelConfig::default(),
        }
    }

    /// Setting up parallel computing configuration
    pub fn with_parallel_config(mut self, config: ParallelConfig) -> Self {
        self.parallel_config = config;
        self
    }

    /// Projection of single-row data
    fn project_row(&self, row: &[Value], col_names: &[String]) -> DBResult<Vec<Value>> {
        let mut projected_row = Vec::new();

        let mut context = DefaultExpressionContext::new();

        // Set the value of the current row to the context variable.
        for (i, col_name) in col_names.iter().enumerate() {
            if i < row.len() {
                context.set_variable(col_name.clone(), row[i].clone());
            }
        }

        // Evaluate each projected column.
        for column in &self.columns {
            // Extract the Expression from the ContextualExpression.
            let expr = match column.expression.expression() {
                Some(meta) => meta.inner().clone(),
                None => continue,
            };

            match ExpressionEvaluator::evaluate(&expr, &mut context) {
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

    /// Processing data set projections
    ///
    /// Choose the processing method based on the amount of data:
    /// The amount of data is less than single_thread_limit: The processing is done using a single thread.
    /// Large amount of data: Parallel processing using Rayon technology
    fn project_dataset(
        &self,
        dataset: crate::core::value::DataSet,
    ) -> DBResult<crate::core::value::DataSet> {
        let mut result_dataset = crate::core::value::DataSet::new();

        // Set new column names
        result_dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();

        let total_size = dataset.rows.len();

        // Determine whether to use parallel computing based on the parallel configuration.
        if !self.parallel_config.should_use_parallel(total_size) {
            // If the amount of data is small or parallel processing is disabled, single-threaded processing should be used.
            for row in dataset.rows {
                let projected_row = self.project_row(&row, &dataset.col_names)?;
                result_dataset.rows.push(projected_row);
            }
        } else {
            // The amount of data is large; therefore, rayon parallel processing is used for processing it.
            let batch_size = self.parallel_config.calculate_batch_size(total_size);
            let columns = self.columns.clone();
            let col_names = dataset.col_names.clone();

            // Use `par_chunks` from `rayon` for parallel processing.
            let projected_rows: Vec<Vec<Value>> = dataset
                .rows
                .par_chunks(batch_size)
                .flat_map(|chunk| {
                    chunk
                        .iter()
                        .filter_map(|row| {
                            let mut context = DefaultExpressionContext::new();

                            // Set the value of the current row to the context variable.
                            for (i, col_name) in col_names.iter().enumerate() {
                                if i < row.len() {
                                    context.set_variable(col_name.clone(), row[i].clone());
                                }
                            }

                            // Evaluate each projected column.
                            let mut projected_row = Vec::new();
                            for column in &columns {
                                // Extract the Expression from the ContextualExpression.
                                let expr = match column.expression.expression() {
                                    Some(meta) => meta.inner().clone(),
                                    None => return None,
                                };

                                match ExpressionEvaluator::evaluate(&expr, &mut context) {
                                    Ok(value) => projected_row.push(value),
                                    Err(_) => return None, // Skip the rows where the evaluation failed.
                                }
                            }
                            Some(projected_row)
                        })
                        .collect::<Vec<_>>()
                })
                .collect();

            result_dataset.rows = projected_rows;
        }

        Ok(result_dataset)
    }

    /// Processing the projection of the vertex list
    fn project_vertices(
        &self,
        vertices: Vec<crate::core::Vertex>,
    ) -> DBResult<crate::core::value::DataSet> {
        let mut result_dataset = crate::core::value::DataSet::new();

        // Set column names
        result_dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();

        // Project each vertex.
        for vertex in vertices {
            let mut context = DefaultExpressionContext::new();
            // Setting vertex information
            context.set_variable(
                "_vertex".to_string(),
                Value::Vertex(Box::new(vertex.clone())),
            );

            // Set the vertex ID as a variable.
            context.set_variable("id".to_string(), *vertex.vid.clone());

            // Set the vertex properties as variables as well, so that the InputProperty can access them.
            for (prop_name, prop_value) in &vertex.properties {
                context.set_variable(prop_name.clone(), prop_value.clone());
            }

            let mut projected_row = Vec::new();
            for column in &self.columns {
                let expr = match column.expression.expression() {
                    Some(meta) => meta.inner().clone(),
                    None => continue,
                };

                match ExpressionEvaluator::evaluate(&expr, &mut context) {
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

    /// Processing of edge list projections
    fn project_edges(
        &self,
        edges: Vec<crate::core::Edge>,
    ) -> DBResult<crate::core::value::DataSet> {
        let mut result_dataset = crate::core::value::DataSet::new();

        // Set column names
        result_dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();

        // Project each edge.
        for edge in edges {
            let mut context = DefaultExpressionContext::new();
            // Set border information
            context.set_variable("_edge".to_string(), Value::Edge(edge.clone()));

            // Set the edge properties as variables.
            context.set_variable("src".to_string(), *edge.src.clone());
            context.set_variable("dst".to_string(), *edge.dst.clone());
            context.set_variable(
                "edge_type".to_string(),
                Value::String(edge.edge_type.clone()),
            );
            context.set_variable("ranking".to_string(), Value::Int(edge.ranking));

            let mut projected_row = Vec::new();
            for column in &self.columns {
                let expr = match column.expression.expression() {
                    Some(meta) => meta.inner().clone(),
                    None => continue,
                };

                match ExpressionEvaluator::evaluate(&expr, &mut context) {
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

impl<S: StorageClient + Send + 'static> InputExecutor<S> for ProjectExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for ProjectExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute()?
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
                        let expr = match column.expression.expression() {
                            Some(meta) => meta.inner().clone(),
                            None => continue,
                        };

                        match ExpressionEvaluator::evaluate(&expr, &mut context) {
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
            ExecutionResult::Empty => ExecutionResult::Empty,
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

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;
    use crate::core::BinaryOperator;
    use crate::storage::test_mock::MockStorage;

    #[test]
    fn test_simple_projection() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("创建Mock存储失败")));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());

        let expr = crate::core::Expression::Variable("col1".to_string());
        let expr_meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let expr_id = expr_context.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(expr_id, expr_context.clone());

        let columns = vec![ProjectionColumn::new(
            "projected_col1".to_string(),
            ctx_expr,
        )];

        let executor = ProjectExecutor::new(1, storage, columns, expr_context);

        // Create a test dataset
        let mut input_dataset = crate::core::value::DataSet::new();
        input_dataset.col_names = vec!["col1".to_string(), "col2".to_string()];
        input_dataset.rows = vec![
            vec![Value::Int(1), Value::String("Alice".to_string())],
            vec![Value::Int(2), Value::String("Bob".to_string())],
            vec![Value::Int(3), Value::String("Charlie".to_string())],
        ];

        // Without setting the `inputExecutor`, directly call the `project_dataset` method to conduct the test.
        let projected_dataset = executor
            .project_dataset(input_dataset)
            .expect("Projection should succeed");

        // Verification results
        assert_eq!(projected_dataset.col_names, vec!["projected_col1"]);
        assert_eq!(projected_dataset.rows.len(), 3);
        assert_eq!(projected_dataset.rows[0], vec![Value::Int(1)]);
        assert_eq!(projected_dataset.rows[1], vec![Value::Int(2)]);
        assert_eq!(projected_dataset.rows[2], vec![Value::Int(3)]);
    }

    #[test]
    fn test_expression_projection() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("创建Mock存储失败")));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());

        let expr = crate::core::Expression::Binary {
            left: Box::new(crate::core::Expression::Variable("col1".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(crate::core::Expression::Variable("col2".to_string())),
        };
        let expr_meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let expr_id = expr_context.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(expr_id, expr_context.clone());

        let columns = vec![ProjectionColumn::new("sum".to_string(), ctx_expr)];

        let executor = ProjectExecutor::new(1, storage, columns, expr_context);

        // Create a test dataset
        let mut input_dataset = crate::core::value::DataSet::new();
        input_dataset.col_names = vec!["col1".to_string(), "col2".to_string()];
        input_dataset.rows = vec![
            vec![Value::Int(1), Value::Int(10)],
            vec![Value::Int(2), Value::Int(20)],
            vec![Value::Int(3), Value::Int(30)],
        ];

        // Directly call the `project_dataset` method to conduct the test.
        let projected_dataset = executor
            .project_dataset(input_dataset)
            .expect("Projection should succeed");

        // Verification results
        assert_eq!(projected_dataset.col_names, vec!["sum"]);
        assert_eq!(projected_dataset.rows.len(), 3);
        assert_eq!(projected_dataset.rows[0], vec![Value::Int(11)]);
        assert_eq!(projected_dataset.rows[1], vec![Value::Int(22)]);
        assert_eq!(projected_dataset.rows[2], vec![Value::Int(33)]);
    }

    #[test]
    fn test_vertex_projection() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("创建Mock存储失败")));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());

        let expr1 = crate::core::Expression::Variable("id".to_string());
        let expr_meta1 = crate::core::types::expr::ExpressionMeta::new(expr1);
        let expr_id1 = expr_context.register_expression(expr_meta1);
        let ctx_expr1 =
            crate::core::types::ContextualExpression::new(expr_id1, expr_context.clone());

        let expr2 = crate::core::Expression::Variable("name".to_string());
        let expr_meta2 = crate::core::types::expr::ExpressionMeta::new(expr2);
        let expr_id2 = expr_context.register_expression(expr_meta2);
        let ctx_expr2 =
            crate::core::types::ContextualExpression::new(expr_id2, expr_context.clone());

        let columns = vec![
            ProjectionColumn::new("vertex_id".to_string(), ctx_expr1),
            ProjectionColumn::new("name".to_string(), ctx_expr2),
        ];

        let executor = ProjectExecutor::new(1, storage, columns, expr_context);

        // Create test vertices.
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

        let vertices = vec![vertex1, vertex2];

        // Directly call the `project_vertices` method to conduct the test.
        let projected_dataset = executor
            .project_vertices(vertices)
            .expect("Projection should succeed");

        // Verification results
        assert_eq!(projected_dataset.col_names, vec!["vertex_id", "name"]);
        assert_eq!(projected_dataset.rows.len(), 2);
        assert_eq!(
            projected_dataset.rows[0],
            vec![Value::Int(1), Value::String("Alice".to_string())]
        );
        assert_eq!(
            projected_dataset.rows[1],
            vec![Value::Int(2), Value::String("Bob".to_string())]
        );
    }

    #[test]
    fn test_edge_projection() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("创建Mock存储失败")));
        let expr_context = Arc::new(ExpressionAnalysisContext::new());

        let expr1 = crate::core::Expression::Variable("src".to_string());
        let expr_meta1 = crate::core::types::expr::ExpressionMeta::new(expr1);
        let expr_id1 = expr_context.register_expression(expr_meta1);
        let ctx_expr1 =
            crate::core::types::ContextualExpression::new(expr_id1, expr_context.clone());

        let expr2 = crate::core::Expression::Variable("dst".to_string());
        let expr_meta2 = crate::core::types::expr::ExpressionMeta::new(expr2);
        let expr_id2 = expr_context.register_expression(expr_meta2);
        let ctx_expr2 =
            crate::core::types::ContextualExpression::new(expr_id2, expr_context.clone());

        let expr3 = crate::core::Expression::Variable("edge_type".to_string());
        let expr_meta3 = crate::core::types::expr::ExpressionMeta::new(expr3);
        let expr_id3 = expr_context.register_expression(expr_meta3);
        let ctx_expr3 =
            crate::core::types::ContextualExpression::new(expr_id3, expr_context.clone());

        let columns = vec![
            ProjectionColumn::new("src_id".to_string(), ctx_expr1),
            ProjectionColumn::new("dst_id".to_string(), ctx_expr2),
            ProjectionColumn::new("edge_type".to_string(), ctx_expr3),
        ];

        let executor = ProjectExecutor::new(1, storage, columns, expr_context);

        // Create a test edge.
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

        let edges = vec![edge1, edge2];

        // Directly call the `project_edges` method to perform the test.
        let projected_dataset = executor
            .project_edges(edges)
            .expect("Projection should succeed");

        // Verification results
        assert_eq!(
            projected_dataset.col_names,
            vec!["src_id", "dst_id", "edge_type"]
        );
        assert_eq!(projected_dataset.rows.len(), 2);
        assert_eq!(
            projected_dataset.rows[0],
            vec![
                Value::Int(1),
                Value::Int(2),
                Value::String("knows".to_string())
            ]
        );
        assert_eq!(
            projected_dataset.rows[1],
            vec![
                Value::Int(2),
                Value::Int(3),
                Value::String("works_with".to_string())
            ]
        );
    }
}
