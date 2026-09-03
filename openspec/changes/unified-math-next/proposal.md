<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Why

The assessment in [`unified_math_next.md`](../../notes/unified_math/unified_math_next.md) read
`deep_causality_unified_math/` against its four consumers and ranked nine areas of work. A
sixteen-agent verification sweep then re-read every claim against the tree. It confirmed most of the
duplication, refuted a third of the ranking's premises, and turned up something the assessment did
not look for: **three of the findings are live defects, not untidiness.**

The largest is in the Meek orientation rules. `brcd_meek.rs:13-24` omits Meek's R4 and defends the
omission on one precondition — that the graph carries no orientation outside its v-structures. BRCD's
own paper, [Algorithm 1](../../../deep_causality_algorithms/papers/30078_Root_Cause_Analysis_of_F.pdf),
falsifies that precondition on line 3: it adds `F → R` as a directed arc, then applies Meek to the
result, and the prose fixes a cut configuration first. Both are background knowledge in Meek's sense,
which is exactly the hypothesis R1–R3 do not cover. The paper's Corollary 4.2 — *Completeness of
Algorithm 1* — rests on that closure being complete, so an incomplete closure yields under-oriented
PDAGs presented as I-CPDAGs, and the error flows through the clique-picking MEC size into
`p(G|R) = Q_i/T` and out into the ranked root causes. It stays plausible the whole way.

Two smaller defects share a shape: `vector_norm_l2` is a naive `Σ modulus_squared` that returns
`inf` for a representable norm, and quantum open-codes the same direct form in at least four places —
while `Normed::modulus` next door is written in the scaled form precisely to avoid it.

The rest is the ordinary consequence of a stack that grew from the bottom up. `deep_causality_linear`
landed after most of its consumers were written, so they still carry the linear algebra they were
built with. The engine's `f64` aliases predate unified math too — but unpinning those turned out to
be a design question rather than a cleanup, so this change adds only the carrier instances that work
will need and leaves the aliases alone.

## What Changes

Five stages plus one small carrier addition. Each is independently shippable; C1, C3 and C5 are
mutually independent.

- **C1 — Meek completeness.** Move the Meek closure from `deep_causality_algorithms` into
  `deep_causality_topology` beside `acyclicity`, as inherent methods on `MixedGraph<T>`. Add R4, so
  the closure is complete in the presence of background knowledge. Keep the R1–R3 closure as a
  separately named, documented entry point for reference parity. Add the chordality precondition
  check that clique-picking assumes and never verifies — a coupled defect, since an incomplete
  closure is one way a component stops being chordal.
- **C3 — `Real::cbrt` and `RealField: ToPrimitive`.** Two additions only. `hypot`, `mul_add`, `max`
  and `min` are **dropped** from the assessment's list: the first has no caller outside
  `deep_causality_num` itself, and the other three are not mathematics a dual number can honour.
  Retire the workarounds the additions make unnecessary — otherwise the stage adds surface and
  removes nothing. **BREAKING** for the nineteen manifests naming `deep_causality_algebra`.
- **C2 — `deep_causality_stats`.** A new crate at **tier 4** over `num`, `algebra` and `linear`.
  Not tier 3, which is arithmetically impossible over `linear`; and **no `rand` dependency**, since
  none of the functions with a consumer uses randomness. Absorbs the three entropy implementations,
  the three log-sum-exp copies, the three Gaussian log-density sites, ridge, logistic IRLS, Pearson,
  descriptive statistics and binning. The nine functions in the assessment's list with no consumer
  today are **not** built.
- **C4 — adopt `linear`.** Retire the hand-rolled matrix kits, cofactor inverses, CSR matvecs and
  Frobenius norms across the consumer crates, each classified *replace*, *replace-with-care* or
  *keep* rather than migrated wholesale. Fix `vector_norm_l2`'s overflow and `eigen_hermitian`'s
  silent densification of a `CsrMatrix`. Collapse the three open-coded reachability pre-passes in
  `deep_causality` onto one.
- **C5 — the verdict carrier at every scalar.** Add the missing `Verdict` instances at `f32` and
  `Float106` beside the trait in `deep_causality_algebra`. Two implementations, no new dependency
  edge. **Unpinning the engine's numeric aliases is deferred** to a change of its own: the aliases in
  `deep_causality_core` and `deep_causality` stay exactly as they are, and `ScalarValue` is not
  touched. They are an early expression of precision-as-a-parameter and reworking them is a design
  question, not a mechanical unpinning. The findings from this investigation are recorded at
  [`changes/deferred/engine-precision-parametric/`](../deferred/engine-precision-parametric/spec.md)
  so the dedicated change starts from them.
- **C6 — root finding in `calculus`.** Bisection, Newton over an explicit or dual-number derivative,
  and a fixed point that signals non-convergence instead of returning its last iterate. `no_std`
  throughout.

