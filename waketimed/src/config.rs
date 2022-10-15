use anyhow::{anyhow, Context, Error as AnyError};
use log::debug;
use serde_derive::{Deserialize, Serialize};

use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

const CONFIG_FILE_VAR: &str = "WAKETIMED_CONFIG";
const DEFAULT_CONFIG_FILE: &str = "/etc/waketimed/config.yaml";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    // Log level.
    #[serde(default = "default_log")]
    pub log: String,
    // Directory with extra config files for the daemon. Can contain
    // custom rules and variables. If specified as relative path, it
    // is interpreted relative to the waketimed process working
    // directory. It is recommended to specify absolute paths.
    #[serde(default = "default_config_dir")]
    pub config_dir: String,
    // Time between re-checking poll-based variables, in milliseconds.
    // Larger values mean less exact times of updating variables (less
    // exact times of falling asleep), but consume less CPU.
    #[serde(default = "default_poll_variable_interval")]
    pub poll_variable_interval: u64,
    // Chassis types where waketimed should normally operate. If
    // launched on a chassis type not in the list, waketimed should
    // enter disabled mode (not performing any actions until restart).
    // Special value "all" can be put into the list to indicate that
    // all chassis types are allowed.
    #[serde(default = "default_allowed_chassis_types")]
    pub allowed_chassis_types: Vec<String>,

    // Time to stay up (prevent sleep) after waketimed starts, in
    // seconds. Results in automatic creation of a "stay up until"
    // rule. Should waketimed get misconfigured and put the device to
    // sleep too often, this gives system admin a chance to fix the
    // situation or stop waketimed after system boot.
    #[serde(default = "default_startup_awake_time")]
    pub startup_awake_time: u64,
    // Minimum time to stay up after waking up. Useful to prevent fast
    // flapping between sleep/wake.
    #[serde(default = "default_minimum_awake_time")]
    pub minimum_awake_time: u64,
    // Minimum time to stay awake after last stayup condition turns
    // false, in seconds. Useful to prevent going to sleep immediately
    // without sending out any "sleep approaching" signals.
    #[serde(default = "default_stayup_cleared_awake_time")]
    pub stayup_cleared_awake_time: u64,

    // Test mode prevents waketimed from actually suspending the
    // system.
    #[serde(default = "default_test_mode")]
    pub test_mode: bool,
    // Test variable to prevent loading builtin rule defs and var
    // defs.
    #[serde(default = "default_test_skip_embedded_defs")]
    pub test_skip_embedded_defs: bool,
}

impl Config {
    pub fn config_dir(&self) -> Option<PathBuf> {
        if self.config_dir.is_empty() {
            None
        } else {
            Some(PathBuf::from(&self.config_dir))
        }
    }

    pub fn config_rule_def_dir(&self) -> Option<PathBuf> {
        self.config_dir().map(|dir| dir.join("rule_def"))
    }

    pub fn config_var_def_dir(&self) -> Option<PathBuf> {
        self.config_dir().map(|dir| dir.join("var_def"))
    }

    pub fn rule_def_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if !self.test_skip_embedded_defs {
            dirs.push(PathBuf::from(crate::embedded_files::PREFIX_RULE_DEF));
        }
        if let Some(dir) = self.config_rule_def_dir() {
            dirs.push(dir);
        }
        dirs
    }

    pub fn var_def_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if !self.test_skip_embedded_defs {
            dirs.push(PathBuf::from(crate::embedded_files::PREFIX_VAR_DEF));
        }
        if let Some(dir) = self.config_var_def_dir() {
            dirs.push(dir);
        }
        dirs
    }
}

pub fn load() -> Result<Config, AnyError> {
    let mut cfg_path_explicit = true;
    let cfg_path = env::var(CONFIG_FILE_VAR).unwrap_or_else(|_| {
        cfg_path_explicit = false;
        DEFAULT_CONFIG_FILE.to_string()
    });

    let mut cfg: Config = if !cfg_path_explicit && !Path::new(&cfg_path).exists() {
        // If config path was not provided explicitly, and the default
        // config does not exist, let's just use built-in defaults. We
        // can't use `Config::default()`, as it would ignore serde
        // defaults.
        serde_yaml::from_str("{}")?
    } else {
        let cfg_reader = File::open(&cfg_path)
            .with_context(|| format!("Failed to open config file '{}'", &cfg_path))?;
        serde_yaml::from_reader(cfg_reader)
            .with_context(|| format!("Failed parse config file '{}'", &cfg_path))?
    };

    populate_config_from_env(&mut cfg)?;
    check_and_repair_config(&mut cfg)?;

    Ok(cfg)
}

pub fn log_config(cfg: &Config) -> Result<(), AnyError> {
    debug!(
        "Config settings:\n{}",
        serde_yaml::to_string::<Config>(cfg)?
    );
    Ok(())
}

fn populate_config_from_env(cfg: &mut Config) -> Result<(), AnyError> {
    if let Ok(value) = env::var("WAKETIMED_LOG") {
        cfg.log = value;
    }
    if let Ok(value) = env::var("WAKETIMED_CONFIG_DIR") {
        cfg.config_dir = value;
    }
    if let Ok(value) = env::var("WAKETIMED_STARTUP_AWAKE_TIME") {
        cfg.startup_awake_time = value.parse::<u64>()?;
    }
    if let Ok(value) = env::var("WAKETIMED_MINIMUM_AWAKE_TIME") {
        cfg.minimum_awake_time = value.parse::<u64>()?;
    }
    if let Ok(value) = env::var("WAKETIMED_STAYUP_CLEARED_AWAKE_TIME") {
        cfg.stayup_cleared_awake_time = value.parse::<u64>()?;
    }
    if let Ok(value) = env::var("WAKETIMED_POLL_VARIABLE_INTERVAL") {
        cfg.poll_variable_interval = value.parse::<u64>()?;
    }
    if let Ok(value) = env::var("WAKETIMED_ALLOWED_CHASSIS_TYPES") {
        cfg.allowed_chassis_types = value.split(',').map(|s| s.to_string()).collect();
    }
    if let Ok(value) = env::var("WAKETIMED_TEST_MODE") {
        cfg.test_mode = value.parse::<bool>()?;
    }
    if let Ok(value) = env::var("WAKETIMED_TEST_SKIP_EMBEDDED_DEFS") {
        cfg.test_skip_embedded_defs = value.parse::<bool>()?;
    }
    Ok(())
}

fn check_and_repair_config(cfg: &mut Config) -> Result<(), AnyError> {
    check_config_dir(cfg)?;
    Ok(())
}

fn check_config_dir(cfg: &Config) -> Result<(), AnyError> {
    if cfg.config_dir == default_config_dir() {
        return Ok(());
    }

    if let Some(dir) = cfg.config_dir() {
        if !dir.exists() {
            return Err(anyhow!(
                "Non-default config dir '{}' specified, but it does not exist.",
                cfg.config_dir
            ));
        }
        if !dir.is_dir() {
            return Err(anyhow!(
                "Config dir '{}' is not a directory.",
                cfg.config_dir
            ));
        }
    }
    Ok(())
}

fn default_log() -> String {
    "info".to_string()
}

fn default_config_dir() -> String {
    "/etc/waketimed".to_string()
}

fn default_startup_awake_time() -> u64 {
    300_000
}

fn default_minimum_awake_time() -> u64 {
    10_000
}

fn default_stayup_cleared_awake_time() -> u64 {
    10_000
}

fn default_poll_variable_interval() -> u64 {
    3_000
}

fn default_allowed_chassis_types() -> Vec<String> {
    vec![
        "convertible".to_string(),
        "embedded".to_string(),
        "handset".to_string(),
        "tablet".to_string(),
        "watch".to_string(),
    ]
}

fn default_test_mode() -> bool {
    false
}

fn default_test_skip_embedded_defs() -> bool {
    false
}
