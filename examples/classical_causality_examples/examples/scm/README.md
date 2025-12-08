# EPP Example: Pearl's Ladder of Causation

This crate demonstrates how the `DeepCausality` library, which implements the Effect Propagation Process (EPP), models the three rungs of Judea Pearl's Ladder of Causation. Each rung represents a different level of causal reasoning, and this example shows how the EPP's unique architecture addresses each one.

The examples are separated into three files, each corresponding to a rung on the ladder:

- `rung1_association.rs`
- `rung2_intervention.rs`
- `rung3_counterfactual.rs`

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p classical_causality_examples --example scm_example
```

---

### Rung 1: Association (Seeing)

- **File:** `rung1_association.rs`
- **Goal:** Demonstrates simple observational inference. It answers the question: "Given that we observe X, what is the likelihood of Y?" (i.e., `P(Y|X)`).

#### EPP Implementation

Association is modeled as a straightforward evaluation of a `CausaloidGraph`. The graph represents the assumed causal chain (`Smoking -> Tar -> Cancer`). We provide an initial `PropagatingEffect` representing the observation (e.g., high nicotine levels), and the graph's evaluation propagates this effect to determine the associated outcome (cancer risk).

This aligns with **Rung 1** by showing how the system processes passive observations to find statistical associations within the model.

### Rung 2: Intervention (Doing)

- **File:** `rung2_intervention.rs`
- **Goal:** Demonstrates taking an action based on an observation. It answers the question: "What would Y be if we *do* X?" (i.e., `P(Y|do(X))`).

#### EPP Implementation

Intervention is modeled using the **Causal State Machine (CSM)**. 

1.  A `CausalState` is defined, with its condition for activation being the result of the causal graph (e.g., "High Cancer Risk" is true).
2.  A `CausalAction` is defined, which represents the real-world intervention (e.g., prescribing therapy).
3.  The CSM links the state to the action. When the CSM is evaluated with an effect that makes the state true, it automatically fires the action.

This aligns with **Rung 2** by providing a formal mechanism to move from inference to a deterministic, real-world action.

### Rung 3: Counterfactuals (Imagining)

- **File:** `rung3_counterfactual.rs`
- **Goal:** Demonstrates reasoning about alternate possibilities. It answers the retrospective question: "What would Y have been, had X been different?"

#### EPP Implementation

The EPP models counterfactuals not by surgically altering the causal model itself, but through **Contextual Alternation**.

1.  A **Factual Context** is created to represent the observed reality (e.g., a person who smokes and has high tar).
2.  A **Counterfactual Context** is created by cloning the factual one and then modifying a specific past condition (e.g., setting the smoking level to low, but leaving the tar level high).
3.  The *exact same* `Causaloid` (representing the causal laws) is evaluated against both contexts.

This example shows that even if we imagine the person had not smoked, their cancer risk remains high because the direct consequence (tar) is still present in the counterfactual world. This demonstrates the EPP's powerful ability to reason about alternative realities by separating causal logic from the context it operates on.

## Reference

For more information on the EPP, please see chapter 5 in the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf
