use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuleError {
    #[error("Rule parameter '{0}' is missing.")]
    ParamMissing(String),
    #[error("Rule parameter '{0}' is of incorrect data type.")]
    IncorrectParamType(String, #[source] zvariant::Error),
}
