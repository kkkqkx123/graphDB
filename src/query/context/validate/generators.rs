//! 匿名生成器模块
//! 提供匿名变量和列的生成功能

/// 匿名变量生成器
#[derive(Debug)]
pub struct AnonVarGenerator {
    counter: std::sync::atomic::AtomicU64,
    prefix: String,
}

impl Clone for AnonVarGenerator {
    fn clone(&self) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(
                self.counter.load(std::sync::atomic::Ordering::Relaxed),
            ),
            prefix: self.prefix.clone(),
        }
    }
}

impl AnonVarGenerator {
    /// 创建新的匿名变量生成器
    pub fn new(prefix: String) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(0),
            prefix,
        }
    }

    /// 生成匿名变量名
    pub fn generate(&self) -> String {
        let count = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("{}_{}", self.prefix, count)
    }

    /// 重置计数器
    pub fn reset(&self) {
        self.counter.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// 获取当前计数器值
    pub fn current_count(&self) -> u64 {
        self.counter.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 设置计数器值
    pub fn set_count(&self, count: u64) {
        self.counter
            .store(count, std::sync::atomic::Ordering::Relaxed);
    }

    /// 检查生成的名称是否匹配模式
    pub fn matches_pattern(&self, name: &str) -> bool {
        name.starts_with(&self.prefix)
    }

    /// 从生成的名称中提取计数器值
    pub fn extract_count(&self, name: &str) -> Option<u64> {
        if !self.matches_pattern(name) {
            return None;
        }

        let parts: Vec<&str> = name.split('_').collect();
        if parts.len() < 2 {
            return None;
        }

        parts.last()?.parse().ok()
    }
}

/// 匿名列生成器
#[derive(Debug)]
pub struct AnonColGenerator {
    counter: std::sync::atomic::AtomicU64,
    prefix: String,
}

impl Clone for AnonColGenerator {
    fn clone(&self) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(
                self.counter.load(std::sync::atomic::Ordering::Relaxed),
            ),
            prefix: self.prefix.clone(),
        }
    }
}

