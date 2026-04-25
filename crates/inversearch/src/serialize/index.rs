//! Index Serialization Implementation Module
//!
//! Provide Index-type import/export functionality.

use crate::error::Result;
use crate::serialize::compression;
use crate::serialize::format;
use crate::serialize::types::*;
use crate::Index;
use std::collections::HashMap;

impl Index {
    /// Exporting Indexed Data
    pub fn export(&self, _config: &SerializeConfig) -> Result<IndexExportData> {
        let index_info = IndexInfo {
            resolution: self.resolution,
            resolution_ctx: self.resolution_ctx,
            tokenize_mode: format!("{:?}", self.tokenize).to_lowercase(),
            depth: self.depth,
            bidirectional: self.bidirectional,
            fastupdate: self.fastupdate,
            rtl: self.rtl,
            encoder_type: "default".to_string(),
        };

        let main_index = self.export_main_index();
        let context_index = self.export_context_index();
        let registry = self.export_registry();

        let config_export = IndexConfigExport {
            index_options: self.get_index_options(),
            encoder_options: self.encoder.get_options(),
            tokenizer_config: self.get_tokenizer_config(),
        };

        let data = ExportData {
            main_index,
            context_index,
            registry,
        };

        Ok(IndexExportData {
            version: "0.1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            index_info,
            config: config_export,
            data,
        })
    }

    /// Exporting the Primary Index
    fn export_main_index(&self) -> HashMap<String, Vec<u64>> {
        let mut result = HashMap::new();

        for doc_ids in self.map.index.values() {
            for (term_str, ids) in doc_ids {
                result.insert(term_str.clone(), ids.clone());
            }
        }

        result
    }

    /// Exporting Context Indexes
    fn export_context_index(&self) -> HashMap<String, HashMap<String, Vec<u64>>> {
        let mut result = HashMap::new();

        for ctx_map in self.ctx.index.values() {
            let mut ctx_data = HashMap::new();
            for (ctx_term, doc_ids) in ctx_map {
                ctx_data.insert(ctx_term.clone(), doc_ids.clone());
            }
            let ctx_key_str = format!("ctx_{}", result.len());
            result.insert(ctx_key_str, ctx_data);
        }

        result
    }

    /// Exporting the registry
    fn export_registry(&self) -> RegistryData {
        match &self.reg {
            crate::index::Register::Set(set) => {
                let mut doc_ids = Vec::new();
                for set_data in set.index.values() {
                    for &doc_id in set_data {
                        doc_ids.push(doc_id);
                    }
                }
                RegistryData::Set(doc_ids)
            }
            crate::index::Register::Map(map) => {
                let mut result = HashMap::new();
                for map_data in map.index.values() {
                    for (&doc_id, refs) in map_data {
                        let ref_data: Vec<IndexRefData> =
                            refs.iter().map(IndexRefData::from_index_ref).collect();
                        result.insert(doc_id, ref_data);
                    }
                }
                RegistryData::Map(result)
            }
        }
    }

    /// Importing Indexed Data
    pub fn import(&mut self, data: IndexExportData, _config: &SerializeConfig) -> Result<()> {
        // Verify version compatibility
        if data.version != "0.1.0" {
            return Err(crate::error::InversearchError::Serialization(format!(
                "Unsupported version: {}",
                data.version
            )));
        }

        // Configuration of the application import
        self.apply_config(&data.config)?;

        // Clear the current index
        self.clear();

        // Importing the Primary Index
        self.import_main_index(&data.data.main_index)?;

        // Importing Context Indexes
        self.import_context_index(&data.data.context_index)?;

        // Importing the registry
        self.import_registry(&data.data.registry)?;

        Ok(())
    }

    /// Importing the Primary Index
    pub fn import_main_index(&mut self, data: &HashMap<String, Vec<u64>>) -> Result<()> {
        for (term, doc_ids) in data {
            let term_hash = self.keystore_hash_str(term);
            self.map.index.insert(term_hash, HashMap::new());
            if let Some(map) = self.map.index.get_mut(&term_hash) {
                map.insert(term.clone(), doc_ids.clone());
            }
        }
        Ok(())
    }

    /// Importing Context Indexes
    pub fn import_context_index(
        &mut self,
        data: &HashMap<String, HashMap<String, Vec<u64>>>,
    ) -> Result<()> {
        for (ctx_key, ctx_data) in data {
            let ctx_hash = self.keystore_hash_str(ctx_key);
            self.ctx.index.insert(ctx_hash, HashMap::new());
            if let Some(ctx_map) = self.ctx.index.get_mut(&ctx_hash) {
                for (term, doc_ids) in ctx_data {
                    ctx_map.insert(term.clone(), doc_ids.clone());
                }
            }
        }
        Ok(())
    }

