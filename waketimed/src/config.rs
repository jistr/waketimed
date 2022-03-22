use anyhow::{Context, Error as AnyError};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::rc::Rc;

static mut CONFIG: Option<Rc<Config>> = None;
const CONFIG_FILE_VAR: &str = "WAKETIMED_CONFIG";
const DEFAULT_CONFIG_FILE: &str = "/etc/waketimed/config.yaml";

#[derive(Debug, Default)]
pub struct Config {
    data: RefCell<ConfigData>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ConfigData {
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
    // Time between re-checking stayup conditions, in seconds. Larger
    // values mean less exact times of falling asleep, but consume
    // less CPU.
    #[serde(default = "default_stayup_rule_check_period")]
    pub stayup_rule_check_period: u32,
    // Time intervals, in seconds before expected time of going into
    // sleep, to announce the approaching sleep via a D-Bus signal.
    // Only signal intervals that are still relevant at the time when
    // waketimed realizes that sleep is approaching will be sent. See
    // `stayup_cleared_awake_time` setting.
    #[serde(default = "default_sleep_approaching_signal_intervals")]
    pub sleep_approaching_signal_intervals: Vec<u32>,
}

pub fn load() -> Result<(), AnyError> {
    let cfg_path = env::var(CONFIG_FILE_VAR).unwrap_or_else(|_| DEFAULT_CONFIG_FILE.to_string());
    let cfg_reader = File::open(&cfg_path)
        .with_context(|| format!("Failed to open config file '{}'", &cfg_path))?;
    let cfg_data: ConfigData = serde_yaml::from_reader(cfg_reader)
        .with_context(|| format!("Failed parse config file '{}'", &cfg_path))?;
    unsafe {
        CONFIG = Some(Rc::new(Config {
            data: RefCell::new(cfg_data),
        }));
    }
    Ok(())
}

pub fn get_config() -> Rc<Config> {
    unsafe {
        CONFIG.clone()
            .expect("Called config::get but config doesn't exist yet.")
    }
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

fn default_stayup_rule_check_period() -> u32 {
    3
}

fn default_sleep_approaching_signal_intervals() -> Vec<u32> {
    vec!(10, 5, 4, 3, 2, 1)
}
