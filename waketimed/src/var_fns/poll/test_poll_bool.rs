use crate::core::vars::{param_required, VarValue};
use crate::var_fns::PollVarFns;
use anyhow::Error as AnyError;
use async_trait::async_trait;
use serde_yaml::Value;
use std::collections::HashMap;

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

#[async_trait]
impl PollVarFns for TestPollBoolFns {
    async fn poll(&mut self) -> Option<VarValue> {
        let value = self.return_value;
        Some(VarValue::Bool(value))
    }
}
