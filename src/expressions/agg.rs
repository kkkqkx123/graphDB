use std::collections::{HashMap, HashSet};
use crate::core::{Value, NullType};

/// Aggregation data to maintain state during aggregation
pub struct AggData {
    cnt: Value,
    sum: Value,
    avg: Value,
    deviation: Value,
    result: Value,
    uniques: HashSet<Value>,
}

impl Default for AggData {
    fn default() -> Self {
        Self {
            cnt: Value::Null(NullType::NaN),
            sum: Value::Null(NullType::NaN),
            avg: Value::Null(NullType::NaN),
            deviation: Value::Null(NullType::NaN),
            result: Value::Null(NullType::NaN),
            uniques: HashSet::new(),
        }
    }
}

impl AggData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cnt(&self) -> &Value {
        &self.cnt
    }

    pub fn cnt_mut(&mut self) -> &mut Value {
        &mut self.cnt
    }

    pub fn set_cnt(&mut self, cnt: Value) {
        self.cnt = cnt;
    }

    pub fn sum(&self) -> &Value {
        &self.sum
    }

    pub fn sum_mut(&mut self) -> &mut Value {
        &mut self.sum
    }

    pub fn set_sum(&mut self, sum: Value) {
        self.sum = sum;
    }

    pub fn avg(&self) -> &Value {
        &self.avg
    }

    pub fn avg_mut(&mut self) -> &mut Value {
        &mut self.avg
    }

    pub fn set_avg(&mut self, avg: Value) {
        self.avg = avg;
    }

    pub fn deviation(&self) -> &Value {
        &self.deviation
    }

    pub fn deviation_mut(&mut self) -> &mut Value {
        &mut self.deviation
    }

    pub fn set_deviation(&mut self, deviation: Value) {
        self.deviation = deviation;
    }

    pub fn result(&self) -> &Value {
        &self.result
    }

    pub fn result_mut(&mut self) -> &mut Value {
        &mut self.result
    }

    pub fn set_result(&mut self, result: Value) {
        self.result = result;
    }

    pub fn uniques(&self) -> &HashSet<Value> {
        &self.uniques
    }

    pub fn uniques_mut(&mut self) -> &mut HashSet<Value> {
        &mut self.uniques
    }

    pub fn set_uniques(&mut self, uniques: HashSet<Value>) {
        self.uniques = uniques;
    }
}

/// Type alias for aggregation functions
pub type AggFunction = fn(&mut AggData, &Value);

/// The aggregate function manager
pub struct AggFunctionManager {
    functions: HashMap<String, AggFunction>,
}

