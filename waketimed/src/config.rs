use anyhow::{Context, Error as AnyError};
use log::debug;
use serde_derive::{Deserialize, Serialize};
use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::rc::Rc;

static mut CONFIG: Option<Rc<RefCell<Config>>> = None;
const CONFIG_FILE_VAR: &str = "WAKETIMED_CONFIG";
const DEFAULT_CONFIG_FILE: &str = "/etc/waketimed/config.yaml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // Log level.
    #[serde(default = "default_log")]
    pub log: String,
    // Directory writable for the daemon, where its state is stored.
    // Contains custom rules definitions, enabled status of any rules
    // (custom or builtin), and tracking of fulfillment of individual
    // rules. This is also the working directory of the daemon.
    #[serde(default = "default_state_dir")]
    pub state_dir: String,
    // Directory with files distributed and upgraded together with the
    // waketimed binary. Contains built-in rule definitions.
    #[serde(default = "default_dist_dir")]
    pub dist_dir: String,
    // Time between re-checking poll-based variables, in milliseconds.
    // Larger values mean less exact times of updating variables (less
    // exact times of falling asleep), but consume less CPU.
    #[serde(default = "default_poll_variable_interval")]
    pub poll_variable_interval: u64,

    // Time to stay up (prevent sleep) after waketimed starts, in
    // seconds. Results in automatic creation of a "stay up until"
    // rule. Should waketimed get misconfigured and put the device to
    // sleep too often, this gives system admin a chance to fix the
    // situation or stop waketimed after system boot.
    #[serde(default = "default_startup_awake_time")]
    pub startup_awake_time: u32,
    // Minimum time to stay up after waking up. Useful to prevent fast
    // flapping between sleep/wake.
    #[serde(default = "default_minimum_awake_time")]
    pub minimum_awake_time: u32,
    // Minimum time to stay awake after last stayup condition turns
    // false, in seconds. Useful to prevent going to sleep immediately
    // without sending out any "sleep approaching" signals.
    #[serde(default = "default_stayup_cleared_awake_time")]
    pub stayup_cleared_awake_time: u32,
    // Time intervals, in seconds before expected time of going into
    // sleep, to announce the approaching sleep via a D-Bus signal.
    // Only signal intervals that are still relevant at the time when
    // waketimed realizes that sleep is approaching will be sent. See
    // `stayup_cleared_awake_time` setting.
    #[serde(default = "default_sleep_approaching_signal_intervals")]
    pub sleep_approaching_signal_intervals: Vec<u32>,
}

impl Config {
    pub fn state_dir(&self) -> PathBuf {
        PathBuf::from(&self.state_dir)
    }

    pub fn local_rule_def_dir(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.state_dir);
        path.push("rule_def");
        path
    }

    pub fn local_var_def_dir(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.state_dir);
        path.push("var_def");
        path
    }

    pub fn rule_state_dir(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.state_dir);
        path.push("rule_state");
        path
    }

    pub fn dist_rule_def_dir(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.dist_dir);
        path.push("rule_def");
        path
    }

    pub fn dist_var_def_dir(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.dist_dir);
        path.push("var_def");
        path
    }

    pub fn var_def_dirs(&self) -> Vec<PathBuf> {
        vec![self.dist_var_def_dir(), self.local_var_def_dir()]
    }
}

pub fn load() -> Result<(), AnyError> {
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
    repair_config(&mut cfg)?;

    unsafe {
        CONFIG = Some(Rc::new(RefCell::new(cfg)));
    }
    Ok(())
}

pub fn get_config() -> Rc<RefCell<Config>> {
    unsafe {
        CONFIG
            .clone()
            .expect("Called get_config but config doesn't exist yet.")
    }
}

pub fn log_config() -> Result<(), AnyError> {
    let cfg = get_config();
    debug!(
        "Config settings:\n{}",
        serde_yaml::to_string::<Config>(&cfg.borrow())?
    );
    Ok(())
}

fn populate_config_from_env(cfg: &mut Config) -> Result<(), AnyError> {
    if let Ok(value) = env::var("WAKETIMED_LOG") {
        cfg.log = value;
    }
    if let Ok(value) = env::var("WAKETIMED_STARTUP_AWAKE_TIME") {
        cfg.startup_awake_time = value.parse::<u32>()?;
    }
    if let Ok(value) = env::var("WAKETIMED_MINIMUM_AWAKE_TIME") {
        cfg.minimum_awake_time = value.parse::<u32>()?;
    }
    if let Ok(value) = env::var("WAKETIMED_STAYUP_CLEARED_AWAKE_TIME") {
        cfg.stayup_cleared_awake_time = value.parse::<u32>()?;
    }
    if let Ok(value) = env::var("WAKETIMED_POLL_VARIABLE_INTERVAL") {
        cfg.poll_variable_interval = value.parse::<u64>()?;
    }
    if let Ok(value) = env::var("WAKETIMED_SLEEP_APPROACHING_SIGNAL_INTERVALS") {
        let mut intervals: Vec<u32> = vec![];
        for interval_str in value.split(',') {
            let interval = interval_str.parse::<u32>()?;
            intervals.push(interval);
        }
        cfg.sleep_approaching_signal_intervals = intervals;
    }
    if let Ok(value) = env::var("WAKETIMED_STATE_DIR") {
        cfg.state_dir = value;
    }
    if let Ok(value) = env::var("WAKETIMED_DIST_DIR") {
        cfg.dist_dir = value;
    }
    Ok(())
}

fn repair_config(cfg: &mut Config) -> Result<(), AnyError> {
    cfg.sleep_approaching_signal_intervals
        .sort_by(|a, b| b.cmp(a)); // descending ordering
    cfg.sleep_approaching_signal_intervals.dedup();
    Ok(())
}

fn default_log() -> String {
    "info".to_string()
}

fn default_startup_awake_time() -> u32 {
    300
}

fn default_minimum_awake_time() -> u32 {
    10
}

fn default_stayup_cleared_awake_time() -> u32 {
    10
}

fn default_poll_variable_interval() -> u64 {
    3000
}

fn default_sleep_approaching_signal_intervals() -> Vec<u32> {
    vec![10, 5, 4, 3, 2, 1]
}

fn default_state_dir() -> String {
    "/var/lib/waketimed".to_string()
}

fn default_dist_dir() -> String {
    "/usr/lib/waketimed".to_string()
}
