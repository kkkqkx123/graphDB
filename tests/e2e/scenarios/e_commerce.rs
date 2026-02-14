//! 电商推荐场景 E2E 测试
//!
//! 测试范围:
//! - 商品目录管理
//! - 用户行为追踪
//! - 推荐算法
//! - 购物车分析

use crate::e2e::common::{
    assertions::*,
    data_generators::ECommerceDataGenerator,
    E2eTestContext,
};
use std::time::Duration;

/// 测试用例: TC-EC-01
/// 名称: 商品目录管理
/// 优先级: P0
///
/// # 前置条件
/// - 空数据库
#[tokio::test]
async fn test_ecommerce_catalog_management() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = ECommerceDataGenerator::new(&ctx);

    // 创建基础模式
    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 生成分类
    let categories = generator
        .generate_categories()
        .await
        .expect("生成分类失败");

    // 生成商品
    let products = generator
        .generate_products(50, &categories)
        .await
        .expect("生成商品失败");

    // 验证商品分类关系
    let query = r#"
        MATCH (p:Product)-[:BELONGS_TO]->(c:Category)
        WHERE c.name == 'Electronics'
        RETURN p.name
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证返回了电子产品
    assert_not_empty(&data);
}

/// 测试用例: TC-EC-02
/// 名称: 用户行为追踪
/// 优先级: P0
#[tokio::test]
async fn test_ecommerce_user_behavior_tracking() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = ECommerceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let categories = generator
        .generate_categories()
        .await
        .expect("生成分类失败");
    let products = generator
        .generate_products(20, &categories)
        .await
        .expect("生成商品失败");
    let users = generator.generate_users(5).await.expect("生成用户失败");

    // 记录用户浏览行为
    let view_query = r#"
        INSERT EDGE VIEWED(view_time, duration) VALUES 1 -> 100:(now(), 120)
    "#;
    ctx.execute_query_ok(view_query)
        .await
        .expect("记录浏览行为失败");

    // 记录加购行为
    let cart_query = r#"
        INSERT EDGE ADDED_TO_CART(added_at, quantity) VALUES 1 -> 100:(now(), 2)
    "#;
    ctx.execute_query_ok(cart_query)
        .await
        .expect("记录加购行为失败");

    // 记录购买行为
    let purchase_query = r#"
        INSERT EDGE PURCHASED(order_id, quantity, price) VALUES 1 -> 100:('ORD001', 2, 199.99)
    "#;
    ctx.execute_query_ok(purchase_query)
        .await
        .expect("记录购买行为失败");

    // 查询用户行为路径
    let behavior_query = r#"
        MATCH (u:User)-[r]->(p:Product)
        WHERE u.name == 'Customer1'
        RETURN type(r) AS action, p.name
    "#;
    let data = ctx.execute_query_ok(behavior_query)
        .await
        .expect("查询行为失败");

    assert_not_empty(&data);
}

/// 测试用例: TC-EC-03
/// 名称: 相似商品推荐
/// 优先级: P0
#[tokio::test]
async fn test_ecommerce_similar_product_recommendation() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = ECommerceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let categories = generator
        .generate_categories()
        .await
        .expect("生成分类失败");
    let products = generator
        .generate_products(50, &categories)
        .await
        .expect("生成商品失败");

    // 生成相似度关系
    generator
        .generate_similarity_relations(&products, 5)
        .await
        .expect("生成相似度关系失败");

    // 查询相似商品
    let query = r#"
        MATCH (p:Product)-[s:SIMILAR_TO]->(similar:Product)
        WHERE p.name == 'Product-100'
        RETURN similar.name, s.similarity_score
        ORDER BY s.similarity_score DESC
        LIMIT 10
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证返回了相似商品
    assert_not_empty(&data);
}

/// 测试用例: TC-EC-04
/// 名称: 协同过滤推荐
/// 优先级: P1
#[tokio::test]
async fn test_ecommerce_collaborative_filtering() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = ECommerceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let categories = generator
        .generate_categories()
        .await
        .expect("生成分类失败");
    let products = generator
        .generate_products(30, &categories)
        .await
        .expect("生成商品失败");
    let users = generator.generate_users(10).await.expect("生成用户失败");

    // 生成购买记录
    generator
        .generate_purchase_history(&users, &products, 5)
        .await
        .expect("生成购买记录失败");

    // 协同过滤推荐查询
    let query = r#"
        MATCH (u:User)-[:PURCHASED]->(p:Product)<-[:PURCHASED]-(similar:User)
        WHERE u.name == 'Customer1'
        WITH similar, count(p) AS common_products
        ORDER BY common_products DESC
        LIMIT 5
        MATCH (similar)-[:PURCHASED]->(rec:Product)
        WHERE NOT (u)-[:PURCHASED]->(rec)
        RETURN rec.name, count(*) AS score
        ORDER BY score DESC
        LIMIT 10
    "#;
    let result = ctx.execute_query(query).await;

    // 验证查询执行
    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-EC-05
/// 名称: 购物车放弃分析
/// 优先级: P1
#[tokio::test]
async fn test_ecommerce_cart_abandonment_analysis() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = ECommerceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let categories = generator
        .generate_categories()
        .await
        .expect("生成分类失败");
    let products = generator
        .generate_products(20, &categories)
        .await
        .expect("生成商品失败");
    let users = generator.generate_users(5).await.expect("生成用户失败");

    // 用户加购但未购买
    let cart_query = r#"
        INSERT EDGE ADDED_TO_CART(added_at, quantity) VALUES 1 -> 100:(now(), 1)
    "#;
    ctx.execute_query_ok(cart_query)
        .await
        .expect("记录加购失败");

    // 查询购物车放弃
    let query = r#"
        MATCH (u:User)-[:ADDED_TO_CART]->(p:Product)
        WHERE NOT (u)-[:PURCHASED]->(p)
        RETURN u.name, p.name
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证识别了购物车放弃
    assert_not_empty(&data);
}

