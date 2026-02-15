//! E2E 测试模块
//!
//! 包含完整的端到端测试套件，覆盖业务场景、工作流、性能和回归测试。
//!
//! # 测试结构
//!
//! ```
//! tests/e2e/
//! ├── common/          # 测试基础设施
//! │   ├── mod.rs       # 测试上下文和核心功能
//! │   ├── assertions.rs # 断言工具
//! │   └── data_generators.rs # 数据生成器
//! ├── scenarios/       # 业务场景测试
//! │   ├── mod.rs
//! │   ├── social_network.rs
//! │   ├── e_commerce.rs
//! │   └── knowledge_graph.rs
//! ├── workflows/       # 工作流测试
//! │   ├── mod.rs
//! │   ├── schema_evolution.rs
//! │   └── data_migration.rs
//! ├── performance/     # 性能测试
//! │   ├── mod.rs
//! │   ├── concurrent_operations.rs
//! │   └── bulk_operations.rs
//! └── regression/      # 回归测试
//!     ├── mod.rs
//!     └── core_features.rs
//! ```
//!
//! # 运行测试
//!
//! ```bash
//! # 运行所有 E2E 测试
//! cargo test --test e2e
//!
//! # 运行特定模块
//! cargo test --test e2e scenarios
//! cargo test --test e2e workflows
//! cargo test --test e2e performance
//! cargo test --test e2e regression
//!
//! # 运行特定测试用例
//! cargo test --test e2e test_sns_user_registration
//! ```

pub mod common;
pub mod scenarios;
pub mod workflows;
pub mod performance;
pub mod regression;

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证 E2E 测试环境可以正常初始化
    #[tokio::test]
    async fn test_e2e_environment_setup() {
        let ctx = common::E2eTestContext::new().await.expect("创建上下文失败");
        
        // 验证可以执行基本查询
        let result = ctx.execute_query("SHOW SPACES").await;
        assert!(result.is_ok(), "基本查询应该成功");
    }

    /// 验证测试数据生成器可以正常工作
    #[tokio::test]
    async fn test_data_generator_setup() {
        let ctx = common::E2eTestContext::new().await.expect("创建上下文失败");
        let generator = common::data_generators::SocialNetworkDataGenerator::new(&ctx);
        
        // 验证可以生成基础模式
        let result = generator.generate_base_schema().await;
        if let Err(ref e) = result {
            println!("生成基础模式失败: {}", e);
        }
        assert!(result.is_ok(), "生成基础模式应该成功: {:?}", result.err());
    }
}
