# Nebula-Graph 对照分析

## 概述

本文档对照 nebula-graph 的实现，分析当前简化实现与 nebula-graph 的差异，并提供改进建议。

## 1. LIKE 操作对照分析

### 当前实现

**位置**: `src\expression\evaluator\expression_evaluator.rs:712`

```rust
pub fn eval_like(
    _value: &Value,
    _pattern: &Value,
    _escape_char: Option<char>,
) -> Result<Value, ExpressionError> {
    todo!("LIKE操作实现")
}
```

**状态**: 未实现

### Nebula-Graph 实现

**位置**: `nebula-3.8.0/src/common/expression/RelationalExpression.cpp`

Nebula-Graph 没有独立的 LIKE 操作，而是使用正则表达式操作 (`kRelREG`)：

```cpp
case Kind::kRelREG: {
    if (lhs.isBadNull() || rhs.isBadNull()) {
        result_ = Value::kNullBadType;
    } else if ((!lhs.isNull() && !lhs.empty() && !lhs.isStr()) ||
               (!rhs.isNull() && !rhs.empty() && !rhs.isStr())) {
        result_ = Value::kNullBadType;
    } else if (lhs.isStr() && rhs.isStr()) {
        try {
            const auto& r = ctx.getRegex(rhs.getStr());
            result_ = std::regex_match(lhs.getStr(), r);
        } catch (const std::exception& ex) {
            LOG(ERROR) << "Regex match error: " << ex.what();
            result_ = Value::kNullBadType;
        }
    } else {
        result_ = Value::kNullValue;
    }
    break;
}
```

### 差异分析

| 方面 | 当前实现 | Nebula-Graph |
|------|---------|--------------|
| 操作类型 | LIKE（未实现） | REG（正则表达式） |
| 功能 | 支持通配符 % 和 _ | 支持完整正则表达式 |
| 错误处理 | 无 | 完善的错误处理 |
| NULL 处理 | 无 | 支持 NULL 值 |

### 改进建议

1. **实现 LIKE 操作**：
   - 支持 SQL LIKE 语法（`%` 和 `_` 通配符）
   - 支持自定义转义字符
   - 参考 SQL 标准实现

2. **可选：添加 REGEXP 操作**：
   - 如果需要更强大的模式匹配
   - 可以使用 Rust 的 `regex` crate

3. **错误处理**：
   - 处理非字符串输入
   - 处理无效的模式
   - 处理 NULL 值

## 2. Date 转换对照分析

### 当前实现

**位置**: `src\expression\storage\row_reader.rs:423`

```rust
// 将天数转换为DateValue（简化实现，从1970-01-01开始计算）
Ok(Value::Date(crate::core::value::DateValue {
    year: 1970 + (days / 365) as i32,
    month: ((days % 365) / 30 + 1) as u32,
    day: ((days % 365) % 30 + 1) as u32,
}))
```

**问题**:
- 固定 365 天/年，不考虑闰年
- 固定 30 天/月，不考虑实际月份天数
- 从 1970-01-01 开始计算

### Nebula-Graph 实现

**位置**: `nebula-3.8.0/src/common/datatypes/Date.h` 和 `Date.cpp`

```cpp
struct Date {
    int16_t year;  // Any integer
    int8_t month;  // 1 - 12
    int8_t day;    // 1 - 31

    Date() : year{0}, month{1}, day{1} {}
    Date(int16_t y, int8_t m, int8_t d) : year{y}, month{m}, day{d} {}
    explicit Date(uint64_t days);

    // Return the number of days since -32768/1/1
    int64_t toInt() const;
    // Convert the number of days since -32768/1/1 to the real date
    void fromInt(int64_t days);
};
```

```cpp
int64_t Date::toInt() const {
    // Year
    int64_t yearsPassed = year + 32768L;
    int64_t days = yearsPassed * 365L;
    // Add one day per leap year
    if (yearsPassed > 0) {
        days += (yearsPassed - 1) / 4 + 1;
    }

    // Month
    if (yearsPassed % 4 == 0) {
        // Leap year
        days += kLeapDaysSoFar[month - 1];
    } else {
        days += kDaysSoFar[month - 1];
    }

    // Day
    days += day;

    // Since we start from -32768/1/1, we need to reduce one day
    return days - 1;
}
```

```cpp
void Date::fromInt(int64_t days) {
    // year
    int64_t yearsPassed = (days + 1) / 365;
    year = yearsPassed - 32768;
    int64_t daysInYear = (days + 1) % 365;

    // Deduce the number of days for leap years
    if (yearsPassed > 0) {
        daysInYear -= (yearsPassed - 1) / 4 + 1;
    }

    // Adjust the year if necessary
    while (daysInYear <= 0) {
        year = year - 1;
        if (year % 4 == 0) {
            // Leap year
            daysInYear += 366;
        } else {
            daysInYear += 365;
        }
    }

    // Month and day
    month = 1;
    while (true) {
        if (year % 4 == 0) {
            // Leap year
            if (daysInYear <= kLeapDaysSoFar[month]) {
                day = daysInYear - kLeapDaysSoFar[month - 1];
                break;
            }
        } else {
            if (daysInYear <= kDaysSoFar[month]) {
                day = daysInYear - kDaysSoFar[month - 1];
                break;
            }
        }
        month++;
    }
}
```

```cpp
const int64_t kDaysSoFar[] = {0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365};
const int64_t kLeapDaysSoFar[] = {0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366};
```

### 差异分析

| 方面 | 当前实现 | Nebula-Graph |
|------|---------|--------------|
| 闰年处理 | 无 | 正确处理闰年 |
| 月份天数 | 固定 30 天 | 实际月份天数 |
| 起始日期 | 1970-01-01 | -32768/1/1 |
| 准确性 | 低 | 高 |
| 日期范围 | 有限 | 广泛（-32768 到 32767） |

### 改进建议

**方案一：使用 chrono crate（推荐）**

```rust
use chrono::{NaiveDate, Datelike};

fn days_to_date(days: i64) -> DateValue {
    let date = NaiveDate::from_ymd_opt(1970, 1, 1)
        .unwrap()
        .checked_add_signed(chrono::Duration::days(days))
        .unwrap();
    
    DateValue {
        year: date.year(),
        month: date.month(),
        day: date.day(),
    }
}
```

**方案二：移植 nebula-graph 逻辑**

```rust
const DAYS_SO_FAR: [i64; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
const LEAP_DAYS_SO_FAR: [i64; 13] = [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366];

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_to_date(days: i64) -> DateValue {
    // year
    let years_passed = (days + 1) / 365;
    let mut year = years_passed - 32768;
    let mut days_in_year = (days + 1) % 365;

    // Deduce the number of days for leap years
    if years_passed > 0 {
        days_in_year -= (years_passed - 1) / 4 + 1;
    }

    // Adjust the year if necessary
    while days_in_year <= 0 {
        year -= 1;
        if is_leap_year(year) {
            days_in_year += 366;
        } else {
            days_in_year += 365;
        }
    }

    // Month and day
    let mut month = 1;
    let day = loop {
        if is_leap_year(year) {
            if days_in_year <= LEAP_DAYS_SO_FAR[month] {
                break days_in_year - LEAP_DAYS_SOFar[month - 1];
            }
        } else {
            if days_in_year <= DAYS_SO_FAR[month] {
                break days_in_year - DAYS_SO_FAR[month - 1];
            }
        }
        month += 1;
    };

    DateValue { year, month, day }
}
```

## 3. DateTime 转换对照分析

### 当前实现

**位置**: `src\expression\storage\row_reader.rs:441`

```rust
// 将时间戳转换为DateTimeValue（简化实现，总是返回1970-01-01）
Ok(Value::DateTime(crate::core::value::DateTimeValue {
    date: crate::core::value::DateValue {
        year: 1970,
        month: 1,
        day: 1,
    },
    time: crate::core::value::TimeValue {
        hour: ((timestamp / 3600) % 24) as u8,
        minute: ((timestamp / 60) % 60) as u8,
        second: (timestamp % 60) as u8,
        microsecond: 0,
    },
}))
```

**问题**:
- 总是返回 1970-01-01
- 只计算时间部分
- 没有使用时间戳计算实际日期

### Nebula-Graph 实现

**位置**: `nebula-3.8.0/src/common/datatypes/Date.h`

```cpp
struct DateTime {
    union {
        struct {
            int64_t year : 16;
            uint64_t month : 4;
            uint64_t day : 5;
            uint64_t hour : 5;
            uint64_t minute : 6;
            uint64_t sec : 6;
            uint64_t microsec : 22;
        };
        uint64_t qword;
    };

    DateTime() : year{0}, month{1}, day{1}, hour{0}, minute{0}, sec{0}, microsec{0} {}
    DateTime(int16_t y, int8_t m, int8_t d, int8_t h, int8_t min, int8_t s, int32_t us) {
        year = y;
        month = m;
        day = d;
        hour = h;
        minute = min;
        sec = s;
        microsec = us;
    }
    explicit DateTime(const Date& date) {
        year = date.year;
        month = date.month;
        day = date.day;
        hour = 0;
        minute = 0;
        sec = 0;
        microsec = 0;
    }
    DateTime(const Date& date, const Time& time) {
        year = date.year;
        month = date.month;
        day = date.day;
        hour = time.hour;
        minute = time.minute;
        sec = time.sec;
        microsec = time.microsec;
    }

    Date date() const {
        return Date(year, month, day);
    }

    Time time() const {
        return Time(hour, minute, sec, microsec);
    }
};
```

