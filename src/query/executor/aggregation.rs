use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use super::base::BaseExecutor;
use crate::core::types::operators::AggregateFunction;
use crate::core::Value;
use crate::query::executor::traits::{DBResult, ExecutionResult, Executor, HasStorage, HasInput};
use crate::storage::StorageClient;
use crate::utils::safe_lock;

#[derive(Debug, Clone)]
pub struct AggData {
    result: Value,
    sum: Option<Value>,
    cnt: Option<Value>,
    avg: Option<Value>,
    deviation: Option<Value>,
    distinct_set: Option<HashSet<String>>,
}

impl AggData {
    pub fn new() -> Self {
        Self {
            result: Value::Null(crate::core::value::NullType::NaN),
            sum: None,
            cnt: None,
            avg: None,
            deviation: None,
            distinct_set: None,
        }
    }

    pub fn result(&self) -> &Value {
        &self.result
    }

    pub fn result_mut(&mut self) -> &mut Value {
        &mut self.result
    }

    pub fn set_result(&mut self, value: Value) {
        self.result = value;
    }

    pub fn sum(&self) -> Option<&Value> {
        self.sum.as_ref()
    }

    pub fn sum_mut(&mut self) -> &mut Value {
        self.sum.get_or_insert_with(|| Value::Float(0.0))
    }

    pub fn cnt(&self) -> Option<&Value> {
        self.cnt.as_ref()
    }

    pub fn cnt_mut(&mut self) -> &mut Value {
        self.cnt.get_or_insert_with(|| Value::Float(0.0))
    }

    pub fn avg(&self) -> Option<&Value> {
        self.avg.as_ref()
    }

    pub fn avg_mut(&mut self) -> &mut Value {
        self.avg.get_or_insert_with(|| Value::Float(0.0))
    }

    pub fn deviation(&self) -> Option<&Value> {
        self.deviation.as_ref()
    }

    pub fn deviation_mut(&mut self) -> &mut Value {
        self.deviation.get_or_insert_with(|| Value::Float(0.0))
    }

    pub fn distinct_set_mut(&mut self) -> &mut HashSet<String> {
        self.distinct_set.get_or_insert_with(HashSet::new)
    }
}

impl Default for AggData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct GroupKey {
    keys: Vec<Value>,
}

impl GroupKey {
    pub fn new(keys: Vec<Value>) -> Self {
        Self { keys }
    }

    pub fn keys(&self) -> &[Value] {
        &self.keys
    }
}

impl PartialEq for GroupKey {
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys
    }
}

impl Eq for GroupKey {}

impl std::hash::Hash for GroupKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for key in &self.keys {
            match key {
                Value::Int(i) => i.hash(state),
                Value::Float(f) => {
                    let bits = f.to_bits();
                    bits.hash(state);
                }
                Value::String(s) => s.hash(state),
                Value::Bool(b) => b.hash(state),
                Value::Null(_) => 0.hash(state),
                Value::List(l) => {
                    for item in l {
                        item.hash(state);
                    }
                }
                Value::Map(m) => {
                    let mut sorted: Vec<_> = m.iter().collect();
                    sorted.sort_by_key(|k| k.0);
                    for (k, v) in sorted {
                        k.hash(state);
                        v.hash(state);
                    }
                }
                _ => format!("{:?}", key).hash(state),
            }
        }
    }
}

pub struct ExpressionContext<'a> {
    variables: HashMap<String, Value>,
    current_row: Option<&'a Value>,
}

