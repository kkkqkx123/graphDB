use super::dataset::DataSet;
use super::date_time::{DateTimeValue, DateValue, DurationValue, TimeValue};
use super::geography::GeographyValue;
use super::list::List;
use super::null::NullType;
use super::value::Value;
use crate::core::DataType;
use std::cmp::Ordering as CmpOrdering;
use std::collections::HashMap;
use std::hash::Hash;

// Manual implementation of PartialEq to handle f64 comparisons correctly
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Empty, Value::Empty) => true,
            (Value::Null(a), Value::Null(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Int8(a), Value::Int8(b)) => a == b,
            (Value::Int16(a), Value::Int16(b)) => a == b,
            (Value::Int32(a), Value::Int32(b)) => a == b,
            (Value::Int64(a), Value::Int64(b)) => a == b,
            (Value::UInt8(a), Value::UInt8(b)) => a == b,
            (Value::UInt16(a), Value::UInt16(b)) => a == b,
            (Value::UInt32(a), Value::UInt32(b)) => a == b,
            (Value::UInt64(a), Value::UInt64(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a == b) || (a.is_nan() && b.is_nan()), // Proper handling of NaN
            (Value::Decimal128(a), Value::Decimal128(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::FixedString { data: a, .. }, Value::FixedString { data: b, .. }) => a == b,
            (Value::Date(a), Value::Date(b)) => a == b,
            (Value::Time(a), Value::Time(b)) => a == b,
            (Value::DateTime(a), Value::DateTime(b)) => a == b,
            (Value::Vertex(a), Value::Vertex(b)) => a == b,
            (Value::Edge(a), Value::Edge(b)) => a == b,
            (Value::Path(a), Value::Path(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Set(a), Value::Set(b)) => a == b,
            (Value::Geography(a), Value::Geography(b)) => a == b,
            (Value::Duration(a), Value::Duration(b)) => a == b,

            // Comparison between integer types: comparison after conversion to i64
            (Value::Int(a), Value::Int8(b)) => *a == *b as i64,
            (Value::Int(a), Value::Int16(b)) => *a == *b as i64,
            (Value::Int(a), Value::Int32(b)) => *a == *b as i64,
            (Value::Int(a), Value::Int64(b)) => *a == *b,
            (Value::Int8(a), Value::Int(b)) => *a as i64 == *b,
            (Value::Int16(a), Value::Int(b)) => *a as i64 == *b,
            (Value::Int32(a), Value::Int(b)) => *a as i64 == *b,
            (Value::Int64(a), Value::Int(b)) => *a == *b,
            (Value::Int8(a), Value::Int16(b)) => *a as i64 == *b as i64,
            (Value::Int8(a), Value::Int32(b)) => *a as i64 == *b as i64,
            (Value::Int8(a), Value::Int64(b)) => *a as i64 == *b,
            (Value::Int16(a), Value::Int8(b)) => *a as i64 == *b as i64,
            (Value::Int16(a), Value::Int32(b)) => *a as i64 == *b as i64,
            (Value::Int16(a), Value::Int64(b)) => *a as i64 == *b,
            (Value::Int32(a), Value::Int8(b)) => *a as i64 == *b as i64,
            (Value::Int32(a), Value::Int16(b)) => *a as i64 == *b as i64,
            (Value::Int32(a), Value::Int64(b)) => *a as i64 == *b,
            (Value::Int64(a), Value::Int8(b)) => *a == *b as i64,
            (Value::Int64(a), Value::Int16(b)) => *a == *b as i64,
            (Value::Int64(a), Value::Int32(b)) => *a == *b as i64,

            _ => false,
        }
    }
}

// Eq is implemented manually, since f64 does not implement Eq
impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // Use actual type comparisons instead of hashes
        match (self, other) {
            // Comparison of the same type
            (Value::Empty, Value::Empty) => CmpOrdering::Equal,
            (Value::Null(a), Value::Null(b)) => Self::cmp_null(a, b),
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Int8(a), Value::Int8(b)) => a.cmp(b),
            (Value::Int16(a), Value::Int16(b)) => a.cmp(b),
            (Value::Int32(a), Value::Int32(b)) => a.cmp(b),
            (Value::Int64(a), Value::Int64(b)) => a.cmp(b),
            (Value::UInt8(a), Value::UInt8(b)) => a.cmp(b),
            (Value::UInt16(a), Value::UInt16(b)) => a.cmp(b),
            (Value::UInt32(a), Value::UInt32(b)) => a.cmp(b),
            (Value::UInt64(a), Value::UInt64(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => Self::cmp_f64(*a, *b),
            (Value::Decimal128(a), Value::Decimal128(b)) => a.cmp(b),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::FixedString { data: a, .. }, Value::FixedString { data: b, .. }) => a.cmp(b),
            (Value::Date(a), Value::Date(b)) => Self::cmp_date(a, b),
            (Value::Time(a), Value::Time(b)) => Self::cmp_time(a, b),
            (Value::DateTime(a), Value::DateTime(b)) => Self::cmp_datetime(a, b),
            (Value::Vertex(a), Value::Vertex(b)) => a.cmp(b),
            (Value::Edge(a), Value::Edge(b)) => a.cmp(b),
            (Value::Path(a), Value::Path(b)) => a.cmp(b),
            (Value::List(a), Value::List(b)) => Self::cmp_list(a, b),
            (Value::Map(a), Value::Map(b)) => Self::cmp_map(a, b),
            (Value::Set(a), Value::Set(b)) => Self::cmp_set(a, b),
            (Value::Geography(a), Value::Geography(b)) => Self::cmp_geography(a, b),
            (Value::Duration(a), Value::Duration(b)) => Self::cmp_duration(a, b),
            (Value::DataSet(a), Value::DataSet(b)) => Self::cmp_dataset(a, b),

            // Comparison between integer types: comparison after conversion to i64
            (Value::Int(a), Value::Int8(b)) => a.cmp(&(*b as i64)),
            (Value::Int(a), Value::Int16(b)) => a.cmp(&(*b as i64)),
            (Value::Int(a), Value::Int32(b)) => a.cmp(&(*b as i64)),
            (Value::Int(a), Value::Int64(b)) => a.cmp(b),
            (Value::Int8(a), Value::Int(b)) => (*a as i64).cmp(b),
            (Value::Int16(a), Value::Int(b)) => (*a as i64).cmp(b),
            (Value::Int32(a), Value::Int(b)) => (*a as i64).cmp(b),
            (Value::Int64(a), Value::Int(b)) => a.cmp(b),
            (Value::Int8(a), Value::Int16(b)) => (*a as i64).cmp(&(*b as i64)),
            (Value::Int8(a), Value::Int32(b)) => (*a as i64).cmp(&(*b as i64)),
            (Value::Int8(a), Value::Int64(b)) => (*a as i64).cmp(b),
            (Value::Int16(a), Value::Int8(b)) => (*a as i64).cmp(&(*b as i64)),
            (Value::Int16(a), Value::Int32(b)) => (*a as i64).cmp(&(*b as i64)),
            (Value::Int16(a), Value::Int64(b)) => (*a as i64).cmp(b),
            (Value::Int32(a), Value::Int8(b)) => (*a as i64).cmp(&(*b as i64)),
            (Value::Int32(a), Value::Int16(b)) => (*a as i64).cmp(&(*b as i64)),
            (Value::Int32(a), Value::Int64(b)) => (*a as i64).cmp(b),
            (Value::Int64(a), Value::Int8(b)) => a.cmp(&(*b as i64)),
            (Value::Int64(a), Value::Int16(b)) => a.cmp(&(*b as i64)),
            (Value::Int64(a), Value::Int32(b)) => a.cmp(&(*b as i64)),

            // Comparison between different types: based on type prioritization
            (a, b) => Self::cmp_by_type_priority(a, b),
        }
    }
}

