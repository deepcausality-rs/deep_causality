<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

Four stages, one per group after the protocol. Every stage runs the same five phases in order —
**P1** an API with unimplemented bodies, **P2** the suite written against it and observed failing,
**P3** the suite audited against deliberate defects, **P4** implementation, **P5** mutation testing —
and no phase starts before the previous one's exit condition is met. A phase-4 task is blocked until
its group's phase-3 task is checked.

Group 1 is done once and binds the rest. Groups 2 and 3 are independent of each other; groups 4, 5
and 6 want group 3 first. No group is done until `bazel test //...` is green for it.

The verdict-carrier stage was cut on 2026-09-04: it had no consumer once the engine work was
deferred, and its stated justification did not survive checking. See
`openspec/changes/deferred/num-verdict-algebra/`.

## 0. Precondition: reconcile the tier documentation

The stats crate adds a row to tables that disagree with each other and with the manifests today.
Adding a row first bakes four existing errors in.

- [ ] 0.1 Reconcile the three tier representations in `deep_causality_unified_math/README.md` — the ASCII block, the markdown crate table, and `graph.png` — against the manifests. The block and the figure put `calculus` and `fft` at tier 4 and `num_complex`/`num_dual` at 3; the table puts them at 3 and 2
- [ ] 0.2 Correct `AGENTS.md`'s tier block, which omits the `deep_causality_haft` dependency that `num_complex` and `num_dual` both declare, and places both at a tier its own stated derivation contradicts
- [ ] 0.3 Regenerate `graph.png` by the recipe in `Bazel.md`, and confirm it writes where the README expects rather than to the repository root
- [ ] 0.4 Verify: all three representations and `AGENTS.md` agree with the manifests, checked by deriving the tiers from `Cargo.toml` rather than by reading them

## 1. The test-first protocol

- [x] 1.1 Write the per-stage corner-case enumeration template: empty input, single element, the case where two distinct quantities coincide, the index expression that degenerates, each documented threshold and its two sides, zero and negative and domain-boundary values, non-finite inputs where the type admits them, and every precision-generic case at `f32`, `f64` and `Float106`
- [x] 1.2 Write the defect-class template for phase 3: off-by-one in an index or loop bound, flipped comparison, inverted sign, changed constant factor, dropped normalisation, loosened tolerance, skipped case, removed guard, and a returned value replaced by a plausible neighbour
- [x] 1.3 Write the anti-circularity checklist used in review: an expectation is acceptable only from a hand-evaluated closed form written as a literal, a cited published value, a demonstrably different algorithm, an algebraic invariant, or a generated property — and never from the code under test, its formula retyped, a helper sharing either, or the implementation being replaced

## 2. C1 — Meek orientation completeness

