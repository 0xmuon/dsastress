use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;

use crate::exec::{format_exec_error, run_with_timeout, ExecError};
use crate::minimize::{ddmin_cached, MinimizeConfig, MinimizeMode};

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub generator_cmd: String,
    pub solution_cmd: String,
    pub reference_cmd: Option<String>,
    pub tests: u64,
    pub time_limit: Duration,
    pub keep_going: bool,
    pub verbose: bool,
    pub seed: Option<u64>,
    pub minimize: bool,
    pub minimize_mode: MinimizeMode,
    pub minimize_time_limit: Duration,
    pub save_dir: Option<PathBuf>,
    pub save_failing: bool,
    pub input_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct RunSummary {
    pub tests_run: u64,
    pub failures: u64,
    pub mismatches: u64,
}

fn normalize(bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(bytes);
    let s = s.replace("\r\n", "\n");
    s.trim_end().to_string()
}

fn ensure_dir(p: &Path) -> Result<()> {
    fs::create_dir_all(p)?;
    Ok(())
}

fn write_artifact(dir: &Path, name: &str, bytes: &[u8]) -> Result<()> {
    fs::write(dir.join(name), bytes)?;
    Ok(())
}

fn interesting_mismatch(input: &[u8], cfg: &RunConfig, env: &[(&str, String)]) -> bool {
    let timeout = cfg.time_limit;
    let exp = match &cfg.reference_cmd {
        Some(r) => run_with_timeout(r, Some(input), timeout, env).ok(),
        None => return false,
    };
    let got = run_with_timeout(&cfg.solution_cmd, Some(input), timeout, env).ok();
    match (exp, got) {
        (Some(e), Some(g)) => normalize(&e.stdout) != normalize(&g.stdout),
        _ => true, // crash/timeout also "interesting" for minimization
    }
}

