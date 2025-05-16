use std::time::{Duration, Instant};
use regex::Regex;

pub fn is_mobile_device(user_agent: &str) -> bool {
    let mobile_regex = Regex::new(r"(?i)(android|webos|iphone|ipad|ipod|blackberry|iemobile|opera mini)").unwrap();
    mobile_regex.is_match(user_agent)
}

pub fn generate_message_key(username: &str, text: &str, timestamp: u64) -> String {
    format!("{}:{}:{}", username, text.chars().take(20).collect::<String>(), timestamp)
}

pub fn format_timestamp(timestamp: u64) -> String {
    let date = chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    date.format("%H:%M").to_string()
}

pub fn calculate_latency(start_time: Instant) -> u64 {
    start_time.elapsed().as_millis() as u64
}

pub fn should_cleanup_messages(message_count: usize) -> bool {
    message_count > 200
}

pub fn should_cleanup_displayed_messages(displayed_count: usize) -> bool {
    displayed_count > 100
} 