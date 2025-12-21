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
        if day < 1 || day > 31 {
            return Err("Day must be between 1 and 31".to_string());
        }

        Ok(Self { year, month, day })
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
