use crate::var_fns::{OptVarValueFuture, PollVarFns};

#[derive(Clone, Debug)]
pub struct TestInactiveFns {}

impl TestInactiveFns {
    pub fn new() -> Self {
        Self {}
    }
}

impl PollVarFns for TestInactiveFns {
    fn poll_fn(&self) -> Box<dyn FnOnce() -> OptVarValueFuture + Send + Sync> {
        Box::new(move || Box::pin(async { None }))
    }
}
