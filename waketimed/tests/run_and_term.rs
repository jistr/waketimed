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
    supervisor.wait_for_stderr("Using rule_def directories: [\"tests/data/run_and_term/dist/rule_def\", \"tests/data/run_and_term/state/rule_def\"].")?;
    supervisor.wait_for_stderr(
        "Overriden rule def paths: [\"tests/data/run_and_term/dist/rule_def/test_masked.yaml\"]",
    )?;
    supervisor.wait_for_stderr_unordered(&[
        "Rule def 'tests/data/run_and_term/state/rule_def/test_masked.yaml' is void.",
    ])?;
    supervisor.wait_for_stderr("Nearest possible suspend:")?;
    supervisor.wait_for_stderr("Using var_def directories: [\"tests/data/run_and_term/dist/var_def\", \"tests/data/run_and_term/state/var_def\"].")?;
    supervisor.wait_for_stderr(
        "Overriden var def paths: [\"tests/data/run_and_term/dist/var_def/test_masked.yaml\"]",
    )?;
    supervisor.wait_for_stderr_unordered(&[
        "Var def 'tests/data/run_and_term/state/var_def/test_masked.yaml' is void.",
    ])?;
    supervisor.wait_for_stderr("Engine entering state 'Running'.")?;
    supervisor.wait_for_stderr_unordered(&[
        "ReturnVarPoll(VarName(\"test_inactive\"), None)",
        "Variable changed: test_poll_true = true",
    ])?;
    supervisor.wait_for_stderr("Variable changed: test_category = true")?;
    supervisor.wait_for_stderr_unordered(&[
        "Stayup rule changed: test_stayup_bool = true",
        "Stayup rule changed: test_is_defined_nonexistent_var = false",
        "Failed to evaluate stayup rule 'test_use_nonexistent_var'",
    ])?;
    supervisor.wait_for_stderr("Received EngineMsg::PollVarsTick.")?;
    supervisor.wait_for_stderr("Received EngineMsg::ReturnVarPoll")?;
    supervisor.wait_for_stderr("Nearest possible suspend:")?;
    supervisor.wait_for_stderr("Received EngineMsg::PollVarsTick.")?;
    supervisor.wait_for_stderr("Received EngineMsg::ReturnVarPoll")?;
    supervisor.wait_for_stderr("Nearest possible suspend:")?;
    supervisor.terminate()?;
    supervisor.wait_for_stderr_unordered(&[
        "waketimed] Joining signal thread.",
        "waketimed] Joining worker thread.",
    ])?;
    supervisor.wait_for_stderr("waketimed] Terminating main thread.")?;
    supervisor.assert_success()?;
    Ok(())
}
