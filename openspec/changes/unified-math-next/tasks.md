<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

Five stages — C1 Meek, C3 the real bound, C2 statistics (split across two groups), C4 linear
adoption, C6 solver convergence. Every stage runs the same five phases in order —
**P1** an API with unimplemented bodies, **P2** the suite written against it and observed failing,
**P3** the suite audited against deliberate defects, **P4** implementation, **P5** mutation testing —
and no phase starts before the previous one's exit condition is met. A phase-4 task is blocked until
its group's phase-3 task is checked.

Group 1 is done once and binds the rest. Groups 2 and 3 are independent of each other; groups 4, 5,
6 and 7 want group 3 first. No group is done until `bazel test //...` is green for it.

The verdict-carrier stage was cut on 2026-09-04: it had no consumer once the engine work was
deferred, and its stated justification did not survive checking. See
`openspec/changes/deferred/num-verdict-algebra/`.

## 0. Precondition: reconcile the tier documentation

The stats crate adds a row to tables that disagree with each other and with the manifests today.
Adding a row first bakes the existing errors in.

Of the four representations, two were already correct: the README's ASCII block and `graph.png`.
The errors were in the README's crate table and in `AGENTS.md`.

- [x] 0.1 Reconcile the three tier representations in `deep_causality_unified_math/README.md` — the ASCII block, the markdown crate table, and `graph.png` — against the manifests. Four table rows were wrong: `num_complex` and `num_dual` at 2 rather than 3, `calculus` and `fft` at 3 rather than 4. The table is now sorted by the corrected tier, and states why the two number types sit above `num_rational`
- [x] 0.1a Correct two further README claims found while checking: "Two edges leave the folder" counts only `ast`, omitting `par`, which `fft` and `topology` both require — two dependencies leave, over four edges; and the sentence introducing the optional external dependencies was broken mid-clause
- [x] 0.2 Correct `AGENTS.md`'s tier block, which omits the `deep_causality_haft` dependency that `num_complex` and `num_dual` both declare, and places both at a tier its own stated derivation contradicts. Regenerated from the manifests rather than hand-patched. This also fixed a fifth error the task did not record: `quantum`'s dependency list omitted `homology`, `linear` and `num_rational`
- [x] 0.3 `graph.png` needs no regeneration: it is rendered from the artifact at `https://claude.ai/code/artifact/7808f976-a88c-42e3-a919-9c85c5795360`, which already carries the derived tiers, and it agrees with the manifests. The `Bazel.md:84` recipe this task pointed at is a generic `bazel query rdeps(//..., //deep_causality_haft, 1) | dot` for reverse dependencies of `haft` — it produces a different graph and was never this figure's source
- [x] 0.4 Verify: all three representations and `AGENTS.md` agree with the manifests, checked by deriving the tiers from `Cargo.toml` rather than by reading them — `scripts/check_tiers.py`, run against a reintroduction of each fixed defect to confirm it fails

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

