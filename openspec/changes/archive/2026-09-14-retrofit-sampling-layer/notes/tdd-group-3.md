<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Group 3: precision as a parameter in `stats`

## Phase 1 — the API

`RandWidth` declared with four implementations; `StandardUniform`, `Open01`, `OpenClosed01` and
`StandardNormal` each declared as **one blanket implementation** over
`RealField + FromPrimitive + RandWidth`, bodies `unimplemented!()`.

**No E0119.** The conflict that blocked this retrofit for the life of the crate is gone, and it is
worth being precise about why, because two separate things removed it:

- Group 1 split `StandardUniform` from `StandardWord` and `StandardBool`, so the blanket no longer
  meets a `Distribution<u64>` implementation on the same type.
- Group 2 moved the type to `stats`, which has no reason to sample a machine word at all.

Either alone would have silenced the error. Together they mean it cannot come back: `stats` does
not own a word sampler to collide with.

### Deleted

| File | Why |
|---|---|
| `dist_float_32.rs`, `dist_float_64.rs`, `dist_float_106.rs` | one generic body replaces nine per-type implementations |
| `dist_float_common.rs` | the shared bit kernels the three called |
| `ziggurat_sampler.rs`, `ziggurat_tables.rs` | the `f64` normal's kernel, unreachable once Box–Muller is generic |

**12 per-type `Distribution` implementations become 4 generic ones.**

### Two costs, stated rather than buried

The ziggurat is faster than Box–Muller at `f64` — it avoids two transcendentals per draw. Trading
it away is a real performance decision. It is the right trade because a ziggurat cannot serve a
scalar wider than the table it is built from, and precision as a parameter is the point; a
specialised `f64` path can return behind the same generic surface later as a *measured*
optimisation.

It also removes an unbounded loop. The ziggurat's tail is a rejection sampler with no iteration
cap, so on a degenerate generator it does not terminate — which is what the two group-1 mutation
"timeouts" were really reporting. Box–Muller has no rejection on its main path.

### Phase-1 exit

`cargo build` and `bazel build` both succeed. 500 passed, **30 failed**, and every failure is the
`unimplemented!()` panic at `unit_draws.rs:28` or `standard_normal.rs:40` — no compile error, no
panic from elsewhere.

## Phase 2 — the suite, observed failing

`tests/types/distr/unit_interval/precision_tests.rs`, 10 tests, against the phase-1 surface.

```
501 passed, 39 failed
```

Every one of the 39 is an `unimplemented!()` panic, at exactly the four declared bodies:

| Site | Distribution | Failures |
|---|---|---|
| `unit_draws.rs:28` | `StandardUniform` | 18 |
| `unit_draws.rs:38` | `Open01` | 7 |
| `unit_draws.rs:48` | `OpenClosed01` | 5 |
| `standard_normal.rs:40` | `StandardNormal` | 9 |

No compile error, no missing import, no panic from anywhere else. The phase-2 exit condition.

The count is larger than the 10 new tests because the suites moved in group 2 exercise the same
bodies; they fail for the same reason and recover with the same implementation.

### What the suite pins

- **The load-bearing test**: more than half of 200 `Float106` draws must differ from their own
  `f64` round trip. Every other test in this file passes against a single-word draw, so this is
  the only one that can catch a double-double that is secretly an `f64`.
- Uniform mean and `E[x^2]`, and normal variance, at `f32`, `f64` and `Float106`.
- Every draw in `[0, 1)` — 2 000 per scalar, and **5 000 for `BFloat16`**, whose 8-bit significand
  is the one that can round onto the upper bound.
- `Open01` and `OpenClosed01` exclude their endpoints, and `ln(u)` is finite over 10 000 `Open01`
  draws — a zero there would give an infinity no bounds check on the *result* would catch.
- `BFloat16` samples at all, which no amount of effort achieved before this change.
- `BFloat16`'s mean over 64 draws, a count chosen to stay below the saturation the scalar's 8-bit
  significand imposes. The saturation is the scalar's property, not the sampler's.
