use crate::files;
use crate::get_config;
use crate::messages::WorkerMsg;
use crate::var_fns::{new_poll_var_fns, PollVarFns};
use anyhow::Error as AnyError;
use log::debug;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;
use wtd_core::vars::{VarDef, VarName};

pub struct VarManager {
    worker_send: UnboundedSender<WorkerMsg>,
    // vars: HashMap<VarName, VarValue>,
    poll_var_fns: HashMap<VarName, Box<dyn PollVarFns>>,
    var_defs: HashMap<VarName, VarDef>,
    waitlist_active: HashSet<VarName>,
}

impl VarManager {
    pub fn new(worker_send: UnboundedSender<WorkerMsg>) -> Self {
        Self {
            worker_send,
            // vars: HashMap::new(),
            poll_var_fns: HashMap::new(),
            var_defs: HashMap::new(),
            waitlist_active: HashSet::new(),
        }
    }

    pub fn init(&mut self) -> Result<(), AnyError> {
        self.load_var_defs()?;
        self.load_poll_var_fns()?;
        self.forget_inactive_poll_vars()?;
        Ok(())
    }

    pub fn poll_vars(&mut self) -> Result<(), AnyError> {
        // TODO: implement
        Ok(())
    }

    pub fn spawn_poll_var_interval(&mut self) -> Result<(), AnyError> {
        let interval = get_config().borrow().poll_variable_interval;
        self.worker_send
            .send(WorkerMsg::SpawnPollVarInterval(interval))?;
        Ok(())
    }

    pub fn handle_return_var_is_active(&mut self, var_name: &VarName, is_active: bool) {
        if is_active {
            debug!("Var '{}' is active.", var_name.as_ref());
        } else {
            debug!("Var '{}' is inactive, forgetting it.", var_name.as_ref());
            self.poll_var_fns.remove(var_name);
            self.var_defs.remove(var_name);
        }
        self.waitlist_active.remove(var_name);
    }

    pub fn waitlist_active_is_empty(&self) -> bool {
        self.waitlist_active.is_empty()
    }

    fn load_var_defs(&mut self) -> Result<(), AnyError> {
        self.var_defs = files::load_var_defs()?;
        Ok(())
    }

    fn load_poll_var_fns(&mut self) -> Result<(), AnyError> {
        self.poll_var_fns = HashMap::new();
        for var_def in self.var_defs.values() {
            if let Some(var_fns) = new_poll_var_fns(var_def)? {
                self.poll_var_fns.insert(var_def.name().clone(), var_fns);
            }
        }
        Ok(())
    }

    fn forget_inactive_poll_vars(&mut self) -> Result<(), AnyError> {
        self.waitlist_active = HashSet::with_capacity(self.poll_var_fns.len());
        for (var_name, var_fns) in self.poll_var_fns.iter() {
            self.waitlist_active.insert(var_name.clone());
            self.worker_send.send(WorkerMsg::CallVarIsActive(
                var_name.clone(),
                var_fns.is_active_fn(),
            ))?;
        }
        Ok(())
    }
}
