//! 集合操作执行器集成测试
//!
//! 测试集合操作执行器的实际功能

#[cfg(test)]
mod tests {
    use graphdb::core::{Value, DataSet};
    use graphdb::query::executor::{ExecutionResult, ExecutionContext};
    use graphdb::query::executor::data_processing::set_operations::{
        UnionExecutor, UnionAllExecutor, 
        IntersectExecutor, MinusExecutor
    };
    use graphdb::storage::StorageEngine;
    use graphdb::storage::native_storage::NativeStorage;
    use graphdb::query::executor::Executor;
    use std::sync::{Arc, Mutex};

    /// 创建测试用的执行器上下文
    fn create_test_context_with_data(
        left_var: &str,
        left_data: DataSet,
        right_var: &str,
        right_data: DataSet,
    ) -> ExecutionContext {
        let mut context = ExecutionContext::new();
        
        // 将DataSet转换为Values并设置到上下文中
        let left_values: Vec<Value> = left_data.rows.into_iter()
            .flat_map(|row| row.into_iter())
            .collect();
        let right_values: Vec<Value> = right_data.rows.into_iter()
            .flat_map(|row| row.into_iter())
            .collect();
            
        context.set_result(left_var.to_string(), ExecutionResult::Values(left_values));
        context.set_result(right_var.to_string(), ExecutionResult::Values(right_values));
        
        context
    }

    #[tokio::test]
    async fn test_union_integration() {
        let storage = Arc::new(Mutex::new(NativeStorage::new("test_db_union").unwrap()));
        
        // 创建测试数据
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::String("Bob".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(2), Value::String("Bob".to_string())], // 重复行
                vec![Value::Int(3), Value::String("Charlie".to_string())],
            ],
        };

        let context = create_test_context_with_data(
            "left_input", left_dataset.clone(),
            "right_input", right_dataset.clone()
        );

        // 创建Union执行器
        let mut executor = UnionExecutor::new(
            1,
            storage,
            "left_input".to_string(),
            "right_input".to_string(),
        );
        
        // 设置上下文
        executor.set_executor.base_mut().context = context;

        // 执行UNION操作
        let result = executor.execute().await;
        
        assert!(result.is_ok());
        if let Ok(ExecutionResult::Values(values)) = result {
            // UNION应该去重，所以期望3个唯一的值：1, Alice, 2, Bob, 3, Charlie
            // 3行 × 2列 = 6个值
            assert_eq!(values.len(), 6);
        }
    }

    #[tokio::test]
    async fn test_union_all_integration() {
        let storage = Arc::new(Mutex::new(NativeStorage::new("test_db_union_all").unwrap()));
        
        // 创建测试数据
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::String("Bob".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(2), Value::String("Bob".to_string())], // 重复行
                vec![Value::Int(3), Value::String("Charlie".to_string())],
            ],
        };

        let context = create_test_context_with_data(
            "left_input", left_dataset.clone(),
            "right_input", right_dataset.clone()
        );

        // 创建UnionAll执行器
        let mut executor = UnionAllExecutor::new(
            2,
            storage,
            "left_input".to_string(),
            "right_input".to_string(),
        );
        
        // 设置上下文
        executor.set_executor.base_mut().context = context;

        // 执行UNION ALL操作
        let result = executor.execute().await;
        
        assert!(result.is_ok());
        if let Ok(ExecutionResult::Values(values)) = result {
            // UNION ALL应该保留重复行，所以期望4个值：1, Alice, 2, Bob, 2, Bob, 3, Charlie
            // 4行 × 2列 = 8个值
            assert_eq!(values.len(), 8);
        }
    }

    #[tokio::test]
    async fn test_intersect_integration() {
        let storage = Arc::new(Mutex::new(NativeStorage::new("test_db_intersect").unwrap()));
        
        // 创建测试数据
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::String("Bob".to_string())],
                vec![Value::Int(3), Value::String("Charlie".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(2), Value::String("Bob".to_string())], // 共同行
                vec![Value::Int(3), Value::String("Charlie".to_string())], // 共同行
                vec![Value::Int(4), Value::String("David".to_string())],
            ],
        };

        let context = create_test_context_with_data(
            "left_input", left_dataset.clone(),
            "right_input", right_dataset.clone()
        );

        // 创建Intersect执行器
        let mut executor = IntersectExecutor::new(
            3,
            storage,
            "left_input".to_string(),
            "right_input".to_string(),
        );
        
        // 设置上下文
        executor.set_executor.base_mut().context = context;

        // 执行INTERSECT操作
        let result = executor.execute().await;
        
        assert!(result.is_ok());
        if let Ok(ExecutionResult::Values(values)) = result {
            // INTERSECT应该只返回共同的行：Bob和Charlie
            // 2行 × 2列 = 4个值
            assert_eq!(values.len(), 4);
        }
    }

    #[tokio::test]
    async fn test_minus_integration() {
        let storage = Arc::new(Mutex::new(NativeStorage::new("test_db_minus").unwrap()));
        
        // 创建测试数据
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(1), Value::String("Alice".to_string())],
                vec![Value::Int(2), Value::String("Bob".to_string())],
                vec![Value::Int(3), Value::String("Charlie".to_string())],
                vec![Value::Int(4), Value::String("David".to_string())],
            ],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![Value::Int(2), Value::String("Bob".to_string())], // 要排除的行
                vec![Value::Int(4), Value::String("David".to_string())], // 要排除的行
                vec![Value::Int(5), Value::String("Eve".to_string())], // 左数据集中不存在的行
            ],
        };

        let context = create_test_context_with_data(
            "left_input", left_dataset.clone(),
            "right_input", right_dataset.clone()
        );

        // 创建Minus执行器
        let mut executor = MinusExecutor::new(
            4,
            storage,
            "left_input".to_string(),
            "right_input".to_string(),
        );
        
        // 设置上下文
        executor.set_executor.base_mut().context = context;

        // 执行MINUS操作
        let result = executor.execute().await;
        
        assert!(result.is_ok());
        if let Ok(ExecutionResult::Values(values)) = result {
            // MINUS应该只包含Alice和Charlie（Bob和David被排除）
            // 2行 × 2列 = 4个值
            assert_eq!(values.len(), 4);
        }
    }

    #[tokio::test]
    async fn test_column_mismatch_error() {
        let storage = Arc::new(Mutex::new(NativeStorage::new("test_db_mismatch").unwrap()));
        
        // 创建列名不匹配的测试数据
        let left_dataset = DataSet {
            col_names: vec!["id".to_string(), "name".to_string()],
            rows: vec![vec![Value::Int(1), Value::String("Alice".to_string())]],
        };

        let right_dataset = DataSet {
            col_names: vec!["id".to_string(), "title".to_string()], // 不同的列名
            rows: vec![vec![Value::Int(1), Value::String("Ms".to_string())]],
        };

        let context = create_test_context_with_data(
            "left_mismatch", left_dataset.clone(),
            "right_mismatch", right_dataset.clone()
        );

        // 创建Union执行器
        let mut executor = UnionExecutor::new(
            5,
            storage,
            "left_mismatch".to_string(),
            "right_mismatch".to_string(),
        );
        
        // 设置上下文
        executor.set_executor.base_mut().context = context;

        // 执行应该失败
        let result = executor.execute().await;
        assert!(result.is_err());
    }
}