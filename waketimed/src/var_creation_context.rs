use anyhow::{anyhow, Error as AnyError};
use zbus::Connection as ZbusConnection;

pub struct VarCreationContext {
    pub system_dbus_conn: Option<ZbusConnection>,
}

impl VarCreationContext {
    pub fn new(system_dbus_conn: Option<ZbusConnection>) -> Self {
        Self { system_dbus_conn }
    }

    pub fn system_dbus_conn(&self) -> Result<ZbusConnection, AnyError> {
        self.system_dbus_conn.clone().ok_or_else(|| {
            anyhow!("VarCreationContext does not contain a connection to system D-Bus.")
        })
    }
}
