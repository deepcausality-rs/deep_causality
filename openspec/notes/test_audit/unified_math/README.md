<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Unified-math test-suite audit

This report applies the physics test-audit scanner to every crate directly under
`deep_causality_unified_math/`. It is an inventory and a calibrated review, not a proposal and not
a claim that every syntactic match is a defective test.

The existing [simplicial Hodge-star note](topology/simplicial_hodge_star/README.md) remains a
separate, specific implementation finding. It did not audit the rest of the topology suite. The
topology row below covers that crate's complete `tests/` tree.

## Finding in one line

**The weakness seen in physics is widespread, but uneven: the scanner found 1,222 unexplained
single-point literal oracles and 192 potentially circular expectations among 6,824 statically
declared tests; calibration also showed that raw scanner counts are ceilings, not defect counts.**

The clearest defects are concentrated in a smaller set: an assertion-free topology test, finite-
only assertions that do not check numeric answers, branch tests whose branches are identical, and
quaternion expectations that reproduce the implementation formula.

## Scope and method

The scan covered these seventeen crates:

`algebra`, `calculus`, `fft`, `haft`, `homology`, `linear`, `metric`, `multivector`, `num`,
`num_complex`, `num_dual`, `num_rational`, `rand`, `stats`, `tensor`, `topology`, and `uncertain`.

The unchanged classification logic from `openspec/notes/test_audit/physics/audit_tests.py` was run
once per crate with that crate's `tests/` directory as its root. It inspected 680 Rust test files,
containing 125,286 lines, and recognized 6,824 explicit `#[test]` functions. Macro-generated cases
are counted as their source declaration rather than as every runtime expansion.

The classes overlap. A test can be both single-input and cherry-picked, or both circular and
cherry-picked. Their counts therefore must not be added to obtain a number of defective tests.

| Class | Raw count | Share | Scanner meaning |
|---|---:|---:|---|
| single-input | 4,792 | 70.2% | No loop or table was recognized. |
| cherry-picked | 1,222 | 17.9% | One input, literal expected value, and no recognized provenance. |
| circular | 192 | 2.8% | Arithmetic used to construct the expected value. |
| tautology | 73 | 1.1% | Only a weak predicate such as `is_ok`, `is_some`, or `is_finite` was recognized. |
| no-assertion | 46 | 0.7% | No assertion was recognized in the function body. |

## Per-crate inventory

Percentages use the explicit test-function count for that crate. The final five columns are raw,
overlapping scanner classifications.

| Crate | Test files | Tests | Single-input | Cherry-picked | Tautology | Circular | No assertion |
|---|---:|---:|---:|---:|---:|---:|---:|
| `deep_causality_algebra` | 48 | 358 | 224 (62.6%) | 66 (18.4%) | 2 (0.6%) | 0 | 22 (6.1%) |
| `deep_causality_calculus` | 6 | 30 | 28 (93.3%) | 13 (43.3%) | 0 | 6 (20.0%) | 0 |
| `deep_causality_fft` | 7 | 58 | 25 (43.1%) | 0 | 0 | 0 | 0 |
| `deep_causality_haft` | 75 | 347 | 297 (85.6%) | 3 (0.9%) | 0 | 0 | 0 |
| `deep_causality_homology` | 5 | 40 | 13 (32.5%) | 0 | 0 | 1 (2.5%) | 0 |
| `deep_causality_linear` | 44 | 725 | 548 (75.6%) | 203 (28.0%) | 0 | 17 (2.3%) | 0 |
| `deep_causality_metric` | 5 | 109 | 103 (94.5%) | 0 | 2 (1.8%) | 0 | 0 |
| `deep_causality_multivector` | 35 | 366 | 242 (66.1%) | 104 (28.4%) | 0 | 17 (4.6%) | 0 |
| `deep_causality_num` | 67 | 1,005 | 769 (76.5%) | 255 (25.4%) | 10 (1.0%) | 7 (0.7%) | 0 |
| `deep_causality_num_complex` | 58 | 532 | 456 (85.7%) | 160 (30.1%) | 20 (3.8%) | 66 (12.4%) | 0 |
| `deep_causality_num_dual` | 10 | 90 | 76 (84.4%) | 55 (61.1%) | 0 | 4 (4.4%) | 1 (1.1%) |
| `deep_causality_num_rational` | 8 | 71 | 48 (67.6%) | 0 | 0 | 1 (1.4%) | 0 |
| `deep_causality_rand` | 22 | 157 | 97 (61.8%) | 14 (8.9%) | 1 (0.6%) | 4 (2.5%) | 0 |
| `deep_causality_stats` | 19 | 466 | 36 (7.7%) | 14 (3.0%) | 0 | 14 (3.0%) | 22 (4.7%) |
| `deep_causality_tensor` | 54 | 549 | 367 (66.8%) | 58 (10.6%) | 2 (0.4%) | 14 (2.6%) | 0 |
| `deep_causality_topology` | 187 | 1,668 | 1,228 (73.6%) | 216 (12.9%) | 29 (1.7%) | 40 (2.4%) | 1 (0.1%) |
| `deep_causality_uncertain` | 30 | 253 | 235 (92.9%) | 61 (24.1%) | 7 (2.8%) | 1 (0.4%) | 0 |
| **Total** | **680** | **6,824** | **4,792 (70.2%)** | **1,222 (17.9%)** | **73 (1.1%)** | **192 (2.8%)** | **46 (0.7%)** |

