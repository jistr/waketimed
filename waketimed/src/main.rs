extern crate waketimed_core as wd_core;

mod config;
mod dbus;
mod engine;
pub mod messages;

use anyhow::Error as AnyError;
use engine::Engine;
use messages::{DbusMsg, EngineMsg, WorkerMsg};
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub use crate::config::get_config;

fn main() -> Result<(), AnyError> {
    config::load()?;
    setup_logger();
    config::log_config()?;

    let (dbus_send, dbus_recv) = unbounded_channel::<DbusMsg>();
    let (main_send, main_recv) = unbounded_channel::<EngineMsg>();
    let (worker_send, _worker_recv) = unbounded_channel::<WorkerMsg>();

    let dbus_thread = dbus_thread_spawn(dbus_recv, main_send);
    main_thread_main(main_recv, dbus_send, worker_send);
    dbus_thread.join().expect("Failed to join DBus thread.")?;
    Ok(())
}

fn setup_logger() {
    let cfg = get_config();
    env_logger::builder()
        .parse_filters(&cfg.borrow().log)
        .init();
}

fn main_thread_main(
    mut main_recv: UnboundedReceiver<EngineMsg>,
    dbus_send: UnboundedSender<DbusMsg>,
    worker_send: UnboundedSender<WorkerMsg>,
) {
    let mut engine = Engine::new(dbus_send, worker_send);
    while let Some(msg) = main_recv.blocking_recv() {
        engine.handle_msg(msg);
    }
}

fn dbus_thread_spawn(
    dbus_recv: UnboundedReceiver<DbusMsg>,
    main_send: UnboundedSender<EngineMsg>,
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
    main_send: UnboundedSender<EngineMsg>,
) -> Result<(), AnyError> {
    let conn = dbus::server::spawn_dbus_server_and_get_conn(main_send).await?;
    dbus::server::spawn_recv_loop(conn, dbus_recv).await?;
    std::future::pending::<()>().await;
    Ok(())
}
