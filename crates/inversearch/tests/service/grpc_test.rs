//! gRPC 服务接口测试
//!
//! 测试范围：
//! - AddDocument
//! - UpdateDocument
//! - RemoveDocument
//! - Search
//! - ClearIndex

use inversearch_service::service::InversearchService;
use inversearch_service::proto::inversearch_service_server::InversearchService as InversearchServiceTrait;
use inversearch_service::proto::*;
use tonic::Request;

/// 测试添加文档接口
#[tokio::test]
async fn test_grpc_add_document() {
    let service = InversearchService::new().await;

    let request = Request::new(AddDocumentRequest {
        id: 1,
        content: "Test document content".to_string(),
        metadata: Default::default(),
    });

    let response = InversearchServiceTrait::add_document(&service, request).await.unwrap();
    let result = response.into_inner();

    assert!(result.success, "添加文档应该成功");
    assert!(result.error.is_empty(), "不应该有错误信息");
}

/// 测试添加多个文档
#[tokio::test]
async fn test_grpc_add_multiple_documents() {
    let service = InversearchService::new().await;

    for i in 1..=5 {
        let request = Request::new(AddDocumentRequest {
            id: i,
            content: format!("Document {} content", i),
            metadata: Default::default(),
        });

        let response = InversearchServiceTrait::add_document(&service, request).await.unwrap();
        let result = response.into_inner();
        assert!(result.success, "文档 {} 添加应该成功", i);
    }
}

/// 测试更新文档接口
#[tokio::test]
async fn test_grpc_update_document() {
    let service = InversearchService::new().await;

    // 先添加文档
    let add_request = Request::new(AddDocumentRequest {
        id: 1,
        content: "Original content".to_string(),
        metadata: Default::default(),
    });
    InversearchServiceTrait::add_document(&service, add_request).await.unwrap();

    // 更新文档
    let update_request = Request::new(UpdateDocumentRequest {
        id: 1,
        content: "Updated content".to_string(),
        metadata: Default::default(),
    });

    let response = InversearchServiceTrait::update_document(&service, update_request).await.unwrap();
    let result = response.into_inner();

    assert!(result.success, "更新文档应该成功");
}

/// 测试删除文档接口
#[tokio::test]
async fn test_grpc_remove_document() {
    let service = InversearchService::new().await;

    // 先添加文档
    let add_request = Request::new(AddDocumentRequest {
        id: 1,
        content: "Content to remove".to_string(),
        metadata: Default::default(),
    });
    InversearchServiceTrait::add_document(&service, add_request).await.unwrap();

    // 删除文档
    let remove_request = Request::new(RemoveDocumentRequest {
        id: 1,
    });

    let response = InversearchServiceTrait::remove_document(&service, remove_request).await.unwrap();
    let result = response.into_inner();

    assert!(result.success, "删除文档应该成功");
}

/// 测试搜索接口
#[tokio::test]
async fn test_grpc_search() {
    let service = InversearchService::new().await;

    // 添加测试文档
    for i in 1..=3 {
        let request = Request::new(AddDocumentRequest {
            id: i,
            content: format!("Searchable document {}", i),
            metadata: Default::default(),
        });
        InversearchServiceTrait::add_document(&service, request).await.unwrap();
    }

    // 搜索
    let search_request = Request::new(SearchRequest {
        query: "Searchable".to_string(),
        limit: 10,
        offset: 0,
        context: false,
        suggest: false,
        resolve: true,
        enrich: false,
        cache: false,
        highlight: false,
        highlight_options: None,
    });

    let response = InversearchServiceTrait::search(&service, search_request).await.unwrap();
    let result = response.into_inner();

    // SearchResponse 没有 success 字段，直接检查结果
    assert!(!result.results.is_empty(), "应该返回搜索结果");
}

/// 测试清空索引接口
#[tokio::test]
async fn test_grpc_clear_index() {
    let service = InversearchService::new().await;

    // 添加一些文档
    for i in 1..=5 {
        let request = Request::new(AddDocumentRequest {
            id: i,
            content: format!("Document {}", i),
            metadata: Default::default(),
        });
        InversearchServiceTrait::add_document(&service, request).await.unwrap();
    }

    // 清空索引
    let clear_request = Request::new(ClearIndexRequest {});
    let response = InversearchServiceTrait::clear_index(&service, clear_request).await.unwrap();
    let result = response.into_inner();

    assert!(result.success, "清空索引应该成功");
}

/// 测试空查询搜索
#[tokio::test]
async fn test_grpc_search_empty_query() {
    let service = InversearchService::new().await;

    let search_request = Request::new(SearchRequest {
        query: "".to_string(),
        limit: 10,
        offset: 0,
        context: false,
        suggest: false,
        resolve: true,
        enrich: false,
        cache: false,
        highlight: false,
        highlight_options: None,
    });

    let response = InversearchServiceTrait::search(&service, search_request).await;
    
    // 空查询可能返回错误或空结果，取决于实现
    let _ = response;
}

/// 测试搜索不存在的内容
#[tokio::test]
async fn test_grpc_search_nonexistent() {
    let service = InversearchService::new().await;

    let search_request = Request::new(SearchRequest {
        query: "xyznonexistent".to_string(),
        limit: 10,
        offset: 0,
        context: false,
        suggest: false,
        resolve: true,
        enrich: false,
        cache: false,
        highlight: false,
        highlight_options: None,
    });

    let response = InversearchServiceTrait::search(&service, search_request).await.unwrap();
    let result = response.into_inner();

    // SearchResponse 没有 success 字段，直接检查结果
    assert!(result.results.is_empty(), "应该返回空结果");
}
