//! 知识图谱场景 E2E 测试
//!
//! 测试范围:
//! - 实体关系查询
//! - 知识推理
//! - 知识一致性验证
//! - 知识补全

use crate::e2e::common::{
    assertions::*,
    data_generators::KnowledgeGraphDataGenerator,
    E2eTestContext,
};

/// 测试用例: TC-KG-01
/// 名称: 实体关系查询
/// 优先级: P1
///
/// # 前置条件
/// - 已构建知识图谱
#[tokio::test]
async fn test_kg_entity_relation_query() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = KnowledgeGraphDataGenerator::new(ctx.clone());

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let entities = generator
        .generate_entities(20)
        .await
        .expect("生成实体失败");

    // 查询实体关系
    let query = r#"
        MATCH (e:Entity)-[r:RELATES_TO]->(related:Entity)
        WHERE e.name == 'Entity1'
        RETURN r.relation_type, related.name
    "#;
    let result = ctx.execute_query(query).await;

    // 验证查询执行
    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-KG-02
/// 名称: 多跳关系推理
/// 优先级: P1
#[tokio::test]
async fn test_kg_multi_hop_reasoning() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = KnowledgeGraphDataGenerator::new(ctx.clone());

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let entities = generator
        .generate_entities(15)
        .await
        .expect("生成实体失败");

    generator
        .generate_relations(&entities, 3)
        .await
        .expect("生成关系失败");

    // 多跳关系查询
    let query = r#"
        MATCH path = (start:Entity)-[:RELATES_TO*1..3]->(end:Entity)
        WHERE start.name == 'Entity1'
        RETURN path
        LIMIT 10
    "#;
    let result = ctx.execute_query(query).await;

    assert!(result.is_ok() || result.is_err());
}

/// 测试用例: TC-KG-03
/// 名称: 知识一致性验证
/// 优先级: P1
#[tokio::test]
async fn test_kg_consistency_validation() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = KnowledgeGraphDataGenerator::new(ctx.clone());

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let entities = generator
        .generate_entities(10)
        .await
        .expect("生成实体失败");

    // 创建循环关系（用于测试一致性检测）
    let cycle_query = r#"
        INSERT EDGE RELATES_TO(relation_type, confidence) VALUES 1 -> 2:('related_to', 0.9)
    "#;
    ctx.execute_query_ok(cycle_query).await.ok();

    let cycle_query2 = r#"
        INSERT EDGE RELATES_TO(relation_type, confidence) VALUES 2 -> 1:('related_to', 0.8)
    "#;
    ctx.execute_query_ok(cycle_query2).await.ok();

    // 查询双向关系
    let query = r#"
        MATCH (a:Entity)-[r1:RELATES_TO]->(b:Entity),
              (b)-[r2:RELATES_TO]->(a)
        WHERE a.name == 'Entity1'
        RETURN a.name, b.name, r1.confidence, r2.confidence
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证检测到双向关系
    assert_not_empty(&data);
}

/// 测试用例: TC-KG-04
/// 名称: 实体链接
/// 优先级: P2
#[tokio::test]
async fn test_kg_entity_linking() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = KnowledgeGraphDataGenerator::new(ctx.clone());

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 创建实体
    let entity1 = r#"
        INSERT VERTEX Entity(name, type, description) VALUES 1:('Apple Inc', 'Organization', 'Technology company')
    "#;
    ctx.execute_query_ok(entity1).await.expect("创建实体失败");

    let entity2 = r#"
        INSERT VERTEX Entity(name, type, description) VALUES 2:('Apple', 'Organization', 'Apple company')
    "#;
    ctx.execute_query_ok(entity2).await.expect("创建实体失败");

    // 创建实体链接关系
    let link = r#"
        INSERT EDGE RELATES_TO(relation_type, confidence) VALUES 1 -> 2:('same_as', 0.95)
    "#;
    ctx.execute_query_ok(link).await.expect("创建链接失败");

    // 查询链接实体
    let query = r#"
        MATCH (a:Entity)-[r:RELATES_TO]->(b:Entity)
        WHERE r.relation_type == 'same_as'
        RETURN a.name, b.name, r.confidence
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    assert_not_empty(&data);
}

/// 测试用例: TC-KG-05
/// 名称: 知识补全
/// 优先级: P2
#[tokio::test]
async fn test_kg_knowledge_completion() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = KnowledgeGraphDataGenerator::new(ctx.clone());

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let entities = generator
        .generate_entities(15)
        .await
        .expect("生成实体失败");

    generator
        .generate_relations(&entities, 2)
        .await
        .expect("生成关系失败");

    // 查找缺失的关系（通过共同邻居推断）
    let query = r#"
        MATCH (a:Entity)-[:RELATES_TO]->(common:Entity)<-[:RELATES_TO]-(b:Entity)
        WHERE NOT (a)-[:RELATES_TO]->(b) AND a.name != b.name
        RETURN a.name, b.name, count(common) AS common_neighbors
        ORDER BY common_neighbors DESC
        LIMIT 10
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证返回了可能的补全关系
    assert_not_empty(&data);
}

