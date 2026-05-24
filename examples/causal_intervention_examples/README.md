# Causal Intervention Examples

Nine worked examples showing what the causal monad gives you beyond `bind`.

Two senses of "intervention" live here. Both use the same
`Intervenable::intervene` operation; the difference is the loop structure
around it.

* **Counterfactual interventions** (five examples). The Judea Pearl
  tradition. Hold a factual world in mind, hypothetically substitute a
  different value, read off the difference. One-shot, retrospective,
  estimand-defining.
* **Corrective interventions** (four examples). The control-theory
  tradition. Monitor a trajectory tick by tick; when the value drifts
  outside the safe envelope, intervene to bring it back; continue from
  the corrected state. Closed-loop, real-time, failure-preventing.

This crate sits alongside
[`causal_discovery_examples`](../causal_discovery_examples) and
[`causal_uncertain_examples`](../causal_uncertain_examples).

## What `intervene` does that `bind` cannot

`Intervenable::intervene(self, new_value)` replaces the value carried by
an in-flight chain. Several properties follow from that contract; `bind`
chains can fake any of them with branching code or external state, but
the monad makes them properties of the type instead of properties of the
program.

| Property | Demonstrated by |
|---|---|
| Same chain, different worlds. Replay the same `bind` pipeline under several interventions without branching code. | All nine examples |
| Audit trail in `EffectLog`. Every `intervene` appends `!!Intervention!!: <old> replaced with <new>`. The intervention history becomes inspectable. | `counterfactual_envelope_fault`, `counterfactual_cascade_failure`, `counterfactual_resection_intervention`, all four corrective examples |
| Error-state preservation. If the chain is already in error, `intervene` is a no-op. Intervention cannot paper over an upstream bug. | (Property of the trait; not the punchline of a dedicated example.) |
| State and Context preservation. On `PropagatingProcess`, only the value swaps. Accumulated `State` and read-only `Context` are untouched. | `counterfactual_envelope_fault`, `counterfactual_cascade_failure`, all four corrective examples |
| Intervention site encodes a causal claim. Where in the chain you intervene says what counts as upstream of the manipulated quantity. | `counterfactual_treatment_options` |
| Comparative evaluation as the estimand. The difference between factual and counterfactual is the causal quantity you wanted. | `counterfactual_treatment_effect`, `counterfactual_treatment_options`, `counterfactual_cascade_failure` |
| Interventions compose into chains. A cascade is a chain of interventions, each applied to the result of the previous one. | `counterfactual_cascade_failure`, all four corrective examples |
| Closed-loop correction. The monitor decides each tick whether to intervene; the chain advances from the corrected value. Failure averted instead of measured. | `corrective_lane_keeping`, `corrective_glucose_pump`, `corrective_decompression_stops`, `corrective_network_failover` |

## Counterfactual examples

The intervention is a one-shot value substitution. The chain is evaluated
on the factual world and on one or more counterfactual worlds; the
difference is the quantity of interest.

| Example | Monad | What it shows | Command |
|---|---|---|---|
| `counterfactual_treatment_effect` | `PropagatingEffect` (stateless) | CATE as `do(T=1) − do(T=0)` on one chain. Age strata show treatment-effect heterogeneity. | `cargo run -p causal_intervention_examples --example counterfactual_treatment_effect` |
| `counterfactual_envelope_fault` | `PropagatingProcess<_, FlightState, AircraftConfig>` | A stall airspeed injected mid-chain. Same aircraft, same configuration, different value, different verdict. | `cargo run -p causal_intervention_examples --example counterfactual_envelope_fault` |
| `counterfactual_treatment_options` | `PropagatingEffect` | One patient, two intervention sites. Beta-blocker (intervene on blood pressure) versus surgical clip (intervene on wall shear stress). | `cargo run -p causal_intervention_examples --example counterfactual_treatment_options` |
| `counterfactual_cascade_failure` | `PropagatingProcess<_, NetworkState, NetworkConfig>` | A fluid network's N-k contingency analysis written as a chain of interventions. One trigger leads to total collapse; another is absorbed gracefully. | `cargo run -p causal_intervention_examples --example counterfactual_cascade_failure` |
| `counterfactual_resection_intervention` | `PropagatingEffect` | Epilepsy surgery screening as `do(connectome = resected_at_R)` for each candidate region. The hub-node resection comes out as the most curative. | `cargo run -p causal_intervention_examples --example counterfactual_resection_intervention` |

## Corrective examples

The intervention is a closed-loop control action. The chain advances one
tick at a time. A monitor inspects each result; when the value drifts
outside the safe envelope, `.intervene(corrected)` snaps it back, and
the next tick continues from the corrected state. Each example runs the
same chain twice: open loop (no intervention, catastrophic failure) and
closed loop (monitor + correction, failure averted).

