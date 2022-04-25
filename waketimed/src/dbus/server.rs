use crate::messages::{DbusMsg, EngineMsg};
use anyhow::{Context, Error as AnyError};
use log::trace;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Notify;
use zbus::{dbus_interface, fdo, Connection, ConnectionBuilder};

pub struct Server {
    _engine_send: UnboundedSender<EngineMsg>,
}

impl Server {
    pub async fn handle_msg(&mut self, msg: DbusMsg) {
        trace!("Received DbusMsg::{:?}.", &msg);
        match msg {
            DbusMsg::Terminate => {} // handled in the recv loop
        }
    }
}

#[dbus_interface(name = "org.waketimed.waketimed1")]
impl Server {
    #[dbus_interface(out_args("earliest_sleep_time", "stayup_active"))]
    fn get_status(&self) -> fdo::Result<(u64, bool)> {
        Ok((0, false))
    }
}

pub async fn spawn_dbus_server_and_get_conn(
    engine_send: UnboundedSender<EngineMsg>,
) -> Result<Connection, AnyError> {
    let dbus_server = Server {
        _engine_send: engine_send,
    };
    let builder = if let Ok(address) = env::var("WAKETIMED_BUS_ADDRESS") {
        trace!("Using D-Bus address: '{}'", &address);
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
    terminate_notify: Arc<Notify>,
) -> Result<(), AnyError> {
    tokio::spawn(async move {
        trace!("Starting D-Bus thread receiver loop.");
        while let Some(msg) = dbus_recv.recv().await {
            let srv_ref = conn
                .object_server()
                .interface::<_, Server>("/org/waketimed/waketimed")
                .await
                .expect(
                    "Unable to lookup dbus::server::Server instance to process internal messages",
                );
            let mut srv = srv_ref.get_mut().await;
            let terminate = msg == DbusMsg::Terminate;
            srv.handle_msg(msg).await;
            if terminate {
                terminate_notify.notify_one();
                break;
            }
        }
        trace!("Exiting D-Bus thread receiver loop.")
    });
    Ok(())
}