impl<'a> ExpressionContext<'a> {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            current_row: None,
        }
    }

    pub fn with_row(row: &'a Value) -> Self {
        Self {
            variables: HashMap::new(),
            current_row: Some(row),
        }
    }

    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn eval_input_property(&self, prop_name: &str) -> Value {
        if let Some(row) = self.current_row {
            Self::extract_property(row, prop_name)
        } else {
            Value::Null(crate::core::value::NullType::NaN)
        }
    }

    fn extract_property(value: &Value, prop_name: &str) -> Value {
        match value {
            Value::Vertex(vertex) => {
                if let Some(prop_value) = vertex.get_property_any(prop_name) {
                    prop_value.clone()
                } else {
                    Value::Null(crate::core::value::NullType::NaN)
                }
            }
            Value::Edge(edge) => {
                if let Some(prop_value) = edge.get_property(prop_name) {
                    prop_value.clone()
                } else {
                    Value::Null(crate::core::value::NullType::NaN)
                }
            }
            Value::Map(map) => {
                if let Some(prop_value) = map.get(prop_name) {
                    prop_value.clone()
                } else {
                    Value::Null(crate::core::value::NullType::NaN)
                }
            }
            Value::List(list) => {
                if let Ok(idx) = prop_name.parse::<usize>() {
                    if idx < list.len() {
                        return list[idx].clone();
                    }
                }
                Value::Null(crate::core::value::NullType::NaN)
            }
            _ => value.clone(),
        }
    }
}

impl<'a> Default for ExpressionContext<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct AggregationExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    input_var: String,
    aggregation_functions: Vec<AggregateFunction>,
    group_by_keys: Vec<String>,
    col_names: Vec<String>,
    output_var: String,
}

