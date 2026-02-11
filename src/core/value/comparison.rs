use super::types::{
    DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue, Value,
};
use crate::core::DataType;
use std::cmp::Ordering as CmpOrdering;
use std::collections::HashMap;
use std::hash::Hash;

// 手动实现PartialEq以正确处理f64比较
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Empty, Value::Empty) => true,
            (Value::Null(a), Value::Null(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a == b) || (a.is_nan() && b.is_nan()), // 正确处理NaN
            (Value::String(a), Value::String(b)) => a == b,
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
            _ => false,
        }
    }
}

// 手动实现Eq，因为f64没有实现Eq
impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // 使用实际的类型比较而不是哈希值
        match (self, other) {
            // 相同类型的比较
            (Value::Empty, Value::Empty) => CmpOrdering::Equal,
            (Value::Null(a), Value::Null(b)) => Self::cmp_null(a, b),
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => Self::cmp_f64(*a, *b),
            (Value::String(a), Value::String(b)) => a.cmp(b),
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

            // 不同类型之间的比较：基于类型优先级
            (a, b) => Self::cmp_by_type_priority(a, b),
        }
    }
}

// 手动实现Hash以处理f64哈希
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
            Value::Float(f) => {
                4u8.hash(state);
                // 从浮点数的位表示创建哈希
                if f.is_nan() {
                    // 所有NaN值应该哈希到相同的值
                    (0x7ff80000u32 as u64).hash(state);
                } else if *f == 0.0 {
                    // 确保+0.0和-0.0哈希到相同的值
                    0.0_f64.to_bits().hash(state);
                } else {
                    f.to_bits().hash(state);
                }
            }
            Value::String(s) => {
                5u8.hash(state);
                s.hash(state);
            }
            Value::Date(d) => {
                6u8.hash(state);
                d.hash(state);
            }
            Value::Time(t) => {
                7u8.hash(state);
                t.hash(state);
            }
            Value::DateTime(dt) => {
                8u8.hash(state);
                dt.hash(state);
            }
            Value::Vertex(v) => {
                9u8.hash(state);
                v.hash(state);
            }
            Value::Edge(e) => {
                10u8.hash(state);
                e.hash(state);
            }
            Value::Path(p) => {
                11u8.hash(state);
                p.hash(state);
            }
            Value::List(l) => {
                12u8.hash(state);
                l.hash(state);
            }
            Value::Map(m) => {
                13u8.hash(state);
                // 通过排序后的键值对哈希映射
                let mut pairs: Vec<_> = m.iter().collect();
                pairs.sort_by_key(|&(k, _)| k);
                pairs.hash(state);
            }
            Value::Set(s) => {
                14u8.hash(state);
                // 对于集合，我们将按排序顺序哈希所有值以确保一致性
                let mut values: Vec<_> = s.iter().collect();
                values.sort();
                values.hash(state);
            }
            Value::Geography(g) => {
                15u8.hash(state);
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
    // Null类型比较辅助函数
    fn cmp_null(a: &NullType, b: &NullType) -> CmpOrdering {
        // 基于枚举变体的顺序比较
        match (a, b) {
            (NullType::Null, NullType::Null) => CmpOrdering::Equal,
            (NullType::NaN, NullType::NaN) => CmpOrdering::Equal,
            (NullType::BadData, NullType::BadData) => CmpOrdering::Equal,
            (NullType::BadType, NullType::BadType) => CmpOrdering::Equal,
            _ => {
                // 不同类型按变体顺序比较
                let priority_a = Self::null_type_priority(a);
                let priority_b = Self::null_type_priority(b);
                priority_a.cmp(&priority_b)
            }
        }
    }

    // Null类型优先级映射函数
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

    // 浮点数比较辅助函数
    fn cmp_f64(a: f64, b: f64) -> CmpOrdering {
        // 处理浮点数比较，包括NaN
        if a.is_nan() && b.is_nan() {
            CmpOrdering::Equal
        } else if a.is_nan() {
            CmpOrdering::Less // NaN 小于任何非NaN值
        } else if b.is_nan() {
            CmpOrdering::Greater // 任何非NaN值大于NaN
        } else {
            a.partial_cmp(&b).unwrap_or(CmpOrdering::Equal)
        }
    }

    // 日期比较辅助函数
    fn cmp_date(a: &DateValue, b: &DateValue) -> CmpOrdering {
        // 比较日期：年 -> 月 -> 日
        match a.year.cmp(&b.year) {
            CmpOrdering::Equal => match a.month.cmp(&b.month) {
                CmpOrdering::Equal => a.day.cmp(&b.day),
                ord => ord,
            },
            ord => ord,
        }
    }

    // 时间比较辅助函数
    fn cmp_time(a: &TimeValue, b: &TimeValue) -> CmpOrdering {
        // 比较时间：时 -> 分 -> 秒 -> 微秒
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

    // 日期时间比较辅助函数
    fn cmp_datetime(a: &DateTimeValue, b: &DateTimeValue) -> CmpOrdering {
        // 比较日期时间：年 -> 月 -> 日 -> 时 -> 分 -> 秒 -> 微秒
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

    // 持续时间比较辅助函数
    fn cmp_duration(a: &DurationValue, b: &DurationValue) -> CmpOrdering {
        // 比较持续时间：秒 -> 微秒 -> 月
        match a.seconds.cmp(&b.seconds) {
            CmpOrdering::Equal => match a.microseconds.cmp(&b.microseconds) {
                CmpOrdering::Equal => a.months.cmp(&b.months),
                ord => ord,
            },
            ord => ord,
        }
    }

    // 列表比较辅助函数
    fn cmp_list(a: &Vec<Value>, b: &Vec<Value>) -> CmpOrdering {
        // 按字典序比较列表
        let min_len = a.len().min(b.len());
        for i in 0..min_len {
            match a[i].cmp(&b[i]) {
                CmpOrdering::Equal => continue,
                ord => return ord,
            }
        }
        a.len().cmp(&b.len())
    }

    // 映射比较辅助函数
    fn cmp_map(a: &HashMap<String, Value>, b: &HashMap<String, Value>) -> CmpOrdering {
        // 先比较键的数量
        match a.len().cmp(&b.len()) {
            CmpOrdering::Equal => {
                // 按排序后的键值对比较
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

    // 集合比较辅助函数
    fn cmp_set(
        a: &std::collections::HashSet<Value>,
        b: &std::collections::HashSet<Value>,
    ) -> CmpOrdering {
        // 先比较集合大小
        match a.len().cmp(&b.len()) {
            CmpOrdering::Equal => {
                // 按排序后的元素比较
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

    // 地理信息比较辅助函数 - 版本
    fn cmp_geography(a: &GeographyValue, b: &GeographyValue) -> CmpOrdering {
        // 比较纬度和经度
        match a.latitude.total_cmp(&b.latitude) {
            CmpOrdering::Equal => a.longitude.total_cmp(&b.longitude),
            ord => ord,
        }
    }

    // 数据集比较辅助函数
    fn cmp_dataset(a: &DataSet, b: &DataSet) -> CmpOrdering {
        // 先比较列名
        match Self::cmp_string_list(&a.col_names, &b.col_names) {
            CmpOrdering::Equal => {
                // 比较行数据
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

    // 类型优先级比较辅助函数
    fn cmp_by_type_priority(a: &Value, b: &Value) -> CmpOrdering {
        // 类型优先级：Empty < Null < Bool < Int < Float < String < Date < Time < DateTime < Duration <
        //             Vertex < Edge < Path < List < Map < Set < Geography < DataSet
        let type_a = a.get_type();
        let type_b = b.get_type();

        if type_a == type_b {
            // 相同类型应该已经在主函数中处理，这里作为fallback
            return CmpOrdering::Equal;
        }

        // 使用枚举值比较替代字符串比较
        Self::type_priority(&type_a).cmp(&Self::type_priority(&type_b))
    }

    // 类型优先级映射函数
    fn type_priority(typ: &DataType) -> u8 {
        match typ {
            DataType::Empty => 0,
            DataType::Null => 1,
            DataType::Bool => 2,
            DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => 3,
            DataType::Float | DataType::Double => 4,
            DataType::String => 5,
            DataType::FixedString(_) => 5,
            DataType::Date => 6,
            DataType::Time => 7,
            DataType::Timestamp => 7,
            DataType::DateTime => 8,
            DataType::VID => 9,
            DataType::Duration => 10,
            DataType::Vertex => 11,
            DataType::Edge => 12,
            DataType::Path => 13,
            DataType::List => 14,
            DataType::Map => 15,
            DataType::Set => 16,
            DataType::Blob => 17,
            DataType::Geography => 18,
            DataType::DataSet => 19,
        }
    }

    // 字符串列表比较辅助函数
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

    // 值列表比较辅助函数
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