- [x] 2.1 Write the small-graph search: enumerate graphs to a stated vertex bound, add `F → R` arcs and each cut configuration as BRCD's Algorithm 1 does, close under R1–R3 and under R1–R4, and compare
- [x] 2.2 Run the search and record the bound and the result; if a difference exists, commit the smallest witness as a regression fixture and record the propagation path through the MEC size to the posterior
- [x] 2.3 **P1** Declare the closure on `MixedGraph<T>` in `deep_causality_topology` — the R1–R4 method, the R1–R3 method, and the chordality check — all with unimplemented bodies, at the abstraction level `mixed_graph/acyclicity/` already uses
- [x] 2.4 **P2** Write the suite against the unimplemented surface: each rule fires where it applies and does not where it does not; the closure reaches a fixpoint; a pattern is oriented identically by both closures; an F-augmented graph is closed under all four rules; the two closures differ exactly on the edges R4 orients; a chordal component is accepted and a chordless four-cycle is refused; a non-extendable PDAG terminates without pinning a direction
- [x] 2.5 **P2** Enumerate and cover the corner cases: the empty graph, one vertex, two vertices, a graph with no undirected edge, a graph with no directed arc, a complete graph, a graph where a rule fires on the last edge of a pass, and the witness from 2.2 if one exists
- [x] 2.6 **P2** Verify every test fails with the unimplemented panic and record the failing run and its test count
- [x] 2.7 **P3** Audit: introduce each defect class into a throwaway closure — drop R4, drop R3, orient the wrong direction, omit the non-adjacency check in each rule, terminate after one pass instead of at a fixpoint, accept a non-chordal component — and confirm the suite rejects each
- [x] 2.8 **P3** Widen the suite for any defect it misses, repeat the audit, then discard the throwaway
- [x] 2.9 **P4** `git mv` `brcd_meek.rs` into `deep_causality_topology/src/types/mixed_graph/meek/`, adapt it to inherent methods, and register the module
- [x] 2.10 **P4** Implement R4 as: orient `a → b` when there are `c, d` with `d — a — c` undirected, `d → c → b`, and `b` not adjacent to `d`
- [x] 2.11 **P4** Implement the chordality check and wire it into both MEC paths — the `dag_sampling` clique-picking path and the BRCD MEC path
- [x] 2.12 **P4** Document both closures: the completeness hypothesis of each, the reference the R1–R3 entry point matches, and that neither validates its input (the search bound is recorded in the stage notes, not the module doc)
- [x] 2.13 **P4** Repoint the three call sites and `git mv` the test file into topology
- [x] 2.14 **P5** Run `scripts/mutants.sh` over the moved and added files; kill every survivor or record an escaped, `comm`-confirmed equivalence — `rules.rs` 17/17 and `meek/mod.rs` 10/10, no survivors; `chordality/mod.rs` not run
- [x] 2.15 Verify: tests green under both build systems, clippy clean, and BRCD's existing corpus produces its previous orientations

## 3. C3 — `Real::cbrt` and `RealField: ToPrimitive`

- [ ] 3.1 **P1** Declare `Real::cbrt` and add `ToPrimitive` to `RealField`'s supertraits, with the blanket and `Dual` bodies unimplemented; confirm the blanket's where-clause needs no change because `Float: NumCast: ToPrimitive`
- [ ] 3.2 **P2** Write the suite: `cbrt` on a positive, a negative and a zero argument at each of the three scalars; the dual chain rule `b/(3·cbrt(a)²)`; the negative-argument case that `powf(1/3)` returns `NaN` for; the infinite dual component at zero, matching `sqrt`'s existing behaviour at the same point; `RealField` code converting to a primitive without restating the bound; `Dual` still not implementing `RealField`
- [ ] 3.3 **P2** Cover the corner cases: `cbrt` of exact cubes at each precision, of a value near the type's maximum and minimum positive, of a negative zero, and of a non-finite input
- [ ] 3.4 **P2** Verify every test fails for the intended reason and record the run
- [ ] 3.5 **P3** Audit: drop the sign handling so negatives return `NaN`, replace the derivative denominator's `3` with `2`, square instead of cube-rooting in the derivative, and return the argument unchanged — confirm each is rejected
- [ ] 3.6 **P4** Implement `cbrt` in the blanket by forwarding to `Float::cbrt`, and in `Dual` by the chain rule with no guard at the singularity
- [ ] 3.7 **P4** Retire `signed_cbrt` in the coherent-structures kernel, replacing the sign branch over `powf` with a direct call
- [ ] 3.8 **P4** Retire the integer-floor stepping loop in the DEC surface-force sampler — unbounded, bidirectional, and carrying three silent `unwrap_or_else` substitutions — converting through `ToPrimitive` instead
- [ ] 3.9 **P4** Remove the seven redundant `RealField + ToPrimitive` bound restatements
- [ ] 3.10 **P4** Enumerate the implementors of `Real` and of `RealField`, and record which the additions oblige to change — the compatibility blast radius, not the dependent count
- [ ] 3.11 **P5** Run `scripts/mutants.sh` over the added and edited files and resolve every survivor
- [ ] 3.12 Verify: `bazel test //...` is green, every retired site's existing tests pass unchanged, and no workaround from the retirement list remains

## 3b. Physics sampling precision

