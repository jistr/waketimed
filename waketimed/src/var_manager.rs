use crate::files;
use crate::get_config;
use crate::messages::WorkerMsg;
use crate::var_fns::{new_poll_var_fns, PollVarFns};
use anyhow::{anyhow, Error as AnyError};
use log::{debug, error, trace};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;
use wtd_core::vars::{VarDef, VarKind, VarName, VarValue};

pub struct VarManager {
    worker_send: UnboundedSender<WorkerMsg>,
    vars: HashMap<VarName, VarValue>,
    poll_var_fns: HashMap<VarName, Box<dyn PollVarFns>>,
    var_defs: HashMap<VarName, VarDef>,
    category_vars: HashMap<VarName, HashSet<VarName>>,
    waitlist_active: HashSet<VarName>,
    waitlist_poll: HashSet<VarName>,
}

impl VarManager {
    pub fn new(worker_send: UnboundedSender<WorkerMsg>) -> Self {
        Self {
            worker_send,
            vars: HashMap::new(),
            poll_var_fns: HashMap::new(),
            var_defs: HashMap::new(),
            category_vars: HashMap::new(),
            waitlist_active: HashSet::new(),
            waitlist_poll: HashSet::new(),
        }
    }

    pub fn init(&mut self) -> Result<(), AnyError> {
        self.load_var_defs()?;
        self.load_poll_var_fns()?;
        self.forget_inactive_poll_vars()?;
        Ok(())
    }

    /// NOTE: is_initialized can return false later in the life time of
    /// VarManager, but it only makes sense to check is_initialized
    /// when Engine is trying to enter Running state, and there it will
    /// behave correctly.
    pub fn is_initialized(&self) -> bool {
        self.waitlist_active.is_empty() && self.waitlist_poll.is_empty()
    }

    pub fn waitlist_poll_is_empty(&self) -> bool {
        self.waitlist_poll.is_empty()
    }

    pub fn poll_vars(&mut self) -> Result<(), AnyError> {
        self.waitlist_poll = HashSet::with_capacity(self.poll_var_fns.len());
        for (var_name, var_fns) in self.poll_var_fns.iter() {
            self.waitlist_poll.insert(var_name.clone());
            self.worker_send
                .send(WorkerMsg::CallVarPoll(var_name.clone(), var_fns.poll_fn()))?;
        }
        Ok(())
    }

    pub fn update_category_vars(&mut self) {
        for (var_name, var_def) in self.var_defs.iter() {
            #[allow(clippy::single_match)]
            match &var_def.kind {
                VarKind::CategoryAny(def) => {
                    let result = self.is_any_bool_var_true(
                        self.category_vars
                            .get(&def.category_name)
                            .expect("List of category vars not populated"),
                    );
                    let value = result.unwrap_or_else(|e| {
                        error!(
                            "Could not compute CategoryAny variable '{}': {:#}",
                            var_name, e
                        );
                        false
                    });
                    trace!("CategoryAny var '{}' is: {:?}", var_name, &value);
                    self.vars.insert(var_name.clone(), VarValue::Bool(value));
                }
                _ => {}
            }
        }
    }

    pub fn spawn_poll_var_interval(&mut self) -> Result<(), AnyError> {
        let interval = get_config().borrow().poll_variable_interval;
        self.worker_send
            .send(WorkerMsg::SpawnPollVarInterval(interval))?;
        Ok(())
    }

    pub fn handle_return_var_is_active(&mut self, var_name: VarName, is_active: bool) {
        if is_active {
            debug!("Var '{}' is active.", var_name.as_ref());
        } else {
            debug!("Var '{}' is inactive, forgetting it.", var_name.as_ref());
            self.poll_var_fns.remove(&var_name);
            self.var_defs.remove(&var_name);
        }
        self.waitlist_active.remove(&var_name);
        // If the last variable has been checked for activity, intitialize variables.
        if self.waitlist_active.is_empty() {
            self.poll_vars().expect("Unable to poll vars");
        }
    }

