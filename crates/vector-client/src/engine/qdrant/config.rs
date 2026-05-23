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

#[allow(unused_variables)]
pub fn build_create_collection_body(
    name: &str,
    vector_size: usize,
    distance: DistanceMetric,
    index_type: Option<IndexType>,
    hnsw_config: &Option<HnswConfig>,
    quantization_config: &Option<QuantizationConfig>,
    on_disk_payload: Option<bool>,
    shard_number: Option<usize>,
) -> Value {
    let distance_str = distance_to_qdrant(distance);

    let mut vectors = json!({
        "size": vector_size,
        "distance": distance_str
    });

    if let Some(ref hnsw) = hnsw_config {
        let mut hnsw_obj = serde_json::Map::new();
        hnsw_obj.insert("m".to_string(), json!(hnsw.m));
        hnsw_obj.insert("ef_construct".to_string(), json!(hnsw.ef_construct));
        if let Some(threshold) = hnsw.full_scan_threshold {
            hnsw_obj.insert("full_scan_threshold".to_string(), json!(threshold));
        }
        if let Some(threads) = hnsw.max_indexing_threads {
            hnsw_obj.insert("max_indexing_threads".to_string(), json!(threads));
        }
        if let Some(on_disk) = hnsw.on_disk {
            hnsw_obj.insert("on_disk".to_string(), json!(on_disk));
        }
        if let Some(payload_m) = hnsw.payload_m {
            hnsw_obj.insert("payload_m".to_string(), json!(payload_m));
        }

        let mut vectors_obj =
            serde_json::Map::from_iter(vec![("hnsw_config".to_string(), Value::Object(hnsw_obj))]);
        if let Some(s) = index_type {
            if s != IndexType::HNSW {
                tracing::warn!(
                    "Index type {:?} not supported by Qdrant, using HNSW with config",
                    s
                );
            }
        }
        vectors_obj.insert("size".to_string(), json!(vector_size));
        vectors_obj.insert("distance".to_string(), json!(distance_str));
        vectors = Value::Object(vectors_obj);
    }

    if let Some(ref quant) = quantization_config {
        if quant.enabled {
            if let Some(ref qt) = quant.quant_type {
                let quant_val = match qt {
                    QuantizationType::Scalar {
                        quantile,
                        always_ram,
                    } => {
                        let mut obj = serde_json::Map::new();
                        obj.insert("type".to_string(), json!("scalar"));
                        if let Some(q) = quantile {
                            obj.insert("quantile".to_string(), json!(q));
                        }
                        if let Some(ar) = always_ram {
                            obj.insert("always_ram".to_string(), json!(ar));
                        }
                        Value::Object(obj)
                    }
                    QuantizationType::Product {
                        compression,
                        always_ram,
                    } => {
                        let ratio = match compression {
                            CompressionRatio::X4 => 4,
                            CompressionRatio::X8 => 8,
                            CompressionRatio::X16 => 16,
                            CompressionRatio::X32 => 32,
                            CompressionRatio::X64 => 64,
                        };
                        let mut obj = serde_json::Map::new();
                        obj.insert("type".to_string(), json!("product"));
                        obj.insert("compression".to_string(), json!(ratio));
                        if let Some(ar) = always_ram {
                            obj.insert("always_ram".to_string(), json!(ar));
                        }
                        Value::Object(obj)
                    }
                    QuantizationType::Binary { always_ram } => {
                        let mut obj = serde_json::Map::new();
                        obj.insert("type".to_string(), json!("binary"));
                        if let Some(ar) = always_ram {
                            obj.insert("always_ram".to_string(), json!(ar));
                        }
                        Value::Object(obj)
                    }
                };
                let mut vectors_obj = serde_json::Map::new();
                vectors_obj.insert("size".to_string(), json!(vector_size));
                vectors_obj.insert("distance".to_string(), json!(distance_str));
                vectors_obj.insert("quantization_config".to_string(), quant_val);
                if hnsw_config.is_some() {
                    if let Some(ref hnsw) = hnsw_config {
                        let mut hnsw_obj = serde_json::Map::new();
                        hnsw_obj.insert("m".to_string(), json!(hnsw.m));
                        hnsw_obj.insert("ef_construct".to_string(), json!(hnsw.ef_construct));
                        vectors_obj.insert("hnsw_config".to_string(), Value::Object(hnsw_obj));
                    }
                }
                vectors = Value::Object(vectors_obj);
            }
        }
    }

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
