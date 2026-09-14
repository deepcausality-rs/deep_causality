<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Group 1: the three-way split

## Phase 1 — the API

`StandardWord` and `StandardBool` declared; `Distribution<u64>`, `<u32>` and `<bool>` removed from
`StandardUniform`; `Rng::random` split into `random_word` and `random_boolean`. Both `cargo` and
`bazel` build. The two new bodies were confirmed to panic rather than return.

**Internal callers redirected, and the redirection is the finding.** Five sites inside `rand` were
drawing a *machine word* through `StandardUniform` because one type used to mean both:

| Site | What it draws |
|---|---|
| `dist_float_common.rs`, three bit kernels | a `F::UInt` to build a float from bits |
| `uniform_f32.rs`, `uniform_f64.rs` | a `u32` / `u64` to shift into a mantissa |
| `Bernoulli::sample` | a `u64` for the fixed-point comparison |

Each now says `StandardWord`, which is what it always meant. None of these is a real-valued draw.

**One API gap surfaced.** `Rng::map` and `Rng::random_iter` were hardwired to `StandardUniform`, so
a word caller had no route through them. Added `map_word` and `random_word_iter` as their
word-sided siblings. This was not in the task list; it is the split being honest about a hole that
the conflated type had hidden.

## Phase 2 — the suite, observed failing

`tests/types/dist/uniform/standard_word_bool_tests.rs`, 7 tests. Covered by the existing
`tests/types/dist/*/*_tests.rs` glob, so no BUILD change.

```
test result: FAILED. 0 passed; 7 failed; 0 ignored; 154 filtered out
```

Whole crate: **103 passed, 58 failed**.

Every one of the 58 is the phase-1 `unimplemented!()` panic at
`src/types/distr/uniform/standard_uniform.rs`, verified by counting: 58 of 58 panic lines name that
file. No compile error, no missing import, no panic from elsewhere — the phase-2 exit condition.

The 58 is larger than the 7 new tests because the float distributions draw their words through
`StandardWord`, so every float test fails too until phase 4 lands. That is the dependency the
redirection created and it resolves with the same implementation.

## Migrated tests, and why that is not weakening them

Three existing files asserted word and Boolean behaviour spelled as `StandardUniform`:

- `types/dist/uniform/standard_uniform_tests.rs` — `test_standard_uniform_{u32,u64,bool}` renamed
  to `test_standard_word_{u32,u64}` and `test_standard_bool`, same assertions.
- `traits/distribution_tests.rs` — the `Map` over a `u32` source is a word source.
- `types/rand/std_rng_tests.rs` — the `u32` iterator and the `u32 -> u64` map are word-sided.

One assertion genuinely changed. The old Boolean test pinned `next_u64() % 2 == 0`, a parity rule
that reads one bit of information from a whole word. `StandardBool` takes a bit directly, so the
mock is now selected by its top bit. The test asserts the same contract — a word yields a
determinate Boolean — against the replacement rule rather than the one being removed.

## Phase 3 — the deliberate-defect audit

Each defect was written against the working implementation, the suite run, and the result recorded.

| # | Defect | Caught by | How |
|---|---|---|---|
| a | `StandardBool` as `next_u64() % 2 == 0` | `test_standard_bool` | 160 passed, 1 failed |
| b | `random_word` delegating to `StandardUniform` | **the compiler** | `error[E0277]: the trait bound StandardUniform: Distribution<u64> is not satisfied` |
| c | `StandardBool` returning a constant | 3 tests | `test_standard_bool`, `standard_bool_yields_both_values`, `standard_bool_is_unbiased` |

**Defect (b) is rejected at compile time, not by a test.** Once `StandardUniform` no longer
implements `Distribution<u64>`, a `random_word` that reaches for it cannot be written. That is a
stronger guarantee than a test: the separation is enforced by the type system, and a future
contributor cannot reintroduce the conflation by accident. This is the payoff of splitting the
types rather than splitting only the method names.

