use crate::{RuleError, Value};
use std::collections::HashMap;

pub fn param_required<T>(params: &HashMap<String, Value>, key: &str) -> Result<T, RuleError>
where
    T: TryFrom<Value, Error = zvariant::Error>,
{
    params
        .get(key)
        .cloned()
        .ok_or_else(|| RuleError::ParamMissing(key.to_string()))
        .and_then(|value| {
            value
                .try_into()
                .map_err(|e| RuleError::IncorrectParamType(key.to_string(), e))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_params() -> HashMap<String, Value> {
        let mut params = HashMap::new();
        params.insert("a key".into(), "a val".into());
        params.insert("b key".into(), "b val".into());
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