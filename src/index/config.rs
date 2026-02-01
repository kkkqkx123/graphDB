#[derive(Clone, Debug)]
pub struct IndexServiceConfig {
    pub max_memory_bytes: u64,
    pub enable_auto_cleanup: bool,
    pub cleanup_interval_secs: u64,
    pub exact_lookup_cache_size: usize,
    pub enable_cache_stats: bool,
    pub cache_ttl_secs: u64,
}

impl Default for IndexServiceConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1024 * 1024 * 1024,
            enable_auto_cleanup: true,
            cleanup_interval_secs: 300,
            exact_lookup_cache_size: 10000,
            enable_cache_stats: true,
            cache_ttl_secs: 3600,
        }
    }
}
