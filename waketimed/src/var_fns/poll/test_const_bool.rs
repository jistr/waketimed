use crate::var_fns::PollVarFns;
use async_trait::async_trait;
use wtd_core::vars::VarValue;

pub struct TestConstBoolFns {
    return_value: bool,
}

impl TestConstBoolFns {
    pub fn new() -> Self {
        // TODO: allow specifying return value from var params
        Self { return_value: true }
    }
}

#[async_trait]
impl PollVarFns for TestConstBoolFns {
    async fn check_is_active(&self) -> bool {
        true
    }

    async fn poll(&self) -> VarValue {
        VarValue::Bool(self.return_value)
    }
}
