# gRPC 端口实现方案

## 概述

本文档描述如何为 GraphDB 添加 gRPC 端口支持，参考 `crates/inversearch/src/api/server/grpc.rs` 的架构实现，保持版本一致性。

## 架构设计

### 1. 模块结构

```
src/api/server/
├── grpc/
│   ├── mod.rs           # 模块导出
│   ├── server.rs        # gRPC 服务实现
│   └── proto/           # 生成的 proto 代码（编译时生成）
├── http/
│   └── ...              # 现有 HTTP 服务
└── mod.rs               # 导出 gRPC 模块
```

### 2. 依赖管理

#### Cargo.toml 添加

```toml
[dependencies]
# gRPC 相关依赖（可选）
tonic = { version = "0.12", optional = true }
prost = { version = "0.13", optional = true }
tracing = "0.1"  # 如果尚未添加

[build-dependencies]
tonic-build = { version = "0.12", optional = true }
prost-build = { version = "0.13", optional = true }

[features]
# 添加 gRPC 功能
grpc = ["tonic", "prost", "tonic-build", "prost-build"]
```

### 3. Proto 文件定义

创建 `proto/graphdb.proto`：

```protobuf
syntax = "proto3";

package graphdb;

// GraphDB 服务定义
service GraphDBService {
  // 健康检查
  rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);

  // 认证相关
  rpc Login(LoginRequest) returns (LoginResponse);
  rpc Logout(LogoutRequest) returns (LogoutResponse);

  // 会话管理
  rpc CreateSession(CreateSessionRequest) returns (CreateSessionResponse);
  rpc GetSession(GetSessionRequest) returns (GetSessionResponse);
  rpc CloseSession(CloseSessionRequest) returns (CloseSessionResponse);

  // 查询执行
  rpc ExecuteQuery(ExecuteQueryRequest) returns (ExecuteQueryResponse);
  rpc ValidateQuery(ValidateQueryRequest) returns (ValidateQueryResponse);
  rpc ExecuteQueryStream(ExecuteQueryRequest) returns (stream QueryResultChunk);

  // 事务管理
  rpc BeginTransaction(BeginTransactionRequest) returns (BeginTransactionResponse);
  rpc CommitTransaction(CommitTransactionRequest) returns (CommitTransactionResponse);
  rpc RollbackTransaction(RollbackTransactionRequest) returns (RollbackTransactionResponse);

  // Schema 管理
  rpc CreateSpace(CreateSpaceRequest) returns (CreateSpaceResponse);
  rpc GetSpace(GetSpaceRequest) returns (GetSpaceResponse);
  rpc DropSpace(DropSpaceRequest) returns (DropSpaceResponse);
  rpc ListSpaces(ListSpacesRequest) returns (ListSpacesResponse);

  rpc CreateTag(CreateTagRequest) returns (CreateTagResponse);
  rpc GetTag(GetTagRequest) returns (GetTagResponse);
  rpc ListTags(ListTagsRequest) returns (ListTagsResponse);
  rpc DropTag(DropTagRequest) returns (DropTagResponse);

  rpc CreateEdgeType(CreateEdgeTypeRequest) returns (CreateEdgeTypeResponse);
  rpc GetEdgeType(GetEdgeTypeRequest) returns (GetEdgeTypeResponse);
  rpc ListEdgeTypes(ListEdgeTypesRequest) returns (ListEdgeTypesResponse);
  rpc DropEdgeType(DropEdgeTypeRequest) returns (DropEdgeTypeResponse);

  // 批量操作
  rpc CreateBatch(CreateBatchRequest) returns (CreateBatchResponse);
  rpc AddBatchItems(AddBatchItemsRequest) returns (AddBatchItemsResponse);
  rpc ExecuteBatch(ExecuteBatchRequest) returns (ExecuteBatchResponse);
  rpc GetBatchStatus(GetBatchStatusRequest) returns (GetBatchStatusResponse);
  rpc CancelBatch(CancelBatchRequest) returns (CancelBatchResponse);

  // 统计信息
  rpc GetSessionStatistics(GetSessionStatisticsRequest) returns (GetSessionStatisticsResponse);
  rpc GetQueryStatistics(GetQueryStatisticsRequest) returns (GetQueryStatisticsResponse);
  rpc GetDatabaseStatistics(GetDatabaseStatisticsRequest) returns (GetDatabaseStatisticsResponse);
  rpc GetSystemStatistics(GetSystemStatisticsRequest) returns (GetSystemStatisticsResponse);

  // 配置管理
  rpc GetConfig(GetConfigRequest) returns (GetConfigResponse);
  rpc UpdateConfig(UpdateConfigRequest) returns (UpdateConfigResponse);
  rpc ResetConfig(ResetConfigRequest) returns (ResetConfigResponse);

  // 自定义函数
  rpc RegisterFunction(RegisterFunctionRequest) returns (RegisterFunctionResponse);
  rpc UnregisterFunction(UnregisterFunctionRequest) returns (UnregisterFunctionResponse);
  rpc ListFunctions(ListFunctionsRequest) returns (ListFunctionsResponse);
  rpc GetFunctionInfo(GetFunctionInfoRequest) returns (GetFunctionInfoResponse);

  // 向量索引
  rpc CreateVectorIndex(CreateVectorIndexRequest) returns (CreateVectorIndexResponse);
  rpc GetVectorIndex(GetVectorIndexRequest) returns (GetVectorIndexResponse);
  rpc ListVectorIndexes(ListVectorIndexesRequest) returns (ListVectorIndexesResponse);
  rpc DropVectorIndex(DropVectorIndexRequest) returns (DropVectorIndexResponse);
  rpc SearchVector(SearchVectorRequest) returns (SearchVectorResponse);
}

// ============================================================
// 通用消息类型
// ============================================================

message HealthCheckRequest {}

message HealthCheckResponse {
  bool healthy = 1;
  string version = 2;
  int64 uptime_seconds = 3;
}

// ============================================================
// 认证相关
// ============================================================

message LoginRequest {
  string username = 1;
  string password = 2;
  optional string space = 3;
}

message LoginResponse {
  bool success = 1;
  string session_id = 2;
  string error = 3;
}

message LogoutRequest {
  string session_id = 1;
}

message LogoutResponse {
  bool success = 1;
  string error = 2;
}

// ============================================================
// 会话管理
// ============================================================

message CreateSessionRequest {
  string username = 1;
  string password = 2;
  optional string space = 3;
}

message CreateSessionResponse {
  bool success = 1;
  string session_id = 2;
  int32 space_id = 3;
  string error = 4;
}

message GetSessionRequest {
  string session_id = 1;
}

message GetSessionResponse {
  bool exists = 1;
  string session_id = 2;
  string username = 3;
  int32 space_id = 4;
  int64 created_at = 5;
  int64 last_accessed = 6;
}

message CloseSessionRequest {
  string session_id = 1;
}

message CloseSessionResponse {
  bool success = 1;
  string error = 2;
}

// ============================================================
// 查询执行
// ============================================================

message ExecuteQueryRequest {
  string query = 1;
  optional string session_id = 2;
  optional string transaction_id = 3;
  optional QueryParameters parameters = 4;
}

message QueryParameters {
  map<string, Value> params = 1;
}

message ExecuteQueryResponse {
  bool success = 1;
  QueryResult result = 2;
  string error = 3;
  ExecutionMetadata metadata = 4;
}

message ValidateQueryRequest {
  string query = 1;
}

message ValidateQueryResponse {
  bool valid = 1;
  string error = 2;
  repeated string parameter_names = 3;
}

message QueryResult {
  repeated string column_names = 1;
  repeated Row rows = 2;
  map<string, string> plan_descriptions = 3;
}

message Row {
  repeated Value values = 1;
}

message Value {
  oneof value {
    string string_value = 1;
    int64 int_value = 2;
    double double_value = 3;
    bool bool_value = 4;
    bytes bytes_value = 5;
    int64 timestamp_value = 6;
    double float_value = 7;
  }
}

message QueryResultChunk {
  repeated Row rows = 1;
  bool is_last = 2;
}

message ExecutionMetadata {
  uint64 rows_returned = 1;
  uint64 execution_time_ms = 2;
  uint64 rows_scanned = 3;
  map<string, uint64> custom_stats = 4;
}

// ============================================================
// 事务管理
// ============================================================

message BeginTransactionRequest {
  optional string session_id = 1;
  TransactionOptions options = 2;
}

message TransactionOptions {
  bool read_only = 1;
  bool autocommit = 2;
  int32 isolation_level = 3;
  int64 timeout_ms = 4;
}

message BeginTransactionResponse {
  bool success = 1;
  string transaction_id = 2;
  string error = 3;
}

message CommitTransactionRequest {
  string transaction_id = 1;
}

message CommitTransactionResponse {
  bool success = 1;
  string error = 2;
}

message RollbackTransactionRequest {
  string transaction_id = 1;
}

message RollbackTransactionResponse {
  bool success = 1;
  string error = 2;
}

// ============================================================
// Schema 管理 - Space
// ============================================================

message CreateSpaceRequest {
  string name = 1;
  SpaceOptions options = 2;
}

message SpaceOptions {
  int32 partition_num = 1;
  int32 replica_num = 2;
  string charset = 3;
  string collate = 4;
  bool vid_fixed_length = 5;
  int32 vid_length = 6;
}

message CreateSpaceResponse {
  bool success = 1;
  int32 space_id = 2;
  string error = 3;
}

message GetSpaceRequest {
  string name = 1;
}

message GetSpaceResponse {
  bool exists = 1;
  SpaceInfo space = 2;
  string error = 3;
}

message SpaceInfo {
  int32 id = 1;
  string name = 2;
  SpaceOptions options = 3;
  int64 created_at = 4;
}

message DropSpaceRequest {
  string name = 1;
  bool if_exists = 2;
}

message DropSpaceResponse {
  bool success = 1;
  string error = 2;
}

message ListSpacesRequest {}

message ListSpacesResponse {
  repeated SpaceInfo spaces = 1;
  string error = 2;
}

// ============================================================
// Schema 管理 - Tag
// ============================================================

message CreateTagRequest {
  string space_name = 1;
  string tag_name = 2;
  repeated PropertyDef properties = 3;
  optional TagOptions options = 4;
}

message PropertyDef {
  string name = 1;
  PropertyType type = 2;
  bool nullable = 3;
  optional Value default_value = 4;
  bool is_primary_key = 5;
}

enum PropertyType {
  PROPERTY_TYPE_BOOL = 0;
  PROPERTY_TYPE_INT = 1;
  PROPERTY_TYPE_FLOAT = 2;
  PROPERTY_TYPE_DOUBLE = 3;
  PROPERTY_TYPE_STRING = 4;
  PROPERTY_TYPE_TIMESTAMP = 5;
  PROPERTY_TYPE_DATE = 6;
  PROPERTY_TYPE_DATETIME = 7;
  PROPERTY_TYPE_VID = 8;
  PROPERTY_TYPE_EDGE = 9;
  PROPERTY_TYPE_TAG = 10;
  PROPERTY_TYPE_LIST = 11;
  PROPERTY_TYPE_SET = 12;
  PROPERTY_TYPE_MAP = 13;
}

message TagOptions {
  int64 ttl_seconds = 1;
  string ttl_column = 2;
}

message CreateTagResponse {
  bool success = 1;
  int32 tag_id = 2;
  string error = 3;
}

message GetTagRequest {
  string space_name = 1;
  string tag_name = 2;
}

message GetTagResponse {
  bool exists = 1;
  TagInfo tag = 2;
  string error = 3;
}

message TagInfo {
  int32 id = 1;
  string name = 2;
  repeated PropertyDef properties = 3;
  TagOptions options = 4;
  int64 created_at = 5;
}

message ListTagsRequest {
  string space_name = 1;
}

message ListTagsResponse {
  repeated TagInfo tags = 1;
  string error = 2;
}

message DropTagRequest {
  string space_name = 1;
  string tag_name = 2;
  bool if_exists = 3;
}

message DropTagResponse {
  bool success = 1;
  string error = 2;
}

// ============================================================
// Schema 管理 - Edge Type
// ============================================================

message CreateEdgeTypeRequest {
  string space_name = 1;
  string edge_type_name = 2;
  repeated PropertyDef properties = 3;
  optional EdgeTypeOptions options = 4;
}

message EdgeTypeOptions {
  bool directed = 1;
  int64 ttl_seconds = 2;
  string ttl_column = 3;
}

message CreateEdgeTypeResponse {
  bool success = 1;
  int32 edge_type_id = 2;
  string error = 3;
}

message GetEdgeTypeRequest {
  string space_name = 1;
  string edge_type_name = 2;
}

message GetEdgeTypeResponse {
  bool exists = 1;
  EdgeTypeInfo edge_type = 2;
  string error = 3;
}

message EdgeTypeInfo {
  int32 id = 1;
  string name = 2;
  repeated PropertyDef properties = 3;
  EdgeTypeOptions options = 4;
  int64 created_at = 5;
}

message ListEdgeTypesRequest {
  string space_name = 1;
}

message ListEdgeTypesResponse {
  repeated EdgeTypeInfo edge_types = 1;
  string error = 2;
}

message DropEdgeTypeRequest {
  string space_name = 1;
  string edge_type_name = 2;
  bool if_exists = 3;
}

message DropEdgeTypeResponse {
  bool success = 1;
  string error = 2;
}

// ============================================================
// 批量操作
// ============================================================

message CreateBatchRequest {
  string space_name = 1;
  optional BatchOptions options = 2;
}

message BatchOptions {
  int64 timeout_ms = 1;
  bool atomic = 2;
}

message CreateBatchResponse {
  bool success = 1;
  string batch_id = 2;
  string error = 3;
}

message AddBatchItemsRequest {
  string batch_id = 1;
  repeated BatchItem items = 2;
}

message BatchItem {
  oneof operation {
    InsertVertex insert_vertex = 1;
    InsertEdge insert_edge = 2;
    UpdateVertex update_vertex = 3;
    UpdateEdge update_edge = 4;
    DeleteVertex delete_vertex = 5;
    DeleteEdge delete_edge = 6;
  }
}

message InsertVertex {
  string tag_name = 1;
  string vid = 2;
  map<string, Value> properties = 3;
}

message InsertEdge {
  string edge_type = 1;
  string src = 2;
  string dst = 3;
  int64 ranking = 4;
  map<string, Value> properties = 5;
}

message UpdateVertex {
  string tag_name = 1;
  string vid = 2;
  map<string, Value> properties = 3;
}

message UpdateEdge {
  string edge_type = 1;
  string src = 2;
  string dst = 3;
  int64 ranking = 4;
  map<string, Value> properties = 5;
}

message DeleteVertex {
  string vid = 1;
  repeated string tag_names = 2;
}

message DeleteEdge {
  string edge_type = 1;
  string src = 2;
  string dst = 3;
  int64 ranking = 4;
}

message AddBatchItemsResponse {
  bool success = 1;
  int32 items_added = 2;
  string error = 3;
}

message ExecuteBatchRequest {
  string batch_id = 1;
}

message ExecuteBatchResponse {
  bool success = 1;
  repeated BatchResult results = 2;
  string error = 3;
}

message BatchResult {
  bool success = 1;
  string error = 2;
}

message GetBatchStatusRequest {
  string batch_id = 1;
}

message GetBatchStatusResponse {
  string status = 1;  // PENDING, RUNNING, COMPLETED, FAILED, CANCELLED
  int32 total_items = 2;
  int32 processed_items = 3;
  int32 failed_items = 4;
  string error = 5;
}

message CancelBatchRequest {
  string batch_id = 1;
}

message CancelBatchResponse {
  bool success = 1;
  string error = 2;
}

// ============================================================
// 统计信息
// ============================================================

message GetSessionStatisticsRequest {
  optional string session_id = 1;
}

message GetSessionStatisticsResponse {
  int64 active_sessions = 1;
  int64 total_sessions = 2;
  int64 failed_sessions = 3;
  map<string, int64> session_by_user = 4;
}

message GetQueryStatisticsRequest {
  optional string session_id = 1;
  optional int64 from_timestamp = 2;
  optional int64 to_timestamp = 3;
}

message GetQueryStatisticsResponse {
  int64 total_queries = 1;
  int64 slow_queries = 2;
  int64 failed_queries = 3;
  int64 avg_execution_time_ms = 4;
  int64 max_execution_time_ms = 5;
  repeated SlowQuery slow_query_list = 6;
}

message SlowQuery {
  string query = 1;
  int64 execution_time_ms = 2;
  int64 timestamp = 3;
  optional string session_id = 4;
}

message GetDatabaseStatisticsRequest {}

message GetDatabaseStatisticsResponse {
  int32 total_spaces = 1;
  int64 total_vertices = 2;
  int64 total_edges = 3;
  int64 storage_size_bytes = 4;
}

message GetSystemStatisticsRequest {}

message GetSystemStatisticsResponse {
  double cpu_usage_percent = 1;
  int64 memory_used_bytes = 2;
  int64 memory_total_bytes = 3;
  double disk_usage_percent = 4;
  int32 active_connections = 5;
  int64 network_rx_bytes = 6;
  int64 network_tx_bytes = 7;
}

// ============================================================
// 配置管理
// ============================================================

message GetConfigRequest {}

message GetConfigResponse {
  map<string, ConfigSection> config = 1;
  string error = 2;
}

message ConfigSection {
  map<string, ConfigValue> values = 1;
}

message ConfigValue {
  oneof value {
    string string_value = 1;
    int64 int_value = 2;
    double double_value = 3;
    bool bool_value = 4;
  }
}

message UpdateConfigRequest {
  string section = 1;
  string key = 2;
  ConfigValue value = 3;
}

message UpdateConfigResponse {
  bool success = 1;
  string error = 2;
}

message ResetConfigRequest {
  string section = 1;
  string key = 2;
}

message ResetConfigResponse {
  bool success = 1;
  string error = 2;
}

// ============================================================
// 自定义函数
// ============================================================

message RegisterFunctionRequest {
  string name = 1;
  string function_type = 2;  // SCALAR, AGGREGATE, TABLE
  repeated string parameters = 3;
  string return_type = 4;
  string description = 5;
  string implementation = 6;  // 函数实现（如 Lua 代码或 WASM 二进制）
}

message RegisterFunctionResponse {
  bool success = 1;
  string function_id = 2;
  string error = 3;
}

message UnregisterFunctionRequest {
  string name = 1;
}

message UnregisterFunctionResponse {
  bool success = 1;
  string error = 2;
}

message ListFunctionsRequest {}

message ListFunctionsResponse {
  repeated FunctionInfo functions = 1;
  string error = 2;
}

message FunctionInfo {
  string name = 1;
  string function_type = 2;
  repeated string parameters = 3;
  string return_type = 4;
  string description = 5;
}

message GetFunctionInfoRequest {
  string name = 1;
}

message GetFunctionInfoResponse {
  bool exists = 1;
  FunctionInfo function = 2;
  string error = 3;
}

// ============================================================
// 向量索引
// ============================================================

message CreateVectorIndexRequest {
  string space_name = 1;
  string tag_name = 2;
  string field_name = 3;
  VectorIndexOptions options = 4;
}

message VectorIndexOptions {
  int32 dimension = 1;
  DistanceMetric metric = 2;
  string index_type = 3;  // HNSW, FLAT, IVF, etc.
  map<string, string> parameters = 4;
}

enum DistanceMetric {
  DISTANCE_METRIC_COSINE = 0;
  DISTANCE_METRIC_L2 = 1;
  DISTANCE_METRIC_DOT = 2;
}

message CreateVectorIndexResponse {
  bool success = 1;
  string error = 2;
}

message GetVectorIndexRequest {
  string space_name = 1;
  string tag_name = 2;
  string field_name = 3;
}

message GetVectorIndexResponse {
  bool exists = 1;
  VectorIndexInfo index = 2;
  string error = 3;
}

message VectorIndexInfo {
  string space_name = 1;
  string tag_name = 2;
  string field_name = 3;
  VectorIndexOptions options = 4;
  int64 created_at = 5;
  int64 indexed_vectors = 6;
}

message ListVectorIndexesRequest {
  optional string space_name = 1;
}

message ListVectorIndexesResponse {
  repeated VectorIndexInfo indexes = 1;
  string error = 2;
}

message DropVectorIndexRequest {
  string space_name = 1;
  string tag_name = 2;
  string field_name = 3;
}

message DropVectorIndexResponse {
  bool success = 1;
  string error = 2;
}

message SearchVectorRequest {
  string space_name = 1;
  string tag_name = 2;
  string field_name = 3;
  repeated float vector = 4;
  int32 limit = 5;
  SearchFilter filter = 6;
  SearchOptions options = 7;
}

message SearchFilter {
  string expression = 1;  // 过滤表达式
}

message SearchOptions {
  int32 ef_search = 1;  // HNSW 搜索参数
  bool with_vector = 2;  // 是否返回向量数据
}

message SearchVectorResponse {
  repeated VectorSearchResult results = 1;
  string error = 2;
}

message VectorSearchResult {
  string vid = 1;
  float score = 2;
  map<string, Value> properties = 3;
  repeated float vector = 4;  // 如果 with_vector=true
}
```

