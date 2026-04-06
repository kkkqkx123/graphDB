use serde::{Deserialize, Serialize};

pub type PayloadValue = serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorFilter {
    pub must: Option<Vec<FilterCondition>>,
    pub must_not: Option<Vec<FilterCondition>>,
    pub should: Option<Vec<FilterCondition>>,
    pub min_should: Option<MinShouldCondition>,
}

impl VectorFilter {
    pub fn new() -> Self {
        Self {
            must: None,
            must_not: None,
            should: None,
            min_should: None,
        }
    }

    pub fn must(mut self, condition: FilterCondition) -> Self {
        self.must.get_or_insert_with(Vec::new).push(condition);
        self
    }

    pub fn must_not(mut self, condition: FilterCondition) -> Self {
        self.must_not.get_or_insert_with(Vec::new).push(condition);
        self
    }

    pub fn should(mut self, condition: FilterCondition) -> Self {
        self.should.get_or_insert_with(Vec::new).push(condition);
        self
    }
}

impl Default for VectorFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinShouldCondition {
    pub conditions: Vec<FilterCondition>,
    pub min_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition {
    pub field: String,
    pub condition: ConditionType,
}

impl FilterCondition {
    pub fn new(field: impl Into<String>, condition: ConditionType) -> Self {
        Self {
            field: field.into(),
            condition,
        }
    }

    pub fn match_value(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(field, ConditionType::Match { value: value.into() })
    }

    pub fn match_any(field: impl Into<String>, values: Vec<String>) -> Self {
        Self::new(field, ConditionType::MatchAny { values })
    }

    pub fn range(field: impl Into<String>, range: RangeCondition) -> Self {
        Self::new(field, ConditionType::Range(range))
    }

    pub fn is_empty(field: impl Into<String>) -> Self {
        Self::new(field, ConditionType::IsEmpty)
    }

    pub fn is_null(field: impl Into<String>) -> Self {
        Self::new(field, ConditionType::IsNull)
    }

    pub fn has_id(ids: Vec<String>) -> Self {
        Self::new("_id", ConditionType::HasId { ids })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    Match { value: String },
    MatchAny { values: Vec<String> },
    Range(RangeCondition),
    IsEmpty,
    IsNull,
    HasId { ids: Vec<String> },
    Nested { filter: Box<VectorFilter> },
    Payload { key: String, value: PayloadValue },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeCondition {
    pub gt: Option<f64>,
    pub gte: Option<f64>,
    pub lt: Option<f64>,
    pub lte: Option<f64>,
}

impl RangeCondition {
    pub fn new() -> Self {
        Self {
            gt: None,
            gte: None,
            lt: None,
            lte: None,
        }
    }

    pub fn gt(mut self, value: f64) -> Self {
        self.gt = Some(value);
        self
    }

    pub fn gte(mut self, value: f64) -> Self {
        self.gte = Some(value);
        self
    }

    pub fn lt(mut self, value: f64) -> Self {
        self.lt = Some(value);
        self
    }

    pub fn lte(mut self, value: f64) -> Self {
        self.lte = Some(value);
        self
    }
}

impl Default for RangeCondition {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadSelector {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

impl PayloadSelector {
    pub fn include(fields: Vec<String>) -> Self {
        Self {
            include: Some(fields),
            exclude: None,
        }
    }

    pub fn exclude(fields: Vec<String>) -> Self {
        Self {
            include: None,
            exclude: Some(fields),
        }
    }

    pub fn all() -> Self {
        Self {
            include: None,
            exclude: None,
        }
    }
}