- The width constant is what each scalar declares, so a wrong value fails here rather than silently.
- **One bound**: a Monte Carlo integral written against `RealField + FromPrimitive + RandWidth`
  alone, with no `where StandardUniform: Distribution<S>` clause.

## Phase 4 — implementation

Four generic bodies. `StandardUniform` accumulates `T::WORDS` words, each placed `2^-53` below the
last, and rejects a value that rounded onto `1.0`. `Open01` rejects a zero. `OpenClosed01` is
`1 - u`. `StandardNormal` is Box–Muller over `Real`'s `ln`, `sqrt`, `cos` and `pi`.

**542 passed, 0 failed.**

## Phase 3 — the deliberate-defect audit

Run against the working implementation.

| # | Defect | Result |
|---|---|---|
| a | a single word for every scalar | **1 failure**: the low-limb test, `0/200` |
| b | `WORDS = 1` for `Float106` | **2 failures**: the low-limb test and the width-constant test |
| c | Box–Muller computed in `f64` and widened | **1 failure**: `test_double_double_entropy`, a suite moved in group 2 |
| d | clamp instead of redraw | **survived** — see below |

### (a) is why the low-limb test exists

539 of 540 tests passed with a `Float106` that was secretly an `f64`. Every moment test, every
bounds test, every `BFloat16` test, and the whole 500-test body of the crate. Only the low-limb
test caught it, reporting exactly the `0/200` the earlier probe predicted.

That is the argument for the requirement, measured rather than asserted: a wrong width is
undetectable by any test of the *distribution*, because the distribution is correct. Only the
*representation* is wrong.

### (d) survived, and the suite was repaired

The clamp — returning the largest value below 1 instead of redrawing — passed every test in the
file. Bounds hold, the mean holds, the moments hold; only the shape at one point changes, and
nothing was looking there.

Task 3.3(d) anticipated this and said to add the check if it was missing. It was. Calibrated by
measurement over 200 000 `BFloat16` draws:

| Behaviour | Share taken by the most frequent value |
|---|---|
| redraw (correct) | 0.00429 |
| clamp (defect) | 0.00584 |

A 36% excess on one point. The new test asserts the share is below 0.005, which sits between the
two and nearer the correct value. Re-running defect (d) against it: **1 failure**, the new test.

### An implementation change the audit prompted, and a claim withdrawn

While calibrating, the intermediate was moved from the target scalar into `f64`, converted once —
`BFloat16` keeps 8 significand bits, so accumulating there would round before the value is formed.

I expected that to move the measured distribution. **It did not**: 0.00429 either way. The two
paths agree because the division is exact at both widths for the values involved. The `f64` form
is kept, because it rounds once by construction rather than by coincidence and is the same shape
for every scalar — but the docstring now records that the measurement did not support the
motivation, rather than implying a fix that was not one.

## Phase 4b — the range seam, collapsed too

After the distribution layer was generic, one spec scenario still failed:
`deep_causality_rand` carried `uniform_f32.rs`, `uniform_f64.rs` and `uniform_f106.rs`, each with a
hand-written `[0, 1)` draw. The requirement says **neither** crate carries a per-type source file
for a float it supports, and the range seam was not exempt just because group 2 pinned its
*location*.

The orphan rule pins the `SampleUniform` bindings — `impl SampleUniform for f64` can only be
written in the crate owning the trait, and a blanket over `RealField` would collide with the
integer bindings beside it, which is the same `E0119` that shaped this whole change. It does **not**
pin the draw logic.

So `RandFloat::rand_float_gen` became a provided method with one generic body, mirroring
`stats`'s `RandWidth`, and the three files became one holding three empty implementations, one
`WORDS = 2`, and three one-line bindings that carry no logic.

The two traits stay separate rather than one being reused: the crates are separate, the range
sampler owes nothing to the distribution layer, and a scalar may implement one without the other.

`deep_causality_rand`: **101 passed, 0 failed** — the collapse is behaviour-preserving.

### The scenario now holds

```
find rand/src stats/src \( -name '*f32*' -o -name '*f64*' -o -name '*106*' -o -name '*bf16*' \)
  -> nothing
```

**Zero per-type float source files in either crate.**

| Crate | Lines in `src/` | Baseline |
|---|---|---|
| `deep_causality_rand` | 1398 | 2293 |
| `deep_causality_stats` | 3498 | 2788 |
| combined | **4896** | 5081 |

185 lines net removed, while gaining `BFloat16` sampling and a scalar bound that no longer names a
concrete distribution type.

## Phase 5 — mutation testing, and two gaps it found

First run on `unit_draws.rs`: 5 caught, **4 missed**, 14 timeout, 4 unviable. The four survivors
were two distinct problems, and both were real.

### Three survivors, one dead statement

```
unit_draws.rs:59  replace *= with /= in unit
unit_draws.rs:59  replace *= with += in unit
unit_draws.rs:57  replace += with -= in unit
```

The loop was:

```rust
for _ in 1..T::WORDS {
    acc += word() * scale;
    scale *= word_scale;   // last statement
}
```

`WORDS` is at most 2, so the loop runs at most once and **the final `scale` update is never read**.
Every mutation of a dead statement is equivalent, so no test could have caught these — the suite
was not at fault.

Fixed by deepening the scale *before* use rather than after, which is the same arithmetic with no
dead write:

```rust
for _ in 1..T::WORDS {
    scale = scale * word_scale;
    acc += word() * scale;
}
```

Recording it as an equivalent mutant would have been true and useless. The dead statement was the
defect, and mutation testing is what exposed it.

### One survivor, a case sampling cannot reach

```
unit_draws.rs:95  replace > with >= in Open01::sample
```

`Open01` must exclude zero, and a zero draw has probability `2^-53` — one in nine quadrillion. The
10 000-draw guard in the suite will never see one, so `>=` passes every test while admitting the
endpoint.

Closed with a generator whose words are all zero, which reaches the case on the first draw. Under
correct behaviour `Open01` redraws forever on such a generator, so the test asserts the guard is
*reached* — a bounded wait on a separate thread — rather than that a value comes back. Its
complement asserts `StandardUniform` does return zero there, which is the reason the two types
differ at all.

Verified: with `>=` in place, the new test fails with "Open01 returned a value from an all-zero
generator".

### The 14 timeouts

The same shape as group 1's. Mutating the rejection comparison — `if v < T::one()` to `>` or `==` —
makes the loop unsatisfiable, so the suite hangs rather than fails. A hang is a detection; these
are caught, not survived, and are not equivalent mutants.

### Second run, and the last survivor

After both repairs: **27 mutants — 1 missed, 7 caught, 4 unviable, 15 timeouts.** Down from 4
missed.

The remaining one:

```
unit_draws.rs:62  replace += with -= in unit
```

Subtracting the second limb instead of adding it. It is invisible to every statistical test in the
file, and for a reason worth stating: the shift is about `1e-16`, far below any moment tolerance;
the low limb is still populated, so the entropy test still passes; and the value stays inside
`[0, 1)` unless the first limb is exactly zero, which has probability `2^-53`.

So it is not a weak assertion anywhere — **no distributional test can see it**. The rule being
broken is arithmetic, not statistical, and it needed an arithmetic test.

Closed with a generator that hands back a scripted sequence of words:

- a zero first limb and a positive second one: a sum stays at or above zero, a difference goes
  below it;
- two chosen words: the result must equal `hi + lo * 2^-53` exactly.

Verified: with `-=` in place, **both** new tests fail.

### What the three repairs have in common

None was a lazily written test. One was dead code the suite could not have reached, one was a case
at probability `2^-53`, and one was an arithmetic rule that no statistical assertion constrains.
Mutation testing earned its place here by finding the kinds of gap that reading the tests would
not have revealed.

### Third run: zero survivors

```
27 mutants tested: 8 caught, 4 unviable, 15 timeouts
```

The `missed` category is gone. The 4 unviable are mutants of the `WORD_SCALE` constant that do not
compile, and the 15 timeouts are the unsatisfiable rejection loops described above.

`deep_causality_stats`: **546 passed, 0 failed.** Clippy clean.
