use anyhow::{anyhow, Context, Error as AnyError};
use std::sync::Arc;
use zbus::blocking::Connection as BlockingConnection;

pub fn is_chassis_allowed<S: AsRef<str>>(chassis_type: &str, allowed_chassis_types: &[S]) -> bool {
    let allowed_types_str: Vec<&str> = allowed_chassis_types.iter().map(|t| t.as_ref()).collect();
    allowed_types_str.contains(&"all") || allowed_types_str.contains(&chassis_type)
}

pub fn chassis_type() -> Result<String, AnyError> {
    let conn = BlockingConnection::system()?;
    let chassis_res = conn.call_method(
        Some("org.freedesktop.hostname1"),
        "/org/freedesktop/hostname1",
        Some("org.freedesktop.DBus.Properties"),
        "Get",
        &["org.freedesktop.hostname1", "Chassis"],
    );
    process_chassis_call_result(chassis_res).context("Failed to fetch chassis type.")
}

fn process_chassis_call_result(
    chassis_res: Result<Arc<zbus::Message>, zbus::Error>,
) -> Result<String, AnyError> {
    let chassis_msg = chassis_res?;
    let body_value: zvariant::Value = chassis_msg.body()?;
    if let zvariant::Value::Str(chassis) = body_value {
        Ok(chassis.to_string())
    } else {
        Err(anyhow!("Wrong data type."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_chassis_allowed() {
        assert!(is_chassis_allowed("handset", &["handset", "tablet"]));
        assert!(is_chassis_allowed("anythinggoes", &["all"]));
        assert!(is_chassis_allowed(
            "anythinggoes",
            &["handset", "all", "tablet"]
        ));
        assert!(!is_chassis_allowed("desktop", &["handset", "tablet"]));
    }
}
