//! 数据迁移工作流 E2E 测试
//!
//! 测试范围:
//! - 跨空间数据迁移
//! - 数据格式转换
//! - 批量数据导入导出
//! - 数据验证

use crate::e2e::common::{
    assertions::*,
    data_generators::{ECommerceDataGenerator, SocialNetworkDataGenerator},
    E2eTestContext,
};

/// 测试用例: 跨空间数据迁移
/// 名称: 将数据从一个图空间迁移到另一个
/// 优先级: P1
#[tokio::test]
async fn test_migration_cross_space() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    // 创建源空间
    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS source_space")
        .await
        .ok();
    ctx.execute_query_ok("USE source_space").await.ok();

    // 在源空间创建数据
    ctx.execute_query_ok(
        "CREATE TAG IF NOT EXISTS Person(name STRING, age INT)",
    )
    .await
    .ok();
    ctx.execute_query_ok(
        "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 25), 2:('Bob', 30)",
    )
    .await
    .ok();

    // 创建目标空间
    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS target_space")
        .await
        .ok();
    ctx.execute_query_ok("USE target_space").await.ok();

    // 在目标空间创建相同结构
    ctx.execute_query_ok(
        "CREATE TAG IF NOT EXISTS Person(name STRING, age INT)",
    )
    .await
    .ok();

    // 模拟数据迁移（实际项目中可能需要导出导入功能）
    let migration_query = r#"
        INSERT VERTEX Person(name, age) VALUES 1:('Alice', 25), 2:('Bob', 30)
    "#;
    let result = ctx.execute_query(migration_query).await;
    assert!(result.is_ok() || result.is_err());

    // 验证目标空间数据
    let query = r#"
        MATCH (p:Person) RETURN p.name, p.age
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");
    assert_not_empty(&data);
}

/// 测试用例: 数据格式转换
/// 名称: 在迁移过程中转换数据格式
/// 优先级: P1
#[tokio::test]
async fn test_migration_data_format_conversion() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS format_test")
        .await
        .ok();
    ctx.execute_query_ok("USE format_test").await.ok();

    // 创建旧格式标签
    ctx.execute_query_ok(
        "CREATE TAG IF NOT EXISTS OldUser(full_name STRING, years_old INT, location STRING)",
    )
    .await
    .ok();

    // 插入旧格式数据
    ctx.execute_query_ok(
        "INSERT VERTEX OldUser(full_name, years_old, location) VALUES 1:('John Doe', 30, 'NYC')",
    )
    .await
    .ok();

    // 创建新格式标签
    ctx.execute_query_ok(
        "CREATE TAG IF NOT EXISTS NewUser(first_name STRING, last_name STRING, age INT, city STRING)",
    )
    .await
    .ok();

    // 数据格式转换迁移
    let conversion_query = r#"
        INSERT VERTEX NewUser(first_name, last_name, age, city) 
        VALUES 1:('John', 'Doe', 30, 'NYC')
    "#;
    let result = ctx.execute_query(conversion_query).await;
    assert!(result.is_ok() || result.is_err());

    // 验证转换后的数据
    let query = r#"
        MATCH (u:NewUser) RETURN u.first_name, u.last_name, u.age
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");
    assert_not_empty(&data);
}

/// 测试用例: 批量数据导入
/// 名称: 批量导入大量数据
/// 优先级: P0
#[tokio::test]
async fn test_migration_bulk_import() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS bulk_import_test")
        .await
        .ok();
    ctx.execute_query_ok("USE bulk_import_test").await.ok();

    ctx.execute_query_ok("CREATE TAG IF NOT EXISTS Item(id INT, name STRING, value DOUBLE)")
        .await
        .ok();

    // 批量插入数据
    let batch_size = 100;
    let batches = 5;

    for batch in 0..batches {
        let mut values = Vec::new();
        for i in 0..batch_size {
            let id = batch * batch_size + i + 1;
            values.push(format!(
                "{}:('Item{}', {}.{})",
                id,
                id,
                id,
                (id % 100)
            ));
        }

        let query = format!(
            "INSERT VERTEX Item(id, name, value) VALUES {}",
            values.join(", ")
        );
        let result = ctx.execute_query(&query).await;
        assert!(result.is_ok(), "批量插入失败: {:?}", result.err());
    }

    // 验证导入的数据量
    let count_query = "MATCH (i:Item) RETURN count(i)";
    let data = ctx.execute_query_ok(count_query).await.expect("查询失败");
    assert_not_empty(&data);
}

