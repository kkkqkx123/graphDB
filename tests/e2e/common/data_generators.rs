//! E2E 测试数据生成器
//!
//! 提供各种测试场景的数据生成功能

use super::*;
use std::collections::HashMap;

/// 社交网络数据生成器
pub struct SocialNetworkDataGenerator {
    ctx: Arc<E2eTestContext>,
}

impl SocialNetworkDataGenerator {
    pub fn new(ctx: &Arc<E2eTestContext>) -> Self {
        Self { ctx: Arc::clone(ctx) }
    }
    
    /// 生成基础模式
    pub async fn generate_base_schema(&self) -> anyhow::Result<()> {
        let schema_queries = vec![
            "CREATE SPACE IF NOT EXISTS social_network",
            "USE social_network",
            "CREATE TAG IF NOT EXISTS Person(name: STRING, age: INT, city: STRING, created_at: TIMESTAMP)",
            "CREATE TAG IF NOT EXISTS Post(content: STRING, created_at: TIMESTAMP, likes: INT)",
            "CREATE TAG IF NOT EXISTS UserComment(content: STRING, created_at: TIMESTAMP)",
            "CREATE TAG IF NOT EXISTS UserGroup(name: STRING, description: STRING, created_at: TIMESTAMP)",
            "CREATE EDGE IF NOT EXISTS KNOWS(since: DATE, strength: DOUBLE)",
            "CREATE EDGE IF NOT EXISTS FOLLOWS(since: DATE)",
            "CREATE EDGE IF NOT EXISTS POSTED",
            "CREATE EDGE IF NOT EXISTS LIKES(created_at: TIMESTAMP)",
            "CREATE EDGE IF NOT EXISTS COMMENTED",
            "CREATE EDGE IF NOT EXISTS BELONGS_TO",
            "CREATE EDGE IF NOT EXISTS MEMBER_OF(joined_at: TIMESTAMP, member_role: STRING)",
        ];
        
        for query in schema_queries {
            println!("执行查询: {}", query);
            match self.ctx.execute_query_ok(query).await {
                Ok(result) => println!("  结果: {}", result),
                Err(e) => {
                    println!("  错误: {}", e);
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
    
    /// 生成指定数量的用户
    pub async fn generate_users(&self, count: usize) -> anyhow::Result<Vec<i64>> {
        let mut user_ids = Vec::new();
        let cities = vec!["Beijing", "Shanghai", "Guangzhou", "Shenzhen", "Hangzhou"];
        
        for i in 0..count {
            let id = (i + 1) as i64;
            let name = format!("User{}", id);
            let age = 20 + (i % 50) as i32;
            let city = cities[i % cities.len()];
            
            let query = format!(
                "INSERT VERTEX Person(name, age, city, created_at) VALUES {}:('{}', {}, '{}', now())",
                id, name, age, city
            );
            
            self.ctx.execute_query_ok(&query).await?;
            user_ids.push(id);
        }
        
        Ok(user_ids)
    }
    
    /// 生成好友关系网络
    pub async fn generate_friendships(
        &self,
        user_ids: &[i64],
        avg_friends_per_user: usize,
    ) -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();
        use rand::seq::SliceRandom;
        
        for (i, &user_id) in user_ids.iter().enumerate() {
            let num_friends = avg_friends_per_user + (i % 3);
            let mut friends: Vec<i64> = user_ids
                .iter()
                .filter(|&&id| id != user_id)
                .copied()
                .collect();
            
            friends.shuffle(&mut rng);
            friends.truncate(num_friends);
            
            for friend_id in friends {
                let query = format!(
                    "INSERT EDGE KNOWS(since, strength) VALUES {} -> {}:('2024-01-{:02}', {}.{})",
                    user_id,
                    friend_id,
                    (user_id % 28 + 1),
                    (user_id % 10),
                    (friend_id % 10)
                );
                
                self.ctx.execute_query_ok(&query).await?;
            }
        }
        
        Ok(())
    }
    
    /// 生成动态和互动数据
    pub async fn generate_posts_and_interactions(
        &self,
        user_ids: &[i64],
        posts_per_user: usize,
    ) -> anyhow::Result<Vec<i64>> {
        let mut post_ids = Vec::new();
        let mut next_post_id = 1000i64;
        
        for user_id in user_ids {
            for _ in 0..posts_per_user {
                let post_id = next_post_id;
                next_post_id += 1;
                
                let content = format!("Post content from user {} - post {}", user_id, post_id);
                let likes = (user_id % 100) as i32;
                
                let query = format!(
                    "INSERT VERTEX Post(content, created_at, likes) VALUES {}:('{}', now(), {})",
                    post_id, content, likes
                );
                self.ctx.execute_query_ok(&query).await?;
                
                let posted_query = format!("INSERT EDGE POSTED VALUES {} -> {}", user_id, post_id);
                self.ctx.execute_query_ok(&posted_query).await?;
                
                post_ids.push(post_id);
            }
        }
        
        Ok(post_ids)
    }
    
    /// 生成完整社交网络
    pub async fn generate_social_graph(
        &self,
        user_count: usize,
    ) -> anyhow::Result<SocialGraph> {
        self.generate_base_schema().await?;
        
        let user_ids = self.generate_users(user_count).await?;
        self.generate_friendships(&user_ids, 5).await?;
        let post_ids = self.generate_posts_and_interactions(&user_ids, 3).await?;
        
        Ok(SocialGraph {
            user_ids,
            post_ids,
        })
    }
}

/// 社交网络图数据
pub struct SocialGraph {
    pub user_ids: Vec<i64>,
    pub post_ids: Vec<i64>,
}

/// 电商数据生成器
pub struct ECommerceDataGenerator {
    ctx: E2eTestContext,
}

impl ECommerceDataGenerator {
    pub fn new(ctx: &E2eTestContext) -> Self {
        Self { ctx: ctx.clone() }
    }
    
    /// 生成基础模式
    pub async fn generate_base_schema(&self) -> anyhow::Result<()> {
        let schema_queries = vec![
            "CREATE SPACE IF NOT EXISTS ecommerce",
            "USE ecommerce",
            "CREATE TAG IF NOT EXISTS User(name: STRING, age: INT, gender: STRING, city: STRING)",
            "CREATE TAG IF NOT EXISTS Product(name: STRING, category: STRING, price: DOUBLE, brand: STRING)",
            "CREATE TAG IF NOT EXISTS Category(name: STRING, parent_id: INT)",
            "CREATE TAG IF NOT EXISTS Order(total_amount: DOUBLE, status: STRING, created_at: TIMESTAMP)",
            "CREATE EDGE IF NOT EXISTS VIEWED(view_time: TIMESTAMP, duration: INT)",
            "CREATE EDGE IF NOT EXISTS ADDED_TO_CART(added_at: TIMESTAMP, quantity: INT)",
            "CREATE EDGE IF NOT EXISTS PURCHASED(order_id: STRING, quantity: INT, price: DOUBLE)",
            "CREATE EDGE IF NOT EXISTS BELONGS_TO",
            "CREATE EDGE IF NOT EXISTS SIMILAR_TO(similarity_score: DOUBLE)",
            "CREATE EDGE IF NOT EXISTS BOUGHT_TOGETHER(frequency: INT)",
        ];
        
        for query in schema_queries {
            self.ctx.execute_query_ok(query).await?;
        }
        
        Ok(())
    }
    
    /// 生成分类体系
    pub async fn generate_categories(&self) -> anyhow::Result<HashMap<String, i64>> {
        let mut categories = HashMap::new();
        let category_data = vec![
            (1, "Electronics", None),
            (2, "Phones", Some(1)),
            (3, "Laptops", Some(1)),
            (4, "Clothing", None),
            (5, "Men", Some(4)),
            (6, "Women", Some(4)),
            (7, "Books", None),
            (8, "Fiction", Some(7)),
            (9, "Non-fiction", Some(7)),
        ];
        
        for (id, name, parent_id) in category_data {
            let parent = parent_id.map(|p| p.to_string()).unwrap_or_else(|| "NULL".to_string());
            let query = format!(
                "INSERT VERTEX Category(name, parent_id) VALUES {}:('{}', {})",
                id, name, parent
            );
            self.ctx.execute_query_ok(&query).await?;
            categories.insert(name.to_string(), id);
        }
        
        Ok(categories)
    }
    
    /// 生成商品
    pub async fn generate_products(
        &self,
        count: usize,
        categories: &HashMap<String, i64>,
    ) -> anyhow::Result<Vec<i64>> {
        let mut product_ids = Vec::new();
        let brands = vec!["Apple", "Samsung", "Nike", "Adidas", "Sony"];
        let category_names: Vec<&String> = categories.keys().collect();
        
        for i in 0..count {
            let id = (i + 100) as i64;
            let name = format!("Product-{}", id);
            let category = category_names[i % category_names.len()];
            let price = 10.0 + (i % 1000) as f64;
            let brand = brands[i % brands.len()];
            
            let query = format!(
                "INSERT VERTEX Product(name, category, price, brand) VALUES {}:('{}', '{}', {}, '{}')",
                id, name, category, price, brand
            );
            self.ctx.execute_query_ok(&query).await?;
            
            if let Some(&cat_id) = categories.get(category) {
                let belongs_query = format!(
                    "INSERT EDGE BELONGS_TO VALUES {} -> {}",
                    id, cat_id
                );
                self.ctx.execute_query_ok(&belongs_query).await?;
            }
            
            product_ids.push(id);
        }
        
        Ok(product_ids)
    }
    
    /// 生成用户
    pub async fn generate_users(&self, count: usize) -> anyhow::Result<Vec<i64>> {
        let mut user_ids = Vec::new();
        let cities = vec!["Beijing", "Shanghai", "Guangzhou", "Shenzhen"];
        let genders = vec!["Male", "Female"];
        
        for i in 0..count {
            let id = (i + 1) as i64;
            let name = format!("Customer{}", id);
            let age = 18 + (i % 60) as i32;
            let gender = genders[i % genders.len()];
            let city = cities[i % cities.len()];
            
            let query = format!(
                "INSERT VERTEX User(name, age, gender, city) VALUES {}:('{}', {}, '{}', '{}')",
                id, name, age, gender, city
            );
            self.ctx.execute_query_ok(&query).await?;
            user_ids.push(id);
        }
        
        Ok(user_ids)
    }
    
    /// 生成购买记录
    pub async fn generate_purchase_history(
        &self,
        user_ids: &[i64],
        product_ids: &[i64],
        records_per_user: usize,
    ) -> anyhow::Result<()> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        
        for user_id in user_ids {
            let mut products: Vec<i64> = product_ids.to_vec();
            products.shuffle(&mut rng);
            
            for (i, product_id) in products.iter().take(records_per_user).enumerate() {
                let order_id = format!("ORD-{}-{}", user_id, i);
                let quantity = 1 + (user_id % 5) as i32;
                let price = 10.0 + (product_id % 1000) as f64;
                
                let query = format!(
                    "INSERT EDGE PURCHASED(order_id, quantity, price) VALUES {} -> {}:('{}', {}, {})",
                    user_id, product_id, order_id, quantity, price
                );
                self.ctx.execute_query_ok(&query).await?;
            }
        }
        
        Ok(())
    }
    
    /// 生成相似商品关系
    pub async fn generate_similarity_relations(
        &self,
        product_ids: &[i64],
        similarities_per_product: usize,
    ) -> anyhow::Result<()> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        
        for product_id in product_ids {
            let mut others: Vec<i64> = product_ids
                .iter()
                .filter(|&&id| id != *product_id)
                .copied()
                .collect();
            
            others.shuffle(&mut rng);
            
            for (i, similar_id) in others.iter().take(similarities_per_product).enumerate() {
                let score = 0.5 + (i as f64 * 0.05);
                
                let query = format!(
                    "INSERT EDGE SIMILAR_TO(similarity_score) VALUES {} -> {}:({})",
                    product_id, similar_id, score
                );
                self.ctx.execute_query_ok(&query).await?;
            }
        }
        
        Ok(())
    }
}

/// 知识图谱数据生成器
pub struct KnowledgeGraphDataGenerator {
    ctx: E2eTestContext,
}

impl KnowledgeGraphDataGenerator {
    pub fn new(ctx: &E2eTestContext) -> Self {
        Self { ctx: ctx.clone() }
    }
    