### 差异分析

| 方面 | 当前实现 | Nebula-Graph |
|------|---------|--------------|
| 日期计算 | 固定 1970-01-01 | 从时间戳计算 |
| 时间计算 | 正确 | 正确 |
| 微秒支持 | 无 | 支持 |
| 内存布局 | 分离的 Date 和 Time | 使用 union 优化 |

### 改进建议

**方案一：使用 chrono crate（推荐）**

```rust
use chrono::{NaiveDateTime, Timelike};

fn timestamp_to_datetime(timestamp: i64) -> DateTimeValue {
    let datetime = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    
    DateTimeValue {
        date: DateValue {
            year: datetime.year(),
            month: datetime.month(),
            day: datetime.day(),
        },
        time: TimeValue {
            hour: datetime.hour(),
            minute: datetime.minute(),
            second: datetime.second(),
            microsecond: datetime.nanosecond() / 1000,
        },
    }
}
```

**方案二：移植 nebula-graph 逻辑**

结合 Date 转换逻辑和时间计算：

```rust
fn timestamp_to_datetime(timestamp: i64) -> DateTimeValue {
    let days = timestamp / 86400; // 秒转天
    let seconds_in_day = timestamp % 86400;
    
    let date = days_to_date(days);
    
    DateTimeValue {
        date,
        time: TimeValue {
            hour: (seconds_in_day / 3600) as u8,
            minute: ((seconds_in_day % 3600) / 60) as u8,
            second: (seconds_in_day % 60) as u8,
            microsecond: 0,
        },
    }
}
```

## 4. ColumnDef 类型定义对照分析

### 当前实现

**位置**: `src\expression\storage\types.rs:15`

```rust
/// 列定义（简化版本，保持向后兼容）
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}
```

### Nebula-Graph 实现

**位置**: `nebula-3.8.0/src/common/datatypes/Value.h`

```cpp
enum class Type : uint64_t {
    __EMPTY__ = 1UL,
    BOOL = 1UL << 1,
    INT = 1UL << 2,
    FLOAT = 1UL << 3,
    STRING = 1UL << 4,
    DATE = 1UL << 5,
    TIME = 1UL << 6,
    DATETIME = 1UL << 7,
    VERTEX = 1UL << 8,
    EDGE = 1UL << 9,
    PATH = 1UL << 10,
    LIST = 1UL << 11,
    MAP = 1UL << 12,
    SET = 1UL << 13,
    DATASET = 1UL << 14,
    GEOGRAPHY = 1UL << 15,
    DURATION = 1UL << 16,
    NULLVALUE = 1UL << 63,
};
```

### 差异分析

| 方面 | 当前实现 | Nebula-Graph |
|------|---------|--------------|
| 类型表示 | String | 枚举 |
| 类型安全 | 低 | 高 |
| 编译时检查 | 无 | 有 |
| 拼写错误风险 | 高 | 低 |

### 改进建议

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldType {
    Bool,
    Int,
    Float,
    String,
    Date,
    Time,
    DateTime,
    Vertex,
    Edge,
    Path,
    List,
    Map,
    Set,
    DataSet,
    Geography,
    Duration,
    Null,
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::Bool => write!(f, "bool"),
            FieldType::Int => write!(f, "int"),
            FieldType::Float => write!(f, "float"),
            FieldType::String => write!(f, "string"),
            FieldType::Date => write!(f, "date"),
            FieldType::Time => write!(f, "time"),
            FieldType::DateTime => write!(f, "datetime"),
            FieldType::Vertex => write!(f, "vertex"),
            FieldType::Edge => write!(f, "edge"),
            FieldType::Path => write!(f, "path"),
            FieldType::List => write!(f, "list"),
            FieldType::Map => write!(f, "map"),
            FieldType::Set => write!(f, "set"),
            FieldType::DataSet => write!(f, "dataset"),
            FieldType::Geography => write!(f, "geography"),
            FieldType::Duration => write!(f, "duration"),
            FieldType::Null => write!(f, "null"),
        }
    }
}