impl<S: StorageClient> AggregationExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        input_var: String,
        aggregation_functions: Vec<AggregateFunction>,
        group_by_keys: Vec<String>,
        col_names: Vec<String>,
        output_var: String,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "AggregationExecutor".to_string(), storage),
            input_var,
            aggregation_functions,
            group_by_keys,
            col_names,
            output_var,
        }
    }

    fn execute_aggregation(&self, input_data: &[Value]) -> DBResult<ExecutionResult> {
        let mut result: HashMap<GroupKey, Vec<AggData>> = HashMap::new();

        if input_data.is_empty() {
            return self.handle_empty_input();
        }

        for row in input_data {
            let group_key = self.extract_group_keys(row)?;
            let ctx = ExpressionContext::with_row(row);

            let entry = result.entry(group_key).or_insert_with(|| {
                (0..self.aggregation_functions.len())
                    .map(|_| AggData::new())
                    .collect()
            });

            for (i, agg_func) in self.aggregation_functions.iter().enumerate() {
                self.apply_aggregate_function(
                    agg_func,
                    &ctx,
                    entry.get_mut(i).expect("聚合数据索引超出范围"),
                )?;
            }
        }

        self.build_result(result)
    }

    fn handle_empty_input(&self) -> DBResult<ExecutionResult> {
        let mut default_results = Vec::new();

        for agg_func in &self.aggregation_functions {
            let mut agg_data = AggData::new();
            self.apply_aggregate_function(agg_func, &ExpressionContext::new(), &mut agg_data)?;
            default_results.push(agg_data.result().clone());
        }

        let mut result_rows = Vec::new();
        if !default_results.is_empty() {
            result_rows.push(Value::List(default_results));
        }

        Ok(ExecutionResult::Values(result_rows))
    }

    fn extract_group_keys(&self, row: &Value) -> DBResult<GroupKey> {
        let mut keys = Vec::new();

        for key_name in &self.group_by_keys {
            let key_value = self.extract_field_value(row, key_name)?;
            keys.push(key_value);
        }

        Ok(GroupKey::new(keys))
    }

    fn apply_aggregate_function(
        &self,
        agg_func: &AggregateFunction,
        ctx: &ExpressionContext,
        agg_data: &mut AggData,
    ) -> DBResult<()> {
        match agg_func {
            AggregateFunction::Count(field_name) => {
                let val = match field_name {
                    None => Value::Null(crate::core::value::NullType::NaN),
                    Some(name) => ctx.eval_input_property(name),
                };
                Self::apply_count(agg_data, &val);
            }
            AggregateFunction::Sum(field_name) => {
                let val = ctx.eval_input_property(field_name);
                Self::apply_sum(agg_data, &val);
            }
            AggregateFunction::Avg(field_name) => {
                let val = ctx.eval_input_property(field_name);
                Self::apply_avg(agg_data, &val);
            }
            AggregateFunction::Min(field_name) => {
                let val = ctx.eval_input_property(field_name);
                Self::apply_min(agg_data, &val);
            }
            AggregateFunction::Max(field_name) => {
                let val = ctx.eval_input_property(field_name);
                Self::apply_max(agg_data, &val);
            }
            AggregateFunction::Collect(field_name) => {
                let val = ctx.eval_input_property(field_name);
                Self::apply_collect(agg_data, &val);
            }
            AggregateFunction::Distinct(field_name) => {
                let val = ctx.eval_input_property(field_name);
                Self::apply_distinct(agg_data, &val);
            }
            AggregateFunction::Percentile(field_name, percentile) => {
                let val = ctx.eval_input_property(field_name);
                Self::apply_percentile(agg_data, &val, *percentile);
            }
            AggregateFunction::Std(field_name) => {
                let val = ctx.eval_input_property(field_name);
                Self::apply_std(agg_data, &val);
            }
            AggregateFunction::BitAnd(field_name) => {
                let val = ctx.eval_input_property(field_name);
                Self::apply_bit_and(agg_data, &val);
            }
            AggregateFunction::BitOr(field_name) => {
                let val = ctx.eval_input_property(field_name);
                Self::apply_bit_or(agg_data, &val);
            }
            AggregateFunction::GroupConcat(field_name, separator) => {
                let val = ctx.eval_input_property(field_name);
                Self::apply_group_concat(agg_data, &val, separator);
            }
        }
        Ok(())
    }

    fn apply_count(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(crate::core::value::NullType::BadData)) {
            return;
        }

        if val.is_null() || val.is_empty_value() {
            agg_data.set_result(Value::Int(0));
            return;
        }

        let current = match agg_data.result() {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            Value::Null(crate::core::value::NullType::NaN) => 0.0,
            _ => return,
        };

        agg_data.set_result(Value::Int((current + 1.0) as i64));
    }

    fn apply_sum(agg_data: &mut AggData, val: &Value) {
        if !matches!(val, Value::Int(_) | Value::Float(_)) && !val.is_null() && !val.is_empty_value() {
            agg_data.set_result(Value::Null(crate::core::value::NullType::BadData));
            return;
        }

        if val.is_null() || val.is_empty_value() {
            return;
        }

        let res = agg_data.result();
        match res {
            Value::Null(_) => {
                agg_data.set_result(val.clone());
            }
            _ => {
                let sum_val = res.as_numeric_value().unwrap_or(0.0) + val.as_numeric_value().unwrap_or(0.0);
                agg_data.set_result(Value::Float(sum_val));
            }
        }
    }

    fn apply_avg(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(crate::core::value::NullType::BadData)) {
            return;
        }

        if !val.is_numeric_value() && !val.is_null() && !val.is_empty_value() {
            agg_data.set_result(Value::Null(crate::core::value::NullType::BadData));
            return;
        }

        if val.is_null() || val.is_empty_value() {
            return;
        }

        let val_numeric = val.as_numeric_value().unwrap_or(0.0);

        let mut sum = agg_data.sum.take().unwrap_or(Value::Float(0.0));
        let mut cnt = agg_data.cnt.take().unwrap_or(Value::Float(0.0));

        if let (Some(sum_f), Some(cnt_f)) = (sum.as_numeric_mut(), cnt.as_numeric_mut()) {
            *sum_f += val_numeric;
            *cnt_f += 1.0;

            let avg = if *cnt_f > 0.0 { *sum_f / *cnt_f } else { 0.0 };
            agg_data.set_result(Value::Float(avg));
        }

        agg_data.sum = Some(sum);
        agg_data.cnt = Some(cnt);
    }

    fn apply_min(agg_data: &mut AggData, val: &Value) {
        if val.is_null() || val.is_empty_value() {
            return;
        }

        let res = agg_data.result();
        if matches!(res, Value::Null(crate::core::value::NullType::BadData)) {
            return;
        }

        match res {
            Value::Null(_) => {
                agg_data.set_result(val.clone());
            }
            _ => {
                if Self::value_less_than(val, res) {
                    agg_data.set_result(val.clone());
                }
            }
        }
    }

    fn apply_max(agg_data: &mut AggData, val: &Value) {
        if val.is_null() || val.is_empty_value() {
            return;
        }

        let res = agg_data.result();
        if matches!(res, Value::Null(crate::core::value::NullType::BadData)) {
            return;
        }

        match res {
            Value::Null(_) => {
                agg_data.set_result(val.clone());
            }
            _ => {
                if Self::value_greater_than(val, res) {
                    agg_data.set_result(val.clone());
                }
            }
        }
    }

    fn apply_collect(agg_data: &mut AggData, val: &Value) {
        let current: Vec<Value> = match agg_data.result() {
            Value::List(list) => list.clone(),
            Value::Null(_) => Vec::new(),
            _ => return,
        };

        let mut new_list = current;
        new_list.push(val.clone());
        agg_data.set_result(Value::List(new_list));
    }

    fn apply_distinct(agg_data: &mut AggData, val: &Value) {
        if val.is_null() || val.is_empty_value() {
            return;
        }

        let set = agg_data.distinct_set_mut();
        let key = format!("{:?}", val);

        if set.insert(key) {
            let current: Vec<Value> = match agg_data.result() {
                Value::List(list) => list.clone(),
                Value::Null(_) => Vec::new(),
                _ => return,
            };

            let mut new_list = current;
            new_list.push(val.clone());
            agg_data.set_result(Value::List(new_list));
        }
    }

    fn apply_percentile(agg_data: &mut AggData, val: &Value, _percentile: f64) {
        let numeric_values: Vec<f64> = match &agg_data.result() {
            Value::List(list) => list
                .iter()
                .filter_map(|v| v.as_numeric_value())
                .collect(),
            Value::Null(_) => Vec::new(),
            _ => return,
        };

        if let Some(n) = val.as_numeric_value() {
            let mut new_list = numeric_values;
            new_list.push(n);
            new_list.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            if let Some(p) = Some(_percentile) {
                let idx = ((p / 100.0) * (new_list.len() as f64 - 1.0)).round() as usize;
                let idx = std::cmp::min(idx, new_list.len().saturating_sub(1));

                if !new_list.is_empty() {
                    agg_data.set_result(Value::Float(new_list[idx]));
                }
            }
        }
    }

    fn apply_std(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(crate::core::value::NullType::BadData)) {
            return;
        }

        if !val.is_numeric_value() && !val.is_null() && !val.is_empty_value() {
            agg_data.set_result(Value::Null(crate::core::value::NullType::BadData));
            return;
        }

        if val.is_null() || val.is_empty_value() {
            return;
        }

        let numeric_val = val.as_numeric_value().unwrap_or(0.0);

        let mut cnt = agg_data.cnt.take().unwrap_or(Value::Float(0.0));
        let mut avg = agg_data.avg.take().unwrap_or(Value::Float(0.0));
        let mut deviation = agg_data.deviation.take().unwrap_or(Value::Float(0.0));

        if let (Some(cnt_f), Some(avg_f), Some(deviation_f)) = (
            cnt.as_numeric_mut(),
            avg.as_numeric_mut(),
            deviation.as_numeric_mut(),
        ) {
            if *cnt_f == 0.0 {
                *cnt_f = 1.0;
                *avg_f = numeric_val;
                *deviation_f = 0.0;
            } else {
                *cnt_f += 1.0;
                let old_avg = *avg_f;
                *avg_f += (numeric_val - old_avg) / *cnt_f;
                *deviation_f = (*cnt_f - 1.0) / (*cnt_f * *cnt_f) * ((numeric_val - old_avg) * (numeric_val - old_avg))
                    + (*cnt_f - 1.0) / *cnt_f * *deviation_f;
            }

            let stdev = if *deviation_f > 0.0 { deviation_f.sqrt() } else { 0.0 };
            agg_data.set_result(Value::Float(stdev));
        }

        agg_data.cnt = Some(cnt);
        agg_data.avg = Some(avg);
        agg_data.deviation = Some(deviation);
    }

    fn apply_bit_and(agg_data: &mut AggData, val: &Value) {
        if !matches!(val, Value::Int(_)) && !val.is_null() && !val.is_empty_value() {
            agg_data.set_result(Value::Null(crate::core::value::NullType::BadData));
            return;
        }

        if val.is_null() || val.is_empty_value() {
            return;
        }

        let res = agg_data.result();
        match res {
            Value::Null(_) => {
                agg_data.set_result(val.clone());
            }
            Value::Int(i) => {
                if let Value::Int(v) = val {
                    agg_data.set_result(Value::Int(i & v));
                }
            }
            _ => {}
        }
    }

    fn apply_bit_or(agg_data: &mut AggData, val: &Value) {
        if !matches!(val, Value::Int(_)) && !val.is_null() && !val.is_empty() {
            agg_data.set_result(Value::Null(crate::core::value::NullType::BadData));
            return;
        }

        if val.is_null() || val.is_empty() {
            return;
        }

        let res = agg_data.result();
        match res {
            Value::Null(_) => {
                agg_data.set_result(val.clone());
            }
            Value::Int(i) => {
                if let Value::Int(v) = val {
                    agg_data.set_result(Value::Int(i | v));
                }
            }
            _ => {}
        }
    }

    fn apply_group_concat(agg_data: &mut AggData, val: &Value, separator: &str) {
        if val.is_null() || val.is_empty_value() {
            return;
        }

        let current: String = match agg_data.result() {
            Value::String(s) => s.clone(),
            Value::Null(_) => String::new(),
            _ => return,
        };

        let val_str = match val {
            Value::String(s) => s.clone(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            _ => format!("{:?}", val),
        };

        if current.is_empty() {
            agg_data.set_result(Value::String(val_str));
        } else {
            agg_data.set_result(Value::String(current + separator + &val_str));
        }
    }

    fn value_less_than(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(a_int), Value::Int(b_int)) => a_int < b_int,
            (Value::Float(a_float), Value::Float(b_float)) => a_float < b_float,
            (Value::Int(a_int), Value::Float(b_float)) => (*a_int as f64) < *b_float,
            (Value::Float(a_float), Value::Int(b_int)) => *a_float < (*b_int as f64),
            (Value::String(a_str), Value::String(b_str)) => a_str < b_str,
            _ => false,
        }
    }

    fn value_greater_than(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(a_int), Value::Int(b_int)) => a_int > b_int,
            (Value::Float(a_float), Value::Float(b_float)) => a_float > b_float,
            (Value::Int(a_int), Value::Float(b_float)) => (*a_int as f64) > *b_float,
            (Value::Float(a_float), Value::Int(b_int)) => *a_float > (*b_int as f64),
            (Value::String(a_str), Value::String(b_str)) => a_str > b_str,
            _ => false,
        }
    }

    fn build_result(&self, result: HashMap<GroupKey, Vec<AggData>>) -> DBResult<ExecutionResult> {
        let mut rows = Vec::new();

        for (group_key, agg_datas) in result {
            let mut row_values = Vec::new();

            if !self.group_by_keys.is_empty() {
                for key in group_key.keys() {
                    row_values.push(key.clone());
                }
            }

            for agg_data in agg_datas {
                row_values.push(agg_data.result().clone());
            }

            rows.push(Value::List(row_values));
        }

        Ok(ExecutionResult::Values(rows))
    }

    fn extract_field_value(&self, value: &Value, field_name: &str) -> DBResult<Value> {
        match value {
            Value::Vertex(vertex) => {
                if let Some(prop_value) = vertex.get_property_any(field_name) {
                    Ok(prop_value.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::NaN))
                }
            }
            Value::Edge(edge) => {
                if let Some(prop_value) = edge.get_property(field_name) {
                    Ok(prop_value.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::NaN))
                }
            }
            Value::Map(map) => {
                if let Some(prop_value) = map.get(field_name) {
                    Ok(prop_value.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::NaN))
                }
            }
            _ => Ok(value.clone()),
        }
    }
}

