# EPP Example: Causal State Machine (CSM)

This example builds a Causal State Machine (CSM) with the `DeepCausality` library, which implements the Effect Propagation Process (EPP). 

It models an industrial monitoring system with three sensors: smoke, fire, and explosion. Each sensor is a `CausalState` that triggers a `CausalAction` (e.g., raising an alert) when its condition is met.

The CSM links causal reasoning to deterministic intervention, which corresponds to Rung 2 (Intervention) of Pearl's Ladder of Causation.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p csm_examples --example csm_example
```

---

### How It Works: Mapping CSM Concepts to EPP

The CSM links causal inferences to real-world actions. It is a collection of state-action pairs, and a causal model decides whether each state is active.

1.  **Causal Logic as `Causaloid`s:**
    A `Causaloid` holds each sensor's trigger condition. For example, the smoke sensor's `causal_fn` checks whether the incoming sensor reading reaches a threshold (65.0).

2.  **States as `CausalState`s:**
    Each sensor is a `CausalState` that holds the `Causaloid` defining its logic; `smoke_cs`, for instance, holds the smoke sensor causaloid. When the CSM evaluates the state, the causaloid decides whether it is active.

3.  **Actions as `CausalAction`s:**
    Each intervention is a `CausalAction` that wraps a function to run when the action fires. Here the actions (`get_smoke_alert_action`, `get_fire_alert_action`, etc.) print a message; they could equally trigger an API call, send an email, or control a physical device.

4.  **The `CSM` as an Orchestrator:**
    The `CSM` starts with a collection of state-action pairs (the explosion sensor is added afterwards with `add_single_state`) and orchestrates evaluation. The `main` loop simulates a stream of sensor data. In each iteration:
    - Each raw sensor reading is wrapped with `PropagatingEffect::pure`.
    - `csm.eval_single_state()` is called for each sensor.
    - The CSM finds the corresponding `CausalState`, evaluates its `Causaloid` against the data, and, if the state is active, fires the associated `CausalAction`.

### Conclusion

The CSM connects causal reasoning to action. By linking `CausalState`s (defined by `Causaloid`s) to `CausalAction`s, the EPP gives an auditable, deterministic way to build systems that act on cause and effect.


## Reference

For more on the EPP, see chapter 5 of the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf
