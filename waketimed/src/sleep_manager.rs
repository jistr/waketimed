use crate::config::Config;
use anyhow::Error as AnyError;
use chrono::{DateTime, Utc};
use log::debug;
use nix::time::{clock_gettime, ClockId};
use std::rc::Rc;
use std::time::Duration;

const SUSPEND_CLOCK: ClockId = ClockId::CLOCK_BOOTTIME_ALARM;

pub struct SleepManager {
    cfg: Rc<Config>,
    nearest_possible_suspend: Duration,
}

impl SleepManager {
    pub fn new(cfg: Rc<Config>) -> Self {
        Self {
            cfg,
            nearest_possible_suspend: Duration::ZERO,
        }
    }

    pub fn init(&mut self) -> Result<(), AnyError> {
        self.bump_nearest_possible_suspend_from_now(Duration::from_millis(
            self.cfg.startup_awake_time,
        ))?;
        Ok(())
    }

    pub fn update(&mut self, stayup_active: bool) -> Result<(), AnyError> {
        if stayup_active {
            self.bump_nearest_possible_suspend_from_now(Duration::from_millis(
                self.cfg.stayup_cleared_awake_time + self.cfg.poll_variable_interval,
            ))?;
        }
        Ok(())
    }

    // pub fn suspend_if_allowed(&mut self) -> Result<(), AnyError> {
    // }

    fn bump_nearest_possible_suspend_from_now(
        &mut self,
        from_now: Duration,
    ) -> Result<(), AnyError> {
        let now: Duration = clock_gettime(SUSPEND_CLOCK)?.into();
        let old_nearest_possible_suspend: Duration = self.nearest_possible_suspend;
        self.nearest_possible_suspend = self.nearest_possible_suspend.max(now + from_now);
        if old_nearest_possible_suspend != self.nearest_possible_suspend {
            debug!(
                "Nearest possible suspend: {}",
                clock_from_suspend_to_utc(self.nearest_possible_suspend)?
            );
        }
        Ok(())
    }
}

pub fn clock_from_suspend_to_utc(suspend_clock_time: Duration) -> Result<DateTime<Utc>, AnyError> {
    let now: Duration = clock_gettime(SUSPEND_CLOCK)?.into();
    let from_now = chrono::Duration::from_std(suspend_clock_time - now)?;
    Ok(Utc::now() + from_now)
}

#[cfg(test)]
mod tests {
    // TODO: Add unit tests
}
