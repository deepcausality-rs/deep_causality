# Functional SCM Example: The Ladder of Causation

This example demonstrates how the `CausalMonad` can be used to climb all three rungs of Judea Pearl's Ladder of Causation using a simple Structural Causal Model (SCM) implemented with functional composition.

## The Causal Model

The example uses a simple SCM where smoking causes tar in the lungs, and tar can cause cancer. It also includes a hidden background variable (an exogenous factor): a genetic predisposition to cancer.

The causal links are defined as simple Rust functions:
1.  `smoking_logic`: `Nicotine Level -> Smoking`
2.  `tar_logic`: `Smoking -> Tar`
3.  `cancer_logic`: `(Tar, GeneticPredisposition) -> Cancer`

## Climbing the Ladder

The `main.rs` file is structured to demonstrate each of the three rungs of causal reasoning.

### Rung 1: Association (Seeing)

*   **Question:** "Given a person has high nicotine levels, what is their cancer risk?"
*   **Concept:** This is passive observation. We are seeing what the model associates with high nicotine levels by propagating the initial observation through an uninterrupted chain of `bind` calls.

```rust
// We assume we don't know the background genetic factor for this observation.
let has_genetic_predisposition_rung1 = false;
let observation_rung1 = CausalMonad::pure(EffectValue::Numerical(0.8));

let factual_effect = observation_rung1
    .bind(model::smoking_logic)
    .bind(model::tar_logic)
    .bind(|has_tar| model::cancer_logic(has_tar, has_genetic_predisposition_rung1));
```

### Rung 2: Intervention (Doing)

*   **Question:** "What would the general cancer risk be if we forced everyone to stop smoking?"
*   **Concept:** This is a predictive, forward-looking question about the effect of an action. We use the `intervene()` method to force a value in the causal chain, representing Pearl's `do()` operator.

```rust
let observation_rung2 = CausalMonad::pure(EffectValue::Numerical(0.8));

let interventional_effect = observation_rung2
    .bind(model::smoking_logic)
    // DO-OPERATOR: Force the value to `false`, regardless of the nicotine level.
    .intervene(EffectValue::Boolean(false)) // do(Smoking = false)
    .bind(model::tar_logic)
    .bind(|has_tar| model::cancer_logic(has_tar, has_genetic_predisposition_rung2));
```

### Rung 3: Counterfactual (Imagining)

*   **Question:** "We observed a patient has cancer and was a smoker. Would they still have had cancer if they had not smoked?"
*   **Concept:** This is a retrospective question that requires using evidence from the real world to reason about a hypothetical past. It involves a three-step process:
    1.  **Abduction:** We use the observed facts (Smoker=true, Cancer=true) to infer the state of unobserved background variables. In this case, we infer the patient must have a `genetic_predisposition`.
    2.  **Action:** We use `intervene()` to change the past action (e.g., set `smoking` to `false`).
    3.  **Prediction:** We run the rest of the causal chain forward, but critically, we use the inferred background state from the abduction step.

```rust
// 1. Abduction: Infer the background state from real-world observations.
let inferred_genetic_predisposition = true;

// 2. Action & 3. Prediction
let counterfactual_effect = CausalMonad::pure(EffectValue::Numerical(0.8))
    .bind(model::smoking_logic)
    // INTERVENE on the past action.
    .intervene(EffectValue::Boolean(false)) // If they had not smoked...
    .bind(model::tar_logic)
    // ...but use the inferred background condition from the real world.
    .bind(|has_tar| model::cancer_logic(has_tar, inferred_genetic_predisposition));
```

## Conclusion

This monadic approach provides a robust, transparent, and composable Causal Algebra that simplifies causality.

1.  **From Specialized Models to Composable Functions:** Traditional computational causality often relies on building large, monolithic statistical models (e.g., Bayesian Networks). This monadic approach reframes the problem. Each causal link becomes a small, independent, testable function. The overall causal model is then built by composing these functions together in a chain. This is a paradigm shift from declarative modeling to functional composition.

2.  **Built-in Explainability and Provenance:** A major challenge in complex causal models is understanding *why* a certain output was produced. The `CausalMonad`, by carrying a `CausalEffectLog` through the entire computation, solves this directly. Every `bind` and `intervene` step automatically adds to this log, providing a complete, step-by-step audit trail of the reasoning process. This makes the results transparent and explainable, which is critical for building trustworthy AI systems.

3.  **Unification of Causal Queries:** In many systems, observation (`P(Y|X)`) and intervention (`P(Y|do(X))`) are handled by different mechanisms (e.g., standard inference vs. graph surgery). The monadic framework unifies them. Both are just chains of computation. The only difference is whether an `.intervene()` call is present. This provides a single, coherent API and mental model for all types of causal queries, simplifying both development and understanding.

4.  **Type Safety and Reliability:** By leveraging Rust's strong type system and the Higher-Kinded Type patterns from `deep_causality_haft`, the framework provides compile-time guarantees about the flow of causal effects. This prevents entire classes of errors that might occur in dynamically typed languages or systems where the data flow is less constrained, making the causal models more robust and reliable for production-grade applications.


## How to Run

From the root of the `deep_causality` repository, run:
```bash
cargo run -p example-scm-functional
```
