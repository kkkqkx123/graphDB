//! 服务统计信息测试
//!
//! 测试范围：
//! - GetStats 接口
//! - 统计信息准确性

use inversearch_service::service::InversearchService;
use inversearch_service::proto::inversearch_service_server::InversearchService as InversearchServiceTrait;
use inversearch_service::proto::*;
use tonic::Request;

/// 测试获取空索引统计
#[tokio::test]
async fn test_stats_empty_index() {
    let service = InversearchService::new().await;

    let stats_request = Request::new(GetStatsRequest {});
    let response = InversearchServiceTrait::get_stats(&service, stats_request).await.unwrap();
    let result = response.into_inner();

    // GetStatsResponse 没有 success 字段，直接检查 document_count
    assert_eq!(result.document_count, 0, "空索引文档数应该为 0");
}

/// 测试添加文档后统计更新
#[tokio::test]
async fn test_stats_after_add() {
    let service = InversearchService::new().await;

    // 添加 5 个文档
    for i in 1..=5 {
        let request = Request::new(AddDocumentRequest {
            id: i,
            content: format!("Document {}", i),
            metadata: Default::default(),
        });
        InversearchServiceTrait::add_document(&service, request).await.unwrap();
    }

    // 获取统计
    let stats_request = Request::new(GetStatsRequest {});
    let response = InversearchServiceTrait::get_stats(&service, stats_request).await.unwrap();
    let result = response.into_inner();

    assert_eq!(result.document_count, 5, "文档数应该为 5");
}

/// 测试删除文档后统计更新
#[tokio::test]
async fn test_stats_after_remove() {
    let service = InversearchService::new().await;

    // 添加 5 个文档
    for i in 1..=5 {
        let request = Request::new(AddDocumentRequest {
            id: i,
            content: format!("Document {}", i),
            metadata: Default::default(),
        });
        InversearchServiceTrait::add_document(&service, request).await.unwrap();
    }

    // 删除 2 个文档
    for i in 1..=2 {
        let request = Request::new(RemoveDocumentRequest { id: i });
        InversearchServiceTrait::remove_document(&service, request).await.unwrap();
    }

    // 获取统计
    let stats_request = Request::new(GetStatsRequest {});
    let response = InversearchServiceTrait::get_stats(&service, stats_request).await.unwrap();
    let result = response.into_inner();

    assert_eq!(result.document_count, 3, "文档数应该为 3");
}

/// 测试清空索引后统计
#[tokio::test]
async fn test_stats_after_clear() {
    let service = InversearchService::new().await;

    // 添加一些文档
    for i in 1..=10 {
        let request = Request::new(AddDocumentRequest {
            id: i,
            content: format!("Document {}", i),
            metadata: Default::default(),
        });
        InversearchServiceTrait::add_document(&service, request).await.unwrap();
    }

    // 清空索引
    let clear_request = Request::new(ClearIndexRequest {});
    InversearchServiceTrait::clear_index(&service, clear_request).await.unwrap();

    // 获取统计
    let stats_request = Request::new(GetStatsRequest {});
    let response = InversearchServiceTrait::get_stats(&service, stats_request).await.unwrap();
    let result = response.into_inner();

    assert_eq!(result.document_count, 0, "清空后文档数应该为 0");
}
