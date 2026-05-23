use serde_json::{json, Value};

use crate::error::{Result, VectorClientError};
use crate::types::{ConditionType, FilterCondition, VectorFilter};

fn point_id_json(id: &str) -> Value {
    if let Ok(num) = id.parse::<u64>() {
        json!(num)
    } else {
        json!(id)
    }
}

pub fn convert_filter(filter: &VectorFilter) -> Result<Option<Value>> {
    let must = if let Some(conditions) = &filter.must {
        let items: Result<Vec<Value>> = conditions.iter().map(convert_condition).collect();
        Some(items?)
    } else {
        None
    };

    let must_not = if let Some(conditions) = &filter.must_not {
        let items: Result<Vec<Value>> = conditions.iter().map(convert_condition).collect();
        Some(items?)
    } else {
        None
    };

    let should = if let Some(conditions) = &filter.should {
        let items: Result<Vec<Value>> = conditions.iter().map(convert_condition).collect();
        Some(items?)
    } else {
        None
    };

    if must.is_none() && must_not.is_none() && should.is_none() {
        return Ok(None);
    }

    let mut filter_obj = serde_json::Map::new();
    if let Some(m) = must {
        filter_obj.insert("must".to_string(), Value::Array(m));
    }
    if let Some(m) = must_not {
        filter_obj.insert("must_not".to_string(), Value::Array(m));
    }
    if let Some(s) = should {
        filter_obj.insert("should".to_string(), Value::Array(s));
    }

    if let Some(ref min_should) = filter.min_should {
        let conditions: Result<Vec<Value>> = min_should
            .conditions
            .iter()
            .map(convert_condition)
            .collect();
        filter_obj.insert("should".to_string(), Value::Array(conditions?));
        filter_obj.insert(
            "min_should".to_string(),
            json!({ "conditions": min_should.min_count }),
        );
    }

    Ok(Some(Value::Object(filter_obj)))
}

fn convert_condition(condition: &FilterCondition) -> Result<Value> {
    match &condition.condition {
        ConditionType::Match { value } => Ok(json!({
            "key": condition.field,
            "match": { "value": value }
        })),
        ConditionType::MatchAny { values } => Ok(json!({
            "key": condition.field,
            "match_any": { "any": values }
        })),
        ConditionType::Range(range) => {
            let mut range_obj = serde_json::Map::new();
            if let Some(gt) = range.gt {
                range_obj.insert("gt".to_string(), json!(gt));
            }
            if let Some(gte) = range.gte {
                range_obj.insert("gte".to_string(), json!(gte));
            }
            if let Some(lt) = range.lt {
                range_obj.insert("lt".to_string(), json!(lt));
            }
            if let Some(lte) = range.lte {
                range_obj.insert("lte".to_string(), json!(lte));
            }
            Ok(json!({
                "key": condition.field,
                "range": range_obj
            }))
        }
        ConditionType::IsEmpty => Ok(json!({
            "is_empty": { "key": condition.field }
        })),
        ConditionType::IsNull => Ok(json!({
            "is_null": { "key": condition.field }
        })),
        ConditionType::HasId { ids } => {
            let point_ids: Vec<Value> = ids.iter().map(|id| point_id_json(id)).collect();
            Ok(json!({
                "has_id": point_ids
            }))
        }
        ConditionType::GeoRadius(radius) => Ok(json!({
            "key": condition.field,
            "geo_radius": {
                "center": { "lat": radius.center.lat, "lon": radius.center.lon },
                "radius": radius.radius as f32
            }
        })),
        ConditionType::GeoBoundingBox(bbox) => Ok(json!({
            "key": condition.field,
            "geo_bounding_box": {
                "top_left": { "lat": bbox.top_left.lat, "lon": bbox.top_left.lon },
                "bottom_right": { "lat": bbox.bottom_right.lat, "lon": bbox.bottom_right.lon }
            }
        })),
        ConditionType::ValuesCount(count) => {
            let mut count_obj = serde_json::Map::new();
            if let Some(gt) = count.gt {
                count_obj.insert("gt".to_string(), json!(gt));
            }
            if let Some(gte) = count.gte {
                count_obj.insert("gte".to_string(), json!(gte));
            }
            if let Some(lt) = count.lt {
                count_obj.insert("lt".to_string(), json!(lt));
            }
            if let Some(lte) = count.lte {
                count_obj.insert("lte".to_string(), json!(lte));
            }
            Ok(json!({
                "key": condition.field,
                "values_count": count_obj
            }))
        }
        ConditionType::Contains { value } => Ok(json!({
            "key": condition.field,
            "match": { "text": value }
        })),
        ConditionType::Nested { filter } => {
            let nested_filter = convert_filter(filter)?
                .ok_or_else(|| VectorClientError::FilterError("Empty nested filter".to_string()))?;
            Ok(json!({
                "nested": {
                    "key": condition.field,
                    "filter": nested_filter
                }
            }))
        }
        ConditionType::Payload { key, value } => {
            let field_path = format!("{}.{}", condition.field, key);
            Ok(json!({
                "key": field_path,
                "match": { "value": value }
            }))
        }
    }
}
