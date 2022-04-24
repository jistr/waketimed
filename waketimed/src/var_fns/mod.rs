use anyhow::{anyhow, Error as AnyError};
use wtd_core::vars::{BuiltinPollDef, VarDef, VarKind};

pub mod poll;

// #[async_trait]
pub trait PollVarFns {
    /// Check whether the variable is "relevant" to the host. This is
    /// run when waketimed starts. Active variables are stored in the
    /// runtime variable map and they get polled every poll cycle for
    /// current value. Inactive variables are never polled, and do not
    /// get stored in the runtime variable map.
    // TODO: Can/should the returned function be async (return Future)?
    fn is_active_fn(&self) -> Box<dyn FnOnce() -> bool + Send + Sync>;
    // Poll current value of the variable. Used for updating variable
    // values in runtime variable map.
    //  async fn poll(&self) -> VarValue;
}

pub fn new_poll_var_fns(var_def: &VarDef) -> Result<Option<Box<dyn PollVarFns>>, AnyError> {
    let kind = &var_def.kind;
    match kind {
        VarKind::BuiltinPoll(def) => Ok(Some(new_builtin_poll_var_fns(
            var_def.name().as_ref(),
            def,
        )?)),
    }
}

fn new_builtin_poll_var_fns(
    name: &str,
    bp_def: &BuiltinPollDef,
) -> Result<Box<dyn PollVarFns>, AnyError> {
    match bp_def.builtin_name.as_str() {
        "test_poll_bool" => Ok(Box::new(poll::test_poll_bool::TestPollBoolFns::new())),
        "test_inactive" => Ok(Box::new(poll::test_inactive::TestInactiveFns::new())),
        _ => Err(anyhow!(
            "Var '{}' definition specified unknown builtin_name: '{}'.",
            name,
            &bp_def.builtin_name
        )),
    }
}
