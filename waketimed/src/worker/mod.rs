mod sleep_worker;
mod var_worker;
use self::sleep_worker::SleepWorker;
use self::var_worker::VarWorker;
use crate::messages::{EngineMsg, WorkerMsg};
use anyhow::Error as AnyError;
use log::{trace, warn};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use zbus::Connection as ZbusConnection;

pub struct Worker {
    sleep_worker: SleepWorker,
    var_worker: VarWorker,
}

impl Worker {
    pub async fn new(engine_send: UnboundedSender<EngineMsg>) -> Self {
        let system_dbus_conn = ZbusConnection::system().await;
        if let Err(ref e) = system_dbus_conn {
            warn!("Unable to connect to system D-Bus, variables and features relying on it will not work. Reason: {}", e);
        }
        let system_dbus_conn = system_dbus_conn.ok();
        let sleep_worker = SleepWorker::new(engine_send.clone(), system_dbus_conn.clone());
        let var_worker = VarWorker::new(engine_send, system_dbus_conn);

        Self {
            sleep_worker,
            var_worker,
        }
    }

    pub async fn handle_msg(&mut self, msg: WorkerMsg) {
        use WorkerMsg::*;
        trace!("Received WorkerMsg::{:?}.", &msg);
        match msg {
            CallVarPoll(var_name) => self.var_worker.handle_call_var_poll(var_name).await,
            LoadPollVarFns(var_def) => self.var_worker.handle_load_poll_var_fns(var_def).await,
            SpawnPollVarInterval(interval) => {
                self.var_worker
                    .handle_spawn_poll_var_interval(interval)
                    .await
            }
            Suspend(test_mode) => self.sleep_worker.handle_suspend(test_mode).await,
            Terminate => {} // handled in the recv loop
        }
    }
}

pub async fn run_recv_loop(
    mut worker_recv: UnboundedReceiver<WorkerMsg>,
    engine_send: UnboundedSender<EngineMsg>,
) -> Result<(), AnyError> {
    let mut worker = Worker::new(engine_send).await;
    while let Some(msg) = worker_recv.recv().await {
        let terminate = matches!(msg, WorkerMsg::Terminate);
        worker.handle_msg(msg).await;
        if terminate {
            break;
        }
    }
    trace!("Exiting worker thread receiver loop.");
    Ok(())
}
