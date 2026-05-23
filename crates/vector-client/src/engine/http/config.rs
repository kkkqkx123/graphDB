use serde_json::{json, Value};

use crate::types::{
    CompressionRatio, DistanceMetric, HnswConfig, IndexType, PayloadSchemaType, QuantizationConfig,
    QuantizationType,
};

pub fn distance_to_qdrant(distance: DistanceMetric) -> &'static str {
    match distance {
        DistanceMetric::Cosine => "Cosine",
        DistanceMetric::Euclid => "Euclid",
        DistanceMetric::Dot => "Dot",
        DistanceMetric::Manhattan => "Manhattan",
    }
}

pub fn field_type_to_qdrant(schema: PayloadSchemaType) -> &'static str {
    match schema {
        PayloadSchemaType::Keyword => "keyword",
        PayloadSchemaType::Integer => "integer",
        PayloadSchemaType::Float => "float",
        PayloadSchemaType::Text => "text",
        PayloadSchemaType::Bool => "bool",
        PayloadSchemaType::Geo => "geo",
        PayloadSchemaType::Datetime => "datetime",
    }
}

fn build_vectors_json(
    vector_size: usize,
    distance: DistanceMetric,
    index_type: Option<IndexType>,
    hnsw_config: &Option<HnswConfig>,
    quantization_config: &Option<QuantizationConfig>,
) -> Value {
    let distance_str = distance_to_qdrant(distance);
    let mut vectors = json!({
        "size": vector_size,
        "distance": distance_str
    });

    if let Some(ref hnsw) = hnsw_config {
        vectors["hnsw_config"] = build_hnsw_json(hnsw);
    }

    if let Some(s) = index_type {
        if s != IndexType::HNSW {
            tracing::warn!(
                "Index type {:?} not supported by Qdrant, using HNSW with config",
                s
            );
        }
    }

    if let Some(ref quant) = quantization_config {
        if quant.enabled {
            if let Some(ref qt) = quant.quant_type {
                vectors["quantization_config"] = build_quantization_json(qt);
            }
        }
    }

    vectors
}

fn build_hnsw_json(hnsw: &HnswConfig) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("m".to_string(), json!(hnsw.m));
    obj.insert("ef_construct".to_string(), json!(hnsw.ef_construct));
    if let Some(v) = hnsw.full_scan_threshold {
        obj.insert("full_scan_threshold".to_string(), json!(v));
    }
    if let Some(v) = hnsw.max_indexing_threads {
        obj.insert("max_indexing_threads".to_string(), json!(v));
    }
    if let Some(v) = hnsw.on_disk {
        obj.insert("on_disk".to_string(), json!(v));
    }
    if let Some(v) = hnsw.payload_m {
        obj.insert("payload_m".to_string(), json!(v));
    }
    Value::Object(obj)
}

fn build_quantization_json(qt: &QuantizationType) -> Value {
    let mut obj = serde_json::Map::new();
    let type_name = match qt {
        QuantizationType::Scalar { .. } => "scalar",
        QuantizationType::Product { .. } => "product",
        QuantizationType::Binary { .. } => "binary",
    };
    obj.insert("type".to_string(), json!(type_name));

    if let QuantizationType::Product { compression, .. } = qt {
        let ratio = match compression {
            CompressionRatio::X4 => 4,
            CompressionRatio::X8 => 8,
            CompressionRatio::X16 => 16,
            CompressionRatio::X32 => 32,
            CompressionRatio::X64 => 64,
        };
        obj.insert("compression".to_string(), json!(ratio));
    }

    let always_ram = match qt {
        QuantizationType::Scalar { always_ram, .. }
        | QuantizationType::Product { always_ram, .. }
        | QuantizationType::Binary { always_ram } => always_ram,
    };
    if let Some(v) = always_ram {
        obj.insert("always_ram".to_string(), json!(v));
    }

    if let QuantizationType::Scalar { quantile: Some(v), .. } = qt {
        obj.insert("quantile".to_string(), json!(v));
    }

    Value::Object(obj)
}

#[allow(clippy::too_many_arguments)]
pub fn build_create_collection_body(
    _name: &str,
    vector_size: usize,
    distance: DistanceMetric,
    index_type: Option<IndexType>,
    hnsw_config: &Option<HnswConfig>,
    quantization_config: &Option<QuantizationConfig>,
    on_disk_payload: Option<bool>,
    shard_number: Option<usize>,
) -> Value {
    let vectors = build_vectors_json(vector_size, distance, index_type, hnsw_config, quantization_config);

    let mut body = serde_json::Map::new();
    body.insert("vectors".to_string(), vectors);

    if let Some(on_disk) = on_disk_payload {
        body.insert("on_disk_payload".to_string(), json!(on_disk));
    }

    if let Some(shards) = shard_number {
        body.insert("shard_number".to_string(), json!(shards));
    }

    Value::Object(body)
}

pub fn build_upsert_body(points_json: Value) -> Value {
    json!({
        "points": points_json
    })
}

pub fn build_delete_by_ids_body(ids: Vec<Value>) -> Value {
    json!({
        "points": ids
    })
}

pub fn build_delete_by_filter_body(filter: Value) -> Value {
    json!({
        "filter": filter
    })
}

#[allow(clippy::too_many_arguments)]
pub fn build_search_body(
    vector: Vec<f32>,
    limit: usize,
    offset: Option<usize>,
    score_threshold: Option<f32>,
    filter_json: Option<Value>,
    with_payload: Option<bool>,
    with_vector: Option<bool>,
    nprobe: Option<usize>,
) -> Value {
    let mut body = serde_json::Map::new();
    body.insert("vector".to_string(), json!(vector));
    body.insert("limit".to_string(), json!(limit));
    body.insert("with_payload".to_string(), json!(with_payload.unwrap_or(true)));
    body.insert("with_vector".to_string(), json!(with_vector.unwrap_or(false)));

    if let Some(off) = offset {
        body.insert("offset".to_string(), json!(off));
    }

    if let Some(threshold) = score_threshold {
        body.insert("score_threshold".to_string(), json!(threshold));
    }

    if let Some(ref filter) = filter_json {
        body.insert("filter".to_string(), filter.clone());
    }

    let mut params = serde_json::Map::new();
    if let Some(ef) = nprobe {
        params.insert("hnsw_ef".to_string(), json!(ef));
    }
    if !params.is_empty() {
        body.insert("params".to_string(), Value::Object(params));
    }

    Value::Object(body)
}

pub fn build_get_body(
    ids: Vec<Value>,
    with_payload: Option<bool>,
    with_vector: Option<bool>,
) -> Value {
    let mut body = serde_json::Map::new();
    body.insert("ids".to_string(), json!(ids));
    body.insert("with_payload".to_string(), json!(with_payload.unwrap_or(true)));
    body.insert("with_vector".to_string(), json!(with_vector.unwrap_or(false)));
    Value::Object(body)
}

pub fn build_scroll_body(
    limit: usize,
    offset: Option<Value>,
    with_payload: Option<bool>,
    with_vector: Option<bool>,
) -> Value {
    let mut body = serde_json::Map::new();
    body.insert("limit".to_string(), json!(limit));
    body.insert("with_payload".to_string(), json!(with_payload.unwrap_or(true)));
    body.insert("with_vector".to_string(), json!(with_vector.unwrap_or(false)));

    if let Some(off) = offset {
        body.insert("offset".to_string(), off);
    }

    Value::Object(body)
}

pub fn build_set_payload_body(ids: Vec<Value>, payload: Value) -> Value {
    json!({
        "payload": payload,
        "points": ids
    })
}

pub fn build_delete_payload_body(ids: Vec<Value>, keys: Vec<String>) -> Value {
    json!({
        "keys": keys,
        "points": ids
    })
}

pub fn build_create_payload_index_body(
    field_name: &str,
    field_type: PayloadSchemaType,
) -> Value {
    json!({
        "field_name": field_name,
        "field_type": field_type_to_qdrant(field_type)
    })
}

pub fn build_search_batch_body(searches: Vec<Value>) -> Value {
    json!({
        "searches": searches
    })
}