// Manually implementing Hash to handle f64 hashes
impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Empty => 0u8.hash(state),
            Value::Null(n) => {
                1u8.hash(state);
                n.hash(state);
            }
            Value::Bool(b) => {
                2u8.hash(state);
                b.hash(state);
            }
            Value::Int(i) => {
                3u8.hash(state);
                i.hash(state);
            }
            Value::Int8(i) => {
                4u8.hash(state);
                i.hash(state);
            }
            Value::Int16(i) => {
                5u8.hash(state);
                i.hash(state);
            }
            Value::Int32(i) => {
                6u8.hash(state);
                i.hash(state);
            }
            Value::Int64(i) => {
                7u8.hash(state);
                i.hash(state);
            }
            Value::UInt8(i) => {
                8u8.hash(state);
                i.hash(state);
            }
            Value::UInt16(i) => {
                9u8.hash(state);
                i.hash(state);
            }
            Value::UInt32(i) => {
                10u8.hash(state);
                i.hash(state);
            }
            Value::UInt64(i) => {
                11u8.hash(state);
                i.hash(state);
            }
            Value::Float(f) => {
                12u8.hash(state);
                // Creating a hash from a bitwise representation of a floating point number
                if f.is_nan() {
                    // All NaN values should hash to the same value
                    (0x7ff80000u32 as u64).hash(state);
                } else if *f == 0.0 {
                    // 确保+0.0和-0.0哈希到相同的值
                    0.0_f64.to_bits().hash(state);
                } else {
                    f.to_bits().hash(state);
                }
            }
            Value::Decimal128(d) => {
                19u8.hash(state);
                d.hash(state);
            }
            Value::String(s) => {
                13u8.hash(state);
                s.hash(state);
            }
            Value::FixedString { data, .. } => {
                20u8.hash(state);
                data.hash(state);
            }
            Value::Blob(b) => {
                14u8.hash(state);
                b.hash(state);
            }
            Value::Date(d) => {
                15u8.hash(state);
                d.hash(state);
            }
            Value::Time(t) => {
                16u8.hash(state);
                t.hash(state);
            }
            Value::DateTime(dt) => {
                17u8.hash(state);
                dt.hash(state);
            }
            Value::Vertex(v) => {
                18u8.hash(state);
                v.hash(state);
            }
            Value::Edge(e) => {
                19u8.hash(state);
                e.hash(state);
            }
            Value::Path(p) => {
                12u8.hash(state);
                p.hash(state);
            }
            Value::List(l) => {
                13u8.hash(state);
                l.hash(state);
            }
            Value::Map(m) => {
                14u8.hash(state);
                // Hash mapping by sorted key-value pairs
                let mut pairs: Vec<_> = m.iter().collect();
                pairs.sort_by_key(|&(k, _)| k);
                pairs.hash(state);
            }
            Value::Set(s) => {
                15u8.hash(state);
                // For collections, we will hash all values in sorted order to ensure consistency
                let mut values: Vec<_> = s.iter().collect();
                values.sort();
                values.hash(state);
            }
            Value::Geography(g) => {
                16u8.hash(state);
                g.hash(state);
            }
            Value::Duration(d) => {
                17u8.hash(state);
                d.hash(state);
            }
            Value::DataSet(ds) => {
                18u8.hash(state);
                ds.hash(state);
            }
        }
    }
}

