use crate::RuleError;
use std::collections::HashMap;

pub fn clone_param_required(
    params: &HashMap<String, String>,
    key: &str,
) -> Result<String, RuleError> {
    params
        .get(key)
        .cloned()
        .ok_or_else(|| RuleError::ParamMissing(key.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_params() -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("a key".into(), "a val".into());
        params.insert("b key".into(), "b val".into());
        params
    }

    #[test]
    fn test_clone_param_required() -> Result<(), RuleError> {
        let params = create_params();
        assert_eq!("a val", &(clone_param_required(&params, "a key")?));
        assert_eq!("b val", &(clone_param_required(&params, "b key")?));

        let res = clone_param_required(&params, "c key");
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.to_string().contains("c key"));

        Ok(())
    }
}
