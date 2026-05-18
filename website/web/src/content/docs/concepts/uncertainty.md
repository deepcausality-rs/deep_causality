---
title: Uncertainty
description: A first-order type for uncertain values, plus a companion type for probabilistic presence.
section: concepts
order: 10
---

DeepCausality treats uncertainty as a first-class type. Two related types ship in the [`deep_causality_uncertain`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_uncertain) crate. Both follow the design in Bornholt, Mytkowicz, and McKinley, "Uncertain⟨T⟩: A First-Order Type for Uncertain Data" (ASPLOS '14).

## The uncertainty bug

Most engineering code treats a noisy estimate as an exact value. A single `f64` represents a sensor reading whose real distribution might be `Normal(50.0, 2.5)`. Compound a few of these and the final number carries no record of where it came from or how confident it is. Conditionals on such values silently produce false positives and false negatives. The crate README calls this "the uncertainty bug" and gives it a name worth fixing in the type system.

## Uncertain&lt;T&gt;

`Uncertain<T>` wraps a value along with the distribution that produced it:

```rust
use deep_causality_uncertain::Uncertain;

let noisy = Uncertain::<f64>::normal(50.0, 2.5);  // mean 50, std-dev 2.5
let range = Uncertain::<f64>::uniform(0.0, 100.0);
let flip  = Uncertain::<bool>::bernoulli(0.5);
let exact = Uncertain::<f64>::point(10.0);        // a known value, lifted in
```

Arithmetic, comparison, and logical operators are overloaded. They build an implicit computation graph rather than collapsing to a number:

```rust
let a = Uncertain::<f64>::normal(10.0, 1.0);
let b = Uncertain::<f64>::normal(5.0, 0.5);
let total = a + b;  // still Uncertain<f64>; the distribution is preserved
```

Evaluation is lazy and sampling-based. `expected_value(n)` and `standard_deviation(n)` draw samples; `estimate_probability(n)` runs the chain to a target precision; `to_bool(confidence)` and `probability_exceeds(threshold, confidence, samples)` use the Sequential Probability Ratio Test so they stop sampling as soon as the verdict reaches the requested confidence. A thread-local cache memoizes draws so repeated reads of the same graph node do not redo work.

## Conditionals that respect the distribution

Branching on an uncertain boolean returns an uncertain value rather than collapsing it:

```rust
let traffic_heavy = Uncertain::<bool>::bernoulli(0.7);
let via_main = Uncertain::<f64>::normal(30.0, 5.0);
let via_back = Uncertain::<f64>::normal(45.0, 2.0);

let eta = Uncertain::conditional(traffic_heavy, via_back, via_main);
```

`Uncertain::conditional` is the controlled exit from the uncertainty world; it produces a single uncertain estimate that mixes both branches in proportion to the condition. `implicit_conditional()` is the convenience for "more likely than not" decisions; `to_bool(confidence)` is the explicit form when a specific confidence bar is required.

## MaybeUncertain&lt;T&gt;: probabilistic presence

Some values may not exist at all. A sensor misses a frame; a clinical-trial subject does not report on a given day. `MaybeUncertain<T>` separates two questions: is the value present, and if it is, what is its distribution.

```rust
use deep_causality_uncertain::{MaybeUncertain, Uncertain};

let always_known   = MaybeUncertain::<f64>::from_uncertain(Uncertain::normal(10.0, 2.0));
let always_missing = MaybeUncertain::<f64>::always_none();
let sometimes      = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(
    0.7,
    Uncertain::normal(5.0, 1.0),
);
```

Arithmetic propagates absence. Adding `sometimes + always_missing` returns absence with the probability the operand had of being missing. `is_some()` returns an `Uncertain<bool>` the rest of the framework can reason about. `lift_to_uncertain(presence_threshold, confidence, ...)` collapses the type back into a plain `Uncertain<T>` once there is enough evidence the value is actually there.

## Where it shows up in the framework

A `CausalState` carries an optional `UncertainParameter`, so a Causaloid that emits an uncertain effect can be tested against state-specific confidence and sample-budget settings at fire time. The CDL pipeline can feed uncertain features into discovery without flattening them to point estimates. The Effect Ethos can gate actions on uncertain conditions with explicit confidence bars rather than thresholds on means.

## See also

- Crate README: [`deep_causality_uncertain`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_uncertain/README.md).
- Examples: [`gps`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_uncertain/examples/gps), [`sensor`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_uncertain/examples/sensor), and [`clinical_trial`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_uncertain/examples/clinical_trial) cover route choice under noise, sensor fusion with anomaly detection, and trial data with probabilistic presence.
- Concept: [Causal State Machine](/docs/concepts/csm/), which uses `UncertainParameter` to gate action firing.
- Background: Bornholt, J., Mytkowicz, T., McKinley, K. S. "Uncertain⟨T⟩: A First-Order Type for Uncertain Data." ASPLOS '14.
