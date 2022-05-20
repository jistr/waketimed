use crate::var_creation_context::VarCreationContext;
use anyhow::{anyhow, Error as AnyError};

use wtd_core::vars::{BuiltinPollDef, VarDef, VarKind, VarValue};

pub mod poll;

pub trait PollVarFns {
    /// Check whether the variable is "relevant" to the host. This is
    /// run when waketimed starts. Active variables are stored in the
    /// runtime variable map and they get polled every poll cycle for
    /// current value. Inactive variables are never polled, and do not
    /// get stored in the runtime variable map.
    // FIXME: The returned function should be async when Rust supports
    // async closures.
    fn is_active_fn(&self) -> Box<dyn FnOnce() -> bool + Send + Sync>;
    /// Poll current value of the variable. Used for updating variable
    /// values in runtime variable map.
    // FIXME: The returned function should be async when Rust supports
    // async closures.
    fn poll_fn(&self) -> Box<dyn FnOnce() -> VarValue + Send + Sync>;
}

pub fn new_poll_var_fns(
    var_def: &VarDef,
    context: &VarCreationContext,
) -> Result<Option<Box<dyn PollVarFns>>, AnyError> {
    let kind = &var_def.kind;
    match kind {
        VarKind::BuiltinPoll(def) => Ok(Some(new_builtin_poll_var_fns(
            var_def.name().as_ref(),
            def,
            context,
        )?)),
        _ => Ok(None),
    }
}

fn new_builtin_poll_var_fns(
    name: &str,
    bp_def: &BuiltinPollDef,
    context: &VarCreationContext,
) -> Result<Box<dyn PollVarFns>, AnyError> {
    match bp_def.builtin_name.as_str() {
        "login_seat_busy" => Ok(Box::new(
            poll::login_seat_busy::LoginSeatBusyFns::new(&bp_def.params, context)?,
        )),
        "test_poll_bool" => Ok(Box::new(poll::test_poll_bool::TestPollBoolFns::new(
            &bp_def.params,
        )?)),
        "test_inactive" => Ok(Box::new(poll::test_inactive::TestInactiveFns::new())),
        _ => Err(anyhow!(
            "Var '{}' definition specified unknown builtin_name: '{}'.",
            name,
            &bp_def.builtin_name
        )),
    }
}
