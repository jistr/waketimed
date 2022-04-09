use crate::files;
use crate::messages::{DbusMsg, EngineMsg, WorkerMsg};
use anyhow::Error as AnyError;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use wtd_core::model::VarDef;
use wtd_core::VarName;

pub struct Engine {
    dbus_send: UnboundedSender<DbusMsg>,
    worker_send: UnboundedSender<WorkerMsg>,

    // vars: HashMap<VarName, VarValue>,
    var_defs: HashMap<VarName, VarDef>,
}

impl Engine {
    pub fn new(
        dbus_send: UnboundedSender<DbusMsg>,
        worker_send: UnboundedSender<WorkerMsg>,
    ) -> Self {
        Self {
            dbus_send,
            worker_send,

            // vars: HashMap::new(),
            var_defs: HashMap::new(),
        }
    }

    pub fn init(&mut self) -> Result<(), AnyError> {
        self.load_var_defs()?;
        Ok(())
    }

    pub fn handle_msg(&mut self, msg: EngineMsg) {
        match msg {
            EngineMsg::Terminate => self.handle_terminate(),
        }
    }

    fn load_var_defs(&mut self) -> Result<(), AnyError> {
        self.var_defs = files::load_var_defs()?;
        Ok(())
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
