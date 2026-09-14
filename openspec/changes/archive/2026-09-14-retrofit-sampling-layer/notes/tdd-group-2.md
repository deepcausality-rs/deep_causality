<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Group 2: the move, `rand` → `stats`

## Phase 0 — what the orphan rule pins, and a correction to the plan

The proposal listed `Uniform<T>` among the types moving to `stats`. Measurement says it cannot go,
and the reason is worth recording because it changes the shape of the boundary.

`Uniform<X>` is generic over `SampleUniform`, whose `f64` implementation must live wherever the
trait lives — `impl SampleUniform for f64` is a foreign type, so only the trait's own crate may
write it. `SampleUniform` in turn backs `Rng::random_range`, and **every external caller of
`random_range` draws an integer range**: `ultragraph` and `deep_causality_data_structures` in their
benchmarks, `deep_causality_tensor` in its. Those are index draws, and the whole point of the split
is that a graph library reaching a statistics crate to pick an index would be absurd.

So the range machinery — `SampleRange`, `SampleUniform`, `UniformSampler`, `Uniform<X>`,
`UniformFloat`, `RandFloat` — stays in `rand`. The line is not "anything with a distribution's
name" but **does it need the analytic surface**: a uniform over a range is arithmetic on its
bounds, while a normal needs `ln`, `sqrt` and `cos`.

`Distribution` stays in `rand` for the same reason. It is the bridge — "something you can sample
from a generator" — and `StandardWord` implements it. `stats` implements it for its own types,
which is a local type against a foreign trait and therefore allowed.

## Phase 0b — a pre-existing defect the decoupling exposed

To move `StandardUniform` out, `rand`'s range machinery had to stop depending on it. Two sites did:
`RandFloat for Float106`, and the `SampleRange` implementations for `Range<f32>` and `Range<f64>`.
Both now build from machine words directly.

Re-pointing `SampleRange` at `RandFloat` broke a test, and the break was a finding rather than a
regression. **`rand` carried two different `[0, 1)` constructions and used them behind two names
for one operation.** Measured at a mock word of `0.5`:

| Path | Kernel | Value on `10.0f32..20.0f32` |
|---|---|---|
| `random_range(a..b)` | 24-bit multiply, via `StandardUniform` | `15.0` |
| `Uniform::new(a, b).sample()` | 23-bit transmute, via `RandFloat` | `14.999999` |

Both are valid uniform draws. Having two of them, reachable through two APIs documented as the same
thing, was the defect. The decoupling unifies them on `RandFloat`.

`test_f32_sample_single_in_range` and its `f64` sibling asserted the literal `15.0`, so they were
pinning the divergence. They now assert that the two paths **agree** — a property that cannot
silently drift the way a magic constant can — alongside the range contract they already checked.
The literal is recorded in the test's comment rather than in an assertion, so a reader can see what
changed and why.

`deep_causality_rand`: **161 passed, 0 failed** after the decoupling, unchanged from the group-1
close.

## Phase 1 — the move

24 files moved with `git mv`, history preserved: 14 source, 10 test.

| To `stats` | Why |
|---|---|
| `Bernoulli`, `Normal`, `StandardNormal` | shaped distributions; the normal needs `ln`, `sqrt`, `cos` |
| `StandardUniform`, `Open01`, `OpenClosed01` + their float impls | a value on `[0, 1)` is a real number |
| the four inverse-CDF functions | the mathematics of a named distribution |
| the ziggurat sampler and its tables | the normal's kernel |
| `BernoulliDistributionError`, `NormalDistributionError` | travel with their types |

| Stays in `rand` | Why |
|---|---|
| `Distribution` | the bridge both crates speak; `StandardWord` implements it |
| `Uniform<X>`, `UniformFloat`, `RandFloat`, `SampleUniform`, `SampleRange` | range sampling, pinned by the orphan rule and by integer callers |
| `UniformDistributionError` | the range machinery's error |
| `SobolSequence`, `Xoshiro256`, `RngCore`, `Fill` | entropy |

`Rng` lost `random`, `random_iter` and `map`, which named `StandardUniform`. They reappear in
`stats` as **`RandomExt`**, an extension trait blanket-implemented for every `Rng`. That is an
extension, not a second `Rng`: it adds methods to existing generators and introduces no new bound.

`RealRng` was removed here rather than in task 5.1. Its blanket names `StandardNormal`, which moved,
so it could not compile; pulled forward for the same reason as task 1.7.

### What `stats` re-exports, and what it does not

Named re-exports only: `Distribution`, `Rng`, `RngCore`, `Xoshiro256`, `rng`. A distribution that
cannot be sampled is an incomplete surface, so the generator comes with it. Never a glob — a
`pub use` is an alias to the same item, while a second trait of the same shape would be a different
item entirely, and a glob would give every entropy type two paths.

## Phase 2 — consumers, repaired in the group that broke them

Per the ordering invariant from group 1, every consumer this group breaks is repaired here rather
than deferred to group 5.

| Crate | Repair |
|---|---|
| `uncertain` | 3 import sites: the distribution layer, the error enum, the QMC inverse CDFs |
| `topology` | **all 13 `rand` references**; `rand` removed from its manifest, `stats` added, and the `os-random` feature forwarded through `stats` |
| `physics` | `Normal` plus 2 numerical-draw files |
| `algorithms` | 1 numerical draw, plus 5 test and bench files |
| `discovery`, examples | import moves |

### A measurement error, corrected

My consumer classification in the proposal counted "distribution uses" by searching for
distribution *type names*. That missed `rng.random::<f64>()`, which is a numerical draw and
therefore a distribution use. `algorithms` was recorded as entropy-only with 0 distribution uses;
it has one, in `dag_sampling/sample.rs`. `physics` was recorded with 6; it has 9.

The conclusion the table supported still holds — `ultragraph` and `data_structures` really do draw
only integer ranges, and that is what pins the range machinery in `rand` — but the counts were low
and the method was the reason.

## Phase 3 — verification

```
deep_causality_rand        101 passed, 0 failed   (154 baseline; 53 moved out)
deep_causality_stats       529 passed, 0 failed   (467 baseline; 62 moved in, incl. 5 new)
deep_causality_uncertain   248 passed, 0 failed   (baseline)
deep_causality_topology   1697 passed, 0 failed   (baseline)
```

630 against a combined 621 baseline for the two moved crates: +7 from group 1's word/bool suite,
+5 new `RandomExt` tests, −3 numerical generator tests that moved rather than duplicated.

`cargo build --workspace --all-targets` clean. Clippy clean on both crates, no `#[allow]` added.

**Bazel caught two things cargo did not.** The new `stats` test directories needed `rust_test_suite`
targets — `tests/types/distr/*/`, `tests/traits/`, `tests/utils/` — or their tests would have run
under cargo and silently not under bazel. And `rand`'s `extensions` suite became an empty glob,
which bazel rejects outright (`allow_empty` defaults to false) while cargo neither knew nor cared.
