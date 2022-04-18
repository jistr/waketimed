use anyhow::{Context, Error as AnyError};

mod helpers;

#[test]
fn test_run_and_term() -> Result<(), AnyError> {
    let mut cmd = helpers::waketimed_command();
    cmd.env("WAKETIMED_CONFIG", "tests/data/run_and_term/config.yaml");
    let wtd_proc = cmd.spawn().context("Failed to spawn waketimed process.")?;
    let mut supervisor = helpers::Supervisor::new(wtd_proc);
    // Start 3 threads.
    for _ in 0..3 {
        supervisor.wait_for_stderr_ms(2000, "waketimed] Starting")?;
    }
    supervisor.wait_for_stderr_ms(2000, "Engine entering state 'Running'.")?;
    supervisor.terminate()?;
    // Join 3 threads.
    for _ in 0..3 {
        supervisor.wait_for_stderr_ms(2000, "waketimed] Joining")?;
    }
    supervisor.wait_for_stderr_ms(2000, "waketimed] Terminating main thread.")?;
    supervisor.assert_success()?;
    Ok(())
}
