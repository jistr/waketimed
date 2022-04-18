use super::VarError;
use std::collections::HashMap;
use zvariant::OwnedValue;

pub fn param_required<T>(params: &HashMap<String, OwnedValue>, key: &str) -> Result<T, VarError>
where
    T: TryFrom<OwnedValue, Error = zvariant::Error>,
{
    params
        .get(key)
        .cloned()
        .ok_or_else(|| VarError::ParamMissing(key.to_string()))
        .and_then(|value| {
            value
                .try_into()
                .map_err(|e| VarError::IncorrectParamType(key.to_string(), e))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use zvariant::Value;

    fn create_params() -> HashMap<String, OwnedValue> {
        let mut params = HashMap::new();
        params.insert("a key".into(), Value::from("a val").into());
        params.insert("b key".into(), Value::from("b val").into());
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