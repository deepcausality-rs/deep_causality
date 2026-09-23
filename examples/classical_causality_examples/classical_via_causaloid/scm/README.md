# EPP Example: Pearl's Ladder of Causation

This example models the three rungs of Judea Pearl's Ladder of Causation with the `DeepCausality` library, which implements the Effect Propagation Process (EPP). Each rung is a different level of causal reasoning.

Each rung has its own file:

- `rung1_association.rs`
- `rung2_intervention.rs`
- `rung3_counterfactual.rs`

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p classical_causality_examples --example scm_via_causaloid
```

---

### Rung 1: Association (Seeing)

- **File:** `rung1_association.rs`
- **Goal:** Observational inference. It answers the question: "Given that we observe X, what is the likelihood of Y?" (i.e., `P(Y|X)`).

#### EPP Implementation

Association is a plain evaluation of a `CausaloidGraph` that represents the assumed causal chain (`Smoking -> Tar -> Cancer`). An initial `PropagatingEffect` carries the observation (e.g., high nicotine levels), and the graph's evaluation propagates it to the associated outcome (cancer risk).

This matches **Rung 1**: the system processes passive observations to find associations within the model.

### Rung 2: Intervention (Doing)

- **File:** `rung2_intervention.rs`
- **Goal:** Taking an action based on an observation. It answers the question: "What would Y be if we *do* X?" (i.e., `P(Y|do(X))`).

#### EPP Implementation

Intervention acts on the result of the causal graph, in the style of a Causal State Machine (CSM):

1.  The `CausaloidGraph` is evaluated along the shortest path from smoking to cancer for a high-nicotine observation.
2.  If the result reports high cancer risk, the example fires the intervention (prescribing cessation therapy).

This matches **Rung 2**: the model moves from inference to a deterministic, real-world action.

### Rung 3: Counterfactuals (Imagining)

- **File:** `rung3_counterfactual.rs`
- **Goal:** Reasoning about alternate possibilities. It answers the retrospective question: "What would Y have been, had X been different?"

#### EPP Implementation

The EPP models counterfactuals through **Contextual Alternation**, leaving the causal model unchanged:

1.  A **Factual Context** represents the observed reality (e.g., a person who smokes and has high tar).
2.  A **Counterfactual Context** clones the factual one and modifies a past condition (e.g., sets the smoking level to low but leaves the tar level high).
3.  The *same* causal logic (the causal laws), bound into one contextual `Causaloid` per context, is evaluated against both contexts.

Even if the person had not smoked, their cancer risk stays high, because the direct consequence (tar) remains in the counterfactual world. Separating causal logic from its context lets the EPP reason about such alternative realities.

## Reference

For more on the EPP, see chapter 5 of the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf
