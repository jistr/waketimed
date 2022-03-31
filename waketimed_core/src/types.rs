use super::RuleNameError;
use lazy_static::lazy_static;
use regex::Regex;

const RULE_NAME_MAX_LENGTH: usize = 80;
const RULE_NAME_CHARSET_REGEX: &str = r"(?-u)^[a-zA-Z0-9_\.]+$";
const RULE_NAME_PATTERN_REGEX: &str = r"(?-u)^[a-zA-Z0-9_]+(?:\.[a-zA-Z0-9_]+)+$";

pub type Value = zvariant::Value<'static>;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_name_valid() -> Result<(), RuleNameError> {
        let name =
            "org.waketimed.test_rule.UUID_length_and_more_lorem_ipsum_dolor_sit_amet".to_string();
        let _rn: RuleName = name.try_into()?;
        Ok(())
    }

    #[test]
    fn test_rule_name_invalid() {
        let name = "".to_string(); // empty
        assert!(RuleName::try_from(name).is_err());
        let name = "X".repeat(81); // too long
        assert!(RuleName::try_from(name).is_err());
        let name = "a.b.c-d".to_string(); // contains '-'
        assert!(RuleName::try_from(name).is_err());
        let name = "a.b.č".to_string(); // contains 'č'
        assert!(RuleName::try_from(name).is_err());
        let name = "a..b.c".to_string(); // consecutive periods
        assert!(RuleName::try_from(name).is_err());
        let name = ".a.b.c".to_string(); // starts with a period
        assert!(RuleName::try_from(name).is_err());
        let name = "a.b.c.".to_string(); // ends with a period
        assert!(RuleName::try_from(name).is_err());
    }
}