    /// Importing the registry
    pub fn import_registry(&mut self, data: &RegistryData) -> Result<()> {
        match data {
            RegistryData::Set(doc_ids) => {
                let mut items_to_insert = Vec::new();
                for &doc_id in doc_ids {
                    let doc_hash = self.keystore_hash(&doc_id.to_string());
                    items_to_insert.push((doc_hash, doc_id));
                }

                if let crate::index::Register::Set(set) = &mut self.reg {
                    for (doc_hash, doc_id) in items_to_insert {
                        set.index
                            .entry(doc_hash)
                            .or_insert_with(std::collections::HashSet::new);
                        if let Some(set_data) = set.index.get_mut(&doc_hash) {
                            set_data.insert(doc_id);
                        }
                    }
                }
            }
            RegistryData::Map(doc_map) => {
                let mut items_to_insert = Vec::new();
                for (&doc_id, refs) in doc_map {
                    let doc_hash = self.keystore_hash(&doc_id.to_string());
                    let index_refs: Vec<crate::index::IndexRef> =
                        refs.iter().map(|r| r.to_index_ref()).collect();
                    items_to_insert.push((doc_hash, doc_id, index_refs));
                }

                if let crate::index::Register::Map(map) = &mut self.reg {
                    for (doc_hash, doc_id, index_refs) in items_to_insert {
                        map.index.entry(doc_hash).or_insert_with(HashMap::new);
                        if let Some(map_data) = map.index.get_mut(&doc_hash) {
                            map_data.insert(doc_id, index_refs);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Serialize to JSON string
    pub fn to_json(&self, config: &SerializeConfig) -> Result<String> {
        let data = self.export(config)?;
        format::to_json_string(&data)
    }

    /// Deserialization from JSON strings
    pub fn from_json(json_str: &str, config: &SerializeConfig) -> Result<Index> {
        let data = format::from_json_str(json_str)?;

        let mut index = Index::new(crate::index::IndexOptions::default())?;
        index.import(data, config)?;

        Ok(index)
    }

    /// Serialization to binary format
    pub fn to_binary(&self, config: &SerializeConfig) -> Result<Vec<u8>> {
        let data = self.export(config)?;

        let serialized = format::serialize_to_bytes(&data, &config.format)?;

        if config.compression {
            compression::compress_data(
                &serialized,
                config.compression_algorithm,
                config.compression_level,
            )
        } else {
            Ok(serialized)
        }
    }

    /// Deserialization from binary format
    pub fn from_binary(binary_data: &[u8], config: &SerializeConfig) -> Result<Index> {
        let decompressed = if config.compression {
            compression::decompress_data(binary_data, config.compression_algorithm)?
        } else {
            binary_data.to_vec()
        };

        let data = format::deserialize_from_bytes(&decompressed, &config.format)?;

        let mut index = Index::new(crate::index::IndexOptions::default())?;
        index.import(data, config)?;

        Ok(index)
    }

    /// Serialization to compressed binary format (convenience method)
    pub fn to_binary_compressed(&self, level: i32) -> Result<Vec<u8>> {
        let config = SerializeConfig::with_compression(CompressionAlgorithm::Zstd, level);
        self.to_binary(&config)
    }

    /// Deserialization from compressed binary format (convenience method)
    pub fn from_binary_compressed(binary_data: &[u8]) -> Result<Index> {
        let config = SerializeConfig::with_compression(CompressionAlgorithm::Zstd, 3);
        Self::from_binary(binary_data, &config)
    }

    /// Get Indexing Options
    fn get_index_options(&self) -> crate::r#type::IndexOptions {
        crate::r#type::IndexOptions {
            preset: None,
            context: Some(crate::r#type::ContextOptions {
                depth: Some(self.depth),
                bidirectional: Some(self.bidirectional),
                resolution: Some(self.resolution_ctx),
            }),
            encoder: None,
            resolution: Some(self.resolution),
            tokenize: Some(format!("{:?}", self.tokenize).to_lowercase()),
            fastupdate: Some(self.fastupdate),
            keystore: None,
            rtl: Some(self.rtl),
            cache: None,
            commit: None,
            priority: None,
        }
    }

    /// Get Splitter Configuration
    fn get_tokenizer_config(&self) -> TokenizerConfig {
        TokenizerConfig {
            mode: format!("{:?}", self.tokenize).to_lowercase(),
            separator: None,
            normalize: true,
        }
    }

    /// Configuration of the application import
    /// Applying Configuration to Indexes
    pub fn apply_config(&mut self, config: &IndexConfigExport) -> Result<()> {
        self.resolution = config.index_options.resolution.unwrap_or(9);
        self.resolution_ctx = config
            .index_options
            .context
            .as_ref()
            .and_then(|c| c.resolution)
            .unwrap_or(self.resolution);
        self.depth = config
            .index_options
            .context
            .as_ref()
            .and_then(|c| c.depth)
            .unwrap_or(0);
        self.bidirectional = config
            .index_options
            .context
            .as_ref()
            .and_then(|c| c.bidirectional)
            .unwrap_or(false);
        self.fastupdate = config.index_options.fastupdate.unwrap_or(false);
        self.rtl = config.index_options.rtl.unwrap_or(false);

        self.tokenize = match config.tokenizer_config.mode.as_str() {
            "strict" => crate::index::TokenizeMode::Strict,
            "forward" => crate::index::TokenizeMode::Forward,
            "reverse" => crate::index::TokenizeMode::Reverse,
            "full" => crate::index::TokenizeMode::Full,
            _ => crate::index::TokenizeMode::Strict,
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_import_json() {
        let mut original_index = Index::default();
        original_index
            .add(1, "hello world", false)
            .expect("add should succeed");
        original_index
            .add(2, "rust programming", false)
            .expect("add should succeed");
        original_index
            .add(3, "hello rust", false)
            .expect("add should succeed");

        let config = SerializeConfig::default();
        let json_str = original_index
            .to_json(&config)
            .expect("to_json should succeed");

        let imported_index =
            Index::from_json(&json_str, &config).expect("from_json should succeed");

        let results = imported_index
            .search_simple("hello")
            .expect("search should succeed");
        assert!(results.contains(&1));
        assert!(results.contains(&3));
        assert!(!results.contains(&2));
    }

    #[test]
    fn test_export_import_binary() {
        let mut original_index = Index::default();
        original_index
            .add(1, "test document", false)
            .expect("add should succeed");
        original_index
            .add(2, "another test", false)
            .expect("add should succeed");

        let config = SerializeConfig::default();
        let binary_data = original_index
            .to_binary(&config)
            .expect("to_binary should succeed");

        let imported_index =
            Index::from_binary(&binary_data, &config).expect("from_binary should succeed");

        let results = imported_index
            .search_simple("test")
            .expect("search should succeed");
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
    }

    #[test]
    fn test_serialize_format_variants() {
        let mut index = Index::default();
        index
            .add(1, "test document", false)
            .expect("add should succeed");
        index
            .add(2, "another test", false)
            .expect("add should succeed");

        let json_config = SerializeConfig {
            format: SerializeFormat::Json,
            compression: false,
            ..SerializeConfig::default()
        };
        let json_data = index
            .to_binary(&json_config)
            .expect("to_binary with JSON should succeed");
        let imported_json = Index::from_binary(&json_data, &json_config)
            .expect("from_binary with JSON should succeed");
        let results = imported_json
            .search_simple("test")
            .expect("search should succeed");
        assert_eq!(results.len(), 2);

        let mp_config = SerializeConfig {
            format: SerializeFormat::MessagePack,
            compression: false,
            ..SerializeConfig::default()
        };
        let mp_data = index
            .to_binary(&mp_config)
            .expect("to_binary with MessagePack should succeed");
        let imported_mp = Index::from_binary(&mp_data, &mp_config)
            .expect("from_binary with MessagePack should succeed");
        let results = imported_mp
            .search_simple("test")
            .expect("search should succeed");
        assert_eq!(results.len(), 2);

        let cbor_config = SerializeConfig {
            format: SerializeFormat::Cbor,
            compression: false,
            ..SerializeConfig::default()
        };
        let cbor_data = index
            .to_binary(&cbor_config)
            .expect("to_binary with CBOR should succeed");
        let imported_cbor = Index::from_binary(&cbor_data, &cbor_config)
            .expect("from_binary with CBOR should succeed");
        let results = imported_cbor
            .search_simple("test")
            .expect("search should succeed");
        assert_eq!(results.len(), 2);
    }
}
