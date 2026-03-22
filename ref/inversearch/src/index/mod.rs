use crate::encoder::Encoder;
use crate::keystore::{KeystoreMap, KeystoreSet, KeystoreArray, DocId, ResolutionSlot};
use crate::tokenizer::Tokenizer;
use crate::error::{Result, InversearchError};
use crate::search::SearchCache;
use std::collections::HashMap;

pub mod builder;
pub mod remover;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenizeMode {
    Strict,
    Forward,
    Reverse,
    Full,
    Bidirectional,
}

// 定义对索引数组的引用枚举，包含定位信息
#[derive(Clone)]
pub enum IndexRef {
    MapRef(String),      // 普通索引的键
    CtxRef(String, String),      // 上下文索引的键 (keyword, term)
}

#[derive(Clone)]
pub enum Register {
    Set(KeystoreSet<DocId>),
    Map(KeystoreMap<DocId, Vec<IndexRef>>),
}

#[derive(Clone)]
pub struct Index {
    pub map: KeystoreMap<String, Vec<DocId>>,
    pub ctx: KeystoreMap<String, Vec<DocId>>,
    pub reg: Register,
    pub resolution: usize,
    pub resolution_ctx: usize,
    pub tokenize: TokenizeMode,
    pub depth: usize,
    pub bidirectional: bool,
    pub fastupdate: bool,
    pub score: Option<ScoreFn>,
    pub encoder: Encoder,
    pub rtl: bool,
    pub cache: Option<SearchCache>,
}

pub type ScoreFn = fn(&[u8], &str, usize, Option<usize>, Option<usize>) -> usize;

impl Index {
    pub fn new(options: IndexOptions) -> Result<Self> {
        let resolution = options.resolution.unwrap_or(9);
        let resolution_ctx = options.resolution_ctx.unwrap_or(resolution);
        let depth = options.depth.unwrap_or(0);
        let bidirectional = options.bidirectional.unwrap_or(false);
        let fastupdate = options.fastupdate.unwrap_or(false);
        let rtl = options.rtl.unwrap_or(false);
        
        let encoder = Encoder::new(options.encoder.unwrap_or_default())?;
        
        let tokenize = match options.tokenize_mode {
            Some("strict") => TokenizeMode::Strict,
            Some("forward") => TokenizeMode::Forward,
            Some("reverse") => TokenizeMode::Reverse,
            Some("full") => TokenizeMode::Full,
            _ => TokenizeMode::Strict,
        };

        let reg = if fastupdate {
            Register::Map(KeystoreMap::new(8))
        } else {
            Register::Set(KeystoreSet::<DocId>::new(8))
        };

        // 初始化缓存（可选）
        let cache = if options.cache_size.unwrap_or(0) > 0 {
            Some(SearchCache::new(options.cache_size.unwrap(), options.cache_ttl))
        } else {
            None
        };

        Ok(Index {
            map: KeystoreMap::new(8),
            ctx: KeystoreMap::new(8),
            reg,
            resolution,
            resolution_ctx,
            tokenize,
            depth,
            bidirectional,
            fastupdate,
            score: options.score,
            encoder,
            rtl,
            cache,
        })
    }

    pub fn add(&mut self, id: DocId, content: &str, append: bool) -> Result<()> {
        builder::add_document(self, id, content, append, false)
    }

    pub fn remove(&mut self, id: DocId, skip_deletion: bool) -> Result<()> {
        remover::remove_document(self, id, skip_deletion)
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.ctx.clear();
        match &mut self.reg {
            Register::Set(set) => set.clear(),
            Register::Map(map) => map.clear(),
        }
    }

    pub fn contains(&self, id: DocId) -> bool {
        match &self.reg {
            Register::Set(set) => set.has(&id),
            Register::Map(map) => map.has(&id),
        }
    }

    pub fn get_score(
        &self,
        resolution: usize,
        length: usize,
        i: usize,
        term_length: Option<usize>,
        x: Option<usize>,
    ) -> usize {
        if let Some(score_fn) = self.score {
            return score_fn(&[], "", i, None, x);
        }

        builder::get_score(resolution, length, i, term_length, x)
    }