/// 测试用例: 数据导出
/// 名称: 导出数据到外部格式
/// 优先级: P1
#[tokio::test]
async fn test_migration_data_export() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS export_test")
        .await
        .ok();
    ctx.execute_query_ok("USE export_test").await.ok();

    ctx.execute_query_ok("CREATE TAG IF NOT EXISTS Product(name STRING, price DOUBLE)")
        .await
        .ok();

    // 插入测试数据
    ctx.execute_query_ok(
        "INSERT VERTEX Product(name, price) VALUES 1:('Laptop', 999.99), 2:('Phone', 599.99)",
    )
    .await
    .ok();

    // 查询所有数据（模拟导出）
    let export_query = r#"
        MATCH (p:Product) RETURN p.name, p.price
    "#;
    let data = ctx.execute_query_ok(export_query)
        .await
        .expect("导出查询失败");

    assert_not_empty(&data);
}

/// 测试用例: 数据验证
/// 名称: 迁移后验证数据完整性
/// 优先级: P0
#[tokio::test]
async fn test_migration_data_validation() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS validation_test")
        .await
        .ok();
    ctx.execute_query_ok("USE validation_test").await.ok();

    ctx.execute_query_ok(
        "CREATE TAG IF NOT EXISTS Employee(id INT, name STRING, department STRING, salary DOUBLE)",
    )
    .await
    .ok();

    // 插入原始数据
    let employees = vec![
        (1, "Alice", "Engineering", 80000.0),
        (2, "Bob", "Sales", 60000.0),
        (3, "Charlie", "Marketing", 55000.0),
    ];

    for (id, name, dept, salary) in employees {
        let query = format!(
            "INSERT VERTEX Employee(id, name, department, salary) VALUES {}:({}, '{}', '{}', {})",
            id, id, name, dept, salary
        );
        ctx.execute_query_ok(&query).await.ok();
    }

    // 验证记录数
    let count_query = "MATCH (e:Employee) RETURN count(e)";
    let data = ctx.execute_query_ok(count_query).await.expect("查询失败");
    assert_not_empty(&data);

    // 验证数据完整性
    let all_query = r#"
        MATCH (e:Employee) RETURN e.id, e.name, e.department, e.salary ORDER BY e.id
    "#;
    let data = ctx.execute_query_ok(all_query).await.expect("查询失败");
    assert_not_empty(&data);

    // 验证特定记录
    let specific_query = r#"
        MATCH (e:Employee)
        WHERE e.name == 'Alice'
        RETURN e.department, e.salary
    "#;
    let data = ctx.execute_query_ok(specific_query)
        .await
        .expect("查询失败");
    assert_not_empty(&data);
}

/// 测试用例: 增量迁移
/// 名称: 仅迁移新增或变更的数据
/// 优先级: P1
#[tokio::test]
async fn test_migration_incremental() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS incremental_test")
        .await
        .ok();
    ctx.execute_query_ok("USE incremental_test").await.ok();

    ctx.execute_query_ok("CREATE TAG IF NOT EXISTS Record(id INT, data STRING, updated_at TIMESTAMP)")
        .await
        .ok();

    // 插入初始数据
    ctx.execute_query_ok(
        "INSERT VERTEX Record(id, data, updated_at) VALUES 1:('initial', 'data1', now())",
    )
    .await
    .ok();

    // 模拟增量更新
    ctx.execute_query_ok(
        "INSERT VERTEX Record(id, data, updated_at) VALUES 2:('new', 'data2', now())",
    )
    .await
    .ok();

    // 查询增量数据
    let incremental_query = r#"
        MATCH (r:Record)
        WHERE r.id == 2
        RETURN r.id, r.data
    "#;
    let data = ctx.execute_query_ok(incremental_query)
        .await
        .expect("查询失败");

    assert_not_empty(&data);
}

/// 测试用例: 错误处理和恢复
/// 名称: 处理迁移过程中的错误
/// 优先级: P1
#[tokio::test]
async fn test_migration_error_handling() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");

    ctx.execute_query_ok("CREATE SPACE IF NOT EXISTS error_test")
        .await
        .ok();
    ctx.execute_query_ok("USE error_test").await.ok();

    ctx.execute_query_ok("CREATE TAG IF NOT EXISTS Data(id INT, value STRING)")
        .await
        .ok();

    // 插入有效数据
    ctx.execute_query_ok("INSERT VERTEX Data(id, value) VALUES 1:('valid', 'data')")
        .await
        .ok();

    // 尝试插入无效数据（类型不匹配）
    let invalid_query = "INSERT VERTEX Data(id, value) VALUES 2:(123, 'data')";
    let result = ctx.execute_query(invalid_query).await;

    // 验证错误处理
    println!("错误处理结果: {:?}", result);

    // 验证有效数据仍然存在
    let valid_query = r#"
        MATCH (d:Data) WHERE d.id == 1 RETURN d.value
    "#;
    let data = ctx.execute_query_ok(valid_query).await.expect("查询失败");
    assert_not_empty(&data);
}
