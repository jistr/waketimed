use crate::config::Config;
use crate::core::vars::{VarDef, VarKind, VarName, VarValue};
use crate::files;
use crate::messages::WorkerMsg;
use anyhow::{anyhow, Error as AnyError};
use getset::Getters;
use log::{debug, error, trace};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Getters)]
pub struct VarManager {
    cfg: Rc<Config>,
    worker_send: UnboundedSender<WorkerMsg>,
    #[getset(get = "pub")]
    vars: HashMap<VarName, VarValue>,
    var_defs: HashMap<VarName, VarDef>,
    category_vars: HashMap<VarName, HashSet<VarName>>,
    waitlist_poll: HashSet<VarName>,
}

impl VarManager {
    pub fn new(cfg: Rc<Config>, worker_send: UnboundedSender<WorkerMsg>) -> Result<Self, AnyError> {
        Ok(Self {
            cfg,
            worker_send,
            vars: HashMap::new(),
            var_defs: HashMap::new(),
            category_vars: HashMap::new(),
            waitlist_poll: HashSet::new(),
        })
    }

    pub fn init(&mut self) -> Result<(), AnyError> {
        self.load_var_defs()?;
        self.load_poll_var_fns()?;
        Ok(())
    }

    pub fn waitlist_poll_is_empty(&self) -> bool {
        self.waitlist_poll.is_empty()
    }

    pub fn poll_vars(&mut self) -> Result<(), AnyError> {
        self.waitlist_poll = HashSet::new();
        for (var_name, var_def) in self.var_defs.iter() {
            if matches!(var_def.kind, VarKind::BuiltinPoll(_)) {
                self.waitlist_poll.insert(var_name.clone());
                self.worker_send
                    .send(WorkerMsg::CallVarPoll(var_name.clone()))?;
            }
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
                    Self::set_var(&mut self.vars, var_name.clone(), VarValue::Bool(value));
                }
                _ => {}
            }
        }
    }

    pub fn spawn_poll_var_interval(&mut self) -> Result<(), AnyError> {
        let interval = self.cfg.poll_variable_interval;
        self.worker_send
            .send(WorkerMsg::SpawnPollVarInterval(interval))?;
        Ok(())
    }

    pub fn handle_return_var_poll(&mut self, var_name: VarName, opt_value: Option<VarValue>) {
        self.waitlist_poll.remove(&var_name);
        if let Some(value) = opt_value {
            Self::set_var(&mut self.vars, var_name, value);
        }
    }

    fn load_var_defs(&mut self) -> Result<(), AnyError> {
        self.var_defs = files::load_var_defs(&self.cfg)?;
        self.category_vars = self.compute_category_vars_map();
        Ok(())
    }

    fn load_poll_var_fns(&mut self) -> Result<(), AnyError> {
        for var_def in self.var_defs.values() {
            if matches!(var_def.kind, VarKind::BuiltinPoll(_)) {
                self.worker_send
                    .send(WorkerMsg::LoadPollVarFns(var_def.clone()))?;
            }
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

    fn set_var(vars: &mut HashMap<VarName, VarValue>, name: VarName, value: VarValue) {
        let old_value = vars.get(&name);
        if old_value != Some(&value) {
            debug!("Variable changed: {} = {}", &name, &value);
        }
        vars.insert(name, value);
    }

    fn is_any_bool_var_true(&self, var_names: &HashSet<VarName>) -> Result<bool, AnyError> {
        let var_bools: Result<Vec<bool>, AnyError> = var_names
            .iter()
            .map(|v| {
                #[allow(irrefutable_let_patterns)]
                if let VarValue::Bool(b) = self.get_cloned_or(v, VarValue::Bool(false)) {
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
    use crate::test_helpers::{run_and_term_without_builtin_defs_config, var_name};
    use tokio::sync::mpsc::UnboundedReceiver;

    fn create_var_manager(cfg: Config) -> (VarManager, UnboundedReceiver<WorkerMsg>) {
        let (worker_send, worker_recv) = tokio::sync::mpsc::unbounded_channel();
        let mgr = VarManager::new(Rc::new(cfg), worker_send).expect("Failed to create VarManager.");
        (mgr, worker_recv)
    }

    #[test]
    fn test_category_vars() {
        let (mut mgr, _worker_recv) =
            create_var_manager(run_and_term_without_builtin_defs_config());
        mgr.init().expect("Failed to init VarManager.");
        let test_category = var_name("test_category");

        // Test that category_vars map was computed correctly.
        assert_eq!(mgr.category_vars.len(), 1, "{:?}", mgr.category_vars);
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
    }
}
