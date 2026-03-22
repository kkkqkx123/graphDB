pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bm25_service=debug,tower_http=debug".into()),
        )
        .init();
}

pub fn init_metrics() {
    metrics::describe_counter!(
        "bm25_search_requests_total",
        "Total number of search requests"
    );
    metrics::describe_counter!(
        "bm25_index_documents_total",
        "Total number of indexed documents"
    );
    metrics::describe_counter!(
        "bm25_cache_hits",
        "Total number of cache hits"
    );
    metrics::describe_counter!(
        "bm25_cache_misses",
        "Total number of cache misses"
    );
    metrics::describe_histogram!(
        "bm25_search_duration_seconds",
        "Search request duration"
    );
    metrics::describe_histogram!(
        "bm25_index_duration_seconds",
        "Index operation duration"
    );
}
