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

- [x] 3b.1 Confirm the two comments are stale: `rand` implements `Distribution<Float106>` for
      `StandardUniform`, `Open01`, `OpenClosed01` and `StandardNormal` — all four are in
      `dist_float_106.rs`, so both comments are wrong about the capability
- [x] 3b.2 Decide whether the entropy claim holds — whether Lund-model sampling at `Float106` is
      distinguishable from sampling at `f64` and lifting. **It holds.** There are five draw sites,
      not the two this task assumed. Three produce discrete outputs — a flavour index, a bool, an
      accept/reject bit — where bits below `2^-53` change the outcome only on a set of measure
      ~`1e-16`. The fourth is an affine map onto the bounded interval `[0.01, 0.99]`, which neither
      amplifies nor compresses, and carries no singular transform and no granularity-set tail. The
      fifth is the Gaussian, where the wider draw is the *narrower* one: the `f64` `StandardNormal`
      is a ziggurat with the Marsaglia tail algorithm and unbounded reach, while the `Float106`
      path is Box–Muller over an `Open01` with a `2^-53` floor, capping `|z|` at a measured
      **8.5717**. Both caps sit past `P ≈ 1e-17`
- [x] 3b.3 Correct the comments only, and record that the `f64` sampling stays for a stated reason
      rather than a stale one — both module docs plus the `generate_transverse_momentum` doc, which
      repeated the stale claim a third time. Physics tests unchanged at 1748, clippy clean
- [x] 3b.4 Not taken: the entropy claim holds, so the sites are not rerouted. Recorded here because
      the cost is part of the finding — a `Float106` stream is not a refinement of the `f64` one. A
      `Float106` uniform consumes two `f64` draws and its normal four, so rerouting would replace
      every seeded sequence rather than extend it, and nothing instantiates these kernels at
      `Float106` today (`generic_real_field_tests.rs` covers `f32` and `f64` only)

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

The site inventory is established: 31 sites, 23 in library crates and 8 in examples, in
`notes/c2-site-inventory.md`. The stage's justification survives contact with the code — three
shipped entropy implementations disagree on base, normalisation and zero policy at once, and two
of them disagree on the unit of the answer.

- [x] 4.0 Establish the inventory by scanning every source file in all 29 library crates and all 16
      example crates, for named statistical functions and for the inline idioms that carry no such
      name. Recorded in `notes/c2-site-inventory.md` with the divergence table
- [x] 4.1 **P1** Scaffold the crate at `deep_causality_unified_math/deep_causality_stats` with its manifest, `BUILD.bazel`, `[lints] workspace = true`, README, error type and `src/utils_tests/`; declare dependencies on `num`, `algebra` and `linear` only
- [x] 4.2 **P1** Declare the full public surface with unimplemented bodies: entropy and conditional entropy taking a base and a zero policy, log-sum-exp and the two-term form, mean, the corrected `n−1` variance, Pearson, ridge in materialised and streaming forms, logistic IRLS, the Gaussian log-density, and equal-width and equal-frequency binning — every signature generic in its scalar, none naming a concrete float
- [x] 4.2a **P1** Do **not** declare a population `÷n` variance. An earlier draft of 4.2 asked for
      "both variance forms"; every variance in the workspace is the corrected `n−1` form —
      `variance_ddof1`, `standard_deviation`, `standard_deviation_qmc`, `uncertain_f64.rs:19` and
      `bridge.rs:97`. There is no `÷n` caller, so building one would break 4.3 and the identity
      spec's rule that the crate implements only functions with a caller in this workspace
- [x] 4.2b **P1** Declare one log-sum-exp for the three slice sites and one two-term `logaddexp`.
      All four shipped copies agree semantically, including the empty case, so no parameter is
      needed to reproduce any of them
