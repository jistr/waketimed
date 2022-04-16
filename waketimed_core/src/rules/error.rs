use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuleError {
    #[error("Rule parameter '{0}' is missing.")]
    ParamMissing(String),
    #[error("Rule parameter '{0}' is of incorrect data type.")]
    IncorrectParamType(String, #[source] zvariant::Error),
    #[error("Incorrect rule name.")]
    IncorrectName(#[source] RuleNameError),
}

#[derive(Error, Debug)]
pub enum RuleNameError {
    #[error("Rule name cannot be empty.")]
    Empty,
    #[error("Rule name '{0}' is too long. Maximum length is {1} characters.")]
    TooLong(String, usize),
    #[error("Rule name '{0}' contains disallowed charecters. Allowed are ASCII alphanumerics, underscore, and period.)")]
    DisallowedCharacters(String),
    #[error("Rule name '{0}' follows an incorrect pattern. It must not start or end with a period or contain consecutive periods.")]
    IncorrectPattern(String),
}

impl From<RuleNameError> for RuleError {
    fn from(e: RuleNameError) -> Self {
        Self::IncorrectName(e)
    }
}
