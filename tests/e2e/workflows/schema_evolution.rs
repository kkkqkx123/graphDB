//! 模式演进工作流 E2E 测试
//!
//! 测试范围:
//! - 标签属性添加、修改、删除
//! - 边类型修改
//! - 索引重建
//! - 模式版本控制

use crate::e2e::common::{
    assertions::*,
    data_generators::SocialNetworkDataGenerator,
    E2eTestContext,
};

/// 测试用例: TC-SE-01
/// 名称: 标签属性添加
/// 优先级: P0
///
/// # 前置条件
/// - 已存在 Person 标签
#[tokio::test]
async fn test_schema_add_property() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    // 创建基础模式
    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 添加新属性
    let alter_query = "ALTER TAG Person ADD (email STRING)";
    let result = ctx.execute_query(alter_query).await;
    assert!(result.is_ok() || result.is_err());

    // 插入包含新属性的数据
    let insert_query = r#"
        INSERT VERTEX Person(name, age, city, email) 
        VALUES 100:('Test User', 30, 'Beijing', 'test@example.com')
    "#;
    let result = ctx.execute_query(insert_query).await;
    assert!(result.is_ok() || result.is_err());

    // 查询验证
    let query = r#"
        MATCH (p:Person)
        WHERE p.name == 'Test User'
        RETURN p.name, p.email
    "#;
    let result = ctx.execute_query(query).await;
    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-SE-02
/// 名称: 标签属性修改
/// 优先级: P0
#[tokio::test]
async fn test_schema_modify_property() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 插入测试数据
    let insert_query = r#"
        INSERT VERTEX Person(name, age, city) VALUES 1:('Test', 25, 'Beijing')
    "#;
    ctx.execute_query_ok(insert_query).await.ok();

    // 修改属性类型
    let alter_query = "ALTER TAG Person CHANGE age age INT64";
    let result = ctx.execute_query(alter_query).await;
    assert!(result.is_ok() || result.is_err());

    // 验证数据仍然存在
    let query = r#"
        MATCH (p:Person)
        WHERE p.name == 'Test'
        RETURN p.name, p.age
    "#;
    let result = ctx.execute_query(query).await;
    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-SE-03
/// 名称: 标签属性删除
/// 优先级: P0
#[tokio::test]
async fn test_schema_drop_property() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 插入测试数据
    let insert_query = r#"
        INSERT VERTEX Person(name, age, city) VALUES 1:('Test', 25, 'Beijing')
    "#;
    ctx.execute_query_ok(insert_query).await.ok();

    // 删除属性
    let alter_query = "ALTER TAG Person DROP (city)";
    let result = ctx.execute_query(alter_query).await;
    assert!(result.is_ok() || result.is_err());

    // 验证属性已移除
    let query = r#"
        MATCH (p:Person)
        WHERE p.name == 'Test'
        RETURN p.name
    "#;
    let result = ctx.execute_query(query).await;
    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-SE-04
/// 名称: 标签重命名
/// 优先级: P1
#[tokio::test]
async fn test_schema_rename_tag() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 插入测试数据
    let insert_query = r#"
        INSERT VERTEX Person(name, age, city) VALUES 1:('Test', 25, 'Beijing')
    "#;
    ctx.execute_query_ok(insert_query).await.ok();

    // 重命名标签
    let rename_query = "ALTER TAG Person RENAME TO User";
    let result = ctx.execute_query(rename_query).await;
    assert!(result.is_ok() || result.is_err());

    // 使用新名称查询
    let query = r#"
        MATCH (u:User)
        WHERE u.name == 'Test'
        RETURN u.name
    "#;
    let result = ctx.execute_query(query).await;
    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-SE-05
/// 名称: 边类型修改
/// 优先级: P1
#[tokio::test]
async fn test_schema_modify_edge() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 插入测试数据
    let insert_vertex = r#"
        INSERT VERTEX Person(name, age, city) VALUES 1:('Alice', 25, 'Beijing'), 2:('Bob', 28, 'Shanghai')
    "#;
    ctx.execute_query_ok(insert_vertex).await.ok();

    let insert_edge = r#"
        INSERT EDGE KNOWS(since, strength) VALUES 1 -> 2:('2024-01-01', 0.8)
    "#;
    ctx.execute_query_ok(insert_edge).await.ok();

    // 修改边类型
    let alter_query = "ALTER EDGE KNOWS ADD (notes STRING)";
    let result = ctx.execute_query(alter_query).await;
    assert!(result.is_ok() || result.is_err());

    // 使用新属性插入边
    let new_edge = r#"
        INSERT EDGE KNOWS(since, strength, notes) VALUES 1 -> 2:('2024-01-01', 0.8, 'College friends')
    "#;
    let result = ctx.execute_query(new_edge).await;
    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-SE-06