impl Value {
    // Null type comparison helper function
    fn cmp_null(a: &NullType, b: &NullType) -> CmpOrdering {
        // Sequential comparison based on enumerated variants
        match (a, b) {
            (NullType::Null, NullType::Null) => CmpOrdering::Equal,
            (NullType::NaN, NullType::NaN) => CmpOrdering::Equal,
            (NullType::BadData, NullType::BadData) => CmpOrdering::Equal,
            (NullType::BadType, NullType::BadType) => CmpOrdering::Equal,
            _ => {
                // Comparison of different types in order of variants
                let priority_a = Self::null_type_priority(a);
                let priority_b = Self::null_type_priority(b);
                priority_a.cmp(&priority_b)
            }
        }
    }

    // Null type priority mapping function
    fn null_type_priority(typ: &NullType) -> u8 {
        match typ {
            NullType::Null => 0,
            NullType::NaN => 1,
            NullType::BadData => 2,
            NullType::BadType => 2,
            NullType::ErrOverflow => 3,
            NullType::UnknownProp => 4,
            NullType::DivByZero => 5,
            NullType::OutOfRange => 6,
        }
    }

    // Floating Point Comparison Helper Functions
    fn cmp_f64(a: f64, b: f64) -> CmpOrdering {
        // Handling floating-point comparisons, including NaN
        if a.is_nan() && b.is_nan() {
            CmpOrdering::Equal
        } else if a.is_nan() {
            CmpOrdering::Less // NaN Less than any non-NaN value
        } else if b.is_nan() {
            CmpOrdering::Greater // Any non-NaN value greater than NaN
        } else {
            a.partial_cmp(&b).unwrap_or(CmpOrdering::Equal)
        }
    }

