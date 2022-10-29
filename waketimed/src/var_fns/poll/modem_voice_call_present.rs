use crate::core::vars::VarValue;
use crate::var_creation_context::VarCreationContext;
use crate::var_fns::PollVarFns;
use anyhow::Error as AnyError;
use async_trait::async_trait;
use log::{trace, warn};
use serde_yaml::Value;
use std::collections::HashMap;
use std::sync::Arc;
use zbus::Connection as ZbusConnection;

#[derive(Clone, Debug)]
pub struct ModemVoiceCallPresentFns {
    system_dbus_conn: ZbusConnection,
    voice_devices: Option<Vec<String>>,
}

impl ModemVoiceCallPresentFns {
    pub fn new(
        _params: &HashMap<String, Value>,
        context: &VarCreationContext,
    ) -> Result<Self, AnyError> {
        Ok(Self {
            system_dbus_conn: context.system_dbus_conn()?,
            voice_devices: None,
        })
    }
}

#[async_trait]
impl PollVarFns for ModemVoiceCallPresentFns {
    async fn poll(&mut self) -> Option<VarValue> {
        let system_dbus_conn = self.system_dbus_conn.clone();

        if self.voice_devices.is_none() {
            self.voice_devices = fetch_voice_devices(&system_dbus_conn).await;
            trace!("List of voice devices: {:?}", self.voice_devices);
        }
        let voice_devices = if let Some(vd) = &self.voice_devices {
            vd
        } else {
            return None;
        };

        // Do something smarter when async_iter is stable:
        // https://doc.rust-lang.org/std/async_iter/index.html
        // E.g.:
        // let devices_have_calls: Vec<bool> = voice_devices.iter().map(|d| {
        //     device_has_calls_present(&system_dbus_conn, d)
        // }).collect();
        let mut devices_have_calls: Vec<bool> = Vec::new();
        for device in voice_devices.iter() {
            devices_have_calls.push(device_has_calls_present(&system_dbus_conn, device).await);
        }
        let any_device_has_calls = devices_have_calls.iter().any(|has_calls| *has_calls);
        Some(VarValue::Bool(any_device_has_calls))
    }
}

async fn fetch_voice_devices(system_dbus_conn: &ZbusConnection) -> Option<Vec<String>> {
    let list_devices_res = system_dbus_conn
        .call_method(
            Some("org.freedesktop.ModemManager1"),
            "/org/freedesktop/ModemManager1",
            Some("org.freedesktop.DBus.ObjectManager"),
            "GetManagedObjects",
            &[] as &[&'static str; 0],
        )
        .await;
    process_list_devices_result(list_devices_res)
        .map_err(|e| warn!("Failed to list modem manager voice devices: {:?}", e))
        .ok()
}

fn process_list_devices_result(
    list_calls_res: Result<Arc<zbus::Message>, zbus::Error>,
) -> Result<Vec<String>, AnyError> {
    let list_calls_msg = list_calls_res?;
    // Keys are DBus object paths, values are hash maps of <interface
    // name, props>. We're looking only for devices which have a Voice
    // interface defined on them.
    let mut devices: HashMap<
        zvariant::ObjectPath,
        HashMap<String, HashMap<String, zvariant::Value>>,
    > = list_calls_msg.body()?;
    devices.retain(|_, ifaces| ifaces.contains_key("org.freedesktop.ModemManager1.Modem.Voice"));
    Ok(devices.keys().map(|path| path.to_string()).collect())
}

async fn device_has_calls_present(system_dbus_conn: &ZbusConnection, device_path: &str) -> bool {
    let list_calls_res = system_dbus_conn
        .call_method(
            Some("org.freedesktop.ModemManager1"),
            device_path,
            Some("org.freedesktop.ModemManager1.Modem.Voice"),
            "ListCalls",
            &[] as &[&'static str; 0],
        )
        .await;
    process_has_calls_result(list_calls_res)
        .map_err(|e| warn!("Failed to call modem manager ListCalls: {:?}", e))
        .unwrap_or(false)
}

fn process_has_calls_result(
    list_calls_res: Result<Arc<zbus::Message>, zbus::Error>,
) -> Result<bool, AnyError> {
    let list_calls_msg = list_calls_res?;
    let calls: Vec<zvariant::ObjectPath> = list_calls_msg.body()?;
    Ok(!calls.is_empty())
}
