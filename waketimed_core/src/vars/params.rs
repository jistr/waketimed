use super::VarError;
use serde::de::DeserializeOwned;
use serde_yaml::{from_value, Value};
use std::collections::HashMap;

pub fn param_required<T>(params: &HashMap<String, Value>, key: &str) -> Result<T, VarError>
where
    T: DeserializeOwned,
{
    let value = param_required_value(params, key)?;
    from_value(value).map_err(|e| VarError::IncorrectParamType(key.to_string(), e))
}

pub fn param_required_value(params: &HashMap<String, Value>, key: &str) -> Result<Value, VarError> {
    params
        .get(key)
        .ok_or_else(|| VarError::ParamMissing(key.to_string()))
        .cloned()
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
    fn test_param_required() -> Result<(), VarError> {
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