    /// 生成基础模式
    pub async fn generate_base_schema(&self) -> anyhow::Result<()> {
        let schema_queries = vec![
            "CREATE SPACE IF NOT EXISTS knowledge_graph",
            "USE knowledge_graph",
            "CREATE TAG IF NOT EXISTS Entity(name: STRING, type: STRING, description: STRING)",
            "CREATE TAG IF NOT EXISTS Concept(name: STRING, domain: STRING)",
            "CREATE TAG IF NOT EXISTS Document(title: STRING, content: STRING, source: STRING)",
            "CREATE EDGE IF NOT EXISTS RELATES_TO(relation_type: STRING, confidence: DOUBLE)",
            "CREATE EDGE IF NOT EXISTS INSTANCE_OF",
            "CREATE EDGE IF NOT EXISTS MENTIONS(frequency: INT)",
            "CREATE EDGE IF NOT EXISTS PART_OF",
            "CREATE EDGE IF NOT EXISTS CAUSES",
        ];
        
        for query in schema_queries {
            self.ctx.execute_query_ok(query).await?;
        }
        
        Ok(())
    }
    
    /// 生成实体
    pub async fn generate_entities(
        &self,
        count: usize,
    ) -> anyhow::Result<HashMap<String, i64>> {
        let mut entities = HashMap::new();
        let entity_types = vec!["Person", "Organization", "Location", "Technology", "Event"];
        
        for i in 0..count {
            let id = (i + 1) as i64;
            let name = format!("Entity{}", id);
            let entity_type = entity_types[i % entity_types.len()];
            let description = format!("Description of {}", name);
            
            let query = format!(
                "INSERT VERTEX Entity(name, type, description) VALUES {}:('{}', '{}', '{}')",
                id, name, entity_type, description
            );
            self.ctx.execute_query_ok(&query).await?;
            
            entities.insert(name, id);
        }
        
        Ok(entities)
    }
    
