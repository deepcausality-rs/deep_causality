<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Remaining-crates test-suite audit

This report applies the physics test-audit scanner to the nine workspace crates not covered by the
physics, utility, or unified-math audits. It is an inventory and a calibrated review, not a
proposal and not a claim that every syntactic match is a defective test.

## Scope

The audit covers:

- `deep_causality`
- `deep_causality_algorithms`
- `deep_causality_cfd`
- `deep_causality_core`
- `deep_causality_data_structures`
- `deep_causality_discovery`
- `deep_causality_ethos`
- `deep_causality_quantum`
- `ultragraph`

Together with the existing physics, utility, and unified-math reports, this completes static
scanner coverage of all 30 workspace library crates.

## Finding in one line

**The same weakness exists outside physics and unified math, but at a lower raw rate: 3,005 of
3,633 explicit tests are single-input, while 326 use an unexplained single-point literal oracle;
the clearest confirmed gaps are value-returning operations that assert only success.**

The raw hard classes contain substantial false-positive populations. Independent analytic CFD
solutions are classified as circular, explicit match-and-panic error checks are classified as
assertion-free, and exact `Result` contracts are classified as tautologies. The raw results are
review queues, not defect counts.

## Method

The unchanged classification logic from `openspec/notes/test_audit/physics/audit_tests.py` was run
once per crate with that crate's `tests/` directory as its root. It inspected 490 Rust test files,
containing 82,315 lines, and recognized 3,633 explicit `#[test]` functions. Tests embedded in
`src/`, doctests, and macro expansions are outside that static count.

The classes overlap. A test can be both single-input and cherry-picked, or both circular and
cherry-picked. Counts must not be added to obtain a number of defective tests.

| Class | Raw count | Share | Scanner meaning |
|---|---:|---:|---|
| single-input | 3,005 | 82.7% | No loop or table was recognized. |
| cherry-picked | 326 | 9.0% | One input, literal expected value, and no recognized provenance. |
| circular | 62 | 1.7% | Arithmetic was used to construct an expected value. |
| tautology | 60 | 1.7% | Only a weak predicate such as `is_ok`, `is_some`, or `is_finite` was recognized. |
| no-assertion | 5 | 0.1% | No assertion macro or asserting helper was recognized. |

## Per-crate inventory

Percentages use each crate's explicit test-function count. The final five columns are raw,
overlapping classifications.

| Crate | Test files | Tests | Single-input | Cherry-picked | Tautology | Circular | No assertion |
|---|---:|---:|---:|---:|---:|---:|---:|
| `deep_causality` | 160 | 1,137 | 1,041 (91.6%) | 90 (7.9%) | 21 (1.8%) | 4 (0.4%) | 2 (0.2%) |
| `deep_causality_algorithms` | 37 | 334 | 254 (76.0%) | 23 (6.9%) | 3 (0.9%) | 7 (2.1%) | 0 |
| `deep_causality_cfd` | 123 | 938 | 649 (69.2%) | 129 (13.8%) | 22 (2.3%) | 46 (4.9%) | 0 |
| `deep_causality_core` | 31 | 252 | 243 (96.4%) | 0 | 0 | 0 | 0 |
| `deep_causality_data_structures` | 10 | 81 | 66 (81.5%) | 2 (2.5%) | 0 | 0 | 0 |
| `deep_causality_discovery` | 52 | 194 | 178 (91.8%) | 5 (2.6%) | 3 (1.5%) | 0 | 3 (1.5%) |
| `deep_causality_ethos` | 17 | 106 | 105 (99.1%) | 0 | 1 (0.9%) | 0 | 0 |
| `deep_causality_quantum` | 45 | 462 | 371 (80.3%) | 76 (16.5%) | 9 (1.9%) | 3 (0.6%) | 0 |
| `ultragraph` | 15 | 129 | 98 (76.0%) | 1 (0.8%) | 1 (0.8%) | 2 (1.6%) | 0 |
| **Total** | **490** | **3,633** | **3,005 (82.7%)** | **326 (9.0%)** | **60 (1.7%)** | **62 (1.7%)** | **5 (0.1%)** |

