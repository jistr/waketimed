use crate::messages::{DbusMsg, EngineMsg, WorkerMsg};
use crate::var_manager::VarManager;
use anyhow::Error as AnyError;
use log::{debug, warn};
use tokio::sync::mpsc::UnboundedSender;
use wtd_core::vars::VarName;

pub struct Engine {
    dbus_send: UnboundedSender<DbusMsg>,
    worker_send: UnboundedSender<WorkerMsg>,

    state: EngineState,
    var_manager: VarManager,
}

impl Engine {
    pub fn new(
        dbus_send: UnboundedSender<DbusMsg>,
        worker_send: UnboundedSender<WorkerMsg>,
    ) -> Self {
        let var_manager = VarManager::new(worker_send.clone());
        Self {
            dbus_send,
            worker_send,
            state: EngineState::Initializing,
            var_manager,
        }
    }

    pub fn init(&mut self) -> Result<(), AnyError> {
        self.set_state(EngineState::Initializing);
        self.var_manager.init()?;
        Ok(())
    }

    pub fn handle_msg(&mut self, msg: EngineMsg) {
        match self.state {
            EngineState::Initializing => match msg {
                EngineMsg::ReturnVarIsActive(var_name, is_active) => {
                    self.handle_return_var_is_active(var_name, is_active)
                }
                EngineMsg::Terminate => {
                    warn!("Received Terminate while still in Initializing state. Terminating.");
                    self.handle_terminate();
                }
                #[allow(unreachable_patterns)]
                _ => {
                    warn!(
                        "Engine state is Initializing, ignoring incoming message: '{:?}'",
                        msg
                    );
                }
            },
            EngineState::Running => match msg {
                EngineMsg::Terminate => {
                    self.handle_terminate();
                }
                _ => {
                    warn!(
                        "Engine state is Running, ignoring incoming message: '{:?}'",
                        msg
                    );
                }
            },
            EngineState::Terminating => {
                warn!(
                    "Engine state is Terminating, ignoring incoming message: '{:?}'",
                    msg
                );
            }
        }
    }

    fn handle_terminate(&mut self) {
        self.set_state(EngineState::Terminating);
        self.dbus_send
            .send(DbusMsg::Terminate)
            .expect("Failed to send DbusMsg::Terminate");
        self.worker_send
            .send(WorkerMsg::Terminate)
            .expect("Failed to send WorkerMsg::Terminate");
    }

    fn handle_return_var_is_active(&mut self, var_name: VarName, is_active: bool) {
        self.var_manager
            .handle_return_var_is_active(&var_name, is_active);
        self.set_state_running_maybe();
    }

    fn set_state_running_maybe(&mut self) {
        if self.var_manager.waitlist_active_is_empty() {
            self.set_state(EngineState::Running);
        }
    }

    fn set_state(&mut self, state: EngineState) {
        debug!("Engine entering state '{:?}'.", state);
        self.state = state;
    }
}

#[derive(Debug)]
enum EngineState {
    Initializing,
    Running,
    Terminating,
}