impl AggFunctionManager {
    /// Get a singleton instance of the aggregate function manager
    pub fn instance() -> &'static Self {
        use std::sync::{Once, OnceLock};
        static INSTANCE: OnceLock<AggFunctionManager> = OnceLock::new();
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            INSTANCE.get_or_init(|| {
                let mut manager = AggFunctionManager {
                    functions: HashMap::new(),
                };
                
                // Initialize built-in functions
                manager.functions.insert("COUNT".to_string(), Self::count_func);
                manager.functions.insert("SUM".to_string(), Self::sum_func);
                manager.functions.insert("AVG".to_string(), Self::avg_func);
                manager.functions.insert("MAX".to_string(), Self::max_func);
                manager.functions.insert("MIN".to_string(), Self::min_func);
                manager.functions.insert("STD".to_string(), Self::std_func);
                manager.functions.insert("BIT_AND".to_string(), Self::bit_and_func);
                manager.functions.insert("BIT_OR".to_string(), Self::bit_or_func);
                manager.functions.insert("BIT_XOR".to_string(), Self::bit_xor_func);
                manager.functions.insert("COLLECT".to_string(), Self::collect_func);
                manager.functions.insert("COLLECT_SET".to_string(), Self::collect_set_func);
                
                manager
            });
        });

        INSTANCE.get().unwrap()
    }

    /// Get an aggregate function by name
    pub fn get(name: &str) -> Option<AggFunction> {
        let upper_name = name.to_uppercase();
        Self::instance().functions.get(&upper_name).copied()
    }

    /// Check if a function exists
    pub fn exists(name: &str) -> bool {
        let upper_name = name.to_uppercase();
        Self::instance().functions.contains_key(&upper_name)
    }

    // Built-in aggregate functions
    fn count_func(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(NullType::Null) | Value::Null(NullType::BadType)) {
            return;
        }

        if matches!(agg_data.result(), Value::Null(_)) {
            *agg_data.result_mut() = Value::Int(0);
        }

        if matches!(val, Value::Null(_) | Value::Empty) {
            return;
        }

        if let (Value::Int(_), Value::Int(1)) = (agg_data.result(), Value::Int(1)) {
            if let Value::Int(ref mut current_count) = agg_data.result_mut() {
                *current_count += 1;
            }
        } else if let Value::Int(1) = val {
            *agg_data.result_mut() = Value::Int(1);
        } else {
            // For other types, just increment if the current result is an integer
            if let Value::Int(ref mut current_count) = agg_data.result_mut() {
                *current_count += 1;
            } else {
                *agg_data.result_mut() = Value::Int(1); // Start with 1 if first value
            }
        }
    }

    fn sum_func(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(NullType::Null) | Value::Null(NullType::BadType)) {
            return;
        }

        if matches!(agg_data.result(), Value::Null(_)) {
            *agg_data.result_mut() = Value::Int(0); // Initialize sum to 0
        }

        if matches!(val, Value::Null(_) | Value::Empty) {
            return;
        }

        // Only process numeric values
        match (agg_data.result_mut(), val) {
            (Value::Int(ref mut sum), Value::Int(addend)) => {
                *sum += addend;
            }
            (Value::Float(ref mut sum), Value::Float(addend)) => {
                *sum += addend;
            }
            (Value::Float(ref mut sum), Value::Int(addend)) => {
                *sum += *addend as f64;
            }
            (Value::Int(ref mut sum), Value::Float(addend)) => {
                *sum = (*sum as f64 + addend) as i64;
            }
            _ => {
                // Type mismatch, set result to bad type
                *agg_data.result_mut() = Value::Null(NullType::BadType);
            }
        }
    }

    fn avg_func(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(NullType::Null) | Value::Null(NullType::BadType)) {
            return;
        }

        if matches!(val, Value::Null(_) | Value::Empty) {
            return;
        }

        // Type check for numeric values
        if !matches!(val, Value::Int(_) | Value::Float(_)) {
            *agg_data.result_mut() = Value::Null(NullType::BadType);
            return;
        }

        if matches!(agg_data.result(), Value::Null(_)) {
            *agg_data.result_mut() = Value::Float(0.0);
            *agg_data.sum_mut() = Value::Float(0.0);
            *agg_data.cnt_mut() = Value::Float(0.0);
        }

        // Update sum and count
        match (agg_data.sum_mut(), val) {
            (Value::Float(ref mut s), Value::Int(v)) => *s += *v as f64,
            (Value::Float(ref mut s), Value::Float(v)) => *s += v,
            (Value::Int(ref mut s), Value::Int(v)) => *s += v,
            (Value::Int(ref mut s), Value::Float(v)) => {
                *agg_data.sum_mut() = Value::Float(*s as f64 + v);
            }
            _ => {
                // Type mismatch, set result to bad type
                *agg_data.result_mut() = Value::Null(NullType::BadType);
                return;
            }
        }

        // Update count
        if let Value::Float(ref mut c) = agg_data.cnt_mut() {
            *c += 1.0;
        } else if let Value::Int(ref mut c) = agg_data.cnt_mut() {
            *c += 1;
        }

        // Calculate average
        match (agg_data.sum(), agg_data.cnt()) {
            (Value::Float(s), Value::Float(c)) if *c != 0.0 => {
                *agg_data.result_mut() = Value::Float(s / c);
            }
            (Value::Int(s), Value::Int(c)) if *c != 0 => {
                *agg_data.result_mut() = Value::Float(*s as f64 / *c as f64);
            }
            _ => {
                *agg_data.result_mut() = Value::Null(NullType::NaN);
            }
        }
    }

    fn max_func(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(NullType::Null) | Value::Null(NullType::BadType)) {
            return;
        }

        if matches!(val, Value::Null(_) | Value::Empty) {
            return;
        }

        if matches!(agg_data.result(), Value::Null(_)) {
            *agg_data.result_mut() = val.clone();
            return;
        }

        // Compare to update max
        if val > agg_data.result() {
            *agg_data.result_mut() = val.clone();
        }
    }

    fn min_func(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(NullType::Null) | Value::Null(NullType::BadType)) {
            return;
        }

        if matches!(val, Value::Null(_) | Value::Empty) {
            return;
        }

        if matches!(agg_data.result(), Value::Null(_)) {
            *agg_data.result_mut() = val.clone();
            return;
        }

        // Compare to update min
        if val < agg_data.result() {
            *agg_data.result_mut() = val.clone();
        }
    }

    fn std_func(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(NullType::Null) | Value::Null(NullType::BadType)) {
            return;
        }

        // Type check for numeric values
        if !matches!(val, Value::Int(_) | Value::Float(_)) {
            *agg_data.result_mut() = Value::Null(NullType::BadType);
            return;
        }

        if matches!(val, Value::Null(_) | Value::Empty) {
            return;
        }

        if matches!(agg_data.result(), Value::Null(_)) {
            *agg_data.result_mut() = Value::Float(0.0);
            *agg_data.cnt_mut() = Value::Float(0.0);
            *agg_data.avg_mut() = Value::Float(0.0);
            *agg_data.deviation_mut() = Value::Float(0.0);
        }

        // Increment count
        if let Value::Float(ref mut c) = agg_data.cnt_mut() {
            *c += 1.0;
        }

        // Update average
        let val_f = match val {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            _ => 0.0, // Shouldn't happen due to earlier check
        };
        let cnt_f = match agg_data.cnt() {
            Value::Float(c) => *c,
            _ => 1.0, // fallback
        };
        if let Value::Float(ref mut a) = agg_data.avg_mut() {
            *a = *a + (val_f - *a) / cnt_f;
        }

        // Update deviation
        let current_avg = match agg_data.avg() {
            Value::Float(a) => *a,
            _ => 0.0, // fallback
        };
        if let Value::Float(ref mut dev) = agg_data.deviation_mut() {
            *dev = (cnt_f - 1.0) / (cnt_f * cnt_f) * ((val_f - current_avg) * (val_f - current_avg))
                + (cnt_f - 1.0) / cnt_f * *dev;
        }

        // Set standard deviation as square root of variance
        if let Value::Float(dev) = agg_data.deviation() {
            *agg_data.result_mut() = Value::Float(dev.sqrt());
        }
    }

    fn bit_and_func(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(NullType::Null) | Value::Null(NullType::BadType)) {
            return;
        }

        // Type check for integer values
        if !matches!(val, Value::Int(_)) {
            *agg_data.result_mut() = Value::Null(NullType::BadType);
            return;
        }

        if matches!(val, Value::Null(_) | Value::Empty) {
            return;
        }

        if matches!(agg_data.result(), Value::Null(_)) {
            *agg_data.result_mut() = val.clone();
            return;
        }

        if let (Value::Int(ref mut res), Value::Int(v)) = (agg_data.result_mut(), val) {
            *res = *res & v;
        }
    }

    fn bit_or_func(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(NullType::Null) | Value::Null(NullType::BadType)) {
            return;
        }

        // Type check for integer values
        if !matches!(val, Value::Int(_)) {
            *agg_data.result_mut() = Value::Null(NullType::BadType);
            return;
        }

        if matches!(val, Value::Null(_) | Value::Empty) {
            return;
        }

        if matches!(agg_data.result(), Value::Null(_)) {
            *agg_data.result_mut() = val.clone();
            return;
        }

        if let (Value::Int(ref mut res), Value::Int(v)) = (agg_data.result_mut(), val) {
            *res = *res | v;
        }
    }

    fn bit_xor_func(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(NullType::Null) | Value::Null(NullType::BadType)) {
            return;
        }

        // Type check for integer values
        if !matches!(val, Value::Int(_)) {
            *agg_data.result_mut() = Value::Null(NullType::BadType);
            return;
        }

        if matches!(val, Value::Null(_) | Value::Empty) {
            return;
        }

        if matches!(agg_data.result(), Value::Null(_)) {
            *agg_data.result_mut() = val.clone();
            return;
        }

        if let (Value::Int(ref mut res), Value::Int(v)) = (agg_data.result_mut(), val) {
            *res = *res ^ v;
        }
    }

    fn collect_func(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(NullType::Null) | Value::Null(NullType::BadType)) {
            return;
        }

        if matches!(val, Value::Null(_) | Value::Empty) {
            return;
        }

        if matches!(agg_data.result(), Value::Null(_)) {
            *agg_data.result_mut() = Value::List(vec![]);
        }

        if let Value::List(ref mut list) = agg_data.result_mut() {
            list.push(val.clone());
        } else {
            // Type mismatch
            *agg_data.result_mut() = Value::Null(NullType::BadType);
        }
    }

    fn collect_set_func(agg_data: &mut AggData, val: &Value) {
        if matches!(agg_data.result(), Value::Null(NullType::Null) | Value::Null(NullType::BadType)) {
            return;
        }

        if matches!(val, Value::Null(_) | Value::Empty) {
            return;
        }

        if matches!(agg_data.result(), Value::Null(_)) {
            *agg_data.result_mut() = Value::Set(std::collections::HashSet::new());
        }

        if let Value::Set(ref mut set) = agg_data.result_mut() {
            set.insert(val.clone());
        } else {
            // Type mismatch
            *agg_data.result_mut() = Value::Null(NullType::BadType);
        }
    }
}