# CATE Example: Conditional Average Treatment Effect

This example demonstrates how to calculate the **Conditional Average Treatment Effect (CATE)** using the `DeepCausality` library.

Specifically, it models the effect of a medication on blood pressure for a specific subgroup: **patients over 65 years old**.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p classical_causality_examples --example cate_example
```

---

## Key Concepts

### What is CATE?

The Conditional Average Treatment Effect is the average causal effect of a treatment for a specific subgroup of the population. Unlike the Average Treatment Effect (ATE), which considers the entire population, CATE focuses on individuals who share certain characteristics.

### How It Works

1. **Patient Population**: A population of patients is created, each with attributes like age and initial blood pressure stored in a `BaseContext`.

2. **Subgroup Selection**: Patients over 65 are filtered to form the target subgroup.

3. **Counterfactual Contexts**: For each patient in the subgroup, two alternate realities are created:
   - **Treatment Context**: Drug is administered (`drug_administered = 1.0`)
   - **Control Context**: No drug (`drug_administered = 0.0`)

4. **Causaloid Evaluation**: The same causal logic (`drug_effect_logic`) is evaluated against both contexts to compute:
   - `Y(1)`: Potential outcome with treatment
   - `Y(0)`: Potential outcome without treatment

5. **Individual Treatment Effect (ITE)**: For each patient: `ITE = Y(1) - Y(0)`

6. **CATE Calculation**: The average of all ITEs in the subgroup gives the CATE.

### EPP Principle

This example demonstrates the EPP's power of **Contextual Alternation** - the same causal model can be evaluated against different contexts to simulate potential outcomes, enabling counterfactual reasoning without modifying the underlying causal laws.

## Reference

For more information on the EPP, please see chapter 5 in the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf
