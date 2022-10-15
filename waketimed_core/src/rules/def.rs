use super::RuleName;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleDef {
    #[serde(skip)]
    pub name: Option<RuleName>,
    #[serde(with = "serde_yaml::with::singleton_map")]
    pub kind: RuleKind,
}

impl RuleDef {
    pub fn name(&self) -> &RuleName {
        self.name
            .as_ref()
            .expect("Fatal: rule def structure without a name.")
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleKind {
    #[serde(rename = "stayup_bool")]
    StayupBool(StayupBoolDef),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StayupBoolDef {
    pub value_script: String,
}