trait ValueUtils {
    fn is_empty_value(&self) -> bool;
    fn is_numeric_value(&self) -> bool;
    fn as_numeric_value(&self) -> Option<f64>;
    fn as_numeric_mut(&mut self) -> Option<&mut f64>;
}

impl ValueUtils for Value {
    fn is_empty_value(&self) -> bool {
        match self {
            Value::Null(_) => true,
            Value::String(s) => s.is_empty(),
            Value::List(l) => l.is_empty(),
            Value::Map(m) => m.is_empty(),
            _ => false,
        }
    }

    fn is_numeric_value(&self) -> bool {
        matches!(self, Value::Int(_) | Value::Float(_))
    }

    fn as_numeric_value(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    fn as_numeric_mut(&mut self) -> Option<&mut f64> {
        match self {
            Value::Float(f) => Some(f),
            _ => None,
        }
    }
}

impl<S: StorageClient> Executor<S> for AggregationExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage_clone = self.get_storage().clone();
        let storage = safe_lock(&storage_clone)?;

        let input_data = self.get_input_data(&storage)?;
        self.execute_aggregation(&input_data)
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Aggregation executor - performs statistical aggregation operations with proper null handling"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> AggregationExecutor<S> {
    fn get_input_data(&self, storage: &S) -> DBResult<Vec<Value>> {
        let input_result = storage.get_input(&self.input_var)?;

        match input_result {
            Some(data) => {
                let mut values = Vec::new();
                for item in data {
                    values.push(item);
                }
                Ok(values)
            }
            None => {
                Ok(Vec::new())
            }
        }
    }
}

