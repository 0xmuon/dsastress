use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub enum MinimizeMode {
    Lines,
    Tokens,
}

#[derive(Debug, Clone)]
pub struct MinimizeConfig {
    pub mode: MinimizeMode,
    pub time_limit: Duration,
    pub max_rounds: usize,
}

impl Default for MinimizeConfig {
    fn default() -> Self {
        Self {
            mode: MinimizeMode::Lines,
            time_limit: Duration::from_secs(10),
            max_rounds: 10_000,
        }
    }
}

fn split_units(mode: MinimizeMode, input: &[u8]) -> Vec<Vec<u8>> {
    match mode {
        MinimizeMode::Lines => {
            // Keep '\n' delimiters to preserve formatting.
            // If input has no trailing newline, keep last fragment as-is.
            let mut units = Vec::new();
            let mut start = 0usize;
            for (i, &b) in input.iter().enumerate() {
                if b == b'\n' {
                    units.push(input[start..=i].to_vec());
                    start = i + 1;
                }
            }
            if start < input.len() {
                units.push(input[start..].to_vec());
            }
            if units.is_empty() {
                units.push(Vec::new());
            }
            units
        }
        MinimizeMode::Tokens => {
            // Tokenize by ASCII whitespace; rebuild with single spaces/newlines not preserved.
            // This is more aggressive but still usually fine for CP inputs.
            let s = String::from_utf8_lossy(input);
            let toks: Vec<&str> = s.split_whitespace().collect();
            if toks.is_empty() {
                return vec![Vec::new()];
            }
            toks.into_iter().map(|t| t.as_bytes().to_vec()).collect()
        }
    }
}

fn join_units(mode: MinimizeMode, units: &[Vec<u8>]) -> Vec<u8> {
    match mode {
        MinimizeMode::Lines => units.concat(),
        MinimizeMode::Tokens => {
            let mut out = Vec::new();
            for (i, u) in units.iter().enumerate() {
                if i > 0 {
                    out.push(b' ');
                }
                out.extend_from_slice(u);
            }
            out.push(b'\n');
            out
        }
    }
}

/// A classic `ddmin`-style reducer.
///
/// `is_interesting(input)` must be deterministic for best results.
pub fn ddmin<F>(input: &[u8], cfg: &MinimizeConfig, mut is_interesting: F) -> Vec<u8>
where
    F: FnMut(&[u8]) -> bool,
{
    if !is_interesting(input) {
        return input.to_vec();
    }

    let start = Instant::now();
    let mut units = split_units(cfg.mode, input);
    let mut n = 2usize;
    let mut rounds = 0usize;

    while units.len() >= 2 && start.elapsed() < cfg.time_limit && rounds < cfg.max_rounds {
        rounds += 1;

        let len = units.len();
        let chunk = len.div_ceil(n);
        if chunk == 0 {
            break;
        }

        let mut reduced = false;
        let mut i = 0usize;
        while i < len && start.elapsed() < cfg.time_limit && rounds < cfg.max_rounds {
            rounds += 1;

            let from = i;
            let to = usize::min(i + chunk, len);

            // Try removing this chunk (complement).
            let mut candidate = Vec::with_capacity(len - (to - from));
            candidate.extend_from_slice(&units[..from]);
            candidate.extend_from_slice(&units[to..]);
            let cand_input = join_units(cfg.mode, &candidate);

            if is_interesting(&cand_input) {
                units = candidate;
                n = 2;
                reduced = true;
                break;
            }

            i = to;
        }

        if reduced {
            continue;
        }

        if n >= units.len() {
            break;
        }
        n = usize::min(units.len(), n * 2);
    }

    join_units(cfg.mode, &units)
}
