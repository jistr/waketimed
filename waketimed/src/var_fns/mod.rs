use anyhow::{anyhow, Error as AnyError};
use async_trait::async_trait;
use wtd_core::model::{BuiltinPollDef, VarDef, VarKind};
use wtd_core::VarValue;

pub mod poll;

#[async_trait]
pub trait PollVarFns {
    /// Check whether the variable is "relevant" to the host. This is
    /// run when waketimed starts. Active variables are stored in the
    /// runtime variable map and they get polled every poll cycle for
    /// current value. Inactive variables are never polled, and do not
    /// get stored in the runtime variable map.
    async fn check_is_active(&self) -> bool;
    /// Poll current value of the variable. Used for updating variable
    /// values in runtime variable map.
    async fn poll(&self) -> VarValue;
}

pub fn new_poll_var_fns(var_def: &VarDef) -> Result<Option<impl PollVarFns>, AnyError> {
    let kind = &var_def.kind;
    match kind {
        VarKind::BuiltinPoll(def) => {
            Ok(Some(new_builtin_poll_var_fns(var_def.name.as_ref(), def)?))
        }
    }
}

fn new_builtin_poll_var_fns(
    name: &str,
    bp_def: &BuiltinPollDef,
) -> Result<impl PollVarFns, AnyError> {
    match bp_def.builtin_name.as_str() {
        "test_const_bool" => Ok(poll::test_const_bool::TestConstBoolFns::new()),
        _ => Err(anyhow!(
            "Var '{}' definition specified unknown builtin_name: '{}'.",
            name,
            &bp_def.builtin_name
        )),
    }
}
