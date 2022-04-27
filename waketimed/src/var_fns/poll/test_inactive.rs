use crate::var_fns::PollVarFns;
use wtd_core::vars::VarValue;

#[derive(Clone, Debug, PartialEq)]
pub struct TestInactiveFns {}

impl TestInactiveFns {
    pub fn new() -> Self {
        Self {}
    }
}

// #[async_trait]
impl PollVarFns for TestInactiveFns {
    fn is_active_fn(&self) -> Box<dyn FnOnce() -> bool + Send + Sync> {
        Box::new(move || false)
    }

    fn poll_fn(&self) -> Box<dyn FnOnce() -> VarValue + Send + Sync> {
        Box::new(move || VarValue::Bool(false))
    }
}
