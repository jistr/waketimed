use anyhow::Error as AnyError;
use chrono::{DateTime, Utc};
use nix::time::{clock_gettime, ClockId};
use std::time::Duration;

const SUSPEND_CLOCK: ClockId = ClockId::CLOCK_BOOTTIME_ALARM;

pub fn now() -> Result<Duration, AnyError> {
    Ok(clock_gettime(SUSPEND_CLOCK)?.into())
}

pub fn from_suspend_to_utc(suspend_clock_time: Duration) -> Result<DateTime<Utc>, AnyError> {
    let time_now = now()?;
    let from_now = chrono::Duration::from_std(suspend_clock_time - time_now)?;
    Ok(Utc::now() + from_now)
}
