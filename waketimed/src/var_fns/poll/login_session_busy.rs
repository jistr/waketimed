use crate::var_creation_context::VarCreationContext;
use crate::var_fns::PollVarFns;
use anyhow::Error as AnyError;
use log::error;
use serde_yaml::Value;
use std::collections::HashMap;
use tokio::runtime::Handle as TokioHandle;
use wtd_core::vars::VarValue;

#[derive(Clone, Debug)]
pub struct LoginSessionBusyFns {
    system_dbus_conn: zbus::Connection,
}

impl LoginSessionBusyFns {
    pub fn new(
        _params: &HashMap<String, Value>,
        context: &VarCreationContext,
    ) -> Result<Self, AnyError> {
        Ok(Self {
            system_dbus_conn: context.system_dbus_conn()?,
        })
    }
}

impl PollVarFns for LoginSessionBusyFns {
    fn is_active_fn(&self) -> Box<dyn FnOnce() -> bool + Send + Sync> {
        Box::new(move || true)
    }

    fn poll_fn(&self) -> Box<dyn FnOnce(&TokioHandle) -> VarValue + Send + Sync> {
        let system_dbus_conn = self.system_dbus_conn.clone();
        Box::new(move |async_rt| {
            let idle_hint_res = async_rt.block_on(system_dbus_conn.call_method(
                Some("org.freedesktop.login1"),
                "/org/freedesktop/login1/session/auto",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &["org.freedesktop.login1.Session", "IdleHint"],
            ));
            if idle_hint_res.is_err() {
                error!(
                    "Failed to fetch login session IdleHint: {:?}",
                    idle_hint_res
                );
                return VarValue::Bool(true);
            }
            let idle_hint_msg = idle_hint_res.unwrap();
            let idle_hint_var: zvariant::Value = match idle_hint_msg.body() {
                Ok(body) => body,
                Err(e) => {
                    error!("Failed to fetch login session IdleHint: {:?}", e);
                    return VarValue::Bool(true);
                }
            };
            if let zvariant::Value::Bool(idle_hint) = idle_hint_var {
                VarValue::Bool(!idle_hint)
            } else {
                error!(
                    "Failed to fetch login session IdleHint - wrong data type: {:?}",
                    idle_hint_var,
                );
                VarValue::Bool(true)
            }
        })
    }
}
