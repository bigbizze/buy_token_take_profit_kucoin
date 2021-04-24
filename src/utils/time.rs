use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{ Result };

pub fn get_ms() -> Result<u128> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)?;
    Ok(since_the_epoch.as_millis())
}

pub fn get_ms_str() -> Result<String> {
    Ok(get_ms()?.to_string())
}
