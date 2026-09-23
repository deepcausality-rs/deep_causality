# Sensor Data Processing Example

A stateful six-stage `PropagatingProcess<_, FleetState, FleetConfig>` pipeline
triages a heterogeneous sensor fleet, fuses healthy readings, detects
anomalies, runs physics cross-checks, and emits a reliability verdict.

## Pipeline

```
PropagatingProcess { value: RawReadings, state: FleetState::default(), context: Some(config), ... }
    .bind(process_stage)        // Stage 1: per-sensor triage → Uncertain<f64> | error
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
| `context`| `FleetConfig`    | Read-only plausibility bands, calibration offsets, anomaly thresholds. |
| `logs`   | `EffectLog`      | Each stage appends one or more entries; `main.rs` prints them once at the end.               |
| `error`  | `CausalityError` | Shares the outcome `Result` with `value`; set if a stage's preconditions fail, after which downstream `bind` calls short-circuit.  |

## What the example demonstrates

- **`PropagatingProcess` with non-trivial state and context:** the
  multi-stage pattern of the avionics
  [`flight_envelope_monitor`](../../avionics_examples/control/flight_envelope_monitor)
  example, applied to a sensor fleet.
- **Configuration in the `Context` channel:** physical-plausibility ranges,
  calibration offsets, and anomaly thresholds live in `FleetConfig`, so the
  same stages run against a different fleet by swapping the context.
- **Per-stage `EffectLog` observability:** stages append log entries
  instead of printing during the chain; `main.rs` prints the final state and
  log once at the end.
- **Uncertainty-aware triage:** `Healthy`, `Degraded`, `OutOfRange`,
  `CalibrationDrift`, `Failed`, and `CommunicationError` each map to a
  different `Uncertain<f64>` construction (or to a stage-local error
  string).
- **Inverse-variance fusion:** temperature sensors weighted by
  `1 / (σ + ε)`; large disagreement raises a fleet anomaly.
- **Cross-modal validation:** Stage 5 checks temperature against a
  pressure-derived estimate as a physics sanity test.

## How to run

```bash
cargo run -p causal_uncertain_examples --example sensor_processing
```
