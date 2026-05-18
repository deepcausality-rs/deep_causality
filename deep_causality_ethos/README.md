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

The crate implements the teleological layer described in section 8 of the Effect Propagation Process paper. It pairs a defeasible deontic logic with the DeepCausality `Context` so that norms can read the same spatio-temporal state that causal reasoning operates on.

## Overview

The unit of regulation is a `Teloid`: a single norm that names an action, an activation predicate over `(Context, ProposedAction)`, a modality, and three heuristics used for conflict resolution (specificity, priority, timestamp). Teloids are kept in a `TeloidStore`, indexed by tag in a `TagIndex`, and linked in a `TeloidGraph` whose edges carry a `TeloidRelation` of either `Inherits` or `Defeats`.

The `EffectEthos` struct owns these components and exposes the reasoning API. Evaluation runs in five steps:

1. Tag-based filtering selects candidate norms from the `TagIndex`.
2. Each candidate's activation predicate is run against the `Context` and the `ProposedAction`. Uncertain predicates are tested with the teloid's `UncertainParameter` (threshold, confidence, epsilon, sample bound).
3. Active norms are reduced through the `Defeats` edges in the graph (defeasance).
4. Survivors are checked for consistency under `Lex Specialis`, `Lex Superior`, and `Lex Posterior`.
5. A `Verdict` is returned, carrying the final modality and the IDs of the norms that justify it.

The graph must be frozen and verified for acyclicity before evaluation; calling `verify_graph()` performs both.

## Features

* **Deterministic and uncertain norms:** `add_deterministic_norm` takes a `fn` predicate. `add_uncertain_norm` takes an `UncertainActivationPredicate` and an `UncertainParameter`, lifting probabilistic activation into the deontic layer.
* **Explicit conflict resolution:** specificity, priority, and recency are first-class fields on every `Teloid`. Resolution is deterministic and reproducible.
* **Auditable verdicts:** every `Verdict` carries a `justification: Vec<TeloidID>` so a decision can be traced back to the norms that produced it. The `DeonticExplainable` trait exposes this trail.
* **Context-aware predicates:** norms read the full DeepCausality `Context<D, S, T, ST, SYM, VS, VT>`, so deontic rules can depend on space, time, symbolic state, and data in one expression.
* **Static dispatch:** no `dyn` in the public API; the engine is generic over the same seven type parameters as the rest of the DeepCausality core.

## Public API

`lib.rs` exports:

* Types: `EffectEthos`, `Teloid`, `TeloidStore`, `TeloidGraph`, `TagIndex`, `TeloidModal`, `TeloidRelation`, `Verdict`.
* Traits: `DeonticInferable`, `DeonticExplainable`, `TeloidStorable`, `Teloidable`.
* Aliases: `BaseTeloidStore`, `TeloidID` (`u64`), `TeloidTag`.
* Errors: `DeonticError`.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
deep_causality_ethos = "0.2"
deep_causality = "0.13"
```

### Building an EffectEthos

```rust
use deep_causality_ethos::{EffectEthos, TeloidModal, DeonticInferable};
use deep_causality::{ActionParameterValue, Context, ProposedAction};
use std::collections::HashMap;

// Define a deterministic predicate over Context and ProposedAction.
// "A drone must not take off when battery is below 20%."
fn battery_below_minimum<D, S, T, ST, SYM, VS, VT>(
    _ctx: &Context<D, S, T, ST, SYM, VS, VT>,
    action: &ProposedAction,
) -> bool {
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
        &["flight_safety".to_string()], // tags
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
let context = /* a deep_causality::Context */;

let verdict = ethos
    .evaluate_action(&action, &context, &["flight_safety".to_string()])
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

A full worked example, including the `Context` setup and a CSM integration, lives at
[`examples/csm_examples/csm_effect_ethos`](../examples/csm_examples/csm_effect_ethos).

## Modalities

| Modality           | Meaning                                                                |
|--------------------|------------------------------------------------------------------------|
| `Obligatory`       | The action must be taken; omission is a violation.                     |
| `Impermissible`    | The action must not be taken; performing it is a violation.            |
| `Optional(i64)`    | The action is permitted and carries an explicit cost.                  |

## Relation to other DeepCausality crates

* `deep_causality` supplies `Context`, `ProposedAction`, `Uncertain`, and the seven generic parameters used here.
* `ultragraph` backs the `TeloidGraph`; freeze and acyclicity checks come from it.
* The ethos layer is independent of `deep_causality_effects`. Causal reasoning answers what will happen; the ethos answers what should happen.

## References

* Olson, T., Salas-Damian, R., and Forbus, K. D. *A Defeasible Deontic Calculus for Resolving Norm Conflicts.* Department of Computer Science, Northwestern University. The DDIC formalism underlying the conflict resolution rules used here:
  [docs/papers/ddic.pdf](../docs/papers/ddic.pdf)
* Effect Propagation Process paper, section 8 (Teleology):
  <https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf>
* In-repo overview: [docs/ETHOS.md](../docs/ETHOS.md)

## License

This project is licensed under the [MIT license](LICENSE).
