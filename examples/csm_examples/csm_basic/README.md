# EPP Example: Causal State Machine (CSM)

This crate demonstrates how the `DeepCausality` library, which implements the Effect Propagation Process (EPP), can be used to build a Causal State Machine (CSM). 

Specifically, this example models a simple industrial monitoring system with three sensors: smoke, fire, and explosion. Each sensor is represented by a `CausalState` that, when its conditions are met, triggers a corresponding `CausalAction` (e.g., raising an alert).

This showcases how the EPP's architecture provides a formal bridge between causal reasoning and deterministic intervention, a concept that aligns with Rung 2 (Intervention) of Pearl's Ladder of Causation.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p csm_examples --example csm_example
```

---

### How It Works: Mapping CSM Concepts to EPP

The CSM provides a mechanism to link causal inferences to real-world actions. It is a collection of state-action pairs, where each state's activation is determined by a causal model.

1.  **Causal Logic as `Causaloid`s:**
    The trigger condition for each sensor is encapsulated in a `Causaloid`. For example, the `smoke_sensor_causaloid` contains a simple `causal_fn` that checks if an incoming numerical value (the sensor reading) exceeds a predefined threshold (e.g., 65.0).

2.  **States as `CausalState`s:**
    Each sensor in the system is represented by a `CausalState`. The `CausalState` struct holds a reference to the `Causaloid` that defines its logic. For instance, the `smoke_cs` holds the `smoke_sensor_causaloid`. When the CSM evaluates this state, it uses the causaloid to determine if the state is active.

3.  **Actions as `CausalAction`s:**
    Each potential intervention is defined as a `CausalAction`. This struct wraps a function that will be executed when the action is fired. In this example, the actions (`get_smoke_alert_action`, `get_fire_alert_action`, etc.) simply print a message to the console, but they could just as easily trigger an API call, send an email, or control a physical device.

4.  **The `CSM` as an Orchestrator:**
    The `CSM` is initialized with a collection of state-action pairs. Its primary role is to orchestrate the evaluation process. The `main` loop simulates a stream of sensor data. In each iteration:
    - A `PropagatingEffect::Numerical` is created from the raw sensor data.
    - `csm.eval_single_state()` is called for each sensor.
    - The CSM finds the corresponding `CausalState`, evaluates its `Causaloid` against the provided data, and if the result is `Deterministic(true)`, it automatically calls the `fire()` method on the associated `CausalAction`.

### Conclusion

This example demonstrates how the CSM acts as a powerful bridge between the abstract world of causal reasoning and the concrete world of action and intervention. By formally linking `CausalState`s (defined by `Causaloid`s) to `CausalAction`s, the EPP provides a robust, auditable, and deterministic way to build systems that not only understand cause and effect but can also act on that understanding.


## Reference

For more information on the EPP, please see chapter 5 in the EPP document:
https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf
