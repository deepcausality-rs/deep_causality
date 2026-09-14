<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Design: retrofitting the sampling layer

Every claim below was measured on `main` at `89bf19df6` by building a working prototype and
reading the compiler, then reverting. Error codes are quoted where a decision rests on one.

## Context

Unified math is implemented, not proposed. Sixteen of seventeen crates take their scalar as a
parameter and ask for it through the algebra tower. `deep_causality_rand` predates the pattern by
years and was never brought across. This change is the retrofit.

`deep_causality_fft` is the reference implementation of the target:

```rust
pub trait FftScalar: RealField + FromPrimitive + MaybeParallel {}
impl<T: RealField + FromPrimitive + MaybeParallel> FftScalar for T {}
```

Zero per-type files. Two `f64` mentions in the crate, both `R::from_f64(0.5)`. Twiddle factors
compute `R::pi()` and `theta.cos()` — arithmetic in `R`, no bit manipulation. `rand` has six
per-type files doing what one generic body can do.

## Decision 0: the seam is between two crates, not inside one

The first reading of this problem was that `rand` needs generalising. That reading produced a
proposal that kept distributions in `rand` and worked around `error[E0119]` with three
distribution types in one crate. It was wrong, and the measurement that shows why is the consumer
table in the proposal: the crates split cleanly into those wanting a random index and those wanting
a Gaussian, and only one crate gains a dependency it does not already have.

`deep_causality_stats` is already the conformant crate. Bounded `T: RealField + FromPrimitive`,
depending on `num`, `algebra` and `linear` and **not** on `rand`, carrying `gaussian_log_density`.
It models the Gaussian as a density and cannot draw from it; `rand` draws from it and cannot say
what it is.

Moving the distributions makes `error[E0119]` disappear rather than be worked around, because
`stats` has no reason to sample a `u64`. The three-way split survives — `StandardWord` and
`StandardBool` are still separate from `StandardUniform` — but the important separation is the
crate boundary, and the type split is what keeps the conflict from reappearing inside `rand`.

**Where Sobol goes, and why.** A Sobol point is a deterministic function of an index and a digital
shift drawn from `Xoshiro256`. It is a source of numbers in `[0, 1)`, not a statement about a
distribution, so it stays in `rand`. The inverse-CDF functions go the other way: they are the
mathematics of named distributions and move to `stats`, which is why `uncertain`'s QMC sampler ends
up importing from both — Sobol for the point set, the inverse CDFs for the transforms.

## Decision 1: split `StandardUniform` by what it samples

The direct move fails:

```
error[E0119]: conflicting implementations of trait `Distribution<u64>` for type `StandardUniform`
error[E0119]: conflicting implementations of trait `Distribution<u32>` for type `StandardUniform`
error[E0119]: conflicting implementations of trait `Distribution<bool>` for type `StandardUniform`
```

A blanket over `RealField` collides with the existing integer and Boolean impls, because coherence
cannot prove `u64` will never be a real field. **This is why the crate was never retrofitted the
way `fft` was.** `FftScalar` blankets cleanly because nothing else implements it; `Distribution<T>`
is shared between floats and non-floats.

The abstraction is the problem, not the compiler. One type currently claims three different
mathematical objects:

| What is sampled | Type after the split | Algebra |
|---|---|---|
| a uniform on `[0, 1)` | `StandardUniform` | a real field |
| a machine word | `StandardWord` | none — the generator's own output |
| a coin | `StandardBool` | the two-element Boolean algebra |

Separated, the blanket compiles with no E0119. Verified on the prototype.

`Rng::random` splits to match: `random` numerical, `random_word` machine words,
`random_boolean` the coin. A call site now says which it means.

**Incidental finding.** The current `bool` impl is `next_u64() % 2 == 0` — a parity test on a
64-bit draw. `StandardBool` takes the top bit instead. Measured true fraction over 10 000 draws:
0.5013.

## Decision 2: one capability trait, `RandWidth`, carrying one constant

Following `FftScalar`. The only genuinely per-type fact a numerical draw needs is how many 53-bit
words the significand absorbs:

```rust
pub trait RandWidth {
    const WORDS: u32 = 1;
}
impl RandWidth for Float106 { const WORDS: u32 = 2; }
```

Everything else is algebra. The default covers every scalar of 53 bits or fewer, so `f32`, `f64`
and `BFloat16` are empty impls.

**Why the constant cannot be dropped.** A single 53-bit draw returned as a `Float106` is an `f64`
wearing a wider type. It passes every bounds check and every moment test. Measured: the naive
single-draw generic carried a non-zero low limb in **0 of 200** draws; consuming the declared
number of words carried one in **400 of 400**. The spec pins this as a requirement rather than
trusting the constant to be right.

**Open, and flagged for review.** `WORDS` is a manual list of four impls rather than a fact derived
from `Float`. Deriving it would need a significand-width query in `deep_causality_num`, which is a
change to a tier-0 crate and outside this scope. The manual list is four lines and a new scalar
adds a fifth; the alternative is a wider change with a wider blast radius.

