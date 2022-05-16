use anyhow::{anyhow, Error as AnyError};
use log::warn;
use zbus::blocking::Connection as ZbusConnection;

pub struct VarCreationContext {
    // FIXME: Blocking connection will spawn a per-request async
    // runtime. We should use async connection when PollVarFns trait
    // can support async fns (when Rust supports async closures on
    // stable).
    pub system_dbus_conn: Option<ZbusConnection>,
}

impl VarCreationContext {
    pub fn new() -> Result<Self, AnyError> {
        let system_dbus_conn = ZbusConnection::system();
        if let Err(ref e) = system_dbus_conn {
            warn!("Unable to connect to system D-Bus, variables relying on it will not work. Reason: {}", e);
        }
        let system_dbus_conn = system_dbus_conn.ok();
        Ok(Self { system_dbus_conn })
    }

    pub fn system_dbus_conn(&self) -> Result<ZbusConnection, AnyError> {
        self.system_dbus_conn.clone().ok_or_else(|| {
            anyhow!("VarCreationContext does not contain a connection to system D-Bus.")
        })
    }
}
