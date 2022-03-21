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
