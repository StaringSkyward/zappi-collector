use chrono::{Timelike, NaiveTime};

pub fn electricity_rate(time: NaiveTime) -> f32 {
    if time.hour() == 23 && time.minute() >= 30 || (time < NaiveTime::from_hms_opt(5, 30, 0).unwrap()) {
        return 7.50
    }

    29.560
}

pub fn joules_to_watts(value: i64, period_seconds: u64) -> i64 {
    let secs_i64 = period_seconds as i64;

    if value == 0 || value.abs() < secs_i64 {
        return 0;
    }

    value / secs_i64
}