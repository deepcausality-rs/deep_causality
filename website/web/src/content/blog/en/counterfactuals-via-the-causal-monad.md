---
title: "Counterfactuals via the Causal Monad"
description: "How DeepCausality's Causaloid/Context separation, the CausalMonad trait, and the Alternatable family deliver counterfactuals on the value, the context and the state, and closed-loop corrective control, without the abduction step that gates Pearl's SCM."
date: 2026-05-31
updated: 2026-10-13
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

DeepCausality is the reference implementation of the **Effect Propagation Process (EPP)**, a single axiomatic foundation for dynamic causality. Its axiom is `m₂ = m₁ >>= f`: effect propagation as monadic dependency, with no background spacetime assumed.

Classical causal frameworks (Pearl's SCM, Granger causality, Dynamic Bayesian Networks) couple causal law and world state into one structure with implicit exogenous noise. That coupling is why Pearl's standard counterfactual algorithm needs three steps: *abduction*, *action*, *prediction*. The abduction step infers the posterior over the unobserved noise from factual evidence, and it is the well-known bottleneck. Recent neural-SCM work (Pawlowski, Castro, Glocker, [NeurIPS 2020](https://proceedings.neurips.cc/paper/2020/file/0987b8b338d6c90bbedd8631bc499221-Paper.pdf)) exists in large part because abduction is intractable for general deep SCMs.

The EPP makes a different foundational move: it separates the causal law (a **Causaloid**) from the world state (a **Context**), and threads effects through one of two carrier types. The stateless `PropagatingEffect<T>` carries a value or an error, and an audit log; it is used for Markov-free chains. The stateful `PropagatingProcess<T, S, C>` adds a `State` channel and a typed `Context` channel for Markovian chains. Both are aliases of one underlying struct (`CausalEffectPropagationProcess<...>`), and both implement the same `CausalMonad` and `Alternatable` traits. Once world state is a first-class queryable value rather than implicit noise, three things follow that do not follow under SCM:

1. **Counterfactuals need no inference step for an observed world.** The "world as it actually was" is the factual Context itself: a value, not a posterior. A counterfactual is *clone the Context, modify a Contextoid, re-evaluate the same Causaloid*. The Bareinboim/Correa/Ibeling/Icard Causal Hierarchy Theorem ([R-60, 2020](https://causalai.net/r60.pdf)) still applies; counterfactuals contain strictly more information than interventions. In the EPP that extra information lives in the explicit Context.
2. **The substrate of intervention is unrestricted.** SCM's `do(X=x)` acts only on endogenous variables; abduction holds `U` fixed. Contextual Alternation can substitute any Contextoid (`Datoid`, `Spaceoid`, `Tempoid`, `SpaceTempoid`), so counterfactuals over data, time and space are first-class. The DBN example in [`classical_via_causaloid/dbn`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples/classical_via_causaloid/dbn) demonstrates this: the day counter lives in the Context, so a counterfactual over time is an alternation of a contextoid.
3. **Corrective control becomes a one-method extension of evaluation.** Because `PropagatingProcess` threads `state` and `context` through every `bind`, a monitor running inside the chain can detect an anomaly using the model's own state and context, compute a correction from the same model, and apply `.alternate_value(corrected)` mid-flight. The next `bind` continues from the corrected value.

What follows is the technical tour. Three foundational pieces first: **CausalMonad** as a trait, the **Alternatable** family for value, context, and state, and the **two carrier types** (`PropagatingEffect` for stateless chains, `PropagatingProcess` for stateful ones). Then Pearl's Ladder in a dozen lines. Then three illustrations: SCM on the monad, DBN with a mid-stream regime change, and one model with a counterfactual on each channel. Then the closed-loop corrective examples that take counterfactual reasoning one step further.

## The Causal Monad Trait

In an earlier iteration of DeepCausality the Causal Monad shipped as a concrete struct (`CausalMonad<S, C>`) that sat beside the Causaloid as a co-equal primitive. That design had two problems. First, it forced the user to talk about the monad as if it were a thing you instantiate; the everyday code was littered with `CausalMonad::<i32, String>::bind(...)`. Second, the value-only `bind: FnMut(T) -> M<U>` froze any Markovian state the chain wanted to thread, so the stateful and stateless worlds needed two separate `bind` implementations.

The current design eliminates both problems. `CausalMonad` is now a trait that both `PropagatingEffect` and `PropagatingProcess` implement (through their shared underlying struct):

```rust
pub trait CausalMonad: Sized {
    type Value;
    type State;
    type Context;

    fn pure(value: Self::Value) -> Self;

    fn bind<NewValue, F>(self, f: F)
        -> CausalEffectPropagationProcess<NewValue, Self::State, Self::Context, CausalityError, EffectLog>
    where
        F: FnOnce(
            CausalEffect<Self::Value>,
            Self::State,
            Option<Self::Context>,
        ) -> CausalEffectPropagationProcess<NewValue, Self::State, Self::Context, CausalityError, EffectLog>;
}
```

One `bind` exists, and it threads state. The trait lets generic code compose against the contract; everyday code calls the inherent `effect.bind(...)` and `PropagatingEffect::pure(x)` (or the `PropagatingProcess` equivalents), or writes the chain with the fluent `CausalFlow` builder over the same carrier, and never imports the trait. `PropagatingEffect` and `PropagatingProcess` *are* the monad. The monad is not a third primitive.

Two operations carry the whole algebra. `pure` lifts a plain value into a `PropagatingEffect` (or, with `with_state`, into a `PropagatingProcess`). `bind` chains the next step, threading value, state, context, error short-circuit, and log accumulation through one signature. The three monad laws (left identity, right identity, associativity) hold, are tested, and are machine-checked in Lean.

For more details about the causal monad, see the [Causal Monad](https://docs.deepcausality.com/concepts/causal-monad/) concept page.

## The Alternatable family: three channels, three traits

The underlying struct (`CausalEffectPropagationProcess`) that `PropagatingEffect` and `PropagatingProcess` alias has four fields: `outcome`, `state`, `context`, `logs`. The `outcome` holds either a value or the error that ended the chain, never both. Three channels are substitutable mid-chain: the **value** on the success side of the outcome, the **context**, and the **state**. The error and the log are not: substituting an error would be a safety violation, and the log is append-only by design. On a stateless `PropagatingEffect` only the value carries real information, so value alternation is where the work happens; on a stateful `PropagatingProcess` all three are meaningful.

DeepCausality exposes one single-method trait per channel:

```rust
pub trait AlternatableValue<V>   { fn alternate_value(self,   new_value: V) -> Self; }
pub trait AlternatableContext<C> { fn alternate_context(self, new_context: C) -> Self; }
pub trait AlternatableState<S>   { fn alternate_state(self,   new_state: S) -> Self; }
```

Plus a marker super-trait, auto-implemented via blanket impl, for code that needs all three:

```rust
pub trait Alternatable<V, C, S>:
    AlternatableValue<V> + AlternatableContext<C> + AlternatableState<S> {}

impl<T, V, C, S> Alternatable<V, C, S> for T
where T: AlternatableValue<V> + AlternatableContext<C> + AlternatableState<S> {}
```

The fluent `CausalFlow` builder carries the same three methods and implements the same traits, so a `CausalFlow` satisfies the `Alternatable` bound as well.

Every method in the family follows the same contract:

- **Error short-circuit.** If the upstream chain has already errored, the call is a no-op. An alternation cannot fix a broken chain.
- **Channel isolation.** Only the named channel is rewritten; the other two alternable channels and the audit log continue unchanged.
- **Automatic audit entry.** A distinctive marker is appended to the log: `!!ValueAlternation!!: <old> replaced with <new>`, `!!ContextAlternation!!: context replaced`, `!!StateAlternation!!: state replaced`.

Earlier releases called value substitution `intervene`, through an `Intervenable` trait. Each method now names the channel it replaces, so the value form is `alternate_value`. The name `do`/`intervene` is reserved for the graph layer, where the full operator (variable isolation, graph surgery) belongs; on a single chain, substituting the carried value is its value-level form.

### Why this goes beyond abduction

Pearl's standard counterfactual procedure (abduction → action → prediction) swaps the value of endogenous variables, with the exogenous noise held fixed by the abduced posterior. Everything a counterfactual can change has to be a variable of the model, and the inference step that pins down "the actual world" is the bottleneck.

The Alternatable family substitutes on three independent channels, and where the chain carries the observed context and state, it needs no inference step:

- **Value alternation** (`alternate_value`): replaces one result, the value-level form of Pearl's `do(...)`, as one method call.
- **Context alternation** (`alternate_context`): swap the entire world (or any structured piece of it) without rebuilding the chain. This is what makes RCM potential outcomes, climate-regime changes, and patient-cohort substitutions a single line.
- **State alternation** (`alternate_state`): replace the running Markov state, the history the chain has accumulated. The next steps run the same law from a different history, which can put them in a different part of that law.

The three operators compose freely. On a `PropagatingProcess` or a `CausalFlow` you can alternate the value, the context, and the state in any order within the same chain; each emits its own audit entry, and downstream `bind` steps see the substituted channels.

For the long-form treatment see the [Counterfactuals](https://docs.deepcausality.com/concepts/counterfactuals/) concept page and the [Causal Monad](https://docs.deepcausality.com/concepts/causal-monad/) concept page.

## The two carrier types: PropagatingEffect and PropagatingProcess

Both aliases share one underlying struct:

```rust
pub struct CausalEffectPropagationProcess<Value, State, Context, Error, Log> {
    outcome: Result<CausalEffect<Value>, Error>,
    state:   State,
    context: Option<Context>,
    logs:    Log,
}
```

The fields are private; constructors build a process and getters read it. Two pinned aliases cover almost all common use cases:

```rust
pub type PropagatingEffect<T> =
    CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>;

pub type PropagatingProcess<T, S, C> =
    CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>;
```

`PropagatingEffect<T>` pins state and context to `()`; the chain stays Markov-free. Use it for stateless pipelines: pure transforms, single-shot evaluations, Pearl-style counterfactual examples that do not need to carry world parameters across steps.

`PropagatingProcess<T, S, C>` keeps state and context generic; the chain threads real state and a typed context. Use it for Markovian pipelines: temporal simulations, closed-loop control, anything where one step's behavior depends on what earlier steps did.

Lifting from the stateless to the stateful form is a single constructor call: `PropagatingProcess::with_state(some_effect, initial_state, Some(initial_context))`. The four fields together are the irreducible structure for verifiable end-to-end reasoning. Drop the state field and Markovian cases need a separate type; drop the log field and audit becomes an external concern; split value and error into two fields and a process can hold both at once.

For the long-form treatment see the [Effect Propagation Process](https://docs.deepcausality.com/concepts/effect-propagation-process/) concept page.

## Pearl's Ladder in a dozen lines

The three rungs of Pearl's Ladder of Causation map cleanly onto a stateless chain (no state, no context needed for this example). `CausalFlow` writes it most compactly: a fluent builder over `PropagatingEffect` whose steps are plain functions on the value.

| Rung | Question | Operator | `CausalFlow` expression |
|---|---|---|---|
| 1. Association | "If I see X, what do I expect about Y?" | `P(Y \| X)` | `CausalFlow::value(x).map(f)` |
| 2. Intervention | "If I *do* X, what happens to Y?" | `P(Y \| do(X))` | `CausalFlow::value(x).map(f).alternate_value(new)` |
| 3. Counterfactual | "Given the world as it is, what would have happened if X had been different?" | `P(Y_x \| X', Y')` | the same chain run twice, factually and with `alternate_value`, then compared |

Rung one is a chain of steps. Rung two adds `alternate_value`. Rung three runs rung two against a held factual reference and compares the two outcomes.

```rust
use deep_causality_core::{CausalFlow, CausalityError};

fn main() -> Result<(), CausalityError> {
    // Causal chain: Dose -> Absorption -> Metabolism -> Response.
    let absorption = |dose: f64| dose * 0.8;
    let metabolism = |level: f64| level - 2.0;

    // Rung 1: Association. Run the chain factually: 10.0 -> 8.0 -> 6.0.
    let observed = CausalFlow::value(10.0).map(absorption).map(metabolism);

    // Rung 2: Intervention. do(BloodLevel := 3.0) after absorption: 10.0 -> 8.0 => 3.0 -> 1.0.
    let intervened = CausalFlow::value(10.0)
        .map(absorption)
        .alternate_value(3.0)
        .map(metabolism);

    // Rung 3: Counterfactual. The causal effect is the difference between the
    // intervened and the observed outcome: ITE = Y(do(X)) - Y(X_observed).
    let y_obs = observed.finish()?;
    let y_int = intervened.finish()?;

    println!("Observed Y      = {y_obs:.2}");
    println!("Intervened Y    = {y_int:.2}");
    // -5.00: the intervention lowered the response by five units.
    println!("Causal effect Δ = {:+.2}", y_int - y_obs);
    Ok(())
}
```

The two runs share their structure and their composition law. The only difference is the `.alternate_value(3.0)` call. The causal-effect estimate is the *difference* `Y(do(X)) − Y(X_observed)`; here it is `1.00 − 6.00 = −5.00`, meaning the intervention lowered the response by five units. The log of `intervened` records the substitution, `!!ValueAlternation!!: Value(8.0) replaced with Value(3.0)`; the run stays replayable and auditable. `finish` returns the value, or the error that stopped the chain.

## The classical methods, reimplemented

The [`classical_causality_examples`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples) crate ships the five textbook methods of computational causality (CATE, DBN, Granger, RCM, SCM), each implemented **twice**: once with `Causaloid` + Contextual Alternation, once with `PropagatingProcess` + the `Alternatable` family. They differ only in *where* the alternation lives. The [crate README](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples) carries a full side-by-side comparison.

For the scope of this post, we focus on the **monad approach**. Three illustrations together exercise the three alternation channels.

### Illustration 1: SCM (Pearl's Ladder) on a chain, exercising value and context alternation

[`scm_via_monad`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples/classical_via_causal_monad/scm) puts the smoking-tar-cancer chain on `PropagatingProcess<FloatType, (), SmokingContext>`. Each rung uses one operator:

| Rung | Operator | Channel |
|---|---|---|
| 1. Association | plain `bind` chain | none |
| 2. Intervention | `.alternate_value(0.0)` mid-chain, applying `do(Tar := 0.0)` | value |
| 3. Counterfactual | `.alternate_context(other_world)` at the seed | context |

The intervention severs the Smoking → Tar link before stage 2 reads it, so cancer risk drops to false even though the patient still has high nicotine. The counterfactual swaps the world to one where nicotine is low but tar already accumulated; cancer risk stays high because the tar is still in the lungs. Same chain in both cases, two distinct alternation operators, two distinct audit-log markers (`!!ValueAlternation!!`, `!!ContextAlternation!!`).

### Illustration 2: DBN with a mid-stream regime change, context alternation plus state threading

[`dbn_via_monad`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples/classical_via_causal_monad/dbn) runs the Umbrella World over ten days on `PropagatingProcess<FloatType, WeatherState, ClimateContext>`. The `ClimateContext` holds the climate's conditional probability table and the rain probability above which the person carries an umbrella, as three `Datoid` contextoids; the `WeatherState` carries the running Markov state (`rained_yesterday`, day counter, rainy-day and umbrella counters).

Two ten-day simulations run side by side:

1. **Baseline all the way.** Dry-leaning climate for ten days. `umbrellas_carried = 0`.
2. **Regime change mid-stream.** Baseline for five days, then `process.alternate_context(monsoon)` switches the probability table, and the remaining five days run under the alternated context. `umbrellas_carried = 5`.

The Markov state threads through the alternation untouched: the day counter continues `1..10` across the switch, and the umbrella counter accumulates correctly. The `EffectLog` records exactly one `!!ContextAlternation!!` entry pinpointing the switch.

### Illustration 3: one model, a counterfactual on each channel

[`counterfactual_fast_charge`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_counterfactual_examples/counterfactual_fast_charge) charges a lithium-ion cell for 30 minutes on a `CausalFlow<FloatType, FloatType, CoolingContext>`. Each minute is one step: the cell temperature is the value, the charge is the state, and the coolant is the context. The charger halves its current above 45 °C, stops above 55 °C, and above 80 % charge holds the cell voltage so the current tapers to zero at 100 %. The run is observed for ten minutes and then branches three ways:

```rust
// The first ten minutes, as observed. Every run continues from this flow for the rest.
let at_branch = plug_in(full_flow_cooling()?)?.iterate_n(BRANCH_MINUTE, one_minute);
let finish = |run: Charge| {
    run.iterate_n(RUN_MINUTES - BRANCH_MINUTE, one_minute)
        .into_process()
};

let observed = finish(at_branch.clone());
// What if the cell had been at 52 °C after ten minutes?
let hot_cell = finish(at_branch.clone().alternate_value(HOT_CELL_C));
// What if the coolant pump had dropped to a quarter of its flow?
let weak_pump = finish(at_branch.clone().alternate_context(quarter_flow_cooling()?));
// What if the cell had held 75 % charge after ten minutes?
let full_cell = finish(at_branch.alternate_state(HIGH_CHARGE_PCT));
```

| Run | Replaced at minute 10 | Cell after 30 min | Charge | Next minute's current | Log marker |
|---|---|---|---|---|---|
| Observed | nothing | 38.8 °C | 70.0 % | 50.0 A | none |
| Hot cell | value: 33.5 °C → 52 °C | 41.2 °C | 65.8 % | 50.0 A | `!!ValueAlternation!!` |
| Weak pump | context: coolant at a quarter of its flow | 45.5 °C | 65.0 % | 25.0 A | `!!ContextAlternation!!` |
| Full cell | state: 36.7 % → 75 % charge | 30.3 °C | 95.4 % | 11.4 A | `!!StateAlternation!!` |

The hot cell runs at half current until it cools below 45 °C and ends 4.2 percentage points behind. The weak pump changes the world: the cell warms past 45 °C at minute 23, and the charger then switches between half and full current. The full cell changes the history: it reaches 80 % at minute 13, the charger moves to its taper, and the cell cools while it fills. The same `one_minute` step runs in all four runs.

Between these three illustrations, the **value**, **context**, and **state** channels are all exercised, each by its own operator, each producing its own audit-log marker. The remaining three classical methods (RCM, CATE, Granger) follow the same patterns on a smaller scale; their [`via_monad`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples/classical_via_causal_monad) implementations live alongside. Read the [comparison README](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples) for more details.

## Closed-loop corrective control: the part classical inference cannot reach

Five corrective examples push the EPP beyond what SCM, Granger, or DBN can do. `PropagatingProcess` threads `state` and `context` through every `bind`, so a monitor running inside the chain has direct access to the model's own current understanding of the world. The same model that drove the forward simulation drives the correction: the threshold lives in the `Context`, the counters accumulate in the `State`, and the corrective value is computed from the same context that produced the anomalous value. Then `.alternate_value(corrected)` rewrites the value of the process, and the next `bind` advances from the corrected value. The loop is fully internal to the causal model.

This is not "an inference framework with a controller bolted on." It is one `PropagatingProcess` doing both jobs because the EPP makes state, context, and value first-class on the same struct. Four of the examples run the same chain twice: open loop (failure) and closed loop (failure averted); the difference between the two is the monitor shown below. The fifth runs the closed loop alone.

| Example | Carrier | What it shows |
|---|---|---|
| `corrective_lane_keeping` | `PropagatingProcess<_, VehicleState, LaneContext>` | Vehicle drifts under a crosswind. Monitor fires a P-controller correction whenever the offset crosses 0.30 m. Open loop runs off the road at tick 24; closed loop stays in lane for all 60 ticks. |
| `corrective_glucose_pump` | `PropagatingProcess<_, PatientState, PumpContext>` | Blood glucose climbs across three meals. Monitor triggers a corrective bolus above 180 mg/dL. Open loop crosses the ketoacidosis line at tick 9; closed loop stays in the safe range. |
| `corrective_decompression_stops` | `PropagatingProcess<_, DiveState, DiveContext>` | A diver ascends from 30 m. Monitor inserts a decompression stop whenever the Bühlmann ratio crosses the safety threshold. Open loop crosses the DCS line at tick 8; closed loop surfaces safely. |
| `corrective_network_failover` | `PropagatingProcess<_, NetworkState, NetworkContext>` | Active/standby switch pair. Primary fails on a scheduled tick; monitor detects zero delivery and replaces the active switch with the standby. Open loop loses 25 000 of 30 000 packets; closed loop loses 1 000 and stays inside the service objective. |
| `corrective_ddos_detector` | `PropagatingProcess<_, DetectorState, DetectorContext>` | A volumetric surge hits a link. Monitor scores each second against a sliding-window baseline and engages a throttle after consecutive anomalies. The surge peaks at 901 Mbps; the throttle engages at tick 46 and holds the link at 420 Mbps. |

The lane-keeping example is the most direct read of what `alternate_value` is doing in this mode. The open-loop main loop is the canonical bind chain:

```rust
fn run_open_loop(lane: LaneContext) -> LaneProcess<FloatType> {
    CausalFlow::from(model::initial_process(lane))
        .iterate_n(N_TICKS as usize, |tick| tick.bind(model::simulate_step))
        .into_process()
}
```

The closed-loop version adds a monitor as a `branch_with`:

```rust
fn run_closed_loop(lane: LaneContext) -> LaneProcess<FloatType> {
    CausalFlow::from(model::initial_process(lane))
        .iterate_n(N_TICKS as usize, |tick| {
            tick.bind(model::simulate_step).branch_with(
                |offset, _state, ctx| {
                    let lane = ctx.expect("simulate_step returns the lane context it read");
                    let threshold = read(lane, ANOMALY_THRESHOLD)
                        .expect("simulate_step read the anomaly threshold from this context");
                    offset.abs() > threshold
                },
                |hot| {
                    hot.update_state(|mut state, _offset| {
                        state.correction_count += 1;
                        state
                    })
                    .bind(model::correct)
                },
                |cold| cold,
            )
        })
        .into_process()
}
```

Same chain, same simulation step, same drift schedule. The only difference is the `branch_with`: a predicate that reads the anomaly threshold from the context, and a corrective arm that counts the correction and binds `model::correct`. That step reads the controller gain from the same context and ends in `.alternate_value(corrected)`. The `EffectLog` records every correction with the offset it replaced and the corrected value, so the run is reviewable after the fact.

Read the five corrective examples together and a pattern emerges. The monitor is small and identical in shape across them:

```rust
tick.bind(model::step).branch_with(
    |value, state, ctx| anomaly(value, state, ctx), // threshold read from the context
    |hot| hot.update_state(count_correction).alternate_value(corrected),
    |cold| cold,
)
```

Same shape in [`corrective_lane_keeping`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_correction_examples/corrective_lane_keeping), [`corrective_glucose_pump`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_correction_examples/corrective_glucose_pump), [`corrective_decompression_stops`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_correction_examples/corrective_decompression_stops), [`corrective_network_failover`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_correction_examples/corrective_network_failover), and [`corrective_ddos_detector`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_correction_examples/corrective_ddos_detector). The anomaly predicate, the correction, and the units of the state counter change with the domain (lateral offset in meters, blood glucose in mg/dL, Bühlmann supersaturation ratio, packet delivery counts, throughput in Mbps), but the architecture is one `branch_with` operating on the same carrier. Where the correction needs the context, as in lane keeping, the corrective arm binds a step that reads it and then calls `alternate_value`.

Two consequences are worth naming explicitly:

**The anomaly detector and the corrector are inside the causal model.** The context the predicate reads is the same `Context` the forward steps read. The value it inspects is the same value the next `bind` would otherwise consume. The corrected value comes from a function defined in the same `model` module as the forward dynamics. There is no separate controller process running alongside the model with its own duplicated world view; the controller *is* the model, reading its own state.

**The correction does not break the audit log.** The `EffectLog` records every monitor tick, every detection, and every `!!ValueAlternation!!: <old> replaced with <new>` event. A post-incident review reads the same log a debugger would. A regulator audit reads the same log a developer would. The corrective autonomy is observable by construction.

## What this means

The design choices add up to a small set of concrete properties:

* **Three independent alternation channels.** Value, context, and state are each substitutable through their own trait and method, on the same `PropagatingProcess` or `CausalFlow`, composable in any order. (On a stateless `PropagatingEffect` only value alternation does observable work; the context and state methods accept the unit value `()` but only emit a log entry.) The error and the log stay non-alternable by design: overwriting an error is a safety violation, and the log is append-only.
* **Same chain, different worlds.** Replay the same `bind` pipeline under several alternations without branching code.
* **Audit trail on rails.** Every alternation appends a distinctive marker (`!!ValueAlternation!!`, `!!ContextAlternation!!`, `!!StateAlternation!!`); the substitution history is inspectable in the same `EffectLog` as the rest of the run.
* **Error-state preservation.** If the chain is already in error, every alternation method is a no-op. None of them can fix a broken upstream.
* **Channel isolation.** A `.alternate_value(...)` rewrites only the value; the context and state continue through the chain untouched. The two other operators behave symmetrically.
* **Intervention site encodes a causal claim.** Where in the chain you alternate says what counts as upstream of the manipulated quantity.
* **Comparative evaluation as the estimand.** The difference between factual and counterfactual *is* the causal quantity you wanted; the SCM rung-3 example writes this definition out as code.
* **Composable closed-loop control.** A monitor decides each tick whether to alternate the value; the chain advances from the corrected value. Failure averted instead of measured.

None of these properties is new in isolation. What is new is that they fall out of one foundational move (separating Causaloid from Context, then threading effects through `PropagatingEffect` or `PropagatingProcess`) and compose under one trait family. Counterfactuals without abduction for an observed world, mid-chain substitution without graph mutilation, three-channel substitution without a separate operator language, closed-loop corrective control without a separate controller process: all of them use the same `bind` plus `Alternatable` primitives on the same carrier types. The rest of the framework (Causaloids, Context, Effect Ethos) composes against those types without glue code.

## Getting started

* Read the [Counterfactuals](https://docs.deepcausality.com/concepts/counterfactuals/) concept page for the canonical write-up.
* Read the [Causal Monad](https://docs.deepcausality.com/concepts/causal-monad/) and [Effect Propagation Process](https://docs.deepcausality.com/concepts/effect-propagation-process/) pages for the supporting algebra.
* Run the three illustrations from this post:
  * `git clone https://github.com/deepcausality-rs/deep_causality.git && cd deep_causality`
  * `cargo run -p classical_causality_examples --example scm_via_monad`
  * `cargo run -p classical_causality_examples --example dbn_via_monad`
  * `cargo run -p causal_counterfactual_examples --example counterfactual_fast_charge`
* Read the [classical-examples README](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples) for the full side-by-side comparison and the practitioner decision guide.
* Run any of the closed-loop corrective examples: `cargo run -p causal_correction_examples --example corrective_lane_keeping` (pick any of the five).
* Join the [community](https://www.deepcausality.com/community/) or the [Discord server](https://discord.gg/Bxj9P7JXSj).

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and deterministic context-aware causal reasoning in Rust. Please give us a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open source innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all members of the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).
