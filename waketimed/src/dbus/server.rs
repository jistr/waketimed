use std::env;
use std::str::FromStr;
use zbus::{dbus_interface, fdo, ConnectionBuilder};

use anyhow::{Context, Error as AnyError};

pub struct Server {}

#[dbus_interface(name = "org.waketimed.waketimed1")]
impl Server {
    #[dbus_interface(out_args("earliest_sleep_time", "stayup_active"))]
    fn get_status(&self) -> fdo::Result<(u64, bool)> {
        Ok((0, false))
    }
}

pub async fn spawn() -> Result<(), AnyError> {
    let dbus_server = Server {};
    let builder = if let Ok(address) = env::var("WAKETIMED_BUS_ADDRESS") {
        ConnectionBuilder::address(
            zbus::Address::from_str(&address)
                .with_context(|| "Failed to parse WAKETIMED_BUS_ADDRESS")?,
        )
        .with_context(|| "Failed to connect to D-Bus")?
    } else {
        ConnectionBuilder::system().with_context(|| "Failed to connect to D-Bus")?
    };
    builder
        .name("org.waketimed.waketimed")?
        .serve_at("/org/waketimed/waketimed", dbus_server)?
        .build()
        .await
        .with_context(|| "Failed to spawn server on D-Bus")?;
    Ok(())
}
