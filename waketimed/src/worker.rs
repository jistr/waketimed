use crate::messages::{EngineMsg, WorkerMsg};
use anyhow::Error as AnyError;
use log::trace;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct Worker {
    #[allow(dead_code)]
    main_send: UnboundedSender<EngineMsg>,
}

impl Worker {
    pub async fn handle_msg(&mut self, _msg: WorkerMsg) {}
}

pub async fn run_recv_loop(
    mut worker_recv: UnboundedReceiver<WorkerMsg>,
    main_send: UnboundedSender<EngineMsg>,
) -> Result<(), AnyError> {
    let mut worker = Worker { main_send };
    while let Some(msg) = worker_recv.recv().await {
        let terminate = msg == WorkerMsg::Terminate;
        worker.handle_msg(msg).await;
        if terminate {
            break;
        }
    }
    trace!("Exiting worker thread receiver loop.");
    Ok(())
}
