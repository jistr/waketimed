use anyhow::{anyhow, Context, Error as AnyError};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::collections::HashSet;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

const DEFAULT_OUTPUT_WAIT_MS: u64 = 3000;

pub fn waketimed_command() -> Command {
    let mut cmd_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    cmd_path.push("../target/debug/waketimed");
    let mut cmd = Command::new(&cmd_path);
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("WAKETIMED_BUS_ADDRESS", env!("WAKETIMED_BUS_ADDRESS"))
        .env("WAKETIMED_TEST_MODE", "true")
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());
    cmd
}

pub struct Supervisor {
    pid: u32,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    stderr: Arc<Mutex<BufReader<ChildStderr>>>,
    child: Option<Child>,
}

impl Supervisor {
    pub fn new(mut wtd_proc: Child) -> Self {
        Self {
            pid: wtd_proc.id(),
            stdout: Arc::new(Mutex::new(BufReader::new(
                wtd_proc
                    .stdout
                    .take()
                    .expect("Cannot get stdout of waketimed."),
            ))),
            stderr: Arc::new(Mutex::new(BufReader::new(
                wtd_proc
                    .stderr
                    .take()
                    .expect("Cannot get stderr of waketimed."),
            ))),
            child: Some(wtd_proc),
        }
    }

    pub fn terminate(&mut self) -> Result<(), AnyError> {
        signal::kill(Pid::from_raw(self.pid as i32), Signal::SIGTERM)
            .context("Failed to terminate waketimed process.")
    }

    pub fn assert_success(&mut self) -> Result<(), AnyError> {
        let mut child = self
            .child
            .take()
            .expect("Called assert_success but child process has already been consumed.");
        let rc_result = self
            .wait_upto_ms_or_kill(3000, move || child.wait())
            .context("Failed waiting for waketimed to terminate.")?;
        match rc_result {
            Ok(rc) => {
                if rc.success() {
                    self.dump_stdout_and_stderr();
                    Ok(())
                } else {
                    self.dump_stdout_and_stderr();
                    Err(anyhow!("Waketimed failed with return code {0}.", rc))
                }
            }
            Err(e) => Err(e).context("Could not query waketimed process return status."),
        }
    }

    pub fn wait_for_stderr<S: AsRef<str> + Debug>(&mut self, substr: S) -> Result<(), AnyError> {
        self.wait_for_stderr_ms(DEFAULT_OUTPUT_WAIT_MS, substr)
    }

    pub fn wait_for_stderr_unordered<S: AsRef<str> + Debug>(
        &mut self,
        substrs: &[S],
    ) -> Result<(), AnyError> {
        self.wait_for_stderr_unordered_ms(DEFAULT_OUTPUT_WAIT_MS, substrs)
    }

    pub fn wait_for_stderr_ms<S: AsRef<str> + Debug>(
        &mut self,
        timeout: u64,
        substr: S,
    ) -> Result<(), AnyError> {
        let substrs = vec![substr];
        self.wait_for_stderr_unordered_ms(timeout, &substrs)
    }

    pub fn wait_for_stderr_unordered_ms<S: AsRef<str> + Debug>(
        &mut self,
        timeout: u64,
        substrs: &[S],
    ) -> Result<(), AnyError> {
        let stderr = Arc::clone(&self.stderr);
        let mut substrs_set: HashSet<String> =
            substrs.iter().map(|s| s.as_ref().to_string()).collect();
        self.wait_upto_ms_or_kill(timeout, move || {
            let mut line_buf = String::new();
            loop {
                stderr
                    .lock()
                    .expect("Failed to lock stderr in wait_for_stderr_ms")
                    .read_line(&mut line_buf)
                    .expect("Failed to read line from stderr.");
                print!("{}", &line_buf);
                let mut found = None;
                for substr in substrs_set.iter() {
                    if line_buf.contains(substr.as_str()) {
                        found = Some(substr.clone());
                        break;
                    }
                }
                if let Some(substr) = found {
                    substrs_set.remove(&substr);
                }
                if substrs_set.is_empty() {
                    break;
                }
                line_buf.clear();
            }
        })
        .with_context(|| format!("Failed waiting for stderr substrings {:?}", substrs))
    }

    pub fn wait_upto_ms_or_kill<R: 'static, F: 'static>(
        &mut self,
        timeout: u64,
        func: F,
    ) -> Result<R, AnyError>
    where
        R: Send,
        F: FnOnce() -> R + Send,
    {
        wait_upto_ms(timeout, func).or_else(|e| {
            signal::kill(Pid::from_raw(self.pid as i32), Signal::SIGKILL)
                .expect("Failed to kill waketimed process.");
            self.dump_stdout_and_stderr();
            Err(e).context("Waketimed expectation timed out and waketimed was killed.")
        })
    }

    fn dump_stdout_and_stderr(&mut self) {
        let mut out = String::new();
        self.stdout
            .lock()
            .expect("Failed to lock stdout.")
            .read_to_string(&mut out)
            .expect("Failed reading stdout to string.");
        println!("Remaining stdout: {}", &out);
        self.stderr
            .lock()
            .expect("Failed to lock stderr.")
            .read_to_string(&mut out)
            .expect("Failed reading stderr to string.");
        println!("Remaining stderr: {}", &out);
    }
}

pub fn wait_upto_ms<R: 'static, F: 'static>(timeout: u64, func: F) -> Result<R, AnyError>
where
    R: Send,
    F: FnOnce() -> R + Send,
{
    #[allow(clippy::mutex_atomic)]
    let finished_setter = Arc::new((Mutex::new(false), Condvar::new()));
    let finished_waiter = (&finished_setter).clone();
    let join_handle = thread::spawn(move || {
        let res: R = func();
        let (s_lock, s_cvar) = &*finished_setter;
        *s_lock.lock().unwrap() = true;
        s_cvar.notify_one();
        res
    });

    let (w_lock, w_cvar) = &*finished_waiter;
    let finished_wait = w_lock.lock().unwrap();
    let wait_result = w_cvar
        .wait_timeout(finished_wait, Duration::from_millis(timeout))
        .unwrap();
    let finished = wait_result.0;
    if *finished {
        Ok(join_handle.join().expect("Failed to join timeout thread."))
    } else {
        Err(anyhow!("Timed out."))
    }
}
