/// 日期计算工具模块
///
/// 提供日期转换和计算功能，支持闰年和不同月份的天数

/// 月份天数表（非闰年）
const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// 累计天数表（非闰年）
const DAYS_SO_FAR: [i64; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

/// 累计天数表（闰年）
const LEAP_DAYS_SO_FAR: [i64; 13] = [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366];

/// 判断是否为闰年
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// 获取某年某月的天数
pub fn days_in_month(year: i32, month: u32) -> u32 {
    if month == 2 && is_leap_year(year) {
        29
    } else if month >= 1 && month <= 12 {
        DAYS_IN_MONTH[(month - 1) as usize]
    } else {
        0
    }
}

/// 将天数转换为日期（从 1970-01-01 开始）
pub fn days_to_date(days: i64) -> (i32, u32, u32) {
    let epoch_year = 1970;
    let mut year = epoch_year;
    let mut remaining_days = days;

    while remaining_days >= 365 {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days >= days_in_year {
            remaining_days -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    let month_days = if is_leap_year(year) {
        &LEAP_DAYS_SO_FAR
    } else {
        &DAYS_SO_FAR
    };

    let mut month = 1;
    for m in 1..=12 {
        if remaining_days < month_days[m as usize] {
            month = m;
            break;
        }
    }

    let day = (remaining_days - month_days[(month - 1) as usize] + 1) as u32;

    (year, month, day)
}

/// 将日期转换为天数（从 1970-01-01 开始）
pub fn date_to_days(year: i32, month: u32, day: u32) -> i64 {
    let epoch_year = 1970;
    let mut days = 0i64;

    for y in epoch_year..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    let month_days = if is_leap_year(year) {
        &LEAP_DAYS_SO_FAR
    } else {
        &DAYS_SO_FAR
    };

    days += month_days[(month - 1) as usize];
    days += day as i64 - 1;

    days
}

/// 将时间戳转换为 DateTime
pub fn timestamp_to_datetime(timestamp: i64) -> (i32, u32, u32, u32, u32, u32, u32) {
    let days = timestamp / 86400;
    let seconds_in_day = timestamp % 86400;

    let (year, month, day) = days_to_date(days);

    let hour = (seconds_in_day / 3600) as u32;
    let minute = ((seconds_in_day % 3600) / 60) as u32;
    let second = (seconds_in_day % 60) as u32;
    let microsecond = 0;

    (year, month, day, hour, minute, second, microsecond)
}

/// 将 DateTime 转换为时间戳
pub fn datetime_to_timestamp(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    _microsecond: u32,
) -> i64 {
    let days = date_to_days(year, month, day);
    let seconds = days * 86400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64;
    seconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2020));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2021));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2020, 2), 29);
        assert_eq!(days_in_month(2021, 2), 28);
        assert_eq!(days_in_month(2021, 1), 31);
        assert_eq!(days_in_month(2021, 4), 30);
    }

    #[test]
    fn test_days_to_date_epoch() {
        let (year, month, day) = days_to_date(0);
        assert_eq!((year, month, day), (1970, 1, 1));
    }

    #[test]
    fn test_days_to_date_one_year() {
        let (year, month, day) = days_to_date(365);
        assert_eq!((year, month, day), (1971, 1, 1));
    }

    #[test]
    fn test_days_to_date_leap_year() {
        let (year, month, day) = days_to_date(366);
        assert_eq!((year, month, day), (1971, 1, 2));
    }

    #[test]
    fn test_date_to_days_epoch() {
        let days = date_to_days(1970, 1, 1);
        assert_eq!(days, 0);
    }

    #[test]
    fn test_date_to_days_roundtrip() {
        let original_days = 10000;
        let (year, month, day) = days_to_date(original_days);
        let converted_days = date_to_days(year, month, day);
        assert_eq!(original_days, converted_days);
    }

    #[test]
    fn test_timestamp_to_datetime_epoch() {
        let (year, month, day, hour, minute, second, microsecond) = timestamp_to_datetime(0);
        assert_eq!((year, month, day, hour, minute, second, microsecond), (1970, 1, 1, 0, 0, 0, 0));
    }

    #[test]
    fn test_timestamp_to_datetime_one_hour() {
        let (year, month, day, hour, minute, second, microsecond) = timestamp_to_datetime(3600);
        assert_eq!((year, month, day, hour, minute, second, microsecond), (1970, 1, 1, 1, 0, 0, 0));
    }

    #[test]
    fn test_timestamp_to_datetime_one_day() {
        let (year, month, day, hour, minute, second, microsecond) = timestamp_to_datetime(86400);
        assert_eq!((year, month, day, hour, minute, second, microsecond), (1971, 1, 1, 0, 0, 0, 0));
    }

    #[test]
    fn test_datetime_to_timestamp_epoch() {
        let timestamp = datetime_to_timestamp(1970, 1, 1, 0, 0, 0, 0);
        assert_eq!(timestamp, 0);
    }

    #[test]
    fn test_datetime_to_timestamp_roundtrip() {
        let original_timestamp = 1234567890;
        let (year, month, day, hour, minute, second, microsecond) = timestamp_to_datetime(original_timestamp);
        let converted_timestamp = datetime_to_timestamp(year, month, day, hour, minute, second, microsecond);
        assert_eq!(original_timestamp, converted_timestamp);
    }
}
