use anyhow::{anyhow, Error as AnyError};
use log::warn;
use std::thread;
use zbus::blocking::Connection as ZbusConnection;

pub struct VarCreationContext {
    // FIXME: Blocking connection will spawn a per-request async
    // runtime. We should use async connection when PollVarFns trait
    // can support async fns (when Rust supports async closures on
    // stable).
    pub system_dbus_conn: Option<ZbusConnection>,
}

impl VarCreationContext {
    pub fn new() -> Self {
        let system_dbus_conn = thread::spawn(ZbusConnection::system)
            .join()
            .expect("Failed to join D-Bus connection creation thread.");
        if let Err(ref e) = system_dbus_conn {
            warn!("Unable to connect to system D-Bus, variables relying on it will not work. Reason: {}", e);
        }
        let system_dbus_conn = system_dbus_conn.ok();

        Self { system_dbus_conn }
    }

    pub fn system_dbus_conn(&self) -> Result<ZbusConnection, AnyError> {
        self.system_dbus_conn.clone().ok_or_else(|| {
            anyhow!("VarCreationContext does not contain a connection to system D-Bus.")
        })
    }
}