### 4. Build 脚本

创建 `build.rs`：

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate proto code for gRPC service
    // Only compile proto when "grpc" feature is enabled

    #[cfg(feature = "grpc")]
    {
        println!("cargo:rerun-if-changed=proto/graphdb.proto");

        tonic_build::configure()
            .build_server(true)
            .build_client(true)
            .out_dir(std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()))
            .compile_protos(&["proto/graphdb.proto"], &["proto/"])?;
    }

    Ok(())
}
```

### 5. gRPC 服务实现

创建 `src/api/server/grpc/mod.rs`：

```rust
//! gRPC Service Module
//!
//! Provides an interface to GraphDB services based on the gRPC protocol.

#![cfg(feature = "grpc")]

pub mod server;

// Proto module will be generated at compile time
pub mod proto {
    tonic::include_proto!("graphdb");
}

pub use server::{run_server, run_server_with_grpc_service, GraphDBService};
```

创建 `src/api/server/grpc/server.rs`：

```rust
//! gRPC Server Implementation
//!
//! Provides a gRPC-based interface to GraphDB services.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

use crate::api::server::http::AppState;
use crate::api::server::GraphService;
use crate::config::Config;
use crate::storage::StorageClient;

// Import generated proto types
use super::proto::graph_db_service_server::{
    GraphDBService as GraphDBServiceTrait, GraphDBServiceServer,
};
use super::proto::*;