## Calibrated findings

### One of the 46 no-assertion flags is real

All 46 no-assertion matches were inspected. Forty-five are scanner false positives:

- the 22 algebra tests use assertion helpers, `#[should_panic]`, or compile-time trait checks;
- the 22 stats tests call helpers such as `expect_insufficient_samples`,
  `assert_no_finite_answer`, and `check_*`, which contain the assertions;
- the one dual-number test is a compile-time type-marker check.

The remaining topology test is ineffective:

- `deep_causality_topology/tests/types/simplicial_complex/map_and_default_tests.rs`,
  `test_simplicial_complex_default_is_empty`, creates a default complex and clones it into
  `_cloned`. It asserts neither that the default is empty nor that cloning preserves any state.

This calibration matters. Reporting 46 assertion-free defects would exaggerate the result by a
factor of 46.

### Weak predicates conceal missing answer checks

Most of the 73 raw tautology matches are not tautologies. For example, an error constructor's
`is_ok()`, a floating-point type's `is_nan()`, and a chordality decision represented by
`Result<(), TopologyError>` may be the exact contract under test. The following cases do fail to
pin the claimed answer:

- `deep_causality_tensor/tests/extensions/causal_tensor_ext_stats_f64_tests.rs`,
  `gaussian_log_density_floors_zero_variance` and its negative-variance counterpart, assert only
  that the result is finite. Any incorrect finite result passes, although nearby tests calculate
  exact expected densities.
- `deep_causality_uncertain/tests/types/uncertain/uncertain_sampling_tests.rs`, `test_from_sample`,
  asserts only that sampling succeeds and never checks the sampled value.
- `deep_causality_uncertain/tests/types/sampler/sampler_seed_tests.rs`,
  `clear_seed_restores_default_and_samples`, asserts only that a sample succeeds. It does not show
  that the default seed was restored.
- `deep_causality_uncertain/tests/types/sampler/qmc_sampler_tests.rs`,
  `test_qmc_samples_conditional_both_branches`, passes the same `shared` uncertain value to both
  branches and then checks only that results are finite. It cannot distinguish branch selection.
- `deep_causality_topology/tests/types/point_cloud/point_cloud_tests.rs`,
  `test_triangulate_varying_radius`, calls two radii but checks only that both calls succeed. It
  does not inspect the complexes or show that the radius affects the result.
- `deep_causality_topology/tests/types/gauge/gauge_field_lattice/gradient_flow_tests.rs`,
  `test_try_energy_density_random_positive`, says the answer must be nonzero and finite but asserts
  only finiteness.
- Several lattice-gauge and link-variable tests assert only `is_ok()` or `is_some()` after an
  operation. The clearest is `lattice_gauge_field_tests.rs::test_set_link`, which writes the
  identity value into an identity field and then checks only that a link exists. A no-op or a wrong
  stored value survives.
- `deep_causality_algebra/tests/algebra/field_real_f64_tests.rs::test_nan` constructs
  `f64::NAN` and calls the standard library's `is_nan`; it exercises no algebra-crate behavior.

These examples justify a targeted weak-assertion repair, but not relabelling every raw predicate
match as a defect.

### Circular candidates require semantic review

Arithmetic in an expected value is not sufficient proof of circularity. The scan also catches
independent closed forms, metamorphic laws, scale equivariance, and cross-implementation checks.
Examples include the exact-rational adjugate and LU cross-checks in `deep_causality_linear`, the
analytic derivatives used to test automatic differentiation in `deep_causality_calculus`, and
closed-form scaling laws in topology and stats. Those are useful controls.

There is nevertheless a clear circular family in
`deep_causality_num_complex/tests/complex/quaternion_number/ops_tests.rs`. Tests such as
`test_sin_general` and `test_cos_general` reconstruct the scalar/vector decomposition, norm,
trigonometric factors, and component scaling used by the production implementation. The file has
24 raw circular matches. These tests can catch transcription mistakes between two copies, but not
a shared error in the chosen formula. The general quaternion multiplication test similarly
retypes the component formula; the basis identities in the same suite are the stronger independent
checks.

The 192 raw circular matches are therefore a review ceiling. The quaternion formula family is a
confirmed priority; the remaining matches should be retained when they are demonstrably
independent and rewritten only when their oracle shares the implementation's derivation.

### Single inputs and unexplained literal oracles are the broadest risk

The 4,792 single-input matches are not 4,792 defects. Singular boundary cases and compile-time
trait checks often need only one input. Likewise, many `deep_causality_num` tests compare a trait
method with the corresponding standard-library operation at one value; those are differential
checks, not circular literal oracles.