    // Date Comparison Helper Functions
    fn cmp_date(a: &DateValue, b: &DateValue) -> CmpOrdering {
        // Comparison date: year -> month -> day
        match a.year.cmp(&b.year) {
            CmpOrdering::Equal => match a.month.cmp(&b.month) {
                CmpOrdering::Equal => a.day.cmp(&b.day),
                ord => ord,
            },
            ord => ord,
        }
    }

    // Time Comparison Auxiliary Functions
    fn cmp_time(a: &TimeValue, b: &TimeValue) -> CmpOrdering {
        // Compare time: hours -> minutes -> seconds -> microseconds
        match a.hour.cmp(&b.hour) {
            CmpOrdering::Equal => match a.minute.cmp(&b.minute) {
                CmpOrdering::Equal => match a.sec.cmp(&b.sec) {
                    CmpOrdering::Equal => a.microsec.cmp(&b.microsec),
                    ord => ord,
                },
                ord => ord,
            },
            ord => ord,
        }
    }

    // Date-Time Comparison Helper Functions
    fn cmp_datetime(a: &DateTimeValue, b: &DateTimeValue) -> CmpOrdering {
        // Compare date and time: year -> month -> day -> hour -> minute -> second -> microseconds
        match a.year.cmp(&b.year) {
            CmpOrdering::Equal => match a.month.cmp(&b.month) {
                CmpOrdering::Equal => match a.day.cmp(&b.day) {
                    CmpOrdering::Equal => match a.hour.cmp(&b.hour) {
                        CmpOrdering::Equal => match a.minute.cmp(&b.minute) {
                            CmpOrdering::Equal => match a.sec.cmp(&b.sec) {
                                CmpOrdering::Equal => a.microsec.cmp(&b.microsec),
                                ord => ord,
                            },
                            ord => ord,
                        },
                        ord => ord,
                    },
                    ord => ord,
                },
                ord => ord,
            },
            ord => ord,
        }
    }

    // Duration Comparison Auxiliary Functions
    fn cmp_duration(a: &DurationValue, b: &DurationValue) -> CmpOrdering {
        // Comparison duration: seconds -> microseconds -> months
        match a.seconds.cmp(&b.seconds) {
            CmpOrdering::Equal => match a.microseconds.cmp(&b.microseconds) {
                CmpOrdering::Equal => a.months.cmp(&b.months),
                ord => ord,
            },
            ord => ord,
        }
    }

    // List Comparison Helper Functions
    fn cmp_list(a: &List, b: &List) -> CmpOrdering {
        // Compare lists by dictionary order
        let min_len = a.values.len().min(b.values.len());
        for i in 0..min_len {
            match a.values[i].cmp(&b.values[i]) {
                CmpOrdering::Equal => continue,
                ord => return ord,
            }
        }
        a.values.len().cmp(&b.values.len())
    }

    // Mapping Comparison Auxiliary Functions
    fn cmp_map(a: &HashMap<String, Value>, b: &HashMap<String, Value>) -> CmpOrdering {
        // Compare the number of keys first
        match a.len().cmp(&b.len()) {
            CmpOrdering::Equal => {
                // Comparison by sorted key-value pairs
                let mut a_sorted: Vec<_> = a.iter().collect();
                let mut b_sorted: Vec<_> = b.iter().collect();
                a_sorted.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
                b_sorted.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

                for ((k1, v1), (k2, v2)) in a_sorted.iter().zip(b_sorted.iter()) {
                    match k1.cmp(k2) {
                        CmpOrdering::Equal => match v1.cmp(v2) {
                            CmpOrdering::Equal => continue,
                            ord => return ord,
                        },
                        ord => return ord,
                    }
                }
                CmpOrdering::Equal
            }
            ord => ord,
        }
    }

