//! 解析器特化缓存
//!
//! 为解析器提供专门的缓存功能

use super::cache_impl::*;
use super::config::*;
use super::manager::CacheManager;
use super::stats_marker::StatsEnabled;
use super::traits::*;
use crate::query::parser::ast::expr::Expr;
use crate::query::parser::lexer::{Token, TokenKind};
use std::sync::Arc;

// 定义统计缓存类型 - 统一使用 StatsEnabled 版本
type KeywordStatsType =
    Arc<StatsCacheWrapper<String, TokenKind, ConcurrentLruCache<String, TokenKind>, StatsEnabled>>;
type TokenStatsType =
    Arc<StatsCacheWrapper<usize, Token, ConcurrentLruCache<usize, Token>, StatsEnabled>>;
type ExpressionStatsType =
    Arc<StatsCacheWrapper<String, Expr, ConcurrentTtlCache<String, Expr>, StatsEnabled>>;
type PatternStatsType =
    Arc<StatsCacheWrapper<String, Pattern, ConcurrentTtlCache<String, Pattern>, StatsEnabled>>;

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
///
/// 所有缓存统一使用 StatsEnabled 版本，提供完整的统计功能
/// 移除了条件分发，每次缓存访问零分支预测失败
#[derive(Debug)]
pub struct ParserCache {
    manager: Arc<CacheManager>,

    // 统计缓存 - 统一使用 StatsEnabled 版本
    keyword_cache: KeywordStatsType,
    token_cache: TokenStatsType,
    expression_cache: ExpressionStatsType,
    pattern_cache: PatternStatsType,

    config: ParserCacheConfig,
}

impl ParserCache {
    /// 创建新的解析器缓存
    ///
    /// 所有缓存都创建为 StatsEnabled 版本，提供完整的统计功能
    pub fn new(config: CacheConfig) -> Self {
        let manager = Arc::new(CacheManager::new(config.clone()));
        let parser_config = config.parser_cache.clone();

        // 创建关键字缓存
        let keyword_cache = manager.create_lru_cache(parser_config.keyword_cache_capacity);
        let keyword_cache = manager.create_stats_cache(keyword_cache);

        // 创建标记缓存
        let token_cache = manager.create_lru_cache(parser_config.token_cache_capacity);
        let token_cache = manager.create_stats_cache(token_cache);

        // 创建表达式缓存
        let expression_cache = manager.create_ttl_cache(
            parser_config.expression_cache_capacity,
            parser_config.expression_cache_ttl,
        );
        let expression_cache = manager.create_stats_cache(expression_cache);

        // 创建模式缓存
        let pattern_cache = manager.create_ttl_cache(
            parser_config.pattern_cache_capacity,
            parser_config.pattern_cache_ttl,
        );
        let pattern_cache = manager.create_stats_cache(pattern_cache);

        Self {
            manager,
            keyword_cache,
            token_cache,
            expression_cache,
            pattern_cache,
            config: parser_config,
        }
    }

    /// 获取缓存的关键字类型
    pub fn get_keyword_type(&self, word: &str) -> Option<TokenKind> {
        let key = word.to_uppercase();
        self.keyword_cache.get(&key)
    }

    /// 缓存关键字类型
    pub fn cache_keyword_type(&self, word: &str, token_type: TokenKind) {
        let key = word.to_uppercase();
        self.keyword_cache.put(key, token_type);
    }

    /// 获取缓存的预取标记
    pub fn get_prefetched_token(&self, position: usize) -> Option<Token> {
        self.token_cache.get(&position)
    }

    /// 缓存预取的标记
    pub fn cache_prefetched_token(&self, position: usize, token: Token) {
        self.token_cache.put(position, token);
    }

