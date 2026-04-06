use qdrant_client::qdrant::{
    Distance, HnswConfigDiffBuilder, FieldType,
    ScalarQuantizationBuilder, ProductQuantizationBuilder, BinaryQuantizationBuilder,
    quantization_config::Quantization as QdrantQuantization,
};

use crate::types::{DistanceMetric, HnswConfig, QuantizationConfig, QuantizationType, CompressionRatio, PayloadSchemaType};

pub fn convert_distance(distance: DistanceMetric) -> Distance {
    match distance {
        DistanceMetric::Cosine => Distance::Cosine,
        DistanceMetric::Euclid => Distance::Euclid,
        DistanceMetric::Dot => Distance::Dot,
    }
}

pub fn build_hnsw_config(config: &Option<HnswConfig>) -> Option<HnswConfigDiffBuilder> {
    config.as_ref().map(|hnsw| {
        let mut builder = HnswConfigDiffBuilder::default()
            .m(hnsw.m as u64)
            .ef_construct(hnsw.ef_construct as u64);

        if let Some(threshold) = hnsw.full_scan_threshold {
            builder = builder.full_scan_threshold(threshold as u64);
        }
        if let Some(threads) = hnsw.max_indexing_threads {
            builder = builder.max_indexing_threads(threads as u64);
        }
        if let Some(on_disk) = hnsw.on_disk {
            builder = builder.on_disk(on_disk);
        }
        if let Some(payload_m) = hnsw.payload_m {
            builder = builder.payload_m(payload_m as u64);
        }

        builder
    })
}

pub fn build_quantization_config(
    config: &Option<QuantizationConfig>,
) -> Option<QdrantQuantization> {
    config.as_ref().and_then(|quant| {
        if !quant.enabled {
            return None;
        }

        let qdrant_quant_config = match &quant.quant_type {
            Some(QuantizationType::Scalar { quantile, always_ram }) => {
                let mut builder = ScalarQuantizationBuilder::default();
                if let Some(q) = quantile {
                    builder = builder.quantile(*q);
                }
                if let Some(ar) = always_ram {
                    builder = builder.always_ram(*ar);
                }
                QdrantQuantization::from(builder)
            }
            Some(QuantizationType::Product { compression, always_ram }) => {
                let compression_value = match compression {
                    CompressionRatio::X4 => 4,
                    CompressionRatio::X8 => 8,
                    CompressionRatio::X16 => 16,
                    CompressionRatio::X32 => 32,
                    CompressionRatio::X64 => 64,
                };
                let mut builder = ProductQuantizationBuilder::new(compression_value);
                if let Some(ar) = always_ram {
                    builder = builder.always_ram(*ar);
                }
                QdrantQuantization::from(builder)
            }
            Some(QuantizationType::Binary { always_ram }) => {
                let builder = BinaryQuantizationBuilder::new(always_ram.unwrap_or(true));
                QdrantQuantization::from(builder)
            }
            None => return None,
        };

        Some(qdrant_quant_config)
    })
}

pub fn convert_field_type(schema: PayloadSchemaType) -> FieldType {
    match schema {
        PayloadSchemaType::Keyword => FieldType::Keyword,
        PayloadSchemaType::Integer => FieldType::Integer,
        PayloadSchemaType::Float => FieldType::Float,
        PayloadSchemaType::Text => FieldType::Text,
        PayloadSchemaType::Bool => FieldType::Bool,
        PayloadSchemaType::Geo => FieldType::Geo,
        PayloadSchemaType::Datetime => FieldType::Datetime,
    }
}
