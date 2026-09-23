# DeepCausality Ethos

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/deep_causality.svg
[crates-url]: https://crates.io/crates/deep_causality_ethos
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/deepcausality/deep_causality.rs/blob/main/LICENSE
[actions-badge]: https://github.com/deepcausality/deep_causality.rs/workflows/CI/badge.svg
[actions-url]: https://github.com/deepcausality/deep_causality.rs/actions

**DeepCausality Ethos** is a programmable deontic reasoning layer for the DeepCausality stack. It evaluates a `ProposedAction` against a set of norms and returns a justified `Verdict` (`Obligatory`, `Impermissible`, or `Optional(cost)`).

The crate implements the teleological layer from section 8 of the Effect Propagation Process paper. It pairs a defeasible deontic logic with the DeepCausality `Context`, so norms read the same spatio-temporal state as causal reasoning.

## Overview

The unit of regulation is a `Teloid`: a single norm that names an action, an activation predicate over `(Context, ProposedAction)`, a modality, and three conflict-resolution heuristics (specificity, priority, timestamp). A `TeloidStore` holds the teloids, a `TagIndex` indexes them by tag, and a `TeloidGraph` links them with edges carrying a `TeloidRelation` of `Inherits` or `Defeats`.

The `EffectEthos` struct owns these components and exposes the reasoning API. Evaluation runs in five steps:

1. Tag-based filtering selects candidate norms from the `TagIndex`.
2. Each candidate's activation predicate runs against the `Context` and the `ProposedAction`. The teloid's `UncertainParameter` (threshold, confidence, epsilon, sample bound) tests uncertain predicates.
3. The `Defeats` edges in the graph remove defeated norms from the active set (defeasance).
4. `Lex Specialis`, `Lex Superior`, and `Lex Posterior` check the survivors for consistency.
5. The engine returns a `Verdict` carrying the final modality and the IDs of the norms that justify it.

Evaluation requires a frozen, acyclic graph; `verify_graph()` freezes the graph and checks it for cycles.

## Features

* **Deterministic and uncertain norms:** `add_deterministic_norm` takes a `fn` predicate. `add_uncertain_norm` takes an `UncertainActivationPredicate` and an `UncertainParameter`, bringing probabilistic activation into the deontic layer.
* **Explicit conflict resolution:** specificity, priority, and recency are first-class fields on every `Teloid`. Resolution is deterministic and reproducible.
* **Auditable verdicts:** every `Verdict` carries a `justification: Vec<TeloidID>` that traces the decision to the norms that produced it. The `DeonticExplainable` trait exposes this trail.
* **Context-aware predicates:** norms read the full DeepCausality `Context<D, S, T, ST>`, so deontic rules can depend on data, space, time, and spacetime in one expression.
* **Static dispatch:** no `dyn` in the public API; the engine is generic over the same four type parameters as the context layer.

## Public API

`lib.rs` exports:

* Types: `EffectEthos`, `Teloid`, `TeloidStore`, `TeloidGraph`, `TagIndex`, `TeloidModal`, `TeloidRelation`, `Verdict`.
* Traits: `DeonticInferable`, `DeonticExplainable`, `TeloidStorable`, `Teloidable`.
* Aliases: `BaseTeloidStore`, `FloatType` (`f64`), `TeloidID` (`u64`), `TeloidTag` (`&'static str`).
* Errors: `DeonticError`.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
deep_causality_ethos = "0.3"
deep_causality = "0.17"
deep_causality_context = "0.1"
```

### Building an EffectEthos

```rust
use deep_causality_ethos::{EffectEthos, TeloidModal, DeonticInferable};
use deep_causality::{ActionParameterValue, ProposedAction};
use deep_causality_context::{Context, Datable, SpaceTemporal, Spatial, Temporal};
use std::collections::HashMap;

// Define a deterministic predicate over Context and ProposedAction.
// "A drone must not take off when battery is below 20%."
fn battery_below_minimum<D, S, T, ST>(
    _ctx: &Context<D, S, T, ST>,
    action: &ProposedAction,
) -> bool
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    match action.parameters().get("battery_pct") {
        Some(ActionParameterValue::Number(pct)) => *pct < 20.0,
        _ => false,
    }
}

// Build the ethos with a single norm, then freeze and verify the graph.
let mut ethos = EffectEthos::new()
    .add_deterministic_norm(
        1,                            // TeloidID
        "takeoff",                    // action identifier
        &["flight_safety"],           // tags
        battery_below_minimum,        // predicate
        TeloidModal::Impermissible,   // modality
        0,                            // timestamp
        10,                           // specificity
        100,                          // priority
    )
    .expect("failed to add norm");

ethos.verify_graph().expect("graph must be acyclic");
```

### Evaluating a proposed action

```rust
let mut params = HashMap::new();
params.insert("battery_pct".to_string(), ActionParameterValue::Number(12.5));
let action = ProposedAction::new(1, "takeoff".to_string(), params);
let context = /* a deep_causality_context::Context */;

let verdict = ethos
    .evaluate_action(&action, &context, &["flight_safety"])
    .expect("evaluation failed");

match verdict.outcome() {
    TeloidModal::Impermissible => { /* forbidden */ }
    TeloidModal::Obligatory    => { /* required */ }
    TeloidModal::Optional(_)   => { /* permitted with cost */ }
}

for norm_id in verdict.justification() {
    if let Some(norm) = ethos.get_norm(*norm_id) {
        println!("#{} {} -> {:?}", norm_id, norm.action_identifier(), norm.modality());
    }
}
```

A full example, including the `Context` setup and a CSM integration, lives at
[`examples/csm_examples/csm_effect_ethos`](../examples/csm_examples/csm_effect_ethos).

## Modalities

| Modality           | Meaning                                                                |
|--------------------|------------------------------------------------------------------------|
| `Obligatory`       | The action must be taken; omission is a violation.                     |
| `Impermissible`    | The action must not be taken; performing it is a violation.            |
| `Optional(i64)`    | The action is permitted and carries an explicit cost.                  |

## Relation to other DeepCausality crates

* `deep_causality` supplies `ProposedAction`, `ActionParameterValue`, `UncertainActivationPredicate`, and `UncertainParameter`.
* `deep_causality_context` supplies `Context` and the four generic parameters used here.
* `ultragraph` backs the `TeloidGraph`; freeze and acyclicity checks come from it.

## References

* Olson, T., Salas-Damian, R., and Forbus, K. D. *A Defeasible Deontic Calculus for Resolving Norm Conflicts.* Department of Computer Science, Northwestern University. The DDIC formalism underlying the conflict resolution rules used here:
  [arxiv](https://arxiv.org/abs/2407.04869) [Copy](papers/ddic.pdf)
* Effect Propagation Process paper, section 8 (Teleology):
  <https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf>

## Contribution

Contributions are welcome, especially documentation, example code, and fixes.
If unsure where to start, open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).