    pub fn push_index(
        &mut self,
        dupes: &mut HashMap<String, bool>,
        term: &str,
        _score: usize,
        id: DocId,
        append: bool,
        keyword: Option<&str>,
    ) {
        let term_key = term.to_string();
        let has_dupe = dupes.get(&term_key).copied().unwrap_or(false);
        let has_keyword_dupe = if let Some(kw) = keyword {
            dupes.get(kw).copied().unwrap_or(false)
        } else {
            false
        };

        if !has_dupe || (keyword.is_some() && !has_keyword_dupe) {
            dupes.insert(term_key.clone(), true);
            if let Some(kw) = keyword {
                dupes.insert(kw.to_string(), true);
            }

            if let Some(kw) = keyword {
                let kw_key = kw.to_string();
                let kw_hash = self.keystore_hash_str(&kw_key);
                let doc_ids_vec = self.ctx.index
                    .entry(kw_hash)
                    .or_insert_with(HashMap::new)
                    .entry(term_key.clone())
                    .or_insert_with(Vec::new);

                if !append || !doc_ids_vec.contains(&id) {
                    doc_ids_vec.push(id);

                    if self.fastupdate {
                        let id_hash = self.keystore_hash_str(&id.to_string());
                        if let Register::Map(reg) = &mut self.reg {
                            let index_ref = IndexRef::CtxRef(kw_key, term_key.clone());

                            reg.index
                                .entry(id_hash)
                                .or_insert_with(HashMap::new)
                                .entry(id)
                                .or_insert_with(Vec::new)
                                .push(index_ref);
                        }
                    }
                }
            } else {
                let term_hash = self.keystore_hash_str(&term_key);
                let doc_ids_vec = self.map.index
                    .entry(term_hash)
                    .or_insert_with(HashMap::new)
                    .entry(term_key.clone())
                    .or_insert_with(Vec::new);

                if !append || !doc_ids_vec.contains(&id) {
                    doc_ids_vec.push(id);

                    if self.fastupdate {
                        let id_hash = self.keystore_hash_str(&id.to_string());
                        if let Register::Map(reg) = &mut self.reg {
                            let index_ref = IndexRef::MapRef(term_key.clone());

                            reg.index
                                .entry(id_hash)
                                .or_insert_with(HashMap::new)
                                .entry(id)
                                .or_insert_with(Vec::new)
                                .push(index_ref);
                        }
                    }
                }
            }
        }
    }

    pub fn keystore_hash(&self, id: &str) -> usize {
        let id_str = id.to_string();
        let mut crc: u32 = 0;
        for c in id_str.chars() {
            crc = (crc << 8) ^ (crc >> (32 - 8)) ^ (c as u32);
        }
        (crc as usize) % (1 << 8)
    }

    pub fn keystore_hash_str(&self, s: &str) -> usize {
        let mut crc: u32 = 0;
        for c in s.chars() {
            crc = (crc << 8) ^ (crc >> (32 - 8)) ^ (c as u32);
        }
        (crc as usize) % (1 << 8)
    }

    pub fn keystore_hash_static(id: &str) -> usize {
        let id_str = id.to_string();
        let mut crc: u32 = 0;
        for c in id_str.chars() {
            crc = (crc << 8) ^ (crc >> (32 - 8)) ^ (c as u32);
        }
        (crc as usize) % (1 << 8)
    }

    pub fn update(&mut self, id: DocId, content: &str) -> Result<()> {
        self.remove(id, false)?;
        self.add(id, content, false)
    }

    pub fn search(&self, options: &crate::r#type::SearchOptions) -> Result<crate::search::SearchResult> {
        crate::search::search(self, options)
    }

    /// 带缓存的搜索
    pub fn search_cached(&mut self, options: &crate::r#type::SearchOptions) -> Result<crate::search::SearchResult> {
        let query = options.query.as_deref().unwrap_or("");
        if query.is_empty() {
            return Ok(crate::search::SearchResult {
                results: Vec::new(),
                total: 0,
                query: String::new(),
            });
        }

        // 首先执行搜索
        let result = self.search(options)?;
        
        // 如果有缓存，缓存结果
        if let Some(ref mut cache) = self.cache {
            use crate::search::CacheKeyGenerator;
            let cache_key = CacheKeyGenerator::generate_search_key(query, options);
            cache.set(cache_key, result.results.clone());
        }
        
        Ok(result)
    }

    pub fn search_simple(&self, query: &str) -> Result<crate::r#type::SearchResults> {
        let options = crate::r#type::SearchOptions {
            query: Some(query.to_string()),
            ..Default::default()
        };
        let result = self.search(&options)?;
        Ok(result.results)
    }

    /// 获取缓存统计信息
    pub fn cache_stats(&self) -> Option<crate::search::CacheStats> {
        self.cache.as_ref().map(|cache| cache.stats())
    }

    /// 清空缓存
    pub fn clear_cache(&mut self) {
        if let Some(ref mut cache) = self.cache {
            cache.clear();
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexOptions {
    pub resolution: Option<usize>,
    pub resolution_ctx: Option<usize>,
    pub tokenize_mode: Option<&'static str>,
    pub depth: Option<usize>,
    pub bidirectional: Option<bool>,
    pub fastupdate: Option<bool>,
    pub score: Option<ScoreFn>,
    pub encoder: Option<crate::EncoderOptions>,
    pub rtl: Option<bool>,
    pub cache_size: Option<usize>,
    pub cache_ttl: Option<std::time::Duration>,
}

impl Default for IndexOptions {
    fn default() -> Self {
        IndexOptions {
            resolution: None,
            resolution_ctx: None,
            tokenize_mode: None,
            depth: None,
            bidirectional: None,
            fastupdate: None,
            score: None,
            encoder: None,
            rtl: None,
            cache_size: Some(1000),  // 默认启用缓存，大小1000
            cache_ttl: None,         // 默认无过期时间
        }
    }
}

impl Default for Index {
    fn default() -> Self {
        Index::new(IndexOptions::default()).unwrap()
    }
}
