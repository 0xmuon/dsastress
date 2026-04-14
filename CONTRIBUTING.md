## Contributing to dsastress

Thanks for contributing! This repo is meant to stay **small, reliable, and easy to use** for competitive-programming style stress testing.

---

### Ground rules

- **Understand what you submit**: Please don’t submit AI-generated code (or any copied code) that you can’t explain and maintain. If you use an AI tool to help write code, you’re responsible for correctness, edge cases, security implications, and future maintenance.
- **Keep it practical**: Prefer changes that improve reliability, reproducibility, debuggability, cross-platform behavior, or UX.
- **Small PRs are best**: Focused changes are easier to review and safer to merge.

---

### What makes a good contribution

- **New features** that are broadly useful:
  - Better failure reporting/saving
  - Deterministic replay (seed/test index)
  - Better minimization/shrinking
  - New examples (Python/Rust/C++)
  - CI improvements that catch real regressions
- **Bug fixes**:
  - Windows/Linux shell quirks
  - Timeouts / process handling
  - Output normalization issues
- **Docs**:
  - Clearer “copy/paste” commands
  - Troubleshooting notes

---

### Development setup

#### Requirements

- Rust stable toolchain (edition 2021)
- (Optional) Python 3 for running the example generators/solutions
- (Optional) C++ compiler if you work on C++ examples

#### Build + test

```bash
cargo build
cargo test
```

---

### Code formatting guidelines

Keep formatting boring and consistent. If your editor auto-formats, great—please ensure it matches these rules.

#### Rust

- **Run**:

```bash
cargo fmt
```

- **Clippy (recommended)**:

```bash
cargo clippy -- -D warnings
```

#### Python (examples)

- Prefer clear, straightforward code over clever tricks.
- If you touch Python examples, keep them **PEP8-ish** and readable.
- If you use formatters/linters locally, recommended defaults are:
  - `black`
  - `ruff`

*(These tools are not required by CI unless the repo later adds them.)*

#### C++ (examples)

- Use a consistent style: `snake_case` for variables, `UpperCamelCase` for types is fine, but be consistent within a file.
- Prefer `std::` facilities and avoid non-portable compiler extensions except `#include <bits/stdc++.h>` (acceptable for CP-style examples).
- If you use a formatter locally, recommended is `clang-format` (LLVM style or Google style—pick one and apply consistently within your change).

#### Line endings / whitespace

- Don’t add trailing whitespace.
- End files with a newline.
- Keep diffs minimal (avoid reformatting unrelated code).

---

### Reproducibility for generators

If you contribute or modify generators, please make them reproducible by reading:

- `DSASTRESS_SEED`
- `DSASTRESS_TEST`

This lets users replay failures with `--seed` and the test index printed in the logs.

---

### Submitting a change

- Update docs if behavior/CLI changes.
- Add or update an example if it helps demonstrate the feature.
- Ensure `cargo build` and `cargo test` pass.
- In your PR description, include:
  - **What** changed
  - **Why** it matters
  - **How** you tested it

