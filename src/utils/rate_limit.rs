use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use anyhow::Result;

static LIMITER: OnceLock<Mutex<HashMap<String, Vec<Instant>>>> = OnceLock::new();

fn store() -> &'static Mutex<HashMap<String, Vec<Instant>>> {
    LIMITER.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn check_limit(key: &str) -> Result<()> {
    let per_min = std::env::var("KANDIL_RATE_LIMIT_PER_MIN").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(60);
    let mut map = store().lock().unwrap();
    let entry = map.entry(key.to_string()).or_insert_with(Vec::new);
    let now = Instant::now();
    entry.retain(|t| now.duration_since(*t) < Duration::from_secs(60));
    if entry.len() >= per_min { return Err(anyhow::anyhow!("Rate limit exceeded")); }
    entry.push(now);
    Ok(())
}
