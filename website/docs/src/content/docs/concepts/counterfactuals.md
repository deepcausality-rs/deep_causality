---
title: Counterfactuals
description: Pearl's Ladder of Causation in the Effect Propagation Process. The Alternatable trait family substitutes value, context, or state mid-chain, preserves the audit trail, and dissolves Pearl's abduction step.
sidebar:
  order: 13
---

Counterfactual reasoning is first-class in DeepCausality. The same machinery that runs factual evaluation runs counterfactual evaluation. The mechanism is the [`Alternatable`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_core/src/traits) family of traits: one trait per substitutable channel on the carrier, plus a marker super-trait that bundles all three. The classic causal-inference operator `intervene` survives as a thin vocabulary alias atop value alternation.

## Pearl's Ladder of Causation

Pearl distinguishes three rungs of causal reasoning, each strictly stronger than the one below:

| Rung | Question | Operator | EPP expression |
|---|---|---|---|
| 1. Association | "If I see X, what do I expect about Y?" | `P(Y \| X)` | `pure(x).bind(f)`; read-only composition |
| 2. Intervention | "If I *do* X, what happens to Y?" | `P(Y \| do(X))` | `pure(x).bind(f).intervene(new)`; overrides a value mid-chain |
| 3. Counterfactual | "Given the world as it is, what *would* have happened if X had been different?" | `P(Y_x \| X', Y')` | the same chain run twice, factually and with an alternation, then compared |

The first rung is a `bind`. The second adds value alternation (`intervene` / `alternate_value`). The third rung runs rung two against a held factual reference and compares the two outcomes. The architecture is the same in every case: a chain whose value, state, context, error, and log are the only thing being threaded.

## The Alternatable family: three channels, three traits

The carrier struct (`CausalEffectPropagationProcess`) has five fields: `value`, `state`, `context`, `error`, `logs`. Three are legitimately substitutable mid-chain; two are not. Substituting `error` would silently paper over a failure upstream, and `logs` is append-only by design. That leaves three alternable channels.

DeepCausality exposes one single-method trait per channel:

```rust
pub trait AlternatableValue<V>   { fn alternate_value(self,   new: V) -> Self; }
pub trait AlternatableContext<C> { fn alternate_context(self, new: C) -> Self; }
pub trait AlternatableState<S>   { fn alternate_state(self,   new: S) -> Self; }
```

Plus a marker super-trait, auto-implemented via blanket impl, for generic code that needs all three at once:

```rust
pub trait Alternatable<V, C, S>:
    AlternatableValue<V> + AlternatableContext<C> + AlternatableState<S> {}

impl<T, V, C, S> Alternatable<V, C, S> for T
where T: AlternatableValue<V> + AlternatableContext<C> + AlternatableState<S> {}
```

The familiar `Intervenable<V>` trait survives as a thin vocabulary alias atop `AlternatableValue<V>`, so existing code that calls `effect.intervene(x)` keeps working unchanged:

```rust
pub trait Intervenable<V>: AlternatableValue<V> {
    fn intervene(self, new_value: V) -> Self where Self: Sized {
        self.alternate_value(new_value)
    }
}
impl<T, V> Intervenable<V> for T where T: AlternatableValue<V> {}
```

