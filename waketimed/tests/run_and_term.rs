use anyhow::{Context, Error as AnyError};

mod helpers;

/// Test basic run-and-terminate scenario: the daemon starts, loads
/// data definitions, the engine enters running state, the daemon
/// receives SIGTERM and terminates safely.
#[test]
fn test_run_and_term() -> Result<(), AnyError> {
    let mut cmd = helpers::waketimed_command();
    cmd.env("WAKETIMED_CONFIG", "tests/data/run_and_term/config.yaml");
    let wtd_proc = cmd.spawn().context("Failed to spawn waketimed process.")?;
    let mut supervisor = helpers::Supervisor::new(wtd_proc);
    supervisor.wait_for_stderr_unordered(&[
        "waketimed] Starting D-Bus thread.",
        "waketimed] Starting signal thread.",
        "waketimed] Starting worker thread.",
        "Using var_def directories: [\"tests/data/run_and_term/dist/var_def\", \"tests/data/run_and_term/state/var_def\"].",
    ])?;
    supervisor.wait_for_stderr_unordered(&[
        "var_manager] Var 'test_const_true' is active.",
        "var_manager] Var 'test_inactive' is inactive, forgetting it.",
    ])?;
    supervisor.wait_for_stderr("Engine entering state 'Running'.")?;
    supervisor.terminate()?;
    supervisor.wait_for_stderr_unordered(&[
        "waketimed] Joining D-Bus thread.",
        "waketimed] Joining signal thread.",
        "waketimed] Joining worker thread.",
    ])?;
    supervisor.wait_for_stderr("waketimed] Terminating main thread.")?;
    supervisor.assert_success()?;
    Ok(())
}
