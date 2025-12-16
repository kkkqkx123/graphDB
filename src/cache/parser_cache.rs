//! 解析器特化缓存
//!
//! 为Cypher解析器提供专门的缓存功能

use std::sync::Arc;
use std::time::Duration;
use super::traits::*;
use super::config::*;
use super::implementations::*;
use crate::query::parser::cypher::lexer::{Token, TokenType};
use crate::query::parser::ast::expr::Expr;

#[derive(Debug, Clone)]
pub struct Pattern {
    pub pattern: String,
}

impl Pattern {
    pub fn new(pattern: String) -> Self {
        Self { pattern }
    }
}

/// 解析器专用缓存
#[derive(Debug)]
pub struct ParserCache {
    manager: Arc<CacheManager>,
    
    // 特化缓存实例
    keyword_cache: Arc<dyn Cache<String, TokenType>>,
    token_cache: Arc<dyn Cache<usize, Token>>,
    expression_cache: Arc<dyn Cache<String, Expr>>,
    pattern_cache: Arc<dyn Cache<String, Pattern>>,
    
    // 统计缓存
    keyword_stats: Option<Arc<dyn StatsCache<String, TokenType>>>,
    token_stats: Option<Arc<dyn StatsCache<usize, Token>>>,
    expression_stats: Option<Arc<dyn StatsCache<String, Expr>>>,
    pattern_stats: Option<Arc<dyn StatsCache<String, Pattern>>>,
    
    config: ParserCacheConfig,
}

impl ParserCache {
    /// 创建新的解析器缓存
    pub fn new(config: CacheConfig) -> Self {
        let manager = Arc::new(CacheManager::new(config.clone()));
        let parser_config = config.parser_cache.clone();
        
        // 创建关键字缓存
        let keyword_cache = manager.create_lru_cache(parser_config.keyword_cache_capacity);
        let keyword_stats = if config.collect_stats {
            Some(manager.create_stats_cache(keyword_cache.clone()))
        } else {
            None
        };
        
        // 创建标记缓存
        let token_cache = manager.create_lru_cache(parser_config.token_cache_capacity);
        let token_stats = if config.collect_stats {
            Some(manager.create_stats_cache(token_cache.clone()))
        } else {
            None
        };
        
        // 创建表达式缓存
        let expression_cache = manager.create_ttl_cache(
            parser_config.expression_cache_capacity,
            parser_config.expression_cache_ttl,
        );
        let expression_stats = if config.collect_stats {
            Some(manager.create_stats_cache(expression_cache.clone()))
        } else {
            None
        };
        
        // 创建模式缓存
        let pattern_cache = manager.create_ttl_cache(
            parser_config.pattern_cache_capacity,
            parser_config.pattern_cache_ttl,
        );
        let pattern_stats = if config.collect_stats {
            Some(manager.create_stats_cache(pattern_cache.clone()))
        } else {
            None
        };
        
        Self {
            manager,
            keyword_cache,
            token_cache,
            expression_cache,
            pattern_cache,
            keyword_stats,
            token_stats,
            expression_stats,
            pattern_stats,
            config: parser_config,
        }
    }
    
    /// 缓存关键字识别结果
    pub fn get_keyword_type(&self, word: &str) -> Option<TokenType> {
        let key = word.to_uppercase();
        if let Some(stats_cache) = &self.keyword_stats {
            stats_cache.get(&key)
        } else {
            self.keyword_cache.get(&key)
        }
    }
    
    pub fn cache_keyword_type(&self, word: &str, token_type: TokenType) {
        let key = word.to_uppercase();
        self.keyword_cache.put(key, token_type);
    }
    
    /// 缓存标记预取结果
    pub fn get_prefetched_token(&self, position: usize) -> Option<Token> {
        if let Some(stats_cache) = &self.token_stats {
            stats_cache.get(&position)
        } else {
            self.token_cache.get(&position)
        }
    }
    
    pub fn cache_prefetched_token(&self, position: usize, token: Token) {
        self.token_cache.put(position, token);
    }
    
    /// 批量预取标记
    pub fn prefetch_tokens(&self, tokens: &[Token], start_position: usize) {
        if !self.config.prefetch_window > 0 {
            return;
        }
        
        let end_position = (start_position + self.config.prefetch_window).min(tokens.len());
        for (i, token) in tokens[start_position..end_position].iter().enumerate() {
            let position = start_position + i;
            if !self.token_cache.contains(&position) {
                self.cache_prefetched_token(position, token.clone());
            }
        }
    }
    
