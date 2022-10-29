use super::RuleError;
use serde::de::DeserializeOwned;
use serde_yaml::{from_value, Value};
use std::collections::HashMap;

#[allow(dead_code)]
pub fn param_required<T>(params: &HashMap<String, Value>, key: &str) -> Result<T, RuleError>
where
    T: DeserializeOwned,
{
    let value = param_required_value(params, key)?;
    from_value(value).map_err(|e| RuleError::IncorrectParamType(key.to_string(), e))
}

pub fn param_required_value(
    params: &HashMap<String, Value>,
    key: &str,
) -> Result<Value, RuleError> {
    params
        .get(key)
        .ok_or_else(|| RuleError::ParamMissing(key.to_string()))
        // TODO: replace with .cloned() when Debian testing compiler supports it
        .map(|value| value.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_params() -> HashMap<String, Value> {
        let mut params = HashMap::new();
        params.insert("a key".into(), Value::from("a val"));
        params.insert("b key".into(), Value::from("b val"));
        params
    }

    #[test]
    fn test_param_required() -> Result<(), RuleError> {
        let params = create_params();
        assert_eq!("a val", &(param_required::<String>(&params, "a key")?));
        assert_eq!("b val", &(param_required::<String>(&params, "b key")?));

        let res = param_required::<String>(&params, "c key");
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.to_string().contains("c key"));

        Ok(())
    }
}
