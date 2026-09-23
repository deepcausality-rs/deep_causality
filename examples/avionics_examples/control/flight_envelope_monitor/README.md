# Flight Envelope Monitor

This example monitors a flight envelope with a three-stage stateful pipeline: a
`Causaloid` collection, a `CausalMonad` `bind`-chain, and a `Causaloid`
hypergraph compose through a single `PropagatingProcess<T, FlightState, AircraftConfig>`.

## Getting Started

```bash
cargo run -p avionics_examples --example flight_envelope_monitor
```

## Why this example exists

The framework's three composition shapes, **collection** (parallel
aggregation), **bind-chain** (sequential transform), and **graph** (hypergraph
of cross-influencing effects), compose end-to-end because all three produce the
same stateful process type. The State channel carries gradual deterioration
trending.

For readers who know the stateless `PropagatingEffect<T>` (where `State =
()` and `Context = ()`), the example shows what changes when real state and
context cross stages.

## Domain background

**Flight envelope monitoring** in commercial and military aviation tracks the
aircraft's operating point against the boundaries of its certified envelope
in the joint (airspeed, altitude, angle-of-attack, load factor, weight,
configuration) space. Real avionics combine three styles of monitoring, which
this example keeps separate:

- **Sensor health** — per-sensor validity and continuous health indices.
  Voting, BITE (Built-In Test Equipment) and signal-management functions live
  here. Real-world analogues: ADC (Air Data Computer) sensor validity,
  AHRS/IRS BIT, fuel-flow transducer trends, HUMS (Health and Usage
  Monitoring) for engines and rotor systems.
- **State estimation** — fusion of validated sensor inputs into a trusted
  estimate of aircraft state via Kalman or particle filtering. Real-world
  analogues: ADIRS estimation core, GPS/INS integration.
- **Envelope protection** — boundary checks against the certified flight
  envelope, with **cross-coupling between failure modes**. Real-world
  analogues: alpha protection (stall), VMO/MMO protection (overspeed),
  EGPWS (terrain), TCAS (traffic), ice-rate-of-accretion estimators,
  CG-out-of-limits warnings on fly-by-wire aircraft.

A common production pattern computes a **continuous risk score** per
protection, fuses the per-protection scores into a single advisory level
(EICAS/ECAM caution → warning → master-warning), and applies the discrete
classification at the display layer rather than inside the reasoning core.
This example mirrors that pattern: the State channel carries cumulative
risk, the value channel carries the state estimate, and `main.rs` derives the
verdict.

> **Scope.** This is a teaching example, not certified avionics. The
> Kalman step is one-iteration scalar; envelope nodes evaluate fixed
> analytical pressures rather than calibrated probabilistic models; there
> is no redundancy management, latency budget, fault-containment
> partitioning, or DAL classification. For
> real-system patterns see DO-178C (software), DO-254 (hardware),
> ARP4754A (system development), and ARP4761A (safety assessment).

## The two channels

Every stage produces a `PropagatingProcess<T, FlightState, AircraftConfig>`,
which carries two distinct channels:

| Channel                                | Purpose           | What flows                                                                                                              |
|----------------------------------------|-------------------|-------------------------------------------------------------------------------------------------------------------------|
| **Value channel** (`T`)                | Per-stage payload | Type *changes* across stages: `SensorReading → f64 → FlightStateEstimate → FlightStateEstimate`                         |
| **State channel** (`FlightState`)      | Markovian state   | *Accumulates* across stages: covariance evolves via Kalman, risk accumulates from sensor degradation and envelope nodes |
| **Context channel** (`AircraftConfig`) | Read-only config  | Mass, MTOW, stall margin, service ceiling; fixed for one monitor cycle                                                  |

The State channel shows why the framework provides `S` separately from `T`:
the value channel cannot carry the cumulative risk because its type changes
between stages.

## Pipeline diagram