`intervene` is the causal-inference word for value substitution (Pearl's `do(...)`). `alternate_value` is the same operation under the EPP's substitution vocabulary. Same method body, two surfaces.

### The shared contract

Every method in the family follows the same three rules:

- **Error short-circuit.** If the upstream chain has already errored, the call is a no-op. An alternation cannot fix a broken chain.
- **Channel isolation.** Only the named channel is rewritten; the other two alternable channels, plus the error and log, continue unchanged.
- **Automatic audit entry.** A distinctive marker is appended to the log: `!!ValueAlternation!!: <old> replaced with <new>`, `!!ContextAlternation!!: context replaced`, or `!!StateAlternation!!: state replaced`.

The three operators compose freely. On a `PropagatingProcess` you can intervene on the value, alternate the context, and reset the state in any order within the same chain; each emits its own audit entry, and downstream `bind` steps see the substituted channels.

### Which channels carry real information

The trait family is implemented uniformly on the underlying struct, so both pinned aliases satisfy it. The channels that carry information depend on which alias you use:

- **`PropagatingEffect<T>`** pins `State` and `Context` to `()`. Only value alternation does observable work on this alias; `alternate_context(())` and `alternate_state(())` are well-typed but only append a log entry.
- **`PropagatingProcess<T, S, C>`** keeps state and context generic. All three operators are meaningful.

Use `PropagatingEffect` for Pearl-style stateless chains. Use `PropagatingProcess` whenever a chain needs to thread Markovian state or a typed world context.

## Walking the Ladder in code

The intervention example from the project [README](https://github.com/deepcausality-rs/deep_causality) walks all three rungs in roughly a dozen lines on a stateless `PropagatingEffect`:

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

The two runs share their structure and their composition law. The only difference is the `.intervene(3.0)` call. The causal-effect estimate is the **difference** `Y(do(X)) − Y(X_observed)`; in this run that is `1.00 − 6.00 = −5.00`. The log on `intervened` records the original blood level, the substituted value, and the marker that an intervention occurred; the run stays replayable and auditable.

## Beyond the value channel

Pearl's `do(...)` operator only swaps **one** thing: the value of a single endogenous variable, with the exogenous noise held fixed by the abduced posterior. The `Alternatable` family widens the substitution surface to three independent channels and removes the inference step entirely:

- **Value alternation** (`alternate_value` / `intervene`): Pearl's `do(...)`, expressed as one method call.
- **Context alternation** (`alternate_context`): swap the entire world (or any structured piece of it) without rebuilding the chain. The classical [SCM example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples/classical_via_causal_monad/scm) uses this for Pearl's rung 3; the [RCM example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples/classical_via_causal_monad/rcm) uses it to compute potential outcomes across treatment and control.
- **State alternation** (`alternate_state`): force the running Markov state to a new value. Useful for simulator resets, regime changes that touch accumulated counters, and test fixtures. The [DBN example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples/classical_via_causal_monad/dbn) shows context alternation alongside state threading: a mid-stream climate-regime change leaves the day counter and umbrella count intact.

Abduction never enters this picture, because `PropagatingProcess` already carries the world state explicitly. The trade is concrete: the world state must be represented as a Context up front, instead of inferred from observations.

## Why mid-chain matters

Most counterfactual frameworks require structural manipulation (mutilating the SCM, rebuilding the graph, re-evaluating from the root). That works at the model level but is expensive and obscures the trace. The `Alternatable` family operates at the **value, context, or state level** along the existing chain. The causal law, the Causaloid graph, and the audit log are unchanged; the rewritten channel flows through the remaining steps as if it had been produced upstream. For interactive what-if analysis, sensitivity testing, and Pearl-style do-calculus over a running pipeline, that is the cheap and honest operation to have.

These traits are not a substitute for structural intervention when the question genuinely changes the model (deleting an edge, removing a Causaloid). Those are graph-level edits, and the EPP expresses them by composing a different Causaloid graph against the same Context. The `Alternatable` family is the channel-level rung; structural surgery is its model-level counterpart.

## What this means

- **Counterfactuals as a one-line API.** A counterfactual is one alternation call between `start(ctx)` and the binds.
- **Three independent substitution channels.** Value, context, and state can be alternated independently or in any combination, each emitting its own distinctive audit marker.
- **Replayable counterfactual analysis.** Every alternation is recorded in the same log as the factual run, so a downstream consumer can reproduce both.
- **No abduction step.** The world state lives in the Context explicitly, so "the world as it actually was" is a value, not an inferred posterior over hidden noise.
- **Composable with the rest of the algebra.** A counterfactual chain returns a `PropagatingEffect` or `PropagatingProcess`, so it composes with Causaloid evaluations, downstream `bind` steps, and the Effect Ethos check just like any factual chain would.

## See also

- [Causal Monad](/concepts/causal-monad/): the `pure`/`bind` algebra that the `Alternatable` family plugs into.
- [Effect Propagation Process](/concepts/effect-propagation-process/): the carrier whose channels the family rewrites.
- [Causaloid](/concepts/causaloid/): for structural (graph-level) counterfactuals.
- [Classical Causality Examples](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples): five textbook methods (CATE, DBN, Granger, RCM, SCM) implemented twice, once with the Causaloid + Contextual Alternation pattern and once with `PropagatingProcess` + the `Alternatable` family. Includes a practitioner decision guide.
- [`examples/starter_example`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/starter_example): walks Pearl's Ladder end to end on `PropagatingEffect`.
