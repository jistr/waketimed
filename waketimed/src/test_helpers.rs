use crate::config::Config;
use waketimed_core::vars::VarName;

pub use crate::config::with_config;

pub fn default_config() -> Config {
    serde_yaml::from_str("{}").expect("Unable to create default Config.")
}

pub fn run_and_term_config() -> Config {
    let mut cfg = default_config();
    cfg.state_dir = format!(
        "{}/tests/data/run_and_term/state",
        env!("CARGO_MANIFEST_DIR"),
    );
    cfg.state_dir = format!(
        "{}/tests/data/run_and_term/dist",
        env!("CARGO_MANIFEST_DIR"),
    );
    cfg.poll_variable_interval = 100;
    cfg
}

pub fn var_name(name: &str) -> VarName {
    VarName::try_from(name.to_string()).expect("Invalid VarName")
}
