use crate::error::{Result, VectorClientError};
use crate::types::*;

pub trait ConditionHandler {
    type Condition;
    type Filter;

    fn handle_match(&self, field: &str, value: &str) -> Self::Condition;
    fn handle_match_any(&self, field: &str, values: &[String]) -> Self::Condition;
    fn handle_range(&self, field: &str, range: &RangeCondition) -> Self::Condition;
    fn handle_is_empty(&self, field: &str) -> Self::Condition;
    fn handle_is_null(&self, field: &str) -> Self::Condition;
    fn handle_has_id(&self, ids: &[String]) -> Self::Condition;
    fn handle_geo_radius(&self, field: &str, radius: &GeoRadius) -> Self::Condition;
    fn handle_geo_bounding_box(&self, field: &str, bbox: &GeoBoundingBox) -> Self::Condition;
    fn handle_values_count(&self, field: &str, count: &ValuesCountCondition) -> Self::Condition;
    fn handle_contains(&self, field: &str, value: &str) -> Self::Condition;
    fn handle_nested(&self, field: &str, filter: Self::Filter) -> Self::Condition;
    fn handle_payload(&self, field: &str, key: &str, value: &serde_json::Value) -> Self::Condition;
    fn build_filter(
        &self,
        must: Vec<Self::Condition>,
        must_not: Vec<Self::Condition>,
        should: Vec<Self::Condition>,
        min_should: Option<(Vec<Self::Condition>, usize)>,
    ) -> Option<Self::Filter>;
}

pub fn process_filter<H: ConditionHandler>(
    filter: &VectorFilter,
    handler: &H,
) -> Result<Option<H::Filter>> {
    let mut should: Vec<H::Condition> = Vec::new();
    let mut must: Vec<H::Condition> = Vec::new();
    let mut must_not: Vec<H::Condition> = Vec::new();

    if let Some(ref conditions) = filter.must {
        for c in conditions {
            must.push(handle_condition(c, handler)?);
        }
    }

    if let Some(ref conditions) = filter.must_not {
        for c in conditions {
            must_not.push(handle_condition(c, handler)?);
        }
    }

    if let Some(ref conditions) = filter.should {
        for c in conditions {
            should.push(handle_condition(c, handler)?);
        }
    }

    let min_should = if let Some(ref ms) = filter.min_should {
        let mut conditions = Vec::new();
        for c in &ms.conditions {
            conditions.push(handle_condition(c, handler)?);
        }
        Some((conditions, ms.min_count))
    } else {
        None
    };

    if should.is_empty() && must.is_empty() && must_not.is_empty() && min_should.is_none() {
        return Ok(None);
    }

    Ok(handler.build_filter(must, must_not, should, min_should))
}

fn handle_condition<H: ConditionHandler>(c: &FilterCondition, handler: &H) -> Result<H::Condition> {
    match &c.condition {
        ConditionType::Match { value } => Ok(handler.handle_match(&c.field, value)),
        ConditionType::MatchAny { values } => Ok(handler.handle_match_any(&c.field, values)),
        ConditionType::Range(range) => Ok(handler.handle_range(&c.field, range)),
        ConditionType::IsEmpty => Ok(handler.handle_is_empty(&c.field)),
        ConditionType::IsNull => Ok(handler.handle_is_null(&c.field)),
        ConditionType::HasId { ids } => Ok(handler.handle_has_id(ids)),
        ConditionType::Nested { filter } => {
            let nested = process_filter(filter, handler)?
                .ok_or_else(|| VectorClientError::FilterError("Empty nested filter".to_string()))?;
            Ok(handler.handle_nested(&c.field, nested))
        }
        ConditionType::Payload { key, value } => Ok(handler.handle_payload(&c.field, key, value)),
        ConditionType::GeoRadius(radius) => Ok(handler.handle_geo_radius(&c.field, radius)),
        ConditionType::GeoBoundingBox(bbox) => Ok(handler.handle_geo_bounding_box(&c.field, bbox)),
        ConditionType::ValuesCount(count) => Ok(handler.handle_values_count(&c.field, count)),
        ConditionType::Contains { value } => Ok(handler.handle_contains(&c.field, value)),
    }
}