The percentages do not measure test quality by themselves. `deep_causality_core` has a 96.4%
single-input rate but no hard or cherry-picked matches: its tests mostly pin discrete monadic,
state, error, and control-flow behavior at individually named cases. Conversely, one test with many
inputs can still repeat the same ineffective assertion.

## Calibrated findings

### Two of five no-assertion flags make no observation

The three discovery flags are false positives. `test_from_csv_error`, `test_from_parquet_error`,
and `test_from_mrmr_error` use explicit variant matching and panic on the wrong branch. The scanner
recognizes neither `if let ... else { panic!() }` nor match-and-panic assertions.

The two `deep_causality` matches genuinely contain no observation:

- `deep_causality/tests/types/csm_types/csm_action/csm_action_tests.rs`,
  `test_causal_action_creation`, constructs and clones an action and formats its debug output into
  an unused binding. It verifies compilation and absence of panic, but none of the constructed
  state or rendered output.
- `deep_causality/tests/utils/time_utils_tests.rs`, `test_time`, calls `time_execution` around a
  function that prints a line. It does not assert that the wrapped function's return value passes
  through, and it does not observe the timing label.

### Quantum has the clearest weak-assertion cluster

Eight of quantum's nine raw tautology flags are ineffective answer checks:

- `deep_causality_quantum/tests/types/qgates/mechanics_tests.rs` tests valid expectation-value,
  identity-gate, and commutator kernels only with `result.is_ok()`. These kernels return numeric or
  state results that are never inspected.
- `deep_causality_quantum/tests/types/qgates/wrappers_tests.rs` tests five successful wrappers—Born
  probability, expectation value, gate application, commutator, and fidelity—only with
  `effect.is_ok()`. A wrapper returning any successful value would pass.

The ninth flag, `faithfulness_tests.rs::test_complete_bipartite_is_c3_free`, is not ineffective. It
first checks the independently observable absence of a C3 witness and then checks that the
corresponding validation returns `Ok`.

The 76 quantum cherry-picked matches are also a mixed population. For example,
`operator_residual_tests.rs` explicitly documents hand-evaluated 3-4-5 oracles, overflow scaling,
and enumerated corner cases at module level. The scanner misses that provenance because it is too
far from individual assertions. That file is evidence of good test design, not an 11-test repair
target.

### Root `deep_causality` tests often assert completion instead of behavior

Several CSM tests name an action or state transition but assert only that evaluation returned
`Ok`:

- `csm_single_state_tests.rs::eval_single_state_success_fires_action` uses an action that succeeds,
  so `is_ok()` does not prove it was invoked.
- `csm_all_states_tests.rs::eval_all_states_success_active_state_fires_action` has the same gap.
- the uncertain-state success tests establish only acceptance, not the selected state or action.

This does not apply to every CSM `is_ok()` test. Tests that install a deliberately failing action
for an inactive state use success as evidence that the action was not invoked; that is a meaningful
negative control.

Other confirmed or high-confidence gaps are:

- `causality_graph_reasoning_sub_tests.rs::test_evaluate_subgraph_follows_a_relay_to_a_valid_target`
  asserts only success and explicitly discards the target index. It does not prove the relay reached
  that target or returned its effect.
- the collection `test_get_item_by_id` cases assert only `is_some()`. They do not show that the
  returned item has the requested identity.
- `inferable_vec_tests.rs::test_item_conjoint_delta` constructs its expected value by repeating the
  default method's stated `abs(1 - observation)` expression at one input. It lacks an independent
  law or range that could expose a shared formula error.

The tangent-spacetime and NED distance tests are also classified as circular because they calculate
Euclidean norms in the test. Those formulas can be legitimate independent definitions, but one
3-4-12 or 100-50-10 example cannot distinguish a generally correct metric implementation from an
implementation specialized to the same uncomplicated case.

