use crate::messages::EngineMsg;
use log::{error, info, warn};
use tokio::sync::mpsc::UnboundedSender;
use zbus::Connection as ZbusConnection;

pub struct SleepWorker {
    #[allow(dead_code)]
    engine_send: UnboundedSender<EngineMsg>,
    system_dbus_conn: Option<ZbusConnection>,
}

impl SleepWorker {
    pub fn new(
        engine_send: UnboundedSender<EngineMsg>,
        system_dbus_conn: Option<ZbusConnection>,
    ) -> Self {
        Self {
            engine_send,
            system_dbus_conn,
        }
    }

    pub async fn handle_suspend(&mut self, test_mode: bool) {
        if test_mode {
            info!("Suspending in test mode.");
            return;
        }

        info!("Suspending.");
        if matches!(self.system_dbus_conn, None) {
            error!("Attempted to suspend but system_dbus_conn is None.");
            return;
        }
        let system_dbus_conn = self.system_dbus_conn.as_ref().unwrap();
        let suspend_res = system_dbus_conn
            .call_method(
                Some("org.freedesktop.login1"),
                "/org/freedesktop/login1",
                Some("org.freedesktop.login1.Manager"),
                "Suspend",
                &[false], // non-interactive - do not prompt for authentication
            )
            .await;
        match suspend_res {
            Ok(_) => info!("Suspend successful."),
            Err(e) => warn!("Suspend unsuccessful: {}", e),
        }
    }
}
