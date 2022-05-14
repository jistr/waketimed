use crate::config::Config;
use crate::files;
use anyhow::Error as AnyError;
use log::{error, trace};
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
                trace!("Stayup rule '{}' is: {}.", &rule_name, value);
                self.stayup_values.insert(rule_name.clone(), value);
            } else {
                error!(
                    "Failed to evaluate stayup rule '{}': '{:?}'",
                    &rule_name, &result
                );
                self.stayup_values.insert(rule_name.clone(), false);
            }
        }
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
    // TODO: add unit tests for stayup rules
    // use crate::test_helpers::{run_and_term_config, with_config};
}