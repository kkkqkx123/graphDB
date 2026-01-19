use chrono::{DateTime, Datelike, Local, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Represents a timestamp in the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp {
    pub secs: i64,
    pub nsecs: u32,
}

impl Timestamp {
    /// Creates a new timestamp from seconds and nanoseconds
    pub fn new(secs: i64, nsecs: u32) -> Self {
        Self { secs, nsecs }
    }

    /// Creates a timestamp for the current moment
    pub fn now() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        Self {
            secs: now.as_secs() as i64,
            nsecs: now.subsec_nanos(),
        }
    }

    /// Converts the timestamp to a DateTime<Utc>
    pub fn to_datetime(&self) -> DateTime<Utc> {
        let dt = DateTime::from_timestamp(self.secs, self.nsecs).expect("Invalid timestamp");
        dt
    }

    /// Creates a timestamp from a DateTime<Utc>
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self {
            secs: dt.timestamp(),
            nsecs: dt.timestamp_subsec_nanos(),
        }
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

/// Represents a date in the database (year, month, day)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    /// Creates a new date
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self, String> {
        if month < 1 || month > 12 {
            return Err("Month must be between 1 and 12".to_string());
        }

        let max_day = Self::days_in_month(year, month);
        if day < 1 || day > max_day {
            return Err(format!("Day must be between 1 and {} for month {}", max_day, month));
        }

        Ok(Self { year, month, day })
    }

    /// Get the number of days in a month
    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 31,
        }
    }

    /// Check if a year is a leap year
    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Creates a date for today
    pub fn today() -> Self {
        let now: DateTime<Local> = Local::now();
        Self {
            year: now.year(),
            month: now.month(),
            day: now.day(),
        }
    }

    /// Converts the date to a NaiveDate
    pub fn to_naive_date(&self) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(self.year, self.month, self.day).expect("Invalid date")
    }

    /// Creates a date from a NaiveDate
    pub fn from_naive_date(date: chrono::NaiveDate) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        }
    }

    /// 添加天数到日期
    pub fn add_days(&self, days: i64) -> Date {
        let naive_date = self.to_naive_date();
        let new_date = naive_date + chrono::Duration::days(days);
        Self::from_naive_date(new_date)
    }

    /// 从日期减去天数
    pub fn sub_days(&self, days: i64) -> Date {
        let naive_date = self.to_naive_date();
        let new_date = naive_date - chrono::Duration::days(days);
        Self::from_naive_date(new_date)
    }

    /// 添加月份到日期
    pub fn add_months(&self, months: i32) -> Date {
        let naive_date = self.to_naive_date();
        let new_date = naive_date.with_month(self.month)
            .and_then(|d| d.with_year(self.year))
            .unwrap_or(naive_date);
        let new_date = new_date + chrono::Duration::days((months as i64) * 30);
        Self::from_naive_date(new_date)
    }

    /// 从日期减去月份
    pub fn sub_months(&self, months: i32) -> Date {
        self.add_months(-months)
    }

    /// 添加年份到日期
    pub fn add_years(&self, years: i32) -> Date {
        let naive_date = self.to_naive_date();
        let new_year = self.year + years;
        let new_date = naive_date.with_year(new_year).unwrap_or(naive_date);
        Self::from_naive_date(new_date)
    }

    /// 从日期减去年份
    pub fn sub_years(&self, years: i32) -> Date {
        self.add_years(-years)
    }

    /// 添加持续时间到日期
    pub fn add_duration(&self, duration: &DurationValue) -> Date {
        let mut naive_date = self.to_naive_date();

        if duration.months != 0 {
            naive_date = naive_date + chrono::Duration::days((duration.months as i64) * 30);
        }

        if duration.days != 0 {
            naive_date = naive_date + chrono::Duration::days(duration.days as i64);
        }

        if duration.nsecs != 0 {
            let seconds = duration.nsecs / 1_000_000_000;
            naive_date = naive_date + chrono::Duration::seconds(seconds);
        }

        Self::from_naive_date(naive_date)
    }

    /// 从日期减去持续时间
    pub fn sub_duration(&self, duration: &DurationValue) -> Date {
        let neg_duration = DurationValue {
            months: -duration.months,
            days: -duration.days,
            nsecs: -duration.nsecs,
        };
        self.add_duration(&neg_duration)
    }

    /// 计算两个日期之间的天数差
    pub fn days_between(&self, other: &Date) -> i64 {
        let self_date = self.to_naive_date();
        let other_date = other.to_naive_date();
        (self_date - other_date).num_days()
    }

    /// 计算两个日期之间的月数差（近似值）
    pub fn months_between(&self, other: &Date) -> i32 {
        let year_diff = (self.year - other.year) * 12;
        let month_diff = self.month as i32 - other.month as i32;
        year_diff + month_diff
    }

    /// 计算两个日期之间的年数差
    pub fn years_between(&self, other: &Date) -> i32 {
        self.year - other.year
    }

    /// 将日期转换为自纪元以来的天数（用于数据库存储）
    pub fn to_int(&self) -> i64 {
        let naive_date = self.to_naive_date();
        naive_date.signed_duration_since(chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()).num_days()
    }

    /// 从自纪元以来的天数创建日期（用于数据库存储）
    pub fn from_int(days: i64) -> Date {
        let naive_date = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
            + chrono::Duration::days(days);
        Self::from_naive_date(naive_date)
    }

    /// 将日期序列化为字节数组（用于数据库存储）
    pub fn to_bytes(&self) -> [u8; 3] {
        [
            ((self.year >> 8) & 0xFF) as u8,
            (self.year & 0xFF) as u8,
            self.month as u8,
        ]
    }

    /// 从字节数组反序列化日期（用于数据库存储）
    pub fn from_bytes(bytes: &[u8]) -> Result<Date, String> {
        if bytes.len() < 3 {
            return Err("字节数组长度不足".to_string());
        }

        let year = ((bytes[0] as i32) << 8) | (bytes[1] as i32);
        let month = bytes[2] as u32;

        Self::new(year, month, 1)
    }

    /// 格式化日期为字符串
    pub fn format(&self, format: &str) -> String {
        let naive_date = self.to_naive_date();
        naive_date.format(format).to_string()
    }

    /// 解析日期字符串
    pub fn parse(s: &str, format: &str) -> Result<Date, String> {
        let naive_date = chrono::NaiveDate::parse_from_str(s, format)
            .map_err(|e| format!("解析日期失败: {}", e))?;
        Ok(Self::from_naive_date(naive_date))
    }
}

impl Default for Date {
    fn default() -> Self {
        Self::today()
    }
}

/// Represents a time of day (hour, minute, second, microsecond)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Time {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub microsecond: u32,
}

impl Time {
    /// Creates a new time
    pub fn new(hour: u32, minute: u32, second: u32, microsecond: u32) -> Result<Self, String> {
        if hour > 23 {
            return Err("Hour must be between 0 and 23".to_string());
        }
        if minute > 59 {
            return Err("Minute must be between 0 and 59".to_string());
        }
        if second > 59 {
            return Err("Second must be between 0 and 59".to_string());
        }
        if microsecond > 999_999 {
            return Err("Microsecond must be between 0 and 999,999".to_string());
        }

        Ok(Self {
            hour,
            minute,
            second,
            microsecond,
        })
    }

    /// Creates a time for now
    pub fn now() -> Self {
        let now: DateTime<Local> = Local::now();
        Self {
            hour: now.hour(),
            minute: now.minute(),
            second: now.second(),
            microsecond: now.timestamp_subsec_micros(),
        }
    }

    /// Converts to a NaiveTime
    pub fn to_naive_time(&self) -> chrono::NaiveTime {
        chrono::NaiveTime::from_hms_micro_opt(self.hour, self.minute, self.second, self.microsecond)
            .expect("Invalid time")
    }

    /// Creates a time from a NaiveTime
    pub fn from_naive_time(time: chrono::NaiveTime) -> Self {
        Self {
            hour: time.hour(),
            minute: time.minute(),
            second: time.second(),
            microsecond: time.nanosecond() / 1000, // Convert nanos to micros
        }
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::now()
    }
}

