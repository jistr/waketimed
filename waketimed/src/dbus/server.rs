use crate::messages::{DbusMsg, MainMsg};
use anyhow::{Context, Error as AnyError};
use std::env;
use std::str::FromStr;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use zbus::{dbus_interface, fdo, Connection, ConnectionBuilder};

pub struct Server {
    _main_send: UnboundedSender<MainMsg>,
}

impl Server {
    pub async fn handle_msg(&mut self, _msg: DbusMsg) {}
}

#[dbus_interface(name = "org.waketimed.waketimed1")]
impl Server {
    #[dbus_interface(out_args("earliest_sleep_time", "stayup_active"))]
    fn get_status(&self) -> fdo::Result<(u64, bool)> {
        Ok((0, false))
    }
}

pub async fn spawn_dbus_server_and_get_conn(
    main_send: UnboundedSender<MainMsg>,
) -> Result<Connection, AnyError> {
    let dbus_server = Server {
        _main_send: main_send,
    };
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
        .with_context(|| "Failed to spawn server on D-Bus")
}

pub async fn spawn_recv_loop(
    conn: Connection,
    mut dbus_recv: UnboundedReceiver<DbusMsg>,
) -> Result<(), AnyError> {
    tokio::spawn(async move {
        while let Some(msg) = dbus_recv.recv().await {
            let srv_ref = conn
                .object_server()
                .interface::<_, Server>("/org/waketimed/waketimed")
                .await
                .expect(
                    "Unable to lookup dbus::server::Server instance to process internal messages",
                );
            let mut srv = srv_ref.get_mut().await;
            srv.handle_msg(msg).await;
        }
    });
    Ok(())
}
