use crate::messages::{EngineMsg, WorkerMsg};
use anyhow::{Context, Error as AnyError};
use log::{error, trace};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::{self, Duration};
use wtd_core::vars::{VarName, VarValue};

pub struct Worker {
    engine_send: UnboundedSender<EngineMsg>,
    poll_var_task: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new(engine_send: UnboundedSender<EngineMsg>) -> Self {
        Self {
            engine_send,
            poll_var_task: None,
        }
    }

    pub async fn handle_msg(&mut self, msg: WorkerMsg) {
        use WorkerMsg::*;
        trace!("Received WorkerMsg::{:?}.", &msg);
        match msg {
            CallVarIsActive(var_name, is_active_fn) => {
                self.handle_call_var_is_active(var_name, is_active_fn).await
            }
            CallVarPoll(var_name, poll_fn) => self.handle_call_var_poll(var_name, poll_fn).await,
            SpawnPollVarInterval(interval) => self.handle_spawn_poll_var_interval(interval).await,
            Terminate => {} // handled in the recv loop
        }
    }

    async fn handle_call_var_is_active(
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

    async fn handle_call_var_poll(
        &mut self,
        var_name: VarName,
        poll_fn: Box<dyn FnOnce() -> VarValue + Send + Sync>,
    ) {
        // TODO: For now this will block one of the worker threads. It
        // should be investigated if poll_fn can be async.
        self.engine_send
            .send(EngineMsg::ReturnVarPoll(var_name, poll_fn()))
            .context("Could not send EngineMsg::ReturnVarIsActive")
            .unwrap_or_else(|e| error!("{:?}", e));
    }

    async fn handle_spawn_poll_var_interval(&mut self, millis: u64) {
        if let Some(task) = self.poll_var_task.take() {
            trace!("Aborting old poll var interval task.");
            task.abort();
            task.await.ok();
        }
        trace!("Spawning a poll var interval task.");
        let engine_send = self.engine_send.clone();
        self.poll_var_task = Some(tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(millis));
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                engine_send
                    .send(EngineMsg::PollVarsTick)
                    .unwrap_or_else(|e| error!("Failed to send EngineMsg::TickPollVars: {}", e));
            }
        }));
    }
}

pub async fn run_recv_loop(
    mut worker_recv: UnboundedReceiver<WorkerMsg>,
    engine_send: UnboundedSender<EngineMsg>,
) -> Result<(), AnyError> {
    let mut worker = Worker::new(engine_send);
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