```text
┌──────────────────────────────────────────────────────────────────────┐
│ Stage 1: Causaloid collection                                        │
│   value:   SensorReading ──[5 per-sensor closures]──► f64 (joint)    │
│   state:   FlightState::default() ── threaded ──►                    │
│   context: AircraftConfig ── threaded ──►                            │
│   call:    sensors.evaluate_collection_stateful(&inc, All, _)        │
└──────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────────┐
│ Stage 2: CausalMonad bind chain (3 steps)                            │
│   value:   f64 ──► FlightStateEstimate ──► FlightStateEstimate ──►   │
│           FlightStateEstimate                                        │
│   state:   risk += (1.0 - health); covariance Kalman-updated;        │
│           estimate written                                           │
│   call:    .bind(health_fold).bind(kalman).bind(estimate)            │
└──────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────────┐
│ Stage 3: Causaloid hypergraph (6 envelope nodes)                     │
│   value:   FlightStateEstimate ── preserved end-to-end (V == V) ──►  │
│   state:   risk += per-node increments                               │
│   call:    graph.evaluate_subgraph_from_cause_stateful(0, &p)        │
└──────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
              SafetyVerdict::from_risk(final_state.risk)
              {Nominal | Caution | Warning | Failure}
```

## The per-sensor health convention

Each per-sensor causaloid returns an `f64 ∈ [0.0, 1.0]` representing that
sensor's **health probability** (`1.0` = perfectly healthy, `0.0` = fully
degraded). The collection aggregates via `AggregateLogic::All`, which the
framework's `Aggregatable for f64` impl interprets as the **product** `∏ p_i`,
the joint health probability under the independent-sensor assumption.

> **Note:** The framework's `Aggregatable for f64` interprets f64
> values as **probabilities of activation**, not as health-percent, and
> ignores the `threshold_value` argument. The example uses the
> "health-probability + `All`" convention because it reads forward: the
> variable name and the number tell the same story without a display flip.
> The mathematically equivalent "anomaly + `Any` (noisy-OR)" convention
> works too but requires inverting the value at the display layer.

As any one sensor degrades, the product drops smoothly, producing the
**continuous deterioration signal** the bind-chain folds into `state.risk`.
A discrete trip threshold (`AggregateLogic::Some(k)`) would produce a hard
1.0/0.0 with no gradient, the wrong shape for trend monitoring.

## The verdict-from-state pattern

The graph reasoning trait
`StatefulMonadicCausableGraphReasoning<V, S, C>` is implemented for
`CausaloidGraph<Causaloid<V, V, S, C>>`: the framework constrains the
graph's input value type to **equal** its output value type. The graph
operates with `V = FlightStateEstimate` end-to-end; each envelope node reads
the estimate from the value channel and `AircraftConfig` from the context,
then accumulates an envelope-specific risk increment into `state.risk`.

`main.rs` derives the final `SafetyVerdict` (`Nominal | Caution | Warning | Failure`)
from the final `state.risk` after all three stages have run. The graph does
not turn the value channel into a `SafetyVerdict`, which matches avionics
practice: a flight-envelope monitor computes a continuous risk score and
applies thresholds at the display layer.

## Design decisions worth noting

### Stage shape matches the monitoring architecture

A **Causaloid collection** rolls up parallel sensor health the way aircraft
BITE rolls up per-LRU validity. A **bind-chain** sequences the pure
transforms of state estimation (validate → fuse → estimate). A
**hypergraph** captures cross-influencing envelope protections (icing
raises stall risk; low altitude amplifies stall risk into terrain-CFIT
risk; traffic density modulates overspeed-recovery margins). Each shape
matches one monitoring layer, so a domain reader can identify each layer at a
glance.

### Trend monitoring, not discrete tripping

`AggregateLogic::Some(k)` suits flight-critical fail-flag logic; the
product under `All` suits HUMS-style trend monitoring, and the example uses
it (see the per-sensor health convention above). A production system
combines both: continuous HUMS trends *and* discrete BITE flags.

### Slow vs. fast separation: Context vs. State

`AircraftConfig` (mass, MTOW, stall-margin multiplier, service ceiling)
is read-only and supplied once per cycle in the framework's `Context`
channel, as W&B and certified-ceiling data load at dispatch and stay fixed
during flight. `FlightState` (estimate, covariance, risk) lives in the
`State` channel and accumulates across stages. Risk in `Context` or config
in `State` would type-check but blur the separation of slow and fast data.

### Error short-circuit preserves the moment-of-failure state

When a sensor closure returns an error, the bind-chain and the envelope
graph do not execute, but the State channel still carries the
`FlightState` value from the moment the failing closure ran. The
final `EffectLog` contains every entry produced *before and including*
the failing stage and nothing after. The framework's `bind` and
stateful-evaluator short-circuit guards enforce this, and the framework
crate's unit tests cover it. In the failing-sensor scenario only `[Step 1]`
prints, `state.risk` is `0.000`, and the log carries one entry.

