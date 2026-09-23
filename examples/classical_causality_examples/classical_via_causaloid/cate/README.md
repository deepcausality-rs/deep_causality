# CATE Example: Conditional Average Treatment Effect

This example computes the **Conditional Average Treatment Effect (CATE)** with the `DeepCausality` library: the effect of a medication on blood pressure for **patients over 65 years old**.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p classical_causality_examples --example cate_via_causaloid
```

---

## Key Concepts

### What is CATE?

The Conditional Average Treatment Effect is the average causal effect of a treatment on a subgroup of the population. The Average Treatment Effect (ATE) covers the entire population; CATE covers individuals who share certain characteristics.

### How It Works

1. **Patient Population**: Each patient's attributes, such as age and initial blood pressure, are stored in a `BaseContext`.

2. **Subgroup Selection**: A filter keeps patients over 65.

3. **Counterfactual Contexts**: For each patient in the subgroup, the example clones the context into two alternate realities:
   - **Treatment Context**: Drug is administered (`drug_administered = 1.0`)
   - **Control Context**: No drug (`drug_administered = 0.0`)

4. **Causaloid Evaluation**: The same causal logic (`drug_effect_logic`) runs against both contexts to compute:
   - `Y(1)`: Potential outcome with treatment
   - `Y(0)`: Potential outcome without treatment

5. **Individual Treatment Effect (ITE)**: For each patient: `ITE = Y(1) - Y(0)`

6. **CATE Calculation**: The CATE is the average of all ITEs in the subgroup.

### EPP Principle

The example applies the EPP's **Contextual Alternation**: the same causal model runs against different contexts to simulate potential outcomes, so counterfactual reasoning leaves the causal laws unchanged.

## Reference

For more on the EPP, see chapter 5 of the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf
