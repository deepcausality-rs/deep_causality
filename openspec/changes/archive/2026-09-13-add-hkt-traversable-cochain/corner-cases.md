<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Corner-case enumeration

Committed before the suites are written, per `unified-math-tdd-protocol`: "Enumerating in advance
is what distinguishes a corner case that was considered from one that was discovered through a
defect." The `Test` column is filled in as each suite lands; a row with no test means phase 2 is
unfinished.

## Shared by the three `Traversable` container witnesses

| # | Case | Why it is a corner | Test |
|---|---|---|---|
| C1 | empty container | the fold's identity element; returns `M::pure(empty)` rather than a failure | `sequence_empty_is_pure_empty` |
| C2 | single element | the only length where order and reversal coincide, so it cannot pin order | `sequence_single_element` |
| C3 | all elements succeed | the happy path, and the only one that exercises the full accumulator | `sequence_all_success_preserves_order` |
| C4 | **first** element fails | distinguishes first-error-wins from last | `sequence_first_error_wins` |
| C5 | **interior** element fails | a failure neither first nor last, which a bounds defect can skip | `sequence_interior_failure_collapses` |
| C6 | **last** element fails | the off-by-one partner of C4 | `sequence_last_error_still_fails` |
| C7 | two failures, different errors | pins *which* failure is reported, not merely that one is | `sequence_first_error_wins` |
| C8 | shaped inner applicative (cartesian) | the fold must delegate to `M::apply`, not assume a failure-shaped carrier | `sequence_cartesian_inner_applicative` |
| C9 | effect-order carrier (Writer-style log) | the substitute for the unavailable composition law; catches a right-to-left traversal that still returns the right result | `sequence_effect_order_is_left_to_right` |

C9 is required by the spec in place of the composition law. C2 is listed precisely because it is
*blind* to the order defects C3 and C9 catch — a test at length 1 must never be the only one.

## Laws, per container witness

| # | Case | Test |
|---|---|---|
| L1 | identity at the Identity applicative (not the vacuous phrasing) | `sequence_identity_law` |
| L2 | naturality across an applicative morphism | `sequence_naturality_law` |
| L3 | every law exercised at ≥ 2 distinct elements | inherent to L1/L2 inputs |

Composition is excluded and cannot be tested; see the spec. C9 substitutes for it.

## `CausalTensorWitness` only — shape

| # | Shape | Why it is a corner | Test |
|---|---|---|---|
| T1 | `[]` (rank 0, one element) | shape and length disagree; a `[len]` defect returns `[1]` | `sequence_rank0_shape_survives` |
| T2 | `[0, 3]` (zero extent, no elements) | length 0 but shape is not `[0]`; C1 and T2 differ | `sequence_zero_extent_shape_survives` |
| T3 | `[1]` | one element *and* a non-empty shape, separating T1 from C2 | `sequence_single_element_shape_survives` |
| T4 | `[2, 3]` | rank 2, six elements: the only row where a flattening defect is visible | `sequence_rank2_shape_survives` |

T1 and T3 both hold one element and must not be collapsed into one test: they differ exactly on the
shape a defect would fabricate.

## `CochainWitness`

| # | Case | Why it is a corner | Test |
|---|---|---|---|
| K1 | no values | `fold` must return the initial accumulator and never call the folding function | `cochain_fold_empty_returns_init` |
| K2 | degree 0 | the degree a defect most plausibly substitutes for a missing one | `cochain_fmap_degree_zero_survives` |
| K3 | degree ≠ value count | separates "carries the degree" from "derives the degree from the length" | `cochain_fmap_degree_differs_from_len` |
| K4 | ≥ 3 distinct values | pins index order for both `fmap` and `fold` | `cochain_fmap_index_order`, `cochain_fold_index_order` |
| K5 | order-sensitive fold operation | a commutative fold cannot detect a reversal | `cochain_fold_index_order` |

K3 is the row that matters most: `Cochain` carries `values` and `degree` independently, and every
other row is satisfiable by a witness that recomputes the degree from the length.

## Deliberately not enumerated

The protocol asks for non-finite inputs and the three precisions "for every precision-generic
path". Neither applies: these witnesses are generic in the element type and do no arithmetic on it
— `fmap`, `fold` and `sequence` move elements without computing on them — so there is no numeric
path to exercise at `f32`, `f64` and `Float106`, and no `NaN` a fold could mishandle. The element
types used in the suites are `i32` and small structs, chosen because equality on them is exact.
