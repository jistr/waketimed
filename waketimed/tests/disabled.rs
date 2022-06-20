use anyhow::{Context, Error as AnyError};
use std::path::Path;

mod helpers;

/// This is a 'disabled' variant of 'minimal' scenario. The daemon
/// starts, discovers that the chassis type is not among the allowed
/// ones, enters Disabled state and waits for SIGTERM.
#[test]
fn test_disabled() -> Result<(), AnyError> {
    // Assert that /etc/waketimed/config.yaml does not exist, it would
    // interfere with the test.
    assert!(!Path::new("/etc/waketimed/config.yaml").exists());

    let mut cmd = helpers::waketimed_command();
    cmd.env("WAKETIMED_LOG", "waketimed=trace");
    // Dist dir has to exist but its subdirs do not have to exist.
    cmd.env("WAKETIMED_DIST_DIR", "tests/data/minimal/dist");
    // State dir has to exist, its subdirs do not have to exist, some
    // of which may be auto-created on startup.
    cmd.env("WAKETIMED_STATE_DIR", "tests/data/minimal/state");
    cmd.env("WAKETIMED_ALLOWED_CHASSIS_TYPES", "");
    let wtd_proc = cmd.spawn().context("Failed to spawn waketimed process.")?;
    let mut supervisor = helpers::Supervisor::new(wtd_proc);
    supervisor.wait_for_stderr_unordered(&[
        "waketimed] Starting signal thread.",
        "waketimed] Starting worker thread.",
    ])?;
    supervisor.wait_for_stderr("not in the list of configured allowed chassis types")?;
    supervisor.wait_for_stderr("Engine entering state 'Disabled'.")?;
    supervisor.terminate()?;
    supervisor.wait_for_stderr_unordered(&[
        "waketimed] Joining signal thread.",
        "waketimed] Joining worker thread.",
    ])?;
    supervisor.wait_for_stderr("waketimed] Terminating main thread.")?;
    supervisor.assert_success()?;
    Ok(())
}
