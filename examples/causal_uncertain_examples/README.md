# Causal Uncertain Examples

Runnable examples for [`deep_causality_uncertain`](../../deep_causality_uncertain),
each written as a chained monadic pipeline. The `Uncertain<f64>` and
`MaybeUncertain<f64>` API (sampling, distributions, comparisons,
`lift_to_uncertain`, etc.) does the numerical work inside each stage; the
monad supplies the chaining, log accumulation, and short-circuit on failure.

## Examples

| Example          | Monad                                          | Topic                                                                                          | Command                                                                       |
|------------------|------------------------------------------------|------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------|
| GPS Navigation   | `PropagatingEffect` (stateless)                | Propagate position noise through distance → time → route decision → fuel                       | `cargo run -p causal_uncertain_examples --example gps_navigation`             |
| Sensor Processing| `PropagatingProcess<_, FleetState, FleetConfig>` (stateful) | Six-stage fleet pipeline: triage → validate → fuse → anomaly → fallback → reliability verdict  | `cargo run -p causal_uncertain_examples --example sensor_processing`          |
| Clinical Trial   | `PropagatingEffect` over `MaybeUncertain<f64>` | Five-stage aspirin trial: cohort → presence → lift → aggregate → verdict                       | `cargo run -p causal_uncertain_examples --example clinical_trial`             |

## Why a Monad per Example, Not One for All

The three examples have different shapes:

- **gps_navigation** is one-shot data flow with no accumulated state. A
  stateless `PropagatingEffect::pure(x).bind(...).bind(...)` chain is the
  simplest fit. Each stage pulls an `Uncertain<f64>` out of
  the `CausalEffect` via `.into_value()` and re-lifts a transformed one.
- **sensor_processing** has real per-stage state (`healthy_count`,
  `failed_count`, `total_uncertainty`, `fused_temp`, `anomalies`, `verdict`)
  and a read-only configuration (plausibility bands, calibration offsets).
  These map one-to-one onto `PropagatingProcess`'s `State` and `Context`
  channels. Each stage logs to `EffectLog`, printed once at the end, the
  same shape as the avionics
  [`flight_envelope_monitor`](../avionics_examples/control/flight_envelope_monitor)
  pipeline.
- **clinical_trial** uses `MaybeUncertain<f64>` because data presence is
  itself uncertain. The chain's short-circuit semantics
  (`CausalEffect::none()`) mirror the `None` propagation of
  `MaybeUncertain` arithmetic: failed `lift_to_uncertain` calls become
  `CausalEffect::none()` and skip downstream verdict stages without an
  explicit `if let Err(_) = ... { return; }` ladder.

## Layout

```
causal_uncertain_examples/
├── gps_navigation/
│   ├── main.rs        — chain definition
│   ├── model.rs       — four stage functions
│   └── README.md
├── sensor_processing/
│   ├── main.rs        — chain definition + final-state printer
│   ├── model.rs       — six stage functions
│   ├── model_types.rs — FleetState, FleetConfig, RawReadings, FleetProcess<T>
│   ├── model_config.rs— nominal config + seed readings
│   └── README.md
└── clinical_trial/
    ├── main.rs        — chain definition
    ├── model.rs       — five stage functions, Patient / TrialCohort / LiftedCohort
    └── README.md
```
