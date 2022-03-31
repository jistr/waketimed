use super::StayupBuiltinDef;
use crate::{RuleError, RuleName, Value};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use zvariant::Type;

#[derive(Debug, PartialEq, Eq)]
pub struct RuleDef {
    pub name: RuleName,
    pub type_def: RuleTypeDef,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RuleTypeDef {
    StayupBuiltin(StayupBuiltinDef),
}

impl TryFrom<&RawRuleDef> for RuleDef {
    type Error = crate::RuleError;

    fn try_from(raw_def: &RawRuleDef) -> Result<Self, Self::Error> {
        Ok(RuleDef {
            name: raw_def.name.clone().try_into()?,
            type_def: type_def_from_raw(raw_def)?,
        })
    }
}

fn type_def_from_raw(raw_def: &RawRuleDef) -> Result<RuleTypeDef, RuleError> {
    match raw_def.rule_type {
        RawRuleType::StayupBuiltin => Ok(RuleTypeDef::StayupBuiltin(
            StayupBuiltinDef::from_params(&raw_def.params)?,
        )),
    }
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct RawRuleDef {
    pub name: String,
    pub rule_type: RawRuleType,
    pub params: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq)]
pub enum RawRuleType {
    StayupBuiltin,
}

impl From<&RuleDef> for RawRuleDef {
    fn from(rule_def: &RuleDef) -> Self {
        let (rule_type, params) = match &rule_def.type_def {
            RuleTypeDef::StayupBuiltin(def) => (RawRuleType::StayupBuiltin, def.to_params()),
        };
        RawRuleDef {
            name: rule_def.name.clone().into(),
            rule_type,
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_rule_def() -> RawRuleDef {
        let mut params: HashMap<String, Value> = HashMap::new();
        params.insert("builtin_name".to_string(), "stayup_test".into());
        RawRuleDef {
            name: "org.waketimed.stayup_test".to_string(),
            rule_type: RawRuleType::StayupBuiltin,
            params,
        }
    }

    fn rule_def() -> RuleDef {
        let sb_def = StayupBuiltinDef {
            builtin_name: "stayup_test".to_string(),
        };
        RuleDef {
            name: "org.waketimed.stayup_test".to_string().try_into().unwrap(),
            type_def: RuleTypeDef::StayupBuiltin(sb_def),
        }
    }

    #[test]
    fn test_def_from_raw() -> Result<(), RuleError> {
        assert_eq!(rule_def(), RuleDef::try_from(&raw_rule_def())?);
        Ok(())
    }

    #[test]
    fn test_raw_from_def() -> Result<(), RuleError> {
        assert_eq!(raw_rule_def(), RawRuleDef::from(&rule_def()));
        Ok(())
    }
}
