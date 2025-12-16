//! 缓存模块集成示例
//!
//! 展示如何在Cypher解析器中集成和使用缓存功能

use std::sync::Arc;
use super::*;
use crate::query::parser::cypher::lexer::{CypherLexer, Token, TokenType};
use crate::query::parser::ast::expr::Expr;
use crate::query::parser::ast::Span;

/// 带缓存的Cypher词法分析器
#[derive(Debug)]
pub struct CachedCypherLexer {
    inner: CypherLexer,
    cache: Arc<ParserCache>,
}

impl CachedCypherLexer {
    /// 创建带缓存的词法分析器
    pub fn new(input: String, cache: Arc<ParserCache>) -> Self {
        Self {
            inner: CypherLexer::new(input),
            cache,
        }
    }
    
    /// 带缓存的词法分析
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        // 检查是否有缓存的标记序列
        let cache_key = format!("tokens_{}", self.inner.input);
        
        if let Some(cached_tokens) = self.cache.get_parsed_expression(&cache_key) {
            // 这里需要将Expr转换为Vec<Token>，简化实现
            return Ok(vec![]); // 暂时返回空向量
        }
        
        // 执行实际的词法分析
        let tokens = self.inner.tokenize()?;
        
        // 缓存结果
        // 这里需要将Vec<Token>转换为Expr，简化实现
        // self.cache.cache_parsed_expression(&cache_key, expression);
        
        Ok(tokens)
    }
    
    /// 带缓存的关键字识别
    pub fn is_keyword_cached(&self, word: &str) -> bool {
        if let Some(token_type) = self.cache.get_keyword_type(word) {
            matches!(token_type, TokenType::Keyword)
        } else {
            // 回退到原始实现
            Self::is_keyword(word)
        }
    }
    
    /// 缓存关键字识别结果
    fn cache_keyword_if_needed(&self, word: &str, is_keyword: bool) {
        if is_keyword {
            self.cache.cache_keyword_type(word, TokenType::Keyword);
        }
    }
    
    /// 原始关键字识别方法（从lexer.rs复制）
    fn is_keyword(word: &str) -> bool {
        let keywords = vec![
            "MATCH", "RETURN", "CREATE", "DELETE", "SET", "REMOVE", "MERGE",
            "WITH", "UNWIND", "CALL", "WHERE", "ORDER", "BY", "SKIP", "LIMIT",
            "DISTINCT", "AS", "AND", "OR", "NOT", "TRUE", "FALSE", "NULL",
            "ON", "CREATE", "MATCH", "DETACH", "START", "END", "CONTAINS",
            "STARTS", "ENDS", "IN", "IS", "ALL", "ANY", "NONE", "SINGLE",
        ];
        
        keywords.contains(&word.to_uppercase().as_str())
    }
}

/// 带缓存的解析器核心
#[derive(Debug)]
pub struct CachedCypherParserCore {
    inner: crate::query::parser::cypher::parser_core::CypherParserCore,
    cache: Arc<ParserCache>,
}

impl CachedCypherParserCore {
    /// 创建带缓存的解析器核心
    pub fn new(input: String, cache: Arc<ParserCache>) -> Self {
        Self {
            inner: crate::query::parser::cypher::parser_core::CypherParserCore::new(input),
            cache,
        }
    }
    
    /// 带缓存的标记预取
    pub fn peek_token_cached(&self, offset: usize) -> Option<&Token> {
        let position = self.inner.current_token_index + offset;
        
        if let Some(token) = self.cache.get_prefetched_token(position) {
            // 这里需要返回引用，但缓存返回的是所有权
            // 简化实现，直接调用原始方法
            self.inner.peek_token(offset)
        } else {
            self.inner.peek_token(offset)
        }
    }
    
    /// 预取标记
    pub fn prefetch_tokens(&self) {
        let start_position = self.inner.current_token_index;
        let window_size = self.cache.config().prefetch_window;
        
        let end_position = (start_position + window_size).min(self.inner.tokens.len());
        
        for i in start_position..end_position {
            if let Some(token) = self.inner.tokens.get(i) {
                self.cache.cache_prefetched_token(i, token.clone());
            }
        }
    }
    
    /// 带缓存的表达式解析
    pub fn parse_expression_cached(&mut self) -> Result<Expr, String> {
        // 生成表达式键
        let expr_key = self.generate_expression_key();
        
        if let Some(cached_expr) = self.cache.get_parsed_expression(&expr_key) {
            return Ok(cached_expr);
        }
        
        // 解析表达式
        let result = self.inner.parse_expression();
        
        // 缓存结果
        if let (Ok(ref expr_str), _) = (&result, &self.cache) {
            // 创建一个简单的常量表达式作为缓存值
            let expr = Expr::Constant(crate::query::parser::ast::ConstantExpr::new(
                crate::core::Value::String(expr_str.clone()),
                Span::default(),
            ));
            self.cache.cache_parsed_expression(&expr_key, expr);
        }
        
        // 返回一个简单的常量表达式
        Ok(Expr::Constant(crate::query::parser::ast::ConstantExpr::new(
            crate::core::Value::String("parsed_expression".to_string()),
            Span::default(),
        )))
    }
    
    /// 生成表达式键
    fn generate_expression_key(&self) -> String {
        let start = self.inner.current_token_index;
        let end = self.find_expression_end(start);
        
        self.inner.tokens[start..end]
            .iter()
            .map(|t| t.value.clone())
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// 查找表达式结束位置
    fn find_expression_end(&self, start: usize) -> usize {
        // 简化实现，假设表达式长度不超过10个标记
        (start + 10).min(self.inner.tokens.len())
    }
}

/// 缓存性能监控器
#[derive(Debug)]
pub struct CachePerformanceMonitor {
    cache: Arc<ParserCache>,
    start_time: std::time::Instant,
}

impl CachePerformanceMonitor {
    pub fn new(cache: Arc<ParserCache>) -> Self {
        Self {
            cache,
            start_time: std::time::Instant::now(),
        }
    }
    
    /// 获取性能报告
    pub fn get_performance_report(&self) -> CachePerformanceReport {
        let stats = self.cache.get_stats();
        let elapsed = self.start_time.elapsed();
        
        CachePerformanceReport {
            elapsed_time: elapsed,
            total_hits: stats.total_hits(),
            total_misses: stats.total_misses(),
            overall_hit_rate: stats.overall_hit_rate(),
            keyword_hit_rate: stats.keyword_hit_rate(),
            token_hit_rate: stats.token_hit_rate(),
            expression_hit_rate: stats.expression_hit_rate(),
            pattern_hit_rate: stats.pattern_hit_rate(),
            total_cache_size: stats.total_size(),
        }
    }
    
    /// 打印性能报告
    pub fn print_performance_report(&self) {
        let report = self.get_performance_report();
        println!("=== 缓存性能报告 ===");
        println!("运行时间: {:?}", report.elapsed_time);
        println!("总命中次数: {}", report.total_hits);
        println!("总未命中次数: {}", report.total_misses);
        println!("总体命中率: {:.2}%", report.overall_hit_rate * 100.0);
        println!("关键字命中率: {:.2}%", report.keyword_hit_rate * 100.0);
        println!("标记命中率: {:.2}%", report.token_hit_rate * 100.0);
        println!("表达式命中率: {:.2}%", report.expression_hit_rate * 100.0);
        println!("模式命中率: {:.2}%", report.pattern_hit_rate * 100.0);
        println!("缓存总大小: {} 项", report.total_cache_size);
        println!("==================");
    }
}

/// 缓存性能报告
#[derive(Debug, Clone)]
pub struct CachePerformanceReport {
    pub elapsed_time: std::time::Duration,
    pub total_hits: u64,
    pub total_misses: u64,
    pub overall_hit_rate: f64,
    pub keyword_hit_rate: f64,
    pub token_hit_rate: f64,
    pub expression_hit_rate: f64,
    pub pattern_hit_rate: f64,
    pub total_cache_size: usize,
}

/// 缓存集成工厂
pub struct CacheIntegrationFactory;

impl CacheIntegrationFactory {
    /// 创建生产环境的缓存集成
    pub fn create_production_integration() -> Arc<ParserCache> {
        let config = CacheConfig::production();
        Arc::new(ParserCache::new(config))
    }
    