- [x] 4.3 **P1** Confirm the excluded functions are absent: cross-entropy, mutual information, KL divergence, Jensen–Shannon divergence, Hellinger distance and the Bhattacharyya coefficient
- [x] 4.4 **P2** Write the entropy suite: uniform against `log2 n`; a degenerate distribution at exactly zero; the two bases differing by exactly `ln 2`; the two zero policies differing on an entry positive but below epsilon; empty and negative inputs refused with typed errors; conditional entropy equal to `H(X)` under independence and to zero under deterministic dependence, and never negative
- [x] 4.5 **P2** Write the log-sum-exp suite: agreement with the naive form inside its safe range, finiteness where the naive form overflows, accuracy where it underflows, and the documented outcome for infinite and empty inputs
- [x] 4.6 **P2** Write the regression suite: ridge against a closed-form solution, monotone coefficient-norm decrease under increasing penalty, agreement between the materialised and streaming forms over the filtered design, a rank-deficient design refused at zero penalty; IRLS non-convergence returning a typed error with its iteration count, and separable data never returning an unbounded coefficient as success
- [x] 4.7 **P2** Write the remaining suites: descriptive statistics with a typed error on a one-element corrected variance — a **behaviour change**, since `variance_ddof1` returns `T::one()` for `len < 2` and that sentinel feeds a Gaussian density, so it is carried into 5.4a and recorded under 5.11 rather than absorbed silently here; Pearson against a closed form, exact at perfect correlation, refusing zero variance, applying its stated missing-data policy; the Gaussian log-density against a closed form, integrating to one, refusing a non-positive scale, with the variance-or-deviation parameterisation pinned by a case where the two differ; binning's edge convention at every boundary, the maximum in the last bin, a constant column handled explicitly, and equal-frequency balance when `k` divides `n`
- [x] 4.8 **P2** Enumerate and cover the corner cases, and run every numeric test at all **four** precisions — `BFloat16` was added beyond the three the task named, because at ~2 decimal digits it is the strongest available test that the implementation assumes nothing about how much precision it has. Its row needed three fixtures the wider suites cannot supply: `cancel_offset` 64 (the binding constraint is the SUM staying inside the exact-integer range of 256, not the values), no thousand-term reduction (an addend below `epsilon · sum` vanishes entirely), and `nudge` 0.25 (the spacing at magnitude ten is `7.8e-2`)
- [x] 4.9 **P2** Verify every test fails with the unimplemented panic and record the run and test count — 351 tests, 330 failing on the panic. The 21 that passed are `entropy_config_tests`: the parameter types have real bodies, so there is nothing unimplemented behind them
- [x] 4.10 **P3** Audit: drop Bessel's correction, change the entropy base, skip at epsilon instead of zero, remove the max-shift from log-sum-exp, drop the ridge penalty term, halve the Gaussian normalisation, place the maximum one bin past the end, and invert the IRLS convergence test — confirm each is rejected
- [x] 4.11 **P3** Widen the suite for any defect it misses, repeat, discard the throwaway — one of the ten survived. Moving the zero cutoff from zero to `epsilon` changed nothing, because the policy tests used entries at `0.0625`, which every precision holds comfortably. The case that separates them is `[1, δ]` with `δ = epsilon/1000`: keeping `δ` gives a small positive entropy, dropping it gives exactly zero, and those are distinguishable however small `δ` is. Re-injected after widening: 3 tests fail
- [x] 4.11a **P4** Record the defects the suites exposed in code the crate does not own: `Float106::infinity()` and `neg_infinity()` set the low word to `±inf`, where a double-double's low word is a correction to the high word and an infinity has none, so `two_sum` produced a NaN low word beside a correct high word that `is_nan` (a high-word test) did not report. `Div` already carried the guard with its reasoning written out; `Add` and `Mul` did not. Fixed in `deep_causality_num`, which still passes 3839/3839
- [x] 4.12 **P4** Implement the crate against the audited suite — 376 tests green over 1,832 source lines. Three places the suite was right and the first implementation was wrong: a negative ridge penalty is accepted while the diagonal survives (only one that cancels the design is refused, which the vanishing pivot already caught), a logistic label outside `[0, 1]` is `NegativeProbability` because a logistic label *is* a probability, and a singular IRLS Hessian means a rank-deficient design on the first pass (where `β = 0` and every weight is `¼`) but separation on any later one
- [x] 4.12a **P4** Record the two numerical choices the suite forced. Pearson's denominator: `sqrt(sxx)·sqrt(syy)` rounds three times and returns `0.9999999999999998` for `y = 2x + 3`, while `(sxx·syy).sqrt()` rounds once and is exact — but the product overflows near `1e200` and underflows to zero at denormals, so the exact form is used where it is finite and non-zero and the scaled form elsewhere. Equal-frequency binning with ties: neither the block's first rank nor its midpoint satisfies both shipped cases, and largest-share with ties to the lower bin does
- [x] 4.12b **P4** Wrap `StatsError` as `PhysicsError` is wrapped, with a named constructor per variant taking `Into<String>` so a call site passes a `&str`. Pattern matching still reaches the enum, which is what matching needs
- [x] 4.13 **P4** Register the crate: root dependency table at two-digit precision, `AGENTS.md` tier block, unified-math README crate table and ASCII tier block, and the crate's own README. Verified by `scripts/check_tiers.py`, which derives the tiers from the manifests rather than reading them: 30 crates in `AGENTS.md`, 17 in the README. The prose counts moved with it — 29 library crates to 30, sixteen mathematics crates to seventeen
- [x] 4.13a **P4** Update the design artifact `graph.png` is rendered from — `https://claude.ai/code/artifact/7808f976-a88c-42e3-a919-9c85c5795360` — to carry `deep_causality_stats` at tier 4. Recomputed rather than hand-counted: 17 math crates, **54** direct edges and **21** after transitive reduction, where `stats` declares `algebra`, `linear` and `num` but only `stats → linear` survives, the other two already lying on the path through it. The node is placed left of `fft` because the right gutter at x≈662 carries the `multivector → metric` edge through tier 4
- [x] 4.13b Export the updated figure to `deep_causality_unified_math/graph.png`, and update the image's alt text, which still described a tier 4 without statistics. Derived rather than screenshotted by hand: the committed PNG's geometry was recovered off the file itself — a 909 CSS px figure on the ground colour at `deviceScaleFactor` 2, giving 1882 × 1580 — and the plate, the key and the whole stylesheet are lifted verbatim from the artifact, so the figure cannot drift from the page it is derived from. Confirmed by differencing against the old PNG: every pixel outside rows 491–686 and columns 270–916 is identical, and that window is exactly the `stats` node and its one edge to `linear`. `scripts/check_tiers.py` still cannot verify the result, because it is an image
- [x] 4.14 **P5** Run `scripts/mutants.sh` over the crate and resolve every survivor. Four passes: 481 mutants/51 missed, 465/20, 464/8, and 390/2 after the delegation removed 74 mutable sites with `solve_symmetric`. The two that remain are equivalent mutants, provably unkillable: `assert_close`'s `>` and `>=` select the same value at equality, and the pairwise oracle's `skip(i + 1)` and `skip(i)` differ only by diagonal terms that contribute zero. Three timeouts remain in `bin_equal_frequency`, all non-terminating — mutating the block-advance arithmetic stops the loop progressing, so they are killed by hanging rather than by an assertion. The resolutions were of four kinds. **Dead defensive code removed**: `fit_ridge`'s width guard duplicated `accumulate_and_solve`, which already refuses a zero width and checks every row against it, so flipping its `||` to `&&` changed only which of two wordings came back. **A dead initialiser made live**: `binning`'s `best_bin` was unconditionally overwritten on the loop's first pass, so all four mutants on that line were equivalent; reseeding the run from the block's first rank makes the expression the answer outright for a single-value block, and kills them. **A fixture whose stated shape nothing asserted**: deleting either minus sign in `FAMILY` left every property test passing, because properties hold on whatever data they are handed, so `samples_tests.rs` now pins the shape the docstring claims — two samples straddling zero, one constant, every member `n ≥ 2`
- [x] 4.14a **P5** The fourth kind, and the one worth writing down: the pivot search could not be tested at all as the suite was framed. `XᵀX + λI` is positive semi-definite for `λ ≥ 0`, where `|a_ij|² ≤ a_ii·a_jj` forces a zero on the diagonal to carry a zero column with it — so no non-negative penalty can produce a pivot that *must* be moved, and an elimination reaches the same coefficients through any non-zero pivot. Eleven mutants lived there for that reason, and the existing test's docstring claimed they "produce a wrong coefficient", which mutation testing disproved. The crate accepts a **negative** penalty, which is not bound by semi-definiteness: it subtracts from the diagonal while the off-diagonal stands. Three fixtures were built on that — a swap at column 0, one at column 1, and one whose only pivot is two rows below the diagonal at four columns — each against an exact `Fraction` solution, and each verified in `notes/c2-oracles/ridge_pivoting.py` by rerunning the search under every mis-addressed index *before* the Rust was written. One fixture also had to be rescaled: `−7/40` is not dyadic, so the `f64` literal `-0.175` is itself wrong by ~1e-18, which `Float106` sees; scaling the response by 40 makes the answer exact integers
- [x] 4.14b **P4** Delegate both solve sites to `deep_causality_linear`, which 4.15's own gate forced. `cargo machete` reported `deep_causality_linear` unused — the only such crate in the workspace — because the ridge solve was a local `solve_symmetric` rather than a call into `linear`. That is a defect against two records at once: `statistics-crate-identity` carries a SHALL putting the crate over `linear` and warns in the same breath that "declaring an unused dependency would also fail the repository's unused-dependency check", and `c2-site-inventory.md` calls the ridge solve "a substitution, not a re-export". Dropping the dependency instead would have contradicted the SHALL and moved the crate to tier 2, churning the graph, both READMEs, `AGENTS.md` and the artifact a second time. The ridge normal equations and the logistic Newton step now both call `deep_causality_linear::solve`, deleting 58 lines; `RealField + FromPrimitive` already satisfies `NormedScalar` so no bound moved, and Bazel picks the edge up through `all_crate_deps` without a BUILD change. All 416 tests pass unchanged across the substitution, including the three pivot fixtures and every error path, which is the evidence that the local elimination and the delegated LU agree. The three fixtures are kept: a design whose pivot is neither on the diagonal nor in the next row is worth holding against any factorisation ridge delegates to. Group 6's inventory (6.6) should not re-litigate this site
- [x] 4.16 Measure line coverage with `cargo llvm-cov` and close what is reachable. Nine of the thirteen files are at 100%, and the standing bar is 95% per file. One real gap was found and closed: `assert_no_finite_answer`'s `Err(NonFiniteInput)` arm had never run, because its only callers are the moments, which carry no non-finite guard and return a non-finite `Ok` instead — the helper accepts both outcomes and only one was ever exercised. Feeding it the error directly takes `assertions.rs` to 100%.

  Everything still uncovered is one guard repeated four times: `T::from_usize(n)` refused, in the observation count, the pair count, the degrees-of-freedom count and the bin index. **No shipped scalar can reach it.** `f32`, `f64`, `Float106` and `BFloat16` all round or widen, and every one of their `from_usize` implementations returns `Some` for every input, so the suite cannot reach these arms at any precision. They are not dead code — the bound is `T: FromPrimitive` rather than a closed set of four types, and the `Option` exists so a caller's own scalar may decline — but they are unreachable by construction from inside this crate.

  A test scalar that refuses was built and then reverted: reaching four lines cost 356 lines of trait delegation, because `RealField` is blanket-implemented only over `Float` and `Float` alone carries 61 required methods. That made the scaffolding the least-covered file in the crate and added about a hundred mutable sites, to cover four lines that no shipped type reaches. Recorded here so it is not attempted a second time.

  `moments.rs` (93.33%) and `correlation.rs` (94.87%) therefore sit under the bar, both solely on this guard, and both because the files are small enough for two lines to move the percentage — `binning.rs` and `ridge.rs` carry the identical guard and clear the bar only by being larger. Reformatting the guard onto one line would lift the number without changing what is tested, so it was not done
