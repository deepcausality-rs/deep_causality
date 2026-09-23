# Causal Correction Examples

Five worked examples of corrective intervention with the causal monad,
showing what `alternate_value` adds to `bind`.

Each follows the control-theory pattern: monitor a trajectory tick by
tick, intervene when the value drifts outside the safe envelope, and
continue from the corrected state. The intervention is a closed-loop,
real-time control action that prevents failure.

This crate sits alongside
[`causal_counterfactual_examples`](../causal_counterfactual_examples),
[`causal_discovery_examples`](../causal_discovery_examples) and
[`causal_uncertain_examples`](../causal_uncertain_examples).

## What `alternate_value` does that `bind` cannot

`AlternatableValue::alternate_value(self, new_value)` replaces the value carried by
an in-flight chain. The properties below follow from that contract. A `bind`
chain can emulate each with branching code or external state; the monad
makes them properties of the type.

| Property | Demonstrated by |
|---|---|
| Same chain, different worlds. Replay the same `bind` pipeline under several interventions without branching code. | All five examples |
| Audit trail in `EffectLog`. Every `alternate_value` appends `!!ValueAlternation!!: <old> replaced with <new>`, so the intervention history can be inspected. | All five examples |
| Error-state preservation. If the chain is already in error, `alternate_value` is a no-op, so intervention cannot mask an upstream bug. | (Property of the trait; no example is dedicated to it.) |
| State and Context preservation. On `PropagatingProcess`, only the value swaps. Accumulated `State` and read-only `Context` are untouched. | All five examples |
| Interventions compose into chains. A cascade is a chain of interventions, each applied to the result of the previous one. | All five examples |
| Closed-loop correction. The monitor decides each tick whether to intervene; the chain advances from the corrected value, so failure is averted rather than measured. | `corrective_lane_keeping`, `corrective_glucose_pump`, `corrective_decompression_stops`, `corrective_network_failover`, `corrective_ddos_detector` |

## Examples

The chain advances one tick at a time. A monitor inspects each result;
when the value drifts outside the safe envelope, `.alternate_value(corrected)`
snaps it back, and the next tick continues from the corrected state. The
first four examples run the same chain twice: open loop (no intervention,
catastrophic failure) and closed loop (monitor plus correction, failure
averted). `corrective_ddos_detector` runs closed loop only; its subject is
the stateful sliding-window detector that drives the intervention.

| Example | Monad | What it shows | Command |
|---|---|---|---|
| `corrective_lane_keeping` | `PropagatingProcess<_, VehicleState, LaneConfig>` | Vehicle drifts laterally under a deterministic crosswind schedule. The monitor fires a P-controller correction every time the offset crosses 0.30 m. Open loop runs off the road at tick 24; closed loop stays inside the lane indefinitely. | `cargo run -p causal_correction_examples --example corrective_lane_keeping` |
| `corrective_glucose_pump` | `PropagatingProcess<_, PatientState, PumpConfig>` | Blood glucose climbs across three meal events. The monitor triggers a corrective bolus whenever glucose crosses 180 mg/dL. Open loop crosses the ketoacidosis threshold at tick 9; closed loop stays in the safe range. | `cargo run -p causal_correction_examples --example corrective_glucose_pump` |
| `corrective_decompression_stops` | `PropagatingProcess<_, DiveState, DiveConfig>` | A diver ascends from 30 m with one tissue compartment tracking N2 partial pressure. The monitor inserts a decompression stop whenever the Bühlmann supersaturation ratio crosses the safety threshold. Open loop crosses the DCS line at tick 8; closed loop surfaces safely with the ratio bounded. | `cargo run -p causal_correction_examples --example corrective_decompression_stops` |
| `corrective_network_failover` | `PropagatingProcess<_, NetworkState, NetworkPlan>` | Enterprise active/standby switch pair. Primary fails on a scheduled tick; monitor detects zero delivery; `.alternate_value(STANDBY_SWITCH)` reroutes the next tick onward. Open loop loses 25 000 of 30 000 packets; closed loop loses 1 000 (one detection tick) and stays inside the service objective. | `cargo run -p causal_correction_examples --example corrective_network_failover` |
| `corrective_ddos_detector` | `PropagatingProcess<_, DetectorState, DetectorConfig>` | Volumetric DDoS on an enterprise interface. An array-backed sliding window holds a 30 s throughput baseline carried in `State`; each new second is scored as a z-score against it. Five consecutive seconds above 3σ declare the attack and `.alternate_value(THROTTLE_ON)` engages the NIC rate-limiter, clamping throughput from ~900 Mbps back to the 420 Mbps ceiling within one tick. Overload bounded to 5 ticks, inside the service objective. | `cargo run -p causal_correction_examples --example corrective_ddos_detector` |

## Suggested reading order

1. `corrective_lane_keeping` is the basic control loop. The monitor
   detects drift, a P-controller computes the correction, and
   `.alternate_value` feeds it back. Open and closed loop run side by side.
2. `corrective_glucose_pump` has the same shape in a medical domain, with
   asymmetric correction (insulin lowers glucose but cannot raise it)
   and irregular perturbations from meals.
3. `corrective_decompression_stops` uses a value channel that carries a
   control command (ascent metres per tick). The intervention swaps the
   planned ascent for a stop.
4. `corrective_network_failover` uses a discrete value channel (active
   switch id). The intervention reroutes traffic from primary to standby
   when delivery drops to zero.
5. `corrective_ddos_detector` puts a stateful analyzer in the loop: an
   array-backed sliding window, carried in `State`, scores each new
   throughput sample against a rolling baseline. The intervention engages
   a throughput regulator that mitigates the detected attack. The loop
   drives a real-time anomaly detector rather than a setpoint controller,
   and the window is not `Clone`; the monad carries it as state because
   it does not require `State: Clone`.

For the one-shot, retrospective counterpart of these corrections, see
[`causal_counterfactual_examples`](../causal_counterfactual_examples).

## How this differs from the existing intervene examples

Smaller demonstrations of `alternate_value` live in:

* [`starter_example`](../starter_example), which walks Pearl's three
  rungs on a smoking-tar-cancer chain.
* [`classical_causality_examples/scm`](../classical_causality_examples/scm),
  which writes the same three rungs as separate files.
* [`core_examples/propagating_effect_counterfactual`](../core_examples/examples/propagating_effect_counterfactual.rs)
  and `propagating_process_counterfactual`, the smallest possible
  stateless and stateful demos.
* [`avionics_examples/geometric_tcas`](../avionics_examples/control/geometric_tcas),
  where pilot override is modelled as intervention.
* [`material_examples/structural_health_monitor`](../material_examples/structural_health_monitor),
  which asks what would have happened without the observed crack.

Those show *what `alternate_value` is*. The five here show *what
`alternate_value` adds to `bind`*, in domains not built around
intervention: vehicle control, closed-loop medical pumps, decompression
theory, enterprise networking, and network-security anomaly detection.

## Adding New Examples

1. Create directory: `<your_example>/`
2. Add `main.rs` with doc comments (`//!` module docs)
3. Add `README.md` following the [standard template](../physics_examples/README.md)
4. Register in `Cargo.toml`:
   ```toml
   [[example]]
   name = "your_example"
   path = "your_example/main.rs"
   ```
