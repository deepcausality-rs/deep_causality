<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Scenario coverage

Task 9.4: every scenario in the three spec files, and where it is exercised. 45 scenarios across
three specs; 51 tests across four crates.

Some scenarios are **structural** — they assert a property of the source, the build, or a
documented decision rather than a runtime behaviour. Those are marked and say what settles them,
because a scenario with no runtime test is otherwise indistinguishable from one that was forgotten.

`V` = `hkt_vec_ext_tests.rs` (haft), `D` = `witness_tests.rs` (linear), `T` =
`causal_tensor_ext_hkt_tests.rs` (tensor), `K` = `hkt_cochain_tests.rs` (topology),
`L` = `formalization_lean/traversable_list_tests.rs` (haft).

## `hkt-traversable-containers` — 26 scenarios

| Scenario | Covered by |
|---|---|
| Each witness is usable as a `Traversable` carrier | `V/D/T::*_all_success_preserves_order` (three witnesses) |
| The trait's contract is unchanged | **structural** — `sequence`'s bound still reads `M: Applicative<M> + HKT`; the two existing impls are untouched in the diff |
| Order survives a multi-element traversal | `V/D/T::*_all_success_preserves_order` |
| The cost is documented with its actual cause | **structural** — the `# Cost` section on each of the three impls names `apply`'s `FnMut` bound |
| An accumulator moved rather than cloned is rejected at compile time | **structural, measured** — E0525 recorded in design Decision 1 |
| An accumulator taken on first use breaks the cartesian carriers | **structural, measured** — the `Option::take` variant panics on `*_cartesian_inner_applicative`; recorded in Decision 1 |
| One `None` collapses the container | `V/D/T::*_interior_failure_collapses` |
| The first error wins over a later one | `V/D/T::*_first_error_wins` |
| An all-success traversal carries every element | `V/D/T::*_all_success_preserves_order` |
| The empty container succeeds | `V/D::*_empty_is_pure_empty`, `T::*_empty_is_pure_empty` |
| A single-element container round-trips | `V/D::*_single_element`, `T::*_single_element_shape_survives` |
| A rank-2 shape survives | `T::test_traversable_tensor_rank2_shape_survives` |
| A zero-extent shape survives | `T::test_traversable_tensor_zero_extent_shape_survives` |
| The rank-0 shape survives | `T::test_traversable_tensor_rank0_shape_survives` |
| The relationship to `bind` is recorded | **structural** — the `# The input's shape survives` section on the tensor impl |
| Naturality holds across an applicative morphism | `V/D/T::*_naturality_law`, `L::test_traversable_list_naturality` |
| Identity holds at the identity applicative | `V/D/T::*_identity_law`, `L::test_traversable_list_identity` |
| Effect order is pinned by a carrier that observes it | `V/D/T::*_effect_order_is_left_to_right` |
| The composition law is recorded as blocked rather than silently dropped | **structural** — the note at the foot of `hkt_vec_ext.rs` |
| Every law is exercised at more than one element | **structural** — every law test above uses ≥ 3 distinct elements |
| Sequencing at a cartesian inner applicative enumerates the product | `V/D/T::*_cartesian_inner_applicative` |
| The proofs typecheck under the workspace gate | `bazel test //lean:Haft` — 1/1, and `//lean:proofs` 11/11 |
| The theorems are discoverable from the map | `THEOREM_MAP.md` rows 212–214; the theorem-map CI check passes locally (193 ids, fail = 0) |
| The element count is proved preserved | `L::test_traversable_list_length_preserved`, plus the Lean theorem it witnesses |
| The effect monads remain admissible | **structural** — the bound is unchanged, so the population is unchanged |
| A previously named lost carrier still works | `V/D/T::*_box_inner_applicative` |

## `topology-cochain-witness` — 12 scenarios