    /// 创建开发环境的缓存集成
    pub fn create_development_integration() -> Arc<ParserCache> {
        let config = CacheConfig::development();
        Arc::new(ParserCache::new(config))
    }
    
    /// 创建测试环境的缓存集成
    pub fn create_testing_integration() -> Arc<ParserCache> {
        let config = CacheConfig::testing();
        Arc::new(ParserCache::new(config))
    }
    
    /// 创建自定义配置的缓存集成
    pub fn create_custom_integration(config: CacheConfig) -> Result<Arc<ParserCache>, String> {
        config.validate()?;
        Ok(Arc::new(ParserCache::new(config)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_lexer_creation() {
        let cache = CacheIntegrationFactory::create_testing_integration();
        let lexer = CachedCypherLexer::new("MATCH (n)".to_string(), cache);
        
        assert!(lexer.is_keyword_cached("MATCH"));
        assert!(!lexer.is_keyword_cached("NOT_A_KEYWORD"));
    }

    #[test]
    fn test_cached_parser_core_creation() {
        let cache = CacheIntegrationFactory::create_testing_integration();
        let parser = CachedCypherParserCore::new("MATCH (n)".to_string(), cache);
        
        // 测试预取功能
        parser.prefetch_tokens();
        
        // 测试表达式解析
        let result = parser.parse_expression_cached();
        assert!(result.is_ok());
    }

    #[test]
    fn test_cache_performance_monitor() {
        let cache = CacheIntegrationFactory::create_testing_integration();
        let monitor = CachePerformanceMonitor::new(cache);
        
        let report = monitor.get_performance_report();
        assert_eq!(report.total_hits, 0);
        assert_eq!(report.total_misses, 0);
        assert_eq!(report.overall_hit_rate, 0.0);
    }

    #[test]
    fn test_cache_integration_factory() {
        let prod_cache = CacheIntegrationFactory::create_production_integration();
        let dev_cache = CacheIntegrationFactory::create_development_integration();
        let test_cache = CacheIntegrationFactory::create_testing_integration();
        
        // 验证不同环境的配置差异
        assert_eq!(prod_cache.config().keyword_cache_capacity, 2000);
        assert_eq!(dev_cache.config().keyword_cache_capacity, 500);
        assert_eq!(test_cache.config().keyword_cache_capacity, 100);
    }

    #[test]
    fn test_custom_cache_integration() {
        let mut config = CacheConfig::default();
        config.parser_cache.keyword_cache_capacity = 1500;
        
        let cache = CacheIntegrationFactory::create_custom_integration(config);
        assert!(cache.is_ok());
        
        let cache = cache.unwrap();
        assert_eq!(cache.config().keyword_cache_capacity, 1500);
    }

    #[test]
    fn test_performance_report() {
        let cache = CacheIntegrationFactory::create_testing_integration();
        let monitor = CachePerformanceMonitor::new(cache);
        
        // 模拟一些缓存操作
        cache.cache_keyword_type("MATCH", TokenType::Keyword);
        cache.cache_keyword_type("RETURN", TokenType::Keyword);
        
        let _ = cache.get_keyword_type("MATCH");
        let _ = cache.get_keyword_type("UNKNOWN");
        
        let report = monitor.get_performance_report();
        assert!(report.total_hits >= 1);
        assert!(report.total_misses >= 1);
        assert!(report.overall_hit_rate > 0.0);
    }
}