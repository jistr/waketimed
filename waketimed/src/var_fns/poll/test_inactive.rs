use crate::var_fns::{BoolFuture, PollVarFns, VarValueFuture};

use wtd_core::vars::VarValue;

#[derive(Clone, Debug)]
pub struct TestInactiveFns {}

impl TestInactiveFns {
    pub fn new() -> Self {
        Self {}
    }
}

// #[async_trait]
impl PollVarFns for TestInactiveFns {
    fn is_active_fn(&self) -> Box<dyn FnOnce() -> BoolFuture + Send + Sync> {
        Box::new(move || Box::pin(async { false }))
    }

    fn poll_fn(&self) -> Box<dyn FnOnce() -> VarValueFuture + Send + Sync> {
        Box::new(move || Box::pin(async { VarValue::Bool(false) }))
    }
}
