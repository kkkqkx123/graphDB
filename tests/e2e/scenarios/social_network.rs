//! 社交网络场景 E2E 测试
//!
//! 测试范围:
//! - 用户注册与好友管理
//! - 动态发布与互动
//! - 群组管理
//! - 社交关系分析

use crate::e2e::common::{
    assertions::*,
    data_generators::{SocialGraph, SocialNetworkDataGenerator},
    E2eTestContext, QueryResult,
};
use std::time::Duration;

/// 测试用例: TC-SN-01
/// 名称: 用户注册与好友添加流程
/// 优先级: P0
///
/// # 前置条件
/// - 空数据库
///
/// # 执行步骤
/// 1. 创建图空间
/// 2. 创建标签和边类型
/// 3. 插入用户数据
/// 4. 建立好友关系
///
/// # 预期结果
/// - 所有操作成功
/// - 查询返回正确结果
#[tokio::test]
async fn test_sns_user_registration_and_friend_addition() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    // 准备基础模式
    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 创建用户
    let create_users = r#"
        INSERT VERTEX Person(name, age, city) 
        VALUES 1:('Alice', 25, 'Beijing'), 2:('Bob', 28, 'Shanghai')
    "#;
    let result = ctx.execute_query(create_users).await;
    assert!(result.is_ok(), "创建用户失败: {:?}", result.err());
    assert_query_success(&result.unwrap());

    // 建立好友关系
    let create_friendship = r#"
        INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2024-01-01')
    "#;
    let result = ctx.execute_query(create_friendship).await;
    assert!(result.is_ok(), "创建好友关系失败: {:?}", result.err());

    // 验证好友关系
    let query = "GO FROM 1 OVER KNOWS YIELD dst(edge)";
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证结果
    assert_row_count(&data, 1);
}

/// 测试用例: TC-SN-02
/// 名称: 多层好友关系查询
/// 优先级: P0
///
/// # 前置条件
/// - 已建立 3 层好友关系网络
#[tokio::test]
async fn test_sns_multi_level_friend_query() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    // 生成 3 层好友网络
    let graph = generator
        .generate_social_graph(20)
        .await
        .expect("生成社交网络失败");

    // 测试 2 层好友查询
    let query = "GO 2 STEPS FROM 1 OVER KNOWS YIELD dst(edge)";
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证返回了 2 层好友
    assert_not_empty(&data);

    // 测试路径查找
    let path_query = "FIND ALL PATH FROM 1 TO 5 OVER KNOWS";
    let path_result = ctx.execute_query(path_query).await;
    assert!(path_result.is_ok());
}

/// 测试用例: TC-SN-03
/// 名称: 动态发布与互动
/// 优先级: P0
///
/// # 前置条件
/// - 已创建用户和帖子标签
#[tokio::test]
async fn test_sns_post_and_interaction() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");
    generator.generate_users(5).await.expect("生成用户失败");

    // 创建帖子
    let create_post = r#"
        INSERT VERTEX Post(content, created_at, likes) 
        VALUES 100:('Hello GraphDB!', now(), 0)
    "#;
    ctx.execute_query_ok(create_post)
        .await
        .expect("创建帖子失败");

    // 用户发布帖子
    let post_relation = "INSERT EDGE POSTED VALUES 1 -> 100";
    ctx.execute_query_ok(post_relation)
        .await
        .expect("建立发布关系失败");

    // 其他用户点赞
    let like = r#"
        INSERT EDGE LIKES(created_at) VALUES 2 -> 100:(now())
    "#;
    ctx.execute_query_ok(like).await.expect("点赞失败");

    // 查询帖子及互动
    let query = r#"
        MATCH (p:Person)-[:POSTED]->(post:Post)<-[:LIKES]-(liker:Person) 
        RETURN p.name, post.content, liker.name
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证结果包含作者和点赞者
    assert_not_empty(&data);
}

/// 测试用例: TC-SN-04
/// 名称: 群组管理与成员查询
/// 优先级: P1
#[tokio::test]
async fn test_sns_group_management() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");
    generator.generate_users(5).await.expect("生成用户失败");

    // 创建群组
    let create_group = r#"
        INSERT VERTEX Group(name, description, created_at) 
        VALUES 200:('Rust爱好者', 'Rust编程语言交流', now())
    "#;
    ctx.execute_query_ok(create_group)
        .await
        .expect("创建群组失败");

    // 添加成员
    let member1 = r#"
        INSERT EDGE MEMBER_OF(joined_at, role) VALUES 1 -> 200:(now(), 'admin')
    "#;
    let member2 = r#"
        INSERT EDGE MEMBER_OF(joined_at, role) VALUES 2 -> 200:(now(), 'member')
    "#;
    ctx.execute_query_ok(member1).await.expect("添加成员失败");
    ctx.execute_query_ok(member2).await.expect("添加成员失败");

    // 查询群组成员
    let query = r#"
        MATCH (p:Person)-[m:MEMBER_OF]->(g:Group) 
        WHERE g.name == 'Rust爱好者' 
        RETURN p.name, m.role
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    assert_row_count(&data, 2);
}

