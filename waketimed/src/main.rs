extern crate waketimed_core as core;

mod config;

use anyhow::Error as AnyError;

pub use crate::config::get_config;

fn main() -> Result<(), AnyError> {
    config::load()?;
    setup_logger();
    config::log_config()?;
    Ok(())
}

fn setup_logger() {
    let cfg = get_config();
    env_logger::builder()
        .parse_filters(&cfg.borrow().log)
        .init();
}
