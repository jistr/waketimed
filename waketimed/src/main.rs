extern crate waketimed_core as wd_core;

mod config;
mod dbus;
pub mod messages;

use anyhow::Error as AnyError;
use messages::{DbusMsg, MainMsg, WorkerMsg};
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub use crate::config::get_config;

fn main() -> Result<(), AnyError> {
    config::load()?;
    setup_logger();
    config::log_config()?;

    let (_dbus_send, dbus_recv) = unbounded_channel::<DbusMsg>();
    let (main_send, _main_recv) = unbounded_channel::<MainMsg>();
    let (_worker_send, _worker_recv) = unbounded_channel::<WorkerMsg>();

    let dbus_thread = dbus_thread_spawn(dbus_recv, main_send);
    dbus_thread.join().expect("Failed to join DBus thread.")?;
    Ok(())
}

fn setup_logger() {
    let cfg = get_config();
    env_logger::builder()
        .parse_filters(&cfg.borrow().log)
        .init();
}

fn dbus_thread_spawn(
    dbus_recv: UnboundedReceiver<DbusMsg>,
    main_send: UnboundedSender<MainMsg>,
) -> JoinHandle<Result<(), AnyError>> {
    thread::spawn(move || {
        // FIXME: use current_thread for Dbus
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()?;
        runtime.block_on(dbus_thread_main(dbus_recv, main_send))
    })
}

async fn dbus_thread_main(
    dbus_recv: UnboundedReceiver<DbusMsg>,
    main_send: UnboundedSender<MainMsg>,
) -> Result<(), AnyError> {
    let conn = dbus::server::spawn_dbus_server_and_get_conn(main_send).await?;
    dbus::server::spawn_recv_loop(conn, dbus_recv).await?;
    std::future::pending::<()>().await;
    Ok(())
}
