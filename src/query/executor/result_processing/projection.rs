//! 列投影执行器
//!
//! ProjectExecutor - 选择和投影输出列

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::Value;
use crate::graph::expression::{EvalContext, Expression, ExpressionEvaluator};
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, InputExecutor};

/// 投影列定义
#[derive(Debug, Clone)]
pub struct ProjectionColumn {
    pub name: String,                   // 输出列名
    pub expression: Expression,         // 投影表达式
}

impl ProjectionColumn {
    pub fn new(name: String, expression: Expression) -> Self {
        Self { name, expression }
    }
}

/// ProjectExecutor - 投影执行器
///
/// 执行列投影操作，支持表达式求值和列重命名
pub struct ProjectExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    columns: Vec<ProjectionColumn>,    // 投影列定义
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> ProjectExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        columns: Vec<ProjectionColumn>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ProjectExecutor".to_string(), storage),
            columns,
            input_executor: None,
        }
    }

    /// 处理单行数据的投影
    fn project_row(
        &self,
        row: &[Value],
        col_names: &[String],
    ) -> Result<Vec<Value>, QueryError> {
        let mut projected_row = Vec::new();
        let evaluator = ExpressionEvaluator;

        // 为当前行创建评估上下文
        let mut context = EvalContext::new();

        // 将当前行的值设置为上下文变量
        for (i, col_name) in col_names.iter().enumerate() {
            if i < row.len() {
                context.set_variable(col_name.clone(), row[i].clone());
            }
        }

        // 对每个投影列进行求值
        for column in &self.columns {
            match evaluator.evaluate(&column.expression, &context) {
                Ok(value) => projected_row.push(value),
                Err(e) => {
                    return Err(QueryError::ExecutionError(format!(
                        "Failed to evaluate projection expression '{}': {}",
                        column.name, e
                    )));
                }
            }
        }

        Ok(projected_row)
    }

    /// 处理数据集投影
    fn project_dataset(
        &self,
        dataset: crate::core::value::DataSet,
    ) -> Result<crate::core::value::DataSet, QueryError> {
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
    ) -> Result<crate::core::value::DataSet, QueryError> {
        let mut result_dataset = crate::core::value::DataSet::new();

        // 设置列名
        result_dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();

        let evaluator = ExpressionEvaluator;

        // 对每个顶点进行投影
        for vertex in vertices {
            let mut context = EvalContext::with_vertex(&vertex);
            
            // 设置顶点ID作为变量
            context.set_variable("id".to_string(), *vertex.vid.clone());

            let mut projected_row = Vec::new();
            for column in &self.columns {
                match evaluator.evaluate(&column.expression, &context) {
                    Ok(value) => projected_row.push(value),
                    Err(e) => {
                        return Err(QueryError::ExecutionError(format!(
                            "Failed to evaluate projection expression '{}': {}",
                            column.name, e
                        )));
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
    ) -> Result<crate::core::value::DataSet, QueryError> {
        let mut result_dataset = crate::core::value::DataSet::new();

        // 设置列名
        result_dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();

        let evaluator = ExpressionEvaluator;

        // 对每个边进行投影
        for edge in edges {
            let mut context = EvalContext::with_edge(&edge);
            
            // 设置边属性作为变量
            context.set_variable("src".to_string(), *edge.src.clone());
            context.set_variable("dst".to_string(), *edge.dst.clone());
            context.set_variable("edge_type".to_string(), Value::String(edge.edge_type.clone()));
            context.set_variable("ranking".to_string(), Value::Int(edge.ranking as i64));

            let mut projected_row = Vec::new();
            for column in &self.columns {
                match evaluator.evaluate(&column.expression, &context) {
                    Ok(value) => projected_row.push(value),
                    Err(e) => {
                        return Err(QueryError::ExecutionError(format!(
                            "Failed to evaluate projection expression '{}': {}",
                            column.name, e
                        )));
                    }
                }
            }
            result_dataset.rows.push(projected_row);
        }

        Ok(result_dataset)
    }
}

impl<S: StorageEngine> InputExecutor<S> for ProjectExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for ProjectExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::DataSet(crate::core::value::DataSet::new())
        };

        // 对结果应用投影
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
                // 对于值列表，我们创建一个简单的数据集
                let mut dataset = crate::core::value::DataSet::new();
                dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();
                
                // 每个值作为一行
                for value in values {
                    dataset.rows.push(vec![value]);
                }
                ExecutionResult::DataSet(dataset)
            }
            ExecutionResult::Paths(paths) => {
                // 对于路径，我们创建一个包含路径信息的数据集
                let mut dataset = crate::core::value::DataSet::new();
                dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();
                
                let evaluator = ExpressionEvaluator;

                for path in paths {
                    let mut context = EvalContext::new();
                    // 设置路径相关信息作为变量
                    context.set_variable("path_length".to_string(), Value::Int(path.len() as i64));
                    context.set_variable("src".to_string(), Value::String(path.src.vid.to_string()));
                    
                    let mut projected_row = Vec::new();
                    for column in &self.columns {
                        match evaluator.evaluate(&column.expression, &context) {
                            Ok(value) => projected_row.push(value),
                            Err(e) => {
                                return Err(QueryError::ExecutionError(format!(
                                    "Failed to evaluate projection expression '{}': {}",
                                    column.name, e
                                )));
                            }
                        }
                    }
                    dataset.rows.push(projected_row);
                }
                ExecutionResult::DataSet(dataset)
            }
            ExecutionResult::Count(count) => {
                // 对于计数，我们创建一个包含计数值的数据集
                let mut dataset = crate::core::value::DataSet::new();
                dataset.col_names = self.columns.iter().map(|c| c.name.clone()).collect();
                dataset.rows.push(vec![Value::Int(count as i64)]);
                ExecutionResult::DataSet(dataset)
            }
            ExecutionResult::Success => ExecutionResult::Success,
        };

        Ok(projected_result)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化投影所需的任何资源
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        // 清理资源
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::{DataSet, Value};
    use crate::graph::expression::Expression;
    use crate::storage::StorageEngine;

    // #[tokio::test]
    // async fn test_simple_projection() {
    //     let storage = Arc::new(Mutex::new(MockStorageEngine::new()));
    //
    //     // 创建简单的投影：选择第一列
    //     let columns = vec![ProjectionColumn::new(
    //         "col1".to_string(),
    //         Expression::Property("col1".to_string()),
    //     )];
    //
    //     let executor = ProjectExecutor::new(1, storage, columns);

    //     // 测试简单投影功能
    //     // 这里需要模拟输入数据
    // }

    // #[tokio::test]
    // async fn test_expression_projection() {
    //     let storage = Arc::new(Mutex::new(MockStorageEngine::new()));
    //
    //     // 创建表达式投影：计算两列之和
    //     let columns = vec![ProjectionColumn::new(
    //         "sum".to_string(),
    //         Expression::BinaryOp(
    //             Box::new(Expression::Property("col1".to_string())),
    //             crate::graph::expression::binary::BinaryOperator::Add,
    //             Box::new(Expression::Property("col2".to_string())),
    //         ),
    //     )];
    //
    //     let executor = ProjectExecutor::new(1, storage, columns);

    //     // 测试表达式投影功能
    //     // 这里需要模拟输入数据
    // }
}