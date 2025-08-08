# EPP Example: Granger Causality

This crate demonstrates how the `DeepCausality` library, which implements the Effect Propagation Process (EPP), can model a Granger Causality test. 

Specifically, this example answers the question: **"Do past changes in oil prices Granger-cause future changes in shipping activity?"**

This showcases how the EPP's architecture naturally handles the counterfactual reasoning inherent in a Granger test. This aligns with the principles outlined in Section 5.14 of the EPP documentation.

## How to Run

From within the `examples/epp_granger` directory, run:

```bash
cargo run --bin example-granger
```

---

### How It Works: Mapping Granger Causality to EPP Concepts

The core idea behind Granger Causality is to determine if one time series is useful in forecasting another. The EPP models this by comparing the predictive accuracy of a causal model under two different contexts: one with the complete history (factual) and one where the history of the potential causal variable has been removed (counterfactual).

1.  **Causal Logic (`shipping_predictor_logic`):**
    The predictive model is encapsulated in a single, reusable `Causaloid`. This causaloid uses a `ContextualCausalFn`, a function that can inspect the context it's evaluated against. Its logic is to predict the next value of shipping activity based on the historical data it finds in its context. It will use both shipping and oil price data if available, but will gracefully fall back to using only shipping data if oil price data is missing.

2.  **Factual vs. Counterfactual Contexts:**
    This is the key to the Granger test. We create two distinct realities:

    *   **Factual Context:** A `BaseContext` is created containing the complete, observed history of *both* oil prices and shipping activity.
    *   **Counterfactual Context:** A second `BaseContext` is created that contains the history of shipping activity but is *missing* the history of oil prices.

3.  **Evaluating Potential Outcomes:**
    We instantiate two separate `Causaloid`s, each with the same predictive logic but associated with a different context:

    *   The **factual causaloid** is evaluated against the factual context. Its prediction will be informed by the history of oil prices.
    *   The **counterfactual causaloid** is evaluated against the counterfactual context. Its prediction will *not* be informed by the history of oil prices.

4.  **Comparing Prediction Errors:**
    We compare the prediction error from both evaluations against a known, actual outcome. If the error from the factual evaluation (which included oil prices) is significantly lower than the error from the counterfactual evaluation, we can conclude that the oil price time series provides valuable information for predicting the shipping activity time series. In other words, oil prices Granger-cause shipping activity.

### Conclusion

This example highlights a core strength of the Effect Propagation Process: the explicit separation of **causal logic** (the `Causaloid`) from the **state of the world** (the `Context`). This separation makes it trivial to perform the powerful counterfactual reasoning required for a Granger Causality test by simply creating alternate contexts and evaluating the same immutable causal laws against them.


## Reference

For more information on the EPP, please see chapter 5 in the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf
