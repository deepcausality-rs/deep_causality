# Rubin Causal Model (RCM) Example in DeepCausality

This example demonstrates how to implement a simple Rubin Causal Model (RCM) scenario using the `DeepCausality` library. It showcases the Effect Propagation Process (EPP)'s capability for **Contextual Alternation** to directly compute potential outcomes and determine an individual treatment effect.

## Background: The Rubin Causal Model and EPP

The RCM defines a causal effect by comparing two "potential outcomes" for a single unit: the outcome if the unit receives a treatment (Y(1)) versus the outcome if it does not (Y(0)). The fundamental challenge is that only one of these outcomes can be observed in reality.

`DeepCausality`, as an implementation of the EPP, addresses this challenge by separating the causal model (the "science" or physiological response) from its context (the patient's state). This allows for the computational simulation of both potential outcomes:

1.  **Factual Context:** Represents the real-world state of a unit.
2.  **Counterfactual Contexts:** Cloned from the factual context and modified to represent hypothetical scenarios (e.g., drug administered vs. not administered).

By evaluating the *same causal model* against these different contexts, `DeepCausality` can compute both Y(1) and Y(0), thereby enabling the calculation of the Individual Treatment Effect (ITE).

## Example: Drug Effect on Blood Pressure

**Goal:** For a specific patient, determine the causal effect of a new drug on their blood pressure.

### 1. Define the Causal Model (The "Science")

*   **`drug_effect_causaloid`:** A `Causaloid` whose `causal_fn` determines the drug's effect (e.g., -10.0 BP reduction if administered, 0.0 otherwise). This causaloid expects a `PropagatingEffect::Map` as input, containing a flag for `drug_administered` and the `initial_blood_pressure`.
*   **`final_bp_causaloid`:** A `Causaloid` whose `causal_fn` calculates the final blood pressure by adding the `drug_effect` to the `initial_blood_pressure`. It also expects a `PropagatingEffect::Map` as input.
*   **`CausaloidGraph`:** A simple linear graph (`drug_effect_causaloid` -> `final_bp_causaloid`) representing the flow of calculation.

### 2. Define the Unit's Baseline State (The Patient)

*   A `BaseContext` is created to represent the patient's initial state, containing a `Datoid` for their `initial_blood_pressure` (e.g., 145.0).

### 3. Create the Potential Worlds (Contextual Alternation)

*   **`treatment_context`:** A clone of the `patient_baseline_context` with a `Datoid` indicating `drug_administered = 1.0` (representing `true`).
*   **`control_context`:** A clone of the `patient_baseline_context` with a `Datoid` indicating `drug_administered = 0.0` (representing `false`).

### 4. Simulate Both Potential Outcomes

*   The `CausaloidGraph` is evaluated twice, once with the `treatment_context` (to get Y(1)) and once with the `control_context` (to get Y(0)).
*   The `PropagatingEffect::Map` is used to pass all necessary contextual data (initial BP, drug administered status) through the causal graph during evaluation.

### 5. Calculate and Report the Causal Effect

*   The Individual Treatment Effect (ITE) is calculated as `Y(1) - Y(0)`.
*   The result is printed, demonstrating the drug's predicted effect on the patient's blood pressure.

## How to Run

To run this example, navigate to the root of the `deep_causality` project and execute:

```bash
cargo run -p example-rcm
```

## Reference

For more information on the EPP, please see chapter 5 in the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf