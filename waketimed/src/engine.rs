use crate::messages::{DbusMsg, EngineMsg, WorkerMsg};
use crate::var_manager::VarManager;
use anyhow::Error as AnyError;
use tokio::sync::mpsc::UnboundedSender;

pub struct Engine {
    dbus_send: UnboundedSender<DbusMsg>,
    worker_send: UnboundedSender<WorkerMsg>,

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
            var_manager,
        }
    }

    pub fn init(&mut self) -> Result<(), AnyError> {
        self.var_manager.init()?;
        Ok(())
    }

    pub fn handle_msg(&mut self, msg: EngineMsg) {
        match msg {
            EngineMsg::Terminate => self.handle_terminate(),
        }
    }

    fn handle_terminate(&mut self) {
        self.dbus_send
            .send(DbusMsg::Terminate)
            .expect("Failed to send DbusMsg::Terminate");
        self.worker_send
            .send(WorkerMsg::Terminate)
            .expect("Failed to send WorkerMsg::Terminate");
    }
}
