use crate::core::vars::VarValue;
use crate::var_creation_context::VarCreationContext;
use crate::var_fns::PollVarFns;
use anyhow::{anyhow, Error as AnyError};
use async_trait::async_trait;
use log::warn;
use serde_yaml::Value;
use std::collections::HashMap;
use std::sync::Arc;

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
        // NOTE: It's unclear whether there's any practical use to
        // look up multiple seats. For now we assume a single seat
        // device. Manual for sd-login.h states that seat0 always
        // exists.
        // https://www.freedesktop.org/software/systemd/man/sd-login.html
        let idle_hint_res = system_dbus_conn
            .call_method(
                Some("org.freedesktop.login1"),
                "/org/freedesktop/login1/seat/seat0",
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &["org.freedesktop.login1.Seat", "IdleHint"],
            )
            .await;
        process_call_result(idle_hint_res)
            .map_err(|e| warn!("Failed to fetch login manager property IdleHint: {:?}", e))
            .ok()
    }
}

fn process_call_result(
    idle_hint_res: Result<Arc<zbus::Message>, zbus::Error>,
) -> Result<VarValue, AnyError> {
    let idle_hint_msg = idle_hint_res?;
    let body_value: zvariant::Value = idle_hint_msg.body()?;
    if let zvariant::Value::Bool(idle_hint) = body_value {
        // Using ! operator because the semantics is inverted. Login
        // manager uses "idle", we use "busy" because we typically
        // want truthy values for "block suspend when ...".
        Ok(VarValue::Bool(!idle_hint))
    } else {
        Err(anyhow!("Wrong data type."))
    }
}