The combined pattern is still material in numerical code: 1,222 tests use a literal expectation at
one input without recognized provenance. The highest raw cherry-picked shares are
`num_dual` (61.1%), `calculus` (43.3%), `num_complex` (30.1%), `multivector` (28.4%),
`linear` (28.0%), `num` (25.4%), and `uncertain` (24.1%). A plausible wrong coefficient, sign, or
index expression can survive whenever the chosen value is zero, one, symmetric, or otherwise
degenerate.

The strongest counterexamples in this audit are `fft` and `stats`. FFT has no raw tautology,
circular, cherry-picked, or no-assertion flags. Stats uses extensive table/helper-driven testing;
its 22 no-assertion flags all resolve to helper assertions, and only 7.7% of its tests are classified
as single-input. These crates show that the repository can express broader numeric checks without
making each test unreadable.

## Files with the largest raw hard/cherry signal

This table ranks syntactic leads, not confirmed defect counts.

| Raw matches | File | Composition |
|---:|---|---|
| 51 | `deep_causality_num/tests/float/float_32_tests.rs` | 4 circular, 47 cherry-picked |
| 47 | `deep_causality_num/tests/float/float_64_tests.rs` | 47 cherry-picked |
| 36 | `deep_causality_linear/tests/algorithms/small_tests.rs` | 10 circular, 26 cherry-picked |
| 29 | `deep_causality_num_dual/tests/dual/dual_number/real_tests.rs` | 4 circular, 25 cherry-picked |
| 27 | `deep_causality_num_complex/tests/complex/quaternion_number/ops_tests.rs` | 2 tautology, 24 circular, 1 cherry-picked |
| 27 | `deep_causality_topology/tests/types/gauge/gauge_field_lattice/lattice_gauge_field_tests.rs` | 10 tautology, 1 circular, 16 cherry-picked |
| 26 | `deep_causality_num_complex/tests/complex/complex_number/complex_field_tests.rs` | 4 circular, 22 cherry-picked |
| 22 | `deep_causality_num/tests/float_double/double_float_tests.rs` | 2 tautology, 2 circular, 18 cherry-picked |
| 21 | `deep_causality_algebra/tests/float106/double_algebra_tests.rs` | 1 tautology, 20 cherry-picked |
| 21 | `deep_causality_num/tests/float/bfloat16_impl_tests.rs` | 21 cherry-picked |
| 19 | `deep_causality_topology/tests/types/gauge/link_variable/link_variable_tests.rs` | 5 tautology, 14 cherry-picked |

`small_tests.rs` is an important warning about the ranking: its arithmetic expectations include
independently derived exact results and cross-algorithm checks. A high raw count indicates where to
review, not what to delete or mechanically rewrite.

## Topology: aggregate audit versus the existing failure note

The topology scan covered 1,668 explicit tests in 187 files and found 1,228 single-input, 216
cherry-picked, 40 circular, 29 weak-predicate, and one no-assertion candidate. Those figures describe
test construction across the full crate.

The existing simplicial Hodge-star note records a different kind of evidence: an actual
implementation failure at intermediate grades. The implementation's `star_2` scaling on a regular
tetrahedron is `L^2`, where a three-dimensional Hodge star requires `L^-1`. That defect survived the
suite even though the operator is exercised. It is a concrete example of why execution and
coverage do not prove that an oracle constrains the mathematics. The note remains the authoritative
description of that failure and its blast radius.

## Recommended order for a dedicated repair

1. Fix the confirmed ineffective assertions: the topology clone/default test, tensor finite-only
   density tests, uncertain sample/seed/branch tests, and topology mutations whose results are not
   inspected.
2. Replace the quaternion transcendental re-derivations with independent published values,
   algebraic identities, inverse identities on a range, or a higher-precision reference
   implementation. Preserve the useful basis-law tests.
3. Use targeted mutation testing on numeric kernels in the highest raw cherry-picked crates. A
   surviving constant, sign, comparison, or index mutation is evidence of a real gap; a scanner
   match alone is not.
4. For each surviving mutation, add an independent oracle plus a small table spanning ordinary,
   boundary, sign-changing, zero/one, and non-symmetric inputs. Record the oracle's provenance.
5. Re-run this inventory after repairs, but use the mutation survivors—not a target scanner
   percentage—as the acceptance criterion.

## Scanner limits

- Helper bodies are not followed, which caused 45 of 46 no-assertion false positives.
- Macros are not expanded, so the 6,824 functions are not the number of runtime cases.
- A loop or table suppresses `single-input`; the scanner does not judge whether its values are
  diverse.
- Literal provenance is recognized only through nearby source text and can be missed.
- Expected-value arithmetic cannot distinguish a copied implementation formula from an independent
  closed form or metamorphic relation.
- Weak predicates can be exact contracts for classification, validation, domain errors, and
  non-finite behavior.
- Doctests and tests outside each crate's `tests/` directory are outside this inventory.
- Early exits and conditional assertions require manual control-flow review.

## Verification

All seventeen packages were tested together with `cargo test -p ...`; the command exited
successfully. That establishes that the current suites pass. It does not resolve the audit findings:
a passing suite is the condition being evaluated, not evidence that every oracle is effective.

No production code or tests were changed for this audit.