/// GraphDB gRPC service implementation
pub struct GraphDBService<S: StorageClient + Clone + 'static> {
    app_state: AppState<S>,
    config: Config,
    start_time: Instant,
}

impl<S: StorageClient + Clone + 'static> GraphDBService<S> {
    /// Create a new gRPC service instance
    pub fn new(app_state: AppState<S>, config: Config) -> Self {
        Self {
            app_state,
            config,
            start_time: Instant::now(),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get application state
    pub fn app_state(&self) -> &AppState<S> {
        &self.app_state
    }
}

#[tonic::async_trait]
impl<S: StorageClient + Clone + Send + Sync + 'static> GraphDBServiceTrait for GraphDBService<S> {
    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let uptime = self.start_time.elapsed().as_secs();

        Ok(Response::new(HealthCheckResponse {
            healthy: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: uptime as i64,
        }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        // TODO: Implement authentication logic
        // This should integrate with the existing auth service

        Ok(Response::new(LoginResponse {
            success: true,
            session_id: "session_id".to_string(),
            error: String::new(),
        }))
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        let req = request.into_inner();

        // TODO: Implement logout logic

        Ok(Response::new(LogoutResponse {
            success: true,
            error: String::new(),
        }))
    }

    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionResponse>, Status> {
        let req = request.into_inner();

        // TODO: Implement session creation logic

        Ok(Response::new(CreateSessionResponse {
            success: true,
            session_id: "session_id".to_string(),
            space_id: 0,
            error: String::new(),
        }))
    }

    async fn get_session(
        &self,
        request: Request<GetSessionRequest>,
    ) -> Result<Response<GetSessionResponse>, Status> {
        let req = request.into_inner();

        // TODO: Implement session retrieval logic

        Ok(Response::new(GetSessionResponse {
            exists: true,
            session_id: req.session_id,
            username: "user".to_string(),
            space_id: 0,
            created_at: 0,
            last_accessed: 0,
        }))
    }

    async fn close_session(
        &self,
        request: Request<CloseSessionRequest>,
    ) -> Result<Response<CloseSessionResponse>, Status> {
        let req = request.into_inner();

        // TODO: Implement session close logic

        Ok(Response::new(CloseSessionResponse {
            success: true,
            error: String::new(),
        }))
    }

    async fn execute_query(
        &self,
        request: Request<ExecuteQueryRequest>,
    ) -> Result<Response<ExecuteQueryResponse>, Status> {
        let req = request.into_inner();

        // TODO: Implement query execution logic
        // This should integrate with the existing QueryApi

        Ok(Response::new(ExecuteQueryResponse {
            success: true,
            result: None,
            error: String::new(),
            metadata: None,
        }))
    }

    async fn validate_query(
        &self,
        request: Request<ValidateQueryRequest>,
    ) -> Result<Response<ValidateQueryResponse>, Status> {
        let req = request.into_inner();

        // TODO: Implement query validation logic

        Ok(Response::new(ValidateQueryResponse {
            valid: true,
            error: String::new(),
            parameter_names: vec![],
        }))
    }

    async fn execute_query_stream(
        &self,
        request: Request<ExecuteQueryRequest>,
    ) -> Result<Response<tonic::codec::Streaming<QueryResultChunk>>, Status> {
        // TODO: Implement streaming query execution
        unimplemented!("Streaming query not yet implemented")
    }

    async fn begin_transaction(
        &self,
        request: Request<BeginTransactionRequest>,
    ) -> Result<Response<BeginTransactionResponse>, Status> {
        let req = request.into_inner();

        // TODO: Implement transaction begin logic

        Ok(Response::new(BeginTransactionResponse {
            success: true,
            transaction_id: "txn_id".to_string(),
            error: String::new(),
        }))
    }

    async fn commit_transaction(
        &self,
        request: Request<CommitTransactionRequest>,
    ) -> Result<Response<CommitTransactionResponse>, Status> {
        let req = request.into_inner();

        // TODO: Implement transaction commit logic

        Ok(Response::new(CommitTransactionResponse {
            success: true,
            error: String::new(),
        }))
    }

    async fn rollback_transaction(
        &self,
        request: Request<RollbackTransactionRequest>,
    ) -> Result<Response<RollbackTransactionResponse>, Status> {
        let req = request.into_inner();

        // TODO: Implement transaction rollback logic

        Ok(Response::new(RollbackTransactionResponse {
            success: true,
            error: String::new(),
        }))
    }

    // ... 其他 RPC 方法的实现（Schema 管理、批量操作、统计信息等）
    // 为了简洁，这里省略了其他方法的实现
    // 实际实现时，需要按照上述模式实现所有 RPC 方法

    async fn create_space(
        &self,
        request: Request<CreateSpaceRequest>,
    ) -> Result<Response<CreateSpaceResponse>, Status> {
        unimplemented!("CreateSpace not yet implemented")
    }

    async fn get_space(
        &self,
        request: Request<GetSpaceRequest>,
    ) -> Result<Response<GetSpaceResponse>, Status> {
        unimplemented!("GetSpace not yet implemented")
    }

    async fn drop_space(
        &self,
        request: Request<DropSpaceRequest>,
    ) -> Result<Response<DropSpaceResponse>, Status> {
        unimplemented!("DropSpace not yet implemented")
    }

    async fn list_spaces(
        &self,
        request: Request<ListSpacesRequest>,
    ) -> Result<Response<ListSpacesResponse>, Status> {
        unimplemented!("ListSpaces not yet implemented")
    }

    async fn create_tag(
        &self,
        request: Request<CreateTagRequest>,
    ) -> Result<Response<CreateTagResponse>, Status> {
        unimplemented!("CreateTag not yet implemented")
    }

    async fn get_tag(
        &self,
        request: Request<GetTagRequest>,
    ) -> Result<Response<GetTagResponse>, Status> {
        unimplemented!("GetTag not yet implemented")
    }

    async fn list_tags(
        &self,
        request: Request<ListTagsRequest>,
    ) -> Result<Response<ListTagsResponse>, Status> {
        unimplemented!("ListTags not yet implemented")
    }

    async fn drop_tag(
        &self,
        request: Request<DropTagRequest>,
    ) -> Result<Response<DropTagResponse>, Status> {
        unimplemented!("DropTag not yet implemented")
    }

    async fn create_edge_type(
        &self,
        request: Request<CreateEdgeTypeRequest>,
    ) -> Result<Response<CreateEdgeTypeResponse>, Status> {
        unimplemented!("CreateEdgeType not yet implemented")
    }

    async fn get_edge_type(
        &self,
        request: Request<GetEdgeTypeRequest>,
    ) -> Result<Response<GetEdgeTypeResponse>, Status> {
        unimplemented!("GetEdgeType not yet implemented")
    }

    async fn list_edge_types(
        &self,
        request: Request<ListEdgeTypesRequest>,
    ) -> Result<Response<ListEdgeTypesResponse>, Status> {
        unimplemented!("ListEdgeTypes not yet implemented")
    }

    async fn drop_edge_type(
        &self,
        request: Request<DropEdgeTypeRequest>,
    ) -> Result<Response<DropEdgeTypeResponse>, Status> {
        unimplemented!("DropEdgeType not yet implemented")
    }

    async fn create_batch(
        &self,
        request: Request<CreateBatchRequest>,
    ) -> Result<Response<CreateBatchResponse>, Status> {
        unimplemented!("CreateBatch not yet implemented")
    }

    async fn add_batch_items(
        &self,
        request: Request<AddBatchItemsRequest>,
    ) -> Result<Response<AddBatchItemsResponse>, Status> {
        unimplemented!("AddBatchItems not yet implemented")
    }

    async fn execute_batch(
        &self,
        request: Request<ExecuteBatchRequest>,
    ) -> Result<Response<ExecuteBatchResponse>, Status> {
        unimplemented!("ExecuteBatch not yet implemented")
    }

    async fn get_batch_status(
        &self,
        request: Request<GetBatchStatusRequest>,
    ) -> Result<Response<GetBatchStatusResponse>, Status> {
        unimplemented!("GetBatchStatus not yet implemented")
    }

    async fn cancel_batch(
        &self,
        request: Request<CancelBatchRequest>,
    ) -> Result<Response<CancelBatchResponse>, Status> {
        unimplemented!("CancelBatch not yet implemented")
    }

    async fn get_session_statistics(
        &self,
        request: Request<GetSessionStatisticsRequest>,
    ) -> Result<Response<GetSessionStatisticsResponse>, Status> {
        unimplemented!("GetSessionStatistics not yet implemented")
    }

    async fn get_query_statistics(
        &self,
        request: Request<GetQueryStatisticsRequest>,
    ) -> Result<Response<GetQueryStatisticsResponse>, Status> {
        unimplemented!("GetQueryStatistics not yet implemented")
    }

    async fn get_database_statistics(
        &self,
        request: Request<GetDatabaseStatisticsRequest>,
    ) -> Result<Response<GetDatabaseStatisticsResponse>, Status> {
        unimplemented!("GetDatabaseStatistics not yet implemented")
    }

    async fn get_system_statistics(
        &self,
        request: Request<GetSystemStatisticsRequest>,
    ) -> Result<Response<GetSystemStatisticsResponse>, Status> {
        unimplemented!("GetSystemStatistics not yet implemented")
    }

    async fn get_config(
        &self,
        request: Request<GetConfigRequest>,
    ) -> Result<Response<GetConfigResponse>, Status> {
        unimplemented!("GetConfig not yet implemented")
    }

    async fn update_config(
        &self,
        request: Request<UpdateConfigRequest>,
    ) -> Result<Response<UpdateConfigResponse>, Status> {
        unimplemented!("UpdateConfig not yet implemented")
    }

    async fn reset_config(
        &self,
        request: Request<ResetConfigRequest>,
    ) -> Result<Response<ResetConfigResponse>, Status> {
        unimplemented!("ResetConfig not yet implemented")
    }

    async fn register_function(
        &self,
        request: Request<RegisterFunctionRequest>,
    ) -> Result<Response<RegisterFunctionResponse>, Status> {
        unimplemented!("RegisterFunction not yet implemented")
    }

    async fn unregister_function(
        &self,
        request: Request<UnregisterFunctionRequest>,
    ) -> Result<Response<UnregisterFunctionResponse>, Status> {
        unimplemented!("UnregisterFunction not yet implemented")
    }

    async fn list_functions(
        &self,
        request: Request<ListFunctionsRequest>,
    ) -> Result<Response<ListFunctionsResponse>, Status> {
        unimplemented!("ListFunctions not yet implemented")
    }

    async fn get_function_info(
        &self,
        request: Request<GetFunctionInfoRequest>,
    ) -> Result<Response<GetFunctionInfoResponse>, Status> {
        unimplemented!("GetFunctionInfo not yet implemented")
    }

    async fn create_vector_index(
        &self,
        request: Request<CreateVectorIndexRequest>,
    ) -> Result<Response<CreateVectorIndexResponse>, Status> {
        unimplemented!("CreateVectorIndex not yet implemented")
    }

    async fn get_vector_index(
        &self,
        request: Request<GetVectorIndexRequest>,
    ) -> Result<Response<GetVectorIndexResponse>, Status> {
        unimplemented!("GetVectorIndex not yet implemented")
    }

    async fn list_vector_indexes(
        &self,
        request: Request<ListVectorIndexesRequest>,
    ) -> Result<Response<ListVectorIndexesResponse>, Status> {
        unimplemented!("ListVectorIndexes not yet implemented")
    }

    async fn drop_vector_index(
        &self,
        request: Request<DropVectorIndexRequest>,
    ) -> Result<Response<DropVectorIndexResponse>, Status> {
        unimplemented!("DropVectorIndex not yet implemented")
    }

    async fn search_vector(
        &self,
        request: Request<SearchVectorRequest>,
    ) -> Result<Response<SearchVectorResponse>, Status> {
        unimplemented!("SearchVector not yet implemented")
    }
}

/// Run the gRPC server
pub async fn run_server<S: StorageClient + Clone + Send + Sync + 'static>(
    app_state: AppState<S>,
    config: Config,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let service = GraphDBService::new(app_state, config);

    tracing::info!("GraphDB gRPC service listening on {}", addr);

    Server::builder()
        .add_service(GraphDBServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

/// Run the gRPC server with custom service instance
pub async fn run_server_with_grpc_service<S: StorageClient + Clone + Send + Sync + 'static>(
    service: GraphDBService<S>,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("GraphDB gRPC service listening on {}", addr);

    Server::builder()
        .add_service(GraphDBServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        // Test that the service can be created
        // Note: This is a placeholder test
        // Actual tests would require mocking AppState and Config
    }
}
```

### 6. 修改主程序支持 gRPC

修改 `src/api/mod.rs`，添加 gRPC 启动函数：

```rust
/// Start both HTTP and gRPC servers
#[cfg(all(feature = "server", feature = "grpc"))]
pub async fn start_http_and_grpc_servers<S: crate::storage::StorageClient + Clone + Send + Sync + 'static>(
    http_server: Arc<HttpServer<S>>,
    config: &Config,
) -> DBResult<()> {
    use tokio::task;
    use tokio::net::TcpListener;
    use axum::serve;

    let http_state = crate::api::server::http::AppState::new(http_server.clone());

    // Create WebState for web management APIs
    let storage_path = format!("{}/metadata.db", config.storage_path());
    let web_router =
        match crate::api::server::web::WebState::new(&storage_path, http_state.clone()).await {
            Ok(web_state) => Some(crate::api::server::web::create_router(web_state)),
            Err(e) => {
                log::warn!(
                    "Failed to initialize web management: {}, continuing without it",
                    e
                );
                None
            }
        };

    let http_app = crate::api::server::http::router::create_router(http_state.clone(), web_router);

    // Setup gRPC address
    let grpc_addr = format!("{}:{}", config.host(), config.grpc_port())
        .parse::<std::net::SocketAddr>()
        .map_err(|e| crate::core::error::DBError::Other(e.to_string()))?;

    // Setup HTTP address
    let http_addr = format!("{}:{}", config.host(), config.port());

    info!("HTTP server listening on {}", http_addr);
    info!("gRPC server listening on {}", grpc_addr);

    // Clone state for gRPC server
    let grpc_state = http_state.clone();
    let grpc_config = config.clone();

    // Start HTTP server
    let http_future = async move {
        let http_listener = TcpListener::bind(&http_addr).await?;
        serve(http_listener, http_app)
            .with_graceful_shutdown(async_shutdown_signal())
            .await?;
        Ok::<(), crate::core::error::DBError>(())
    };

    // Start gRPC server
    let grpc_future = async move {
        crate::api::server::grpc::run_server(grpc_state, grpc_config, grpc_addr)
            .await
            .map_err(|e| crate::core::error::DBError::Other(e.to_string()))?;
        Ok::<(), crate::core::error::DBError>(())
    };

    // Run both servers concurrently
    tokio::select! {
        result = http_future => result?,
        result = grpc_future => result?,
    }

    Ok(())
}
```

### 7. 添加配置项

在 `src/config.rs` 中添加 gRPC 端口配置：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // ... existing fields ...

    /// gRPC server port
    #[serde(default = "Config::default_grpc_port")]
    grpc_port: u16,
}

impl Config {
    fn default_grpc_port() -> u16 {
        9669  // Default gRPC port for GraphDB
    }

    /// Get the gRPC port
    pub fn grpc_port(&self) -> u16 {
        self.grpc_port
    }

    /// Set the gRPC port
    pub fn set_grpc_port(&mut self, port: u16) {
        self.grpc_port = port;
    }
}
```

