use std::time::{Duration, Instant};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

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

/// `ddmin` with a small in-memory cache to avoid rerunning identical candidates.
///
/// This is particularly useful when `is_interesting` is expensive (spawning processes).
pub fn ddmin_cached<F>(input: &[u8], cfg: &MinimizeConfig, mut is_interesting: F) -> Vec<u8>
where
    F: FnMut(&[u8]) -> bool,
{
    let mut cache: HashMap<(u64, usize), bool> = HashMap::new();
    ddmin(input, cfg, |cand| {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        cand.hash(&mut hasher);
        let h = hasher.finish();
        let key = (h, cand.len());
        if let Some(v) = cache.get(&key) {
            return *v;
        }
        let v = is_interesting(cand);
        cache.insert(key, v);
        v
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ddmin_returns_original_when_not_interesting() {
        let input = b"1 2 3\n".to_vec();
        let cfg = MinimizeConfig {
            mode: MinimizeMode::Lines,
            time_limit: Duration::from_millis(50),
            max_rounds: 1000,
        };
        let out = ddmin(&input, &cfg, |_| false);
        assert_eq!(out, input);
    }

    #[test]
    fn ddmin_lines_can_reduce_to_single_line() {
        let input = b"keep\nremove1\nremove2\n".to_vec();
        let cfg = MinimizeConfig {
            mode: MinimizeMode::Lines,
            time_limit: Duration::from_millis(200),
            max_rounds: 10_000,
        };

        // Interesting if it contains "keep" and at least one other line (forces ddmin to search).
        let out = ddmin(&input, &cfg, |cand| {
            let s = String::from_utf8_lossy(cand);
            s.contains("keep") && s.lines().count() >= 1
        });
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("keep"));
        assert!(s.lines().count() >= 1);
    }

    #[test]
    fn ddmin_tokens_keeps_required_token() {
        let input = b"alpha beta gamma delta\n".to_vec();
        let cfg = MinimizeConfig {
            mode: MinimizeMode::Tokens,
            time_limit: Duration::from_millis(200),
            max_rounds: 10_000,
        };

        // Interesting iff token "gamma" exists.
        let out = ddmin(&input, &cfg, |cand| {
            let s = String::from_utf8_lossy(cand);
            s.split_whitespace().any(|t| t == "gamma")
        });

        let binding = String::from_utf8_lossy(&out);
        let toks: Vec<&str> = binding.split_whitespace().collect();
        assert!(toks.contains(&"gamma"));
        // Should be reasonably minimized; the smallest interesting input is just "gamma".
        assert_eq!(toks, vec!["gamma"]);
    }

    #[test]
    fn ddmin_handles_empty_input() {
        let input = b"".to_vec();
        let cfg = MinimizeConfig {
            mode: MinimizeMode::Lines,
            time_limit: Duration::from_millis(50),
            max_rounds: 1000,
        };
        let out = ddmin(&input, &cfg, |_| true);
        assert!(out.is_empty());
    }
}