impl<S: StorageClient> HasStorage<S> for AggregationExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("存储未初始化")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Value, vertex_edge_path::Vertex};
    use crate::core::vertex_edge_path::Tag;
    use crate::core::types::operators::AggregateFunction;
    use std::collections::HashMap;

    #[test]
    fn test_agg_data_creation() {
        let agg_data = AggData::new();
        assert!(matches!(agg_data.result(), Value::Null(_)));
        assert!(agg_data.sum().is_none());
        assert!(agg_data.cnt().is_none());
    }

    #[test]
    fn test_agg_data_sum_update() {
        let mut agg_data = AggData::new();
        agg_data.set_result(Value::Float(10.0));
        *agg_data.sum_mut() = Value::Float(10.0);
        assert_eq!(agg_data.sum(), Some(&Value::Float(10.0)));
    }

    #[test]
    fn test_expression_context_basic() {
        let mut ctx = ExpressionContext::new();
        ctx.set_variable("name", Value::String("test".to_string()));
        assert_eq!(
            ctx.variables.get("name"),
            Some(&Value::String("test".to_string()))
        );
    }

    #[test]
    fn test_group_key_hash() {
        let key1 = GroupKey::new(vec![Value::Int(1), Value::String("a".to_string())]);
        let key2 = GroupKey::new(vec![Value::Int(1), Value::String("a".to_string())]);
        let key3 = GroupKey::new(vec![Value::Int(2), Value::String("a".to_string())]);

        let mut map: HashMap<GroupKey, i32> = HashMap::new();
        map.insert(key1, 1);
        assert_eq!(map.get(&key2), Some(&1));
        assert_eq!(map.get(&key3), None);
    }

    #[test]
    fn test_apply_count() {
        let mut agg_data = AggData::new();
        AggregationExecutor::<crate::storage::MockStorage>::apply_count(&mut agg_data, &Value::Int(1));
        assert_eq!(agg_data.result(), &Value::Int(1));

        AggregationExecutor::<crate::storage::MockStorage>::apply_count(&mut agg_data, &Value::Int(2));
        assert_eq!(agg_data.result(), &Value::Int(2));
    }

    #[test]
    fn test_apply_count_with_null() {
        let mut agg_data = AggData::new();
        AggregationExecutor::<crate::storage::MockStorage>::apply_count(&mut agg_data, &Value::Null(crate::core::value::NullType::NaN));
        assert_eq!(agg_data.result(), &Value::Int(0));

        AggregationExecutor::<crate::storage::MockStorage>::apply_count(&mut agg_data, &Value::String("test".to_string()));
        assert_eq!(agg_data.result(), &Value::Int(1));
    }

    #[test]
    fn test_apply_sum() {
        let mut agg_data = AggData::new();
        AggregationExecutor::<crate::storage::MockStorage>::apply_sum(&mut agg_data, &Value::Int(10));
        assert_eq!(agg_data.result(), &Value::Int(10));

        AggregationExecutor::<crate::storage::MockStorage>::apply_sum(&mut agg_data, &Value::Int(20));
        assert_eq!(agg_data.result(), &Value::Float(30.0));
    }

    #[test]
    fn test_apply_sum_with_bad_data() {
        let mut agg_data = AggData::new();
        AggregationExecutor::<crate::storage::MockStorage>::apply_sum(&mut agg_data, &Value::String("abc".to_string()));
        assert!(matches!(agg_data.result(), Value::Null(crate::core::value::NullType::BadData)));
    }

    #[test]
    fn test_apply_avg() {
        let mut agg_data = AggData::new();
        AggregationExecutor::<crate::storage::MockStorage>::apply_avg(&mut agg_data, &Value::Int(10));
        AggregationExecutor::<crate::storage::MockStorage>::apply_avg(&mut agg_data, &Value::Int(20));
        AggregationExecutor::<crate::storage::MockStorage>::apply_avg(&mut agg_data, &Value::Int(30));

        assert_eq!(agg_data.result(), &Value::Float(20.0));
    }

    #[test]
    fn test_apply_min() {
        let mut agg_data = AggData::new();
        AggregationExecutor::<crate::storage::MockStorage>::apply_min(&mut agg_data, &Value::Int(10));
        assert_eq!(agg_data.result(), &Value::Int(10));

        AggregationExecutor::<crate::storage::MockStorage>::apply_min(&mut agg_data, &Value::Int(5));
        assert_eq!(agg_data.result(), &Value::Int(5));

        AggregationExecutor::<crate::storage::MockStorage>::apply_min(&mut agg_data, &Value::Int(15));
        assert_eq!(agg_data.result(), &Value::Int(5));
    }

    #[test]
    fn test_apply_max() {
        let mut agg_data = AggData::new();
        AggregationExecutor::<crate::storage::MockStorage>::apply_max(&mut agg_data, &Value::Int(10));
        assert_eq!(agg_data.result(), &Value::Int(10));

        AggregationExecutor::<crate::storage::MockStorage>::apply_max(&mut agg_data, &Value::Int(5));
        assert_eq!(agg_data.result(), &Value::Int(10));

        AggregationExecutor::<crate::storage::MockStorage>::apply_max(&mut agg_data, &Value::Int(15));
        assert_eq!(agg_data.result(), &Value::Int(15));
    }

    #[test]
    fn test_apply_collect() {
        let mut agg_data = AggData::new();
        AggregationExecutor::<crate::storage::MockStorage>::apply_collect(&mut agg_data, &Value::Int(1));
        AggregationExecutor::<crate::storage::MockStorage>::apply_collect(&mut agg_data, &Value::Int(2));

        match agg_data.result() {
            Value::List(list) => {
                assert_eq!(list.len(), 2);
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_apply_distinct() {
        let mut agg_data = AggData::new();
        AggregationExecutor::<crate::storage::MockStorage>::apply_distinct(&mut agg_data, &Value::Int(1));
        AggregationExecutor::<crate::storage::MockStorage>::apply_distinct(&mut agg_data, &Value::Int(1));
        AggregationExecutor::<crate::storage::MockStorage>::apply_distinct(&mut agg_data, &Value::Int(2));

        match agg_data.result() {
            Value::List(list) => {
                assert_eq!(list.len(), 2);
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_apply_bit_and() {
        let mut agg_data = AggData::new();
        AggregationExecutor::<crate::storage::MockStorage>::apply_bit_and(&mut agg_data, &Value::Int(7));
        assert_eq!(agg_data.result(), &Value::Int(7));

        AggregationExecutor::<crate::storage::MockStorage>::apply_bit_and(&mut agg_data, &Value::Int(3));
        assert_eq!(agg_data.result(), &Value::Int(3));
    }

    #[test]
    fn test_apply_bit_or() {
        let mut agg_data = AggData::new();
        AggregationExecutor::<crate::storage::MockStorage>::apply_bit_or(&mut agg_data, &Value::Int(1));
        assert_eq!(agg_data.result(), &Value::Int(1));

        AggregationExecutor::<crate::storage::MockStorage>::apply_bit_or(&mut agg_data, &Value::Int(2));
        assert_eq!(agg_data.result(), &Value::Int(3));
    }

    #[test]
    fn test_apply_group_concat() {
        let mut agg_data = AggData::new();
        AggregationExecutor::<crate::storage::MockStorage>::apply_group_concat(&mut agg_data, &Value::String("a".to_string()), ",");
        AggregationExecutor::<crate::storage::MockStorage>::apply_group_concat(&mut agg_data, &Value::String("b".to_string()), ",");
        AggregationExecutor::<crate::storage::MockStorage>::apply_group_concat(&mut agg_data, &Value::String("c".to_string()), ",");

        assert_eq!(agg_data.result(), &Value::String("a,b,c".to_string()));
    }

    #[test]
    fn test_execute_aggregation_executor_creation() {
        let storage = Arc::new(Mutex::new(crate::storage::MockStorage));
        let agg_funcs = vec![AggregateFunction::Count(None)];
        let group_by_keys = vec!["category".to_string()];

        let executor = AggregationExecutor::new(
            1,
            storage,
            "input_var".to_string(),
            agg_funcs,
            group_by_keys,
            vec!["count".to_string()],
            "output_var".to_string(),
        );

        assert_eq!(executor.id(), 1);
        assert_eq!(executor.name(), "AggregationExecutor");
    }

    #[test]
    fn test_extract_field_value_from_vertex() {
        let mut properties = HashMap::new();
        properties.insert("age".to_string(), Value::Int(25));
        properties.insert("name".to_string(), Value::String("Alice".to_string()));

        let tag = Tag::new("person".to_string(), properties);
        let vertex = Vertex::new_with_properties(
            Value::String("vertex1".to_string()),
            vec![tag],
            HashMap::new(),
        );

        let executor = AggregationExecutor {
            base: BaseExecutor::new(1, "test".to_string(), Arc::new(Mutex::new(crate::storage::MockStorage))),
            input_var: "test_input".to_string(),
            aggregation_functions: vec![],
            group_by_keys: vec![],
            col_names: vec![],
            output_var: "test_output".to_string(),
        };

        let result = executor.extract_field_value(&Value::Vertex(Box::new(vertex)), "age").expect("extract_field_value should succeed");
        assert_eq!(result, Value::Int(25));
    }

    #[tokio::test]
    async fn test_handle_empty_input() {
        let storage = Arc::new(Mutex::new(crate::storage::MockStorage));
        let agg_funcs = vec![AggregateFunction::Count(None)];

        let executor = AggregationExecutor::new(
            1,
            storage,
            "test_input".to_string(),
            agg_funcs,
            vec![],
            vec!["count".to_string()],
            "test_output".to_string(),
        );

        let result = executor.handle_empty_input().expect("handle_empty_input should succeed");
        match result {
            ExecutionResult::Values(values) => {
                assert_eq!(values.len(), 1);
                match &values[0] {
                    Value::List(list) => {
                        assert_eq!(list.len(), 1);
                        assert_eq!(list[0], Value::Int(0));
                    }
                    _ => panic!("Expected List"),
                }
            }
            _ => panic!("Expected Values"),
        }
    }
}
