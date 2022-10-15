use crate::config::Config;
use waketimed_core::rules::RuleName;
use waketimed_core::vars::VarName;

pub fn default_config() -> Config {
    let mut cfg: Config = serde_yaml::from_str("{}").expect("Unable to create default Config.");
    cfg.test_mode = true;
    cfg
}

pub fn run_and_term_config() -> Config {
    let mut cfg = default_config();
    cfg.config_dir = format!("{}/tests/data/run_and_term", env!("CARGO_MANIFEST_DIR"),);
    cfg.test_mode = true;
    cfg.poll_variable_interval = 100;
    cfg
}

pub fn run_and_term_without_builtin_defs_config() -> Config {
    let mut cfg = run_and_term_config();
    cfg.test_skip_embedded_defs = true;
    cfg
}

pub fn rule_name(name: &str) -> RuleName {
    RuleName::try_from(name.to_string()).expect("Invalid RuleName")
}

pub fn var_name(name: &str) -> VarName {
    VarName::try_from(name.to_string()).expect("Invalid VarName")
}
