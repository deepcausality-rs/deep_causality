---
title: Effect Ethos
description: The deontic guardrail that intercepts every action the Causal State Machine proposes before it executes.
section: concepts
order: 4
---

A Causaloid or Causal Monad determines what the system *infers*. The [Causal State Machine](/docs/concepts/csm/) (CSM) translates that inference into a proposed action. The Effect Ethos says whether that proposed action is *allowed to execute*. The three layers encode different responsibilities.

## The three-layer flow

DeepCausality separates reasoning from action by design:

1. **Reasoning**: a Causaloid (Singleton, Collection, or hypergraph) and the Causal Monad produce a `PropagatingEffect`. This is the inference layer. It answers "what is the case?"
2. **Action proposal**: the CSM reads the propagating effect, evaluates which of its registered causal states have become active, and constructs a `ProposedAction` for each active state. This is the bridge between inference and the outside world. It answers "what to do next?"
3. **Action verification**: the Effect Ethos intercepts every proposed action before it fires. It evaluates the proposal against a graph of Teloids and returns a `Verdict`. This is the guardrail layer. It answers "is the system *allowed* to do that here, now, under these rules?"

The Effect Ethos is what safeguards the CSM. An action that the CSM would otherwise fire does not fire if the Ethos returns an impermissible verdict.

## The Origin of the Effect Ethos

The Effect Ethos exists because dynamic, emergent causality is intrinsically non-deterministic. The [Dynamic causality page](/docs/concepts/dynamic-causality/) lays out the four reasoning modalities and shows where the determinism boundary breaks: static, dynamic, and adaptive reasoning are formally verifiable, but emergent reasoning is not. The causal graph can rewire itself at runtime in ways no upfront proof can cover. Once that capability is on the table, "the reasoning is correct" stops being a property you can establish in advance. Something else has to absorb the verification burden.

The realization was that every action the system is about to take needs an independent check, expressed in rules that do not change while the reasoning evolves. The check has to handle conflicting rules, audit trails, and overrides without losing determinism. That is a deontic problem, not a causal one, and it had already been studied.