- [x] 3.1 **P1** Declare `Real::cbrt` and add `ToPrimitive` to `RealField`'s supertraits, with the blanket and `Dual` bodies unimplemented; confirm the blanket's where-clause needs no change because `Float: NumCast: ToPrimitive`
- [x] 3.2 **P2** Write the suite: `cbrt` on a positive, a negative and a zero argument at each of the three scalars; the dual chain rule `b/(3·cbrt(a)²)`; the negative-argument case that `powf(1/3)` returns `NaN` for; the infinite dual component at zero, matching `sqrt`'s existing behaviour at the same point; `RealField` code converting to a primitive without restating the bound; `Dual` still not implementing `RealField`
- [x] 3.3 **P2** Cover the corner cases: `cbrt` of exact cubes at each precision, of a value near the type's maximum and minimum positive, of a negative zero, and of a non-finite input
- [x] 3.4 **P2** Verify every test fails for the intended reason and record the run
- [x] 3.5 **P3** Audit: drop the sign handling so negatives return `NaN`, replace the derivative denominator's `3` with `2`, square instead of cube-rooting in the derivative, and return the argument unchanged — confirm each is rejected
- [x] 3.6 **P4** Implement `cbrt` in the blanket by forwarding to `Float::cbrt`, and in `Dual` by the chain rule with no guard at the singularity
- [x] 3.7 **P4** Retire `signed_cbrt` in the coherent-structures kernel, replacing the sign branch over `powf` with a direct call
- [x] 3.8 **P4** Retire the integer-floor stepping loop in the DEC surface-force sampler — unbounded, bidirectional, and carrying three silent `unwrap_or_else` substitutions — converting through `ToPrimitive` instead
- [x] 3.8a **P4** Retire the *second* copy of the floor scan: the sampler carries it twice, in `sample_velocity` and `sample_scalar`, so six silent substitutions rather than the three recorded. Removing the seed also retires the `Vec<LatticeCell<D>>` each function collected only to find it; the bounds guard in the second is preserved through `num_cells(D)`
- [x] 3.9 **P4** Remove the seven redundant `RealField + ToPrimitive` bound restatements
- [x] 3.10 **P4** Enumerate the implementors of `Real` and of `RealField`, and record which the additions oblige to change — the compatibility blast radius, not the dependent count
- [x] 3.11 **P5** Run `scripts/mutants.sh` over the added and edited files and resolve every survivor — `num_dual/dual/dual_number/real.rs`: 137 mutants, **16 missed** on the first run and none of them in `cbrt`, so the new tests already pinned the added code. The 16 were a pre-existing gap with one root cause: all 47 tests used `Dual::variable`, whose ε seed is 1, and at a seed of 1 `f'(a) * self.du` and `f'(a) / self.du` are the same operation. Seeded-derivative tests plus predicate and boundary cases took it to **1 missed**
- [x] 3.11a **P5** Resolve the last survivor by construction rather than exclusion: `Dual::log10` built ten from `two + two + T::one()`, where swapping the first operator gives `two * two + T::one()`, also five — a decision no test can pin. An `exclude_re` entry for it matched three mutants, two of them killable, which is the over-exclusion `.cargo/mutants.toml`'s own header warns about, so the entry was backed out and the constant rebuilt as `three * three + T::one()`, where every operator changes the value if it changes. Not re-measured after the restructure
- [x] 3.12 Verify: `bazel test //...` is green, every retired site's existing tests pass unchanged, and no workaround from the retirement list remains — 1274 Bazel tests pass, clippy clean, 1748 physics and 937 CFD tests unchanged, and the retirement list greps empty outside the stale agent worktrees

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

## 3c. Test oracles in `deep_causality_num`

Directed mid-stage, and not part of the original plan. Group 3 found that `Float106::cbrt` computed
`1/3` in `f64` and widened it, capping the whole Newton iteration at `f64` accuracy on a 106-bit
type. The existing suite could not have caught that: it asserted on `result.hi()` alone against a
`1e-14` tolerance. The scan below asked how much else was shaped that way.

Notes: `openspec/changes/unified-math-next/notes/num-test-oracles.md`.

