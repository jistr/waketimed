use super::{param_required, VarError, VarName};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use zvariant::{OwnedValue, Value};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VarDef {
    #[serde(skip)]
    pub name: Option<VarName>,
    pub data_type: VarDataType,
    pub kind: VarKind,
}

impl VarDef {
    pub fn name(&self) -> &VarName {
        self.name
            .as_ref()
            .expect("Fatal: var def structure without a name.")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarDataType {
    #[serde(rename = "bool")]
    Bool,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarKind {
    #[serde(rename = "builtin_poll")]
    BuiltinPoll(BuiltinPollDef),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuiltinPollDef {
    pub builtin_name: String,
}

impl BuiltinPollDef {
    pub fn from_params(params: &HashMap<String, OwnedValue>) -> Result<Self, VarError> {
        Ok(Self {
            builtin_name: param_required(params, "builtin_name")?,
        })
    }

    pub fn to_params(&self) -> HashMap<String, OwnedValue> {
        let mut params = HashMap::new();
        params.insert(
            "builtin_name".to_string(),
            Value::from(self.builtin_name.clone()).into(),
        );
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builtin_poll_def() -> BuiltinPollDef {
        BuiltinPollDef {
            builtin_name: "display_on".to_string(),
        }
    }

    fn builtin_poll_params() -> HashMap<String, OwnedValue> {
        let mut params = HashMap::new();
        params.insert("builtin_name".to_string(), Value::from("display_on").into());
        params
    }

    #[test]
    fn test_builtin_poll_def_from_params() -> Result<(), VarError> {
        assert_eq!(
            builtin_poll_def(),
            BuiltinPollDef::from_params(&builtin_poll_params())?
        );
        Ok(())
    }

    #[test]
    fn test_params_from_builtin_poll_def() {
        assert_eq!(builtin_poll_params(), builtin_poll_def().to_params());
    }
}
