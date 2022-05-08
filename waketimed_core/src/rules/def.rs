use super::RuleName;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleDef {
    #[serde(skip)]
    pub name: Option<RuleName>,
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
    // TODO: Have an is-active check and forget rules which are
    // irrelevant on a given device.
    // pub is_active_script: String,
    pub value_script: String,
}
