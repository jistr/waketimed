use crate::var_fns::{BoolFuture, PollVarFns, VarValueFuture};
use anyhow::Error as AnyError;
use serde_yaml::Value;
use std::collections::HashMap;

use wtd_core::vars::{param_required, VarValue};

#[derive(Clone, Debug)]
pub struct TestPollBoolFns {
    return_value: bool,
}

impl TestPollBoolFns {
    pub fn new(params: &HashMap<String, Value>) -> Result<Self, AnyError> {
        let return_value = param_required::<bool>(params, "return_value")?;
        Ok(Self { return_value })
    }
}

impl PollVarFns for TestPollBoolFns {
    fn is_active_fn(&self) -> Box<dyn FnOnce() -> BoolFuture + Send + Sync> {
        Box::new(move || Box::pin(async { true }))
    }

    fn poll_fn(&self) -> Box<dyn FnOnce() -> VarValueFuture + Send + Sync> {
        let value = self.return_value;
        Box::new(move || Box::pin(async move { VarValue::Bool(value) }))
    }
}