### CFD has the largest candidate set, but many candidates are strong controls

CFD accounts for 129 of the 326 cherry-picked and 46 of the 62 circular matches. Most of the
circular label is syntactic noise: analytic decay solutions, Fourier-mode diffusion, Taylor-Green
energy, step-refinement invariance, dense-versus-QTT stencil comparisons, and coordinate Jacobian
identities are independent or metamorphic controls. They should not be mechanically rewritten.

The clear weak cases are narrower:

- `deep_causality_cfd/tests/theories/wrappers_tests.rs`,
  `test_compressible_ns_momentum_rhs_effect_wrapper`, asserts only success while the neighboring
  continuity wrapper checks its carried numeric value.
- `deep_causality_cfd/tests/tensor_bridge/projection_tests.rs::projected_field_is_finite` checks
  every output only for finiteness. It does not assert the defining divergence-free property or
  compare with a dense/reference projection.
- several configuration tests intentionally assert only successful construction. Those are valid
  positive-path tests when acceptance is the contract; they are not numeric oracle tests.

`march_run_tests.rs::test_centerline_profile_is_recorded` checks only that the named series exists.
That exactly establishes presence, but does not validate the series length or values. It is an
incomplete result check rather than a tautology.

### Algorithms' circular flags are predominantly independent oracles

The seven algorithm circular candidates are named closed forms, a Jacobian identity, and an
`f32`/`f64` differential comparison. They exercise a path independent of the implementation under
test and should be preserved unless mutation testing proves otherwise.

The three weak-predicate candidates test finite behavior around covariance singularities and an
automatic log-to-log1p downgrade. Finiteness is a real robustness contract for the two variance
floor/ridge tests. `gaussian_tests.rs::auto_downgrade_keeps_density_finite`, however, does not prove
that log1p was selected: any finite fallback would pass. It should compare the returned density
with the log1p oracle already used by nearby tests.

### Discovery, ethos, and ultragraph each have a localized incomplete check

- `deep_causality_discovery/tests/types/cdl/brcd_pipeline_tests.rs`,
  `test_brcd_full_pipeline_boss_fallback`, asserts only that the final inner result is `Ok`. It does
  not inspect the report or show that the requested BOSS fallback produced the expected outcome.
- `deep_causality_ethos/tests/types/effect_ethos/effect_ethos_linking_tests.rs`,
  `test_linking_success`, asserts only that `link_inheritance` returned `Ok`. A successful no-op
  would pass; the relationship should be observed through graph verification or evaluation.
- `ultragraph/tests/types/ultra_graph/graph_csm_algo_topological_tests.rs`,
  `test_topological_sort_on_empty_graph`, asserts `Ok(Some(_))` but not that the returned ordering
  is empty. Any nonempty ordering would pass the test named for the empty graph.

### Core and data structures form the low-priority baseline

`deep_causality_core` has no raw tautology, circular, cherry-picked, or no-assertion candidates.
Its high single-input percentage reflects individually named cases rather than a detected oracle
failure.

`deep_causality_data_structures` has only two cherry-picked matches. Both are ordinary grid
dimension and storage round-trip tests. The suite also contains explicit distinct-dimension indexing
regressions, boundary cases, storage variants, and stress tests. No hard defect was established by
this audit.

## Files with the largest raw hard/cherry signal

This table ranks syntactic leads, not confirmed defect counts.