    // Set Comparison Auxiliary Functions
    fn cmp_set(
        a: &std::collections::HashSet<Value>,
        b: &std::collections::HashSet<Value>,
    ) -> CmpOrdering {
        // Compare set sizes first
        match a.len().cmp(&b.len()) {
            CmpOrdering::Equal => {
                // Comparison by sorted elements
                let mut a_sorted: Vec<_> = a.iter().collect();
                let mut b_sorted: Vec<_> = b.iter().collect();
                a_sorted.sort();
                b_sorted.sort();

                for (v1, v2) in a_sorted.iter().zip(b_sorted.iter()) {
                    match v1.cmp(v2) {
                        CmpOrdering::Equal => continue,
                        ord => return ord,
                    }
                }
                CmpOrdering::Equal
            }
            ord => ord,
        }
    }

    // Geographic Information Comparison Auxiliary Function - Version
    fn cmp_geography(a: &GeographyValue, b: &GeographyValue) -> CmpOrdering {
        // Comparing Latitude and Longitude
        match a.latitude.total_cmp(&b.latitude) {
            CmpOrdering::Equal => a.longitude.total_cmp(&b.longitude),
            ord => ord,
        }
    }

    // Data Set Comparison Auxiliary Functions
    fn cmp_dataset(a: &DataSet, b: &DataSet) -> CmpOrdering {
        // Compare columns first
        match Self::cmp_string_list(&a.col_names, &b.col_names) {
            CmpOrdering::Equal => {
                // Compare Row Data
                let min_len = a.rows.len().min(b.rows.len());
                for i in 0..min_len {
                    match Self::cmp_value_list(&a.rows[i], &b.rows[i]) {
                        CmpOrdering::Equal => continue,
                        ord => return ord,
                    }
                }
                a.rows.len().cmp(&b.rows.len())
            }
            ord => ord,
        }
    }

    // Type Priority Comparison Helper Functions
    fn cmp_by_type_priority(a: &Value, b: &Value) -> CmpOrdering {
        // Type priority: Empty < Null < Bool < Int < Float < String < Date < Time < DateTime < Duration <
        //             Vertex < Edge < Path < List < Map < Set < Geography < DataSet
        let type_a = a.get_type();
        let type_b = b.get_type();

        if type_a == type_b {
            // The same type should already be handled in the main function, here as a fallback
            return CmpOrdering::Equal;
        }

        // Using Enumerated Value Comparisons Instead of String Comparisons
        Self::type_priority(&type_a).cmp(&Self::type_priority(&type_b))
    }

    // Type Priority Mapping Functions
    fn type_priority(typ: &DataType) -> u8 {
        match typ {
            DataType::Empty => 0,
            DataType::Null => 1,
            DataType::Bool => 2,
            DataType::Int
            | DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64 => 3,
            DataType::Float | DataType::Double => 4,
            DataType::Decimal128 => 5,
            DataType::String => 6,
            DataType::FixedString(_) => 6,
            DataType::Date => 7,
            DataType::Time => 8,
            DataType::Timestamp => 8,
            DataType::DateTime => 9,
            DataType::VID => 10,
            DataType::Duration => 11,
            DataType::Vertex => 12,
            DataType::Edge => 13,
            DataType::Path => 14,
            DataType::List => 15,
            DataType::Map => 16,
            DataType::Set => 17,
            DataType::Blob => 18,
            DataType::Geography => 19,
            DataType::DataSet => 20,
        }
    }

    // String list comparison helper function
    fn cmp_string_list(a: &[String], b: &[String]) -> CmpOrdering {
        let min_len = a.len().min(b.len());
        for i in 0..min_len {
            match a[i].cmp(&b[i]) {
                CmpOrdering::Equal => continue,
                ord => return ord,
            }
        }
        a.len().cmp(&b.len())
    }

    // Value List Comparison Helper Functions
    fn cmp_value_list(a: &[Value], b: &[Value]) -> CmpOrdering {
        let min_len = a.len().min(b.len());
        for i in 0..min_len {
            match a[i].cmp(&b[i]) {
                CmpOrdering::Equal => continue,
                ord => return ord,
            }
        }
        a.len().cmp(&b.len())
    }
}
