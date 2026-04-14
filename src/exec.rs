use std::io::{Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use wait_timeout::ChildExt;

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("command timed out after {0:?}")]
    Timeout(Duration),
    #[error("failed to spawn command: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("command exited with non-zero code {code:?}. Stderr:\n{stderr}")]
    NonZero { code: Option<i32>, stderr: String },
}

#[derive(Debug, Clone)]
pub struct ExecOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub status: ExitStatus,
    pub elapsed: Duration,
}

/// Cross-platform wrapper to run shell commands.
pub fn shell_command(cmd: &str) -> Command {
    #[cfg(windows)]
    {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(cmd);
        c
    }

    #[cfg(not(windows))]
    {
        let mut c = Command::new("sh");
        c.arg("-c").arg(cmd);
        c
    }
}

pub fn run_with_timeout(
    cmd: &str,
    input: Option<&[u8]>,
    timeout: Duration,
    extra_env: &[(&str, String)],
) -> std::result::Result<ExecOutput, ExecError> {
    let start = Instant::now();

    let mut command = shell_command(cmd);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in extra_env {
        command.env(k, v);
    }

    let mut child = command.spawn()?;

    if let Some(data) = input {
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(data)?;
        }
    }

    let status_opt = child.wait_timeout(timeout).map_err(ExecError::Spawn)?;
    let status = match status_opt {
        Some(s) => s,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ExecError::Timeout(timeout));
        }
    };

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_end(&mut stdout);
    }
    if let Some(mut err) = child.stderr.take() {
        let _ = err.read_to_end(&mut stderr);
    }

    if !status.success() {
        return Err(ExecError::NonZero {
            code: status.code(),
            stderr: String::from_utf8_lossy(&stderr).to_string(),
        });
    }

    Ok(ExecOutput {
        stdout,
        stderr,
        status,
        elapsed: start.elapsed(),
    })
}

pub fn format_exec_error(e: &ExecError) -> String {
    match e {
        ExecError::Timeout(dur) => format!("timed out after {:?}", dur),
        ExecError::Spawn(err) => format!("spawn error: {err}"),
        ExecError::NonZero { code, stderr } => {
            format!("non-zero exit code {code:?}, stderr:\n{stderr}")
        }
    }
}
