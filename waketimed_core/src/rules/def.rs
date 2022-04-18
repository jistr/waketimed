use super::{RuleError, RuleName};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use zvariant::{OwnedValue, Type};

#[derive(Debug, PartialEq, Eq)]
pub struct RuleDef {
    pub name: RuleName,
    pub type_def: RuleTypeDef,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RuleTypeDef {
    // NOTE: This will have some definition eventually
    StayupBool,
}

impl TryFrom<&RawRuleDef> for RuleDef {
    type Error = super::RuleError;

    fn try_from(raw_def: &RawRuleDef) -> Result<Self, Self::Error> {
        Ok(RuleDef {
            name: raw_def.name.clone().try_into()?,
            type_def: type_def_from_raw(raw_def)?,
        })
    }
}

fn type_def_from_raw(raw_def: &RawRuleDef) -> Result<RuleTypeDef, RuleError> {
    match raw_def.rule_type {
        RawRuleType::StayupBool => Ok(RuleTypeDef::StayupBool),
    }
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct RawRuleDef {
    pub name: String,
    pub rule_type: RawRuleType,
    pub params: HashMap<String, OwnedValue>,
}

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq)]
pub enum RawRuleType {
    StayupBool,
}

impl From<&RuleDef> for RawRuleDef {
    fn from(rule_def: &RuleDef) -> Self {
        let (rule_type, params) = match &rule_def.type_def {
            RuleTypeDef::StayupBool => (RawRuleType::StayupBool, HashMap::new()),
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
        let params: HashMap<String, OwnedValue> = HashMap::new();
        RawRuleDef {
            name: "org.waketimed.stayup_test".to_string(),
            rule_type: RawRuleType::StayupBool,
            params,
        }
    }

    fn rule_def() -> RuleDef {
        RuleDef {
            name: "org.waketimed.stayup_test".to_string().try_into().unwrap(),
            type_def: RuleTypeDef::StayupBool,
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