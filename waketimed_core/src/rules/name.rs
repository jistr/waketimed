use super::RuleNameError;
use lazy_static::lazy_static;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

const RULE_NAME_MAX_LENGTH: usize = 80;
const RULE_NAME_CHARSET_REGEX: &str = r"(?-u)^[a-z0-9_]+$";
const RULE_NAME_PATTERN_REGEX: &str = r"(?-u)^[a-z][a-z0-9]*(?:_[a-z0-9]+)*$";

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleName(String);

impl TryFrom<String> for RuleName {
    type Error = RuleNameError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref RE_CHARSET: Regex = Regex::new(RULE_NAME_CHARSET_REGEX)
                .expect("Failed to compile RuleName regular expression.");
            static ref RE_PATTERN: Regex = Regex::new(RULE_NAME_PATTERN_REGEX)
                .expect("Failed to compile RuleName regular expression.");
        }
        let len = s.len();
        if len < 1 {
            return Err(RuleNameError::Empty);
        }
        if len > RULE_NAME_MAX_LENGTH {
            return Err(RuleNameError::TooLong(s, RULE_NAME_MAX_LENGTH));
        }
        if !RE_CHARSET.is_match(&s) {
            return Err(RuleNameError::DisallowedCharacters(s));
        }
        if !RE_PATTERN.is_match(&s) {
            return Err(RuleNameError::IncorrectPattern(s));
        }
        Ok(RuleName(s))
    }
}

impl AsRef<str> for RuleName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsMut<str> for RuleName {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl From<RuleName> for String {
    fn from(rule_name: RuleName) -> String {
        rule_name.0
    }
}

impl fmt::Display for RuleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_name_valid() {
        let name = "wtd_display_on".to_string();
        assert!(RuleName::try_from(name).is_ok());
        let name = "abc9".to_string();
        assert!(RuleName::try_from(name).is_ok());
    }

    #[test]
    fn test_rule_name_invalid() {
        let name = "".to_string(); // empty
        assert!(RuleName::try_from(name).is_err());
        let name = "X".repeat(81); // too long
        assert!(RuleName::try_from(name).is_err());
        let name = "a_b_c-d".to_string(); // contains '-'
        assert!(RuleName::try_from(name).is_err());
        let name = "a_b.c".to_string(); // contains '.'
        assert!(RuleName::try_from(name).is_err());
        let name = "abč".to_string(); // contains 'č'
        assert!(RuleName::try_from(name).is_err());
        let name = "a__bc".to_string(); // consecutive underscores
        assert!(RuleName::try_from(name).is_err());
        let name = "_abc".to_string(); // starts with an underscore
        assert!(RuleName::try_from(name).is_err());
        let name = "abc_".to_string(); // ends with an underscore
        assert!(RuleName::try_from(name).is_err());
        let name = "9abc".to_string(); // starts with a number
        assert!(RuleName::try_from(name).is_err());
        let name = "abc9".to_string(); // ends with a number - ok
        assert!(RuleName::try_from(name).is_ok());
    }
}