| Raw matches | File | Composition |
|---:|---|---|
| 13 | `deep_causality_algorithms/tests/causal_discovery/brcd/gaussian_tests.rs` | 3 circular, 1 tautology, 9 cherry-picked |
| 13 | `deep_causality/tests/types/context_node_types/space_time/tangent_spacetime/tangent_spacetime_tests.rs` | 2 circular, 11 cherry-picked |
| 11 | `deep_causality_quantum/tests/types/qgates/operator_residual_tests.rs` | 11 cherry-picked; manually documented oracles |
| 9 | `deep_causality_cfd/tests/types/flow/coupling_tests.rs` | 1 circular, 8 cherry-picked |
| 8 | `deep_causality_cfd/tests/types/flow/blackout_tests.rs` | 2 circular, 1 tautology, 5 cherry-picked |
| 7 | `deep_causality_cfd/tests/types/flow_config/compressible_march_config_tests.rs` | 1 circular, 6 cherry-picked |
| 7 | `deep_causality_cfd/tests/types/flow/corridor/lift_tests.rs` | 7 cherry-picked |
| 7 | `deep_causality_cfd/tests/solvers/dec/surface_force_tests.rs` | 4 circular, 3 cherry-picked |
| 6 | `deep_causality_quantum/tests/types/qgates/gates_tests.rs` | 6 cherry-picked |
| 6 | `deep_causality_quantum/tests/formalization_lean/partial_trace_tests.rs` | 6 cherry-picked |
| 6 | `deep_causality/tests/types/csm_types/csm/csm_all_states_tests.rs` | 6 tautology candidates |

The first and third rows demonstrate why ranking cannot replace review: both contain unusually
explicit independent-oracle documentation even though the scanner ranks them worst.

## Scanner limits

- Helper bodies are recognized only by name and assertion text, not by control flow.
- `panic!` in a rejected match arm is not recognized as an assertion.
- Macros are not expanded, so 3,633 is not the number of runtime cases.
- A loop or table suppresses `single-input`; the scanner does not judge input diversity.
- Literal provenance is recognized only through nearby keywords and can miss module-level oracle
  documentation.
- Expected-value arithmetic cannot distinguish copied production logic from an independent closed
  form, analytic solution, or metamorphic relation.
- Weak predicates may be exact contracts for constructors, classification, validation, domain
  errors, stability, and path selection.
- Doctests and tests embedded in `src/` are outside this inventory.
- Assertions behind early exits and conditional blocks require manual reachability review.

## Verification

All nine packages were tested together with one `cargo test` invocation. The command completed
successfully. This establishes that the current suites pass, not that every oracle would detect an
incorrect implementation.

No production code or tests were changed for this audit.

## Recommended order of fixes

This ordering covers the nine crates in this report. It prioritizes confirmed ineffective behavior
checks, consequence of a missed defect, and then raw mutation risk; it does not sort by scanner
percentage alone.

1. **`deep_causality_quantum`** — repair the eight output-blind kernel and wrapper success tests
   first. Exact values and identity laws are available, so these are high-confidence, bounded
   fixes in a correctness-sensitive crate.
2. **`deep_causality`** — make CSM action execution observable, assert relay targets and effects,
   repair the two assertion-free tests, and replace getter-presence checks with identity checks.
   This is the largest confirmed behavioral cluster and the public root crate.
3. **`deep_causality_cfd`** — fix the momentum wrapper and projection property checks, then use
   targeted mutation testing on coupling, blackout, corridor, force, and inverse kernels. Preserve
   the independent analytic and dense-reference controls.
4. **`deep_causality_algorithms`** — pin the automatic transform downgrade numerically, then run
   mutations over BRCD scoring and Gaussian-family code. The raw circular tests should remain when
   their closed forms are independent.
5. **`deep_causality_discovery`** — make the BOSS fallback pipeline assert the produced report and
   selected outcome. Retain the match-and-panic conversion tests or express them as `matches!` for
   scanner clarity.
6. **`deep_causality_ethos`** — make successful inheritance linking observable through graph or
   verdict behavior. This is one localized, inexpensive repair.
7. **`ultragraph`** — assert that topological sorting an empty graph returns an empty ordering, then
   mutation-test centrality normalization before changing its independently derived expectations.
8. **`deep_causality_data_structures`** — no confirmed hard defect; use focused index mutations as
   a verification pass after the higher-risk crates.
9. **`deep_causality_core`** — no hard candidate from this audit. Review only through mutation
   survivors; do not expand tests merely to reduce the single-input percentage.