pub fn run(cfg: &RunConfig) -> Result<RunSummary> {
    let mut failures = 0u64;
    let mut mismatches = 0u64;
    let mut tests_run = 0u64;

    let base_seed = cfg.seed.unwrap_or_else(|| {
        // Good enough; we just want "some entropy" when not set.
        // Users should set --seed for reproducibility.
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });

    let save_root = cfg.save_dir.clone();
    if let Some(root) = &save_root {
        ensure_dir(root)?;
    }

    println!("dsastress starting...");
    println!("  generator : {}", cfg.generator_cmd);
    println!("  solution  : {}", cfg.solution_cmd);
    if let Some(ref r) = cfg.reference_cmd {
        println!("  reference : {}", r);
    } else {
        println!("  reference : (none, only checking crashes / timeouts)");
    }
    if let Some(p) = &cfg.input_file {
        println!("  input file: {}", p.display());
    } else {
        println!("  tests     : {}", cfg.tests);
    }
    println!(
        "  time limit: {} ms per command",
        cfg.time_limit.as_millis()
    );
    println!("  seed      : {}", base_seed);
    if cfg.minimize {
        let mode = match cfg.minimize_mode {
            MinimizeMode::Lines => "lines",
            MinimizeMode::Tokens => "tokens",
        };
        println!(
            "  minimize  : on ({mode}, time limit {} ms)",
            cfg.minimize_time_limit.as_millis()
        );
    } else {
        println!("  minimize  : off");
    }
    if let Some(root) = &save_root {
        println!("  save dir  : {}", root.display());
        println!("  save fail : {}", cfg.save_failing);
    }
    println!();

    let total_tests = if cfg.input_file.is_some() {
        1
    } else {
        cfg.tests
    };
    for t in 1..=total_tests {
        tests_run = t;
        if cfg.verbose || t % 100 == 0 {
            println!("Running test {t}");
        }

        let env = [
            ("DSASTRESS_SEED", base_seed.to_string()),
            ("DSASTRESS_TEST", t.to_string()),
        ];

        // 1) Generate input (or replay from file)
        let gen_out = if let Some(p) = &cfg.input_file {
            match fs::read(p) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("[TEST {t}] Failed to read input file: {e}");
                    failures += 1;
                    break;
                }
            }
        } else {
            match run_with_timeout(&cfg.generator_cmd, None, cfg.time_limit, &env) {
                Ok(out) => out.stdout,
                Err(e) => {
                    eprintln!("[TEST {t}] Generator failed: {}", format_exec_error(&e));
                    failures += 1;
                    if !cfg.keep_going {
                        break;
                    }
                    continue;
                }
            }
        };

        // 2) Run reference (optional)
        let reference_output = if let Some(ref ref_cmd) = cfg.reference_cmd {
            match run_with_timeout(ref_cmd, Some(&gen_out), cfg.time_limit, &env) {
                Ok(out) => Some(out.stdout),
                Err(e) => {
                    eprintln!("[TEST {t}] Reference failed: {}", format_exec_error(&e));
                    failures += 1;
                    println!("\nFailing input:\n{}", String::from_utf8_lossy(&gen_out));
                    if !cfg.keep_going {
                        break;
                    }
                    continue;
                }
            }
        } else {
            None
        };

        // 3) Run solution
        let sol_out =
            match run_with_timeout(&cfg.solution_cmd, Some(&gen_out), cfg.time_limit, &env) {
                Ok(out) => out.stdout,
                Err(e) => {
                    eprintln!("[TEST {t}] Solution failed: {}", format_exec_error(&e));
                    failures += 1;
                    let mut failing = gen_out.clone();
                    if cfg.minimize {
                        let mcfg = MinimizeConfig {
                            mode: cfg.minimize_mode,
                            time_limit: cfg.minimize_time_limit,
                            ..Default::default()
                        };
                        failing = ddmin_cached(&failing, &mcfg, |cand| {
                            interesting_mismatch(cand, cfg, &env)
                        });
                    }
                    println!("\nFailing input:\n{}", String::from_utf8_lossy(&failing));
                    if let (Some(root), true) = (&save_root, cfg.save_failing) {
                        let case_dir = root.join(format!("case_{t:06}"));
                        ensure_dir(&case_dir)?;
                        write_artifact(&case_dir, "input.txt", &failing)?;
                    }
                    if !cfg.keep_going {
                        break;
                    }
                    continue;
                }
            };

        // 4) Compare (if reference exists)
        if let Some(exp) = reference_output {
            if normalize(&sol_out) != normalize(&exp) {
                failures += 1;
                mismatches += 1;

                let mut failing = gen_out.clone();
                if cfg.minimize {
                    let mcfg = MinimizeConfig {
                        mode: cfg.minimize_mode,
                        time_limit: cfg.minimize_time_limit,
                        ..Default::default()
                    };
                    failing = ddmin_cached(&failing, &mcfg, |cand| {
                        interesting_mismatch(cand, cfg, &env)
                    });
                }

                // Re-run on minimized input to print correct outputs for that input.
                let (exp2, got2) = match (
                    &cfg.reference_cmd,
                    run_with_timeout(&cfg.solution_cmd, Some(&failing), cfg.time_limit, &env),
                ) {
                    (Some(r), Ok(g)) => {
                        let e = run_with_timeout(r, Some(&failing), cfg.time_limit, &env);
                        (e, Ok(g))
                    }
                    (Some(r), Err(e)) => (
                        run_with_timeout(r, Some(&failing), cfg.time_limit, &env),
                        Err(e),
                    ),
                    (None, g) => (
                        Err(ExecError::NonZero {
                            code: None,
                            stderr: "no reference".into(),
                        }),
                        g,
                    ),
                };

                println!("================= MISMATCH FOUND =================");
                println!("Test #{t}");
                println!("\nSeed: {base_seed} (DSASTRESS_TEST={t})");
                println!("\nInput:\n{}", String::from_utf8_lossy(&failing));

                match &exp2 {
                    Ok(e) => println!(
                        "\nExpected (reference):\n{}",
                        String::from_utf8_lossy(&e.stdout)
                    ),
                    Err(e) => println!(
                        "\nExpected (reference):\n<failed: {}>",
                        format_exec_error(e)
                    ),
                }
                match &got2 {
                    Ok(g) => println!("\nGot (solution):\n{}", String::from_utf8_lossy(&g.stdout)),
                    Err(e) => println!("\nGot (solution):\n<failed: {}>", format_exec_error(e)),
                }
                println!("==================================================");

                if let (Some(root), true) = (&save_root, cfg.save_failing) {
                    let case_dir = root.join(format!("case_{t:06}"));
                    ensure_dir(&case_dir)?;
                    write_artifact(&case_dir, "input.txt", &failing)?;
                    if let Ok(e) = &exp2 {
                        write_artifact(&case_dir, "expected.txt", &e.stdout)?;
                        if !e.stderr.is_empty() {
                            write_artifact(&case_dir, "reference.stderr.txt", &e.stderr)?;
                        }
                    }
                    if let Ok(g) = &got2 {
                        write_artifact(&case_dir, "got.txt", &g.stdout)?;
                        if !g.stderr.is_empty() {
                            write_artifact(&case_dir, "solution.stderr.txt", &g.stderr)?;
                        }
                    }
                }

                if !cfg.keep_going {
                    break;
                }
            }
        }
    }

    println!("\nSummary:");
    if cfg.input_file.is_some() {
        println!("  total tests requested : 1 (input-file replay)");
    } else {
        println!("  total tests requested : {}", cfg.tests);
    }
    println!("  total tests run       : {tests_run}");
    println!("  failures              : {failures}");
    if cfg.reference_cmd.is_some() {
        println!("  mismatches            : {mismatches}");
    }

    if failures == 0 {
        println!("\nResult: all tests passed.");
    } else {
        println!("\nResult: some tests failed (see logs above).");
    }

    Ok(RunSummary {
        tests_run,
        failures,
        mismatches,
    })
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn normalize_trims_trailing_whitespace_and_normalizes_crlf() {
        let input = b"hello\r\nworld\r\n\r\n";
        let out = normalize(input);
        assert_eq!(out, "hello\nworld");
    }

    #[test]
    fn normalize_preserves_internal_newlines() {
        let input = b"a\n\nb\n";
        let out = normalize(input);
        assert_eq!(out, "a\n\nb");
    }
}