    /// 批量预取标记
    pub fn prefetch_tokens(&self, tokens: &[Token], start_position: usize) {
        if self.config.prefetch_window == 0 {
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

    /// 获取缓存的解析表达式
    pub fn get_parsed_expression(&self, expr_str: &str) -> Option<Expr> {
        self.expression_cache.get(&expr_str.to_string())
    }

    /// 缓存解析的表达式
    pub fn cache_parsed_expression(&self, expr_str: &str, expression: Expr) {
        self.expression_cache.put(expr_str.to_string(), expression);
    }

    /// 获取缓存的解析模式
    pub fn get_parsed_pattern(&self, pattern_str: &str) -> Option<Pattern> {
        self.pattern_cache.get(&pattern_str.to_string())
    }

    /// 缓存解析的模式
    pub fn cache_parsed_pattern(&self, pattern_str: &str, pattern: Pattern) {
        self.pattern_cache.put(pattern_str.to_string(), pattern);
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> ParserCacheStats {
        ParserCacheStats {
            keyword_hits: self.keyword_cache.hits(),
            keyword_misses: self.keyword_cache.misses(),
            keyword_size: self.keyword_cache.len(),

            token_hits: self.token_cache.hits(),
            token_misses: self.token_cache.misses(),
            token_size: self.token_cache.len(),

            expression_hits: self.expression_cache.hits(),
            expression_misses: self.expression_cache.misses(),
            expression_size: self.expression_cache.len(),

            pattern_hits: self.pattern_cache.hits(),
            pattern_misses: self.pattern_cache.misses(),
            pattern_size: self.pattern_cache.len(),
        }
    }

    /// 重置所有统计信息
    pub fn reset_stats(&self) {
        self.keyword_cache.reset_stats();
        self.token_cache.reset_stats();
        self.expression_cache.reset_stats();
        self.pattern_cache.reset_stats();
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
    cache: KeywordStatsType,
}

impl KeywordCache {
    /// 创建关键字缓存助手
    ///
    /// 内部使用 StatsEnabled 版本的缓存
    pub fn new(cache: KeywordStatsType) -> Self {
        Self { cache }
    }

    /// 检查是否是关键字
    pub fn is_keyword(&self, word: &str) -> bool {
        let key = word.to_uppercase();
        self.cache.get(&key).is_some()
    }

    /// 缓存关键字
    pub fn cache_keyword(&self, word: &str, is_keyword: bool) {
        if is_keyword {
            let key = word.to_uppercase();
            // 使用 Match 作为通用关键字标记
            self.cache.put(key, TokenKind::Match);
        }
    }

    /// 获取统计信息（命中数、未命中数、命中率）
    pub fn get_stats(&self) -> (u64, u64, f64) {
        (
            self.cache.hits(),
            self.cache.misses(),
            self.cache.hit_rate(),
        )
    }
}

/// 表达式缓存助手
pub struct ExpressionCache {
    cache: ExpressionStatsType,
}

impl ExpressionCache {
    /// 创建表达式缓存助手
    ///
    /// 内部使用 StatsEnabled 版本的缓存
    pub fn new(cache: ExpressionStatsType) -> Self {
        Self { cache }
    }

    /// 获取缓存的表达式
    pub fn get_expression(&self, expr_str: &str) -> Option<Expr> {
        self.cache.get(&expr_str.to_string())
    }

    /// 缓存表达式
    pub fn cache_expression(&self, expr_str: &str, expression: Expr) {
        self.cache.put(expr_str.to_string(), expression);
    }

    /// 获取或计算表达式
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

    /// 获取统计信息（命中数、未命中数、命中率）
    pub fn get_stats(&self) -> (u64, u64, f64) {
        (
            self.cache.hits(),
            self.cache.misses(),
            self.cache.hit_rate(),
        )
    }
}

/// 模式缓存助手
pub struct PatternCache {
    cache: PatternStatsType,
}

impl PatternCache {
    /// 创建模式缓存助手
    ///
    /// 内部使用 StatsEnabled 版本的缓存
    pub fn new(cache: PatternStatsType) -> Self {
        Self { cache }
    }

    /// 获取缓存的模式
    pub fn get_pattern(&self, pattern_str: &str) -> Option<Pattern> {
        self.cache.get(&pattern_str.to_string())
    }

    /// 缓存模式
    pub fn cache_pattern(&self, pattern_str: &str, pattern: Pattern) {
        self.cache.put(pattern_str.to_string(), pattern);
    }

    /// 获取或计算模式
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

    /// 获取统计信息（命中数、未命中数、命中率）
    pub fn get_stats(&self) -> (u64, u64, f64) {
        (
            self.cache.hits(),
            self.cache.misses(),
            self.cache.hit_rate(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::lexer::{Token, TokenKind};
    use std::time::Duration;

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
        parser_cache.cache_keyword_type("MATCH", TokenKind::Match);

        // 检查缓存命中
        assert_eq!(
            parser_cache.get_keyword_type("MATCH"),
            Some(TokenKind::Match)
        );
        assert_eq!(
            parser_cache.get_keyword_type("match"),
            Some(TokenKind::Match)
        ); // 大小写不敏感
    }

    #[test]
    fn test_token_prefetching() {
        let config = CacheConfig::default();
        let parser_cache = ParserCache::new(config);

        let tokens = vec![
            Token {
                kind: TokenKind::Match,
                lexeme: "MATCH".to_string(),
                line: 0,
                column: 0,
            },
            Token {
                kind: TokenKind::Identifier("n".to_string()),
                lexeme: "n".to_string(),
                line: 0,
                column: 5,
            },
            Token {
                kind: TokenKind::LParen,
                lexeme: "(".to_string(),
                line: 0,
                column: 7,
            },
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
        let cache = Arc::new(StatsCacheWrapper::new_with_stats(cache));
        let keyword_cache = KeywordCache::new(cache);

        assert!(!keyword_cache.is_keyword("UNKNOWN"));

        keyword_cache.cache_keyword("MATCH", true);
        assert!(keyword_cache.is_keyword("MATCH"));
        assert!(keyword_cache.is_keyword("match")); // 大小写不敏感
    }

    #[test]
    fn test_expression_cache_helper() {
        let cache = Arc::new(ConcurrentTtlCache::new(100, Duration::from_secs(60)));
        let cache = Arc::new(StatsCacheWrapper::new_with_stats(cache));
        let expression_cache = ExpressionCache::new(cache);

        // 测试 get_or_compute
        let _expr = expression_cache.get_or_compute("n.name", || {
            // 返回一个真实的常量表达式
            Expr::Constant(crate::query::parser::ast::ConstantExpr::new(
                crate::core::Value::String("test".to_string()),
                crate::query::parser::ast::Span::default(),
            ))
        });

        // 第二次调用应该从缓存获取
        let _cached_expr = expression_cache.get_or_compute("n.name", || {
            panic!("不应该调用这个函数，因为应该从缓存获取");
        });

        // 验证结果相同（这里简化了比较）
        assert!(expression_cache.get_expression("n.name").is_some());
    }
}
