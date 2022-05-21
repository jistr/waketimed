use super::VarName;
use serde_derive::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VarDef {
    #[serde(skip)]
    pub name: Option<VarName>,
    pub data_type: VarDataType,
    #[serde(default)]
    pub categories: Vec<VarName>,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarKind {
    /// Variable polled in intervals, with is_active & poll functions
    /// built into waketimed.
    #[serde(rename = "builtin_poll")]
    BuiltinPoll(BuiltinPollDef),
    /// Boolean variable which is true if any variables in the
    /// specified category_name are true. If all such variables are
    /// false or if there are no such variables defined/active, the
    /// CategoryAny variable value is false.
    #[serde(rename = "category_any")]
    CategoryAny(CategoryAnyDef),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuiltinPollDef {
    pub builtin_name: String,
    #[serde(default)]
    pub params: HashMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryAnyDef {
    pub category_name: VarName,
}