/// Represents a duration or time interval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationValue {
    pub months: i32,
    pub days: i32,
    pub nsecs: i64,
}

impl DurationValue {
    /// Creates a new duration
    pub fn new(months: i32, days: i32, nsecs: i64) -> Self {
        Self {
            months,
            days,
            nsecs,
        }
    }

    /// Creates a duration from std::time::Duration
    pub fn from_std(duration: Duration, negative: bool) -> Self {
        let mut nsecs = duration.as_nanos() as i64;
        if negative {
            nsecs = -nsecs;
        }

        Self {
            months: 0, // std::time::Duration doesn't support months
            days: 0,   // std::time::Duration doesn't support days in the same way
            nsecs,
        }
    }
}

/// Utility functions for time operations
pub mod time_utils {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Get the current timestamp as milliseconds since epoch
    pub fn now_ms() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as i64
    }

    /// Get the current timestamp as seconds since epoch
    pub fn now_sec() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64
    }

    /// Format timestamp as ISO 8601 string
    pub fn format_timestamp_iso(timestamp: &Timestamp) -> String {
        let datetime = timestamp.to_datetime();
        datetime.to_rfc3339()
    }

    /// Parse an ISO 8601 string to a timestamp
    pub fn parse_timestamp_iso(s: &str) -> Result<Timestamp, String> {
        let dt = DateTime::parse_from_rfc3339(s)
            .map_err(|e| e.to_string())?
            .with_timezone(&Utc);
        Ok(Timestamp::from_datetime(dt))
    }

    /// Calculate the difference between two timestamps
    pub fn timestamp_diff(start: &Timestamp, end: &Timestamp) -> Duration {
        let start_time = UNIX_EPOCH + Duration::new(start.secs as u64, start.nsecs);
        let end_time = UNIX_EPOCH + Duration::new(end.secs as u64, end.nsecs);

        match end_time.duration_since(start_time) {
            Ok(duration) => duration,
            Err(_) => Duration::new(0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp() {
        let ts = Timestamp::now();
        assert!(ts.secs > 0);

        let ts2 = Timestamp::new(1000, 500_000_000);
        assert_eq!(ts2.secs, 1000);
        assert_eq!(ts2.nsecs, 500_000_000);
    }

    #[test]
    fn test_date() {
        let date = Date::today();
        assert!(date.year > 0);

        let specific_date = Date::new(2023, 12, 25).expect("Failed to create date");
        assert_eq!(specific_date.year, 2023);
        assert_eq!(specific_date.month, 12);
        assert_eq!(specific_date.day, 25);
    }

    #[test]
    fn test_time() {
        let time = Time::now();
        assert!(time.hour <= 23);

        let specific_time = Time::new(14, 30, 45, 123456).expect("Failed to create time");
        assert_eq!(specific_time.hour, 14);
        assert_eq!(specific_time.minute, 30);
        assert_eq!(specific_time.second, 45);
        assert_eq!(specific_time.microsecond, 123456);
    }

    #[test]
    fn test_duration() {
        let duration = DurationValue::new(2, 15, 5_000_000_000); // 2 months, 15 days, 5 seconds
        assert_eq!(duration.months, 2);
        assert_eq!(duration.days, 15);
        assert_eq!(duration.nsecs, 5_000_000_000);
    }

    #[test]
    fn test_time_utils() {
        let now_ms = time_utils::now_ms();
        assert!(now_ms > 0);

        let now_sec = time_utils::now_sec();
        assert!(now_sec > 0);

        // Test timestamp formatting and parsing
        let ts = Timestamp::new(1672531200, 0); // 2023-01-01T00:00:00Z
        let iso_str = time_utils::format_timestamp_iso(&ts);
        let parsed_ts =
            time_utils::parse_timestamp_iso(&iso_str).expect("Failed to parse timestamp");
        assert_eq!(ts.secs, parsed_ts.secs);
    }
}
