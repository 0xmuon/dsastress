## Pair Sum (same problem in Python/Rust/C++)

Problem:

- Input: `n K` and an array `a` (sorted)
- Output: number of pairs `(i<j)` such that `a[i] + a[j] == K`

Files:

- `gen.py` (reproducible via `DSASTRESS_SEED` and `DSASTRESS_TEST`)
- `brute.rs` / `fast.rs`
- `brute.cpp` / `fast.cpp`

### Run (Rust)

From repo root:

```bash
rustc -O examples\pair_sum\fast.rs  -o examples\pair_sum\fast_rs.exe
rustc -O examples\pair_sum\brute.rs -o examples\pair_sum\brute_rs.exe

cargo run -- \
  --generator "py -3 examples/pair_sum/gen.py" \
  --solution  ".\\examples\\pair_sum\\fast_rs.exe" \
  --reference ".\\examples\\pair_sum\\brute_rs.exe" \
  --tests 10000 \
  --seed 12345 \
  --minimize \
  --save-dir failing_cases
```

### Run (C++)

From repo root (MSVC `cl` example):

```bat
cl /O2 /EHsc examples\pair_sum\fast.cpp  /Fe:examples\pair_sum\fast_cpp.exe
cl /O2 /EHsc examples\pair_sum\brute.cpp /Fe:examples\pair_sum\brute_cpp.exe

cargo run -- \
  --generator "py -3 examples/pair_sum/gen.py" \
  --solution  ".\\examples\\pair_sum\\fast_cpp.exe" \
  --reference ".\\examples\\pair_sum\\brute_cpp.exe" \
  --tests 10000 \
  --seed 12345 \
  --minimize
```

