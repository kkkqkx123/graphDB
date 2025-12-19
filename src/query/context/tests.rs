//! Context模块集成测试
//!
//! 测试新的context模块结构和功能

use crate::query::context::{
    QueryContext, ExecutionContext, ExpressionContext, AstContext,
    QueryStatistics, ExecutionState, VariableInfo, VariableType, ColumnDefinition,
};
use crate::query::context::managers::r#impl::{
    MockSchemaManager, MockIndexManager, MockMetaClient, MockStorageClient,
};
use std::sync::Arc;

/// 创建测试用的QueryContext
fn create_test_query_context() -> Arc<QueryContext> {
    let schema_manager = Arc::new(MockSchemaManager::new());
    let index_manager = Arc::new(MockIndexManager::new());
    let meta_client = Arc::new(MockMetaClient::new());
    let storage_client = Arc::new(MockStorageClient::new());

    let mut ctx = QueryContext::new(
        "test_session".to_string(),
        "test_user".to_string(),
        schema_manager,
        index_manager,
        meta_client,
        storage_client,
    );

    // 设置一些测试数据
    ctx.set_variable("test_var".to_string(), crate::core::Value::Int(42));
    ctx.set_parameter("test_param".to_string(), crate::core::Value::String("test".to_string()));

    Arc::new(ctx)
}

#[test]
fn test_context_integration() {
    // 创建QueryContext
    let query_ctx = create_test_query_context();
    
    // 创建ExecutionContext
    let exec_ctx = ExecutionContext::new(query_ctx.clone());
    
    // 创建ExpressionContext
    let expr_ctx = ExpressionContext::new(&query_ctx);
    
    // 创建AstContext
    let ast_ctx = AstContext::new(
        "SELECT".to_string(),
        "SELECT * FROM test".to_string(),
    );

    // 验证基本功能
    assert_eq!(query_ctx.session_id, "test_session");
    assert_eq!(query_ctx.user_id, "test_user");
    assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Initialized);
    assert_eq!(ast_ctx.query_type, "SELECT");
    assert_eq!(ast_ctx.query_text(), "SELECT * FROM test");
}

#[test]
fn test_variable_resolution_chain() {
    let query_ctx = create_test_query_context();
    let exec_ctx = ExecutionContext::new(query_ctx.clone());
    let mut expr_ctx = ExpressionContext::new(&query_ctx).with_execution_context(&exec_ctx);

    // 设置局部变量
    expr_ctx.set_local_variable("local_var".to_string(), crate::core::Value::String("local".to_string()));

    // 测试变量解析顺序
    assert_eq!(
        expr_ctx.get_variable("local_var"),
        Some(&crate::core::Value::String("local".to_string()))
    );
    assert_eq!(
        expr_ctx.get_variable("test_var"),
        Some(&crate::core::Value::Int(42))
    );
    assert_eq!(
        expr_ctx.get_variable("test_param"),
        Some(&crate::core::Value::String("test".to_string()))
    );
}

#[test]
fn test_execution_lifecycle() {
    let query_ctx = create_test_query_context();
    let exec_ctx = ExecutionContext::new(query_ctx);

    // 测试执行生命周期
    assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Initialized);
    
    exec_ctx.start();
    assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Running);
    assert!(exec_ctx.is_running());
    
    exec_ctx.pause();
    assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Paused);
    
    exec_ctx.resume();
    assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Running);
    
    exec_ctx.complete();
    assert_eq!(exec_ctx.get_execution_state(), ExecutionState::Completed);
    assert!(exec_ctx.is_completed());
    
    exec_ctx.set_error("测试错误".to_string());
    assert!(exec_ctx.has_error());
    assert_eq!(exec_ctx.get_error(), Some("测试错误".to_string()));
}

#[test]
fn test_ast_context_management() {
    let mut ast_ctx = AstContext::new(
        "MATCH".to_string(),
        "MATCH (n:Person) RETURN n".to_string(),
    );

    // 添加变量
    let mut var_info = VariableInfo::new(VariableType::Vertex);
    var_info.add_label("Person".to_string());
    var_info.add_property("name".to_string(), "string".to_string());
    ast_ctx.add_variable("n".to_string(), var_info);

    // 添加输出列
    let col_def = ColumnDefinition::new("n".to_string(), "vertex".to_string());
    ast_ctx.add_output_column(col_def);

    // 验证
    assert!(ast_ctx.has_variable("n"));
    assert_eq!(ast_ctx.output_column_count(), 1);
    assert_eq!(ast_ctx.get_output_column(0).unwrap().name, "n");
    
    let vertex_vars = ast_ctx.get_variables_by_type(&VariableType::Vertex);
    assert_eq!(vertex_vars.len(), 1);
    assert!(vertex_vars.contains(&"n"));
}

#[test]
fn test_statistics_tracking() {
    let query_ctx = create_test_query_context();
    let exec_ctx = ExecutionContext::new(query_ctx);

    // 测试统计信息
    exec_ctx.metrics.start_timing();
    std::thread::sleep(std::time::Duration::from_millis(10));
    exec_ctx.metrics.end_timing();
    
    exec_ctx.metrics.add_step();
    exec_ctx.metrics.add_step();
    exec_ctx.metrics.add_cache_hit();
    exec_ctx.metrics.add_cache_miss();
    
    assert!(exec_ctx.metrics.duration_ms().unwrap() >= 10);
    assert_eq!(exec_ctx.metrics.steps_executed, 2);
    assert_eq!(exec_ctx.metrics.cache_hits, 1);
    assert_eq!(exec_ctx.metrics.cache_misses, 1);
    assert_eq!(exec_ctx.metrics.cache_hit_rate(), 0.5);
}

#[test]
fn test_resource_management() {
    let query_ctx = create_test_query_context();
    let exec_ctx = ExecutionContext::new(query_ctx);

    // 测试资源管理
    assert_eq!(exec_ctx.resource_manager.memory_usage(), 0);
    exec_ctx.resource_manager.add_memory_usage(1024);
    assert_eq!(exec_ctx.resource_manager.memory_usage(), 1024);
    
    assert_eq!(exec_ctx.resource_manager.open_files(), 0);
    exec_ctx.resource_manager.add_open_file();
    assert_eq!(exec_ctx.resource_manager.open_files(), 1);
    
    assert_eq!(exec_ctx.resource_manager.network_connections(), 0);
    exec_ctx.resource_manager.add_network_connection();
    assert_eq!(exec_ctx.resource_manager.network_connections(), 1);
}