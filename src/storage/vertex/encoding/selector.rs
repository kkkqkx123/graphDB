//! Compression Strategy Selector
//!
//! Automatically selects the optimal compression algorithm based on
//! data characteristics and access patterns.
//!
//! # Strategy
//!
//! 1. Analyze data statistics (cardinality, range, patterns)
//! 2. Consider access patterns (hot/cold data)
//! 3. Select encoding that balances compression ratio and query speed

use std::collections::HashMap;

use crate::core::{DataType, Value};

use super::EncodingType;

#[derive(Debug, Clone)]
pub struct ColumnStats {
    pub row_count: usize,
    pub null_count: usize,
    pub distinct_count: usize,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
    pub avg_length: f64,
    pub run_count: usize,
    pub access_count: usize,
    pub data_type: DataType,
}

impl Default for ColumnStats {
    fn default() -> Self {
        Self {
            row_count: 0,
            null_count: 0,
            distinct_count: 0,
            min_value: None,
            max_value: None,
            avg_length: 0.0,
            run_count: 0,
            access_count: 0,
            data_type: DataType::String,
        }
    }
}

impl ColumnStats {
    pub fn new(data_type: DataType) -> Self {
        Self {
            data_type,
            ..Default::default()
        }
    }

    pub fn null_ratio(&self) -> f64 {
        if self.row_count == 0 {
            return 0.0;
        }
        self.null_count as f64 / self.row_count as f64
    }

    pub fn cardinality_ratio(&self) -> f64 {
        if self.row_count == 0 {
            return 0.0;
        }
        self.distinct_count as f64 / self.row_count as f64
    }

    pub fn run_ratio(&self) -> f64 {
        if self.row_count == 0 {
            return 1.0;
        }
        self.run_count as f64 / self.row_count as f64
    }

    pub fn value_range(&self) -> Option<u64> {
        match (&self.min_value, &self.max_value) {
            (Some(Value::SmallInt(min)), Some(Value::SmallInt(max))) => {
                Some((*max - *min) as u64)
            }
            (Some(Value::Int(min)), Some(Value::Int(max))) => Some((*max - *min) as u64),
            (Some(Value::BigInt(min)), Some(Value::BigInt(max))) => Some((*max - *min) as u64),
            _ => None,
        }
    }

    pub fn is_hot(&self, threshold: usize) -> bool {
        self.access_count > threshold
    }

    pub fn is_cold(&self, threshold: usize) -> bool {
        self.access_count < threshold
    }
}

#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub hot_access_threshold: usize,
    pub cold_access_threshold: usize,
    pub min_rows_for_compression: usize,
    pub max_dictionary_size: usize,
    pub prefer_speed_over_ratio: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            hot_access_threshold: 1000,
            cold_access_threshold: 100,
            min_rows_for_compression: 100,
            max_dictionary_size: 10000,
            prefer_speed_over_ratio: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompressionSelector {
    config: CompressionConfig,
    stats_cache: HashMap<String, ColumnStats>,
}

impl CompressionSelector {
    pub fn new() -> Self {
        Self {
            config: CompressionConfig::default(),
            stats_cache: HashMap::new(),
        }
    }

    pub fn with_config(config: CompressionConfig) -> Self {
        Self {
            config,
            stats_cache: HashMap::new(),
        }
    }

    pub fn select(&self, stats: &ColumnStats) -> EncodingType {
        if stats.row_count < self.config.min_rows_for_compression {
            return EncodingType::None;
        }

        if stats.is_hot(self.config.hot_access_threshold) && self.config.prefer_speed_over_ratio {
            return EncodingType::None;
        }

        match stats.data_type {
            DataType::String => self.select_string_encoding(stats),
            DataType::Int | DataType::SmallInt | DataType::BigInt => {
                self.select_int_encoding(stats)
            }
            DataType::Float | DataType::Double => self.select_float_encoding(stats),
            DataType::Bool => self.select_bool_encoding(stats),
            _ => EncodingType::None,
        }
    }

    fn select_string_encoding(&self, stats: &ColumnStats) -> EncodingType {
        let cardinality_ratio = stats.cardinality_ratio();

        if cardinality_ratio < 0.5 && stats.distinct_count < self.config.max_dictionary_size {
            let estimated_dict_size = stats.distinct_count * stats.avg_length as usize
                + stats.row_count * 4;
            let estimated_raw_size = stats.row_count * stats.avg_length as usize;

            if estimated_dict_size < estimated_raw_size {
                return EncodingType::Dictionary;
            }
        }

        if stats.avg_length >= 20.0 && cardinality_ratio > 0.5 {
            return EncodingType::Fsst;
        }

        EncodingType::None
    }