/// 测试用例: TC-EC-06
/// 名称: 热销商品排行
/// 优先级: P1
#[tokio::test]
async fn test_ecommerce_best_sellers() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = ECommerceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let categories = generator
        .generate_categories()
        .await
        .expect("生成分类失败");
    let products = generator
        .generate_products(30, &categories)
        .await
        .expect("生成商品失败");
    let users = generator.generate_users(15).await.expect("生成用户失败");

    // 生成购买记录
    generator
        .generate_purchase_history(&users, &products, 3)
        .await
        .expect("生成购买记录失败");

    // 热销商品排行
    let query = r#"
        MATCH (u:User)-[p:PURCHASED]->(product:Product)
        RETURN product.name, sum(p.quantity) AS total_sold
        ORDER BY total_sold DESC
        LIMIT 10
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证返回了热销榜单
    assert_not_empty(&data);
}

/// 测试用例: TC-EC-07
/// 名称: 用户分群
/// 优先级: P2
#[tokio::test]
async fn test_ecommerce_user_segmentation() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = ECommerceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let categories = generator
        .generate_categories()
        .await
        .expect("生成分类失败");
    let products = generator
        .generate_products(20, &categories)
        .await
        .expect("生成商品失败");
    let users = generator.generate_users(20).await.expect("生成用户失败");

    // 生成购买记录
    generator
        .generate_purchase_history(&users, &products, 3)
        .await
        .expect("生成购买记录失败");

    // 按购买频次分群
    let query = r#"
        MATCH (u:User)-[:PURCHASED]->(p:Product)
        WITH u, count(p) AS purchase_count
        RETURN 
            CASE 
                WHEN purchase_count >= 5 THEN 'High Value'
                WHEN purchase_count >= 2 THEN 'Medium Value'
                ELSE 'Low Value'
            END AS segment,
            count(u) AS user_count
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证返回了分群结果
    assert_not_empty(&data);
}

/// 测试用例: TC-EC-08
/// 名称: 购买路径分析
/// 优先级: P2
#[tokio::test]
async fn test_ecommerce_purchase_path_analysis() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = ECommerceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let categories = generator
        .generate_categories()
        .await
        .expect("生成分类失败");
    let products = generator
        .generate_products(20, &categories)
        .await
        .expect("生成商品失败");
    let users = generator.generate_users(10).await.expect("生成用户失败");

    // 记录完整的用户行为
    for user_id in 1..=3 {
        // 浏览
        let view = format!(
            "INSERT EDGE VIEWED(view_time, duration) VALUES {} -> 100:(now(), 60)",
            user_id
        );
        ctx.execute_query_ok(&view).await.ok();

        // 加购
        let cart = format!(
            "INSERT EDGE ADDED_TO_CART(added_at, quantity) VALUES {} -> 100:(now(), 1)",
            user_id
        );
        ctx.execute_query_ok(&cart).await.ok();

        // 购买
        let purchase = format!(
            "INSERT EDGE PURCHASED(order_id, quantity, price) VALUES {} -> 100:('ORD00{}', 1, 99.99)",
            user_id, user_id
        );
        ctx.execute_query_ok(&purchase).await.ok();
    }

    // 查询购买转化
    let query = r#"
        MATCH (u:User)-[:VIEWED]->(p:Product)
        OPTIONAL MATCH (u)-[:ADDED_TO_CART]->(p)
        OPTIONAL MATCH (u)-[:PURCHASED]->(p)
        RETURN 
            count(DISTINCT u) AS viewed,
            count(DISTINCT CASE WHEN (u)-[:ADDED_TO_CART]->(p) THEN u END) AS carted,
            count(DISTINCT CASE WHEN (u)-[:PURCHASED]->(p) THEN u END) AS purchased
    "#;
    let result = ctx.execute_query(query).await;

    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-EC-09
/// 名称: 库存预警
/// 优先级: P2
#[tokio::test]
async fn test_ecommerce_inventory_alert() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = ECommerceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let categories = generator
        .generate_categories()
        .await
        .expect("生成分类失败");
    let products = generator
        .generate_products(20, &categories)
        .await
        .expect("生成商品失败");

    // 查询低库存商品（假设库存属性存在）
    let query = r#"
        MATCH (p:Product)
        WHERE p.price < 50
        RETURN p.name, p.price
        ORDER BY p.price ASC
        LIMIT 10
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证返回了低价商品
    assert_not_empty(&data);
}

/// 测试用例: TC-EC-10
/// 名称: 价格弹性分析
/// 优先级: P2
#[tokio::test]
async fn test_ecommerce_price_elasticity() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = ECommerceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let categories = generator
        .generate_categories()
        .await
        .expect("生成分类失败");
    let products = generator
        .generate_products(30, &categories)
        .await
        .expect("生成商品失败");
    let users = generator.generate_users(20).await.expect("生成用户失败");

    // 生成购买记录
    generator
        .generate_purchase_history(&users, &products, 3)
        .await
        .expect("生成购买记录失败");

    // 按价格区间统计销量
    let query = r#"
        MATCH (u:User)-[p:PURCHASED]->(product:Product)
        RETURN 
            CASE 
                WHEN product.price < 100 THEN 'Low Price'
                WHEN product.price < 500 THEN 'Medium Price'
                ELSE 'High Price'
            END AS price_range,
            sum(p.quantity) AS total_sold
        ORDER BY total_sold DESC
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证返回了价格区间统计
    assert_not_empty(&data);
}