impl std::str::FromStr for FieldType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bool" => Ok(FieldType::Bool),
            "int" => Ok(FieldType::Int),
            "float" => Ok(FieldType::Float),
            "string" => Ok(FieldType::String),
            "date" => Ok(FieldType::Date),
            "time" => Ok(FieldType::Time),
            "datetime" => Ok(FieldType::DateTime),
            "vertex" => Ok(FieldType::Vertex),
            "edge" => Ok(FieldType::Edge),
            "path" => Ok(FieldType::Path),
            "list" => Ok(FieldType::List),
            "map" => Ok(FieldType::Map),
            "set" => Ok(FieldType::Set),
            "dataset" => Ok(FieldType::DataSet),
            "geography" => Ok(FieldType::Geography),
            "duration" => Ok(FieldType::Duration),
            "null" => Ok(FieldType::Null),
            _ => Err(format!("未知的数据类型: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: FieldType,
    pub nullable: bool,
}
```

## 5. 集合操作对照分析

### 当前实现

**位置**: `src\expression\evaluator\collection_operations.rs`

```rust
pub fn eval_in_operation(
    left: &Value,
    right: &Value,
) -> Result<Value, ExpressionError> {
    match right {
        Value::List(list) => Ok(Value::Bool(list.contains(left))),
        Value::Set(set) => Ok(Value::Bool(set.contains(left))),
        _ => Err(ExpressionError::InvalidOperation("IN 操作需要列表或集合".to_string())),
    }
}
```

### Nebula-Graph 实现

**位置**: `nebula-3.8.0/src/common/expression/RelationalExpression.cpp`

```cpp
case Kind::kRelIn: {
    if (rhs.isNull() && !rhs.isBadNull()) {
        result_ = Value::kNullValue;
    } else if (rhs.isList()) {
        auto& list = rhs.getList();
        result_ = list.contains(lhs);
        if (UNLIKELY(result_.isBool() && !result_.getBool() && list.contains(Value::kNullValue))) {
            result_ = Value::kNullValue;
        }
    } else if (rhs.isSet()) {
        auto& set = rhs.getSet();
        result_ = set.contains(lhs);
        if (UNLIKELY(result_.isBool() && !result_.getBool() && set.contains(Value::kNullValue))) {
            result_ = Value::kNullValue;
        }
    } else if (rhs.isMap()) {
        auto& map = rhs.getMap();
        result_ = map.contains(lhs);
        if (UNLIKELY(result_.isBool() && !result_.getBool() && map.contains(Value::kNullValue))) {
            result_ = Value::kNullValue;
        }
    } else {
        result_ = Value(NullType::BAD_TYPE);
    }

    if (UNLIKELY(!result_.isBadNull() && lhs.isNull())) {
        result_ = Value::kNullValue;
    }
    break;
}
```

### 差异分析

| 方面 | 当前实现 | Nebula-Graph |
|------|---------|--------------|
| NULL 处理 | 无 | 完整的 NULL 语义 |
| Map 支持 | 无 | 支持 |
| 错误类型 | 简单字符串 | 详细的错误类型 |
| 三值逻辑 | 无 | 支持（true/false/null） |

### 改进建议

```rust
pub fn eval_in_operation(
    left: &Value,
    right: &Value,
) -> Result<Value, ExpressionError> {
    // 如果右侧是 NULL（非 BadNull），返回 NULL
    if right.is_null() && !right.is_bad_null() {
        return Ok(Value::Null);
    }
    
    // 如果左侧是 NULL（非 BadNull），返回 NULL
    if left.is_null() && !left.is_bad_null() {
        return Ok(Value::Null);
    }
    
    match right {
        Value::List(list) => {
            let contains = list.contains(left);
            // 如果不包含但列表中有 NULL，返回 NULL
            if !contains && list.contains(&Value::Null) {
                Ok(Value::Null)
            } else {
                Ok(Value::Bool(contains))
            }
        }
        Value::Set(set) => {
            let contains = set.contains(left);
            // 如果不包含但集合中有 NULL，返回 NULL
            if !contains && set.contains(&Value::Null) {
                Ok(Value::Null)
            } else {
                Ok(Value::Bool(contains))
            }
        }
        Value::Map(map) => {
            let contains = map.contains_key(left);
            // 如果不包含但映射中有 NULL 键，返回 NULL
            if !contains && map.contains_key(&Value::Null) {
                Ok(Value::Null)
            } else {
                Ok(Value::Bool(contains))
            }
        }
        _ => Err(ExpressionError::InvalidOperation("IN 操作需要列表、集合或映射".to_string())),
    }
}
```

## 总结

通过对照分析，我们发现当前实现与 nebula-graph 存在以下主要差异：

1. **功能完整性** - LIKE 操作未实现
2. **数据准确性** - Date 和 DateTime 转换不准确
3. **类型安全** - 使用 String 而不是枚举
4. **NULL 语义** - 缺少完整的三值逻辑支持
5. **错误处理** - 错误类型系统不完善

建议按照上述改进方案逐步修复这些问题，以提高与 nebula-graph 的一致性和代码质量。
