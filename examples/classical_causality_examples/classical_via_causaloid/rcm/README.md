# Rubin Causal Model (RCM) Example in DeepCausality

This example implements a simple Rubin Causal Model (RCM) scenario with the `DeepCausality` library. It computes both potential outcomes for one patient and derives the individual treatment effect.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p classical_causality_examples --example rcm_via_causaloid
```

## Background: The Rubin Causal Model and EPP

The RCM defines a causal effect by comparing two "potential outcomes" for a single unit: the outcome if the unit receives a treatment (Y(1)) and the outcome if it does not (Y(0)). In reality only one of them can be observed.

`DeepCausality` separates the causal model (the "science", here the physiological response) from the unit's state (the patient). The example builds two states that differ only in treatment assignment:

1.  **Treatment state:** the patient with the drug administered.
2.  **Control state:** the same patient without the drug.

Evaluating the *same causal model* on both states yields Y(1) and Y(0), and from them the Individual Treatment Effect (ITE).

## Example: Drug Effect on Blood Pressure

**Goal:** For one patient, determine the causal effect of a new drug on their blood pressure.

### 1. Define the Causal Model (The "Science")

*   **`drug_effect_causaloid`:** A `Causaloid` whose `causal_fn` sets the drug's effect (-10.0 BP if administered, 0.0 otherwise). Its input is an `RcmState` carrying the `drug_administered` flag and the `initial_bp`.
*   **`final_bp_causaloid`:** A `Causaloid` whose `causal_fn` computes the final blood pressure as `initial_bp + drug_effect`. It also takes an `RcmState`.
*   **`CausaloidGraph`:** A linear graph (`drug_effect_causaloid` -> `final_bp_causaloid`) for the flow of calculation.

### 2. Define the Unit's Baseline State (The Patient)

*   The patient's initial blood pressure is 145.0.

### 3. Create the Potential Worlds (Contextual Alternation)

*   **`treatment_state`:** an `RcmState` with the patient's baseline BP and `drug_administered: true`.
*   **`control_state`:** the same `RcmState` with `drug_administered: false`.

### 4. Simulate Both Potential Outcomes

*   The `CausaloidGraph` is evaluated twice, once with the `treatment_state` (Y(1)) and once with the `control_state` (Y(0)).
*   The `RcmState` carries all data the graph needs (initial BP, drug administered status) through the evaluation.

### 5. Calculate and Report the Causal Effect

*   The Individual Treatment Effect (ITE) is `Y(1) - Y(0)`.
*   The example prints the drug's predicted effect on the patient's blood pressure.

## Reference

For more on the EPP, see chapter 5 of the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf