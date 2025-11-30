use std::time::Duration;
use chrono::{DateTime, Utc};

pub fn timestamp_to_german_datetime(timestamp: i64) -> String {
    // Convert timestamp to DateTime<Utc>
    let dt: DateTime<Utc> = match DateTime::from_timestamp(timestamp, 0) {
        Some(datetime) => datetime,
        None => return String::from("unknown"),
    };

    // Format with German locale
    dt.format("%d.%m.%Y").to_string()
}

pub fn format_duration(duration: &Option<Duration>) -> String {
    duration
        .map(|d| {
            let total_seconds = d.as_secs();
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            format!("{}:{}", hours, minutes)
        })
        .unwrap_or_else(|| "unknown".to_string())
}
