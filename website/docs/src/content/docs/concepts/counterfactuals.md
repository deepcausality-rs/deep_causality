---
title: Counterfactuals
description: Pearl's Ladder of Causation in the Effect Propagation Process. The Intervenable trait and the intervene operator make do(X := x) a first-class, mid-chain operation that preserves the audit trail.
sidebar:
  order: 12
---

Counterfactual reasoning is first-class in DeepCausality. The same machinery that runs factual evaluation runs counterfactual evaluation; no separate engine, no separate type, no off-stack tooling. The lever is one trait, [`Intervenable`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_core/src/traits/intervenable/mod.rs), with one method, `intervene`, implemented for the [carrier effect](/concepts/effect-propagation-process/).

## Pearl's Ladder of Causation

Pearl distinguishes three rungs of causal reasoning, each strictly stronger than the one below:

| Rung | Question | Operator | EPP expression |
|---|---|---|---|
| 1. Association | "If I see X, what do I expect about Y?" | `P(Y \| X)` | `pure(x).bind(f)`; read-only composition |
| 2. Intervention | "If I *do* X, what happens to Y?" | `P(Y \| do(X))` | `pure(x).bind(f).intervene(new)`; overrides a value mid-chain |
| 3. Counterfactual | "Given the world as it is, what *would* have happened if X had been different?" | `P(Y_x \| X', Y')` | the same chain run twice, once factually and once with `intervene`, then compared |

The first rung is a `bind`. The second rung adds `intervene`. The third rung is the second rung run against a held factual reference. The architecture is the same in every case: a propagating-effect chain whose value, state, context, error, and log are the only thing being threaded.

## The `intervene` operator

```rust
pub trait Intervenable<T> {
    /// Replace the carried value with `new_value`, preserving the rest of the chain.
    ///
    /// - If the upstream chain already errored, the error is propagated and the
    ///   intervention is **not** applied; an intervention cannot fix a broken chain.
    /// - The log is preserved, and an `"!!Intervention!!"` entry is appended so
    ///   the audit trail records *what* was replaced and *with what*.
    fn intervene(self, new_value: T) -> Self;
}
```

`Intervenable<Value>` is implemented for the [`CausalEffectPropagationProcess`](/concepts/effect-propagation-process/) carrier, so `.intervene(...)` works on both `PropagatingEffect<T>` and `PropagatingProcess<T, S, C>`.

Two properties make the operator usable as a counterfactual primitive:

- **It is mid-chain.** No need to restart the pipeline, rebuild context, or re-evaluate upstream rules. Any step that already produced a value can have that value rewritten before the next step consumes it.
- **It preserves the audit trail.** The log records the original value, the substituted value, and the marker that an intervention occurred. A run that mixes factual and counterfactual evaluation leaves a single replayable trace.

## Walking the Ladder in code

The intervention example from the project [README](https://github.com/deepcausality-rs/deep_causality) walks all three rungs in roughly a dozen lines:

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

The two runs share their structure and their composition law. The only difference is the `.intervene(3.0)` call. The causal-effect estimate is the **difference** `Y(do(X)) − Y(X_observed)`; in this run that is `1.00 − 6.00 = −5.00`, meaning the intervention lowered the response by five units. The log on `intervened` records the original blood level, the substituted value, and that an intervention occurred; the run stays replayable and auditable.

## Why mid-chain matters

Most counterfactual frameworks require structural manipulation (mutilating the SCM, rebuilding the graph, re-evaluating from the root). That works at the model level but is expensive and obscures the trace. `intervene` works at the **value level** along the existing chain: the rule set, the Causaloid graph, the Context, and the audit log are unchanged, and the rewritten value flows through the remaining steps as if it had been produced upstream. For interactive what-if analysis, sensitivity testing, and Pearl-style do-calculus over a running pipeline, that's the cheap and honest operation to have.

It is not a substitute for structural intervention when the question genuinely changes the model (deleting an edge, removing a Causaloid). Those are graph-level edits, and the EPP can express them by composing a different Causaloid graph against the same Context. `intervene` is the value-level rung; structural surgery is its model-level counterpart.

## What this earns you

- **Counterfactuals as a one-line API.** No engine swap, no model rebuild.
- **Replayable counterfactual analysis.** The intervention is recorded in the same log as the factual run, so a downstream consumer can reproduce both.
- **Composable with the rest of the algebra.** A counterfactual chain returns a `PropagatingEffect`, so it composes with Causaloid evaluations, downstream `bind` steps, and the Effect Ethos check just like any factual chain would.

## See also

- [Causal Monad](/concepts/causal-monad/): the `pure`/`bind` algebra `intervene` plugs into.
- [Effect Propagation Process](/concepts/effect-propagation-process/): the carrier whose value `intervene` rewrites.
- [Causaloid](/concepts/causaloid/): for structural (graph-level) counterfactuals.
- Example: [`examples/starter_example`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/starter_example) walks Pearl's Ladder end to end.