    fn select_int_encoding(&self, stats: &ColumnStats) -> EncodingType {
        let run_ratio = stats.run_ratio();

        if run_ratio < 0.3 {
            return EncodingType::Rle;
        }

        if let Some(range) = stats.value_range() {
            let bit_width = if range == 0 { 1 } else { 64 - range.leading_zeros() as u8 };

            if bit_width < 32 {
                return EncodingType::BitPacking;
            }
        }

        EncodingType::None
    }

    fn select_float_encoding(&self, stats: &ColumnStats) -> EncodingType {
        if stats.row_count < self.config.min_rows_for_compression {
            return EncodingType::None;
        }

        if stats.is_hot(self.config.hot_access_threshold) {
            return EncodingType::None;
        }

        EncodingType::Alp
    }

    fn select_bool_encoding(&self, stats: &ColumnStats) -> EncodingType {
        if stats.run_ratio() < 0.5 {
            return EncodingType::Rle;
        }
        EncodingType::None
    }

    pub fn update_stats(&mut self, column_name: &str, stats: ColumnStats) {
        self.stats_cache.insert(column_name.to_string(), stats);
    }

    pub fn get_stats(&self, column_name: &str) -> Option<&ColumnStats> {
        self.stats_cache.get(column_name)
    }

    pub fn reevaluate(&self, column_name: &str) -> Option<EncodingType> {
        self.stats_cache
            .get(column_name)
            .map(|stats| self.select(stats))
    }

    pub fn reevaluate_all(&self) -> HashMap<String, EncodingType> {
        self.stats_cache
            .iter()
            .map(|(name, stats)| (name.clone(), self.select(stats)))
            .collect()
    }

    pub fn config(&self) -> &CompressionConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: CompressionConfig) {
        self.config = config;
    }
}

impl Default for CompressionSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataTemperature {
    Hot,
    Warm,
    Cold,
}

impl DataTemperature {
    pub fn from_access_count(count: usize, hot_threshold: usize, cold_threshold: usize) -> Self {
        if count >= hot_threshold {
            Self::Hot
        } else if count <= cold_threshold {
            Self::Cold
        } else {
            Self::Warm
        }
    }
}

#[derive(Debug, Clone)]
pub struct TieredCompressionStrategy {
    selector: CompressionSelector,
    tier_configs: HashMap<DataTemperature, TierConfig>,
}

#[derive(Debug, Clone)]
pub struct TierConfig {
    pub prefer_speed: bool,
    pub min_compression_ratio: f64,
    pub allowed_encodings: Vec<EncodingType>,
}

impl Default for TierConfig {
    fn default() -> Self {
        Self {
            prefer_speed: false,
            min_compression_ratio: 0.1,
            allowed_encodings: vec![
                EncodingType::None,
                EncodingType::Dictionary,
                EncodingType::Rle,
                EncodingType::BitPacking,
                EncodingType::Fsst,
                EncodingType::Alp,
            ],
        }
    }
}

impl TierConfig {
    pub fn hot() -> Self {
        Self {
            prefer_speed: true,
            min_compression_ratio: 0.3,
            allowed_encodings: vec![EncodingType::None, EncodingType::Rle],
        }
    }

    pub fn cold() -> Self {
        Self {
            prefer_speed: false,
            min_compression_ratio: 0.0,
            allowed_encodings: vec![
                EncodingType::Dictionary,
                EncodingType::Rle,
                EncodingType::BitPacking,
                EncodingType::Fsst,
                EncodingType::Alp,
            ],
        }
    }
}

impl TieredCompressionStrategy {
    pub fn new() -> Self {
        let mut tier_configs = HashMap::new();
        tier_configs.insert(DataTemperature::Hot, TierConfig::hot());
        tier_configs.insert(DataTemperature::Warm, TierConfig::default());
        tier_configs.insert(DataTemperature::Cold, TierConfig::cold());

        Self {
            selector: CompressionSelector::new(),
            tier_configs,
        }
    }

    pub fn select_for_tier(&self, stats: &ColumnStats, temperature: DataTemperature) -> EncodingType {
        let base_encoding = self.selector.select(stats);

        if let Some(tier_config) = self.tier_configs.get(&temperature) {
            if tier_config.allowed_encodings.contains(&base_encoding) {
                return base_encoding;
            }

            for encoding in &tier_config.allowed_encodings {
                if self.is_encoding_suitable(encoding, stats) {
                    return *encoding;
                }
            }
        }

        EncodingType::None
    }

    fn is_encoding_suitable(&self, encoding: &EncodingType, stats: &ColumnStats) -> bool {
        match encoding {
            EncodingType::Dictionary => {
                stats.cardinality_ratio() < 0.5
                    && stats.distinct_count < self.selector.config().max_dictionary_size
            }
            EncodingType::Rle => stats.run_ratio() < 0.3,
            EncodingType::BitPacking => {
                if let Some(range) = stats.value_range() {
                    let bit_width = if range == 0 { 1 } else { 64 - range.leading_zeros() as u8 };
                    bit_width < 32
                } else {
                    false
                }
            }
            EncodingType::Fsst => stats.avg_length >= 20.0 && stats.cardinality_ratio() > 0.5,
            EncodingType::Alp => {
                matches!(stats.data_type, DataType::Float | DataType::Double)
            }
            EncodingType::None => true,
        }
    }

