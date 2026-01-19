use graphdb::common::time::*;
use chrono::{Datelike, Timelike};
use std::time::Duration;

#[test]
fn test_timestamp_new() {
    let ts = Timestamp::new(1000, 500_000_000);
    assert_eq!(ts.secs, 1000);
    assert_eq!(ts.nsecs, 500_000_000);
}

#[test]
fn test_timestamp_now() {
    let ts = Timestamp::now();
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;
    assert!((ts.secs - now_secs).abs() < 2);
}

#[test]
fn test_timestamp_to_datetime() {
    let ts = Timestamp::new(1672531200, 0);
    let dt = ts.to_datetime();
    assert_eq!(dt.timestamp(), 1672531200);
}

#[test]
fn test_timestamp_from_datetime() {
    use chrono::{TimeZone, Utc};
    let dt = Utc.timestamp_opt(1672531200, 0).unwrap();
    let ts = Timestamp::from_datetime(dt);
    assert_eq!(ts.secs, 1672531200);
}

#[test]
fn test_timestamp_default() {
    let ts = Timestamp::default();
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;
    assert!((ts.secs - now_secs).abs() < 2);
}

#[test]
fn test_timestamp_serialization() {
    let ts = Timestamp::new(1672531200, 500_000_000);
    let json = serde_json::to_string(&ts).expect("Failed to serialize");
    let deserialized: Timestamp = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(ts.secs, deserialized.secs);
    assert_eq!(ts.nsecs, deserialized.nsecs);
}

#[test]
fn test_date_new_valid() {
    let date = Date::new(2023, 12, 25).expect("Failed to create date");
    assert_eq!(date.year, 2023);
    assert_eq!(date.month, 12);
    assert_eq!(date.day, 25);
}

#[test]
fn test_date_new_invalid_month() {
    let result = Date::new(2023, 13, 1);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Month must be between 1 and 12");
}

#[test]
fn test_date_new_invalid_day() {
    let result = Date::new(2023, 2, 30);
    assert!(result.is_err());
}

#[test]
fn test_date_new_feb_29_leap_year() {
    let date = Date::new(2024, 2, 29).expect("Should succeed for leap year");
    assert_eq!(date.month, 2);
    assert_eq!(date.day, 29);
}

#[test]
fn test_date_new_feb_29_non_leap_year() {
    let result = Date::new(2023, 2, 29);
    assert!(result.is_err());
}

#[test]
fn test_date_today() {
    let date = Date::today();
    let now = chrono::Local::now();
    assert_eq!(date.year, now.year());
    assert_eq!(date.month, now.month());
    assert_eq!(date.day, now.day());
}

#[test]
fn test_date_to_naive_date() {
    let date = Date::new(2023, 6, 15).expect("Failed to create date");
    let naive_date = date.to_naive_date();
    assert_eq!(naive_date.year(), 2023);
    assert_eq!(naive_date.month(), 6);
    assert_eq!(naive_date.day(), 15);
}

#[test]
fn test_date_from_naive_date() {
    let naive_date = chrono::NaiveDate::from_ymd_opt(2023, 12, 1).unwrap();
    let date = Date::from_naive_date(naive_date);
    assert_eq!(date.year, 2023);
    assert_eq!(date.month, 12);
    assert_eq!(date.day, 1);
}

#[test]
fn test_date_add_days() {
    let date = Date::new(2023, 1, 1).expect("Failed to create date");
    let new_date = date.add_days(5);
    assert_eq!(new_date.day, 6);
}

#[test]
fn test_date_add_days_month_wrap() {
    let date = Date::new(2023, 1, 30).expect("Failed to create date");
    let new_date = date.add_days(5);
    assert_eq!(new_date.month, 2);
    assert_eq!(new_date.day, 4);
}

#[test]
fn test_date_sub_days() {
    let date = Date::new(2023, 1, 10).expect("Failed to create date");
    let new_date = date.sub_days(5);
    assert_eq!(new_date.day, 5);
}

#[test]
fn test_date_sub_days_month_wrap() {
    let date = Date::new(2023, 2, 5).expect("Failed to create date");
    let new_date = date.sub_days(10);
    assert_eq!(new_date.month, 1);
    assert_eq!(new_date.day, 26);
}

#[test]
fn test_date_add_months() {
    let date = Date::new(2023, 1, 15).expect("Failed to create date");
    let new_date = date.add_months(2);
    assert_eq!(new_date.month, 3);
}

#[test]
fn test_date_add_months_wrap() {
    let date = Date::new(2023, 11, 15).expect("Failed to create date");
    let new_date = date.add_months(3);
    assert_eq!(new_date.month, 2);
    assert_eq!(new_date.year, 2024);
}

