//! 日期时间类型模块
//!
//! 本模块定义了日期、时间、日期时间和持续时间类型及其相关操作。

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// 简单日期表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct DateValue {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl DateValue {
    /// 估算日期的内存使用大小
    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    /// 添加持续时间
    pub fn add_duration(&mut self, duration: &DurationValue) {
        if duration.months != 0 {
            self.add_months(duration.months);
        }
        
        let total_days = duration.seconds / 86400;
        if total_days != 0 {
            self.add_days(total_days);
        }
    }

    /// 减去持续时间
    pub fn sub_duration(&mut self, duration: &DurationValue) {
        if duration.months != 0 {
            self.add_months(-duration.months);
        }
        
        let total_days = duration.seconds / 86400;
        if total_days != 0 {
            self.add_days(-total_days);
        }
    }

    fn add_months(&mut self, months: i32) {
        let mut new_month = self.month as i32 + months;
        let mut year_delta = 0;
        
        while new_month > 12 {
            new_month -= 12;
            year_delta += 1;
        }
        
        while new_month < 1 {
            new_month += 12;
            year_delta -= 1;
        }
        
        self.year += year_delta;
        self.month = new_month as u32;
        
        self.normalize_day();
    }

    fn add_days(&mut self, days: i64) {
        let total_days = Self::to_days(self) + days;
        *self = Self::from_days(total_days);
    }

    fn normalize_day(&mut self) {
        let days_in_month = Self::days_in_month(self.year, self.month);
        if self.day > days_in_month {
            self.day = days_in_month;
        }
    }

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
            _ => panic!("无效的月份"),
        }
    }

    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    pub fn to_days(&self) -> i64 {
        let year = self.year as i64;
        let month = self.month as i64;
        let day = self.day as i64;

        let a = (14 - month) / 12;
        let y = year + 4800 - a;
        let m = month + 12 * a - 3;

        day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
    }

    fn from_days(days: i64) -> Self {
        let a = days + 32044;
        let b = (4 * a + 3) / 146097;
        let c = a - (146097 * b) / 4;
        let d = (4 * c + 3) / 1461;
        let e = c - (1461 * d) / 4;
        let m = (5 * e + 2) / 153;

        let day = e - (153 * m + 2) / 5 + 1;
        let month = m + 3 - 12 * (m / 10);
        let year = 100 * b + d - 4800 + m / 10;

        DateValue {
            year: year as i32,
            month: month as u32,
            day: day as u32,
        }
    }
}

impl Default for DateValue {
    fn default() -> Self {
        DateValue {
            year: 1970,
            month: 1,
            day: 1,
        }
    }
}

/// 简单时间表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct TimeValue {
    pub hour: u32,
    pub minute: u32,
    pub sec: u32,
    pub microsec: u32,
}

impl TimeValue {
    /// 估算时间的内存使用大小
    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    /// 添加持续时间
    pub fn add_duration(&mut self, duration: &DurationValue) {
        let total_microseconds = duration.seconds * 1_000_000 + duration.microseconds as i64;
        let mut new_microseconds = self.microsec as i64 + total_microseconds;
        
        while new_microseconds >= 86_400_000_000 {
            new_microseconds -= 86_400_000_000;
        }
        while new_microseconds < 0 {
            new_microseconds += 86_400_000_000;
        }
        
        self.microsec = (new_microseconds % 1_000_000) as u32;
        let total_seconds = new_microseconds / 1_000_000;
        
        let mut total_time = self.hour as i64 * 3600 + self.minute as i64 * 60 + self.sec as i64 + total_seconds;
        
        while total_time >= 86_400 {
            total_time -= 86_400;
        }
        while total_time < 0 {
            total_time += 86_400;
        }
        
        self.hour = (total_time / 3600) as u32;
        self.minute = ((total_time % 3600) / 60) as u32;
        self.sec = (total_time % 60) as u32;
    }

    /// 减去持续时间
    pub fn sub_duration(&mut self, duration: &DurationValue) {
        let neg_duration = DurationValue {
            seconds: -duration.seconds,
            microseconds: -duration.microseconds,
            months: 0,
        };
        self.add_duration(&neg_duration);
    }
}

impl Default for TimeValue {
    fn default() -> Self {
        TimeValue {
            hour: 0,
            minute: 0,
            sec: 0,
            microsec: 0,
        }
    }
}

/// 简单日期时间表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct DateTimeValue {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub sec: u32,
    pub microsec: u32,
}

impl DateTimeValue {
    /// 估算日期时间的内存使用大小
    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    /// 添加持续时间
    pub fn add_duration(&mut self, duration: &DurationValue) {
        let mut date = DateValue {
            year: self.year,
            month: self.month,
            day: self.day,
        };
        date.add_duration(duration);
        
        let mut time = TimeValue {
            hour: self.hour,
            minute: self.minute,
            sec: self.sec,
            microsec: self.microsec,
        };
        time.add_duration(duration);
        
        self.year = date.year;
        self.month = date.month;
        self.day = date.day;
        self.hour = time.hour;
        self.minute = time.minute;
        self.sec = time.sec;
        self.microsec = time.microsec;
    }

    /// 减去持续时间
    pub fn sub_duration(&mut self, duration: &DurationValue) {
        let mut date = DateValue {
            year: self.year,
            month: self.month,
            day: self.day,
        };
        date.sub_duration(duration);
        
        let mut time = TimeValue {
            hour: self.hour,
            minute: self.minute,
            sec: self.sec,
            microsec: self.microsec,
        };
        time.sub_duration(duration);
        
        self.year = date.year;
        self.month = date.month;
        self.day = date.day;
        self.hour = time.hour;
        self.minute = time.minute;
        self.sec = time.sec;
        self.microsec = time.microsec;
    }
}

impl Default for DateTimeValue {
    fn default() -> Self {
        DateTimeValue {
            year: 1970,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            sec: 0,
            microsec: 0,
        }
    }
}

/// 简单持续时间表示
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct DurationValue {
    pub seconds: i64,
    pub microseconds: i32,
    pub months: i32,
}

impl DurationValue {
    /// 估算持续时间的内存使用大小
    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl Default for DurationValue {
    fn default() -> Self {
        DurationValue {
            seconds: 0,
            microseconds: 0,
            months: 0,
        }
    }
}
