use crate::config::Config;
use crate::messages::WorkerMsg;
use anyhow::Error as AnyError;
use chrono::{DateTime, Utc};
use log::debug;
use nix::time::{clock_gettime, ClockId};
use std::rc::Rc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

const SUSPEND_CLOCK: ClockId = ClockId::CLOCK_BOOTTIME_ALARM;

pub struct SleepManager {
    cfg: Rc<Config>,
    worker_send: UnboundedSender<WorkerMsg>,
    nearest_possible_suspend: Duration,
    stayup_active: bool,
    // suspend_in_progress: bool,
}

impl SleepManager {
    pub fn new(cfg: Rc<Config>, worker_send: UnboundedSender<WorkerMsg>) -> Self {
        Self {
            cfg,
            worker_send,
            nearest_possible_suspend: Duration::ZERO,
            stayup_active: true,
        }
    }

    pub fn init(&mut self) -> Result<(), AnyError> {
        self.bump_nearest_possible_suspend_from_now(Duration::from_millis(
            self.cfg.startup_awake_time,
        ))?;
        Ok(())
    }

    pub fn update(&mut self, stayup_active: bool) -> Result<(), AnyError> {
        self.stayup_active = stayup_active;
        if stayup_active {
            self.bump_nearest_possible_suspend_from_now(Duration::from_millis(
                self.cfg.stayup_cleared_awake_time + self.cfg.poll_variable_interval,
            ))?;
        }
        Ok(())
    }

    pub fn suspend_if_allowed(&mut self) -> Result<(), AnyError> {
        if self.is_suspend_allowed()? {
            self.worker_send
                .send(WorkerMsg::Suspend(self.cfg.test_mode))?;
        }
        Ok(())
    }

    fn is_suspend_allowed(&self) -> Result<bool, AnyError> {
        let now: Duration = clock_gettime(SUSPEND_CLOCK)?.into();
        Ok(now > self.nearest_possible_suspend && !self.stayup_active)
    }

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
