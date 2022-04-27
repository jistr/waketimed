use crate::var_fns::PollVarFns;
use wtd_core::vars::VarValue;

#[derive(Clone, Debug, PartialEq)]
pub struct TestPollBoolFns {
    return_value: bool,
}

impl TestPollBoolFns {
    pub fn new() -> Self {
        // TODO: allow specifying return value from var params
        Self { return_value: true }
    }
}

// #[async_trait]
impl PollVarFns for TestPollBoolFns {
    fn is_active_fn(&self) -> Box<dyn FnOnce() -> bool + Send + Sync> {
        Box::new(move || true)
    }

    fn poll_fn(&self) -> Box<dyn FnOnce() -> VarValue + Send + Sync> {
        let value = self.return_value;
        Box::new(move || VarValue::Bool(value))
    }
}