Split out of group 3: this is `rand`/`Distribution` work with no relation to `cbrt` or `ToPrimitive`,
and it is not the comment fix it was written as. The two module docs give a second reason for
sampling at `f64` — that for a wider `R` "the sampling noise sits at the f64 floor anyway, so the
lift does not lose meaningful entropy" — which is a claim about the physics, not about what `rand`
can do. Acting on it changes the random stream, and therefore every seeded expectation downstream.

- [ ] 3b.1 Confirm the two comments are stale: `rand` implements `Distribution<Float106>` for
      `StandardUniform`, `Open01`, `OpenClosed01` and `StandardNormal`
- [ ] 3b.2 Decide whether the entropy claim holds — whether Lund-model sampling at `Float106` is
      distinguishable from sampling at `f64` and lifting. This is a physics question, not a
      capability question, and it gates the rest of the group
- [ ] 3b.3 If it does not hold: correct the comments only, and record that the `f64` sampling stays
      for a stated reason rather than a stale one
- [ ] 3b.4 If it holds: route both sites through `RealRng`, and record the stream change and every
      seeded test whose expectations move with it

## 4. C2 — `deep_causality_stats`

- [ ] 4.1 **P1** Scaffold the crate at `deep_causality_unified_math/deep_causality_stats` with its manifest, `BUILD.bazel`, `[lints] workspace = true`, README, error type and `src/utils_tests/`; declare dependencies on `num`, `algebra` and `linear` only
- [ ] 4.2 **P1** Declare the full public surface with unimplemented bodies: entropy and conditional entropy taking a base and a zero policy, log-sum-exp and the two-term form, mean and both variance forms, Pearson, ridge in materialised and streaming forms, logistic IRLS, the Gaussian log-density, and equal-width and equal-frequency binning — every signature generic in its scalar, none naming a concrete float
- [ ] 4.3 **P1** Confirm the excluded functions are absent: cross-entropy, mutual information, KL divergence, Jensen–Shannon divergence, Hellinger distance and the Bhattacharyya coefficient
- [ ] 4.4 **P2** Write the entropy suite: uniform against `log2 n`; a degenerate distribution at exactly zero; the two bases differing by exactly `ln 2`; the two zero policies differing on an entry positive but below epsilon; empty and negative inputs refused with typed errors; conditional entropy equal to `H(X)` under independence and to zero under deterministic dependence, and never negative
- [ ] 4.5 **P2** Write the log-sum-exp suite: agreement with the naive form inside its safe range, finiteness where the naive form overflows, accuracy where it underflows, and the documented outcome for infinite and empty inputs
- [ ] 4.6 **P2** Write the regression suite: ridge against a closed-form solution, monotone coefficient-norm decrease under increasing penalty, agreement between the materialised and streaming forms over the filtered design, a rank-deficient design refused at zero penalty; IRLS non-convergence returning a typed error with its iteration count, and separable data never returning an unbounded coefficient as success
- [ ] 4.7 **P2** Write the remaining suites: descriptive statistics with the `n/(n-1)` ratio between variance forms and a typed error on a one-element corrected variance; Pearson against a closed form, exact at perfect correlation, refusing zero variance, applying its stated missing-data policy; the Gaussian log-density against a closed form, integrating to one, refusing a non-positive scale, with the variance-or-deviation parameterisation pinned by a case where the two differ; binning's edge convention at every boundary, the maximum in the last bin, a constant column handled explicitly, and equal-frequency balance when `k` divides `n`
- [ ] 4.8 **P2** Enumerate and cover the corner cases, and run every numeric test at all three precisions
- [ ] 4.9 **P2** Verify every test fails with the unimplemented panic and record the run and test count
- [ ] 4.10 **P3** Audit: drop Bessel's correction, change the entropy base, skip at epsilon instead of zero, remove the max-shift from log-sum-exp, drop the ridge penalty term, halve the Gaussian normalisation, place the maximum one bin past the end, and invert the IRLS convergence test — confirm each is rejected
- [ ] 4.11 **P3** Widen the suite for any defect it misses, repeat, discard the throwaway
- [ ] 4.12 **P4** Implement the crate against the audited suite
- [ ] 4.13 **P4** Register the crate: root dependency table at two-digit precision, `AGENTS.md` tier block, unified-math README crate table and tier diagram
- [ ] 4.14 **P5** Run `scripts/mutants.sh` over the crate and resolve every survivor
- [ ] 4.15 Verify: the crate's suite is green at full coverage under both build systems, and the unused-dependency check passes — this gates every task in group 5

