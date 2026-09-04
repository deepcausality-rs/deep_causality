<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# C3 — `Real::cbrt` and `RealField: ToPrimitive`

## Blast radius (task 3.10)

Enumerated by implementor, not by dependent count.

| Trait | Implementors | Obliged to change |
|---|---|---|
| `Real` | blanket over `Float` (`algebra/real.rs`), `Dual<T>` (`num_dual/dual/dual_number/real.rs`) | both, and both are in this workspace |
| `RealField` | blanket over `Float` (`algebra/field_real.rs`) | none |

`RealField`'s single implementor requires `Float`, and `Float: NumCast: ToPrimitive`, so the added
supertrait was already satisfied and the blanket's where-clause is unchanged. Verified: the whole
workspace compiles under `cargo check --workspace --all-targets` with zero errors and zero warnings.

Supertrait methods reach a generic bound without importing the trait. Three of the seven
restatement sites could therefore drop `use deep_causality_num::ToPrimitive` as well as the bound.

## Phase 2 — the suite against the unimplemented surface

24 tests fail with the unimplemented panic: 15 in `algebra/real_tests.rs`, 9 in
`num_dual/dual/dual_number/real_tests.rs`.

Four further tests — the `RealField`/`ToPrimitive` ones — pass at phase 2. That is correct and not
a gap in the suite: adding a supertrait already satisfied by every implementor has no body to leave
unimplemented, so there is nothing for those four to fail against.

## Phase 3 — defect audit (task 3.5)

Each defect injected into the shipped implementation, suite run, implementation restored.

| Defect | algebra failures | dual failures |
|---|---|---|
| D1 `powf(1/3)` — negatives become `NaN` (the `signed_cbrt` defect itself) | 13 | 4 |
| D2 derivative denominator 3 → 2 | — | 5 |
| D3 derivative squares `re` instead of `cbrt(re)` | — | 5 |
| D4 returns the argument unchanged | 11 | 5 |
| D5 `sqrt` instead of `cbrt` (plausible neighbour) | 14 | 7 |
| D6 normalisation dropped (denominator 1) | — | 5 |
| D7 derivative sign inverted | — | 6 |
| D8 `Float106` third widened from `f64` (the defect found below) | 4 | — |

Every class is rejected. Baseline is 0 failures.

## A defect found by the suite, not by the plan

`Float106::cbrt` computed `let third = Self::from(1.0 / 3.0)`. `1/3` was evaluated in `f64` and
then widened, so the constant's low word was zero and the Newton iteration below it was capped at
`f64` accuracy — on a type carrying roughly 106 bits.

Measured before the fix: relative error `1.665e-16` for every input tested, *identical* across
inputs, which is the signature of a constant error source rather than input-dependent rounding.
`cbrt(27)` missed the exactly-representable `3` by `1.67e-16`.

After computing the constant in double-double (`Self::from(1.0) / Self::from(3.0)`): exact on
perfect cubes, `1.02e-32` on general values.

The defect was invisible to the existing tests by construction. `double_float_tests::test_cbrt`
asserts on `result.hi()` — the high word alone, which cannot express a wrong low word — against a
`1e-10` tolerance; `double_transcendental_tests::test_cbrt` uses `1e-14`. Both are satisfied by an
`f64`-accurate answer on a type that delivers `1e-32`.

`swirling_strength_kernel` was reaching cube roots through `powf(R::from_f64(1.0 / 3.0))`, which
carries the same widening. Retiring `signed_cbrt` removes that path as well as the sign branch.

## Retirements

- **`signed_cbrt`** (`physics/kernels/fluids/coherent_structures.rs`) — removed; the two call
  sites call `cbrt` directly. 1748 physics tests pass unchanged.
- **The floor scan** (`cfd/solvers/dec/surface_force.rs`) — replaced by `floor` plus one
  `to_usize`. It existed **twice**, in `sample_velocity` and `sample_scalar`, so six silent
  substitutions rather than the three recorded in the spec. The `base` seed was only a starting
  guess — the loops converge on `floor(g)` clamped at zero from either side — so both samplers
  lost the parameter, and with it the `Vec<LatticeCell<D>>` that each function collected solely to
  produce it. The second function's collection also backed a bounds guard on the registry key;
  that guard is preserved through `num_cells(D)`. 937 CFD tests pass unchanged.
- **Seven `RealField + ToPrimitive` restatements** — removed, plus three now-unused imports.

The scan was latent rather than live: the doc described a "short bounded search" and the sample
point sits within about one cell of the seed, so it ran one or two iterations in practice. The
code carried no bound; the behaviour did.

## Verification

`cargo check --workspace --all-targets`: 0 errors, 0 warnings.
`cargo clippy --workspace --all-targets`: 0 errors, 0 warnings.
`bazel test //...`: 1274 tests pass.

## Phase 5 — mutation (task 3.11)

`num_dual/dual/dual_number/real.rs`, 137 mutants: **16 missed**, then **1**.

None of the 16 were in `cbrt`. The added code was already pinned by the phase-2 suite; the
survivors were a pre-existing gap in the rest of the `Real` implementation, with a single cause.
All 47 tests in the file used `Dual::variable`, whose ε seed is 1, and at a seed of 1
`f'(a) * self.du` and `f'(a) / self.du` are the same operation — so the seed-multiplication in
`sin`, `cos`, `sinh`, `cosh`, `tanh` and `atan2` was never exercised. Tests at a seed of 2.5 and
of 0, plus boundary cases for `clamp`, `abs` at zero, the step functions and the predicates,
killed 15 of the 16.

The last survivor was equivalent: `Dual::log10` built ten from `two + two + T::one()`, and
swapping the first operator gives `two * two + T::one()`, which is also five. An `exclude_re`
entry for it matched three mutants rather than one — the other two being killable — which is the
over-exclusion `.cargo/mutants.toml`'s header warns about and checks for with `comm`. The entry
was backed out and the constant rebuilt as `three * three + T::one()`, where no operator swap
reproduces the value. Removed by construction rather than excluded; not re-measured afterwards.

`float_106_impl.rs` was not mutation-tested. `deep_causality_num` is the most-depended-on crate in
the workspace and every mutant costs a full build and test run for it; the file's accuracy defects
are instead pinned by the reverted-fix controls in `num-test-oracles.md`.
