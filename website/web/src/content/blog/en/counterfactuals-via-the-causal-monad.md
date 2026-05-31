---
title: "Counterfactuals via the Causal Monad"
description: "How DeepCausality's Causaloid/Context separation, the CausalMonad trait, and the Intervenable operator deliver counterfactuals and closed-loop corrective control without the abduction step that gates Pearl's SCM."
date: 2026-05-31
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

DeepCausality is the reference implementation of the **Effect Propagation Process (EPP)**, a single axiomatic foundation for dynamic causality. Its axiom is `m₂ = m₁ >>= f`: effect propagation as monadic dependency, with no background spacetime assumed.

Classical causal frameworks (Pearl's SCM, Granger causality, Dynamic Bayesian Networks) couple causal law and world state into one structure with implicit exogenous noise. That coupling is why Pearl's standard counterfactual algorithm needs three steps: *abduction*, *action*, *prediction*. The abduction step infers the posterior over the unobserved noise from factual evidence, and it is the well-known bottleneck. Recent neural-SCM work (Pawlowski, Castro, Glocker, [NeurIPS 2020](https://proceedings.neurips.cc/paper/2020/file/0987b8b338d6c90bbedd8631bc499221-Paper.pdf)) exists in large part because abduction is intractable for general deep SCMs.

The EPP makes a different foundational move: it separates the causal law (a **Causaloid**) from the world state (a **Context**), and threads effects through one of two carrier types. The stateless `PropagatingEffect<T>` carries a value, an error channel, and an audit log, and is used for Markov-free chains. The stateful `PropagatingProcess<T, S, C>` adds a `State` channel and a typed `Context` channel for Markovian chains. Both are aliases of one underlying struct (`CausalEffectPropagationProcess<...>`), and both implement the same `CausalMonad` and `Alternatable` traits. Once world state is a first-class queryable value rather than implicit noise, three things follow that do not follow under SCM:

1. **Counterfactuals need no inference step.** The "world as it actually was" is the Factual Context itself: a value, not a posterior. A counterfactual is *clone the Context, modify a Contextoid, re-evaluate the same Causaloid*. The Bareinboim/Correa/Ibeling/Icard Causal Hierarchy Theorem ([R-60, 2020](https://causalai.net/r60.pdf)) still applies; counterfactuals contain strictly more information than interventions. In the EPP that extra information lives in the explicit Context.
2. **The substrate of intervention is unrestricted.** SCM's `do(X=x)` acts only on endogenous variables; abduction holds `U` fixed. Contextual Alternation can substitute any Contextoid (`Datoid`, `Spaceoid`, `Tempoid`, `SpaceTempoid`, `Symboid`), so counterfactuals over time, space, and symbolic identifiers are first-class. The DBN example in [`classical_causality_examples/dbn`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples/dbn) demonstrates exactly this: time lives in the Context, so temporal counterfactuals are alternations of temporal Contextoids.
3. **Corrective control becomes a one-method extension of evaluation.** Because `PropagatingProcess` threads `state` and `context` through every `bind`, a monitor running inside the chain can detect an anomaly using the model's own state, compute a correction from the same model, and apply `.intervene(corrected)` mid-flight. The next `bind` continues from the corrected value.

What follows is the technical tour. Three foundational pieces first: **CausalMonad** as a trait, the **Alternatable** family for value, context, and state, and the **two carrier types** (`PropagatingEffect` for stateless chains, `PropagatingProcess` for stateful ones). Then Pearl's Ladder in a dozen lines. Then two illustrations from the classical-methods example crate (SCM on the monad, DBN with a mid-stream regime change). Then the closed-loop corrective example that takes counterfactual reasoning one step further.

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
            EffectValue<Self::Value>,
            Self::State,
            Option<Self::Context>,
        ) -> CausalEffectPropagationProcess<NewValue, Self::State, Self::Context, CausalityError, EffectLog>;
}
```

One `bind` exists, and it threads state. The trait lets generic code compose against the contract; everyday code calls the inherent `effect.bind(...)` and `PropagatingEffect::pure(x)` (or the `PropagatingProcess` equivalents) directly and never imports the trait. `PropagatingEffect` and `PropagatingProcess` *are* the monad. The monad is not a third primitive.

Two operations carry the whole algebra. `pure` lifts a plain value into a `PropagatingEffect` (or, with `with_state`, into a `PropagatingProcess`). `bind` chains the next step, threading value, state, context, error short-circuit, and log accumulation through one signature. The three monad laws (left identity, right identity, associativity) are satisfied and tested.

For more details about the causal monad, see the [Causal Monad](https://docs.deepcausality.com/concepts/causal-monad/) concept page.

## The Alternatable family: three channels, three traits

The underlying struct (`CausalEffectPropagationProcess`) that `PropagatingEffect` and `PropagatingProcess` alias has five fields: `value`, `state`, `context`, `error`, `logs`. Three of these are legitimately substitutable mid-chain; two are not. Substituting the `error` channel would be a safety violation, and the `logs` channel is append-only by design. That leaves three alternable channels: **value, context, and state**. On a stateless `PropagatingEffect` only the `value` channel carries real information, so value alternation is where the work happens; on a stateful `PropagatingProcess` all three are meaningful.

DeepCausality exposes one single-method trait per channel:

```rust
pub trait AlternatableValue<V>   { fn alternate_value(self,   new: V) -> Self; }
pub trait AlternatableContext<C> { fn alternate_context(self, new: C) -> Self; }
pub trait AlternatableState<S>   { fn alternate_state(self,   new: S) -> Self; }
```

Plus a marker super-trait, auto-implemented via blanket impl, for code that needs all three:

```rust
pub trait Alternatable<V, C, S>:
    AlternatableValue<V> + AlternatableContext<C> + AlternatableState<S> {}

impl<T, V, C, S> Alternatable<V, C, S> for T
where T: AlternatableValue<V> + AlternatableContext<C> + AlternatableState<S> {}
```

Every method in the family follows the same contract:

- **Error short-circuit.** If the upstream chain has already errored, the call is a no-op. An alternation cannot fix a broken chain.
- **Channel isolation.** Only the named channel is rewritten; the other two alternable channels and the audit log continue unchanged.
- **Automatic audit entry.** A distinctive marker is appended to the log: `!!ValueAlternation!!: <old> replaced with <new>`, `!!ContextAlternation!!: context replaced`, `!!StateAlternation!!: state replaced`.

The familiar `Intervenable<V>` trait survives as a thin vocabulary alias atop `AlternatableValue<V>`, so existing code that calls `effect.intervene(x)` keeps working:

```rust
pub trait Intervenable<V>: AlternatableValue<V> {
    fn intervene(self, new_value: V) -> Self where Self: Sized {
        self.alternate_value(new_value)
    }
}
impl<T, V> Intervenable<V> for T where T: AlternatableValue<V> {}
```

`intervene` is the causal-inference word for value substitution (Pearl's `do(...)`). `alternate_value` is the same operation under the EPP's substitution vocabulary. Same method body, two surfaces.

### Why this goes beyond abduction

Pearl's standard counterfactual procedure (abduction → action → prediction) only swaps **one** thing: the value of a single endogenous variable, with the exogenous noise held fixed by the abduced posterior. The substitution surface is narrow, and the inference step that pins down "the actual world" is the bottleneck.

The Alternatable family widens the substitution surface to three independent channels and removes the inference step:

- **Value alternation** (`alternate_value` / `intervene`): Pearl's `do(...)` operator, expressed as one method call.
- **Context alternation** (`alternate_context`): swap the entire world (or any structured piece of it) without rebuilding the chain. This is what makes RCM potential outcomes, climate-regime changes, and patient-cohort substitutions a single line.
- **State alternation** (`alternate_state`): force the running Markov state to a new value. Useful for simulator resets, regime changes that touch accumulated counters, and test fixtures.

The three operators compose freely. On a `PropagatingProcess` you can intervene on the value, alternate the context, and reset the state in any order within the same chain; each emits its own audit entry, and downstream `bind` steps see the substituted channels. Abduction never enters, because `PropagatingProcess` already carries the world state explicitly.

For the long-form treatment see the [Counterfactuals](https://docs.deepcausality.com/concepts/counterfactuals/) concept page and the [Causal Monad](https://docs.deepcausality.com/concepts/causal-monad/) concept page.

## The two carrier types: PropagatingEffect and PropagatingProcess

Both aliases share one underlying struct:

```rust
pub struct CausalEffectPropagationProcess<Value, State, Context, Error, Log> {
    pub value:   EffectValue<Value>,
    pub state:   State,
    pub context: Option<Context>,
    pub error:   Option<Error>,
    pub logs:    Log,
}
```

Two pinned aliases cover almost all common use cases:

```rust
pub type PropagatingEffect<T> =
    CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>;

pub type PropagatingProcess<T, S, C> =
    CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>;
```

`PropagatingEffect<T>` pins state and context to `()`; the chain stays Markov-free. Use it for stateless pipelines: pure transforms, single-shot evaluations, Pearl-style counterfactual examples that do not need to carry world parameters across steps.

`PropagatingProcess<T, S, C>` keeps state and context generic; the chain threads real state and a typed context. Use it for Markovian pipelines: temporal simulations, closed-loop control, anything where one step's behavior depends on what earlier steps did.

Lifting from the stateless to the stateful form is a single constructor call: `PropagatingProcess::with_state(some_effect, initial_state, Some(initial_context))`. The five fields together are the irreducible structure for verifiable end-to-end reasoning. Drop the state field and Markovian cases need a separate type; drop the log field and audit becomes an external concern; drop the error field and short-circuit becomes guesswork.

For the long-form treatment see the [Effect Propagation Process](https://docs.deepcausality.com/concepts/effect-propagation-process/) concept page.

## Pearl's Ladder in a dozen lines

The three rungs of Pearl's Ladder of Causation map cleanly onto a `PropagatingEffect` chain (no state, no context needed for this example):

| Rung | Question | Operator | EPP expression |
|---|---|---|---|
| 1. Association | "If I see X, what do I expect about Y?" | `P(Y \| X)` | `pure(x).bind(f)` |
| 2. Intervention | "If I *do* X, what happens to Y?" | `P(Y \| do(X))` | `pure(x).bind(f).intervene(new)` |
| 3. Counterfactual | "Given the world as it is, what would have happened if X had been different?" | `P(Y_x \| X', Y')` | the same chain run twice, factually and with `intervene`, then compared |

Rung one is a `bind`. Rung two adds `intervene`. Rung three runs rung two against a held factual reference and compares the two outcomes.

Here is the canonical walkthrough using a stateless `PropagatingEffect`:

```rust
use deep_causality_core::{Intervenable, PropagatingEffect};

// Causal chain: Dose -> Absorption -> Metabolism -> Response (numeric outcome).

// Rung 1: Association. Run the chain factually.
let observed = PropagatingEffect::pure(10.0_f64)
    .bind(|dose,  _, _| PropagatingEffect::pure(dose.into_value().unwrap_or_default() * 0.8))   // Absorption: 8.0
    .bind(|level, _, _| PropagatingEffect::pure(level.into_value().unwrap_or_default() - 2.0))  // Metabolism: 6.0
    .bind(|level, _, _| PropagatingEffect::pure(level.into_value().unwrap_or_default()));       // Response:   6.0

// Rung 2: Intervention. do(BloodLevel := 3.0) mid-chain.
let intervened = PropagatingEffect::pure(10.0_f64)
    .bind(|dose, _, _| PropagatingEffect::pure(dose.into_value().unwrap_or_default() * 0.8))   // Absorption: 8.0
    .intervene(3.0)                                                                              // do(BloodLevel := 3.0)
    .bind(|level, _, _| PropagatingEffect::pure(level.into_value().unwrap_or_default() - 2.0)) // Metabolism: 1.0
    .bind(|level, _, _| PropagatingEffect::pure(level.into_value().unwrap_or_default()));      // Response:   1.0

// Rung 3: Counterfactual. The causal-effect estimate is the difference between
// the intervened outcome and the observed outcome (individual treatment effect):
//     ITE = Y(do(X)) - Y(X_observed)
let y_obs = observed.value.into_value().unwrap_or_default();
let y_int = intervened.value.into_value().unwrap_or_default();
let causal_effect = y_int - y_obs;

println!("Observed Y      = {y_obs:.2}");
println!("Intervened Y    = {y_int:.2}");
println!("Causal effect Δ = {causal_effect:+.2}"); // -5.00: the intervention lowered the response by five units.
```

The two runs share their structure and their composition law. The only difference is the `.intervene(3.0)` call. The causal-effect estimate is the *difference* `Y(do(X)) − Y(X_observed)`; here it is `1.00 − 6.00 = −5.00`, meaning the intervention lowered the response by five units. The log on `intervened` records the original level, the substituted value, and that an intervention occurred; the run stays replayable and auditable.

## The classical methods, reimplemented

The [`classical_causality_examples`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples) crate ships the five textbook methods of computational causality (CATE, DBN, Granger, RCM, SCM), each implemented **twice**: once with `Causaloid` + Contextual Alternation, once with `PropagatingProcess` + the `Alternatable` family. Both produce identical numbers; they differ only in *where* the alternation lives. The [crate README](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples) carries a full side-by-side comparison.

For the scope of this post, we focus on the **monad approach**. Two illustrations together exercise the three alternation channels.

### Illustration 1: SCM (Pearl's Ladder) on a chain, exercising value and context alternation

[`scm_via_monad`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples/classical_via_causal_monad/scm) puts the smoking-tar-cancer chain on `PropagatingProcess<f64, (), SmokingContext>`. Each rung uses one operator:

| Rung | Operator | Channel |
|---|---|---|
| 1. Association | plain `bind` chain | none |
| 2. Intervention | `.intervene(0.0)` mid-chain, applying `do(Tar := 0.0)` | value |
| 3. Counterfactual | `.alternate_context(other_world)` at the seed | context |

The intervention severs the Smoking → Tar link before stage 2 reads it, so cancer risk drops to false even though the patient still has high nicotine. The counterfactual swaps the world to one where nicotine is low but tar already accumulated; cancer risk stays high because the tar is still in the lungs. Same chain in both cases, two distinct alternation operators, two distinct audit-log markers (`!!ValueAlternation!!`, `!!ContextAlternation!!`).

### Illustration 2: DBN with a mid-stream regime change, context alternation plus state threading

[`dbn_via_monad`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples/classical_via_causal_monad/dbn) runs the Umbrella World over ten days on `PropagatingProcess<f64, WeatherState, WeatherContext>`. The `WeatherContext` carries the climate's conditional probability tables; the `WeatherState` carries the running Markov state (`rained_yesterday`, day counter, umbrella counter).

Two ten-day simulations run side by side:

1. **Baseline all the way.** Dry-leaning climate for ten days. `umbrellas_carried = 0`.
2. **Regime change mid-stream.** Baseline for five days, then `process.alternate_context(monsoon_climate())` switches the CPTs, and the remaining five days run under the alternated context. `umbrellas_carried = 5`.

The Markov state threads through the alternation untouched: the day counter continues `1..10` across the switch, and the umbrella counter accumulates correctly. The `EffectLog` records exactly one `!!ContextAlternation!!` entry pinpointing the switch. The `WeatherState` channel is preserved by `alternate_context`'s contract; were you to also reset the umbrella counter on regime change, that would be one additional `.alternate_state(...)` call away.

Between these two illustrations, the **value**, **context**, and **state** channels are all exercised, each by its own operator, each producing its own audit-log marker. The remaining three methods (RCM, CATE, Granger) follow the same patterns on a smaller scale; their [`via_monad`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples/classical_via_causal_monad) implementations live alongside. Read the [comparison README](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples) for more details.

## Closed-loop corrective control: the part classical inference cannot reach

Four corrective examples push the EPP categorically beyond what SCM, Granger, or DBN can do. `PropagatingProcess` threads `state` and `context` through every `bind`, so a monitor running inside the chain has direct access to the model's own current understanding of the world. The same Causaloids that drove the forward simulation drive the correction: the threshold lives in the `Context`, the drift accumulates in the `State`, and the corrective value is computed by the same model that produced the anomalous value. Then `.intervene(corrected)` rewrites the value field of the `PropagatingProcess`, and the next `bind` advances from the corrected state. The loop is fully internal to the causal model.

This is not "an inference framework with a controller bolted on." It is one `PropagatingProcess` doing both jobs because the EPP makes state, context, and value first-class on the same struct. Each of the four examples runs the same chain twice: open loop (catastrophic failure) and closed loop (failure averted); the diff between the two is the closed-loop block shown below.

| Example | Carrier | What it shows |
|---|---|---|
| `corrective_lane_keeping` | `PropagatingProcess<_, VehicleState, LaneConfig>` | Vehicle drifts under a crosswind. Monitor fires a P-controller correction whenever offset crosses 0.30 m. Open loop runs off the road at tick 24; closed loop stays in lane indefinitely. |
| `corrective_glucose_pump` | `PropagatingProcess<_, PatientState, PumpConfig>` | Blood glucose climbs across three meals. Monitor triggers a corrective bolus above 180 mg/dL. Open loop crosses the ketoacidosis line at tick 9; closed loop stays safe. |
| `corrective_decompression_stops` | `PropagatingProcess<_, DiveState, DiveConfig>` | A diver ascends from 30 m. Monitor inserts a decompression stop whenever the Bühlmann ratio crosses the safety threshold. Open loop crosses the DCS line at tick 8; closed loop surfaces safely. |
| `corrective_network_failover` | `PropagatingProcess<_, NetworkState, NetworkPlan>` | Active/standby switch pair. Primary fails on a scheduled tick; monitor detects zero delivery; `.intervene(STANDBY_SWITCH)` reroutes onward. Open loop loses 25 000 of 30 000 packets; closed loop loses 1 000 and stays inside the service objective. |

The lane-keeping example is the most direct read of what `intervene` is doing in this mode. The open-loop main loop is the canonical bind chain:

```rust
fn run_open_loop() -> LaneProcess<FloatType> {
    let mut process = model::initial_process();
    for _ in 0..N_TICKS {
        process = process.bind(model::simulate_step);
    }
    process
}
```

The closed-loop version adds a five-line monitor:

```rust
fn run_closed_loop() -> LaneProcess<FloatType> {
    let mut process = model::initial_process();
    for _ in 0..N_TICKS {
        process = process.bind(model::simulate_step);

        let current = match &process.value {
            EffectValue::Value(v) => *v,
            _ => continue,
        };
        let cfg = process.context.clone().expect("LaneConfig present");
        if current.abs() > cfg.anomaly_threshold {
            let corrected = model::correction(current, &cfg);
            process.state.correction_count += 1;
            process = process.intervene(corrected);
        }
    }
    process
}
```

Same chain, same simulation step, same drift schedule. The only difference is the four lines of monitor logic that decide whether to call `.intervene(corrected)` on each tick. The `EffectLog` ends up recording every correction with its corrected value, so the run is reviewable after the fact.

Read the four corrective examples together and a pattern emerges. The monitor block is small and identical in shape across them:

```rust
let current = match &process.value { EffectValue::Value(v) => *v, _ => continue };
let cfg = process.context.clone().expect("...");
if anomaly(current, &cfg) {
    let corrected = model::correction(current, &cfg);
    process.state.<correction_counter> += 1;
    process = process.intervene(corrected);
}
```

Same shape in [`corrective_lane_keeping`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_intervention_examples/corrective_lane_keeping), [`corrective_glucose_pump`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_intervention_examples/corrective_glucose_pump), [`corrective_decompression_stops`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_intervention_examples/corrective_decompression_stops), and [`corrective_network_failover`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/causal_intervention_examples/corrective_network_failover). The anomaly predicate, the correction function, and the units of the state counter change with the domain (lateral offset in meters, blood glucose in mg/dL, Bühlmann supersaturation ratio, packet delivery counts), but the architecture is one block of monitor logic operating on the same `PropagatingProcess`.

Two consequences are worth naming explicitly:

**The anomaly detector and the corrector are inside the causal model.** The `cfg` it reads is the same `Context` the forward steps read. The `current` value it inspects is the same value the next `bind` would otherwise consume. The corrected value comes from a function defined in the same `model` module as the forward dynamics. There is no separate controller process running alongside the model with its own duplicated world view; the controller *is* the model, reading its own state.

**The correction does not break the audit log.** The `EffectLog` records every monitor tick, every detection, and every `!!Intervention!!: <old> replaced with <new>` event. A post-incident review reads the same log a debugger would. A regulator audit reads the same log a developer would. The corrective autonomy is observable by construction.

## What this means

The design choices add up to a small set of concrete properties:

* **Three independent alternation channels.** Value, context, and state are each substitutable through their own trait and method, on the same `PropagatingProcess`, composable in any order. (On a stateless `PropagatingEffect` only value alternation does observable work; the context and state methods accept the unit value `()` but only emit a log entry.) The `error` and `logs` channels stay non-alternable by design: overwriting an error is a safety violation, and the log is append-only.
* **Same chain, different worlds.** Replay the same `bind` pipeline under several alternations without branching code.
* **Audit trail on rails.** Every alternation appends a distinctive marker (`!!ValueAlternation!!`, `!!ContextAlternation!!`, `!!StateAlternation!!`); the substitution history is inspectable in the same `EffectLog` as the rest of the run.
* **Error-state preservation.** If the chain is already in error, every alternation method is a no-op. None of them can fix a broken upstream.
* **Channel isolation.** A `.alternate_value(...)` rewrites only the value; the context and state continue through the chain untouched. The two other operators behave symmetrically.
* **Intervention site encodes a causal claim.** Where in the chain you intervene says what counts as upstream of the manipulated quantity.
* **Comparative evaluation as the estimand.** The difference between factual and counterfactual *is* the causal quantity you wanted; the SCM rung-3 example writes this definition out as code.
* **Composable closed-loop control.** A monitor decides each tick whether to intervene; the chain advances from the corrected value. Failure averted instead of measured.

None of these properties is new in isolation. What is new is that they fall out of one foundational move (separating Causaloid from Context, then threading effects through `PropagatingEffect` or `PropagatingProcess`) and compose under one trait family. Counterfactuals without abduction, mid-chain intervention without graph mutilation, three-channel substitution without a separate operator language, closed-loop corrective control without a separate controller process: all of them use the same `bind` plus `Alternatable` primitives on the same carrier types. The rest of the framework (Causaloids, Context, Effect Ethos) composes against those types without glue code.

## Getting started

* Read the [Counterfactuals](https://docs.deepcausality.com/concepts/counterfactuals/) concept page for the canonical write-up.
* Read the [Causal Monad](https://docs.deepcausality.com/concepts/causal-monad/) and [Effect Propagation Process](https://docs.deepcausality.com/concepts/effect-propagation-process/) pages for the supporting algebra.
* Run the SCM and DBN illustrations from this post:
  * `git clone https://github.com/deepcausality-rs/deep_causality.git  && cd deep_causality`
  * `cargo run -p classical_causality_examples --example scm_via_monad`
  * `cargo run -p classical_causality_examples --example dbn_via_monad`
* Read the [classical-examples README](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples) for the full side-by-side comparison and the practitioner decision guide.
* Run any of the closed-loop corrective examples: `cargo run -p causal_intervention_examples --example corrective_lane_keeping` (pick any of the four).
* Join the [community](https://www.deepcausality.com/community/) or the [Discord server](https://discord.gg/Bxj9P7JXSj).

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and deterministic context-aware causal reasoning in Rust. Please give us a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open source innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all members of the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).
