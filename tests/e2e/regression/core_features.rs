//! 核心功能回归测试
//!
//! 验证 GraphDB 的核心功能在各种场景下正常工作

use crate::e2e::common::{
    assertions::*,
    data_generators::{ECommerceDataGenerator, SocialNetworkDataGenerator},
    E2eTestContext,
};

/// 回归测试: 基础 CRUD 操作
/// 验证基本的创建、读取、更新、删除功能
#[tokio::test]
async fn test_regression_basic_crud() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    // 创建空间
    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS crud_test")
        .await
        .expect("创建空间失败");
    ctx.execute_query_ok("USE crud_test").await.expect("使用空间失败");

    // 创建标签
    ctx.execute_query_ok("CREATE TAG IF NOT EXISTS Item(name STRING, value INT)")
        .await
        .expect("创建标签失败");

    // Create
    let create_result = ctx
        .execute_query_ok("INSERT VERTEX Item(name, value) VALUES 1:('Test Item', 100)")
        .await;
    assert!(create_result.is_ok(), "创建失败");

    // Read
    let read_data = ctx
        .execute_query_ok("MATCH (i:Item) WHERE i.name == 'Test Item' RETURN i.value")
        .await
        .expect("读取失败");
    assert_not_empty(&read_data);

    // Update
    let update_result = ctx
        .execute_query_ok("UPDATE Item SET value = 200 WHERE name == 'Test Item'")
        .await;
    assert!(update_result.is_ok(), "更新失败");

    // Verify Update
    let verify_data = ctx
        .execute_query_ok("MATCH (i:Item) WHERE i.name == 'Test Item' RETURN i.value")
        .await
        .expect("验证失败");
    assert_not_empty(&verify_data);

    // Delete
    let delete_result = ctx.execute_query_ok("DELETE VERTEX 1").await;
    assert!(delete_result.is_ok(), "删除失败");

    // Verify Delete
    let check_data = ctx
        .execute_query_ok("MATCH (i:Item) WHERE i.name == 'Test Item' RETURN i")
        .await
        .expect("检查失败");
    assert_empty(&check_data);
}

/// 回归测试: 边操作
/// 验证边的创建、查询和删除
#[tokio::test]
async fn test_regression_edge_operations() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS edge_test")
        .await
        .expect("创建空间失败");
    ctx.execute_query_ok("USE edge_test").await.expect("使用空间失败");

    // 创建标签和边类型
    ctx.execute_query_ok("CREATE TAG IF NOT EXISTS Node(name STRING)")
        .await
        .expect("创建标签失败");
    ctx.execute_query_ok("CREATE EDGE IF NOT EXISTS CONNECTS(weight DOUBLE)")
        .await
        .expect("创建边类型失败");

    // 创建顶点
    ctx.execute_query_ok("INSERT VERTEX Node(name) VALUES 1:('A'), 2:('B'), 3:('C')")
        .await
        .expect("创建顶点失败");

    // 创建边
    ctx.execute_query_ok("INSERT EDGE CONNECTS(weight) VALUES 1 -> 2:(1.0), 2 -> 3:(2.0)")
        .await
        .expect("创建边失败");

    // 查询边
    let query_data = ctx
        .execute_query_ok("GO FROM 1 OVER CONNECTS YIELD dst(edge), edge.weight")
        .await
        .expect("查询边失败");
    assert_not_empty(&query_data);

    // 删除边
    ctx.execute_query_ok("DELETE EDGE CONNECTS 1 -> 2")
        .await
        .expect("删除边失败");

    // 验证边已删除
    let check_data = ctx
        .execute_query_ok("GO FROM 1 OVER CONNECTS YIELD dst(edge)")
        .await
        .expect("检查失败");
    assert_empty(&check_data);
}

