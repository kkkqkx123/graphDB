// 工具模块 - 仅用于导出各个子模块，不包含具体实现

// 对象池模块
pub mod object_pool;
pub use object_pool::ObjectPool;

// LRU缓存模块
pub mod lru_cache;
pub use lru_cache::LruCache;

// ID工具模块
pub mod id_utils;
pub use id_utils::{generate_id, is_valid_id};

// 日志模块
pub mod logger;
pub use logger::Logger;

// 键值构建器模块
pub mod kv_builder;
pub use kv_builder::{
    build_key_value_map,
    merge_key_value_maps,
    to_pairs,
    from_keys_and_values
};

// 字符串工具模块
pub mod string_utils;
pub use string_utils::{
    escape_for_query,
    unescape_for_query,
    normalize_identifier,
    sanitize_input
};

// 类型转换工具模块
pub mod type_utils;
pub use type_utils::{
    value_to_bool,
    value_to_i64,
    value_to_f64,
    value_to_string
};