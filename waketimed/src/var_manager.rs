use crate::files;
use crate::messages::WorkerMsg;
use crate::var_fns::{new_poll_var_fns, PollVarFns};
use anyhow::Error as AnyError;

use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use wtd_core::model::VarDef;
use wtd_core::VarName;

pub struct VarManager {
    #[allow(dead_code)]
    worker_send: UnboundedSender<WorkerMsg>,
    // vars: HashMap<VarName, VarValue>,
    poll_var_fns: HashMap<VarName, Box<dyn PollVarFns>>,
    var_defs: HashMap<VarName, VarDef>,
}

impl VarManager {
    pub fn new(worker_send: UnboundedSender<WorkerMsg>) -> Self {
        Self {
            worker_send,
            // vars: HashMap::new(),
            poll_var_fns: HashMap::new(),
            var_defs: HashMap::new(),
        }
    }

    pub fn init(&mut self) -> Result<(), AnyError> {
        self.load_var_defs()?;
        Ok(())
    }

    fn load_var_defs(&mut self) -> Result<(), AnyError> {
        self.var_defs = files::load_var_defs()?;
        self.load_poll_var_fns()?;
        Ok(())
    }

    fn load_poll_var_fns(&mut self) -> Result<(), AnyError> {
        self.poll_var_fns = HashMap::new();
        for var_def in self.var_defs.values() {
            if let Some(var_fns) = new_poll_var_fns(var_def)? {
                self.poll_var_fns
                    .insert(var_def.name.clone(), Box::new(var_fns));
            }
        }
        Ok(())
    }
}
