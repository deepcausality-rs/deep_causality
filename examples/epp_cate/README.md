# EPP Example: Conditional Average Treatment Effect (CATE)

This crate demonstrates how the `DeepCausality` library, which implements the Effect Propagation Process (EPP), can model and calculate the Conditional Average Treatment Effect (CATE). 

Specifically, this example answers the question: **"What is the average effect of a new medication on blood pressure, specifically for the subgroup of patients over the age of 65?"**

This showcases how the EPP's architecture naturally handles concepts from the Rubin Causal Model (RCM), such as potential outcomes, through its powerful contextual reasoning capabilities. This aligns with the principles outlined in Section 5.16 of the EPP documentation.

## How to Run

From within the `examples/epp_cate` directory, run:

```bash
cargo run --bin example-cate
```

---

### How It Works: Mapping CATE to EPP Concepts

The core idea behind CATE is to estimate the average effect of a treatment for a specific subset of a population. The EPP achieves this through the following mechanisms:

1.  **Causal Logic (`drug_effect_logic`):**
    The fundamental effect of the drug is encapsulated in a single, reusable `Causaloid`. This causaloid uses a `ContextualCausalFn`, a function that can inspect the context it's evaluated against. Its logic is simple: if it finds a `drug_administered` flag in its context, it returns a numerical effect (e.g., -10.0 for a 10-point drop in blood pressure); otherwise, it returns zero.

2.  **Population and Subgroup Selection:**
    The entire patient population is represented as a `Vec<BaseContext>`, where each `Context` is a self-contained representation of an individual, holding their specific attributes (like `age` and `initial_blood_pressure`) as `Datoid`s. We create our subgroup of interest by simply filtering this vector to include only the contexts of patients older than 65.

3.  **Potential Outcomes via Contextual Alternation:**
    This is the key step. To calculate the Individual Treatment Effect (ITE) for each person in the subgroup, we must simulate two parallel realities: one where they received the drug, and one where they didn't. The EPP models this cleanly using **Contextual Alternation**:

    *   **Treatment Context (`Y(1)`):** For each patient, we clone their original context and *add* a `Datoid` indicating the drug was administered. Evaluating our `Causaloid` against this context yields the *potential outcome under treatment*.
    *   **Control Context (`Y(0)`):** We clone the patient's context again, this time adding a `Datoid` indicating the drug was *not* administered. Evaluating the *exact same* `Causaloid` against this second context yields the *potential outcome under control*.

4.  **Aggregation to CATE:**
    By subtracting the control outcome from the treatment outcome (`Y(1) - Y(0)`), we get the ITE for each individual. The CATE for the subgroup is then simply the average of all these ITEs.

### Conclusion

This example highlights a core strength of the Effect Propagation Process: the explicit separation of **causal logic** (the `Causaloid`) from the **state of the world** (the `Context`). This separation makes it trivial to perform powerful counterfactual reasoning by creating alternate contexts and evaluating the same immutable causal laws against them, providing a robust foundation for advanced causal inference.

## Reference

For more information on the EPP, please see chapter 5 in the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf