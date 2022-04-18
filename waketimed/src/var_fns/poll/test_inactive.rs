use crate::var_fns::PollVarFns;

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

    // async fn poll(&self) -> VarValue {
    //     VarValue::Bool(self.return_value)
    // }
}
