## dsastress - DSA Stress Tester CLI

`dsastress` is a small Rust-based command-line tool for **stress-testing data structures and algorithms (DSA) / competitive programming solutions**.

It repeatedly:

- **Generates random tests** using a generator program
- **Runs your solution** on each test
- Optionally **runs a reference / brute-force solution**
- **Compares outputs** and stops on the first mismatch (unless you tell it to keep going)

This is the classic competitive programming "stress testing" workflow, packaged as a reusable CLI.

---

### Installation

You can install the tool with Cargo:

```bash
cargo install --path .
```

After that, the `dsastress` command will be available in your PATH (depending on your Cargo configuration).

---

### Basic Usage

You need:

- A **generator** program (`gen.py`) that prints random valid test input to stdout.
- Your **solution** program (`sol.py`) that reads from stdin and prints the answer.
- Optionally, a **reference** or **brute-force** solution (`brute.py`) that is known to be correct but may be slower.

Example (Python):

```bash
dsastress \
  --generator "python gen.py" \
  --solution "python sol.py" \
  --reference "python brute.py" \
  --tests 1000 \
  --time-limit-ms 5000 \
  --seed 12345 \
  --minimize \
  --save-dir failing_cases
```

If your solution ever disagrees with the reference, `dsastress`:

- Prints the **failing input**
- Prints the **expected output** (from the reference)
- Prints the **output from your solution**
- Stops immediately (unless `--keep-going` is set)

---

### Arguments

- **`-g, --generator <CMD>`**  
  Command to generate random test input.  
  Example: `--generator "python gen.py"`

- **`-s, --solution <CMD>`**  
  Command for your solution under test.  
  Example: `--solution "python my_solution.py"`

- **`-r, --reference <CMD>`** (optional)  
  Command for the reference / brute-force solution.  
  Example: `--reference "python brute.py"`  
  If omitted, the tool only checks that your solution does not **crash** or **time out**.

- **`-n, --tests <N>`** (default: `1000`)  
  Number of tests to run.

- **`--time-limit-ms <MS>`** (default: `5000`)  
  Time limit per command in milliseconds.  
  This applies separately to:
  - The generator
  - Your solution
  - The reference solution

- **`--keep-going`**  
  Continue running tests even after a mismatch or failure.  
  By default, the tool **stops at the first error** to make debugging easier.

- **`-v, --verbose`**  
  Print more detailed logs (e.g. per-test progress).

- **`--seed <U64>`**  
  Base seed for reproducibility. The tool exports:
  - `DSASTRESS_SEED=<seed>`
  - `DSASTRESS_TEST=<test_index>`
  
  If your generator reads these, you can reproduce failures exactly.

- **`--minimize`**  
  Automatically tries to **minimize the failing input** (ddmin-style) so you get the smallest repro.

- **`--minimize-mode <lines|tokens>`** (default: `lines`)  
  Minimization strategy. `lines` is safer (keeps formatting); `tokens` is more aggressive.

- **`--minimize-time-ms <MS>`** (default: `10000`)  
  Time budget per failure for minimization.

- **`--save-dir <DIR>`**  
  Save failing cases into numbered folders with `input.txt`, `expected.txt`, `got.txt`, and stderr (when available).

- **`--no-save-failing`**  
  If you set `--save-dir`, this disables saving artifacts.

- **`--input-file <PATH>`**  
  Replay a specific testcase from a file (skips the generator). This is useful to reproduce a saved `input.txt` from `--save-dir`.

---

### Typical Workflow

1. Write a random **generator** that covers tricky edge cases (small and large sizes, random shapes, etc.).
2. Write a simple but obviously correct **brute-force** solution.
3. Implement your **optimized** solution.
4. Run:

```bash
dsastress -g "python gen.py" -s "python sol_fast.py" -r "python sol_slow.py" -n 10000
```

5. If a mismatch occurs, inspect the printed input and outputs to fix the bug.

---

### Example: Codeforces “Stable Groups”

In `examples/stable_groups/` you’ll find a complete setup for the Codeforces problem **C. Stable Groups**:

- `gen.py` — generator for random testcases (small `n`, `k`, `x` for brute-force).
- `brute.py` — subset-based brute-force that tries all ways of merging/splitting groups within `k` invites.
- `fast.py` — the standard greedy solution: sort levels, compute “big” gaps, compute required invites for each gap, then greedily bridge the cheapest gaps until you run out of `k`.

To stress-test your own implementation of this problem:

```bash
dsastress \
  --generator "python3 examples/stable_groups/gen.py" \
  --solution  "python3 my_stable_groups_fast.py" \
  --reference "python3 examples/stable_groups/brute.py" \
  --tests 10000 \
  --time-limit-ms 5000
```

This is the “optimistic” workflow: you assume your fast CF-style solution is correct, then let `dsastress` hammer it with thousands of random tests to prove it.

---

### Example: Same test in Rust + C++

In `examples/pair_sum/` you’ll find the same tiny problem implemented as:

- `fast.rs` / `brute.rs`
- `fast.cpp` / `brute.cpp`

along with a reproducible generator `gen.py` that reads `DSASTRESS_SEED` and `DSASTRESS_TEST`.

---

### Notes

- The commands you provide are run via the system shell (`sh -c` on Unix, `cmd /C` on Windows).
- Input is passed via **stdin**, and only **stdout** is compared between reference and solution (after trimming trailing whitespace).
- Stderr from failing commands is printed to help with debugging.

#### Reproducible generators

To make your generator reproducible, read `DSASTRESS_SEED` and `DSASTRESS_TEST`.

Example (Python):

```python
import os, random

seed = int(os.environ.get("DSASTRESS_SEED", "0"))
t = int(os.environ.get("DSASTRESS_TEST", "0"))
random.seed((seed << 20) ^ t)
```

This tool is intentionally minimal and focused on being **easy to drop into any DSA / competitive programming workflow**.

#### Replay a saved failing case

If you saved a failure to `failing_cases/case_000123/input.txt`, you can replay it directly:

```bash
dsastress \
  --generator "python gen.py" \
  --solution  "python sol.py" \
  --reference "python brute.py" \
  --input-file "failing_cases/case_000123/input.txt"
```