- [x] 4.15 Verify: the crate's test suite is green at full coverage under both build systems, and the unused-dependency check passes — this gates every task in group 5. Cargo 416 tests green; Bazel `//...` 1307 tests green over 16 stats targets, `samples_tests` picked up by the existing `tests/utils_tests/*_tests.rs` glob without a BUILD change; clippy clean workspace-wide; `cargo machete` reports no unused dependency in any crate directory. 

## 5. C2 — statistics consumer migration

Verified against the code on 2026-09-07, after the stats crate shipped. The scope below differs from the
first draft in two structural ways, both traced to `design.md` **D7** (`tensor` does not depend on the
statistics crate, maintainer-decided *no*): the earlier list named two `tensor` sites as migration
targets, and D7 keeps **all five** of `ext_stats.rs`'s statistics, not the one 5.8a mentioned.

Every consumer needs the dependency edge before its first call site compiles; that is ordinary
migration work, listed once as 5.1a rather than repeated per task. The workspace entry already exists
at `Cargo.toml:66` and is `default-features = false`, so each member states its own features.

- [ ] 5.1 Confirm 4.15 is checked; no task in this group starts before it is
- [ ] 5.1a Add the `deep_causality_stats` workspace dependency to each consuming Cargo member as its first call lands, with explicit features. Existing rules-rs targets lift Cargo dependencies through `all_crate_deps(...)`; use that mechanism and refresh dependency resolution as required, without duplicate explicit Bazel labels. Derive the consuming package list from the inventory. D7 excludes tensor.

