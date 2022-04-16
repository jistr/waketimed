use super::VarNameError;
use lazy_static::lazy_static;
use regex::Regex;

const VAR_NAME_MAX_LENGTH: usize = 40;
const VAR_NAME_CHARSET_REGEX: &str = r"(?-u)^[a-z0-9_]+$";
const VAR_NAME_PATTERN_REGEX: &str = r"(?-u)^[a-z0-9]+(?:_[a-z0-9]+)*$";

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VarName(String);

impl TryFrom<String> for VarName {
    type Error = VarNameError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref RE_CHARSET: Regex = Regex::new(VAR_NAME_CHARSET_REGEX)
                .expect("Failed to compile VarName regular expression.");
            static ref RE_PATTERN: Regex = Regex::new(VAR_NAME_PATTERN_REGEX)
                .expect("Failed to compile VarName regular expression.");
        }
        let len = s.len();
        if len < 1 {
            return Err(VarNameError::Empty);
        }
        if len > VAR_NAME_MAX_LENGTH {
            return Err(VarNameError::TooLong(s, VAR_NAME_MAX_LENGTH));
        }
        if !RE_CHARSET.is_match(&s) {
            return Err(VarNameError::DisallowedCharacters(s));
        }
        if !RE_PATTERN.is_match(&s) {
            return Err(VarNameError::IncorrectPattern(s));
        }
        Ok(VarName(s))
    }
}

impl AsRef<str> for VarName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsMut<str> for VarName {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl From<VarName> for String {
    fn from(var_name: VarName) -> String {
        var_name.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_name_valid() -> Result<(), VarNameError> {
        let name = "wtd_display_on".to_string();
        let _rn: VarName = name.try_into()?;
        Ok(())
    }

    #[test]
    fn test_var_name_invalid() {
        let name = "".to_string(); // empty
        assert!(VarName::try_from(name).is_err());
        let name = "x".repeat(60); // too long
        assert!(VarName::try_from(name).is_err());
        let name = "a_b.c".to_string(); // contains '.'
        assert!(VarName::try_from(name).is_err());
        let name = "abč".to_string(); // contains 'č'
        assert!(VarName::try_from(name).is_err());
        let name = "a__bc".to_string(); // consecutive underscores
        assert!(VarName::try_from(name).is_err());
        let name = "_abc".to_string(); // starts with an underscore
        assert!(VarName::try_from(name).is_err());
        let name = "abc_".to_string(); // ends with an underscore
        assert!(VarName::try_from(name).is_err());
    }
}
