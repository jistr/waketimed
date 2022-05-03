use crate::messages::{DbusMsg, EngineMsg, WorkerMsg};
use crate::var_manager::VarManager;
use anyhow::{Context, Error as AnyError};
use log::{debug, error, trace, warn};
use tokio::sync::mpsc::UnboundedSender;
use wtd_core::vars::{VarName, VarValue};

pub struct Engine {
    dbus_send: UnboundedSender<DbusMsg>,
    engine_send: UnboundedSender<EngineMsg>,
    worker_send: UnboundedSender<WorkerMsg>,

    state: EngineState,
    var_manager: VarManager,
}

impl Engine {
    pub fn new(
        dbus_send: UnboundedSender<DbusMsg>,
        engine_send: UnboundedSender<EngineMsg>,
        worker_send: UnboundedSender<WorkerMsg>,
    ) -> Self {
        let var_manager = VarManager::new(worker_send.clone());
        Self {
            dbus_send,
            engine_send,
            worker_send,
            state: EngineState::Initializing,
            var_manager,
        }
    }

    pub fn init(&mut self) -> Result<(), AnyError> {
        self.set_state(EngineState::Initializing);
        self.var_manager.init()?;
        self.set_state_running_maybe();
        Ok(())
    }

    pub fn handle_msg(&mut self, msg: EngineMsg) {
        trace!("Received EngineMsg::{:?}.", &msg);
        match self.state {
            EngineState::Initializing => match msg {
                EngineMsg::ReturnVarIsActive(var_name, is_active) => {
                    self.handle_return_var_is_active(var_name, is_active)
                }
                EngineMsg::ReturnVarPoll(var_name, value) => {
                    self.handle_return_var_poll(var_name, value)
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
                EngineMsg::PollVarsTick => self.handle_poll_vars_tick(),
                EngineMsg::ReturnVarPoll(var_name, value) => {
                    self.handle_return_var_poll(var_name, value)
                }
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

    fn handle_poll_vars_tick(&mut self) {
        let result = self.var_manager.poll_vars().context("Failed to poll vars.");
        self.term_on_err(result);
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
            .handle_return_var_is_active(var_name, is_active);
        self.set_state_running_maybe();
    }

    fn handle_return_var_poll(&mut self, var_name: VarName, value: VarValue) {
        self.var_manager.handle_return_var_poll(var_name, value);
        if self.var_manager.waitlist_poll_is_empty() {
            self.var_manager.update_category_vars();
        }
        self.set_state_running_maybe();
    }

    fn handle_state_transition(&mut self, _old_state: EngineState, new_state: EngineState) {
        #[allow(clippy::single_match)]
        match new_state {
            EngineState::Running => {
                let res = self
                    .var_manager
                    .spawn_poll_var_interval()
                    .context("Fatal: Failed to set up variable poll interval.");
                self.term_on_err(res);
            }
            _ => {}
        }
    }

    fn set_state_running_maybe(&mut self) {
        if self.var_manager.is_initialized() {
            self.set_state(EngineState::Running);
        }
    }

    fn set_state(&mut self, state: EngineState) {
        debug!("Engine entering state '{:?}'.", state);
        let old_state = self.state;
        self.state = state;
        self.handle_state_transition(old_state, state);
    }

    fn term_on_err<T>(&mut self, result: Result<T, AnyError>) -> Option<T> {
        match result {
            Ok(val) => Some(val),
            Err(e) => {
                error!("{:#}", e);
                self.engine_send
                    .send(EngineMsg::Terminate)
                    .expect("Failed to send Terminate message.");
                None
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum EngineState {
    Initializing,
    Running,
    Terminating,
}
