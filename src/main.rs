use std::time::Duration;

use anyhow::Result;
use clap::{ArgAction, Parser};
use dsastress::minimize::MinimizeMode;
use dsastress::runner::{run, RunConfig};

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
    #[arg(long = "time-limit-ms", default_value_t = 5000)]
    time_limit_ms: u64,

    /// Continue running all tests even after a mismatch.
    #[arg(long = "keep-going", action = ArgAction::SetTrue)]
    keep_going: bool,

    /// Print more detailed logs for each test.
    #[arg(short = 'v', long = "verbose", action = ArgAction::SetTrue)]
    verbose: bool,

    /// Base seed for reproducibility.
    ///
    /// The tool sets environment variables:
    /// - DSASTRESS_SEED=<seed>
    /// - DSASTRESS_TEST=<test_index>
    ///
    /// Update your generator to read these to become reproducible.
    #[arg(long = "seed")]
    seed: Option<u64>,

    /// Try to automatically minimize failing inputs (ddmin-style).
    #[arg(long = "minimize", action = ArgAction::SetTrue)]
    minimize: bool,

    /// Minimization mode: lines (safer) or tokens (more aggressive).
    #[arg(long = "minimize-mode", default_value = "lines")]
    minimize_mode: String,

    /// Time budget for minimization per failure (milliseconds).
    #[arg(long = "minimize-time-ms", default_value_t = 10_000)]
    minimize_time_ms: u64,

    /// Directory to save failing cases (input/expected/got).
    #[arg(long = "save-dir")]
    save_dir: Option<std::path::PathBuf>,

    /// Disable saving failing cases (if --save-dir is set).
    #[arg(long = "no-save-failing", action = ArgAction::SetTrue)]
    no_save_failing: bool,

    /// Replay a specific input from a file (skips the generator).
    ///
    /// Useful to reproduce a failing `input.txt` saved by `--save-dir`.
    #[arg(long = "input-file")]
    input_file: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let minimize_mode = match cli.minimize_mode.as_str() {
        "lines" => MinimizeMode::Lines,
        "tokens" => MinimizeMode::Tokens,
        other => anyhow::bail!("invalid --minimize-mode: {other} (expected: lines|tokens)"),
    };

    let cfg = RunConfig {
        generator_cmd: cli.generator,
        solution_cmd: cli.solution,
        reference_cmd: cli.reference,
        tests: cli.tests,
        time_limit: Duration::from_millis(cli.time_limit_ms),
        keep_going: cli.keep_going,
        verbose: cli.verbose,
        seed: cli.seed,
        minimize: cli.minimize,
        minimize_mode,
        minimize_time_limit: Duration::from_millis(cli.minimize_time_ms),
        save_dir: cli.save_dir,
        save_failing: !cli.no_save_failing,
        input_file: cli.input_file,
    };

    let _summary = run(&cfg)?;
    Ok(())
}
