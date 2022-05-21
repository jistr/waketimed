use crate::var_creation_context::VarCreationContext;
use anyhow::{anyhow, Error as AnyError};
use std::future::Future;
use std::pin::Pin;

use wtd_core::vars::{BuiltinPollDef, VarDef, VarKind, VarValue};

pub mod poll;

pub type OptVarValueFuture = Pin<Box<dyn Future<Output = Option<VarValue>>>>;

pub trait PollVarFns {
    /// Poll current value of the variable. Used for updating variable
    /// values in runtime variable map.
    fn poll_fn(&self) -> Box<dyn FnOnce() -> OptVarValueFuture + Send + Sync>;
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
        "login_seat_busy" => Ok(Box::new(poll::login_seat_busy::LoginSeatBusyFns::new(
            &bp_def.params,
            context,
        )?)),
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