impl AnonColGenerator {
    /// 创建新的匿名列生成器
    pub fn new(prefix: String) -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(0),
            prefix,
        }
    }

    /// 生成匿名列名
    pub fn generate(&self) -> String {
        let count = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("{}_{}", self.prefix, count)
    }

    /// 重置计数器
    pub fn reset(&self) {
        self.counter.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// 获取当前计数器值
    pub fn current_count(&self) -> u64 {
        self.counter.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 设置计数器值
    pub fn set_count(&self, count: u64) {
        self.counter
            .store(count, std::sync::atomic::Ordering::Relaxed);
    }

    /// 检查生成的名称是否匹配模式
    pub fn matches_pattern(&self, name: &str) -> bool {
        name.starts_with(&self.prefix)
    }

    /// 从生成的名称中提取计数器值
    pub fn extract_count(&self, name: &str) -> Option<u64> {
        if !self.matches_pattern(name) {
            return None;
        }

        let parts: Vec<&str> = name.split('_').collect();
        if parts.len() < 2 {
            return None;
        }

        parts.last()?.parse().ok()
    }
}

/// 生成器工厂
pub struct GeneratorFactory;

impl GeneratorFactory {
    /// 创建默认的匿名变量生成器
    pub fn create_anon_var_generator() -> AnonVarGenerator {
        AnonVarGenerator::new("__var".to_string())
    }

    /// 创建默认的匿名列生成器
    pub fn create_anon_col_generator() -> AnonColGenerator {
        AnonColGenerator::new("__col".to_string())
    }

    /// 创建自定义前缀的匿名变量生成器
    pub fn create_custom_var_generator(prefix: String) -> AnonVarGenerator {
        AnonVarGenerator::new(prefix)
    }

    /// 创建自定义前缀的匿名列生成器
    pub fn create_custom_col_generator(prefix: String) -> AnonColGenerator {
        AnonColGenerator::new(prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anon_var_generator() {
        let gen = AnonVarGenerator::new("__var".to_string());

        // 测试生成
        let var1 = gen.generate();
        let var2 = gen.generate();
        let var3 = gen.generate();

        assert_ne!(var1, var2);
        assert_ne!(var2, var3);
        assert!(var1.starts_with("__var_"));
        assert!(var2.starts_with("__var_"));
        assert!(var3.starts_with("__var_"));

        // 测试计数器
        assert_eq!(gen.current_count(), 3);

        // 测试重置
        gen.reset();
        assert_eq!(gen.current_count(), 0);

        let var4 = gen.generate();
        assert_eq!(var4, "__var_0");
        assert_eq!(gen.current_count(), 1);
    }

    #[test]
    fn test_anon_col_generator() {
        let gen = AnonColGenerator::new("__col".to_string());

        // 测试生成
        let col1 = gen.generate();
        let col2 = gen.generate();

        assert_ne!(col1, col2);
        assert!(col1.starts_with("__col_"));
        assert!(col2.starts_with("__col_"));

        // 测试设置计数器
        gen.set_count(10);
        assert_eq!(gen.current_count(), 10);

        let col3 = gen.generate();
        assert_eq!(col3, "__col_10");
        assert_eq!(gen.current_count(), 11);
    }

    #[test]
    fn test_generator_pattern_matching() {
        let gen = AnonVarGenerator::new("__test".to_string());

        // 测试模式匹配
        assert!(gen.matches_pattern("__test_0"));
        assert!(gen.matches_pattern("__test_123"));
        assert!(!gen.matches_pattern("__var_0"));
        assert!(!gen.matches_pattern("test_0"));

        // 测试计数器提取
        assert_eq!(gen.extract_count("__test_0"), Some(0));
        assert_eq!(gen.extract_count("__test_123"), Some(123));
        assert_eq!(gen.extract_count("__var_0"), None);
        assert_eq!(gen.extract_count("test_0"), None);
        assert_eq!(gen.extract_count("__test_invalid"), None);
    }

    #[test]
    fn test_generator_factory() {
        // 测试默认生成器
        let var_gen = GeneratorFactory::create_anon_var_generator();
        let col_gen = GeneratorFactory::create_anon_col_generator();

        let var = var_gen.generate();
        let col = col_gen.generate();

        assert!(var.starts_with("__var_"));
        assert!(col.starts_with("__col_"));

        // 测试自定义生成器
        let custom_var_gen = GeneratorFactory::create_custom_var_generator("custom".to_string());
        let custom_col_gen = GeneratorFactory::create_custom_col_generator("mycol".to_string());

        let custom_var = custom_var_gen.generate();
        let custom_col = custom_col_gen.generate();

        assert!(custom_var.starts_with("custom_"));
        assert!(custom_col.starts_with("mycol_"));
    }

    #[test]
    fn test_generator_concurrent_safety() {
        use std::sync::Arc;
        use std::thread;

        let gen = Arc::new(AnonVarGenerator::new("concurrent".to_string()));
        let mut handles = vec![];

        // 创建多个线程同时生成变量名
        for _ in 0..10 {
            let gen_clone = Arc::clone(&gen);
            let handle = thread::spawn(move || {
                let mut names = vec![];
                for _ in 0..5 {
                    names.push(gen_clone.generate());
                }
                names
            });
            handles.push(handle);
        }

        // 收集所有生成的名称
        let mut all_names = vec![];
        for handle in handles {
            let names = handle
                .join()
                .expect("Expected thread to complete successfully");
            all_names.extend(names);
        }

        // 验证所有名称都是唯一的
        let mut unique_names = all_names.clone();
        unique_names.sort();
        unique_names.dedup();

        assert_eq!(all_names.len(), unique_names.len());
        assert_eq!(all_names.len(), 50); // 10 threads * 5 names each

        // 验证所有名称都符合模式
        for name in &all_names {
            assert!(gen.matches_pattern(name));
        }
    }
}
