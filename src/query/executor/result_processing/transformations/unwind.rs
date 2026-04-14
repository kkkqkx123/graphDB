//! Implementation of UnwindExecutor
//!
//! Responsible for handling the list expansion process, expanding each element in the list into a separate row.

use parking_lot::Mutex;
use std::sync::Arc;

use crate::core::error::{DBError, DBResult};
use crate::query::DataSet;
use crate::core::{Expression, Value};
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::base::{ExecutionResult, Executor};
use crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::expression::{
    DefaultExpressionContext, ExpressionContext as EvalContext,
};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;

/// Unwind Actuator
/// Used to expand each element in the list into a separate row.
pub struct UnwindExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    /// Input variable name
    input_var: String,
    /// The expression to be expanded
    unwind_expression: Expression,
    /// Column names
    col_names: Vec<String>,
    /// Does it come from a pipeline?
    from_pipe: bool,
}

impl<S: StorageClient + Send + 'static> UnwindExecutor<S> {
    /// Create a new UnwindExecutor
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        input_var: String,
        unwind_expression: Expression,
        col_names: Vec<String>,
        from_pipe: bool,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "UnwindExecutor".to_string(), storage, expr_context),
            input_var,
            unwind_expression,
            col_names,
            from_pipe,
        }
    }

    /// Create an UnwindExecutor with context information
    pub fn with_context(
        id: i64,
        storage: Arc<Mutex<S>>,
        input_var: String,
        unwind_expression: Expression,
        col_names: Vec<String>,
        from_pipe: bool,
        context: crate::query::executor::base::ExecutionContext,
    ) -> Self {
        Self {
            base: BaseExecutor::with_context(id, "UnwindExecutor".to_string(), storage, context),
            input_var,
            unwind_expression,
            col_names,
            from_pipe,
        }
    }

    /// Extract a list from a value.
    fn extract_list(&self, val: &Value) -> Vec<Value> {
        match val {
            Value::List(list) => list.clone().into_vec(),
            Value::Null(_) | Value::Empty => vec![],
            _ => vec![val.clone()],
        }
    }

    /// Please provide the text you would like to have translated. I will then perform the translation and provide the translated version.
    fn execute_unwind(&mut self) -> DBResult<DataSet> {
        // Obtain the input result.
        let input_result = self
            .base
            .context
            .get_result(&self.input_var)
            .ok_or_else(|| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "Input variable '{}' not found",
                    self.input_var
                )))
            })?;

        // Create the context for the expression.
        let mut expr_context = DefaultExpressionContext::new();

        // Create an output dataset
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        // Process the input result based on its type
        match input_result {
            ExecutionResult::DataSet(input_data) => {
                for row in input_data.rows {
                    for value in row {
                        expr_context.set_variable("_".to_string(), value.clone());

                        let unwind_value =
                            ExpressionEvaluator::evaluate(&self.unwind_expression, &mut expr_context)
                                .map_err(|e| {
                                DBError::Query(crate::core::error::QueryError::ExecutionError(
                                    e.to_string(),
                                ))
                            })?;

                        let list_values = self.extract_list(&unwind_value);

                        for list_item in list_values {
                            let mut row = Vec::new();

                            if !self.from_pipe {
                                row.push(value.clone());
                            }

                            row.push(list_item);

                            dataset.rows.push(row);
                        }
                    }
                }
            }
            ExecutionResult::Success => {
                let empty_value = Value::Empty;
                expr_context.set_variable("_".to_string(), empty_value.clone());

                let unwind_value =
                    ExpressionEvaluator::evaluate(&self.unwind_expression, &mut expr_context)
                        .map_err(|e| {
                            DBError::Query(crate::core::error::QueryError::ExecutionError(
                                e.to_string(),
                            ))
                        })?;

                let list_values = self.extract_list(&unwind_value);

                for list_item in list_values {
                    dataset.rows.push(vec![list_item]);
                }
            }
            ExecutionResult::Empty => {}
            ExecutionResult::Error(e) => {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(format!(
                        "Error in input result: {}",
                        e
                    )),
                ));
            }
        }

        Ok(dataset)
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for UnwindExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let dataset = self.execute_unwind()?;
        Ok(ExecutionResult::DataSet(dataset))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
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

impl<S: StorageClient + Send + 'static> crate::query::executor::base::HasStorage<S>
    for UnwindExecutor<S>
{
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("UnwindExecutor storage should be set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Expression, List, Value};
    use crate::storage::MockStorage;
    use parking_lot::Mutex;
    use std::sync::Arc;

    #[test]
    fn test_unwind_executor() {
        let storage = Arc::new(Mutex::new(
            MockStorage::new().expect("Failed to create MockStorage"),
        ));

        let list_value = Value::List(List::from(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ]));

        let input_dataset = DataSet::from_rows(
            vec![vec![list_value]],
            vec!["value".to_string()],
        );
        let input_result = ExecutionResult::DataSet(input_dataset);

        let expr_context = Arc::new(ExpressionAnalysisContext::new());
        let context = crate::query::executor::base::ExecutionContext::new(expr_context);
        context.set_result("input".to_string(), input_result);

        let unwind_expression = Expression::Variable("_".to_string());
        let mut executor = UnwindExecutor::with_context(
            1,
            storage,
            "input".to_string(),
            unwind_expression,
            vec!["unwound".to_string()],
            false,
            context,
        );

        let result = executor
            .execute()
            .expect("Executor should execute successfully");

        match result {
            ExecutionResult::DataSet(dataset) => {
                assert_eq!(dataset.rows.len(), 3);
                assert_eq!(dataset.rows[0].len(), 2);
                assert_eq!(dataset.rows[0][0], Value::List(List::from(vec![
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                ])));
                assert_eq!(dataset.rows[0][1], Value::Int(1));
                assert_eq!(dataset.rows[1][1], Value::Int(2));
                assert_eq!(dataset.rows[2][1], Value::Int(3));
            }
            _ => panic!("Expected DataSet result"),
        }
    }
}
