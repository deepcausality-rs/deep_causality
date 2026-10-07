# Sensor Data Processing Example

A stateful six-stage `PropagatingProcess<_, FleetState, FleetContext>` pipeline
triages a heterogeneous sensor fleet, fuses healthy readings, detects
anomalies, runs physics cross-checks, and emits a reliability verdict.

## Pipeline

```
PropagatingProcess { value: RawReadings, state: FleetState::default(), context: Some(fleet), ... }
    .bind(process_stage)        // Stage 1: per-sensor triage → Uncertain<FloatType> | error
    .bind(validate_stage)       // Stage 2: fold counts and total uncertainty into state
    .bind(fusion_stage)         // Stage 3: inverse-variance fuse temperature sensors
    .bind(anomaly_stage)        // Stage 4: flag readings outside nominal bands
    .bind(fallback_stage)       // Stage 5: historical fallback + temp/pressure cross-check
    .bind(reliability_stage)    // Stage 6: derive RiskLevel verdict from state
```

## Channels

| Channel  | Type             | Purpose                                                                                     |
|----------|------------------|---------------------------------------------------------------------------------------------|
| `value`  | `RawReadings` → `ProcessedReadings` | Per-sensor data carried stage-to-stage; type projects after Stage 1.    |
| `state`  | `FleetState`     | Accumulates counts, total uncertainty, fused temperature, anomaly list, final verdict.       |
| `context`| `FleetContext`   | Read-only plausibility and nominal bands, calibration offsets, triage uncertainty factors, historical temperature model, temperature–pressure correlation, anomaly and reliability thresholds; one `Data<FloatType>` contextoid per quantity, keyed by its contextoid id. |
| `logs`   | `EffectLog`      | Each stage appends one or more entries; `main.rs` prints them once at the end.               |
| `error`  | `CausalityError` | Shares the outcome `Result` with `value`; set if a stage's preconditions or body fail, after which downstream `bind` calls short-circuit. A failed stage passes on the state it received.  |

`FloatType` is the example's precision alias, `f64`, declared in `model_types.rs`; every reading,
fleet quantity and uncertain value is a `FloatType`, and literals enter it through
`deep_causality_num::lift`.

## What the example demonstrates

- **`PropagatingProcess` with non-trivial state and context:** the
  multi-stage pattern of the avionics
  [`flight_envelope_monitor`](../../avionics_examples/control/flight_envelope_monitor)
  example, applied to a sensor fleet.
- **Fleet facts in the `Context` channel:** physical-plausibility ranges,
  nominal bands, calibration offsets, triage uncertainty factors, the
  historical temperature model, the temperature–pressure correlation, and the
  anomaly and reliability thresholds live in the fleet context, so the same
  stages run against a different fleet by swapping the context.
- **Per-stage `EffectLog` observability:** stages append log entries
  instead of printing during the chain; `main.rs` prints the final state and
  log once at the end.
- **Uncertainty-aware triage:** `Healthy`, `Degraded`, `OutOfRange`,
  `CalibrationDrift`, `Failed`, and `CommunicationError` each map to a
  different `Uncertain<FloatType>` construction (or to a stage-local error
  string).
- **Inverse-variance fusion:** temperature sensors weighted by
  `1 / (σ + ε)`; large disagreement raises a fleet anomaly.
- **Cross-modal validation:** Stage 5 checks temperature against a
  pressure-derived estimate as a physics sanity test.

## How to run

```bash
cargo run -p causal_uncertain_examples --example sensor_processing
```