## 5. C2 — statistics consumer migration

- [ ] 5.1 Confirm 4.15 is checked; no task in this group starts before it is
- [ ] 5.2 Migrate SURD's `T` and `Option<T>` entropy and conditional-entropy paths onto one shared implementation, the `Option` path supplying its presence policy as a parameter, with each path selecting the parameters that reproduce its current output
- [ ] 5.3 Verify: both SURD paths produce identical outputs to their pre-migration values on the existing corpus, and their existing tests pass unchanged
- [ ] 5.4 Migrate BRCD: both ridge forms, the logistic gate, the Gaussian log-density, log-sum-exp at its three sites, and the mean and Bessel-corrected variance
- [ ] 5.5 Migrate mRMR's Pearson, and confirm it now computes in the caller's scalar rather than internally in `f64`
- [ ] 5.6 Migrate the discovery binning routines
- [ ] 5.7 Migrate the physics entropy kernel to bits, make its name state its base, update its existing tests which pin the nats result, and keep its parallel wrapper for the `MaybeParallel` bound the crate does not carry
- [ ] 5.8 Migrate quantum's mean and unbiased variance where the surface admits it, and record where it does not because the function is `f64` by construction at both ends
- [ ] 5.9 Verify: no superseded implementation remains anywhere in the workspace, and every migrated call site resolves to the crate
- [ ] 5.10 Verify: a previously `f64`-internal path called at `Float106` now carries `Float106` accuracy on an input whose exact result is known, and the same path at `f64` is unchanged
- [ ] 5.11 Record every changed result with its reason; confirm no consumer test was edited to make a failure disappear

## 6. C4 — `linear` adoption

- [ ] 6.1 **P1** Declare the scaled-form `vector_norm_l2` and a `to_row_major` override on `CsrMatrix` with unimplemented bodies
- [ ] 6.2 **P2** Write the defect suite: a vector with a component near the type's maximum returning a finite correct norm; one with components near the minimum positive value not underflowing to zero; the ordinary range unchanged; all three cases at all three precisions; a `CsrMatrix` decomposed sparse and dense agreeing on eigenvalues
- [ ] 6.3 **P2** Verify the tests fail — the overflow test against today's implementation fails by returning infinity, which is the defect
- [ ] 6.4 **P3** Audit: drop the scaling factor, scale by the smaller component instead of the larger, omit the zero-maximum guard — confirm each is rejected
- [ ] 6.5 **P4** Implement both fixes. The conversion override fixes every `MatrixView` algorithm at once — `eigen_hermitian`, `qr`, `svd`, `cholesky` — and is latent: no caller passes a `CsrMatrix` to any of them today
- [ ] 6.6 Build the classified inventory of every hand-rolled linear-algebra site across the nine consumer crates and `examples/`, each marked replace, replace-with-care or keep, with its reason
- [ ] 6.7 **P1–P4** Replace the *replace* class through the five phases: the open-coded complex modulus and multiplication across ten files, the five copies of the entrywise max-modulus residual collapsed to one, and the cofactor inverses where a general path is no slower. Quantum's Frobenius norm is **not** in this class — delegating it buys nothing, since `modulus_squared` is the same direct form
- [ ] 6.8 **P1–P4** Add the finiteness guard in `markov_pairs`, the one place an overflowing Frobenius norm changes a decision: it feeds `CommutatorTolerance::threshold` unguarded, so entries above about `1.34e154` send the threshold to infinity
- [ ] 6.8 **P4** Handle the *replace-with-care* class one at a time: benchmark the 17-state filter kit before and after and revert on regression; change the ideal-MHD CSR matvec from a silent column skip to a typed error and pin the new behaviour with a test
- [ ] 6.9 **P4** Record a reason at each *keep* site — the closed-form symmetric 3×3 eigensolver and the written-out 3×3 products — so the next reader does not re-litigate it
- [ ] 6.10 **P1–P4** Collapse the three open-coded reachability pre-passes in `deep_causality` onto one, preserving each site's behaviour including the not-frozen error and the out-of-range start
- [ ] 6.11 **P4** Add the missing dependency edges the replacements need, in manifests and Bazel targets
- [ ] 6.12 **P4** Record as breaking, with implementors and matchers enumerated, any variant added to `LinearErrorEnum` or any method added to `ultragraph`'s pathfinding trait; update both implementors in the same change
- [ ] 6.13 **P5** Run `scripts/mutants.sh` over the added and edited files and resolve every survivor
- [ ] 6.14 Verify: `bazel test //...` is green, every classified site is resolved, and each behaviour change is recorded with its old and new behaviour

