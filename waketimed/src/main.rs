extern crate waketimed_core as wd_core;

mod config;
mod dbus;
mod engine;
pub mod messages;
mod worker;

use anyhow::Error as AnyError;
use engine::Engine;
use log::{error, trace};
use messages::{DbusMsg, EngineMsg, WorkerMsg};
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Notify;

pub use crate::config::get_config;

const WORKER_THREADS: usize = 3;

fn main() -> Result<(), AnyError> {
    config::load()?;
    setup_logger();
    config::log_config()?;

    let (dbus_send, dbus_recv) = unbounded_channel::<DbusMsg>();
    let (main_send, main_recv) = unbounded_channel::<EngineMsg>();
    let (worker_send, worker_recv) = unbounded_channel::<WorkerMsg>();

    let worker_thread = worker_thread_spawn(worker_recv, main_send.clone());
    let dbus_thread = dbus_thread_spawn(dbus_recv, main_send.clone());
    let signal_thread = signal_thread_spawn(main_send)?;
    main_thread_main(main_recv, dbus_send, worker_send);
    trace!("Joining signal thread.");
    signal_thread.join().expect("Failed to join signal thread.");
    trace!("Joining D-Bus thread.");
    dbus_thread.join().expect("Failed to join D-Bus thread.")?;
    trace!("Joining worker thread.");
    worker_thread
        .join()
        .expect("Failed to join worker thread.")?;
    trace!("Terminating main thread.");
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
        let terminate = msg == EngineMsg::Terminate;
        engine.handle_msg(msg);
        if terminate {
            break;
        }
    }
    trace!("Exiting main thread loop.")
}

fn dbus_thread_spawn(
    dbus_recv: UnboundedReceiver<DbusMsg>,
    main_send: UnboundedSender<EngineMsg>,
) -> JoinHandle<Result<(), AnyError>> {
    thread::Builder::new()
        .name("dbus".to_string())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread().build()?;
            runtime.block_on(dbus_thread_main(dbus_recv, main_send))
        })
        .expect("Failed to spawn D-Bus thread.")
}

async fn dbus_thread_main(
    dbus_recv: UnboundedReceiver<DbusMsg>,
    main_send: UnboundedSender<EngineMsg>,
) -> Result<(), AnyError> {
    let terminate_notify = Arc::new(Notify::new());
    let conn = dbus::server::spawn_dbus_server_and_get_conn(main_send).await?;
    dbus::server::spawn_recv_loop(conn, dbus_recv, terminate_notify.clone()).await?;
    terminate_notify.notified().await;
    trace!("Terminating D-Bus thread.");
    Ok(())
}

fn worker_thread_spawn(
    worker_recv: UnboundedReceiver<WorkerMsg>,
    main_send: UnboundedSender<EngineMsg>,
) -> JoinHandle<Result<(), AnyError>> {
    thread::Builder::new()
        .name("worker".to_string())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(WORKER_THREADS)
                .enable_io()
                .enable_time()
                .build()?;
            runtime.block_on(worker_thread_main(worker_recv, main_send))
        })
        .expect("Failed to spawn worker thread.")
}

async fn worker_thread_main(
    worker_recv: UnboundedReceiver<WorkerMsg>,
    main_send: UnboundedSender<EngineMsg>,
) -> Result<(), AnyError> {
    worker::run_recv_loop(worker_recv, main_send).await?;
    trace!("Terminating worker thread.");
    Ok(())
}

fn signal_thread_spawn(main_send: UnboundedSender<EngineMsg>) -> Result<JoinHandle<()>, AnyError> {
    let mut signals = Signals::new(&[SIGINT, SIGTERM])?;
    Ok(thread::Builder::new()
        .name("signal".to_string())
        .spawn(move || {
            let handle = signals.handle();
            for signal in &mut signals {
                match signal {
                    SIGINT | SIGTERM => {
                        main_send.send(EngineMsg::Terminate).unwrap_or_else(|e| {
                            error!("Could not send message from signal thread: {}", e)
                        });
                        handle.close();
                    }
                    _ => unreachable!(),
                }
            }
            trace!("Terminating signal thread.")
        })?)
}