| Scenario | Covered by |
|---|---|
| The witness is reachable from the crate root | `K` imports `CochainWitness` from `deep_causality_topology` |
| The two claimed traits are usable | `K::test_cochain_fmap_*`, `K::test_cochain_fold_*` |
| The declined traits are absent | **structural** — only `HKT`, `Functor`, `Foldable` are implemented; the reason for declining `Pure` is on the witness |
| The degree survives a map that changes the element type | `K::test_cochain_fmap_changes_element_type` |
| Degree 0 survives | `K::test_cochain_fmap_degree_zero_survives` |
| Values are mapped in index order | `K::test_cochain_fmap_index_order` |
| A non-commutative fold reveals the order | `K::test_cochain_fold_index_order` |
| An empty cochain folds to the initial accumulator | `K::test_cochain_fold_empty_returns_init` (asserts the function is never called) |
| Functor identity holds | `K::test_cochain_functor_identity_law` |
| Functor composition holds | `K::test_cochain_functor_composition_law` |
| `fold` and `fmap` agree | `K::test_cochain_fold_fmap_consistency` |
| The laws are exercised where degree and length differ | `K::test_cochain_fmap_degree_differs_from_len` (degree 2, five values) |

## `haft-vec-traversable` (delta) — 7 scenarios

| Scenario | Covered by |
|---|---|
| The trait's contract is unchanged | **structural** — as above |
| The lapsed premise is confirmed absent rather than assumed | **structural, measured** — `trait Satisfies` exists nowhere in the tree; `HKT` is `type Type<T>;` alone |
| The effect monads remain admissible as inner applicatives | **structural** — bound unchanged |
| `VecWitness` sequences over the carriers the bound move would have lost | `V::test_traversable_vec_box_inner_applicative`, `V::test_traversable_vec_cartesian_inner_applicative` |
| A reader learns which premise changed | **structural** — the rewritten note at the foot of `hkt_vec_ext.rs` |
| The superseded conclusion is not left standing | **structural** — the "only two carriers" claim and the E0277 attribution are both gone |
| Every doctest on the trait executes | `cargo test -p deep_causality_haft --doc` — 5 pass, none `ignore`d |

## Not covered, and why

The composition law has no test and cannot have one: a `Compose<M, N>` applicative is unwritable
against the current `Applicative`. The spec excludes it and requires the blocker to be documented
instead, which the scenario above tracks. The effect-order tests are the substitute, and the
phase-3 audit confirmed they catch the defect class composition would have caught.

## Line coverage of added and edited code (task 9.3)

`cargo llvm-cov`, per file:

| File | Regions | Lines | Added/edited code |
|---|---|---|---|
| `topology/src/extensions/hkt_cochain/mod.rs` (new file) | 100.00% | 100.00% | fully covered |
| `haft/src/extensions/hkt_vec_ext.rs` | 100.00% | 100.00% | fully covered |
| `linear/src/extensions/hkt/dense_vector_witness.rs` | 100.00% | 100.00% | fully covered |
| `tensor/src/extensions/ext_hkt.rs` | 94.57% | 96.08% | **fully covered** — see below |

`ext_hkt.rs` is the one file below 100%, and none of the shortfall is in this change. The
`Traversable` impl spans lines 149–197; every uncovered line lies outside it:

- lines 112–119, 133–135, 141–144 — `Monad::bind`, pre-existing
- lines 258–259 — the `else` branch of `Applicative::apply`, pre-existing

Checked by extracting the zero-count lines from the line-level report and mapping each to its
owning impl block. The range 149–197 yields no uncovered line.

The three `sequence` bodies and both `CochainWitness` bodies are straight-line — no `if`, `match`
or early `return` — so there is no partially-taken branch to miss; the only question is whether the
lines execute, and every one does.

The pre-existing gaps in `bind` and `apply` are out of scope for this change: it did not touch
those functions, and the standing requirement is full coverage of *added or edited* files' new
code, not retroactive coverage of everything sharing a file.