    pub fn handle_return_var_poll(&mut self, var_name: VarName, value: VarValue) {
        self.waitlist_poll.remove(&var_name);
        self.vars.insert(var_name, value);
    }

    fn load_var_defs(&mut self) -> Result<(), AnyError> {
        self.var_defs = files::load_var_defs()?;
        self.category_vars = self.compute_category_vars_map();
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

    fn compute_category_vars_map(&self) -> HashMap<VarName, HashSet<VarName>> {
        let mut category_vars = HashMap::new();
        for var_def in self.var_defs.values() {
            // Populate from var's 'categories'.
            for category in var_def.categories.iter() {
                if !category_vars.contains_key(category) {
                    category_vars.insert(category.clone(), HashSet::new());
                }
                category_vars
                    .get_mut(category)
                    .expect("Var category not found in map")
                    .insert(var_def.name().clone());
            }
            // Make sure Category type var's 'category_name' is at least defined.
            if let VarKind::CategoryAny(def) = &var_def.kind {
                if !category_vars.contains_key(&def.category_name) {
                    category_vars.insert(def.category_name.clone(), HashSet::new());
                }
            }
        }
        trace!("Category vars: {:?}", category_vars);
        category_vars
    }

    fn is_any_bool_var_true(&self, var_names: &HashSet<VarName>) -> Result<bool, AnyError> {
        let var_bools: Result<Vec<bool>, AnyError> = var_names
            .iter()
            .map(|v| {
                #[allow(irrefutable_let_patterns)]
                if let VarValue::Bool(b) = self.get_cloned_or(v, VarValue::Bool(false)) {
                    trace!("Var '{}' is {}", v, b);
                    Ok(b)
                } else {
                    Err(anyhow!(
                        "Variable '{}' is not bool, cannot be processed by is_any_bool_var_true.",
                        v
                    ))
                }
            })
            .collect();
        var_bools.map(|bool_vec| bool_vec.iter().any(|b2| *b2))
    }

    fn get_cloned_or(&self, var_name: &VarName, default: VarValue) -> VarValue {
        self.vars.get(var_name).cloned().unwrap_or(default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{run_and_term_config, var_name, with_config};
    use tokio::sync::mpsc::UnboundedReceiver;

    fn create_var_manager() -> (VarManager, UnboundedReceiver<WorkerMsg>) {
        let (worker_send, worker_recv) = tokio::sync::mpsc::unbounded_channel();
        let mgr = VarManager::new(worker_send);
        (mgr, worker_recv)
    }

    #[test]
    fn test_category_vars() {
        with_config(run_and_term_config(), || {
            let (mut mgr, _worker_recv) = create_var_manager();
            mgr.init().expect("Failed to init VarManager.");
            let test_category = var_name("test_category");

            // Test that category_vars map was computed correctly.
            assert_eq!(mgr.category_vars.len(), 1);
            let test_category_vars = mgr
                .category_vars
                .get(&test_category)
                .expect("Did not find test_category in category_vars.");
            assert_eq!(test_category_vars.len(), 1);
            assert_eq!(
                test_category_vars,
                &HashSet::from([var_name("test_poll_true"),])
            );

            // Test that category vars get updated correctly.
            mgr.update_category_vars();
            assert_eq!(
                mgr.vars
                    .get(&test_category)
                    .expect("Var test_category not found."),
                &VarValue::Bool(false)
            );
            mgr.vars
                .insert(var_name("test_poll_true"), VarValue::Bool(true));
            mgr.update_category_vars();
            assert_eq!(
                mgr.vars
                    .get(&test_category)
                    .expect("Var test_category not found."),
                &VarValue::Bool(true)
            );
            mgr.vars
                .insert(var_name("test_poll_true"), VarValue::Bool(false));
            mgr.update_category_vars();
            assert_eq!(
                mgr.vars
                    .get(&test_category)
                    .expect("Var test_category not found."),
                &VarValue::Bool(false)
            );
        });
    }
}
