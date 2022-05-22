use crate::config::Config;
use crate::files;
use anyhow::Error as AnyError;
use log::{debug, trace, warn};
use rhai::{Dynamic as RhaiDynamic, Engine as RhaiEngine, Scope as RhaiScope, AST as RhaiAST};
use std::rc::Rc;

use std::collections::HashMap;
use wtd_core::rules::{RuleDef, RuleKind, RuleName};
use wtd_core::vars::{VarName, VarValue};

pub struct RuleManager {
    cfg: Rc<Config>,
    script_engine: RhaiEngine,
    script_scope: RhaiScope<'static>,
    stayup_defs: HashMap<RuleName, RuleDef>,
    stayup_value_asts: HashMap<RuleName, RhaiAST>,
    stayup_values: HashMap<RuleName, bool>,
}

impl RuleManager {
    pub fn new(cfg: Rc<Config>) -> Self {
        Self {
            cfg,
            script_engine: RhaiEngine::new(),
            script_scope: RhaiScope::new(),
            stayup_defs: HashMap::new(),
            stayup_value_asts: HashMap::new(),
            stayup_values: HashMap::new(),
        }
    }

    pub fn init(&mut self) -> Result<(), AnyError> {
        let rule_defs = files::load_rule_defs(&self.cfg)?;
        for (rule_name, rule_def) in rule_defs.into_iter() {
            use RuleKind::*;
            match rule_def.kind {
                StayupBool(_) => {
                    self.stayup_defs.insert(rule_name, rule_def);
                }
            }
        }

        self.compile_stayup_value_asts()?;
        Ok(())
    }

    pub fn reset_script_scope(&mut self, vars: &HashMap<VarName, VarValue>) {
        let mut scope = RhaiScope::new();
        for (var_name, var_value) in vars.iter() {
            use VarValue::*;
            match var_value {
                Bool(v) => {
                    scope.push_constant_dynamic(var_name.as_ref(), RhaiDynamic::from_bool(*v));
                }
            }
        }
        self.script_scope = scope;
    }

    pub fn compute_stayup_values(&mut self) {
        for (rule_name, ast) in self.stayup_value_asts.iter() {
            let result = self
                .script_engine
                .eval_ast_with_scope::<bool>(&mut self.script_scope, ast);
            if let Ok(value) = result {
                Self::set_stayup_value(&mut self.stayup_values, rule_name.clone(), value);
            } else {
                warn!(
                    "Failed to evaluate stayup rule '{}': '{:?}'",
                    &rule_name, &result
                );
                self.stayup_values.remove(rule_name);
            }
        }
    }

    fn set_stayup_value(stayup_values: &mut HashMap<RuleName, bool>, name: RuleName, value: bool) {
        let old_value = stayup_values.get(&name);
        if old_value != Some(&value) {
            debug!("Stayup rule changed: {} = {}", &name, &value);
        }
        stayup_values.insert(name, value);
    }

    fn compile_stayup_value_asts(&mut self) -> Result<(), AnyError> {
        for (rule_name, rule_def) in self.stayup_defs.iter() {
            trace!("Compiling value script AST for rule '{}'.", &rule_name);
            use RuleKind::*;
            let ast = match &rule_def.kind {
                StayupBool(def) => self.script_engine.compile(&def.value_script)?,
            };
            self.stayup_value_asts.insert(rule_name.clone(), ast);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{rule_name, run_and_term_config, var_name};

    fn create_rule_manager(cfg: Config) -> RuleManager {
        RuleManager::new(Rc::new(cfg))
    }

    #[test]
    fn test_stayup_rules() {
        let mut mgr = create_rule_manager(run_and_term_config());
        mgr.init().expect("Failed to init RuleManager.");

        let mut vars: HashMap<VarName, VarValue> = HashMap::new();
        vars.insert(var_name("test_category"), VarValue::Bool(true));
        vars.insert(var_name("test_poll_true"), VarValue::Bool(true));

        mgr.reset_script_scope(&vars);
        mgr.compute_stayup_values();
        assert_eq!(
            mgr.stayup_values.get(&rule_name("test_stayup_bool")),
            Some(&true)
        );

        vars.insert(var_name("test_category"), VarValue::Bool(false));
        mgr.reset_script_scope(&vars);
        mgr.compute_stayup_values();
        assert_eq!(
            mgr.stayup_values.get(&rule_name("test_stayup_bool")),
            Some(&false)
        );
    }
}
