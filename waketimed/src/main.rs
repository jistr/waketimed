extern crate waketimed_core as wd_core;

mod config;
mod dbus;

use anyhow::Error as AnyError;

pub use crate::config::get_config;

fn main() -> Result<(), AnyError> {
    config::load()?;
    setup_logger();
    config::log_config()?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()?;
    runtime.block_on(async_main())
}

fn setup_logger() {
    let cfg = get_config();
    env_logger::builder()
        .parse_filters(&cfg.borrow().log)
        .init();
}

async fn async_main() -> Result<(), AnyError> {
    dbus::server::spawn().await?;
    std::future::pending::<()>().await;
    Ok(())
}