### 8. 更新主程序入口

修改 `src/main.rs` 以支持 gRPC：

```rust
#[cfg(feature = "server")]
mod server_main {
    use clap::Parser;
    use graphdb::api;
    use graphdb::config::Config;
    use graphdb::core::error::DBResult;
    use graphdb::utils::{logging, output};

    #[derive(Parser)]
    #[clap(version = "0.1.0", author = "GraphDB Contributors")]
    enum Cli {
        /// Start the GraphDB service
        Serve {
            #[clap(short, long, default_value = "config.toml")]
            config: String,
            #[clap(long, help = "Enable gRPC server")]
            enable_grpc: bool,
        },
        /// Execute a query directly
        Query {
            #[clap(short, long)]
            query: String,
        },
    }

    pub fn main() -> DBResult<()> {
        let cli = Cli::parse();

        match cli {
            Cli::Serve { config, enable_grpc } => {
                let _ = output::print_info(&format!(
                    "Starting GraphDB service with config: {}",
                    config
                ));
                let _ = output::print_info(&format!("Process ID: {}", std::process::id()));

                // Load configuration
                let cfg = match Config::load(&config) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        let _ = output::print_error(&format!(
                            "Failed to load configuration file: {}, using default configuration",
                            e
                        ));
                        Config::default()
                    }
                };

                // Initialize logging system
                if let Err(e) = logging::init(&cfg) {
                    let _ =
                        output::print_error(&format!("Failed to initialize logging system: {}", e));
                }

                // Initialize and start service
                let rt = tokio::runtime::Runtime::new().unwrap();

                #[cfg(feature = "grpc")]
                {
                    if enable_grpc {
                        rt.block_on(api::start_http_and_grpc_servers(cfg))?;
                    } else {
                        rt.block_on(api::start_service_with_config(cfg))?;
                    }
                }

                #[cfg(not(feature = "grpc"))]
                {
                    if enable_grpc {
                        let _ = output::print_error("gRPC feature is not enabled");
                    }
                    rt.block_on(api::start_service_with_config(cfg))?;
                }

                // Ensure logging is flushed before exiting
                logging::shutdown();
            }
            Cli::Query { query } => {
                let _ = output::print_info(&format!("Executing query: {}", query));
                let _ = output::print_info(&format!("Process ID: {}", std::process::id()));

                // Use default configuration to initialize logging
                let cfg = Config::default();
                if let Err(e) = logging::init(&cfg) {
                    let _ =
                        output::print_error(&format!("Failed to initialize logging system: {}", e));
                }

                // Execute query directly using tokio runtime
                let rt = tokio::runtime::Runtime::new()?;
                let result = rt.block_on(api::execute_query(&query));

                // Ensure logging is flushed before exiting
                logging::shutdown();
                result?;
            }
        }

        Ok(())
    }
}
```

