use crate::messages::EngineMsg;
use crate::var_creation_context::VarCreationContext;
use crate::var_fns::{new_poll_var_fns, PollVarFns};
use anyhow::Context;
use log::{error, trace, warn};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tokio::time::{self, Duration};
use wtd_core::vars::{VarDef, VarName};
use zbus::Connection as ZbusConnection;

pub struct VarWorker {
    engine_send: UnboundedSender<EngineMsg>,

    poll_var_fns: HashMap<VarName, Box<dyn PollVarFns>>,
    poll_var_task: Option<JoinHandle<()>>,
    var_creation_context: VarCreationContext,
}

impl VarWorker {
    pub fn new(
        engine_send: UnboundedSender<EngineMsg>,
        system_dbus_conn: Option<ZbusConnection>,
    ) -> Self {
        Self {
            engine_send,
            poll_var_fns: HashMap::new(),
            poll_var_task: None,
            var_creation_context: VarCreationContext::new(system_dbus_conn),
        }
    }

    pub async fn handle_call_var_poll(&mut self, var_name: VarName) {
        let poll_fn_opt = self.poll_var_fns.get_mut(&var_name);
        let sent = match poll_fn_opt {
            Some(fns) => self
                .engine_send
                .send(EngineMsg::ReturnVarPoll(var_name, fns.poll().await)),
            None => {
                warn!("Cannot poll var '{}' - PollVarFns not loaded.", &var_name);
                self.engine_send
                    .send(EngineMsg::ReturnVarPoll(var_name, None))
            }
        };

        sent.context("Could not send EngineMsg::ReturnVarPoll")
            .unwrap_or_else(|e| error!("{:?}", e));
    }

    pub async fn handle_load_poll_var_fns(&mut self, var_def: VarDef) {
        match new_poll_var_fns(&var_def, &self.var_creation_context) {
            Ok(var_fns) => {
                self.poll_var_fns.insert(var_def.name().clone(), var_fns);
            }
            Err(e) => error!(
                "Failed to create PollVarFns for var '{}': {}",
                var_def.name(),
                e
            ),
        }
    }

    pub async fn handle_spawn_poll_var_interval(&mut self, millis: u64) {
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