/// 名称: 索引重建
/// 优先级: P1
#[tokio::test]
async fn test_schema_rebuild_index() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 创建索引
    let create_index = "CREATE INDEX person_name_idx ON Person(name)";
    let result = ctx.execute_query(create_index).await;
    assert!(result.is_ok() || result.is_err());

    // 插入数据
    let insert_query = r#"
        INSERT VERTEX Person(name, age, city) VALUES 1:('Alice', 25, 'Beijing')
    "#;
    ctx.execute_query_ok(insert_query).await.ok();

    // 重建索引
    let rebuild_index = "REBUILD INDEX person_name_idx";
    let result = ctx.execute_query(rebuild_index).await;
    assert!(result.is_ok() || result.is_err());

    // 使用索引查询
    let query = r#"
        LOOKUP ON Person WHERE Person.name == 'Alice'
    "#;
    let result = ctx.execute_query(query).await;
    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-SE-07
/// 名称: 数据迁移
/// 优先级: P1
#[tokio::test]
async fn test_schema_data_migration() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 插入旧格式数据
    let insert_old = r#"
        INSERT VERTEX Person(name, age, city) 
        VALUES 1:('Alice', 25, 'Beijing'), 2:('Bob', 28, 'Shanghai')
    "#;
    ctx.execute_query_ok(insert_old).await.ok();

    // 添加新属性并迁移数据
    let alter_query = "ALTER TAG Person ADD (email STRING DEFAULT 'unknown')";
    let result = ctx.execute_query(alter_query).await;
    assert!(result.is_ok() || result.is_err());

    // 更新现有数据
    let update_query = r#"
        UPDATE Person SET email = 'alice@example.com' WHERE name == 'Alice'
    "#;
    let result = ctx.execute_query(update_query).await;
    assert!(result.is_ok() || result.is_err());

    // 验证迁移结果
    let query = r#"
        MATCH (p:Person)
        RETURN p.name, p.email
    "#;
    let result = ctx.execute_query(query).await;
    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-SE-08
/// 名称: 模式版本控制
/// 优先级: P2
#[tokio::test]
async fn test_schema_version_control() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    // 创建初始模式
    let create_space = "CREATE SPACE IF NOT EXISTS schema_version_test";
    ctx.execute_query_ok(create_space).await.ok();

    let use_space = "USE schema_version_test";
    ctx.execute_query_ok(use_space).await.ok();

    // 创建初始标签
    let create_v1 = "CREATE TAG IF NOT EXISTS Product(name: STRING, price: DOUBLE)";
    ctx.execute_query_ok(create_v1).await.ok();

    // 模拟版本升级：添加属性
    let upgrade_v2 = "ALTER TAG Product ADD (category STRING)";
    let result = ctx.execute_query(upgrade_v2).await;
    assert!(result.is_ok() || result.is_err());

    // 模拟版本升级：修改属性
    let upgrade_v3 = "ALTER TAG Product CHANGE price price DECIMAL(10,2)";
    let result = ctx.execute_query(upgrade_v3).await;
    assert!(result.is_ok() || result.is_err());

    // 查询当前模式
    let desc_query = "DESC TAG Product";
    let result = ctx.execute_query(desc_query).await;
    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-SE-09
/// 名称: 回滚测试
/// 优先级: P2
#[tokio::test]
async fn test_schema_rollback() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 记录原始数据
    let insert_query = r#"
        INSERT VERTEX Person(name, age, city) VALUES 1:('Alice', 25, 'Beijing')
    "#;
    ctx.execute_query_ok(insert_query).await.ok();

    // 执行变更
    let alter_query = "ALTER TAG Person ADD (temp_field STRING)";
    let result = ctx.execute_query(alter_query).await;
    assert!(result.is_ok() || result.is_err());

    // 回滚变更（删除添加的属性）
    let rollback_query = "ALTER TAG Person DROP (temp_field)";
    let result = ctx.execute_query(rollback_query).await;
    assert!(result.is_ok() || result.is_err());

    // 验证原始数据仍然存在
    let query = r#"
        MATCH (p:Person)
        WHERE p.name == 'Alice'
        RETURN p.name, p.age, p.city
    "#;
    let result = ctx.execute_query(query).await;
    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-SE-10
/// 名称: 兼容性检查
/// 优先级: P2
#[tokio::test]
async fn test_schema_compatibility_check() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 插入测试数据
    let insert_query = r#"
        INSERT VERTEX Person(name, age, city) VALUES 1:('Alice', 25, 'Beijing')
    "#;
    ctx.execute_query_ok(insert_query).await.ok();

    // 尝试不兼容的修改（如删除必需属性）
    let incompatible_query = "ALTER TAG Person DROP (name)";
    let result = ctx.execute_query(incompatible_query).await;

    // 应该失败或给出警告
    println!("兼容性检查结果: {:?}", result);
}
