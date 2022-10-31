use crate::messages::EngineMsg;
use anyhow::{anyhow, Error as AnyError};
use futures_util::stream::StreamExt;
use log::{debug, error, info, warn};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use zbus::Connection as ZbusConnection;

pub struct SleepWorker {
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

    pub async fn handle_watch_prepare_for_sleep(&mut self) {
        let system_dbus_conn = self.system_dbus_conn.as_ref().unwrap();

        let login1_opt = self.term_on_err(
            zbus::Proxy::new(
                system_dbus_conn,
                "org.freedesktop.login1",
                "/org/freedesktop/login1",
                "org.freedesktop.login1.Manager",
            )
            .await
            .map_err(|e| anyhow!("Could not get login1 proxy: {}", e)),
        );
        if login1_opt.is_none() {
            return;
        }
        let login1 = login1_opt.unwrap();

        let signal_stream_opt = self.term_on_err(
            login1
                .receive_signal("PrepareForSleep")
                .await
                .map_err(|e| anyhow!("Could not get PrepareForSleep signal stream: {}", e)),
        );
        if signal_stream_opt.is_none() {
            return;
        }

        let mut signal_stream = signal_stream_opt.unwrap();
        let engine_send = self.engine_send.clone();
        tokio::spawn(async move {
            debug!("Spawning PrepareForSleep signal stream handler.");
            while let Some(msg) = signal_stream.next().await {
                if let Err(e) = process_prepare_for_sleep(&engine_send, msg) {
                    error!("Processing of PrepareForSleep signal failed: {:#}", e);
                    engine_send
                        .send(EngineMsg::Terminate)
                        .expect("Failed to send Terminate message.");
                }
            }
        });
    }

    pub async fn handle_suspend(&mut self, test_mode: bool) {
        if test_mode {
            info!("Requesting suspend in test mode.");
            return;
        }

        info!("Requesting suspend.");
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
            Ok(_) => info!("Suspend request successful."),
            Err(e) => warn!("Suspend request unsuccessful: {}", e),
        }
    }

    fn term_on_err<T>(&mut self, result: Result<T, AnyError>) -> Option<T> {
        match result {
            Ok(val) => Some(val),
            Err(e) => {
                error!("{:#}", e);
                self.engine_send
                    .send(EngineMsg::Terminate)
                    .expect("Failed to send Terminate message.");
                None
            }
        }
    }
}

fn process_prepare_for_sleep(
    engine_send: &UnboundedSender<EngineMsg>,
    sleep_msg: Arc<zbus::Message>,
) -> Result<(), AnyError> {
    let suspending: bool = sleep_msg.body()?;
    if suspending {
        engine_send
            .send(EngineMsg::SystemIsSuspending)
            .expect("Failed to send SystemIsSuspending message.");
    } else {
        engine_send
            .send(EngineMsg::SystemIsResuming)
            .expect("Failed to send SystemIsResuming message.");
    }
    Ok(())
}