**Defect (a) is caught by a migrated test, not a new one.** `test_standard_bool` is the old parity
test, re-pointed at `StandardBool` and re-based on the top bit. Rewriting it was necessary because
the rule it pinned is the rule being removed; it still earns its place, because it is what detects
a regression to that rule.

**Defect (c) is caught three times over.** A constant Boolean fails the both-values test, the
unbiased-band test, and the mock-based determinacy test. Redundancy here is cheap and the failure
mode is common.

## Phase 4 — implementation

`StandardWord::sample` returns `next_u64` / `next_u32`; `StandardBool::sample` takes the top bit.
Three lines of body.

```
passed: 161  failed: 0
```

Against the 154 baseline: **+7**, exactly the new suite. No pre-existing test regressed.

## Phase 5 — mutation testing

`scripts/mutants.sh deep_causality_rand src/types/distr/uniform/standard_uniform.rs`.

9 outcomes: 1 baseline, **6 caught, 0 missed, 2 timeout**.

### Caught

| Line | Mutant |
|---|---|
| 41 | `StandardWord::sample -> u32` replaced with `0` |
| 41 | ... with `1` |
| 49 | `StandardBool::sample -> bool` replaced with `true` |
| 49 | ... with `false` |
| 49 | `==` replaced with `!=` |
| 49 | `>>` replaced with `<<` |

The `>>` → `<<` mutant matters: it reads the *low* bit instead of the top one, which is the parity
defect from phase 3 arriving by a different route. The suite rejects it, so the top-bit choice is
pinned rather than incidental.

### The two timeouts, and their verdict

Both mutate the `u64` arm — `StandardWord::sample -> u64` replaced with `0` and with `1`. Neither
is a survivor and neither is a test gap. The suite **hangs**, which I confirmed by writing the
mutant by hand and running `cargo test` under a 120-second limit: it was killed at the limit rather
than failing.

The cause is two rejection loops that consume words until a condition changes:

- `types/distr/normal/standard_normal.rs:44`, the ziggurat tail: `while -2.0 * y < x * x` redraws
  two `Open01` values per iteration.
- `utils/ziggurat_sampler.rs:33`, the ziggurat outer loop.

A constant `u64` makes every `Open01` draw identical, so `x` and `y` never change and the exit
condition can never be reached.

**Verdict: caught, not survived.** A hang is a detection — a suite that never terminates has
rejected the mutant as surely as one that fails an assertion, and the timeout is the tool reporting
that it could not wait, not that the mutant passed. Recording them as survivors would be wrong; so
would recording them as equivalents, because they are not equivalent to the original at all.

They are **not** added to `.cargo/mutants.toml`. That file is for mutants that genuinely cannot be
distinguished from the original, and these are distinguished very clearly. A future run will time
out on them again, and this note is the reason.

There is a real finding underneath: **the ziggurat has no iteration cap**. On a degenerate generator
it does not terminate. That is out of scope here — group 2 moves the normal distribution to `stats`
and group 3 replaces the ziggurat with Box–Muller, which has no rejection loop on the main path —
but it is worth stating that the current code has this property, and that the replacement removes
it rather than inheriting it.

## Phase 6 — close-out

`cargo test -p deep_causality_rand`: **161 passed, 0 failed** (baseline 154, plus the 7 new tests).
`bazel test //deep_causality_unified_math/deep_causality_rand:all`: **24 of 24 targets pass**.
`cargo clippy --all-targets --features os-random`: clean, no `#[allow]` added.

**Bazel caught a file cargo's default features hid.** `tests/types/rand/os_random_rng_tests.rs`
sits behind the `os-random` feature, so `cargo test` never compiled it and reported green while it
was broken. Bazel builds every target, found three word-draw sites there, and they were migrated
the same way as the other four files. Worth recording: a group that checks only `cargo test` can
believe itself finished with a feature-gated file still failing.
