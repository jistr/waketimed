use crate::var_fns::PollVarFns;
use anyhow::Error as AnyError;
use serde_yaml::Value;
use std::collections::HashMap;
use tokio::runtime::Handle as TokioHandle;
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

// #[async_trait]
impl PollVarFns for TestPollBoolFns {
    fn is_active_fn(&self) -> Box<dyn FnOnce() -> bool + Send + Sync> {
        Box::new(move || true)
    }

    fn poll_fn(&self) -> Box<dyn FnOnce(&TokioHandle) -> VarValue + Send + Sync> {
        let value = self.return_value;
        Box::new(move |_| VarValue::Bool(value))
    }
}
