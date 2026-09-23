# Aspirin Headache Trial Analysis Example

A stateless five-stage `CausalFlow` chain over `MaybeUncertain<f64>` compares
aspirin with placebo in a small clinical trial where data presence is itself
uncertain.

## Pipeline

```
CausalFlow::effect()
    .map(|_| cohort_stage())    // Stage 1: assemble per-patient MaybeUncertain values
    .map(presence_stage)        // Stage 2: print Bernoulli-style presence probabilities
    .try_step(lift_stage)       // Stage 3: lift MaybeUncertain → Uncertain per patient; Err if none clears
    .map(aggregate_stage)       // Stage 4: average within each arm
    .map(verdict_stage)         // Stage 5: probability_exceeds verdict
    .run(on_ok, on_err)
```

## Why `MaybeUncertain` + `CausalFlow`

Patient data has two independent uncertainty sources:

- **Presence uncertainty:** whether the measurement exists at all (dropout,
  missed visit, intermittent reporting), modelled by the Bernoulli arm of
  `MaybeUncertain`.
- **Value uncertainty:** how noisy the measurement is, given it exists,
  modelled by the `Uncertain<f64>` arm.

`MaybeUncertain` propagates `None` through arithmetic. `lift_stage` carries
that into the flow at the `lift_to_uncertain_from_entropy` boundary. A patient
who fails the presence gate drops out of their arm. If no patient in either arm
clears the gate, `lift_stage` returns `Err(CausalityError)`, `try_step` skips
the remaining stages, and `run` calls its error handler. An arm left empty
while the other is not reaches `verdict_stage`, which reports insufficient
data instead of a verdict.

## What the example demonstrates

- **`MaybeUncertain<f64>` constructors:** `from_value`, `from_uncertain`,
  `from_bernoulli_and_uncertain`, `always_none`.
- **Presence assessment:** `is_some` returning `Uncertain<bool>`, then
  `estimate_probability_from_entropy`.
- **Probabilistic gating:** `lift_to_uncertain_from_entropy(min_presence,
  confidence, epsilon, samples)` as a per-patient reliability filter; patients who pass
  contribute to the arm average, the others drop out.
- **Per-arm aggregation:** average `Uncertain<f64>` via `reduce` + division
  by cohort size.
- **Comparative verdict:** `greater_than` + `probability_exceeds` for
  evidence-based recommendation.
- **Flow chaining:** four `map` calls and one `try_step` on `CausalFlow`.
  Each stage is a stateless function: `T -> U` for `map`, and
  `T -> Result<U, CausalityError>` for `try_step`.

## How to run

```bash
cargo run -p causal_uncertain_examples --example clinical_trial
```
