<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Physics test audit v2 — comparison with the original audit

## Scope and method

`audit_tests_v2.py` applies the risk-first method from
[Dan Luu's agentic-testing analysis](https://danluu.com/agentic-testing/) and scanned only
`deep_causality_physics/tests`: 1,766 tests in 190 Rust files. It asks a narrower and more useful
question than line coverage: **what plausible wrong
implementation could this test allow to survive?** The scanner therefore looks for missing or
answer-blind observations, assertions that can be skipped, self-comparisons, discarded results,
unexplained oracles, and fixtures whose symmetry or degeneracy can hide a defect.

Every result is a review candidate, not an automatically proved defect. In particular, fixture and
oracle classifications require semantic review or mutation testing.

Run both scanners from the repository root:

```console
python3 openspec/notes/test_audit/physics/audit_tests.py
python3 openspec/notes/test_audit/physics/audit_tests_v2.py
```

The new scanner writes its site-level inventory to `/tmp/physics_test_audit_v2.json` by default.
Use `--json PATH` to select another destination. Its default and this report are deliberately scoped
to physics; `--root` must be supplied explicitly to scan another test tree.

## Count comparison

The published audit recorded 1,751 tests. The current tree contains 15 more, so the fairest
comparison is the original scanner rerun over the same 1,766 tests as v2. The published numbers are
included to make the baseline explicit.

| Original category | Published audit | Original scanner now | Closest v2 category | v2 now | Interpretation |
|---|---:|---:|---|---:|---|
| single-input | 1,579 (90.2%) | 1,585 (89.8%) | single-case | 1,603 (90.8%) | Broad ceiling: one explicit case and no detected table, loop, or sweep |
| cherry-picked | 629 (35.9%) | 632 (35.8%) | unproven-literal-oracle | 545 (30.9%) | Literal expectation with no nearby evidence of an independent source |
| tautology | 116 (6.6%) | 116 (6.6%) | answer-blind | 119 (6.7%) | Assertions observe status or other weak properties, but not the answer |
| circular | 49 (2.8%) | 52 (2.9%) | derived-oracle | 54 (3.1%) | An expected value is calculated in the test |
| no-assertion | 1 (0.1%) | 0 | no-observation | 5 (0.3%) | No assertion, panic expectation, or call to an asserting helper |

The names are intentionally not exact synonyms. V2 splits derived oracles into 54 total and 36
without nearby provenance; an independently sourced analytic calculation is not automatically
circular. Likewise, v2 does not equate every literal with cherry-picking when a nearby comment
identifies a reference, invariant, scaling law, independent method, or high-precision oracle.

## What v2 corrects

### Multiline assertions and compound predicates

The original scanner classified assertion-bearing *lines*. V2 parses balanced test bodies and
balanced macro calls, so it sees predicates split over several lines.

This changes the answer-blind set by a net three: 115 sites agree, v2 adds four sites the old scan
missed, and it removes one old false positive.

The additions are:

- `kernels/astro/solver_convergence_tests.rs:159:test_the_w_mass_solver_converges_at_the_shipped_constants`
- `kernels/em/wrappers_tests.rs:186:test_proca_equation_wrapper_success`
- `kernels/quantum/mechanics_tests.rs:43:test_klein_gordon_kernel_valid`
- `quantities/nuclear_quantities/trait_coverage_tests.rs:9:test_nuclear_scalars_traits`

The removed site is
`kernels/hypersonic/finite_rate_tests.rs:38:dr_pairs_with_the_forward_rate_as_a_finite_equilibrium_constant`.
Its assertion requires both finiteness and positivity. The old scanner saw `.is_finite()` and
discarded the entire line; v2 retains the `k_eq > 0.0` constraint as informative.

### Bounded, exact helper recognition

The original scanner searched 4,000 characters after each function declaration and treated a
helper-name substring anywhere in a test as delegation. That can borrow an assertion from a later
function or accept text that is not a function call. V2 extracts balanced function bodies, builds a
transitive helper call graph, and recognizes exact calls from the executable part of each test.

That exposes five tests with no observation at all:

| Site | Why it passes without testing behavior |
|---|---|
| `kernels/chronometric/wrapper_tests.rs:238:test_wrapper_logs_field_present` | Formats the logs and discards the string; presence or contents are never asserted |
| `kernels/condensed/wrappers_tests.rs:101:test_wrapper_moire_magic_angle` | Computes and discards `effect.is_ok() || effect.is_err()`, which is always true |
| `kernels/mhd/wrappers_tests.rs:174:test_ideal_induction_wrapper` | Computes and discards the same success-or-error tautology |
| `kernels/mhd/wrappers_tests.rs:301:test_energy_momentum_tensor_em_wrapper_dimension_error` | Computes and discards the same success-or-error tautology |
| `quantities/materials_quantities/materials_quantities_tests.rs:142:test_strain_clone_debug` | Clones and formats a value, then discards both results |

These five are high-confidence defects in the tests. None can fail because an observed value is
wrong. The clone/debug cases may have compile-time coverage intent, but a runtime test with no
assertion does not verify the resulting semantics.

### Assertions that are optional at runtime

V2 finds five `conditional-only` tests. Every assertion is inside `if` or `if let`, and no assertion
or panic guard covers the other path:

- `kernels/fluids/coherent_structures_tests.rs:245:test_swirling_strength_consistent_with_delta_sign`
- `kernels/relativity/spacetime_coverage_tests.rs:10:experiment_gamma_clamp`
- `theories/general_relativity/gr_ops_impl_tests.rs:541:test_proper_time_si`
- `theories/general_relativity/gr_ops_impl_tests.rs:691:test_solve_geodesic_interface`
- `theories/general_relativity/gr_ops_impl_tests.rs:995:test_parallel_transport_interface`

For the first test, every case can avoid an assertion when `delta` lies inside the tolerance band
or is NaN. For the other four, an unexpected `Err` skips the `if let Ok(...)` body and passes. These
are also high-confidence defects: each test needs an assertion for the omitted branch or an
unconditional extraction such as `expect(...)` before checking the value.

## New risk signals

| Category | Count | How to use it |
|---|---:|---|
| uniform-fixture | 134 | Review whether repeated values make indices, signs, axes, or coefficients indistinguishable |
| discarded-result | 36 | Review ignored computations; many clone/debug smoke uses are low risk, but ignored semantic results are not |
| palindromic-fixture | 16 | Review whether reversal, transpose, or swapped-index bugs preserve the fixture |
| self-comparison | 8 | Remove or replace reflexive `x == x.clone()` checks; some occur beside useful assertions, so this is an assertion count, not eight wholly empty tests |
| zero-one-fixture | 6 | Review whether multiplication, exponent, scale, or offset defects collapse at 0, 1, or -1 |

Two examples show why fixture discrimination matters:

- `kernels/photonics/ray_tests.rs:test_ray_transfer` uses only the identity ABCD matrix. It verifies
  that the input ray is unchanged, but cannot expose swapped entries or broken off-diagonal matrix
  terms. Add an asymmetric matrix with four distinct coefficients and an independently calculated
  output.
- `kernels/relativity/gravity_tests.rs:test_einstein_tensor_kernel_valid` makes the Ricci tensor and
  metric identical, chooses a scalar that cancels them to zero, and checks only element zero. A
  wrong factor, sign, or untouched component can survive. Add non-equal, non-diagonal tensors and
  assert every output component.

The scanner found no `duplicate-branch` or `random-survival-only` matches. That means no supported
syntax matched; it is not proof that no semantically equivalent pattern exists.

## Finding relative to the original audit

The original conclusion is confirmed, not overturned:

1. The suite is still overwhelmingly point-tested: both scanners put the single-case share at about
   90%.
2. Answer-blind testing remains about 6.7%, and the v2 parser shows the prior count was slightly low.
3. The broad literal-oracle count falls from 632 to 545 because v2 recognizes more evidence of
   independent provenance. This is better triage, not evidence that 87 tests became stronger.
4. Derived expectations need two queues: 36 unexplained candidates deserve priority; 18 with nearby
   provenance need review before anyone calls them circular.
5. The original audit underestimated outright vacuity. V2 adds five no-observation and five
   conditional-only tests that deserve repair before bulk oracle work.

The strongest transferred methodology is therefore: **enumerate plausible mistakes first, then
choose an independent oracle and discriminating inputs that force each mistake to produce a
different observation. Finally, verify sensitivity by injecting those mistakes with targeted
mutation testing.** More test cases or more line coverage without that loop will reproduce the
existing problem.

## Recommended repair and verification order

1. Repair the five no-observation tests and five conditional-only tests. Their failure mechanism is
   directly visible and does not require statistical interpretation.
2. Replace the 119 answer-blind assertions with value, invariant, or precise error assertions.
3. Review the 36 unproven derived oracles and replace same-formula calculations with an independent
   source, cross-kernel identity, higher-precision computation, or hand-worked result.
4. Attack degenerate fixtures in high-risk numeric kernels. Start with identity, uniform, zero, one,
   symmetric, and palindromic inputs; pair each with asymmetric nondegenerate cases.
5. Convert unexplained one-point literal tests into boundary-aware tables only after documenting the
   oracle for each family.
6. Run targeted mutation testing after each family rewrite. A candidate is closed only when the
   tests kill the plausible wrong constants, signs, comparisons, branches, and index mutations it
   was designed against.

## Scanner limitations

- This is a lexical Rust scanner, not a compiler front end. Macros and unusual syntax can evade it.
- Provenance is inferred from nearby words; a comment can claim independence without establishing
  it, and an actual independent oracle can lack a recognized label.
- Fixture categories detect simple literal shapes, not domain semantics.
- `single-case` is a ceiling, not a defect backlog. Singular refusal and boundary tests are often
  correctly one-case tests.
- The scanner cannot determine whether a derived formula is independent of production reasoning.
  Manual review and mutation testing remain the deciding evidence.
