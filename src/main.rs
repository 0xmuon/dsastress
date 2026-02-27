use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::Result;
use clap::{ArgAction, Parser};
use wait_timeout::ChildExt;

/// Cross-platform wrapper to run shell commands.
fn shell_command(cmd: &str) -> Command {
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

#[derive(Debug, thiserror::Error)]
enum ExecError {
    #[error("command timed out after {0:?}")]
    Timeout(Duration),
    #[error("failed to spawn command: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("command exited with non-zero code {code:?}. Stderr:\n{stderr}")]
    NonZero { code: Option<i32>, stderr: String },
}

#[derive(Debug)]
struct ExecOutput {
    stdout: Vec<u8>,
}

fn run_with_timeout(cmd: &str, input: Option<&[u8]>, timeout: Duration) -> std::result::Result<ExecOutput, ExecError> {
    let mut command = shell_command(cmd);
    command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn()?;

    if let Some(data) = input {
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(data)?;
        }
    }

    let status_opt = child
        .wait_timeout(timeout)
        .map_err(|e| ExecError::Spawn(e))?;

    let status = match status_opt {
        Some(s) => s,
        None => {
            // Timeout
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

    Ok(ExecOutput { stdout })
}

/// DSA Stress Tester CLI
#[derive(Parser, Debug)]
#[command(
    name = "dsastress",
    version,
    about = "Stress-test DSA / competitive programming solutions with random generators and reference implementations."
)]
struct Cli {
    /// Command to generate random test input.
    ///
    /// Example: "python gen.py"
    #[arg(short = 'g', long = "generator")]
    generator: String,

    /// Command for your solution under test.
    ///
    /// Example: "python my_solution.py"
    #[arg(short = 's', long = "solution")]
    solution: String,

    /// Command for the reference / brute-force solution.
    ///
    /// Example: "python brute.py"
    /// If omitted, the tool only checks that your solution does not crash / time out.
    #[arg(short = 'r', long = "reference")]
    reference: Option<String>,

    /// Number of tests to run.
    #[arg(short = 'n', long = "tests", default_value_t = 1000)]
    tests: u64,

    /// Time limit per command in milliseconds.
    ///
    /// Applies to generator, solution, and reference commands individually.
    #[arg(long = "time-limit-ms", default_value_t = 2000)]
    time_limit_ms: u64,

    /// Continue running all tests even after a mismatch.
    #[arg(long = "keep-going", action = ArgAction::SetTrue)]
    keep_going: bool,

    /// Print more detailed logs for each test.
    #[arg(short = 'v', long = "verbose", action = ArgAction::SetTrue)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let timeout = Duration::from_millis(cli.time_limit_ms);

    println!("dsastress starting...");
    println!("  generator : {}", cli.generator);
    println!("  solution  : {}", cli.solution);
    if let Some(ref r) = cli.reference {
        println!("  reference : {}", r);
    } else {
        println!("  reference : (none, only checking crashes / timeouts)");
    }
    println!("  tests     : {}", cli.tests);
    println!("  time limit: {} ms per command", cli.time_limit_ms);
    println!();

    let mut failures = 0u64;

    for t in 1..=cli.tests {
        if cli.verbose || t % 100 == 0 {
            println!("Running test {}", t);
        }

        // 1. Generate input
        let gen_out = match run_with_timeout(&cli.generator, None, timeout) {
            Ok(out) => out.stdout,
            Err(e) => {
                eprintln!("[TEST {}] Generator failed: {}", t, format_exec_error(e));
                failures += 1;
                if !cli.keep_going {
                    break;
                } else {
                    continue;
                }
            }
        };

        // 2. Run reference (if provided)
        let reference_output = if let Some(ref ref_cmd) = cli.reference {
            match run_with_timeout(ref_cmd, Some(&gen_out), timeout) {
                Ok(out) => Some(out.stdout),
                Err(e) => {
                    eprintln!(
                        "[TEST {}] Reference solution failed: {}",
                        t,
                        format_exec_error(e)
                    );
                    failures += 1;
                    if !cli.keep_going {
                        println!("\nFailing input:\n{}", String::from_utf8_lossy(&gen_out));
                        break;
                    } else {
                        println!("\nFailing input (reference failed):\n{}", String::from_utf8_lossy(&gen_out));
                        continue;
                    }
                }
            }
        } else {
            None
        };

        // 3. Run candidate solution
        let sol_out_res = run_with_timeout(&cli.solution, Some(&gen_out), timeout);
        let sol_out = match sol_out_res {
            Ok(out) => out.stdout,
            Err(e) => {
                eprintln!(
                    "[TEST {}] Solution failed: {}",
                    t,
                    format_exec_error(e)
                );
                failures += 1;
                println!("\nFailing input:\n{}", String::from_utf8_lossy(&gen_out));
                if !cli.keep_going {
                    break;
                } else {
                    continue;
                }
            }
        };

        // 4. Compare output with reference, if available.
        if let Some(exp) = reference_output {
            if normalize(&sol_out) != normalize(&exp) {
                failures += 1;
                println!("================= MISMATCH FOUND =================");
                println!("Test #{}", t);
                println!("\nInput:\n{}", String::from_utf8_lossy(&gen_out));
                println!(
                    "\nExpected (reference):\n{}",
                    String::from_utf8_lossy(&exp)
                );
                println!(
                    "\nGot (solution):\n{}",
                    String::from_utf8_lossy(&sol_out)
                );
                println!("==================================================");

                if !cli.keep_going {
                    break;
                }
            }
        }
    }

    println!("\nSummary:");
    println!("  total tests run : {}", cli.tests);
    println!("  failures        : {}", failures);

    if failures == 0 {
        println!("\nResult: all tests passed.");
    } else {
        println!("\nResult: some tests failed (see logs above).");
    }

    Ok(())
}

fn normalize(bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(bytes);
    s.trim_end().to_string()
}

fn format_exec_error(e: ExecError) -> String {
    match e {
        ExecError::Timeout(dur) => format!("timed out after {:?}", dur),
        ExecError::Spawn(err) => format!("spawn error: {}", err),
        ExecError::NonZero { code, stderr } => {
            format!("non-zero exit code {:?}, stderr:\n{}", code, stderr)
        }
    }
}

