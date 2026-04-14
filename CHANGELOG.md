## Changelog

All notable changes to this project will be documented in this file.

The format is based on “Keep a Changelog”.

---

### Unreleased

- **Added**: `--seed` and deterministic replay via `DSASTRESS_SEED` / `DSASTRESS_TEST`.
- **Added**: `--minimize` ddmin-style input minimization with `--minimize-mode` and `--minimize-time-ms`.
- **Added**: `--save-dir` to save failing inputs/outputs and stderr.
- **Changed**: default `--time-limit-ms` increased to 5000ms for better Windows reliability.
- **Added**: `examples/pair_sum/` with the same problem in Rust and C++.

