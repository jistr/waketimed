use anyhow::{Context, Error as AnyError};

mod helpers;

/// Test basic run-and-terminate scenario: the daemon starts, loads
/// data definitions, the engine enters running state, the daemon
/// receives SIGTERM and terminates safely.
#[test]
fn test_run_with_dist() -> Result<(), AnyError> {
    let mut cmd = helpers::waketimed_command();
    cmd.env("WAKETIMED_CONFIG", "tests/data/run_with_dist/config.yaml");
    let wtd_proc = cmd.spawn().context("Failed to spawn waketimed process.")?;
    let mut supervisor = helpers::Supervisor::new(wtd_proc);
    supervisor.wait_for_stderr_unordered(&[
        "waketimed] Starting signal thread.",
        "waketimed] Starting worker thread.",
        "Using var_def directories: [\"data/dist/var_def\"].",
    ])?;
    supervisor.wait_for_stderr_unordered(&[
        "Loading var def 'data/dist/var_def/wtd_user_busy.yaml'.",
        "Loading var def 'data/dist/var_def/wtd_login_seat_busy.yaml'.",
    ])?;
    supervisor.wait_for_stderr("Engine entering state 'Running'.")?;
    supervisor.wait_for_stderr("Received EngineMsg::PollVarsTick.")?;
    supervisor.wait_for_stderr("Received EngineMsg::ReturnVarPoll")?;
    supervisor.wait_for_stderr("Received EngineMsg::PollVarsTick.")?;
    supervisor.wait_for_stderr("Received EngineMsg::ReturnVarPoll")?;
    supervisor.terminate()?;
    supervisor.wait_for_stderr_unordered(&[
        "waketimed] Joining signal thread.",
        "waketimed] Joining worker thread.",
    ])?;
    supervisor.wait_for_stderr("waketimed] Terminating main thread.")?;
    supervisor.assert_success()?;
    Ok(())
}