## 实施步骤

### 第一阶段：基础架构（P0）

1. ✅ 更新 `Cargo.toml` 添加 gRPC 依赖
2. ✅ 创建 `proto/graphdb.proto` 文件
3. ✅ 创建 `build.rs` 文件
4. ✅ 创建 `src/api/server/grpc/mod.rs`
5. ✅ 创建 `src/api/server/grpc/server.rs`

### 第二阶段：核心功能实现（P1）

6. ✅ 实现基础 RPC 方法（HealthCheck, Login, Logout）
7. ✅ 实现会话管理 RPC 方法
8. ✅ 实现查询执行 RPC 方法
9. ✅ 实现事务管理 RPC 方法

### 第三阶段：Schema 和批量操作（P2）

10. ✅ 实现 Schema 管理 RPC 方法
11. ✅ 实现批量操作 RPC 方法
12. ✅ 实现统计信息 RPC 方法

### 第四阶段：高级功能（P3）

13. ⏳ 实现配置管理 RPC 方法
14. ⏳ 实现自定义函数 RPC 方法
15. ⏳ 实现向量索引 RPC 方法
16. ⏳ 实现流式查询功能

## 版本兼容性

### 依赖版本（与 inversearch 保持一致）

- `tonic`: 0.12
- `prost`: 0.13
- `tonic-build`: 0.12
- `prost-build`: 0.13

