# EPP Example: Dynamic Bayesian Network (DBN)

This crate demonstrates how the `DeepCausality` library, which implements the Effect Propagation Process (EPP), can model a simple Dynamic Bayesian Network (DBN). 

Specifically, this example models the classic "Umbrella World" scenario, where the decision to take an umbrella today depends on whether it is raining, and the probability of rain today depends on whether it rained yesterday.

This showcases how the EPP's architecture, with its first-class treatment of time and context, provides a natural and powerful framework for modeling temporal causal processes. This aligns with the principles outlined in Section 5.13 of the EPP documentation.

## How to Run

From within the `examples/epp_dbn` directory, run:

```bash
cargo run --bin example-dbn
```

---

### How It Works: Mapping DBN Concepts to EPP

A DBN models a temporal process by "unrolling" a causal graph over discrete time slices. The EPP achieves the same result by evaluating a single, static causal model over a dynamic, temporal context.

1.  **Time Slices as a Dynamic Context:**
    Instead of creating new nodes for each time step (e.g., `Rain_t-1`, `Rain_t`), the EPP represents the entire timeline as a single, dynamic `Context`. This context holds `Datoid` nodes representing the state of variables (like `Rain`) at different points in time. As the simulation progresses, this context is updated, representing the forward flow of time.

2.  **State Variables as Causaloids:**
    The state variables in the DBN (e.g., `Rain` and `Umbrella`) are represented as `Causaloid`s. Each `Causaloid` encapsulates the conditional probability table (CPT) for that variable as its `causal_fn`.
    -   The `rain_causaloid` implements `P(Rain_t | Rain_t-1)`. It queries the context to find the state of rain on the previous day to determine the probability of rain today.
    -   The `umbrella_causaloid` implements `P(Umbrella_t | Rain_t)`. It takes the probability of rain today as input and decides whether to take an umbrella.

3.  **Dependencies as a CausaloidGraph:**
    The directed edges in the DBN, representing causal dependencies, are modeled as a `CausaloidGraph`. In this case, the graph represents the simple chain: `Rain -> Umbrella`.

4.  **Inference as Evaluation over Time (Filtering):**
    The DBN's "filtering" process—updating the belief state as new evidence arrives—is modeled as a loop that simulates the passing of days. In each iteration:
    - The `rain_causaloid` is evaluated to determine the probability of rain for the current day.
    - A random sample is drawn to determine if it actually rained (simulating real-world observation).
    - The `umbrella_causaloid` is evaluated based on the probability of rain.
    - The `Context` is updated with the new state of rain, making it available for the next day's calculation.

### Conclusion

This example demonstrates that the EPP provides a flexible alternative to traditional DBNs. By externalizing time into a dynamic `Context`, the EPP allows for the modeling of complex temporal dependencies with a static and reusable causal graph. This separation of concerns simplifies the model and provides a clear and intuitive way to reason about causality in dynamic systems.

## Reference

For more information on the EPP, please see chapter 5 in the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf