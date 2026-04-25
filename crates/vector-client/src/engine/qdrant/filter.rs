use qdrant_client::qdrant::{
    Condition, Filter, GeoBoundingBox as QdrantGeoBoundingBox, GeoPoint as QdrantGeoPoint,
    GeoRadius as QdrantGeoRadius, PointId, Range, ValuesCount,
};

use crate::error::{Result, VectorClientError};
use crate::types::{ConditionType, FilterCondition, VectorFilter};

pub fn convert_filter(filter: &VectorFilter) -> Result<Filter> {
    let mut qdrant_filter = Filter::default();

    if let Some(must_conditions) = &filter.must {
        for condition in must_conditions {
            if let Some(cond) = convert_condition(condition)? {
                qdrant_filter.must.push(cond);
            }
        }
    }

    if let Some(must_not_conditions) = &filter.must_not {
        for condition in must_not_conditions {
            if let Some(cond) = convert_condition(condition)? {
                qdrant_filter.must_not.push(cond);
            }
        }
    }

    if let Some(should_conditions) = &filter.should {
        for condition in should_conditions {
            if let Some(cond) = convert_condition(condition)? {
                qdrant_filter.should.push(cond);
            }
        }
    }

    Ok(qdrant_filter)
}

fn point_id_from_str(id: &str) -> PointId {
    if let Ok(num) = id.parse::<u64>() {
        num.into()
    } else {
        id.into()
    }
}

fn convert_condition(condition: &FilterCondition) -> Result<Option<Condition>> {
    let cond = match &condition.condition {
        ConditionType::Match { value } => {
            Condition::matches(condition.field.clone(), value.clone())
        }
        ConditionType::MatchAny { values } => {
            Condition::matches(condition.field.clone(), values.clone())
        }
        ConditionType::Range(range) => {
            let mut qdrant_range = Range::default();
            if let Some(gt) = range.gt {
                qdrant_range.gt = Some(gt);
            }
            if let Some(gte) = range.gte {
                qdrant_range.gte = Some(gte);
            }
            if let Some(lt) = range.lt {
                qdrant_range.lt = Some(lt);
            }
            if let Some(lte) = range.lte {
                qdrant_range.lte = Some(lte);
            }
            Condition::range(condition.field.clone(), qdrant_range)
        }
        ConditionType::IsEmpty => Condition::is_empty(condition.field.clone()),
        ConditionType::IsNull => Condition::is_null(condition.field.clone()),
        ConditionType::HasId { ids } => {
            let point_ids: Vec<PointId> = ids.iter().map(|id| point_id_from_str(id)).collect();
            Condition::has_id(point_ids)
        }
        ConditionType::GeoRadius(radius) => Condition::geo_radius(
            condition.field.clone(),
            QdrantGeoRadius {
                center: Some(QdrantGeoPoint {
                    lat: radius.center.lat,
                    lon: radius.center.lon,
                }),
                radius: radius.radius as f32,
            },
        ),
        ConditionType::GeoBoundingBox(bbox) => Condition::geo_bounding_box(
            condition.field.clone(),
            QdrantGeoBoundingBox {
                top_left: Some(QdrantGeoPoint {
                    lat: bbox.top_left.lat,
                    lon: bbox.top_left.lon,
                }),
                bottom_right: Some(QdrantGeoPoint {
                    lat: bbox.bottom_right.lat,
                    lon: bbox.bottom_right.lon,
                }),
            },
        ),
        ConditionType::ValuesCount(count) => {
            let mut qdrant_values_count = ValuesCount::default();
            if let Some(gt) = count.gt {
                qdrant_values_count.gt = Some(gt);
            }
            if let Some(gte) = count.gte {
                qdrant_values_count.gte = Some(gte);
            }
            if let Some(lt) = count.lt {
                qdrant_values_count.lt = Some(lt);
            }
            if let Some(lte) = count.lte {
                qdrant_values_count.lte = Some(lte);
            }
            Condition::values_count(condition.field.clone(), qdrant_values_count)
        }
        ConditionType::Contains { value } => {
            Condition::matches(condition.field.clone(), vec![value.clone()])
        }
        ConditionType::Nested { filter } => {
            let nested_filter = convert_filter(filter)?;
            Condition::nested(condition.field.clone(), nested_filter)
        }
        ConditionType::Payload { key, value } => {
            let field_path = format!("{}.{}", condition.field, key);
            let match_value = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Array(arr) => {
                    let strings: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    if strings.len() == arr.len() {
                        return Ok(Some(Condition::matches(field_path, strings)));
                    }
                    let ints: Vec<i64> = arr.iter().filter_map(|v| v.as_i64()).collect();
                    if ints.len() == arr.len() {
                        return Ok(Some(Condition::matches(field_path, ints)));
                    }
                    return Err(VectorClientError::FilterError(
                        "Payload filter array contains unsupported value types".to_string(),
                    ));
                }
                _ => {
                    return Err(VectorClientError::FilterError(
                        "Payload filter value type not supported".to_string(),
                    ));
                }
            };
            Condition::matches(field_path, match_value)
        }
    };

    Ok(Some(cond))
}
