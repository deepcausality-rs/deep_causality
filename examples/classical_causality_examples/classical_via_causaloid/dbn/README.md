# EPP Example: Dynamic Bayesian Network (DBN)

This example models a simple Dynamic Bayesian Network (DBN) with the `DeepCausality` library, which implements the Effect Propagation Process (EPP). 

It models the "Umbrella World" scenario: the decision to take an umbrella today depends on whether it is raining, and the probability of rain today depends on whether it rained yesterday.

The EPP treats time and context as first-class, which fits temporal causal processes; Section 5.13 of the EPP documentation describes the principles.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p classical_causality_examples --example dbn_via_causaloid
```

---

### How It Works: Mapping DBN Concepts to EPP

A DBN models a temporal process by "unrolling" a causal graph over discrete time slices. The EPP evaluates a single, static causal model over a dynamic, temporal context instead.

1.  **Time Slices as a Dynamic Context:**
    Instead of creating new nodes for each time step (e.g., `Rain_t-1`, `Rain_t`), the EPP represents the timeline as a single, dynamic `Context`. Its `Datoid` nodes hold the state of variables (like `Rain`) at different points in time, and the simulation updates them as time moves forward.

2.  **State Variables as Causaloids:**
    Each DBN state variable (e.g., `Rain` and `Umbrella`) is a `Causaloid` whose `causal_fn` holds the variable's conditional probability table (CPT).
    -   The `rain_causaloid` implements `P(Rain_t | Rain_t-1)`. It reads the previous day's rain state from its input `WeatherState` and returns the probability of rain today.
    -   The `umbrella_causaloid` implements `P(Umbrella_t | Rain_t)`. It takes the probability of rain today as input and decides whether to take an umbrella.

3.  **Dependencies as a CausaloidGraph:**
    A `CausaloidGraph` holds the DBN's directed edges (the causal dependencies), here the chain `Rain -> Umbrella`.

4.  **Inference as Evaluation over Time (Filtering):**
    The DBN's "filtering" process (updating the belief state as new evidence arrives) is a loop over days. In each iteration:
    - The graph is evaluated from the `rain_causaloid` to get the probability of rain for the current day.
    - A random sample decides whether it actually rained (simulating a real-world observation).
    - The umbrella decision follows from the probability of rain.
    - The `Context` records today's rain state, and the next day's input carries it forward.

### Conclusion

The EPP offers an alternative to traditional DBNs. Moving time into a dynamic `Context` lets a static, reusable causal graph model temporal dependencies, which keeps the model simple.

## Reference

For more on the EPP, see chapter 5 of the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf