extern crate waketimed_core as wtd_core;

pub(crate) mod chassis_check;
mod config;
mod engine;
pub(crate) mod files;
pub(crate) mod messages;
pub(crate) mod rule_manager;
pub(crate) mod sleep_manager;
#[cfg(test)]
pub(crate) mod test_helpers;
pub(crate) mod var_creation_context;
pub(crate) mod var_fns;
pub(crate) mod var_manager;
mod worker;

use crate::config::Config;
use crate::engine::Engine;
use crate::messages::{EngineMsg, WorkerMsg};
use anyhow::Error as AnyError;
use log::{error, trace};
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::rc::Rc;

use std::thread::{self, JoinHandle};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

const WORKER_THREADS: usize = 3;

fn main() -> Result<(), AnyError> {
    let cfg = config::load()?;
    setup_logger(&cfg);
    config::log_config(&cfg)?;

    let (engine_send, engine_recv) = unbounded_channel::<EngineMsg>();
    let (worker_send, worker_recv) = unbounded_channel::<WorkerMsg>();

    let worker_thread = worker_thread_spawn(worker_recv, engine_send.clone());
    let signal_thread = signal_thread_spawn(engine_send.clone())?;
    main_thread_main(cfg, engine_recv, engine_send, worker_send)?;
    trace!("Joining signal thread.");
    signal_thread.join().expect("Failed to join signal thread.");
    trace!("Joining worker thread.");
    worker_thread
        .join()
        .expect("Failed to join worker thread.")?;
    trace!("Terminating main thread.");
    Ok(())
}

fn setup_logger(cfg: &Config) {
    env_logger::builder().parse_filters(&cfg.log).init();
}

fn main_thread_main(
    cfg: Config,
    mut engine_recv: UnboundedReceiver<EngineMsg>,
    engine_send: UnboundedSender<EngineMsg>,
    worker_send: UnboundedSender<WorkerMsg>,
) -> Result<(), AnyError> {
    let mut engine = Engine::new(Rc::new(cfg), engine_send, worker_send)?;
    engine.init()?;
    while let Some(msg) = engine_recv.blocking_recv() {
        let terminate = msg == EngineMsg::Terminate;
        engine.handle_msg(msg);
        if terminate {
            break;
        }
    }
    trace!("Exiting main thread loop.");
    Ok(())
}

fn worker_thread_spawn(
    worker_recv: UnboundedReceiver<WorkerMsg>,
    engine_send: UnboundedSender<EngineMsg>,
) -> JoinHandle<Result<(), AnyError>> {
    thread::Builder::new()
        .name("worker".to_string())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(WORKER_THREADS)
                .enable_io()
                .enable_time()
                .build()?;
            runtime.block_on(worker_thread_main(worker_recv, engine_send))
        })
        .expect("Failed to spawn worker thread.")
}

async fn worker_thread_main(
    worker_recv: UnboundedReceiver<WorkerMsg>,
    engine_send: UnboundedSender<EngineMsg>,
) -> Result<(), AnyError> {
    trace!("Starting worker thread.");
    worker::run_recv_loop(worker_recv, engine_send).await?;
    trace!("Terminating worker thread.");
    Ok(())
}

fn signal_thread_spawn(
    engine_send: UnboundedSender<EngineMsg>,
) -> Result<JoinHandle<()>, AnyError> {
    let mut signals = Signals::new(&[SIGINT, SIGTERM])?;
    Ok(thread::Builder::new()
        .name("signal".to_string())
        .spawn(move || {
            trace!("Starting signal thread.");
            let handle = signals.handle();
            for signal in &mut signals {
                match signal {
                    SIGINT | SIGTERM => {
                        engine_send.send(EngineMsg::Terminate).unwrap_or_else(|e| {
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