## Decision 3: both distributions become one generic body each

`StandardUniform` samples words and scales in `T`. `StandardNormal` becomes Box–Muller over
`Real`'s `ln`, `sqrt`, `cos` and `pi`, computed in the caller's scalar.

That replaces an `f64` ziggurat which `f32` narrowed from and `Float106` could not use at all —
`Float106` needed its own hand-written Box–Muller precisely because the shared path was `f64`.
One body now serves all four.

Three files delete: `dist_float_32.rs`, `dist_float_64.rs`, `dist_float_common.rs`. Measured on
the prototype: **129 lines net removed**, per-type float `Distribution` impls **10 → 2**.

**Cost, stated honestly.** The ziggurat is faster than Box–Muller for `f64` — it avoids two
transcendentals per draw. Trading it for one generic body is a real performance decision, not a
free simplification. It is the right trade here because the ziggurat cannot serve a scalar wider
than the table it is built from, and precision as a parameter is the point of the change. If the
`f64` hot path later proves to matter, a specialised impl can return behind the same generic
surface; that would be a measured optimisation rather than the starting design.

## Decision 4: the remaining hardcoded `f64` leaves the public API

`Bernoulli::new(p: f64)`, `Bernoulli::p()`, `Rng::random_bool(p: f64)`,
`SobolSequence::coordinate`, `SobolSequence::point`. A program written against a `FloatType` alias
meets a raw `f64` at each.

Two documented exceptions where the representation is genuinely fixed:

- **`Bernoulli`** stores its parameter as 64-bit fixed point, so no scalar wider than `f64`
  changes the draw. It accepts the caller's scalar and states the limit.
- **`SobolSequence`** resolves 32 bits per coordinate by construction
  (`INV_TWO_POW_32`), a limit `Float106` cannot lift. Recorded in `hkt_uncertain.md` as B6.

## Decision 5: `RealRng` is removed, not kept

It was an earlier attempt at this same retrofit. Zero consumers in the workspace outside its own
test file. Its docstring promises a new float gains the bound "automatically ... and downstream
code is untouched"; measured, the seam it names is `pub(crate)` so no other crate can implement it,
and made public the orphan rule still rejects it with `error[E0117]`. Keeping both traits would
leave two answers to one question, with the unreachable one documented as the reachable one.

## The HKT extension

Three findings, each from a probe, and two of them contradict what analogy would suggest.

**No new container.** An ensemble of `n` draws is a `Vec<T>` with a witness, and `CausalTensor`
already is one, carrying `Functor`, `Foldable`, `Pure`, `Applicative`, `Monad` and `Traversable`.
Sampling into it is an ordinary monomorphised function needing no witness of `rand`'s own. An
earlier draft of this design proposed a `Particles<T>` carrier; it was tested, found to duplicate
`CausalTensor` for no structural gain, and dropped.

**No witness on a lazy sampler.** A carrier storing a sampling closure cannot take the container
traits: `error[E0276]: impl has stricter requirements than trait` when the impl tries to add the
`'static` its `Box<dyn Fn>` needs. This is not a gap in `haft`. Measured: `Functor`, `Pure`,
`Applicative`, `Monad`, `Traversable` and `LaxMonoidal` carry **zero** `'static` bounds, while
`Profunctor` — which stores its functions — carries them on every parameter. The container traits
are for data. A lazy sampler is a program, and `haft`'s home for programs is `Arrow`.

**The traversal must be diagonal, and cannot be today.** Turning a field of ensembles inside out
through `Traversable::sequence` uses the cartesian applicative: a 2×2 field of 50 draws per cell
came back with **6 250 000** entries, 50⁴. For sampling that is wrong — correlation is by index,
draw *i* with draw *i*.

The diagonal already exists as `ZipTensorWitness`, carrying `Semigroupal + Convolutional +
MonoidalApplicative`. But it deliberately has no `Pure`, because the unit of a positional zip is
the infinite repeat and a finite container cannot represent it — `haft`'s `LaxMonoidal`
documentation prescribes exactly this. So it cannot drive `sequence`:

```
error[E0277]: the trait bound `ZipTensorWitness: Applicative<ZipTensorWitness>` is not satisfied
```

A traversal bounded on `Semigroupal` instead closes it. Prototyped and verified: 50 fields not
6 250 000, each `[2, 2]`, each holding draw *i* of every cell — asserted on exact values, not
counts. Ragged columns truncate the ensemble to the shortest while keeping every field cell; the
empty structure returns the caller's seed.

The seed parameter is where the absence of `Pure` surfaces. Without a unit there is nothing to
build the accumulator from, so the caller declares it. For sampling that is correct: the ensemble
size is a decision, not an inference.