    /// 缓存表达式解析结果
    pub fn get_parsed_expression(&self, expr_str: &str) -> Option<Expr> {
        if let Some(stats_cache) = &self.expression_stats {
            stats_cache.get(&expr_str.to_string())
        } else {
            self.expression_cache.get(&expr_str.to_string())
        }
    }
    
    pub fn cache_parsed_expression(&self, expr_str: &str, expression: Expr) {
        self.expression_cache.put(expr_str.to_string(), expression);
    }
    
    /// 缓存模式解析结果
    pub fn get_parsed_pattern(&self, pattern_str: &str) -> Option<Pattern> {
        if let Some(stats_cache) = &self.pattern_stats {
            stats_cache.get(&pattern_str.to_string())
        } else {
            self.pattern_cache.get(&pattern_str.to_string())
        }
    }
    
    pub fn cache_parsed_pattern(&self, pattern_str: &str, pattern: Pattern) {
        self.pattern_cache.put(pattern_str.to_string(), pattern);
    }
    
    /// 获取缓存统计信息
    pub fn get_stats(&self) -> ParserCacheStats {
        let mut stats = ParserCacheStats::new();
        
        if let Some(keyword_stats) = &self.keyword_stats {
            stats.keyword_hits = keyword_stats.hits();
            stats.keyword_misses = keyword_stats.misses();
            stats.keyword_size = keyword_stats.len();
        }
        
        if let Some(token_stats) = &self.token_stats {
            stats.token_hits = token_stats.hits();
            stats.token_misses = token_stats.misses();
            stats.token_size = token_stats.len();
        }
        
        if let Some(expression_stats) = &self.expression_stats {
            stats.expression_hits = expression_stats.hits();
            stats.expression_misses = expression_stats.misses();
            stats.expression_size = expression_stats.len();
        }
        
        if let Some(pattern_stats) = &self.pattern_stats {
            stats.pattern_hits = pattern_stats.hits();
            stats.pattern_misses = pattern_stats.misses();
            stats.pattern_size = pattern_stats.len();
        }
        
        stats
    }
    
    /// 重置所有统计信息
    pub fn reset_stats(&self) {
        if let Some(keyword_stats) = &self.keyword_stats {
            keyword_stats.reset_stats();
        }
        
        if let Some(token_stats) = &self.token_stats {
            token_stats.reset_stats();
        }
        
        if let Some(expression_stats) = &self.expression_stats {
            expression_stats.reset_stats();
        }
        
        if let Some(pattern_stats) = &self.pattern_stats {
            pattern_stats.reset_stats();
        }
    }
    
    /// 清空所有缓存
    pub fn clear_all(&self) {
        self.keyword_cache.clear();
        self.token_cache.clear();
        self.expression_cache.clear();
        self.pattern_cache.clear();
    }
    
    /// 获取配置
    pub fn config(&self) -> &ParserCacheConfig {
        &self.config
    }
    
    /// 获取缓存管理器
    pub fn manager(&self) -> &Arc<CacheManager> {
        &self.manager
    }
}

/// 解析器缓存统计信息
#[derive(Debug, Clone)]
pub struct ParserCacheStats {
    pub keyword_hits: u64,
    pub keyword_misses: u64,
    pub keyword_size: usize,
    
    pub token_hits: u64,
    pub token_misses: u64,
    pub token_size: usize,
    
    pub expression_hits: u64,
    pub expression_misses: u64,
    pub expression_size: usize,
    
    pub pattern_hits: u64,
    pub pattern_misses: u64,
    pub pattern_size: usize,
}

impl ParserCacheStats {
    pub fn new() -> Self {
        Self {
            keyword_hits: 0,
            keyword_misses: 0,
            keyword_size: 0,
            
            token_hits: 0,
            token_misses: 0,
            token_size: 0,
            
            expression_hits: 0,
            expression_misses: 0,
            expression_size: 0,
            
            pattern_hits: 0,
            pattern_misses: 0,
            pattern_size: 0,
        }
    }
    
    pub fn total_hits(&self) -> u64 {
        self.keyword_hits + self.token_hits + self.expression_hits + self.pattern_hits
    }
    
    pub fn total_misses(&self) -> u64 {
        self.keyword_misses + self.token_misses + self.expression_misses + self.pattern_misses
    }
    
    pub fn total_size(&self) -> usize {
        self.keyword_size + self.token_size + self.expression_size + self.pattern_size
    }
    
    pub fn overall_hit_rate(&self) -> f64 {
        let total_hits = self.total_hits();
        let total_misses = self.total_misses();
        
        if total_hits + total_misses == 0 {
            0.0
        } else {
            total_hits as f64 / (total_hits + total_misses) as f64
        }
    }
    
    pub fn keyword_hit_rate(&self) -> f64 {
        if self.keyword_hits + self.keyword_misses == 0 {
            0.0
        } else {
            self.keyword_hits as f64 / (self.keyword_hits + self.keyword_misses) as f64
        }
    }
    
    pub fn token_hit_rate(&self) -> f64 {
        if self.token_hits + self.token_misses == 0 {
            0.0
        } else {
            self.token_hits as f64 / (self.token_hits + self.token_misses) as f64
        }
    }
    
    pub fn expression_hit_rate(&self) -> f64 {
        if self.expression_hits + self.expression_misses == 0 {
            0.0
        } else {
            self.expression_hits as f64 / (self.expression_hits + self.expression_misses) as f64
        }
    }
    
    pub fn pattern_hit_rate(&self) -> f64 {
        if self.pattern_hits + self.pattern_misses == 0 {
            0.0
        } else {
            self.pattern_hits as f64 / (self.pattern_hits + self.pattern_misses) as f64
        }
    }
}

/// 关键字缓存助手
pub struct KeywordCache {
    cache: Arc<dyn Cache<String, TokenType>>,
    stats: Option<Arc<dyn StatsCache<String, TokenType>>>,
}

impl KeywordCache {
    pub fn new(cache: Arc<dyn Cache<String, TokenType>>, collect_stats: bool) -> Self {
        let stats = if collect_stats {
            // 这里需要创建统计包装器
            None // 暂时简化
        } else {
            None
        };
        
        Self { cache, stats }
    }
    
    pub fn is_keyword(&self, word: &str) -> bool {
        let key = word.to_uppercase();
        if let Some(stats_cache) = &self.stats {
            stats_cache.get(&key).is_some()
        } else {
            self.cache.get(&key).is_some()
        }
    }
    
    pub fn cache_keyword(&self, word: &str, is_keyword: bool) {
        if is_keyword {
            let key = word.to_uppercase();
            self.cache.put(key, TokenType::Keyword);
        }
    }
    
    pub fn get_stats(&self) -> Option<(u64, u64, f64)> {
        self.stats.as_ref().map(|stats| {
            (stats.hits(), stats.misses(), stats.hit_rate())
        })
    }
}

/// 表达式缓存助手
pub struct ExpressionCache {
    cache: Arc<dyn Cache<String, Expr>>,
    stats: Option<Arc<dyn StatsCache<String, Expr>>>,
}

impl ExpressionCache {
    pub fn new(cache: Arc<dyn Cache<String, Expr>>, collect_stats: bool) -> Self {
        let stats = if collect_stats {
            None // 暂时简化
        } else {
            None
        };
        
        Self { cache, stats }
    }
    
    pub fn get_expression(&self, expr_str: &str) -> Option<Expr> {
        if let Some(stats_cache) = &self.stats {
            stats_cache.get(&expr_str.to_string())
        } else {
            self.cache.get(&expr_str.to_string())
        }
    }
    
    pub fn cache_expression(&self, expr_str: &str, expression: Expr) {
        self.cache.put(expr_str.to_string(), expression);
    }
    
    pub fn get_or_compute<F>(&self, expr_str: &str, compute: F) -> Expr
    where
        F: FnOnce() -> Expr,
    {
        if let Some(cached) = self.get_expression(expr_str) {
            cached
        } else {
            let expression = compute();
            self.cache_expression(expr_str, expression.clone());
            expression
        }
    }
    
    pub fn get_stats(&self) -> Option<(u64, u64, f64)> {
        self.stats.as_ref().map(|stats| {
            (stats.hits(), stats.misses(), stats.hit_rate())
        })
    }
}

/// 模式缓存助手
pub struct PatternCache {
    cache: Arc<dyn Cache<String, Pattern>>,
    stats: Option<Arc<dyn StatsCache<String, Pattern>>>,
}

impl PatternCache {
    pub fn new(cache: Arc<dyn Cache<String, Pattern>>, collect_stats: bool) -> Self {
        let stats = if collect_stats {
            None // 暂时简化
        } else {
            None
        };
        
        Self { cache, stats }
    }
    