Every stage follows one five-phase test-first cycle: a compiling API with unimplemented bodies, then
the full suite written against it and observed failing, then an audit of the suite against deliberate
defects, then implementation, then mutation testing. This is not new process — it is
[`linear-test-first-development`](../../specs/linear-test-first-development/spec.md) generalised from
one crate to a five-stage programme.

## Capabilities

### New Capabilities

- `meek-orientation-completeness`: the Meek closure as a `MixedGraph` operation — R1–R4, the
  R1–R3 parity entry point, the chordality precondition, and what each guarantees.
- `statistics-crate-identity`: `deep_causality_stats` — its tier, dependency set, scalar contract,
  and the scope boundary that keeps consumerless functions out.
- `statistics-descriptive-and-information`: the function surface, and in particular the entropy
  family's base and zero-policy parameters, which exist because the three implementations being
  replaced disagree on both.
- `statistics-consumer-migration`: how SURD, BRCD, physics and discovery move onto the crate, and
  how the three-way entropy divergence is resolved rather than averaged.
- `real-bound-cbrt`: `Real::cbrt` and `RealField: ToPrimitive`, the dual-number semantics of `cbrt`
  at and around zero, and the retirement of the workarounds they replace.
- `linear-adoption`: the consumer-side retirement of hand-rolled linear algebra, its
  replace/replace-with-care/keep classification, and the two defects inside `linear` itself.
- `root-finding-operators`: root finders in `calculus` under `no_std`, each carrying a convergence
  signal.
- `unified-math-tdd-protocol`: the five-phase cycle, binding on every stage.

### Modified Capabilities

- `num-verdict-algebra`: gains `f32` and `Float106` verdict instances, and its owning crate is
  corrected from `deep_causality_num` to `deep_causality_algebra`, where the trait moved during the
  numeric crate split. Only `bool`, `f64`, the probability carrier and `Uncertain` are instantiated
  today, and that gap is the prerequisite the deferred alias work will need on day one.
- `rand-realfield-sampling`: gains `shuffle` on the `Rng` trait, against two verbatim Fisher–Yates
  duplicates. The capability already specifies the `RealRng` layer, so the assessment's call for
  "blanket `Distribution` impls over `RealField`" is **withdrawn** — it cannot compile (E0119 against
  the existing `Distribution<u64>/<u32>/<bool>`), and the generic sampling it asked for already ships.

## Impact

**New crate.** `deep_causality_unified_math/deep_causality_stats` at tier 4. Picked up by the
workspace glob, but needs its own `Cargo.toml`, `BUILD.bazel`, `[lints] workspace = true`, an entry
in the root dependency table at two-digit precision, and additions to `AGENTS.md`'s tier block and
`deep_causality_unified_math/README.md`.

**Breaking.** `deep_causality_algebra` gains two trait obligations; nineteen manifests name it.
`ultragraph`'s `PathfindingGraphAlgorithms` and `deep_causality_linear`'s `LinearErrorEnum` are both
published, both public, and neither is `#[non_exhaustive]` — any addition to either is breaking and
is called out where it arises.

**Behavioural.** C1 changes the orientation output of BRCD wherever R4 fires; the counterexample
search in its first task is what establishes whether that is anywhere. C4's CSR matvec change turns a
silent skip of out-of-range columns into a typed error. Both are stated, not incidental.

**Behavioural, second instance.** The physics entropy kernel moves from nats to bits, changing a
published kernel's returned value by a factor of `ln 2` and bringing it into agreement with both SURD
implementations. Its name is made to state its base. Its existing tests pin the nats result and
change deliberately alongside it.

**Crates touched.** `topology`, `algebra`, `num`, `linear`, `rand`, `calculus`, the new `stats`; and
`deep_causality`, `_algorithms`, `_discovery`, `_physics`, `_quantum`, `_cfd`, `ultragraph` as
consumers. `deep_causality_core` is **not** touched — its aliases are deferred.

**Not done here, and why.** PageRank, Louvain, spectral clustering, k-means, the retrieval family
(BM25, TF-IDF, RRF, MMR, HNSW, LSH), cubic splines, matrix exponential/logarithm/square root, the
Beta/Gamma/Dirichlet/Student-t samplers, a big-integer type, and `partial_trace`'s move into `tensor`
all have no consumer in this repository. Unpinning the engine's numeric aliases and narrowing
`ScalarValue` are deferred to a dedicated change rather than excluded. Interpolation's out-of-range policies are left divergent
because the divergence is contractual: the `clamped` marker is a requirement of
[`weather-table-consumption`](../../specs/weather-table-consumption/spec.md) and SRP's rejection gates
a shipped march step. Structural similarity of causal chains, the topology boost, the Bayesian
provenance score and the bridge-condition ratio are undefined in every document that names them.