| Example | Monad | What it shows | Command |
|---|---|---|---|
| `corrective_lane_keeping` | `PropagatingProcess<_, VehicleState, LaneConfig>` | Vehicle drifts laterally under a deterministic crosswind schedule. The monitor fires a P-controller correction every time the offset crosses 0.30 m. Open loop runs off the road at tick 24; closed loop stays inside the lane indefinitely. | `cargo run -p causal_intervention_examples --example corrective_lane_keeping` |
| `corrective_glucose_pump` | `PropagatingProcess<_, PatientState, PumpConfig>` | Blood glucose climbs across three meal events. The monitor triggers a corrective bolus whenever glucose crosses 180 mg/dL. Open loop crosses the ketoacidosis threshold at tick 9; closed loop stays in the safe range. | `cargo run -p causal_intervention_examples --example corrective_glucose_pump` |
| `corrective_decompression_stops` | `PropagatingProcess<_, DiveState, DiveConfig>` | A diver ascends from 30 m with one tissue compartment tracking N2 partial pressure. The monitor inserts a decompression stop whenever the Bühlmann supersaturation ratio crosses the safety threshold. Open loop crosses the DCS line at tick 8; closed loop surfaces safely with the ratio bounded. | `cargo run -p causal_intervention_examples --example corrective_decompression_stops` |
| `corrective_network_failover` | `PropagatingProcess<_, NetworkState, NetworkPlan>` | Enterprise active/standby switch pair. Primary fails on a scheduled tick; monitor detects zero delivery; `.intervene(STANDBY_SWITCH)` reroutes the next tick onward. Open loop loses 25 000 of 30 000 packets; closed loop loses 1 000 (one detection tick) and stays inside the service objective. | `cargo run -p causal_intervention_examples --example corrective_network_failover` |

## Suggested reading order

Counterfactuals first. They establish what `intervene` does as a single
operation.

1. `counterfactual_treatment_effect` is the simplest case. The estimand
   is defined as a difference of two interventional chains; that is the
   entire example.
2. `counterfactual_envelope_fault` is the same idea on a stateful
   `PropagatingProcess`. It establishes that intervention swaps the
   value while `State` and `Context` stay put.
3. `counterfactual_treatment_options` puts two interventions in the
   same chain at different sites. The lesson is that where you
   intervene encodes a claim, not just a numerical override.
4. `counterfactual_cascade_failure` is the cascade centrepiece. Each
   cascade step is an intervention applied to the result of the
   previous one; the `EffectLog` accumulates into a forensic timeline.
5. `counterfactual_resection_intervention` is the audit-trail example.
   In a clinical setting the artefact that matters is which region was
   resected, not just the simulator's number. The log records that for
   you.

Then the corrective examples, which add the closed-loop monitor.

6. `corrective_lane_keeping` is the canonical control-loop demo. Monitor
   detects drift; P-controller produces the correction; `.intervene`
   feeds it back. Open versus closed loop side by side makes the
   failure-prevention story concrete.
7. `corrective_glucose_pump` is the same shape in a medical domain.
   Asymmetric correction (insulin reduces glucose but cannot raise it)
   and irregular perturbations from meals.
8. `corrective_decompression_stops` shows the same shape with a value
   channel that carries a control command (ascent metres per tick).
   The intervention swaps the planned ascent for a stop.
9. `corrective_network_failover` shows the same shape with a discrete
   value channel (active switch id). The intervention reroutes traffic
   from primary to standby on detection of zero delivery.

## How this differs from the existing intervene examples

The codebase already has small demonstrations of `intervene` in:

* [`starter_example`](../starter_example), which walks Pearl's three
  rungs on a smoking-tar-cancer chain.
* [`classical_causality_examples/scm`](../classical_causality_examples/scm),
  which writes the same three rungs as separate files.
* [`core_examples/propagating_effect_counterfactual`](../core_examples/examples/propagating_effect_counterfactual.rs)
  and `propagating_process_counterfactual`, the smallest possible
  stateless and stateful demos.
* [`avionics_examples/geometric_tcas`](../avionics_examples/geometric_tcas),
  where pilot override is modelled as intervention.
* [`material_examples/structural_health_monitor`](../material_examples/examples/structural_health_monitor),
  which asks what would have happened without the observed crack.

Those are the *what is `intervene`* examples. The nine here are the
*what does `intervene` give you that `bind` cannot* examples, drawn from
domains that were not built around intervention from the start: clinical
trial design, avionics, hemodynamics, distribution-network reliability,
surgical planning, vehicle control, closed-loop medical pumps,
decompression theory, and enterprise networking.
