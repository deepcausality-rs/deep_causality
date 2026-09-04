<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# C1 stage notes

## Task 2.2 — the R4 search: result

**The omission of R4 is latent, not live.** Up to the bound below, closing under R1–R3 always
reaches the maximally oriented PDAG on the graphs BRCD's Algorithm 1 constructs. R4 never fires.
Neither does R3.

Harness: `deep_causality_algorithms/verification/brcd/verification_meek_r4.rs`.
Run: `cargo run --release -p deep_causality_algorithms --example verification_meek_r4` (~85 s).

### The bound

Exhaustive over labelled DAGs, at `(n, k)` = (3,1), (4,1), (5,1), (3,2), (4,2), (5,2), (5,3), where
`n` is the vertex count and `k` the root-cause set size Algorithm 1 ranges over.

| | |
|---|---|
| DAGs enumerated | 88 979 |
| augmented graphs (CPDAG + `F → R` + an oriented cut) | 4 679 566 |
| with a consistent DAG extension | 3 811 158 |
| without one, skipped | 868 408 |
| **inputs where R1–R3 under-orients** | **0** |

### Why the oracle is not R4

The obvious harness diffs the R1–R3 closure against the R1–R4 closure, and inherits every risk in how
R4 is transcribed — the literature differs on whether `a — d` must be undirected. So the oracle is
the definition instead: an edge is compelled when every consistent DAG extension agrees on it, and
orienting every compelled edge gives the maximally oriented PDAG. That needs no orientation rule at
all. R4 enters only as the hypothesis under test.

### Controls

A negative result is worthless without evidence the search could have found something. Four checks,
all passing:

| Check | Result | What a failure would have meant |
|---|---|---|
| R1–R3 equals the definition on every *pattern* (n ≤ 4: 3, 25, 543 DAGs) | pass | The harness is wrong — this is a theorem |
| **Positive control**: R4 fires on its own canonical configuration, the definition compels it, R1–R3 misses it | pass | R4 as transcribed can never fire, making "R4 never fires" vacuous |
| **Negative control**: the no-rule closure differs from the definition | 2 601 552 inputs | The comparison detects nothing |
| **Soundness**: R4 never orients an edge the definition leaves free | 0 violations | The transcription is unsound |

The first control attempt used R1–R2 as the deliberately-incomplete closure and returned zero — which
looked like a broken comparison and was not. R1–R2 is *also* complete on this family, so R3 never
fires either. The no-rule closure is the correct control.

### Hypothesis for why R4 never fires

Not proved, and recorded so the next reader does not re-derive it. R4 needs `a — d` and `a — c`
undirected with `d → c` directed: a partially directed triangle. A CPDAG has no partially directed
cycle, and adding `F → R` arcs plus a cut orientation appears not to create one. The positive-control
configuration does contain such a triangle and is a legitimate PDAG — so R4 is meaningful for
PDAGs-with-background-knowledge in general, and BRCD's particular construction seems never to reach
one.

If that is a theorem, it would be the right thing to state in the closure's documentation. It is not
one yet.

### What this changes

The proposal's original framing — that this is one of three live defects — is **withdrawn for C1**.
The precondition the docstring relies on is still false: Algorithm 1 does supply background
knowledge, and the docstring's justification does not hold as written. But the consequence it was
claimed to have is not observable at this bound.

R4 is still added, for three reasons: a bound is not a proof; the closure moves into a general-purpose
graph type where callers other than BRCD will reach it; and the docstring's stated precondition needs
to become true rather than stay false-but-harmless.

`vector_norm_l2` is unaffected and is measured above. The quantum modulus is a separate matter: the
direct form is duplicated there, but whether it overflows depends on the entries, and a density
matrix's are bounded by one. It should be described as duplication with a latent numerical hazard,
not as a demonstrated defect, until something measures it.

## Tasks 2.4–2.6 — phase 2: the suite

26 tests, all failing with the phase-1 `unimplemented!` panic and nothing else.

```
test result: FAILED. 1617 passed; 26 failed; 1 ignored
26 failures, 26 carrying "phase 1 declares the surface only"
```

Files: `tests/types/mixed_graph/meek_tests.rs` (14), `chordality_tests.rs` (12), both registered in
`float_bfloat16`. The Bazel suite globs `tests/types/mixed_graph/*_tests.rs`, so no BUILD edit was needed.

### Corner-case enumeration

