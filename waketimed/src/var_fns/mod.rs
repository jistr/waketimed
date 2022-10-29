use crate::core::vars::{BuiltinPollDef, VarDef, VarKind, VarValue};
use crate::var_creation_context::VarCreationContext;
use anyhow::{anyhow, Error as AnyError};
use async_trait::async_trait;

pub mod poll;

#[async_trait]
pub trait PollVarFns {
    /// Poll current value of the variable. Used for updating variable
    /// values in runtime variable map.
    async fn poll(&mut self) -> Option<VarValue>;
}

pub fn new_poll_var_fns(
    var_def: &VarDef,
    context: &VarCreationContext,
) -> Result<Box<dyn PollVarFns>, AnyError> {
    let kind = &var_def.kind;
    match kind {
        VarKind::BuiltinPoll(def) => {
            new_builtin_poll_var_fns(var_def.name().as_ref(), def, context)
        }
        _ => Err(anyhow!(
            "Can't get PollVarFns for non-poll var '{}'.",
            var_def.name()
        )),
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
        "sleep_block_inhibited" => Ok(Box::new(
            poll::sleep_block_inhibited::SleepBlockInhibitedFns::new(&bp_def.params, context)?,
        )),
        "modem_voice_call_present" => Ok(Box::new(
            poll::modem_voice_call_present::ModemVoiceCallPresentFns::new(&bp_def.params, context)?,
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
