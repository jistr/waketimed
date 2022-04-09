use super::var_params::param_required;
use crate::{VarError, VarName};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use zvariant::{OwnedValue, Type, Value};

#[derive(Debug, PartialEq, Eq)]
pub struct VarDef {
    pub name: VarName,
    pub data_type: VarDataType,
    pub kind: VarKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum VarDataType {
    Bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum VarKind {
    BuiltinPoll(BuiltinPollDef),
}

impl TryFrom<(&str, &RawVarDef)> for VarDef {
    type Error = crate::VarError;

    fn try_from(name_and_raw_def: (&str, &RawVarDef)) -> Result<Self, Self::Error> {
        let (name, raw_def) = name_and_raw_def;
        Ok(VarDef {
            name: name.to_string().try_into()?,
            data_type: raw_def.data_type.clone(),
            kind: kind_from_raw(raw_def)?,
        })
    }
}

fn kind_from_raw(raw_def: &RawVarDef) -> Result<VarKind, VarError> {
    match raw_def.var_kind {
        RawVarKind::BuiltinPoll => Ok(VarKind::BuiltinPoll(BuiltinPollDef::from_params(
            &raw_def.params,
        )?)),
    }
}

#[derive(Debug, Deserialize, Serialize, Type, PartialEq)]
pub struct RawVarDef {
    pub data_type: VarDataType,
    pub var_kind: RawVarKind,
    pub params: HashMap<String, OwnedValue>,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq)]
pub enum RawVarKind {
    BuiltinPoll,
}

impl From<&VarDef> for RawVarDef {
    fn from(var_def: &VarDef) -> Self {
        let (var_kind, params) = match &var_def.kind {
            VarKind::BuiltinPoll(def) => (RawVarKind::BuiltinPoll, def.to_params()),
        };
        RawVarDef {
            data_type: var_def.data_type.clone(),
            var_kind,
            params,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
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

    fn raw_var_def() -> RawVarDef {
        let mut params: HashMap<String, OwnedValue> = HashMap::new();
        params.insert("builtin_name".to_string(), Value::from("display_on").into());
        RawVarDef {
            data_type: VarDataType::Bool,
            var_kind: RawVarKind::BuiltinPoll,
            params,
        }
    }

    fn var_def() -> VarDef {
        VarDef {
            name: "wtd_display_on".to_string().try_into().unwrap(),
            data_type: VarDataType::Bool,
            kind: VarKind::BuiltinPoll(builtin_poll_def()),
        }
    }

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
    fn test_def_from_raw() -> Result<(), VarError> {
        assert_eq!(
            var_def(),
            VarDef::try_from(("wtd_display_on", &raw_var_def()))?
        );
        Ok(())
    }

    #[test]
    fn test_raw_from_def() -> Result<(), VarError> {
        assert_eq!(raw_var_def(), RawVarDef::from(&var_def()));
        Ok(())
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