- [x] 3c.1 Scan the 74 test files (13,406 lines) for circular and tautological assertions and classify what is found: **173** high-word-only assertions across the `float_double` suites, which cannot express a wrong low word, and **7** forwarder-versus-source assertions in `integer_all_types_tests.rs` comparing `Integer::count_ones(x)` with `x.count_ones()`, each on a single input
- [x] 3c.2 Build the oracles: mpmath reference values at 60 decimal places, split into the exact `(hi, lo)` `f64` pair the type stores; the published decimal expansions of π, e, ln 2 and ln 10 checked in **both** words; algebraic invariants; exactness where the result is representable; hand-derived bit counts
- [x] 3c.3 Record the oracle trap, found by hitting it: an argument must reach mpmath as the `f64` the test constructs, not as a decimal literal. Comparing `Float106::from(0.05)` against decimal `0.05` measures the `2.8e-18` conversion gap and reads as a uniform `5e-17` error in `atan`, `asin` and `acos` that is not there. Three defects were nearly reported on that basis
- [x] 3c.4 Set the tolerance from measurement rather than convention: `TOL = 1e-29`, against a measured worst case near `5e-31`. The previous `1e-14` admits an answer with no correct low word at all
- [x] 3c.5 Rewrite `double_transcendental_tests.rs` against the reference tables, the two-word constant checks and the invariants
- [x] 3c.6 Replace the high-word-only assertions in `double_float_tests.rs`, `double_arithmetic_tests.rs`, `double_from_tests.rs` and `double_num_traits_tests.rs`: exact both-word assertions where the result is representable, relative checks otherwise
- [x] 3c.7 Replace the 7 tautologies with hand-derived expectations over varied inputs and corner cases — every bit position, zero, all-ones, `MIN`, `MAX` — and state the endianness expectations against the target rather than against the function under test
- [x] 3c.8 Fix the defects the new tests expose, all of which were passing before: `cbrt`'s `f64`-widened third; `atan` applying its argument reduction once, so a large argument leaves the series near 1 where 80 terms do not converge (`atan(100)` wrong by `8.1e-4`); `asin`/`acos` inheriting that through a ratio that grows near `|x| = 1` (`4.2e-5`); `ln(+∞)` returning `NaN` from `inf + inf/inf − 1`; and `tanh` overflowing to `NaN` above `x ≈ 355` while rounding asymmetrically in the last bits
- [x] 3c.9 Remove `atan`'s shortcut returning exactly π/4 for any argument within `1e-15` of 1 — a correct reduction makes it unnecessary, and it returned a value that was not the arctangent of its input
- [x] 3c.10 Withdraw the two apparent defects that did not survive checking: the `acos(cos y)` and `atan(tan y)` round trips outside the principal branch, and `cosh² − sinh² = 1` at large `x`, are properties of the identities rather than of the implementations
- [x] 3c.11 Verify by negative control: revert each fix in turn and confirm the new suite rejects it — 2, 10, 1, 1, 3, 1 and 2 tests respectively. The previous suite caught one of the seven
- [x] 3c.12 Verify: `bazel test //...` green, `cargo clippy --workspace --all-targets` clean. Clippy's `approx_constant` on the literal high words is resolved by naming the constants — `exp(1) = e`, `asin(1/2) = π/6`, `sqrt(2) = √2` — which states the identity, rather than by an `allow`
- [x] 3c.13 **P5** Run `scripts/mutants.sh` over `float_106_impl.rs` and resolve every survivor. Not run: `deep_causality_num` is the workspace's most-depended-on crate and each mutant costs a full build and test run for it

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
- [ ] 6.9 **P4** Handle the *replace-with-care* class one at a time: benchmark the 17-state filter kit before and after and revert on regression; change the ideal-MHD CSR matvec from a silent column skip to a typed error and pin the new behaviour with a test
- [ ] 6.10 **P4** Record a reason at each *keep* site — the closed-form symmetric 3×3 eigensolver and the written-out 3×3 products — so the next reader does not re-litigate it
- [ ] 6.11 **P1–P4** Collapse the three open-coded reachability pre-passes in `deep_causality` onto one, preserving each site's behaviour including the not-frozen error and the out-of-range start
- [ ] 6.12 **P4** Add the missing dependency edges the replacements need, in manifests and Bazel targets
- [ ] 6.13 **P4** Record as breaking, with implementors and matchers enumerated, any variant added to `LinearErrorEnum` or any method added to `ultragraph`'s pathfinding trait; update both implementors in the same change
- [ ] 6.14 **P5** Run `scripts/mutants.sh` over the added and edited files and resolve every survivor
- [ ] 6.15 Verify: `bazel test //...` is green, every classified site is resolved, and each behaviour change is recorded with its old and new behaviour

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
