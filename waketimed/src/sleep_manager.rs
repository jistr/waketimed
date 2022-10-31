use crate::config::Config;
use crate::messages::WorkerMsg;
use crate::time;
use anyhow::Error as AnyError;
use log::{debug, info};
use std::rc::Rc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

pub struct SleepManager {
    cfg: Rc<Config>,
    worker_send: UnboundedSender<WorkerMsg>,
    nearest_possible_suspend: Duration,
    stayup_active: bool,
    suspend_in_progress: bool,
}

impl SleepManager {
    pub fn new(cfg: Rc<Config>, worker_send: UnboundedSender<WorkerMsg>) -> Self {
        Self {
            cfg,
            worker_send,
            nearest_possible_suspend: Duration::ZERO,
            stayup_active: true,
            suspend_in_progress: false,
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
        if self.is_suspend_allowed()? && !self.suspend_in_progress {
            self.worker_send
                .send(WorkerMsg::Suspend(self.cfg.test_mode))?;
        }
        Ok(())
    }

    pub fn handle_system_is_resuming(&mut self) {
        info!("System is resuming.");
        self.suspend_in_progress = false;
        self.bump_nearest_possible_suspend_from_now(Duration::from_millis(
            self.cfg.minimum_awake_time,
        ))
        .expect("Error trying to bump nearest suspend time.");
    }

    pub fn handle_system_is_suspending(&mut self) {
        info!("System is suspending.");
        self.suspend_in_progress = true;
    }

    fn is_suspend_allowed(&self) -> Result<bool, AnyError> {
        let now = time::now()?;
        Ok(now > self.nearest_possible_suspend && !self.stayup_active)
    }

    fn bump_nearest_possible_suspend_from_now(
        &mut self,
        from_now: Duration,
    ) -> Result<(), AnyError> {
        let now = time::now()?;
        let old_nearest_possible_suspend: Duration = self.nearest_possible_suspend;
        self.nearest_possible_suspend = self.nearest_possible_suspend.max(now + from_now);
        if old_nearest_possible_suspend != self.nearest_possible_suspend {
            debug!(
                "Nearest possible suspend: {}",
                time::from_suspend_to_utc(self.nearest_possible_suspend)?
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::default_config;
    use tokio::sync::mpsc::error::TryRecvError;
    use tokio::sync::mpsc::UnboundedReceiver;

    fn create_sleep_manager(cfg: Config) -> (SleepManager, UnboundedReceiver<WorkerMsg>) {
        let (worker_send, worker_recv) = tokio::sync::mpsc::unbounded_channel();
        let mgr = SleepManager::new(Rc::new(cfg), worker_send);
        (mgr, worker_recv)
    }

    #[test]
    fn test_update() -> Result<(), AnyError> {
        let mut cfg = default_config();
        cfg.startup_awake_time = 50;
        cfg.stayup_cleared_awake_time = 1000;
        cfg.poll_variable_interval = 1000;
        let (mut mgr, _worker_recv) = create_sleep_manager(cfg);
        mgr.init().expect("Failed to init SleepManager.");
        assert!(!mgr.is_suspend_allowed()?);
        let initial_nearest_suspend = mgr.nearest_possible_suspend;
        mgr.update(false)?;
        assert_eq!(initial_nearest_suspend, mgr.nearest_possible_suspend);
        assert!(!mgr.is_suspend_allowed()?);
        mgr.update(true)?;
        assert!(initial_nearest_suspend < mgr.nearest_possible_suspend);
        assert!(!mgr.is_suspend_allowed()?);
        Ok(())
    }

    #[test]
    fn test_suspend() -> Result<(), AnyError> {
        let mut cfg = default_config();
        cfg.startup_awake_time = 0;
        cfg.stayup_cleared_awake_time = 1000;
        cfg.poll_variable_interval = 1000;
        let (mut mgr, mut worker_recv) = create_sleep_manager(cfg);
        mgr.init().expect("Failed to init SleepManager.");

        // Initially suspend is not allowed because internal
        // stayup_active is set to true.
        assert!(!mgr.is_suspend_allowed()?);
        mgr.suspend_if_allowed()?;
        assert_eq!(worker_recv.try_recv(), Err(TryRecvError::Empty));

        // First update() will set stayup_active to false and not
        // increase nearest_possible_suspend. Suspend should be
        // allowed.
        mgr.update(false)?;
        assert!(mgr.is_suspend_allowed()?);
        mgr.suspend_if_allowed()?;
        assert_eq!(worker_recv.try_recv(), Ok(WorkerMsg::Suspend(true)));

        // Second update now sets stayup_active true again. Suspend
        // should not be allowed.
        mgr.update(true)?;
        assert!(!mgr.is_suspend_allowed()?);
        mgr.suspend_if_allowed()?;
        assert_eq!(worker_recv.try_recv(), Err(TryRecvError::Empty));

        Ok(())
    }
}