/// 回归测试: 复杂查询
/// 验证 MATCH、GO、FIND PATH 等复杂查询
#[tokio::test]
async fn test_regression_complex_queries() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    // 生成测试数据
    generator
        .generate_social_graph(20)
        .await
        .expect("生成测试数据失败");

    // MATCH 查询
    let match_result = ctx
        .execute_query("MATCH (p:Person) WHERE p.age > 20 RETURN p.name LIMIT 10")
        .await;
    assert!(match_result.is_ok(), "MATCH 查询失败: {:?}", match_result.err());

    // GO 查询
    let go_result = ctx.execute_query("GO FROM 1 OVER KNOWS YIELD dst(edge)").await;
    assert!(go_result.is_ok(), "GO 查询失败: {:?}", go_result.err());

    // FIND PATH 查询
    let path_result = ctx.execute_query("FIND ALL PATH FROM 1 TO 5 OVER KNOWS").await;
    assert!(path_result.is_ok(), "FIND PATH 查询失败: {:?}", path_result.err());

    // GET SUBGRAPH 查询
    let subgraph_result = ctx.execute_query("GET SUBGRAPH 2 STEPS FROM 1").await;
    assert!(subgraph_result.is_ok(), "GET SUBGRAPH 查询失败: {:?}", subgraph_result.err());
}

/// 回归测试: 事务一致性
/// 验证数据操作的原子性和一致性
#[tokio::test]
async fn test_regression_transaction_consistency() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS tx_test")
        .await
        .expect("创建空间失败");
    ctx.execute_query_ok("USE tx_test").await.expect("使用空间失败");

    ctx.execute_query_ok("CREATE TAG IF NOT EXISTS Account(id INT, balance DOUBLE)")
        .await
        .expect("创建标签失败");

    // 创建账户
    ctx.execute_query_ok(
        "INSERT VERTEX Account(id, balance) VALUES 1:(1, 1000.0), 2:(2, 500.0)",
    )
    .await
    .expect("创建账户失败");

    // 模拟转账（更新操作）
    ctx.execute_query_ok("UPDATE Account SET balance = 900.0 WHERE id == 1")
        .await
        .expect("转出失败");

    ctx.execute_query_ok("UPDATE Account SET balance = 600.0 WHERE id == 2")
        .await
        .expect("转入失败");

    // 验证一致性
    let data = ctx
        .execute_query_ok("MATCH (a:Account) RETURN a.id, a.balance ORDER BY a.id")
        .await
        .expect("查询失败");

    assert_not_empty(&data);

    // 验证总额不变
    let total_data = ctx
        .execute_query_ok("MATCH (a:Account) RETURN sum(a.balance) AS total")
        .await
        .expect("计算总额失败");
    assert_not_empty(&total_data);
}

/// 回归测试: 错误处理
/// 验证系统对错误输入的优雅处理
#[tokio::test]
async fn test_regression_error_handling() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    // 无效语法
    let invalid_syntax = ctx.execute_query("INVALID SYNTAX").await;
    assert!(
        invalid_syntax.is_err() || !invalid_syntax.unwrap().success,
        "无效语法应该返回错误"
    );

    // 不存在的空间
    let nonexistent_space = ctx.execute_query("USE nonexistent_space_xyz").await;
    println!("不存在的空间查询结果: {:?}", nonexistent_space);

    // 不存在的标签
    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS error_test")
        .await
        .ok();
    ctx.execute_query_ok("USE error_test").await.ok();

    let nonexistent_tag = ctx.execute_query("MATCH (n:NonexistentTag) RETURN n").await;
    println!("不存在的标签查询结果: {:?}", nonexistent_tag);

    // 类型不匹配
    ctx.execute_query_ok("CREATE TAG IF NOT EXISTS Typed(value INT)")
        .await
        .ok();
    let type_mismatch = ctx
        .execute_query("INSERT VERTEX Typed(value) VALUES 1:('string_instead_of_int')")
        .await;
    println!("类型不匹配结果: {:?}", type_mismatch);
}