#[test]
fn test_date_sub_months() {
    let date = Date::new(2023, 3, 15).expect("Failed to create date");
    let new_date = date.sub_months(2);
    assert_eq!(new_date.month, 1);
}

#[test]
fn test_date_add_years() {
    let date = Date::new(2023, 6, 15).expect("Failed to create date");
    let new_date = date.add_years(5);
    assert_eq!(new_date.year, 2028);
}

#[test]
fn test_date_sub_years() {
    let date = Date::new(2028, 6, 15).expect("Failed to create date");
    let new_date = date.sub_years(5);
    assert_eq!(new_date.year, 2023);
}

#[test]
fn test_date_days_between() {
    let date1 = Date::new(2023, 1, 1).expect("Failed to create date");
    let date2 = Date::new(2023, 1, 10).expect("Failed to create date");
    let days = date2.days_between(&date1);
    assert_eq!(days, 9);
}

#[test]
fn test_date_months_between() {
    let date1 = Date::new(2023, 1, 1).expect("Failed to create date");
    let date2 = Date::new(2023, 6, 1).expect("Failed to create date");
    let months = date2.months_between(&date1);
    assert_eq!(months, 5);
}

#[test]
fn test_date_years_between() {
    let date1 = Date::new(2020, 1, 1).expect("Failed to create date");
    let date2 = Date::new(2023, 1, 1).expect("Failed to create date");
    let years = date2.years_between(&date1);
    assert_eq!(years, 3);
}

#[test]
fn test_date_to_int() {
    let date = Date::new(1970, 1, 1).expect("Failed to create date");
    assert_eq!(date.to_int(), 0);
}

#[test]
fn test_date_from_int() {
    let date = Date::from_int(365);
    assert_eq!(date.year, 1971);
    assert_eq!(date.month, 1);
    assert_eq!(date.day, 1);
}

#[test]
fn test_date_format() {
    let date = Date::new(2023, 12, 25).expect("Failed to create date");
    let formatted = date.format("%Y-%m-%d");
    assert_eq!(formatted, "2023-12-25");
}

#[test]
fn test_date_parse() {
    let date_str = "2023-12-25";
    let date = Date::parse(date_str, "%Y-%m-%d").expect("Failed to parse date");
    assert_eq!(date.year, 2023);
    assert_eq!(date.month, 12);
    assert_eq!(date.day, 25);
}

#[test]
fn test_date_parse_invalid() {
    let result = Date::parse("invalid-date", "%Y-%m-%d");
    assert!(result.is_err());
}

#[test]
fn test_date_default() {
    let date = Date::default();
    let now = chrono::Local::now();
    assert_eq!(date.year, now.year());
    assert_eq!(date.month, now.month());
    assert_eq!(date.day, now.day());
}

#[test]
fn test_date_serialization() {
    let date = Date::new(2023, 12, 25).expect("Failed to create date");
    let json = serde_json::to_string(&date).expect("Failed to serialize");
    let deserialized: Date = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(date.year, deserialized.year);
    assert_eq!(date.month, deserialized.month);
    assert_eq!(date.day, deserialized.day);
}

#[test]
fn test_time_new_valid() {
    let time = Time::new(14, 30, 45, 123456).expect("Failed to create time");
    assert_eq!(time.hour, 14);
    assert_eq!(time.minute, 30);
    assert_eq!(time.second, 45);
    assert_eq!(time.microsecond, 123456);
}

#[test]
fn test_time_new_invalid_hour() {
    let result = Time::new(24, 0, 0, 0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Hour must be between 0 and 23");
}

#[test]
fn test_time_new_invalid_minute() {
    let result = Time::new(0, 60, 0, 0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Minute must be between 0 and 59");
}

#[test]
fn test_time_new_invalid_second() {
    let result = Time::new(0, 0, 60, 0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Second must be between 0 and 59");
}

#[test]
fn test_time_new_invalid_microsecond() {
    let result = Time::new(0, 0, 0, 1_000_000);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Microsecond must be between 0 and 999,999");
}

#[test]
fn test_time_now() {
    let time = Time::now();
    assert!(time.hour <= 23);
    assert!(time.minute <= 59);
    assert!(time.second <= 59);
}

#[test]
fn test_time_to_naive_time() {
    let time = Time::new(14, 30, 45, 0).expect("Failed to create time");
    let naive_time = time.to_naive_time();
    assert_eq!(naive_time.hour(), 14);
    assert_eq!(naive_time.minute(), 30);
    assert_eq!(naive_time.second(), 45);
}

#[test]
fn test_time_from_naive_time() {
    let naive_time = chrono::NaiveTime::from_hms_micro_opt(14, 30, 45, 123456).unwrap();
    let time = Time::from_naive_time(naive_time);
    assert_eq!(time.hour, 14);
    assert_eq!(time.minute, 30);
    assert_eq!(time.second, 45);
}

#[test]
fn test_time_default() {
    let time = Time::default();
    assert!(time.hour <= 23);
    assert!(time.minute <= 59);
}

#[test]
fn test_time_serialization() {
    let time = Time::new(14, 30, 45, 123456).expect("Failed to create time");
    let json = serde_json::to_string(&time).expect("Failed to serialize");
    let deserialized: Time = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(time.hour, deserialized.hour);
    assert_eq!(time.minute, deserialized.minute);
    assert_eq!(time.second, deserialized.second);
    assert_eq!(time.microsecond, deserialized.microsecond);
}

#[test]
fn test_duration_value_new() {
    let duration = DurationValue::new(2, 15, 5_000_000_000);
    assert_eq!(duration.months, 2);
    assert_eq!(duration.days, 15);
    assert_eq!(duration.nsecs, 5_000_000_000);
}

#[test]
fn test_duration_value_from_std() {
    let std_duration = Duration::new(60, 0); // 60 seconds
    let duration = DurationValue::from_std(std_duration, false);
    assert_eq!(duration.nsecs, 60_000_000_000);
}

#[test]
fn test_duration_value_from_std_negative() {
    let std_duration = Duration::new(60, 0); // 60 seconds
    let duration = DurationValue::from_std(std_duration, true);
    assert_eq!(duration.nsecs, -60_000_000_000);
}

#[test]
fn test_duration_value_serialization() {
    let duration = DurationValue::new(2, 15, 5_000_000_000);
    let json = serde_json::to_string(&duration).expect("Failed to serialize");
    let deserialized: DurationValue = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(duration.months, deserialized.months);
    assert_eq!(duration.days, deserialized.days);
    assert_eq!(duration.nsecs, deserialized.nsecs);
}

#[test]
fn test_time_utils_now_ms() {
    let now_ms = time_utils::now_ms();
    let now_sec = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64;
    assert!((now_ms - now_sec).abs() < 1000);
}

#[test]
fn test_time_utils_now_sec() {
    let now_sec = time_utils::now_sec();
    let expected_sec = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;
    assert!((now_sec - expected_sec).abs() < 2);
}

#[test]
fn test_time_utils_format_timestamp_iso() {
    let ts = Timestamp::new(1672531200, 0);
    let iso_str = time_utils::format_timestamp_iso(&ts);
    assert!(iso_str.starts_with("2023-01-01T"));
}

#[test]
fn test_time_utils_parse_timestamp_iso() {
    let ts = Timestamp::new(1672531200, 0);
    let iso_str = time_utils::format_timestamp_iso(&ts);
    let parsed_ts = time_utils::parse_timestamp_iso(&iso_str).expect("Failed to parse");
    assert_eq!(ts.secs, parsed_ts.secs);
    assert_eq!(ts.nsecs, parsed_ts.nsecs);
}

#[test]
fn test_time_utils_parse_timestamp_iso_invalid() {
    let result = time_utils::parse_timestamp_iso("invalid timestamp");
    assert!(result.is_err());
}

#[test]
fn test_time_utils_timestamp_diff() {
    let start = Timestamp::new(0, 0);
    let end = Timestamp::new(60, 0);
    let diff = time_utils::timestamp_diff(&start, &end);
    assert_eq!(diff.as_secs(), 60);
}

#[test]
fn test_timestamp_partial_eq() {
    let ts1 = Timestamp::new(1000, 500_000_000);
    let ts2 = Timestamp::new(1000, 500_000_000);
    let ts3 = Timestamp::new(2000, 0);

    assert_eq!(ts1, ts2);
    assert_ne!(ts1, ts3);
}

#[test]
fn test_date_partial_eq() {
    let date1 = Date::new(2023, 12, 25).expect("Failed to create date");
    let date2 = Date::new(2023, 12, 25).expect("Failed to create date");
    let date3 = Date::new(2023, 12, 26).expect("Failed to create date");

    assert_eq!(date1, date2);
    assert_ne!(date1, date3);
}

#[test]
fn test_time_partial_eq() {
    let time1 = Time::new(14, 30, 45, 0).expect("Failed to create time");
    let time2 = Time::new(14, 30, 45, 0).expect("Failed to create time");
    let time3 = Time::new(14, 30, 46, 0).expect("Failed to create time");

    assert_eq!(time1, time2);
    assert_ne!(time1, time3);
}
