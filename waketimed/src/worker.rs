use crate::messages::{EngineMsg, WorkerMsg};
use anyhow::{Context, Error as AnyError};
use log::{error, trace};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use wtd_core::vars::VarName;

pub struct Worker {
    engine_send: UnboundedSender<EngineMsg>,
}

impl Worker {
    pub async fn handle_msg(&mut self, msg: WorkerMsg) {
        use WorkerMsg::*;
        match msg {
            CallVarIsActive(var_name, is_active_fn) => {
                self.handle_call_is_active(var_name, is_active_fn).await
            }
            Terminate => {} // handled in the recv loop
        }
    }

    async fn handle_call_is_active(
        &mut self,
        var_name: VarName,
        is_active_fn: Box<dyn FnOnce() -> bool + Send + Sync>,
    ) {
        // TODO: For now this will block one of the worker threads. It
        // should be investigated if is_active_fn can be async.
        self.engine_send
            .send(EngineMsg::ReturnVarIsActive(var_name, is_active_fn()))
            .context("Could not send EngineMsg::ReturnVarIsActive")
            .unwrap_or_else(|e| error!("{:?}", e));
    }
}

pub async fn run_recv_loop(
    mut worker_recv: UnboundedReceiver<WorkerMsg>,
    engine_send: UnboundedSender<EngineMsg>,
) -> Result<(), AnyError> {
    let mut worker = Worker { engine_send };
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