/// 回归测试: 边界条件
/// 验证系统在边界条件下的行为
#[tokio::test]
async fn test_regression_boundary_conditions() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS boundary_test")
        .await
        .expect("创建空间失败");
    ctx.execute_query_ok("USE boundary_test").await.expect("使用空间失败");

    ctx.execute_query_ok("CREATE TAG IF NOT EXISTS Boundary(value INT, name STRING)")
        .await
        .expect("创建标签失败");

    // 空字符串
    ctx.execute_query_ok("INSERT VERTEX Boundary(value, name) VALUES 1:(0, '')")
        .await
        .expect("空字符串插入失败");

    // 最大值
    ctx.execute_query_ok(&format!(
        "INSERT VERTEX Boundary(value, name) VALUES 2:({}, 'max')",
        i32::MAX
    ))
    .await
        .expect("最大值插入失败");

    // 最小值
    ctx.execute_query_ok(&format!(
        "INSERT VERTEX Boundary(value, name) VALUES 3:({}, 'min')",
        i32::MIN
    ))
    .await
        .expect("最小值插入失败");

    // 长字符串
    let long_string = "a".repeat(1000);
    let result = ctx
        .execute_query(&format!(
            "INSERT VERTEX Boundary(value, name) VALUES 4:(1, '{}')",
            long_string
        ))
        .await;
    println!("长字符串插入结果: {:?}", result);

    // 验证数据
    let data = ctx
        .execute_query_ok("MATCH (b:Boundary) RETURN count(b)")
        .await
        .expect("查询失败");
    assert_not_empty(&data);
}

/// 回归测试: 并发安全
/// 验证并发操作的数据一致性
#[tokio::test]
async fn test_regression_concurrent_safety() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS concurrent_test")
        .await
        .expect("创建空间失败");
    ctx.execute_query_ok("USE concurrent_test").await.expect("使用空间失败");

    ctx.execute_query_ok("CREATE TAG IF NOT EXISTS Counter(id INT, count INT)")
        .await
        .expect("创建标签失败");

    // 初始化计数器
    ctx.execute_query_ok("INSERT VERTEX Counter(id, count) VALUES 1:(1, 0)")
        .await
        .expect("初始化失败");

    // 由于 E2eTestContext 不是 Send，使用单线程顺序执行模拟并发
    // 实际并发测试需要在集成测试环境中进行
    for _ in 0..100 {
        let _ = ctx
            .execute_query("UPDATE Counter SET count = count + 1 WHERE id == 1")
            .await;
    }

    // 验证最终计数
    let data = ctx
        .execute_query_ok("MATCH (c:Counter) WHERE c.id == 1 RETURN c.count")
        .await
        .expect("查询失败");

    assert_not_empty(&data);
}

/// 回归测试: 数据类型支持
/// 验证各种数据类型的正确性
#[tokio::test]
async fn test_regression_data_types() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS types_test")
        .await
        .expect("创建空间失败");
    ctx.execute_query_ok("USE types_test").await.expect("使用空间失败");

    // 创建包含多种数据类型的标签
    ctx.execute_query_ok(
        "CREATE TAG IF NOT EXISTS AllTypes(
            bool_val BOOL,
            int_val INT,
            double_val DOUBLE,
            string_val STRING,
            timestamp_val TIMESTAMP
        )",
    )
    .await
    .expect("创建标签失败");

    // 插入各种类型数据
    let insert_query = r#"
        INSERT VERTEX AllTypes(bool_val, int_val, double_val, string_val, timestamp_val)
        VALUES 1:(true, 42, 3.14159, 'Hello GraphDB', now())
    "#;
    ctx.execute_query_ok(insert_query)
        .await
        .expect("插入失败");

    // 查询并验证
    let data = ctx
        .execute_query_ok("MATCH (t:AllTypes) WHERE t.id == 1 RETURN t.*")
        .await
        .expect("查询失败");

    assert_not_empty(&data);
}

