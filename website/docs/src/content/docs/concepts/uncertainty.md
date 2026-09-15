---
title: Uncertainty
description: A first-order type for uncertain values, plus a companion type for probabilistic presence.
sidebar:
  order: 11
---

DeepCausality treats uncertainty as a first-class type. Two related types ship in the [`deep_causality_uncertain`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_unified_math/deep_causality_uncertain) crate. Both follow the design in Bornholt, Mytkowicz, and McKinley, "Uncertain⟨T⟩: A First-Order Type for Uncertain Data" (ASPLOS '14).

## The uncertainty bug

Most engineering code treats a noisy estimate as an exact value. A single `f64` represents a sensor reading whose real distribution might be `Normal(50.0, 2.5)`. Compound a few of these and the final number carries no record of where it came from or how confident it is. Conditionals on such values silently produce false positives and false negatives.

## Two carriers over one graph

`Uncertain<R>` is a lazy computation graph, generic in its scalar under `RandScalar` — `RealField + FromPrimitive`, which `f32`, `f64`, `BFloat16` and `Float106` all satisfy. `Real` would not do: a probability is a ratio of two counts, and `Real` carries no division; `RealField` is the weakest structure that can state a frequency.

```rust
use deep_causality_uncertain::{SampleSession, Uncertain, UncertainBool};

let noisy = Uncertain::<f64>::normal(50.0, 2.5);   // mean 50, std-dev 2.5
let range = Uncertain::<f64>::uniform(0.0, 100.0);
let exact = Uncertain::<f64>::point(10.0);         // a known value, lifted in
let flip  = UncertainBool::<f64>::bernoulli(0.5);  // a truth value, not Uncertain<bool>
```

The graph has **two carriers rather than one**. `Uncertain<R>` reads a real off it; `UncertainBool<R>` reads a truth value. `Uncertain<bool>` does not exist, and that is a consequence rather than an omission: `Verdict` is instanced twice over two different algebras — the Boolean class on the Boolean carrier (`meet = &`, `join = |`, complement `!`) and the MV class on `[0, 1]` on the real carrier (`min`, `max`, `1 − p`). One type cannot hold both, and dropping either breaks the aggregation machinery downstream. The Boolean carrier keeps its `R` because the tree beneath it holds `R`.

Arithmetic, comparison, and logical operators are overloaded. They extend the graph rather than collapsing it to a number:

```rust
let a = Uncertain::<f64>::normal(10.0, 1.0);
let b = Uncertain::<f64>::normal(5.0, 0.5);
let total = a + b;  // still Uncertain<f64>; the graph is preserved
```

## Addressed draws

A draw is a pure function of three numbers — the session's seed, the sample index, and the leaf's ordinal — and of nothing else. **Nothing is stored between calls.** That is what makes `x - x` exactly zero: within one graph a leaf reached twice is one draw, because it has one address.

It also means a session is reproducible. `expected_value(session, n)` takes its draws at indices `0..n`, so the estimate is a function of the seed and the count alone: asking twice gives the same answer, and a session rebuilt from the same seed reproduces it in a later process. Every sampling method comes in a pair — one taking a `&SampleSession` the caller owns, and a `_from_entropy` form that seeds one itself and cannot be reproduced afterwards.

```rust
let session = SampleSession::seeded(42);
let mean = total.expected_value(&session, 10_000)?;
let sd   = total.standard_deviation(&session, 10_000)?;
```

`to_bool` and `probability_exceeds` on `UncertainBool<R>` use the Sequential Probability Ratio Test, so they stop sampling as soon as the verdict reaches the requested confidence rather than drawing a fixed budget. `expected_value` accumulates through a `MeanAccumulator` that sums as a balanced tree rather than collecting the draws, which keeps memory constant in the sample count and keeps a narrow scalar from stagnating once the running total outgrows an addend.

Two graphs agree about a shared leaf at a given index exactly when the leaf has the **same ordinal** in both, because the ordinal is part of the address. A leaf's ordinal is its position in that graph's traversal: `x` holds ordinal 0 in `x` and in `x + y` and draws the same in each, while `y` holds ordinal 0 alone and ordinal 1 inside the sum and does not.

## Conditionals that respect the distribution

Branching on an uncertain boolean returns an uncertain value rather than collapsing it:

```rust
let traffic_heavy = UncertainBool::<f64>::bernoulli(0.7);
let via_main = Uncertain::<f64>::normal(30.0, 5.0);
let via_back = Uncertain::<f64>::normal(45.0, 2.0);

let eta = Uncertain::conditional(traffic_heavy, via_back, via_main);
```

`Uncertain::conditional` takes an `UncertainBool<R>` as its condition and produces a single uncertain estimate that mixes both branches in proportion to it. It is the controlled exit from the uncertainty world. `implicit_conditional` is the convenience for "more likely than not" decisions; `to_bool(session, threshold, confidence, epsilon, max_samples)` is the explicit form when a specific confidence bar is required.

## MaybeUncertain&lt;T&gt;: probabilistic presence

Some values may not exist at all. A sensor misses a frame; a clinical-trial subject does not report on a given day. `MaybeUncertain<R>` separates two questions: is the value present, and if it is, what is its distribution.

```rust
use deep_causality_uncertain::{MaybeUncertain, Uncertain};

let always_known   = MaybeUncertain::<f64>::from_uncertain(Uncertain::normal(10.0, 2.0));
let always_missing = MaybeUncertain::<f64>::always_none();
let sometimes      = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(
    0.7,
    Uncertain::normal(5.0, 1.0),
);
```

Arithmetic propagates absence. Adding `sometimes + always_missing` returns absence with the probability the operand had of being missing. `is_some()` returns an `UncertainBool<R>` the rest of the framework can reason about. `lift_to_uncertain(presence_threshold, confidence, ...)` collapses the type back into a plain `Uncertain<R>` once there is enough evidence the value is actually there.

## Where it shows up in the framework

A `CausalState` carries an optional `UncertainParameter`, so a Causaloid that emits an uncertain effect can be tested against state-specific confidence and sample-budget settings at fire time. The CDL pipeline can feed uncertain features into discovery without flattening them to point estimates. The Effect Ethos can gate actions on uncertain conditions with explicit confidence bars rather than thresholds on means.

## See also

- Crate README: [`deep_causality_uncertain`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_unified_math/deep_causality_uncertain/README.md).
- Examples: [`gps_navigation`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_uncertain_examples/gps_navigation), [`sensor_processing`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_uncertain_examples/sensor_processing), and [`clinical_trial`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_uncertain_examples/clinical_trial) cover route choice under noise, sensor fusion with anomaly detection, and trial data with probabilistic presence. Each is a daisy-chained monadic pipeline: the uncertain API does the numerical work, the surrounding monad supplies the plumbing and the short-circuit on failure.
- Concept: [Causal State Machine](/concepts/csm/), which uses `UncertainParameter` to gate action firing.
- Background: Bornholt, J., Mytkowicz, T., McKinley, K. S. "Uncertain⟨T⟩: A First-Order Type for Uncertain Data." ASPLOS '14.
