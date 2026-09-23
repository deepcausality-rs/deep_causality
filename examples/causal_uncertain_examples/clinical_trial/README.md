# Aspirin Headache Trial Analysis Example

A stateless five-stage `PropagatingEffect` chain over `MaybeUncertain<f64>`
compares aspirin with placebo in a small clinical trial where data presence is
itself uncertain.

## Pipeline

```
PropagatingEffect::pure(())
    .bind(|_, _, _| cohort_stage())           // Stage 1: assemble per-patient MaybeUncertain values
    .bind(presence_stage)                     // Stage 2: print Bernoulli-style presence probabilities
    .bind(lift_stage)                         // Stage 3: lift MaybeUncertain → Uncertain per patient
    .bind(aggregate_stage)                    // Stage 4: average within each arm
    .bind(verdict_stage)                      // Stage 5: probability_exceeds verdict
```

## Why `MaybeUncertain` + `PropagatingEffect`

Patient data has two independent uncertainty sources:

- **Presence uncertainty:** whether the measurement exists at all (dropout,
  missed visit, intermittent reporting), modelled by the Bernoulli arm of
  `MaybeUncertain`.
- **Value uncertainty:** how noisy the measurement is, given it exists,
  modelled by the `Uncertain<f64>` arm.

`MaybeUncertain` propagates `None` through arithmetic. Mapping that onto
`CausalEffect::none()` at the `lift_to_uncertain` boundary makes the chain
short-circuit: failed lifts drop out of the arm, and empty arms drop out of
the verdict, with no `if let Err(_) = ... { return; }` ladders.

## What the example demonstrates

- **`MaybeUncertain<f64>` constructors:** `from_value`, `from_uncertain`,
  `from_bernoulli_and_uncertain`, `always_none`.
- **Presence assessment:** `is_some` returning `Uncertain<bool>`, then
  `estimate_probability`.
- **Probabilistic gating:** `lift_to_uncertain(min_presence, confidence,
  epsilon, samples)` as a per-patient reliability filter; patients who pass
  contribute to the arm average, the others drop out.
- **Per-arm aggregation:** average `Uncertain<f64>` via fold + division by
  cohort size.
- **Comparative verdict:** `greater_than` + `probability_exceeds` for
  evidence-based recommendation.
- **Monadic chaining:** five `bind` calls on `PropagatingEffect`; each stage
  is a stateless `CausalEffect<T> -> PropagatingEffect<U>` function.

## How to run

```bash
cargo run -p causal_uncertain_examples --example clinical_trial
```
