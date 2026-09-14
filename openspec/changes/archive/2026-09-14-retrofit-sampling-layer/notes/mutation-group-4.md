<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Group 4, phase 5: mutation testing across all seven distributions

Run in one batch rather than per distribution. The earlier attempt to run 4C's mutants while 4B was
still red failed at the baseline — `cargo mutants` refuses to start against a tree whose unmutated
tests do not pass — so the runs were deferred to a green tree and batched here.

```
cargo mutants -p deep_causality_stats --timeout 90 \
  -f '.../distr/{unit_interval,normal,exponential,log_normal,cauchy,weibull,categorical,poisson,uniform_int}/**'
```

**140 mutants: 75 caught, 31 unviable, 20 timed out, 14 survived.**

## What the survivors were, and what was done about each

Fourteen mutants survived. **Four were genuine test gaps and are now closed**; ten are exact
equivalences that no test can distinguish and none should be written to chase.

### Genuine gaps, now closed

| Mutant | What it does | Closed by |
|---|---|---|
| `categorical:105:28` `len() - 1` → `len() + 1` | the rounding fallback returns an index past the end | `the_rounding_fallback_returns_the_last_category` |
| `categorical:105:28` `len() - 1` → `len() / 1` | the same, one past the end | the same |
| `cauchy:78:56` `pi * (u - half)` → `pi / (u - half)` | a different, nearly-Cauchy law | `the_cdf_matches_the_closed_form_across_the_body` |
| `normal:49:34` `cv < 0` → `cv <= 0` | rejects a zero coefficient of variation | `a_zero_coefficient_of_variation_is_a_degenerate_normal` |

Each was verified by applying the mutation to the source, observing the new test fail, and
restoring. A test added in response to a surviving mutant and never run against it is not evidence.

### The Categorical fallback: a test that aimed at a branch it never reached

This one is worth the space, because the failure was in the test rather than the code.

`the_draw_stays_in_range_at_the_upper_boundary` already existed, written in 4E for exactly this
branch — the fallback the scan takes when the remainder outlives the weights. It drew from `MaxRng`
at five equal `f64` weights and asserted the result was in range. It passed, it looked like
coverage, and the mutant survived anyway.

It never reached the branch. At five weights of 1.0, `(1 - 2^-53) * 5` less four is below one, so
the last comparison fires and the scan returns normally. The fallback was never executed, and a
test that does not execute a line cannot detect a change to it.

Finding weights that do reach it took measurement, and two attempts:

- A first search said ten `f32` weights of 0.1 reach it. **That was wrong** — it had rounded
  `1 - 2^-53` to `f32`, which is `1.0`, a value the sampler never returns because `StandardUniform`
  rejects it. Written as a test it did not fail; it **hung**, because `MaxRng` at `f32` rejects
  every draw and redraws forever.
- Repeating the search with the largest value each scalar can actually return found the fallback
  reachable in about 19% of random weight vectors — and reachable at **two** weights: `[0.3, 0.7]`.

Neither 0.3 nor 0.7 is representable, and their `f64` sum rounds **up** to exactly 1.0. A draw of
`1 - 2^-53` therefore gives a remainder of `1 - 2^-53`; subtracting 0.3 leaves exactly 0.7, which is
not *less than* 0.7, and the scan runs off the end. Under the mutants the caller is handed index 2
or 3 for two categories.

The case is as ordinary as a categorical distribution gets. What made it invisible was not its
rarity but the arrangement of the test that was supposed to find it.

### The Cauchy cluster, and why it is not a gap

Six of the fourteen survivors are in one three-line sampler. Five of them are exact equivalences,
each a consequence of a symmetry the standard Cauchy actually has:

| Mutant | Why it changes nothing |
|---|---|
| `78:23` `loc + scale·tan` → `loc − scale·tan` | `tan(π(u − ½))` is symmetric about zero, so `−S ~ S` |
| `78:36` `scale · tan` → `scale / tan` | the standard Cauchy is closed under reciprocal: `1/X ~ X` |
| `78:61` `(u − half)` → `(u + half)` | `tan` has period `π`, so the shift is the identity |
| `78:61` `(u − half)` → `(u / half)` | `tan(2πu)` sweeps two whole periods instead of one |
| `76:29` `one / two` → `one * two` | `half` becomes 2, and `tan(π(u − 2)) = tan(πu)` |

These are not weaknesses in the suite. A sampler built from `tan` inherits that function's period,
and a distribution invariant under negation and under reciprocal will report every mutation that
negates or inverts as a survivor. **A mutation score is not a quality measure for a sampler whose
law has symmetries**; what the score measures there is how many symmetries the law has.

The sixth, `78:56`, is the one that was a real gap — and it is subtle. `tan(π / (u − ½))` is *not*
the standard Cauchy, but it is very close to it: the argument sweeps through many periods of `tan`
for most of the unit interval, so the result is nearly the same law. Measured over 200 000 draws it
puts 0.508 of its mass inside `|x| < 1` where the closed form says 0.500. The quartile tests could
not see 0.8 of a percentage point. A CDF check against `(2/π) arctan(t)` at five points, with an
absolute tolerance of 0.004 — about three and a half standard errors at that sample size — can, and
does.

### The other exact equivalences

| Mutant | Why it changes nothing |
|---|---|
| `log_normal:74` `mu + sigma·z` → `mu − sigma·z` | `z` is a standard normal, symmetric about zero |
| `categorical:80`, `uniform_int:54` `is_empty` → `false` | both constructors refuse an empty argument, so `false` is the only value either can return |
| `uniform_int:70` `low < len` → `low <=` | the fast path's guard; at `low == len` the rejection test that follows is false anyway, since `2^64 mod len < len` |
| `uniform_int:73` `low < short_block` → `low <=` | rejects one additional word in `2^64` and redraws it uniformly |

`uniform_int:73` is the same mutant as defect (f) in the 4H audit, found there first and recorded
with the same verdict.

## The twenty timeouts are detections, not gaps

A timeout is not a survivor. Every one of the twenty is a mutation **inside a rejection loop**, and
mutating a rejection loop is how you build a loop that never exits.

Sixteen are in `unit_draws.rs`, which every other distribution draws through. Make the unit draw
always return 1.0 and `StandardUniform`'s guard rejects every value it is handed; invert the guard's
comparison and the same thing happens from the other side. The loop then runs forever, and the run
that would have reported a failure never gets to report anything.

`uniform_int:75` is the clearest single case. The mutation `word * len` → `word / len` applies only
to the line **inside** the loop, so the first draw is computed correctly and only a rejected word
enters the mutated path. There, `word / len` is 0 or 1 at the range the bias test uses, which is
always below the rejection threshold — so every redraw is rejected in turn and the loop cannot end.

The four Poisson timeouts are the one group that does terminate: `product *= u` written as `+=` or
`/=` stops the product from decreasing, so the draw runs to `MAX_ITERATIONS`, a million iterations,
fifty thousand times over. The cap turns what would be a hang into a value, and the moment
assertions then fail — just not inside 90 seconds.

Left as timeouts deliberately. Lowering `MAX_ITERATIONS` to make a mutation run finish sooner would
weaken a bound that exists to stop a degenerate generator hanging a real caller, and 4F's notes
record that this cap was added because a test of mine **hung** rather than failed. The unbounded
loops in `unit_draws.rs` are sound for the opposite reason: their rejection probability is `2^-53`
per draw against any real generator, so the expected number of redraws is one.