    pub fn selector(&self) -> &CompressionSelector {
        &self.selector
    }

    pub fn selector_mut(&mut self) -> &mut CompressionSelector {
        &mut self.selector
    }

    pub fn tier_config(&self, temperature: DataTemperature) -> Option<&TierConfig> {
        self.tier_configs.get(&temperature)
    }

    pub fn set_tier_config(&mut self, temperature: DataTemperature, config: TierConfig) {
        self.tier_configs.insert(temperature, config);
    }
}

impl Default for TieredCompressionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_stats() {
        let stats = ColumnStats {
            row_count: 1000,
            null_count: 100,
            distinct_count: 50,
            data_type: DataType::String,
            ..Default::default()
        };

        assert!((stats.null_ratio() - 0.1).abs() < 1e-9);
        assert!((stats.cardinality_ratio() - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_compression_selector_string_dictionary() {
        let stats = ColumnStats {
            row_count: 1000,
            distinct_count: 50,
            avg_length: 20.0,
            data_type: DataType::String,
            ..Default::default()
        };

        let selector = CompressionSelector::new();
        let encoding = selector.select(&stats);

        assert_eq!(encoding, EncodingType::Dictionary);
    }

    #[test]
    fn test_compression_selector_string_fsst() {
        let stats = ColumnStats {
            row_count: 1000,
            distinct_count: 800,
            avg_length: 50.0,
            data_type: DataType::String,
            ..Default::default()
        };

        let selector = CompressionSelector::new();
        let encoding = selector.select(&stats);

        assert_eq!(encoding, EncodingType::Fsst);
    }

    #[test]
    fn test_compression_selector_int_rle() {
        let stats = ColumnStats {
            row_count: 1000,
            run_count: 100,
            data_type: DataType::Int,
            ..Default::default()
        };

        let selector = CompressionSelector::new();
        let encoding = selector.select(&stats);

        assert_eq!(encoding, EncodingType::Rle);
    }

    #[test]
    fn test_compression_selector_int_bitpacking() {
        let stats = ColumnStats {
            row_count: 1000,
            run_count: 800,
            min_value: Some(Value::Int(0)),
            max_value: Some(Value::Int(100)),
            data_type: DataType::Int,
            ..Default::default()
        };

        let selector = CompressionSelector::new();
        let encoding = selector.select(&stats);

        assert_eq!(encoding, EncodingType::BitPacking);
    }

    #[test]
    fn test_compression_selector_float_alp() {
        let stats = ColumnStats {
            row_count: 1000,
            access_count: 50,
            data_type: DataType::Double,
            ..Default::default()
        };

        let selector = CompressionSelector::new();
        let encoding = selector.select(&stats);

        assert_eq!(encoding, EncodingType::Alp);
    }

    #[test]
    fn test_tiered_strategy_hot() {
        let stats = ColumnStats {
            row_count: 1000,
            distinct_count: 50,
            avg_length: 20.0,
            data_type: DataType::String,
            ..Default::default()
        };

        let strategy = TieredCompressionStrategy::new();
        let encoding = strategy.select_for_tier(&stats, DataTemperature::Hot);

        assert!(matches!(encoding, EncodingType::None | EncodingType::Rle));
    }

    #[test]
    fn test_tiered_strategy_cold() {
        let stats = ColumnStats {
            row_count: 1000,
            distinct_count: 50,
            avg_length: 20.0,
            data_type: DataType::String,
            ..Default::default()
        };

        let strategy = TieredCompressionStrategy::new();
        let encoding = strategy.select_for_tier(&stats, DataTemperature::Cold);

        assert_eq!(encoding, EncodingType::Dictionary);
    }

    #[test]
    fn test_data_temperature() {
        assert_eq!(
            DataTemperature::from_access_count(2000, 1000, 100),
            DataTemperature::Hot
        );
        assert_eq!(
            DataTemperature::from_access_count(50, 1000, 100),
            DataTemperature::Cold
        );
        assert_eq!(
            DataTemperature::from_access_count(500, 1000, 100),
            DataTemperature::Warm
        );
    }

    #[test]
    fn test_hot_data_no_compression() {
        let stats = ColumnStats {
            row_count: 1000,
            access_count: 5000,
            data_type: DataType::Double,
            ..Default::default()
        };

        let config = CompressionConfig {
            prefer_speed_over_ratio: true,
            ..Default::default()
        };
        let selector = CompressionSelector::with_config(config);
        let encoding = selector.select(&stats);

        assert_eq!(encoding, EncodingType::None);
    }
}