### Per-sensor closures are stateless; State enters at the collection layer

Each per-sensor causaloid uses the stateless `Causaloid::new` form:
sensor health depends only on the per-sensor reading, not on process
state. The State and Context channels enter at the collection level, in
the incoming `PropagatingProcess` that `evaluate_collection_stateful`
receives, and thread through all subsequent stages. This layering suits
any similar monitor: per-sensor logic stays unit-testable in isolation,
and state threading belongs to the *evaluation*, not to the per-sensor
closure.

## Field-by-field

### `FlightState` (the State channel)

| Field                  | Purpose                                                                                                             |
|------------------------|---------------------------------------------------------------------------------------------------------------------|
| `estimate: [f64; 4]`   | Four-element state vector (airspeed, altitude, attitude, vertical-speed). Written by Stage 2's third bind step.     |
| `covariance: [f64; 4]` | Diagonal covariance. Initialised to `[4.0; 4]` on first Kalman update, then evolved by the Kalman step in Stage 2.  |
| `risk: f64`            | Cumulative scalar risk. Receives contributions from Stage 2's health-fold step and from each Stage 3 envelope node. |

### `AircraftConfig` (the Context channel)

| Field               | Purpose                                                                                                 |
|---------------------|---------------------------------------------------------------------------------------------------------|
| `mass_kg`           | Current aircraft mass; used by the CG-out-of-limits envelope node.                                      |
| `mtow_kg`           | Maximum takeoff weight; reference for CG margins.                                                       |
| `stall_margin`      | Stall-margin multiplier; used by the stall-risk envelope node to scale the lower airspeed band.         |
| `service_ceiling_m` | Service ceiling; used by the traffic-conflict envelope node to dampen traffic density at high altitude. |

## Run it

```bash
cargo run -p avionics_examples --example flight_envelope_monitor
```

Sample output (truncated):

```text
=== Flight Envelope Monitor — Stateful Three-Stage Pipeline ===

--- Nominal ---
  result: verdict=Nominal  (risk=0.098)
  state.estimate:   [175.0, 12500.0, 2.0, 0.0]
  state.covariance: [0.8, 0.8, 0.8, 0.8]
  state.risk:       0.098
  EffectLog:
    sensor.airspeed: health=0.938
    sensor.altitude: health=1.000
    sensor.attitude: health=1.000
    sensor.vsi: health=1.000
    sensor.fuel_flow: health=1.000
    stage2.health_fold: risk += 0.062 (health=0.938)
    stage2.kalman: covariance updated (one-iteration scalar)
    stage2.estimate: state.estimate written
    envelope.stall: risk += 0.006
    envelope.terrain: risk += 0.000
    envelope.icing: risk += 0.030
    envelope.traffic: risk += 0.000
    envelope.overspeed: risk += 0.000
    ...

--- Failing sensor ---
  result: ERROR — CausalityError(Custom("sensor.airspeed: hardware fault — sensor lost"))
  state.estimate:   [0.0, 0.0, 0.0, 0.0]
  state.covariance: [0.0, 0.0, 0.0, 0.0]far
  state.risk:       0.000
  EffectLog:
    Causaloid 0: Incoming effect: Value(SensorReading { ... })
```

Both scenarios exit with status `0`. The failing-sensor scenario prints its
error to stdout: the bind-chain and the envelope graph do **not** execute
after the airspeed sensor's closure fails, the State channel holds the
moment-of-failure state (`FlightState::default()`), and the EffectLog
contains only entries produced before and including the failing stage.

## Notes

- The Kalman step is **illustrative only**: a one-iteration scalar update on
  a diagonal covariance with fixed measurement noise. For a more developed
  filter pattern, see [`magnav`](../../navigation/magnav/README.md), which implements a
  causal particle filter for magnetic navigation.
- The six envelope nodes use **fixed analytical pressure functions** to
  derive each node's risk increment. A production implementation would
  replace these with calibrated probabilistic models, sensor-fused
  uncertainty estimates, configuration-aware tuning, and per-aircraft-type
  envelope tables.
- The example's source files import only the workspace crates
  `deep_causality` and `deep_causality_core`. The shared
  `avionics_examples` package declares more dependencies for its other
  examples.