### 代码风格

- 遵循 `crates/inversearch/src/api/server/grpc.rs` 的架构模式
- 使用条件编译 `#![cfg(feature = "grpc")]`
- 服务实现与 HTTP handler 分离，但复用核心 API

## 测试

### 单元测试

在 `src/api/server/grpc/server.rs` 中添加测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        // Test service creation
    }

    #[tokio::test]
    async fn test_health_check() {
        // Test health check RPC
    }
}
```

### 集成测试

创建 `tests/grpc_integration_test.rs`：

```rust
//! gRPC Integration Tests

#[cfg(all(feature = "server", feature = "grpc"))]
mod tests {
    use graphdb::api::server::grpc::{GraphDBService, run_server};
    use graphdb::api::server::http::AppState;
    use graphdb::config::Config;
    use tonic::transport::Channel;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_grpc_server_startup() {
        // Test that the gRPC server can start successfully
    }

    #[tokio::test]
    async fn test_grpc_health_check() {
        // Test health check via gRPC
    }
}
```

## 参考资料

- [Inversearch gRPC 实现](../../../crates/inversearch/src/api/server/grpc.rs)
- [BM25 gRPC 实现](../../../crates/bm25/src/api/server/grpc.rs)
- [Tonic 官方文档](https://docs.rs/tonic)
- [Protobuf 语言指南](https://protobuf.dev/programming-guides/proto3/)

## 注意事项

1. **条件编译**：所有 gRPC 相关代码必须使用 `#![cfg(feature = "grpc")]` 标记
2. **错误处理**：使用 `tonic::Status` 进行错误处理，保持一致性
3. **性能考虑**：对于大数据量查询，优先使用流式接口
4. **向后兼容**：proto 文件修改需要保持向后兼容
5. **文档完整性**：所有公共 API 都需要完整的文档注释

## 总结

本文档详细描述了如何为 GraphDB 添加 gRPC 端口支持，参考了 `crates/inversearch` 的架构设计，确保了版本一致性和代码风格统一。实施过程分为四个阶段，逐步完善 gRPC 功能。