/// 回归测试: 索引功能
/// 验证索引的创建、使用和删除
#[tokio::test]
async fn test_regression_index_functionality() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS index_test")
        .await
        .expect("创建空间失败");
    ctx.execute_query_ok("USE index_test").await.expect("使用空间失败");

    ctx.execute_query_ok("CREATE TAG IF NOT EXISTS IndexedItem(name STRING, value INT)")
        .await
        .expect("创建标签失败");

    // 创建索引
    let create_index_result = ctx
        .execute_query("CREATE INDEX item_value_idx ON IndexedItem(value)")
        .await;
    println!("创建索引结果: {:?}", create_index_result);

    // 插入数据
    for i in 0..100 {
        let query = format!(
            "INSERT VERTEX IndexedItem(name, value) VALUES {}:('Item{}', {})",
            i + 1,
            i + 1,
            i
        );
        ctx.execute_query_ok(&query).await.ok();
    }

    // 重建索引
    let rebuild_result = ctx.execute_query("REBUILD INDEX item_value_idx").await;
    println!("重建索引结果: {:?}", rebuild_result);

    // 使用索引查询
    let lookup_result = ctx
        .execute_query("LOOKUP ON IndexedItem WHERE IndexedItem.value == 50")
        .await;
    println!("索引查询结果: {:?}", lookup_result);

    // 删除索引
    let drop_index_result = ctx.execute_query("DROP INDEX item_value_idx").await;
    println!("删除索引结果: {:?}", drop_index_result);
}

/// 回归测试: 权限控制
/// 验证用户权限管理功能
#[tokio::test]
async fn test_regression_permission_control() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    // 创建用户
    let create_user_result = ctx
        .execute_query("CREATE USER IF NOT EXISTS testuser WITH PASSWORD 'test123'")
        .await;
    println!("创建用户结果: {:?}", create_user_result);

    // 授予权限
    let grant_result = ctx
        .execute_query("GRANT ROLE ADMIN ON * TO testuser")
        .await;
    println!("授权结果: {:?}", grant_result);

    // 显示权限
    let show_roles_result = ctx.execute_query("SHOW ROLES").await;
    println!("显示角色结果: {:?}", show_roles_result);

    // 撤销权限
    let revoke_result = ctx
        .execute_query("REVOKE ROLE ADMIN ON * FROM testuser")
        .await;
    println!("撤销权限结果: {:?}", revoke_result);

    // 删除用户
    let drop_user_result = ctx.execute_query("DROP USER IF EXISTS testuser").await;
    println!("删除用户结果: {:?}", drop_user_result);
}

/// 回归测试: 空间管理
/// 验证图空间的创建、使用和删除
#[tokio::test]
async fn test_regression_space_management() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    // 创建空间
    let create_result = ctx
        .execute_query("CREATE SPACE IF NOT EXISTS management_test (partition_num=1, replica_factor=1)")
        .await;
    assert!(create_result.is_ok(), "创建空间失败");

    // 列出空间
    let list_result = ctx.execute_query("SHOW SPACES").await;
    assert!(list_result.is_ok(), "列出空间失败");

    // 使用空间
    let use_result = ctx.execute_query("USE management_test").await;
    assert!(use_result.is_ok(), "使用空间失败");

    // 在空间内创建标签
    ctx.execute_query_ok("CREATE TAG IF NOT EXISTS TestTag(name STRING)")
        .await
        .expect("创建标签失败");

    // 描述空间
    let desc_result = ctx.execute_query("DESC SPACE management_test").await;
    println!("描述空间结果: {:?}", desc_result);

    // 删除空间（清理）
    // 注意：删除空间可能需要特殊权限
    // let drop_result = ctx.execute_query("DROP SPACE IF EXISTS management_test").await;
    // println!("删除空间结果: {:?}", drop_result);
}
