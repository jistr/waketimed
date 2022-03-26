use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuleError {
    #[error("Rule parameter '{0}' is missing.")]
    ParamMissing(String),
}