    pub fn get_pattern(&self, pattern_str: &str) -> Option<Pattern> {
        if let Some(stats_cache) = &self.stats {
            stats_cache.get(&pattern_str.to_string())
        } else {
            self.cache.get(&pattern_str.to_string())
        }
    }
    
    pub fn cache_pattern(&self, pattern_str: &str, pattern: Pattern) {
        self.cache.put(pattern_str.to_string(), pattern);
    }
    
    pub fn get_or_compute<F>(&self, pattern_str: &str, compute: F) -> Pattern
    where
        F: FnOnce() -> Pattern,
    {
        if let Some(cached) = self.get_pattern(pattern_str) {
            cached
        } else {
            let pattern = compute();
            self.cache_pattern(pattern_str, pattern.clone());
            pattern
        }
    }
    
    pub fn get_stats(&self) -> Option<(u64, u64, f64)> {
        self.stats.as_ref().map(|stats| {
            (stats.hits(), stats.misses(), stats.hit_rate())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::cypher::lexer::{Token, TokenType};

    #[test]
    fn test_parser_cache_creation() {
        let config = CacheConfig::default();
        let parser_cache = ParserCache::new(config);
        
        assert!(parser_cache.get_keyword_type("MATCH").is_none());
        assert_eq!(parser_cache.get_stats().total_hits(), 0);
    }

    #[test]
    fn test_keyword_caching() {
        let config = CacheConfig::default();
        let parser_cache = ParserCache::new(config);
        
        // 缓存关键字
        parser_cache.cache_keyword_type("MATCH", TokenType::Keyword);
        
        // 检查缓存命中
        assert_eq!(parser_cache.get_keyword_type("MATCH"), Some(TokenType::Keyword));
        assert_eq!(parser_cache.get_keyword_type("match"), Some(TokenType::Keyword)); // 大小写不敏感
    }

    #[test]
    fn test_token_prefetching() {
        let config = CacheConfig::default();
        let parser_cache = ParserCache::new(config);
        
        let tokens = vec![
            Token { token_type: TokenType::Keyword, value: "MATCH".to_string(), position: 0 },
            Token { token_type: TokenType::Identifier, value: "n".to_string(), position: 5 },
            Token { token_type: TokenType::Punctuation, value: "(".to_string(), position: 7 },
        ];
        
        // 预取标记
        parser_cache.prefetch_tokens(&tokens, 0);
        
        // 检查预取的标记
        assert!(parser_cache.get_prefetched_token(0).is_some());
        assert!(parser_cache.get_prefetched_token(1).is_some());
        assert!(parser_cache.get_prefetched_token(2).is_some());
    }

    #[test]
    fn test_parser_cache_stats() {
        let config = CacheConfig::default();
        let parser_cache = ParserCache::new(config);
        
        let stats = parser_cache.get_stats();
        assert_eq!(stats.total_hits(), 0);
        assert_eq!(stats.total_misses(), 0);
        assert_eq!(stats.overall_hit_rate(), 0.0);
    }

    #[test]
    fn test_keyword_cache_helper() {
        let cache = Arc::new(ConcurrentLruCache::new(100));
        let keyword_cache = KeywordCache::new(cache, true);
        
        assert!(!keyword_cache.is_keyword("UNKNOWN"));
        
        keyword_cache.cache_keyword("MATCH", true);
        assert!(keyword_cache.is_keyword("MATCH"));
        assert!(keyword_cache.is_keyword("match")); // 大小写不敏感
    }

    #[test]
    fn test_expression_cache_helper() {
        let cache = Arc::new(ConcurrentLruCache::new(100));
        let expression_cache = ExpressionCache::new(cache, true);
        
        // 测试 get_or_compute
        let expr = expression_cache.get_or_compute("n.name", || {
            // 返回一个真实的常量表达式
            Expr::Constant(crate::query::parser::ast::ConstantExpr::new(
                crate::core::Value::String("test".to_string()),
                crate::query::parser::ast::Span::default(),
            ))
        });
        
        // 第二次调用应该从缓存获取
        let cached_expr = expression_cache.get_or_compute("n.name", || {
            panic!("不应该调用这个函数，因为应该从缓存获取");
        });
        
        // 验证结果相同（这里简化了比较）
        assert!(expression_cache.get_expression("n.name").is_some());
    }
}