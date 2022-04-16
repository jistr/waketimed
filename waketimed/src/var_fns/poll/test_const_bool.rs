use crate::var_fns::PollVarFns;

#[derive(Clone, Debug, PartialEq)]
pub struct TestConstBoolFns {
    return_value: bool,
}

impl TestConstBoolFns {
    pub fn new() -> Self {
        // TODO: allow specifying return value from var params
        Self { return_value: true }
    }
}

// #[async_trait]
impl PollVarFns for TestConstBoolFns {
    fn is_active_fn(&self) -> Box<dyn FnOnce() -> bool + Send + Sync> {
        Box::new(move || true)
    }

    // async fn poll(&self) -> VarValue {
    //     VarValue::Bool(self.return_value)
    // }
}
