use crate::var_creation_context::VarCreationContext;
use crate::var_fns::PollVarFns;
use anyhow::Error as AnyError;
use async_trait::async_trait;
use log::warn;
use serde_yaml::Value;
use std::collections::HashMap;

use wtd_core::vars::VarValue;
use zbus::Connection as ZbusConnection;

#[derive(Clone, Debug)]
pub struct LoginSeatBusyFns {
    system_dbus_conn: ZbusConnection,
}

impl LoginSeatBusyFns {
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
impl PollVarFns for LoginSeatBusyFns {
    async fn poll(&mut self) -> Option<VarValue> {
        let system_dbus_conn = self.system_dbus_conn.clone();
        // NOTE: It would probably be better to query all seats
        // and check them all rather than hardcode "seat0".
        // Hardcoding seems to work fine for now though.
        let idle_hint_res = system_dbus_conn
            .call_method(
                Some("org.freedesktop.login1"),
                "/org/freedesktop/login1/seat/seat0",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &["org.freedesktop.login1.Seat", "IdleHint"],
            )
            .await;
        if idle_hint_res.is_err() {
            warn!("Failed to fetch login seat IdleHint: {:?}", idle_hint_res);
            return None;
        }
        let idle_hint_msg = idle_hint_res.unwrap();
        let idle_hint_var: zvariant::Value = match idle_hint_msg.body() {
            Ok(body) => body,
            Err(e) => {
                warn!("Failed to fetch login seat IdleHint: {:?}", e);
                return None;
            }
        };
        if let zvariant::Value::Bool(idle_hint) = idle_hint_var {
            Some(VarValue::Bool(!idle_hint))
        } else {
            warn!(
                "Failed to fetch login seat IdleHint - wrong data type: {:?}",
                idle_hint_var,
            );
            None
        }
    }
}
