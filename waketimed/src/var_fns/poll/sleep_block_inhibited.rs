use crate::core::vars::VarValue;
use crate::var_creation_context::VarCreationContext;
use crate::var_fns::PollVarFns;
use anyhow::{anyhow, Error as AnyError};
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use serde_yaml::Value;
use std::collections::HashMap;
use std::sync::Arc;
use zbus::Connection as ZbusConnection;

const SLEEP_INHIBITED_REGEX: &str = r"(^|:)sleep($|:)";

#[derive(Clone, Debug)]
pub struct SleepBlockInhibitedFns {
    system_dbus_conn: ZbusConnection,
}

impl SleepBlockInhibitedFns {
    pub fn new(
        _params: &HashMap<String, Value>,
        context: &VarCreationContext,
    ) -> Result<Self, AnyError> {
        Ok(Self {
            system_dbus_conn: context.system_dbus_conn()?,
        })
    }
}

#[async_trait]
impl PollVarFns for SleepBlockInhibitedFns {
    async fn poll(&mut self) -> Option<VarValue> {
        let system_dbus_conn = self.system_dbus_conn.clone();
        let block_inhibited_res = system_dbus_conn
            .call_method(
                Some("org.freedesktop.login1"),
                "/org/freedesktop/login1",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &["org.freedesktop.login1.Manager", "BlockInhibited"],
            )
            .await;
        process_call_result(block_inhibited_res)
            .map_err(|e| {
                warn!(
                    "Failed to fetch login manager property BlockInhibited: {:?}",
                    e
                )
            })
            .ok()
    }
}

fn process_call_result(
    block_inhibited_res: Result<Arc<zbus::Message>, zbus::Error>,
) -> Result<VarValue, AnyError> {
    let block_inhibited_msg = block_inhibited_res?;
    let body_value: zvariant::Value = block_inhibited_msg.body()?;
    if let zvariant::Value::Str(block_inhibited) = body_value {
        Ok(VarValue::Bool(is_sleep_among_block_inhibited(
            block_inhibited.as_str(),
        )))
    } else {
        Err(anyhow!("Wrong data type."))
    }
}

fn is_sleep_among_block_inhibited(block_inhibited: &str) -> bool {
    lazy_static! {
        static ref RE_SLEEP_INHIBITED: Regex = Regex::new(SLEEP_INHIBITED_REGEX)
            .expect("Failed to compile RuleName regular expression.");
    }
    RE_SLEEP_INHIBITED.is_match(block_inhibited)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sleep_among_block_inhibited() {
        assert!(is_sleep_among_block_inhibited("sleep:shutdown"));
        assert!(is_sleep_among_block_inhibited("idle:sleep"));
        assert!(is_sleep_among_block_inhibited("idle:sleep:shutdown"));
        assert!(!is_sleep_among_block_inhibited("idle:shutdown"));
    }
}
