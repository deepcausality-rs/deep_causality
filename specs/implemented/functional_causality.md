# Pre-Specification: Refactoring Counterfactuals to Monadic Interventions

- **Status**: Pre-Spec
- **Version**: 0.1.0
- **Authors**: Gemini
- **Date**: 2025-11-18

---

## 1. Objective

To introduce a simpler and more streamlined method for modeling common counterfactuals (Pearl's `do()` operator) within the DeepCausality framework by leveraging the newly introduced `intervene` method and monadic composition. This approach complements the existing context-based method, which remains valuable for advanced, large-scale counterfactual simulations.

## 2. Background

Currently, examples like `epp_granger` and `epp_scm` model counterfactuals by:
1.  Creating a "factual" `Context` object representing the observed reality.
2.  Cloning this context and modifying specific data points within the clone to create a "counterfactual" `Context`.
3.  Instantiating separate `Causaloid` objects (or re-assigning contexts to existing ones) to evaluate the same causal logic against these different contexts.

This context-based approach is powerful and necessary for advanced use cases, such as simulating a large number of alternate outcomes or complex environmental changes. However, for common, single-chain counterfactuals, the new `intervene` method, designed to directly implement the `do()` operator within a monadic chain, offers a more direct and intuitive way to express these operations.

## 3. Proposed Plan for Refactoring

The core idea is to introduce a simpler alternative for common counterfactual scenarios, shifting from "swapping out the world" (the `Context`) to "surgically altering the flow of events" (the monadic chain). The context-based approach remains valid for advanced use cases.

### 3.1. Elevate `intervene` to a Fluent API on `PropagatingEffect`

-   **What**: Add a new public method `intervene` directly to the `impl` block of `CausalPropagatingEffect`.
-   **How**: This new method, `pub fn intervene(self, new_value: Value) -> Self`, will mirror the existing `bind` method. It will take ownership of the effect (`self`) and internally call the static `CausalMonad::intervene(self, new_value)` function.
-   **Why**: This creates a fluent, chainable API (`effect.bind(...).intervene(...)`) which is crucial for making monadic composition feel natural and easy to use. Users will no longer need to call the static `CausalMonad` methods directly.

### 3.2. Refactor `Causaloid::evaluate` to use the Fluent API

-   **What**: Ensure that the `evaluate` method for `CausaloidType::Singleton` consistently uses the fluent API style for both `bind` and any future `intervene` calls within its internal chain.
-   **How**: The current implementation already uses `incoming_effect.clone().bind(...)`. This pattern will be maintained and extended if `Causaloid` itself needs to perform an intervention internally (though this is less likely for a basic `Singleton` evaluation).
-   **Why**: This makes the internal implementation of a `Causaloid` a direct demonstration of the power of monadic chaining, encouraging the same pattern in user-facing code.

### 3.3. Documenting the Two Approaches for Counterfactuals

-   **What**: Update relevant documentation (e.g., project `README.md`, `epp_scm` example's `README.md`) to clearly distinguish between the monadic intervention approach and the context-swapping approach for counterfactuals.
-   **How**: Frame the monadic intervention (`.intervene()`) as the recommended, simpler method for common, single-chain counterfactuals. Explain that the context-swapping method remains available and is suitable for more advanced scenarios, such as simulating a large number of alternate outcomes or complex environmental changes.
-   **Why**: This provides clear guidance to users on when to use each method, ensuring both simplicity for common cases and power for advanced ones.

### 3.4. Create a New Example for Side-by-Side Comparison

-   **What**: Create a *new, separate* example (`examples/epp_scm_functional/`) that exclusively demonstrates the monadic intervention pattern for counterfactuals.
-   **How**: This new example will showcase the `.bind().intervene().bind()` chain, illustrating its simplicity and directness for common counterfactual scenarios. It will be designed to serve as a side-by-side comparison with the existing `epp_scm` example, highlighting the differences and benefits of the monadic approach. Existing examples (`epp_granger`, `epp_scm`) will *not* be rewritten, as they serve to demonstrate the advanced context-swapping approach.
-   **Why**: This provides a clear, working blueprint for users to follow for the common case, without removing valuable examples of the advanced method, and offers a direct comparison to aid understanding.

## 4. New Example: Simplified Structural Causal Model (SCM) with Monadic Intervention

This example replaces the complex context-switching of the original `epp_scm` example with a direct, easy-to-read monadic chain.

**File: `examples/epp_scm_functional/src/main.rs`** (Conceptual)

```rust
use deep_causality::{CausalMonad, EffectValue, Intervenable, PropagatingEffect};

// --- 1. Define Causal Logic as Simple Functions ---
// Each function represents a structural equation in an SCM.
// They take a value and return a new PropagatingEffect.

// f(U_smoking) -> Smoking
fn smoking_logic(nicotine_obs: EffectValue) -> PropagatingEffect {
    let nicotine_level = nicotine_obs.as_numerical().unwrap_or(&0.0);
    let is_smoking = *nicotine_level > 0.6;
    // The function returns a new effect, starting its own log.
    CausalMonad::pure(EffectValue::Boolean(is_smoking))
}

// f(Smoking, U_tar) -> Tar
fn tar_logic(is_smoking: EffectValue) -> PropagatingEffect {
    let has_tar = is_smoking.as_bool().unwrap_or(false);
    CausalMonad::pure(EffectValue::Boolean(has_tar))
}

// f(Tar, U_cancer) -> Cancer
fn cancer_logic(has_tar: EffectValue) -> PropagatingEffect {
    let has_cancer_risk = has_tar.as_bool().unwrap_or(false);
    CausalMonad::pure(EffectValue::Boolean(has_cancer_risk))
}


fn main() {
    println!("--- Simplified SCM with Monadic Composition ---");

    // --- 2. Factual / Observational Case ---
    println!("\n[Scenario 1: Factual Observation]");
    println!("Query: Given a person with high nicotine levels, what is their cancer risk?");

    // Start the chain with an observation.
    let observation = CausalMonad::pure(EffectValue::Numerical(0.8));

    // Chain the causal functions together using the fluent `bind` method.
    let factual_effect = observation
        .bind(smoking_logic)
        .bind(tar_logic)
        .bind(cancer_logic);

    println!("Result: High cancer risk -> {}", factual_effect.value.as_bool().unwrap());
    println!("Explanation:\n{}", factual_effect.explain());


    // --- 3. Counterfactual / Interventional Case ---
    println!("\n[Scenario 2: Counterfactual Intervention]");
    println!("Query: Given the same person, what if we could magically remove tar from their lungs?");

    let observation_2 = CausalMonad::pure(EffectValue::Numerical(0.8));

    // Build the same chain, but use `intervene` to apply the `do()` operator.
    let counterfactual_effect = observation_2
        .bind(smoking_logic) // -> is_smoking = true
        .bind(tar_logic)     // -> has_tar = true
        // DO-OPERATOR: Regardless of the previous result, force the value.
        .intervene(EffectValue::Boolean(false)) // do(Tar = false)
        .bind(cancer_logic); // This function now receives `false`.

    println!("Result: Low cancer risk -> {}", !counterfactual_effect.value.as_bool().unwrap());
    println!("Explanation:\n{}", counterfactual_effect.explain());

    println!("Conclusion: The monadic chain allows for direct, intuitive intervention, showing that if tar is removed, cancer risk is low, fulfilling Pearl's `do`-calculus.");
}
```

## 5. Why This is Better

-   **Simplicity & Readability**: The causal model is now a linear chain of function calls, which is vastly easier to read and understand than creating and managing multiple `Context` objects.
-   **Directness**: The `.intervene()` call is a direct, explicit representation of a `do()` operation. It clearly communicates the intent: "at this point in the chain, force the value to be X".
-   **Decoupling**: The causal logic (the functions) is now completely decoupled from the data (the `EffectValue` flowing through the monad). The logic doesn't need to know about `Context` at all, making it more pure and reusable.
-   **Full Provenance**: The `explain()` output on the `counterfactual_effect` would automatically include the log entry "Intervention: Value forced to Boolean(false)", providing a perfect audit trail of the counterfactual reasoning.
