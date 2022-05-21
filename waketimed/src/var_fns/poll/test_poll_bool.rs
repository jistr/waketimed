use crate::var_fns::{OptVarValueFuture, PollVarFns};
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
    fn poll_fn(&self) -> Box<dyn FnOnce() -> OptVarValueFuture + Send + Sync> {
        let value = self.return_value;
        Box::new(move || Box::pin(async move { Some(VarValue::Bool(value)) }))
    }
}
