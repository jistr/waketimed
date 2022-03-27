use super::rule_params::param_string_required;
use crate::{RuleError, Value};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub struct StayupBuiltinDef {
    pub builtin_name: String,
}

impl StayupBuiltinDef {
    pub fn from_params(params: &HashMap<String, Value>) -> Result<Self, RuleError> {
        Ok(Self {
            builtin_name: param_string_required(params, "builtin_name")?,
        })
    }

    pub fn to_params(&self) -> HashMap<String, Value> {
        let mut params = HashMap::new();
        params.insert("builtin_name".to_string(), self.builtin_name.clone().into());
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stayup_builtin_def() -> StayupBuiltinDef {
        StayupBuiltinDef {
            builtin_name: "stayup_test".to_string(),
        }
    }

    fn stayup_builtin_params() -> HashMap<String, Value> {
        let mut params = HashMap::new();
        params.insert("builtin_name".to_string(), "stayup_test".into());
        params
    }

    #[test]
    fn test_def_from_params() -> Result<(), RuleError> {
        assert_eq!(
            stayup_builtin_def(),
            StayupBuiltinDef::from_params(&stayup_builtin_params())?
        );
        Ok(())
    }

    #[test]
    fn test_params_from_def() {
        assert_eq!(stayup_builtin_params(), stayup_builtin_def().to_params());
    }
}