### SURD

- [ ] 5.2 Delegate both SURD entropy paths to stats in bits; retain tensor marginalization and conditional-entropy subtraction in algorithms. Plain inputs use SkipZero/None. Optional inputs filter None and preserve all-absent zero, strict mass < epsilon, and normalized probability > epsilon. Stats BySum uses <= floor, so test and adapt the equality boundary rather than passing epsilon blindly. Add FromPrimitive to the four helper bounds.
- [ ] 5.2a Map StatsError locally into CausalTensorError in algorithms. A downstream From implementation for these two foreign types is not legal; do not add a tensor dependency to host one. Map BRCD and physics errors according to their own error types.
- [ ] 5.3 Verify SURD against existing tolerances and independently derived reference fixtures. Record per-term log2 versus final ln(2) division deltas; do not demand bit identity or regenerate expected values from the migrated implementation.
- [ ] 5.3a Preserve the optional helper's all-None zero and valid low-mass policies. Distinguish those from existing public-driver empty-input rejection. Test mapped errors for negative/non-finite marginals and record stricter validation separately from rounding drift.

### BRCD

- [ ] 5.4 Migrate BRCD's log-sum-exp at its **three** sites: `brcd_algo.rs:540` `logsumexp_slice`, `brcd_boss_bootstrap.rs:325` `logsumexp`, and the two-term `brcd_gaussian.rs:631` `logaddexp` (caller `:372`). All three agree with the shipped total form on the empty slice and on a non-finite maximum, so this is the stage's cheapest absorption. The earlier draft counted **four**, including `ext_stats.rs:154` — that one is `tensor`'s, not BRCD's, and D7 keeps it
- [ ] 5.5 Migrate BRCD's Gaussian log-density: `brcd_gaussian.rs:561` `logpdf_rows` and `:579` `single_logpdf`. **Neither is in `notes/c2-site-inventory.md`**, whose only Gaussian entry is `ext_stats.rs:58` — `tensor`'s, which D7 keeps. Both BRCD sites take `sigma2`, a variance, so the shipped parameterisation matches with no conversion. Add both to the inventory
- [ ] 5.5a Preserve BRCD's 1e-12 density variance floor in the consumer wrapper before delegation. Test the floor and mapped errors, including non-finite inputs, without silently replacing the model policy with stats' nonpositive-scale rejection.
- [ ] 5.6 Migrate BRCD's materialised ridge (`brcd_gaussian.rs:86`) onto `fit_ridge`. Three things do not carry over untouched: BRCD floors `sigma2` to `1e-12` (`:130`, `:527`) where the crate applies no floor; the crate refuses non-finite rows where BRCD relies on the caller pre-filtering (`:238-243`); and BRCD's `RidgeFit` has an inherent `predict()` (`:70-74`, used at `:249`) where the shipped `RidgeFit` is data only. Keep `predict` as a local helper over the shipped struct
- [ ] 5.6a Preserve streaming ridge's memory behavior. The shipped stats iterator API collects all rows; a row adapter alone is insufficient. Specify and test a shared path that does not retain the full design before migrating the shared-column/index consumer.
- [ ] 5.7 Migrate sigmoid independently. Before delegating BRCD logistic, specify and test a minimal shared API that preserves its unpenalized intercept, boolean-label conversion, stopping rule and iteration-cap result. Preserve the existing stats entry's contract. Do not substitute the current all-coefficients-penalized objective.
- [ ] 5.8 Migrate BRCD's mean and corrected variance (`brcd_gaussian.rs:661` `mean`, `:670` `variance_ddof1`)
- [ ] 5.8a Preserve BRCD's empty mean of zero and variance of one for fewer than two observations in explicit wrappers. Enumerate every affected call site and retain the existing f_in_parents_single_row_regime_uses_unit_variance regression expectation. Test normal delegation and each fallback.
- [ ] 5.8b Record which solver runs after the ridge migration. Both `brcd_linalg`'s `solve_linear` and `deep_causality_linear::solve` are **LU with partial pivoting**, so the result should not move materially at `f64`; confirm it. Two corrections to the earlier draft: the LU-versus-Cholesky decision is recorded in the **statistics-consumer-migration** spec, not the linear-adoption spec; and `brcd/mod.rs:19` calls `brcd_linalg` "the small dense SPD solver", which `brcd_linalg.rs:11-17` contradicts — fix that prose while here

### mRMR, discovery, physics

- [ ] 5.9 Migrate mRMR's Pearson (`mrmr_utils.rs:29` `pearson_correlation`) onto **`pearson_pairwise_complete`**, not `pearson`: mRMR's policy is pairwise deletion and its `n` counts surviving pairs. The zero-variance contract already matches — both return `Ok((0, n))` rather than an error
- [ ] 5.9a Add RealField + FromPrimitive bounds through the affected mRMR helpers and callers. Float provides arithmetic but does not imply RealField's algebraic law bounds. Compute Pearson in the working scalar and preserve the existing public f64 result boundary; verify wide precision before that conversion.
- [ ] 5.9a.1 **P1–P4** Add a Real-based presence adapter in algebra for scalar and Option inputs, with a declared API and failing tests before implementation. Preserve NaN/Some(NaN) as missing, None as missing, and finite/infinite present values. FloatOption itself requires Float, so replace that consumer bound too; retain num's existing adapter for compatibility and do not add num → algebra.
- [ ] 5.9a.2 Replace the three Float-bound signatures in algorithms' mrmr_algo.rs and mrmr_utils.rs. Propagate the change through discovery's feature_selector/mrmr.rs and both CDL implementations in cdl/surd_cleaned.rs. Use existing Precision where sufficient; select_indices needs the appropriate explicit algebraic bounds. Correct affected bound documentation.
- [ ] 5.9a.3 Compile-check mRMR through a generic RealField + FromPrimitive caller with no Float bound, plus the necessary container bounds. Run existing plain/optional input and discovery integration tests. Sweep algorithms runtime sources and the affected discovery call chain for remaining Float imports/bounds; distinguish parquet Field::Float from trait usage.
- [ ] 5.9a.4 After replacing FloatOption consumers, search the entire workspace, including examples, tests and documentation, for remaining uses. Distinguish actual consumers from its definition, implementations, re-export and dedicated tests. If no consumers remain, flag FloatOption and its dedicated support code for removal, identifying the files and public API impact; do not retain it solely for speculative compatibility. Report the removal candidate rather than deleting it in this task.
- [ ] 5.9b Record mRMR's two changes: the crate refuses non-finite input where mRMR lets infinities through, and the two implementations use different formulas — mRMR the uncentred raw-sums form, the crate the centred two-pass form — so results move in the last places
- [ ] 5.10 Delegate discovery equal-width and equal-frequency binning. Explicitly adopt stats' tied-block largest-share/lower-bin tie policy and rank boundaries. Test non-divisible counts, ties, bin edges and near-constant ranges; equal-width value identity is not guaranteed because arithmetic ordering and constant-range thresholds differ.
- [ ] 5.10a Test discovery's error mapping for empty input, bins < 2, bins > observations, NaN and infinity. Existing helpers already reject bins < 2 and NaN; empty, excess bins and infinities are newly refused. Preserve useful preprocessing error context.
- [ ] 5.11 Migrate the physics entropy kernel (`thermodynamics/stats.rs:154` `shannon_entropy_kernel`) to bits, and widen its bound: it is `RealField + MaybeParallel + Sum`, and `entropy` needs `FromPrimitive`, which `RealField` does **not** imply. Add `From<StatsError> for PhysicsError` — `physics_error.rs` carries `From` only for `CausalTensorError` and `MetricError`. Its zero policy (`p > 0`) and absence of normalisation map exactly onto `EntropyConfig::bits()`
- [ ] 5.11a Rename both public items together, not just the kernel: `shannon_entropy_kernel` and its causal wrapper `wrappers.rs:83` `shannon_entropy` are equally base-ambiguous, and both resolve flat at the crate root through `pub use`. Record as breaking. Exactly **one** test pins a nats value — `test_shannon_entropy_kernel_uniform`, asserting `ln 4 ≈ 1.386` — so the earlier "tests which pin the nats result" is one test, not several. Also record the second behaviour change: the crate refuses a non-finite entry, where the kernel silently drops it

