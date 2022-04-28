use super::VarName;
use serde_derive::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;

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
    #[serde(default)]
    pub params: HashMap<String, Value>,
}