| Class | Answer |
|---|---|
| A — empty input | **n/a**: `MixedGraph::new` refuses zero vertices (`InvalidInput("MixedGraph must have at least one vertex")`). Smallest constructible case is one vertex, covered |
| B — single element | `a_single_vertex_closes_to_itself`, `a_single_vertex_is_chordal` |
| C — quantities coincide | `a_graph_with_no_arc_is_unchanged` (no rule's precondition holds), `a_complete_undirected_graph_is_a_fixpoint`, `a_triangle_is_chordal` (vacuous) |
| D — index degenerates | `a_four_cycle_is_not_chordal` is the smallest cycle the definition can reject; the 3-cycle above is the case where it must not |
| E — threshold, both sides | `a_four_cycle_with_a_chord_is_chordal` against `a_four_cycle_is_not_chordal` |
| F/G/H — zero, negative, boundary | **n/a**: the inputs are graphs, not numbers |
| I — non-finite | **n/a**: same |
| J — overflow reach | **n/a**: same |
| K — three precisions | **n/a**: `MixedGraph<T>`'s closure is not generic in a scalar; `T` is vertex data and takes no part |

Rule-specific cases beyond the classes: `r1_does_not_orient_when_the_parent_is_adjacent_to_the_far_end`
(the shield that removes R1's precondition), `a_rule_firing_on_the_last_edge_of_a_pass_still_completes`
(a second pass is required), `directed_edges_take_no_part` and
`a_directed_chord_does_not_repair_an_undirected_cycle` (the projection boundary),
`two_components_are_both_checked`, `a_non_extendable_input_terminates_without_pinning_a_direction`.

## Claims checked against the tree rather than carried from the assessment

| Claim | Verdict |
|---|---|
| `dag_sampling` assumes chordality and does not check | **true** — `dag_sampling/mod.rs:58`, "This is not checked" |
| `brcd_mec` likewise does not check | **false** — `brcd_mec.rs:324` detects it indirectly: an empty AMO result means the component is not chordal. An earlier draft of the chordality docs said neither checks; corrected |
| An incomplete closure produces non-chordal chain components | **unverified**, and the R4 search gives no instance of it. Removed from the docs; it was speculation stated as mechanism |
| `Real::powf(-8, 1/3)` is `NaN` | **true**, measured |
| `Float::cbrt(-8)` is `-2.0` | **true**, measured — so `cbrt` genuinely replaces the workaround |
| `vector_norm_l2` overflows for a representable norm | **true**, measured: `l2([1e200, 1e200])` = `inf` |
| …and underflows | **true**, measured: `l2([1e-200, 1e-200])` = `0.0`. This is the worse half — `inf` announces itself, `0.0` reads as a legitimate zero vector |
| The electroweak solver returns its last iterate on non-convergence | **true** — `radiative.rs:116-156`: the loop `break`s on convergence and otherwise falls through to `Ok(...)`. It has a `NumericalInstability` path for a negative discriminant, none for non-convergence |

## Tasks 2.7–2.8 — phase 3: the defect audit

Order note: the suite was written and observed failing (phase 2) before any implementation existed,
so writing the implementation before the audit does not let it shape the tests. The audit was then
run by mutating the real implementation and reverting each time, rather than against a separate
throwaway. Same guarantee, one implementation.

First pass: **9 of 12 rejected**. Three real gaps, one equivalent mutant.

| Defect | First pass | After |
|---|---|---|
| drop R4 / R3 / R2 | caught | caught |
| orient the wrong direction | caught | caught |
| R1: drop the non-adjacency check | caught | caught |
| R2: reverse the 2-path direction | caught | caught |
| chordality: always accept / off-by-one / do not block the neighbourhood | caught | caught |
| **stop after one sweep** | **missed** | caught |
| **R3: drop the non-adjacency check** | **missed** | caught |
| **R4: drop the non-adjacency check** | **missed** | caught |

Second pass: **12 of 12**.

Why each was missed, and the test added:

- *Stop after one sweep.* `a_rule_firing_on_the_last_edge_of_a_pass_still_completes` did not need a
  second sweep: `undirected_edges` yields ascending pairs, and its enabling edge came first, so one
  sweep sufficed. Replaced by `an_edge_unlocked_by_a_later_edge_needs_a_second_sweep`, where the
  enabling edge `1 — 2` sorts *after* the dependent edge `0 — 1`.
- *R3 and R4 non-adjacency.* Both rule tests used configurations where the pair was already
  non-adjacent, so relaxing the check to `true` changed nothing. Added
  `r3_does_not_fire_when_the_two_parents_are_shielded` and
  `r4_does_not_fire_when_b_and_d_are_adjacent` — the shielded cases, where the rule must **not**
  fire.

This is the corner-case template's class C: an input where two distinct quantities coincide. The
three misses were all the same shape — testing a rule only where its precondition holds.

### Equivalent mutant

*Chordality: return the longest candidate cycle instead of the shortest.* Not a gap. Every candidate
is `v` plus a shortest path between two non-adjacent neighbours with the rest of `N(v)` blocked; that
path is induced in the blocked subgraph and `v` meets only its endpoints, so every candidate is
already chordless. The contract is "a chordless cycle", not the shortest one, so the mutation changes
nothing the API promises. No test added; recorded here rather than in `.cargo/mutants.toml`, which
covers `cargo mutants` survivors rather than hand-written audit defects.

## Task 2.11 — a live defect found while wiring the chordality check

`brcd_mec::mec_size` returned **`Ok(0)`** for a non-chordal chain component, while
`representative_dag` and `mec_sample_dag` returned `Err(NotACpdag)` for the same input. Measured on a
chordless 4-cycle before the fix:

```
mec_size       = Ok(0)
representative = Err(BrcdError(NotACpdag))
```

No CPDAG has an equivalence class of size zero — every one contains at least itself — so this was a
wrong answer, not a small class, and it disagreed with its two neighbours in the same file.
`mec_size` multiplied by an empty AMO count; `build_member` had an explicit empty-AMO guard that
`mec_size` lacked.

An existing test pinned it: `non_chordal_cycle_is_not_a_cpdag_for_member_building` asserted
`mec_size(&g) == Ok(0)`, and its comment recorded the divergence without questioning it. The API was
wrong, so the test changed with it.

Fix: `validate_cpdag` now rejects a non-chordal undirected projection with `NotACpdag`, so all three
entry points agree and fail before enumeration.

This is a better instance of the failure the stage was aimed at than the R4 omission is. It is live,
reachable through public API, and self-inconsistent within one file.

### Not done: the second MEC path

`dag_sampling::mec_size` returns `T`, with no error channel, and its own docs say chordality "is
**not** checked; an input that violates the assumption may yield a wrong count". Wiring the check
there means changing a public signature to `Result`, which touches a verification harness and three
test files. That is a breaking change to a published crate and a decision for the maintainer, so it
is left open rather than made unilaterally. The spec requirement asking for the check on *both* MEC
paths is therefore not yet met.

## Task 2.11 completed — `dag_sampling::mec_size` now returns `Result`

Signature changed from `-> T` to `-> Result<T, BrcdError>`, on the maintainer's decision. Breaking
for the published crate.

`dag_sampling` already used `BrcdError` — `sample_dag` returns it — so the check reuses `NotACpdag`
rather than introducing a second error vocabulary for the same condition. The chordality check went
into `dag_sampling::sample::validate_cpdag`, a verbatim duplicate of BRCD's, so `sample_dag` and
`representative_dag` gain it too and their "assumed chordal, not checked" precondition line is gone.

### The callers it exposed

Two are inside BRCD's own scoring path, and they are the reason this mattered:

- `brcd_algo.rs:293` — `sizes.push(mec_size(&aug)?)`, whose `sizes` become `total` and then
  `log_p_g`. An unchecked count entered the posterior here.
- `brcd_algo.rs:359` — `let size = mec_size(&aug)?`, folded into a log-likelihood.

Both sat next to a `sample_dag(...)?` or `representative_dag(...)?` on the *same graph*, which was
already `?`-propagating a `BrcdError`. So the surrounding code was prepared for the error all along;
only the count was not.

The rest are one verification harness and four test files, updated to `?` or `.expect`.

### Tests added

`a_non_chordal_component_is_refused_by_both_counters` — the clique-picking counter and the
enumeration oracle now agree on a chordless 4-cycle, where previously one returned a number and the
other `Ok(0)`. `a_chordal_component_is_still_counted` pins that the check rejects only what it
should, using the same 4-cycle plus a chord.

## Task 2.14 — mutation testing

| File | Mutants | Caught | Survivors |
|---|---|---|---|
| `meek/rules.rs` | 17 | 17 | 0 |
| `meek/mod.rs` | 11 | 10 | 0 (1 unviable) |
| `chordality/mod.rs` | — | — | **not run** |

`chordality/mod.rs` was stopped before completing. Its behaviour is covered by the phase-3 audit,
which rejected all four chordality defects introduced there (always-accept, off-by-one on cycle
length, not blocking the neighbourhood, and the longest-vs-shortest equivalence). The mutation run
is outstanding work, not a passed check.

## Task 2.15 — verification

- `cargo test`: 1652 topology, 330 algorithms, 0 failures.
- `bazel test //deep_causality_unified_math/deep_causality_topology:types/mixed_graph` — 10/10, both
  new test files picked up by the existing glob.
- `bazel test //deep_causality_algorithms:all` — 37/37.
- `cargo clippy --all-targets` — 0 warnings on both crates.
- `make check_examples` — the new harness has its Bazel target.
- `verification_base` reproduces its previous ranking: root cause Y at posterior 1.0.

Not done: coverage measurement over the added files.