### Kept, with the reason recorded at the site

- [ ] 5.12 **D7 keeps all five `ext_stats.rs` statistics**, not only `conditional_variance`: `:26` `sample_mean`, `:38` `sample_covariance`, `:58` `gaussian_log_density`, `:81` `conditional_variance`, `:154` `logsumexp`. The binding reason is D7 — `tensor` does not depend on the statistics crate, because delegating would move `tensor` to tier 5, `multivector` to 6 and `topology` to 7, invalidating the tier tables and the dependency figure for two functions. Record that reason at the module, once, covering all five
- [ ] 5.13 Keep quantum's `qpu/bridge.rs:76` `shots_to_observable`: a frequency-weighted mean and `n−1` variance over `(outcome, count)` pairs, which is a different function from a slice mean. Two corrections: the counts are `u64` (the *outcomes* are `usize`), and `Uncertain::normal` is generic over `T: UncertainReal`, instantiated here at `f64` — so "f64 at both ends" is true but is not itself the reason. The reason is the weighted form, plus the crate refusing the degenerate case the bridge must return a number for
- [ ] 5.13a Resolve uncertain from_samples as a plain-slice consumer, separately from the weighted shot bridge. Specify empty, singleton and non-finite policies plus dependency/tier effects before migration; the bridge's weighted-form exclusion does not apply to this helper.