## 7. C6 — solver convergence reporting

Cut down from an operator family (~3100 LOC) to the defect it was justified by. A generic scalar root
finder would replace 55 lines; the three bisections already validate their brackets and already
return typed errors, with caps they cannot reach; dual-number Newton has no caller. What remains is
that four solvers return an unconverged iterate in silence.

- [ ] 7.1 **P1** Declare the non-convergence error path for each of the four sites, with unimplemented bodies where a body is needed: `radiative.rs`, `two_body.rs`, `ks_propagator.rs`, `brcd_gate.rs`
- [ ] 7.2 **P2** Write the suite: each of the four driven to its cap returns its typed error; each converged path is unchanged; and the Kepler case `e = 0.9999, M = 1e-6` returns the root, taken from an independent bisection rather than from the solver under test
- [ ] 7.3 **P2** Verify every test fails for the intended reason and record the run
- [ ] 7.4 **P3** Audit: return the last iterate anyway, invert the convergence test, error before the cap, and accept a residual that is not small — confirm each is rejected
- [ ] 7.5 **P4** Measure and record, per site, whether its non-convergence is reachable at the inputs its callers supply. The electroweak solver converges in 5 of 20 iterations at its shipped constants, so it is latent; mark each of the four live or latent
- [ ] 7.6 **P4** Implement the signalling. Where the step test runs out but the residual is already satisfied — `two_body` at high eccentricity — widen the stopping test rather than erroring, so a correct answer is not turned into a failure
- [ ] 7.7 **P4** Record as breaking any variant added to `PhysicsErrorEnum` or `BrcdErrorEnum`; neither is `#[non_exhaustive]`. Record whether a kernel that does not allocate today now does, since every numerical `PhysicsErrorEnum` variant carries a `String`
- [ ] 7.8 **P5** Run `scripts/mutants.sh` over the edited files and resolve every survivor
- [ ] 7.9 Verify: `bazel test //...` is green, `deep_causality_calculus`' public surface is unchanged, and every converged path produces its previous value

## 8. Programme close

- [ ] 8.1 Verify every stage recorded its phase-2 failing run, its phase-3 audit result and its phase-5 mutation report
- [ ] 8.2 Verify no test was added in a commit later than the one implementing its behaviour, across all four stages
- [ ] 8.3 Update `openspec/notes/unified_math/unified_math_next.md` with the corrections this change established, and note that its item 9 is deferred to a dedicated change rather than done here
- [ ] 8.4 Verify the deferred notes at `openspec/changes/deferred/engine-precision-parametric/` and `.../num-verdict-algebra/` still match the tree, so the dedicated changes start from accurate findings
- [ ] 8.5 Update `deep_causality_unified_math/README.md`: the new crate, the tier diagram, and the trait table if the `Real` change alters it
- [ ] 8.6 Run `make format && make fix`, then `bazel test //...` over the whole workspace
- [ ] 8.7 Prepare the commit messages, one per stage, and ask the maintainer to commit
