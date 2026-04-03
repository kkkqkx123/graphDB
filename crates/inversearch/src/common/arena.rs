//! Arena 分配器模块
//!
//! 提供高效的内存分配，适用于临时数据结构和批量操作

use bumpalo::Bump;

/// Arena 分配器包装器
pub struct Arena {
    inner: Bump,
}

impl Arena {
    /// 创建新的 Arena，默认容量
    pub fn new() -> Self {
        Self { inner: Bump::new() }
    }

    /// 创建新的 Arena，指定初始容量
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Bump::with_capacity(capacity),
        }
    }

    /// 分配一个值到 Arena
    pub fn alloc<T>(&self, value: T) -> &mut T {
        self.inner.alloc(value)
    }

    /// 分配一个切片到 Arena
    pub fn alloc_slice<T: Copy>(&self, slice: &[T]) -> &mut [T] {
        self.inner.alloc_slice_copy(slice)
    }

    /// 分配一个字符串到 Arena
    pub fn alloc_str(&self, s: &str) -> &mut str {
        self.inner.alloc_str(s)
    }

    /// 获取已分配的内存大小
    pub fn allocated_bytes(&self) -> usize {
        self.inner.allocated_bytes()
    }

    /// 重置 Arena，释放所有分配
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// 获取底层 Bump 分配器
    pub fn inner(&self) -> &Bump {
        &self.inner
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

/// 基于 Arena 的字符串分词器
pub struct ArenaTokenizer<'a> {
    arena: &'a Bump,
}

impl<'a> ArenaTokenizer<'a> {
    /// 创建新的 Arena 分词器
    pub fn new(arena: &'a Bump) -> Self {
        Self { arena }
    }

    /// 分词并存储到 Arena
    pub fn tokenize(&self, text: &str) -> Vec<&'a str> {
        let mut tokens = Vec::new();

        for word in text.split_whitespace() {
            let token: &'a str = self.arena.alloc_str(word);
            tokens.push(token);
        }

        tokens
    }

    /// 分词并存储到 Arena（带分隔符）
    pub fn tokenize_with_sep(&self, text: &str, sep: char) -> Vec<&'a str> {
        let mut tokens = Vec::new();

        for word in text.split(sep) {
            if !word.is_empty() {
                let token: &'a str = self.arena.alloc_str(word);
                tokens.push(token);
            }
        }

        tokens
    }
}

/// 基于 Arena 的临时向量
pub struct ArenaVec<'a, T> {
    arena: &'a Bump,
    items: Vec<&'a mut T>,
}

impl<'a, T> ArenaVec<'a, T> {
    /// 创建新的 Arena 向量
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            items: Vec::new(),
        }
    }

    /// 添加元素
    pub fn push(&mut self, value: T) {
        let item = self.arena.alloc(value);
        self.items.push(item);
    }

    /// 获取长度
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 获取元素
    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).map(|item| &**item)
    }

    /// 遍历元素
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(|item| &**item)
    }
}

/// 基于 Arena 的字符串构建器
pub struct ArenaStringBuilder<'a> {
    arena: &'a Bump,
    buffer: Vec<&'a str>,
}

impl<'a> ArenaStringBuilder<'a> {
    /// 创建新的字符串构建器
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            buffer: Vec::new(),
        }
    }

    /// 追加字符串片段
    pub fn append(&mut self, s: &str) {
        let slice = self.arena.alloc_str(s);
        self.buffer.push(slice);
    }

    /// 构建最终字符串（需要复制到堆）
    pub fn build(&self) -> String {
        self.buffer.concat()
    }

    /// 获取所有片段
    pub fn slices(&self) -> &[&'a str] {
        &self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic() {
        let mut arena = Arena::new();

        // 分配一些值
        let a = arena.alloc(42);
        let b = arena.alloc("hello");

        assert_eq!(*a, 42);
        assert_eq!(*b, "hello");

        // 重置后应该可以重新分配
        arena.reset();
        let c = arena.alloc(100);
        assert_eq!(*c, 100);
    }

    #[test]
    fn test_arena_tokenizer() {
        let arena = Bump::new();
        let tokenizer = ArenaTokenizer::new(&arena);

        let tokens = tokenizer.tokenize("hello world rust");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], "hello");
        assert_eq!(tokens[1], "world");
        assert_eq!(tokens[2], "rust");
    }

    #[test]
    fn test_arena_vec() {
        let arena = Bump::new();
        let mut vec = ArenaVec::new(&arena);

        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(0), Some(&1));
        assert_eq!(vec.get(1), Some(&2));
        assert_eq!(vec.get(2), Some(&3));
    }

    #[test]
    fn test_arena_string_builder() {
        let arena = Bump::new();
        let mut builder = ArenaStringBuilder::new(&arena);

        builder.append("hello");
        builder.append(" ");
        builder.append("world");

        assert_eq!(builder.build(), "hello world");
    }
}