**This gap is not `rand`'s.** `ZipTensorWitness` and `ZipDenseVectorWitness` are equally stranded
today, independent of sampling. The traversal belongs in `haft` beside `Traversable`, and sampling
is what exposed it.

## What this change does not do

- It does not renovate `uncertain`. It moves that crate's distribution imports to `stats` and
  fixes two word-draw call sites, and stops there. `SampledValue`, the global sample cache and
  the lazy graph belong to `hkt_uncertain.md` stage 2, which this change is the precondition for.
- It does not add sampling algorithms beyond the six distributions. Importance sampling, SMC and
  MCMC diagnostics become writable once the samplers sit beside the densities, and that is the
  point of the move, but each is its own change with its own verification.
- It does not lift the Sobol 32-bit resolution cap (B6), which is independent of float layout.
- It does not add `Distribution` implementations for new distributions. The surface is the one
  that exists, retyped.

## Risks

| Risk | Assessment |
|---|---|
| Public API breaks | Accepted and stated in the proposal. Measured break surface across twelve consumer crates: **two call sites**, both `random::<u64>()` in `uncertain`. |
| Box–Muller is slower than the ziggurat at `f64` | Real. Decision 3 states the trade and the escape route if it later matters. |
| `RandWidth::WORDS` is a manual list | A wrong value silently degrades a wide scalar to a narrow one. The spec requires the low-limb test, which catches exactly that. |
| The diagonal traversal is new API in `haft` | It is additive; nothing existing changes. Two stranded witnesses gain a use. |
| `quantum` is affected transitively | It reaches `rand` only through `uncertain`'s `qpu` feature and uses no word or Boolean draw. Verified: it declares no `rand` dependency of its own. |

## Decision 7: the six new distributions, and the two that need special tests

Each is one to three lines over `Real` once the foundation lands, and each was implemented and
measured against its closed form before being proposed rather than after:

| Distribution | Method | Measured | Exact |
|---|---|---|---|
| Exponential(2) | `-ln(u)/λ` | 0.5002 | 0.5 |
| LogNormal(0, 0.5) | `exp(μ + σZ)` | 1.1330 | 1.1331 |
| Cauchy | `tan(π(u − ½))` | median −0.0149 | 0 |
| Weibull(2, 1) | `λ(−ln u)^(1/k)` | 0.8861 | 0.8862 |
| Categorical 1:3:6 | cumulative scan | 0.100 / 0.300 / 0.600 | 0.1 / 0.3 / 0.6 |
| Poisson(3) | Knuth product | 3.0004 | 3.0 |

Exponential at `Float106` gave 0.4995 with no additional code, which is the point: they are cheap
*because* the foundation is generic, and impossible to write this way today.

**Cauchy has no mean.** The defining integral diverges, so a sample mean does not converge — it
wanders, and its own distribution is Cauchy again. A test asserting one is not weak but
meaningless, and it will fail intermittently on a seed change while surviving review. The spec
therefore forbids a moment test on Cauchy and requires quantiles instead: the median is the
location, the interquartile range is twice the scale. This is recorded at the item because the
obvious test is the wrong one, and the obvious test is what a later contributor will add.

**Poisson's Knuth algorithm has a domain.** Expected iterations are `λ + 1`, fine for the small
rates a simulation wants. But `exp(-λ)` underflows to zero beyond roughly `λ = 700` at `f64` and
far sooner at `f32`, at which point the loop's termination condition can never be met. The spec
requires the sampler to document its range and either refuse or switch method outside it; which of
the two is an implementation decision, that one of them happens is not.

The other four are inverse-CDF transforms with no such hazard, needing only `u > 0` where they take
a logarithm — obtained by rejection, never by clamping a zero to a small positive value, which
would place mass at that value.

## Decision 8: what the move opens, and what it does not

Putting samplers beside densities is what makes the next layer writable. Importance sampling needs
a density ratio; sequential Monte Carlo needs `log_sum_exp` for weight normalisation; MCMC
diagnostics need the moment machinery. `stats` has `gaussian_log_density`, `log_sum_exp`, `mean`,
`variance` and `covariance_matrix` today. None of those algorithms can be written in `rand` without
dragging the analytic surface two tiers down.

This change does not write them. It removes the reason they could not be written.

## Decision 9: nothing else is misplaced

Checked before proposing, since a retro-design pass should look for its own kind of error
elsewhere. `algorithms::entropy_nvars` calls `stats::entropy` and only marginalises a tensor first;
`topology::covariance_matrix` calls `tensor::sample_covariance`, which calls `stats`;
`discovery::impute_mean` uses `stats::MeanAccumulator` with a comment explaining why it accumulates
rather than slicing. Physics "moments" are momentum and CFD "covariance" is a Kalman state
covariance — physical quantities that share a name with statistical estimators.

No hand-rolled `sum / n` loop bypasses `stats` anywhere in the workspace. That crate is already the
single home for statistics; only the sampling half was missing.
