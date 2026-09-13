<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Retrofit the sampling layer: entropy in `rand`, distributions in `stats`

## Why

Unified math is an implemented pattern, not a proposal: sixteen of the seventeen crates under
`deep_causality_unified_math/` take their scalar as a parameter and ask for it through the algebra
tower. `deep_causality_rand` is the holdout. It is one of the oldest crates in the repository,
added shortly after the core and years before the pattern existed.

The first reading of that gap was that `rand` needs generalising. Measurement says the gap is a
misplaced seam between two crates.

**`rand` holds two different things.** Entropy — bits from the machine, `Xoshiro256`, `next_u64`,
Sobol point sets — is genuinely low-level and belongs at tier 2. Distributions — `Normal`,
`Uniform`, `Bernoulli`, Box–Muller, the inverse CDFs — are mathematics over a real field, and they
sit two tiers below the crate that owns the analytic surface they need.

**`deep_causality_stats` is the crate that should hold them, and it is already conformant.** Its
functions are bounded `T: RealField + FromPrimitive`, exactly what `deep_causality_fft` asks. It
depends on `num`, `algebra` and `linear`, and **not** on `rand`. It already carries
`gaussian_log_density`: the Gaussian is modelled there, as a density. So today `stats` knows what a
Gaussian *is* and cannot draw from one, while `rand` can draw from one and cannot say what it is.

That split is why the retrofit stalls. A blanket
`impl<T: RealField> Distribution<T> for StandardUniform` inside `rand` is
`error[E0119]` against the existing `u64`, `u32` and `bool` implementations, because coherence
cannot prove those will never be real fields. The conflict exists only because one crate samples
both machine words and real numbers. At the crate boundary it does not arise: `stats` has no reason
to sample a machine word.

**The consumers already split along that seam.** Counted, not assumed:

| Crate | distribution uses | entropy uses | needs after the split |
|---|---|---|---|
| `physics` | 6 | 0 | `stats` only |
| `discovery` | 1 | 0 | `stats` only |
| `algorithms` | 0 | 14 | `rand` only |
| `tensor` | 0 | 2 | `rand` only |
| `ultragraph` | 0 | 1 | `rand` only, and only in benchmarks |
| `data_structures` | 0 | 5 | `rand` only, and only in benchmarks |
| `topology` | 4 | 7 | both |
| `uncertain` | 5 | 6 | both, until this change |

`ultragraph` and `data_structures` name `rand` in `[dev-dependencies]` and call one function,
`random_range`, in benchmarks. Under any design that keeps distributions in `rand`, a graph library
would reach a sampling crate to pick a random index. Under this one it reaches an entropy crate,
which is what it wants. Only `topology` gains a dependency it does not already have; `physics`,
`discovery` and `uncertain` all depend on `stats` today.

**Two crates reach `stats` alone.** `deep_causality_topology` was counted next: thirteen `rand`
references, of which six are generator bounds, five are distributions, and one is
`(rng.next_u64() as usize) % num_edges` picking a lattice edge. The last is what the discrete
uniform is for, so after this change topology's manifest names `stats` and not `rand` — and its
gauge field, whose `RandomField` trait is a hand-rolled `f64` uniform today, gains precision as a
parameter along the way.

**The umbrella.** `deep_causality_uncertain` is the crate this is for. It imports `Bernoulli`,
`Normal`, `Uniform`, the three distribution error types and all four inverse-CDF functions from
`rand`, alongside `Xoshiro256` for entropy. Once the distributions move, its distribution layer
names `stats` alone, and `rand` remains only where it draws raw words. That is the shape
`hkt_uncertain.md` stage 2 needs, and `uncertain` is next in line for its own renovation. This
change is its precondition.

## What Changes

**BREAKING, and accepted.** The public API of two crates changes and types move between them.
Usage is internal to this repository. A compatible retrofit would preserve the seam that is wrong.

### The crate boundary

- **BREAKING — distributions move from `rand` to `stats`**: `Normal<T>`, `Uniform<T>`,
  `Bernoulli`, `StandardUniform`, `StandardNormal`, `Open01`, `OpenClosed01`, the `Distribution`
  trait, the four inverse-CDF functions, and the three distribution error types. They land beside
  `gaussian_log_density`, `entropy` and `log_sum_exp`, which are the mathematics they belong with.
- **`rand` keeps entropy**: `RngCore`, `Rng`, `Xoshiro256`, `OsRandomRng`, `SobolSequence`,
  `rng()`, `Fill`, and the range machinery. Sobol stays because a Sobol point is a deterministic
  function of an index and a digital shift drawn from `Xoshiro256` — a source of numbers, not a
  statement about a distribution.
- **`stats` gains a dependency on `rand`.** Downhill, tier 4 to tier 2, so no cycle.

### Precision as a parameter

- **BREAKING — the three-way split of what a sampler samples.** `StandardUniform` moves to `stats`
  and becomes numerical only; `StandardWord` and `StandardBool` stay in `rand` for machine words
  and Booleans. `Rng::random` splits into `random_word` and `random_boolean` there.
- **One generic body per distribution.** Box–Muller over `Real`'s `ln`, `sqrt`, `cos` and `pi` in
  the caller's scalar, replacing an `f64` ziggurat that `f32` narrowed from and `Float106` could
  not use. Three per-type files delete.
- **A `RandWidth` capability trait** carries the one per-type fact: how many 53-bit words a
  significand absorbs. Everything else is algebra.
- **`BFloat16` becomes samplable**, closing `hkt_uncertain.md` B8.
- **BREAKING — hardcoded `f64` leaves both public APIs**: `Bernoulli::new`/`p`,
  `Rng::random_bool`, `SobolSequence::coordinate`/`point`.
- **`RealRng` is removed** — zero consumers, and its documented extension path is forbidden by the
  orphan rule.

### Seven distributions `stats` gains

Trivial only because the foundation lands. Each was implemented and checked against its closed
form before proposing: **Exponential** (0.5002 against 0.5), **LogNormal** (1.1330 against 1.1331),
**Cauchy** (median −0.0149 against 0), **Weibull** (0.8861 against 0.8862), **Categorical**
(0.100/0.300/0.600 against 0.1/0.3/0.6), **Poisson** (3.0004 against 3.0). Exponential at
`Float106` gave 0.4995 with no extra code.

A seventh, **discrete uniform over a range**, is added for a structural reason rather than a
statistical one: Categorical covers weighted choice and nothing covers unweighted choice over `n`
items, so every caller writes `(rng.next_u64() as usize) % n` by hand — a form that is biased
whenever `n` does not divide `2^64`.

Two carry real hazards that the spec pins rather than assumes: Cauchy has **no mean**, so a
moment-based test on it is meaningless, and Poisson's Knuth algorithm degrades at large `λ`.

### The HKT extension

- An ensemble is an existing witnessed container, not a new type.
- A lazy sampler carrier gets no witness, and the reason is recorded.
- A **diagonal** traversal, bounded on `Semigroupal` rather than `Applicative`, so correlated
  draws pair index with index. The cartesian `sequence` returns 50⁴ where sampling wants 50.

## Capabilities

### New Capabilities
- `entropy-source`: what `deep_causality_rand` is after the split — machine words, Booleans,
  generators and low-discrepancy point sets, with no claim about real-valued distributions.
- `stats-sampling`: distributions and sampling in `deep_causality_stats`, the scalar bound they
  ask for, and the contract each distribution satisfies including the ones with no mean.
- `sampling-hkt-composition`: which categorical structures an ensemble carries, and why the
  monoidal route is the meaningful reading for correlated draws.

### Modified Capabilities
<!-- None. No existing spec's requirements change. `unified-math-tdd-protocol` governs how this is
     built; its own requirements are unchanged. -->

## Impact

**Code.** `rand` loses the distribution half; `stats` gains it plus six distributions. Measured on
a working prototype of the split-within-one-crate: **129 lines net removed**, per-type float
`Distribution` impls **10 → 2**, three files deleted.

**Consumers.** Twelve crates depend on `rand`; `quantum` reaches it transitively through
`uncertain`'s `qpu` feature. Two kinds of edit:

- *Import moves* — a crate drawing from a named distribution changes `use deep_causality_rand::` to
  `use deep_causality_stats::`. `physics`, `discovery`, `uncertain` and `topology`.
- *The word/Boolean split* — measured workspace-wide at **two sites**, both
  `uncertain/src/types/sampler/sampler_seed.rs`, `random::<u64>()` becoming `random_word()`.

`algorithms`, `tensor`, `ultragraph` and `data_structures` need no edit: they draw words.

**Verified on the prototype, all reverted.** `f32`, `f64` and `Float106` give mean 0.4998/0.4995
against 0.5, `E[x²]` 0.3332 against 1/3, normal variance 0.998 against 1. `BFloat16` samples for
the first time. The tier-7 leak in `topology`'s `metropolis.rs` compiles with its two clauses
removed. All six new distributions match their closed forms.

**Also measured: nothing else is misplaced.** `algorithms::entropy_nvars`,
`topology::covariance_matrix` and `discovery::impute_mean` all already delegate to `stats`.
Physics "moments" are momentum and CFD "covariance" is a Kalman state covariance — physical
quantities, not statistical estimators. `stats` is already the single home for statistics; only the
sampling half was missing.

**Risk.** The `f64` ziggurat is faster than Box–Muller, and this trades it away. Stated in
`design.md` with the escape route. `RandWidth::WORDS` is a manual list: a wrong value silently
degrades a wide scalar, measured at **0 of 200** low limbs against **400 of 400** correct, and the
spec makes that a requirement rather than trusting the constant.

**Baseline.** `rand` 154 tests, `stats` and `uncertain` suites to be counted at phase 1.