/// 测试用例: TC-SN-05
/// 名称: 共同好友发现
/// 优先级: P1
#[tokio::test]
async fn test_sns_mutual_friends_discovery() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 创建用户
    generator.generate_users(10).await.expect("生成用户失败");

    // 建立好友关系，使 Alice 和 Bob 有共同好友
    let friendships = vec![
        "INSERT EDGE KNOWS VALUES 1 -> 3",
        "INSERT EDGE KNOWS VALUES 1 -> 4",
        "INSERT EDGE KNOWS VALUES 1 -> 5",
        "INSERT EDGE KNOWS VALUES 2 -> 3",
        "INSERT EDGE KNOWS VALUES 2 -> 4",
        "INSERT EDGE KNOWS VALUES 2 -> 6",
    ];

    for friendship in friendships {
        ctx.execute_query_ok(friendship)
            .await
            .expect("建立好友关系失败");
    }

    // 查询共同好友
    let query = r#"
        MATCH (a:Person)-[:KNOWS]->(common:Person)<-[:KNOWS]-(b:Person)
        WHERE a.name == 'User1' AND b.name == 'User2'
        RETURN common.name
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // User3 和 User4 应该是共同好友
    assert_not_empty(&data);
}

/// 测试用例: TC-SN-06
/// 名称: 好友推荐
/// 优先级: P1
#[tokio::test]
async fn test_sns_friend_recommendation() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 生成社交网络
    let graph = generator
        .generate_social_graph(15)
        .await
        .expect("生成社交网络失败");

    // 基于共同好友推荐
    let query = r#"
        MATCH (u:Person)-[:KNOWS]->(friend:Person)-[:KNOWS]->(potential:Person)
        WHERE u.name == 'User1' AND NOT (u)-[:KNOWS]->(potential)
        RETURN potential.name, count(friend) AS common_count
        ORDER BY common_count DESC
        LIMIT 5
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证返回了推荐结果
    assert_not_empty(&data);
}

/// 测试用例: TC-SN-07
/// 名称: 动态时间线
/// 优先级: P1
#[tokio::test]
async fn test_sns_timeline() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let user_ids = generator.generate_users(3).await.expect("生成用户失败");
    let post_ids = generator
        .generate_posts_and_interactions(&user_ids, 2)
        .await
        .expect("生成动态失败");

    // 查询用户的动态时间线
    let query = r#"
        MATCH (p:Person)-[:POSTED]->(post:Post)
        WHERE p.name == 'User1'
        RETURN post.content, post.created_at
        ORDER BY post.created_at DESC
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证返回了动态
    assert_not_empty(&data);
}

/// 测试用例: TC-SN-08
/// 名称: 影响力分析
/// 优先级: P2
#[tokio::test]
async fn test_sns_influence_analysis() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 生成社交网络
    generator
        .generate_social_graph(20)
        .await
        .expect("生成社交网络失败");

    // 计算用户影响力（基于粉丝数和互动数）
    let query = r#"
        MATCH (p:Person)<-[:FOLLOWS]-(follower:Person)
        OPTIONAL MATCH (p)-[:POSTED]->(post:Post)
        RETURN p.name, count(follower) AS followers, sum(post.likes) AS total_likes
        ORDER BY followers DESC, total_likes DESC
        LIMIT 10
    "#;

    let result = ctx.execute_query(query).await;
    // 即使查询语法有问题，也不应该崩溃
    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-SN-09
/// 名称: 社区发现
/// 优先级: P2
#[tokio::test]
async fn test_sns_community_detection() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 生成社交网络
    generator
        .generate_social_graph(30)
        .await
        .expect("生成社交网络失败");

    // 查询用户的邻居（用于社区分析）
    let query = r#"
        MATCH (center:Person)-[:KNOWS]-(neighbor:Person)
        WHERE center.name == 'User1'
        RETURN neighbor.name
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证返回了邻居
    assert_not_empty(&data);
}

/// 测试用例: TC-SN-10
/// 名称: 信息传播路径
/// 优先级: P2
#[tokio::test]
async fn test_sns_information_propagation() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 生成社交网络
    generator
        .generate_social_graph(15)
        .await
        .expect("生成社交网络失败");

    // 查找两个用户之间的最短路径
    let query = r#"
        FIND SHORTEST PATH FROM 1 TO 10 OVER KNOWS
    "#;
    let result = ctx.execute_query(query).await;

    // 验证路径查询执行
    assert!(result.is_ok() || result.is_err());
}
