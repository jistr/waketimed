use anyhow::{anyhow, Error as AnyError};
use log::warn;
use zbus::Connection;

pub struct VarCreationContext {
    pub system_dbus_conn: Option<Connection>,
}

impl VarCreationContext {
    pub fn new() -> Result<Self, AnyError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()?;
        let system_dbus_conn = runtime.block_on(Connection::system()).ok();
        if system_dbus_conn.is_none() {
            warn!("Unable to connect to system D-Bus, variables relying on it will not work.");
        }
        Ok(Self { system_dbus_conn })
    }

    pub fn system_dbus_conn(&self) -> Result<Connection, AnyError> {
        self.system_dbus_conn.clone().ok_or_else(|| {
            anyhow!("VarCreationContext does not contain a connection to system D-Bus.")
        })
    }
}