    /// 生成实体关系
    pub async fn generate_relations(
        &self,
        entities: &HashMap<String, i64>,
        avg_relations_per_entity: usize,
    ) -> anyhow::Result<()> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        
        let relation_types = vec![
            "works_for",
            "located_in",
            "founded_by",
            "competes_with",
            "collaborates_with",
        ];
        
        let entity_ids: Vec<i64> = entities.values().copied().collect();
        
        for (i, &entity_id) in entity_ids.iter().enumerate() {
            let num_relations = avg_relations_per_entity + (i % 3);
            let mut targets: Vec<i64> = entity_ids
                .iter()
                .filter(|&&id| id != entity_id)
                .copied()
                .collect();
            
            targets.shuffle(&mut rng);
            
            for (j, target_id) in targets.iter().take(num_relations).enumerate() {
                let relation_type = relation_types[j % relation_types.len()];
                let confidence = 0.7 + (j as f64 * 0.05);
                
                let query = format!(
                    "INSERT EDGE RELATES_TO(relation_type, confidence) VALUES {} -> {}:('{}', {})",
                    entity_id, target_id, relation_type, confidence
                );
                self.ctx.execute_query_ok(&query).await?;
            }
        }
        
        Ok(())
    }
}

/// 性能测试数据生成器
pub struct PerformanceDataGenerator {
    ctx: E2eTestContext,
}