### Examples

- [ ] 5.14 Migrate the example sites. The earlier draft said "eight"; the inventory table has eight rows but **nine** file:line locations, and a sweep found more. Migratable: `causal_correction_examples/src/math_utils.rs:15` and `causal_counterfactual_examples/src/math_utils.rs:15` (**byte-identical, md5 `d1acd926…`, but in two separate crates — two edits, not one**); `corrective_ddos_detector/model.rs:97`, `:99` and `:100`, which are a mean, a variance and a standard deviation that collapse to one `std_dev` plus one `mean`; `plasma_blackout/weather/model.rs:178` `mean_sd`; `ml_rca/model.rs:92` `mean_score`; `classical_via_causal_monad/granger/main.rs:116`; `granger/model.rs:85` **and `:89`**, which holds two inline means, not one
- [ ] 5.14a Add the five sites the inventory's example scan missed: the CATE mean in `classical_via_causal_monad/cate/main.rs:55` and its twin in the sibling example; `ml_rca/utils.rs:75` `fit_standardizer`, which is the **population** form and so is the first counterexample to the inventory's "every variance in the workspace is the corrected `n−1` form"; and `chronometric_examples/gm_recovery/pipeline.rs:262` and `:268`, a generic-scalar mean and standard deviation
- [ ] 5.14b **Do not** migrate two of the listed sites. `ml_rca/model.rs:102` `sigmoid` is elementwise over a `candle_core::Tensor` in candle ops, so it stays inside candle's autodiff graph — a different function on a different type. `clinical_trial/model.rs:194` `average_arm` folds `Uncertain<f64>`, which is neither `Copy` nor `RealField`, and builds a lazy computation graph rather than reducing a slice. Record both reasons at the site
- [ ] 5.14c Record the example behaviour changes: both `math_utils.rs` files document and rely on `NaN` for the empty slice and the crate returns `Result`, so every caller becomes fallible or grows an explicit empty case; `mean_sd` returns `sd = 0` for a one-sample slice where the crate errors
- [ ] 5.14d Derive the affected example Cargo packages from the reconciled inventory. Use their existing rules-rs dependency macros. Respect per-example FloatType aliases and existing f64 shared-helper interfaces; reuse num lift utilities without inventing aliases or conversion wrappers.

### Close

- [ ] 5.15 Reconcile `notes/c2-site-inventory.md` with what the migration found. Its arithmetic does not close: the header says 31 sites (23 library + 8 examples) while the section headers sum to 24, "Descriptive — seven sites" lists eight, and the examples table's eight rows carry nine locations. Add the two BRCD density sites and the five example sites found here, and restate the total once it is derivable
- [ ] 5.16 Verify every inventory site is resolved — migrated, or kept with its reason at the site. Six sites are named by no task above and need a decision each: `ext_stats.rs:26` and `:38` (covered by 5.12), `uncertain_statistics.rs:33` and `:86` `standard_deviation`, `uncertain_f64.rs:19` (5.13a), and `missing_value_imputer.rs:28` `impute_mean`
- [ ] 5.17 Verify a migrated Pearson computation before its public f64 conversion at Float106 using an independent exact or high-precision oracle. Show the precision retained internally and test f64 against justified tolerances; do not claim the public result type became generic.
- [ ] 5.18 Verify: `bazel test //...` green, `cargo machete` clean for every crate that gained the dependency, and no consumer test edited to make a failure disappear. Every changed result recorded with its old and new value

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
