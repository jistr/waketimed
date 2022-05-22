use crate::var_fns::PollVarFns;
use async_trait::async_trait;
use wtd_core::vars::VarValue;

#[derive(Clone, Debug)]
pub struct TestInactiveFns {}

impl TestInactiveFns {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl PollVarFns for TestInactiveFns {
    async fn poll(&mut self) -> Option<VarValue> {
        None
    }
}