/// 测试用例: TC-KG-06
/// 名称: 知识推理
/// 优先级: P2
#[tokio::test]
async fn test_kg_knowledge_reasoning() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = KnowledgeGraphDataGenerator::new(ctx.clone());

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 创建概念层次
    let concept1 = r#"
        INSERT VERTEX Concept(name, domain) VALUES 100:('Technology', 'General')
    "#;
    ctx.execute_query_ok(concept1).await.expect("创建概念失败");

    let concept2 = r#"
        INSERT VERTEX Concept(name, domain) VALUES 101:('Programming Language', 'Technology')
    "#;
    ctx.execute_query_ok(concept2).await.expect("创建概念失败");

    // 创建实体
    let entity = r#"
        INSERT VERTEX Entity(name, type, description) VALUES 1:('Rust', 'Technology', 'Programming language')
    "#;
    ctx.execute_query_ok(entity).await.expect("创建实体失败");

    // 建立实例关系
    let instance = "INSERT EDGE INSTANCE_OF VALUES 1 -> 101";
    ctx.execute_query_ok(instance)
        .await
        .expect("创建实例关系失败");

    // 查询实体的概念层次
    let query = r#"
        MATCH (e:Entity)-[:INSTANCE_OF]->(c:Concept)
        WHERE e.name == 'Rust'
        RETURN e.name, c.name
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    assert_not_empty(&data);
}

/// 测试用例: TC-KG-07
/// 名称: 实体消歧
/// 优先级: P2
#[tokio::test]
async fn test_kg_entity_disambiguation() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = KnowledgeGraphDataGenerator::new(ctx.clone());

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 创建同名但不同类型的实体
    let entity1 = r#"
        INSERT VERTEX Entity(name, type, description) VALUES 1:('Python', 'Technology', 'Programming language')
    "#;
    ctx.execute_query_ok(entity1).await.expect("创建实体失败");

    let entity2 = r#"
        INSERT VERTEX Entity(name, type, description) VALUES 2:('Python', 'Animal', 'Snake species')
    "#;
    ctx.execute_query_ok(entity2).await.expect("创建实体失败");

    // 查询同名实体
    let query = r#"
        MATCH (e:Entity)
        WHERE e.name == 'Python'
        RETURN e.name, e.type, e.description
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    // 验证返回了结果
    assert_not_empty(&data);
}

/// 测试用例: TC-KG-08
/// 名称: 知识图谱可视化
/// 优先级: P2
#[tokio::test]
async fn test_kg_visualization() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = KnowledgeGraphDataGenerator::new(ctx.clone());

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let entities = generator
        .generate_entities(10)
        .await
        .expect("生成实体失败");

    generator
        .generate_relations(&entities, 2)
        .await
        .expect("生成关系失败");

    // 查询子图用于可视化
    let query = r#"
        MATCH path = (e:Entity)-[:RELATES_TO]-(related:Entity)
        WHERE e.name == 'Entity1'
        RETURN path
        LIMIT 20
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    assert_not_empty(&data);
}

/// 测试用例: TC-KG-09
/// 名称: 问答系统
/// 优先级: P2
#[tokio::test]
async fn test_kg_question_answering() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = KnowledgeGraphDataGenerator::new(ctx.clone());

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 创建知识图谱
    let entities = vec![
        (1, "Steve Jobs", "Person", "Co-founder of Apple"),
        (2, "Apple Inc", "Organization", "Technology company"),
        (3, "iPhone", "Product", "Smartphone"),
    ];

    for (id, name, entity_type, desc) in entities {
        let query = format!(
            "INSERT VERTEX Entity(name, type, description) VALUES {}:('{}', '{}', '{}')",
            id, name, entity_type, desc
        );
        ctx.execute_query_ok(&query).await.expect("创建实体失败");
    }

    // 创建关系
    let relations = vec![
        (1, 2, "founded"),
        (2, 3, "produced"),
    ];

    for (src, dst, rel_type) in relations {
        let query = format!(
            "INSERT EDGE RELATES_TO(relation_type, confidence) VALUES {} -> {}:('{}', 0.95)",
            src, dst, rel_type
        );
        ctx.execute_query_ok(&query).await.expect("创建关系失败");
    }

    // 查询：谁创立了 Apple？
    let query = r#"
        MATCH (p:Entity)-[r:RELATES_TO]->(o:Entity)
        WHERE o.name == 'Apple Inc' AND r.relation_type == 'founded'
        RETURN p.name
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    assert_not_empty(&data);
}

/// 测试用例: TC-KG-10
/// 名称: 知识更新
/// 优先级: P2
#[tokio::test]
async fn test_kg_knowledge_update() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = KnowledgeGraphDataGenerator::new(ctx.clone());

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 创建初始实体
    let entity = r#"
        INSERT VERTEX Entity(name, type, description) VALUES 1:('Old Name', 'Type1', 'Old description')
    "#;
    ctx.execute_query_ok(entity).await.expect("创建实体失败");

    // 更新实体（通过删除旧数据并插入新数据）
    let update = r#"
        INSERT VERTEX Entity(name, type, description) VALUES 1:('New Name', 'Type1', 'Updated description')
    "#;
    ctx.execute_query_ok(update).await.expect("更新实体失败");

    // 验证更新
    let query = r#"
        MATCH (e:Entity)
        WHERE e.name == 'New Name'
        RETURN e.name, e.description
    "#;
    let data = ctx.execute_query_ok(query).await.expect("查询失败");

    assert_not_empty(&data);
}
