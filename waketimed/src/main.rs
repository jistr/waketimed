extern crate waketimed_core as core;

mod config;

use anyhow::Error as AnyError;

pub use crate::config::get_config;

fn main() -> Result<(), AnyError> {
    config::load()?;
    Ok(())
}
