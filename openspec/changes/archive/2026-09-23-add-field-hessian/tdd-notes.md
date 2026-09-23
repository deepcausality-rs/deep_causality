# TDD protocol record: field Hessian

The record the `unified-math-tdd-protocol` asks each stage to keep, one subsection per phase, with
the observed output and counts. Measured on the M3 Max benchmark machine.

## Phase 1: API-only

`DifferentiateFieldExt::hessian<R: Scalar>(&self, x: &[R; N]) -> [[R; N]; N]` landed with the body
`let _ = x; unimplemented!()`. `cargo build -p deep_causality_calculus` and
`bazel build //deep_causality_unified_math/deep_causality_calculus` both succeeded. The method is
reachable from the crate root through the existing `DifferentiateFieldExt` export; no other surface
was added.

## Phase 2: the suite, observed failing

Suite: 15 test functions appended to `tests/extensions/differentiate_ext_tests.rs`, the mirror of
`src/extensions/differentiate_ext.rs`. The file is already registered in `tests/extensions/mod.rs`
and globbed by the `extensions` `rust_test_suite`, so no registration changed.

Run against the phase-1 API:

```
cargo test -p deep_causality_calculus --test mod hessian
test result: FAILED. 0 passed; 15 failed; 0 ignored; 0 measured; 30 filtered out
```

All 15 panicked at `src/extensions/differentiate_ext.rs:70:9`, the `unimplemented!()` line. No
compile error, no panic elsewhere.

### Corner cases

| Class | Case | Test |
|---|---|---|
| Empty input | `N = 0` returns `[]` | `test_hessian_zero_inputs` |
| Single element | `N = 1`, `x³` at 2 and at −0.5 | `test_hessian_single_input` |
| Coinciding quantities | Rosenbrock at (1, 1), equal coordinates; supplemented off the minimum | `test_hessian_rosenbrock_at_minimum`, `test_hessian_rosenbrock_off_minimum` |
| Coinciding quantities | Every fixture has distinct off-diagonal entries, so a transposed or swapped index changes the answer | `test_hessian_cubic_distinct_entries`, `test_hessian_quadratic_n4_every_position` |
| Index degeneracy | `N = 2`, `3`, `4`; at `N = 4` six distinct off-diagonal positions | `test_hessian_quadratic_n4_every_position` |
| Zero entry | `H_yy = 0` of `Cubic3`, beside non-zero entries | `test_hessian_cubic_distinct_entries` |
| Negative values | Negative point, negative entries | `test_hessian_cubic_negative_point` |
| Non-finite | NaN coordinate propagates to every entry; paired with a finite point | `test_hessian_nan_input_propagates` |
| Precision | `Cubic3` at (1, 2, 3) at `f32`, `f64`, `Float106`; transcendental at `Float106` below `1e-28` | `test_hessian_precision_*` |
| Invariant | Exact bitwise symmetry | `test_hessian_is_exactly_symmetric` |
| Independent algorithm | Central differences of `gradient` | `test_hessian_agrees_with_finite_difference_of_gradient` |

Threshold class: `hessian` has no tolerance or branch on a value, so there is none.

### Independent sources

Every literal is a hand-derived closed-form second partial, written in the comment above its
fixture. `SinExp` compares against the analytic Hessian expression, not against a dual-number
computation. The finite-difference test uses a different algorithm: first-order AD plus central
differences.

### Error variants

The calculus crate declares no error type and `hessian` returns no `Result`. There are no variants
to construct.

## Phase 3: defect audit

A throwaway implementation passed all 15 tests. Each defect was applied to it alone and the suite
re-run.

| Defect | Class | Tests failed |
|---|---|---|
| D1 `j` starts at `i + 1`, diagonal skipped | off-by-one | 13 |
| D2 `i` starts at 1 | off-by-one | 13 |
| D3 inner loop `i..i+1`, diagonal only | early return | 12 |
| D4 result negated | inverted sign | 13 |
| D5 result doubled | constant factor | 13 |
| D6 result halved | constant factor | 13 |
| D7 mirror write removed | guard removed | 12, incl. `is_exactly_symmetric` |
| D8 mirror written to `[j][j]` | plausible neighbour | 12, incl. `is_exactly_symmetric` |
| D9 outer seed on `k == i` | wrong index | 12 |
| D10 inner seed on `k == j` | wrong index | 12 |
| D11 reads `.value().derivative()` (`∂f/∂xᵢ`) | plausible neighbour | 13 |
| D12 reads `.derivative().value()` (`∂f/∂xⱼ`) | plausible neighbour | 13 |
| D13 constant seed reads `x[i]` not `x[k]` | wrong index | 10 |
| D14 inner seed comparison flipped | flipped comparison | 13 |

All 14 rejected. D13 survives at Rosenbrock (1, 1), where both coordinates coincide, and is
rejected by `rosenbrock_off_minimum`, both `cubic_*` tests and the transcendental tests: the
coincidence case is already supplemented. Tolerance loosening does not apply because the
implementation holds no tolerance. No test was added.

## Phase 4: implementation

The audited throwaway became the implementation. Clippy's `needless_range_loop` then rejected the
nested index loops, so they were rewritten as an iterator over the upper-triangle pairs `(i, j)`,
with no change in behaviour.

- `cargo clippy -p deep_causality_calculus --all-targets -- -D warnings`: clean.
- `cargo test -p deep_causality_calculus`: 45 passed.
- `differentiate_ext_tests`: 25 passed under Cargo, 25 passed under
  `bazel test //deep_causality_unified_math/deep_causality_calculus:extensions_tests/extensions/differentiate_ext_tests_test`.
- `cargo llvm-cov`: `src/extensions/differentiate_ext.rs` 100 % regions, functions and lines.

## Phase 5: mutation testing

```
scripts/mutants.sh deep_causality_calculus src/extensions/differentiate_ext.rs
9 mutants tested in 6m: 3 caught, 6 unviable
```

Caught: `==` → `!=` at lines 78 and 83 of `hessian` (the two seed comparisons), and at line 48 of
`gradient`. Unviable: the six whole-body `Default::default()` replacements, because `Scalar` has no
`Default` bound. No survivors, so `.cargo/mutants.toml` is unchanged. `hessian` has no arithmetic
or comparison operators beyond the two `==`, so `cargo mutants` generates nothing for its loop
bounds, channel reads or mirror writes. The phase-3 audit covers those (D1 to D3, D7, D8, D11,
D12).
