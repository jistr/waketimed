use thiserror::Error;

#[derive(Error, Debug)]
pub enum VarError {
    #[error("Var parameter '{0}' is missing.")]
    ParamMissing(String),
    #[error("Var parameter '{0}' is of incorrect data type.")]
    IncorrectParamType(String, #[source] serde_yaml::Error),
    #[error("Incorrect var name.")]
    IncorrectName(#[source] VarNameError),
}

#[derive(Error, Debug)]
pub enum VarNameError {
    #[error("Var name cannot be empty.")]
    Empty,
    #[error("Var name '{0}' is too long. Maximum length is {1} characters.")]
    TooLong(String, usize),
    #[error("Var name '{0}' contains disallowed charecters. Allowed are lower case ASCII alphanumerics, and underscore.)")]
    DisallowedCharacters(String),
    #[error("Var name '{0}' follows an incorrect pattern.")]
    IncorrectPattern(String),
}

impl From<VarNameError> for VarError {
    fn from(e: VarNameError) -> Self {
        Self::IncorrectName(e)
    }
}