impl PerformanceDataGenerator {
    pub fn new(ctx: &E2eTestContext) -> Self {
        Self { ctx: ctx.clone() }
    }
    
    /// 生成基础模式
    pub async fn generate_base_schema(&self) -> anyhow::Result<()> {
        let schema_queries = vec![
            "CREATE SPACE IF NOT EXISTS performance_test",
            "USE performance_test",
            "CREATE TAG IF NOT EXISTS Node(name: STRING, value: INT, category: STRING)",
            "CREATE EDGE IF NOT EXISTS CONNECTS(weight: DOUBLE)",
        ];
        
        for query in schema_queries {
            self.ctx.execute_query_ok(query).await?;
        }
        
        Ok(())
    }
    
    /// 生成大规模图
    pub async fn generate_large_graph(
        &self,
        node_count: usize,
        edge_count: usize,
    ) -> anyhow::Result<()> {
        self.generate_base_schema().await?;
        
        let categories = vec!["A", "B", "C", "D", "E"];
        
        // 批量插入顶点
        let batch_size = 1000;
        for batch_start in (0..node_count).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(node_count);
            let mut values = Vec::new();
            
            for i in batch_start..batch_end {
                let id = (i + 1) as i64;
                let name = format!("Node{}", id);
                let value = (i % 1000) as i32;
                let category = categories[i % categories.len()];
                values.push(format!("{}:('{}', {}, '{}')", id, name, value, category));
            }
            
            let query = format!(
                "INSERT VERTEX Node(name, value, category) VALUES {}",
                values.join(", ")
            );
            self.ctx.execute_query_ok(&query).await?;
        }
        
        // 批量插入边
        let edges_per_batch = 1000;
        for batch_start in (0..edge_count).step_by(edges_per_batch) {
            let batch_end = (batch_start + edges_per_batch).min(edge_count);
            let mut edges = Vec::new();
            
            for i in batch_start..batch_end {
                let src = ((i % node_count) + 1) as i64;
                let dst = (((i * 7 + 13) % node_count) + 1) as i64;
                let weight = (i % 100) as f64;
                edges.push(format!("{} -> {}:({})", src, dst, weight));
            }
            
            let query = format!(
                "INSERT EDGE CONNECTS(weight) VALUES {}",
                edges.join(", ")
            );
            self.ctx.execute_query_ok(&query).await?;
        }
        
        Ok(())
    }
}