The answer came from Olson, Salas-Damian, and Forbus at Northwestern University in [*A Defeasible Deontic Calculus for Resolving Norm Conflicts*](https://github.com/deepcausality-rs/deep_causality/blob/main/docs/papers/ddic.pdf). The paper introduces the Defeasible Deontic Inheritance Calculus (DDIC): a formalism for resolving a continuous stream of possibly conflicting norms. It defines the three deontic modalities (Obligatory, Optional, Impermissible), characterizes the three conflict types (direct, indirect, intersecting), and proves that three resolution heuristics (Lex Specialis, Lex Posterior, Lex Superior) are sufficient to axiomatize conflict resolution under deontic inheritance. The paper goes further and shows that one widely used multi-agent strategy is a red herring once defeasance is modeled correctly.

The Effect Ethos is an implementation of DDIC inside DeepCausality. A `Teloid` is one norm tuple from the calculus. The `TeloidGraph` carries the inheritance and defeasance edges DDIC requires. `evaluate_action` runs the activation, conflict detection, and resolution steps from the paper, in the order DDIC prescribes, and returns a `Verdict` whose justification field is the audit trail the formalism implies. The mapping is intentional. DDIC gave a theoretically justified axiomatization of norm conflict detection and resolution; the Effect Ethos makes it runnable, embeds it in a typed context, and wires it to the Causal State Machine so that every proposed action passes through the calculus before it executes.


## The problem the Ethos solves

The Effect Ethos pulls scattered checks into one structured object that can answer the thorny questions. The CSM hands every proposed action to the Ethos before execution. The Ethos returns a verdict carrying both the outcome and the chain of Teloids that justified it.

## What it is

The `EffectEthos` lives in the [`deep_causality_ethos`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_ethos) crate. The shape:

```rust
pub struct EffectEthos<D, S, T, ST, SYM, VS, VT>
where
    /* … same context bounds as Context … */
{
    teloid_store: TeloidStore<D, S, T, ST, SYM, VS, VT>,
    tag_index: TagIndex,
    teloid_graph: TeloidGraph,
    id_to_index_map: HashMap<TeloidID, usize>,
    is_verified: bool,
}
```

Two parts to read. The `teloid_store` holds the active rules. The `teloid_graph` and `tag_index` make the rules navigable, both by id and by category.

A `Teloid` is the atom inside the store. It is a computable unit of purpose, and it instantiates one norm tuple from the DDIC calculus described in the [origin section above](#the-origin-of-the-effect-ethos). Concretely a Teloid carries:

- A **deontic modality**: `Obligatory`, `Impermissible`, or `Optional(cost)`.
- A **condition** evaluated against the Context (either a deterministic Causaloid or an `Uncertain` predicate).
- A **scope tag** used by the CSM to filter which Teloids apply to a given proposed action.
- An **id** that survives logging and shows up in every audit trail.

The [Teleology preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/teleology_effect_propagation_process/epp_teleology.pdf) introduces Teloids as the answer to the question, "What stops an emergent system from inferring its way into a state you cannot let it act on?"

## Building an Ethos

Norms are added through `add_deterministic_norm` (for deterministic Causaloid conditions) or `add_uncertain_norm` (for `Uncertain` predicates). Inheritance and defeasance edges between Teloids are wired up with `link_inheritance` and `link_defeasance`. Before the Ethos can be queried by a CSM, it must be `freeze()`'d. Freezing finalizes the Teloid graph the same way it finalizes a Causaloid graph, switching the underlying [`ultragraph`](https://github.com/deepcausality-rs/deep_causality/tree/main/ultragraph) backend to its query-optimized CSR form.

```rust
use deep_causality_ethos::EffectEthos;

let mut ethos: EffectEthos<_, _, _, _, _, _, _> = EffectEthos::new();
// ethos.add_deterministic_norm(...);
// ethos.link_inheritance(general_id, specific_id);
ethos.freeze();
```

The full API is on [docs.rs](https://docs.rs/deep_causality_ethos).

## Evaluating a proposed action

The CSM hands a `ProposedAction` to the Ethos along with the current Context and the scope tags that apply to the proposal. The Ethos returns a `Verdict`:

```rust
use deep_causality_ethos::{DeonticInferable, TeloidModal};

let verdict = ethos.evaluate_action(&proposed_action, &context, &tags)?;

match verdict.outcome() {
    TeloidModal::Obligatory     => csm.fire(proposed_action)?,
    TeloidModal::Impermissible  => csm.reject(verdict.justification())?,
    TeloidModal::Optional(cost) => csm.fire_if_within_budget(proposed_action, *cost)?,
}
```

The verdict carries both the outcome and a justification: the ordered list of Teloid ids that produced it. The CSM uses the outcome to decide whether to fire; the justification goes into the audit log so any decision the system makes can be replayed later with the same Context and the same Ethos.

## Conflict resolution

Real rule sets contradict each other. Two requirements both apply, one says obligatory, the other says impermissible. The Effect Ethos resolves the contradiction with three principles:

- **Lex Posterior**: the later-issued rule wins over the earlier one.
- **Lex Specialis**: the more specific rule wins over the more general one.
- **Lex Superior**: the higher-priority rule wins over the lower-priority one.

These run in a fixed order when the Ethos is asked to reconcile a conflict. The combination is enough to handle most rule-set evolution in practice without giving up determinism. The proof that these three heuristics are sufficient, and the analysis of which common resolution strategies they subsume, comes from the [DDIC paper](https://github.com/deepcausality-rs/deep_causality/blob/main/docs/papers/ddic.pdf) cited above.

## Why this is the right place to guardrail

The Effect Ethos can disagree with the inference layer, and the disagreement is the point.

A Causaloid graph reasons forward from inputs to a propagating effect. It is concerned with *what is inferable*. The CSM translates that inference into a *proposed action*. The Ethos reasons against the proposal from operational constraints. It is concerned with *what is permissible*. The two answers disagree often enough to be worth modelling separately. When they agree, the CSM fires. When they disagree, the rejection is structured and explainable.

This is what restores verifiability under emergent reasoning. The [Dynamic causality page](/docs/concepts/dynamic-causality/) breaks the four reasoning modalities at the determinism boundary: static, dynamic, and adaptive reasoning are all formally verifiable, while emergent reasoning is not. The reasoning graph may evolve in ways no static proof can foresee. The Effect Ethos accepts that reality and moves the verifiability checkpoint to the one place where it stays feasible: the action layer. Every action the CSM proposes is checked against an immutable Ethos before it leaves the system. The reasoning is free to be emergent; the actions are not.

## Where to look next

[Causal State Machine](/docs/concepts/csm/) is the layer that proposes the actions the Ethos checks. [Causaloid](/docs/concepts/causaloid/) is the inference layer the CSM reads from. The API reference is on docs.rs at [`deep_causality_ethos`](https://docs.rs/deep_causality_ethos).

Two papers ground the design:

- Olson, T., Salas-Damian, R., and Forbus, K. D. [*A Defeasible Deontic Calculus for Resolving Norm Conflicts*](https://github.com/deepcausality-rs/deep_causality/blob/main/docs/papers/ddic.pdf), Northwestern University. The DDIC formalism. This is where the Effect Ethos comes from.
- The [Teleology preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/teleology_effect_propagation_process/epp_teleology.pdf) shows how DDIC is embedded into the Effect Propagation Process and the DeepCausality runtime.
