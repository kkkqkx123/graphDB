//! gRPC 服务集成测试入口
//!
//! 统一导出 service 子目录下的所有测试模块

// 首先声明 common 模块
mod common;

#[cfg(feature = "service")]
mod service {
    mod grpc_test;
    mod stats_test;
}
