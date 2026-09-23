# EPP Example: Granger Causality

This example runs a Granger Causality test with the `DeepCausality` library, which implements the Effect Propagation Process (EPP). 

It answers the question: **"Do past changes in oil prices Granger-cause future changes in shipping activity?"**

A Granger test is a counterfactual comparison, which the EPP expresses directly; Section 5.14 of the EPP documentation describes the principles.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p classical_causality_examples --example granger_via_causaloid
```

---

### How It Works: Mapping Granger Causality to EPP Concepts

Granger Causality asks whether one time series helps forecast another. The EPP compares the predictive accuracy of a causal model under two contexts: one with the complete history (factual) and one without the history of the candidate cause (counterfactual).

1.  **Causal Logic (`shipping_predictor_logic`):**
    One reusable function holds the predictive model. It is a `ContextualCausalFn`, which can inspect the context it is evaluated against, and it predicts the next value of shipping activity from the historical data in that context. It uses both shipping and oil price data if available and falls back to shipping data alone if oil price data is missing.

2.  **Factual vs. Counterfactual Contexts:**
    The Granger test compares two realities:

    *   **Factual Context:** A `BaseContext` with the complete, observed history of *both* oil prices and shipping activity.
    *   **Counterfactual Context:** A second `BaseContext` with the history of shipping activity but *without* the history of oil prices.

3.  **Evaluating Potential Outcomes:**
    Two `Causaloid`s share the same predictive logic, each bound to a different context:

    *   The **factual causaloid** runs against the factual context, so the history of oil prices informs its prediction.
    *   The **counterfactual causaloid** runs against the counterfactual context, so its prediction ignores the history of oil prices.

4.  **Comparing Prediction Errors:**
    Both prediction errors are measured against a known, actual outcome. If the factual error (with oil prices) is significantly lower than the counterfactual error, the oil price series carries information that predicts shipping activity: oil prices Granger-cause shipping activity.

### Conclusion

The Effect Propagation Process separates **causal logic** (the `Causaloid`) from the **state of the world** (the `Context`). A Granger test then reduces to creating alternate contexts and evaluating the same immutable causal laws against them.


## Reference

For more on the EPP, see chapter 5 of the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf
