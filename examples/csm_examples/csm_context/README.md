# EPP Example: Server Monitoring with Sensor Fusion

This example monitors a server's health with the `DeepCausality` library by fusing data from three sensors:
- Fan Speed
- CPU Temperature
- Power Draw

A warning fires only when all three sensors report a "high" reading at once, which signals a risk of failure.

A single contextual `Causaloid` fuses the readings from a shared `Context`, and a `CausalState` triggers a `CausalAction` from the fused result.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p csm_examples --example csm_context_example
```

---

### How It Works: Mapping Concepts to EPP

1.  **Server State (`Context`):**
    - A `BaseContext`, shared as `Arc<RwLock<BaseContext>>`, holds the server's current state.
    - It holds three `Datoid`s, each storing the latest reading from one sensor.

2.  **Sensor Fusion (contextual `Causaloid`):**
    - One `Causaloid`, built with `Causaloid::new_with_context`, reads all three readings from the context.
    - It compares each reading with its "high" threshold (fan speed 80.0, CPU temperature 85.0, power draw 250.0) and returns `true` only if *all* three are high.

3.  **State-Based Action (`CSM`):**
    - A `CausalState` uses the fused `Causaloid` as its evaluation logic, so it becomes active only when all sensors are high.
    - A `CausalAction` prints a warning message to the console.
    - A `CSM` (Causal State Machine) links the "high load" state to the warning action.

4.  **Simulation:**
    - The `main` function runs 10 monitoring cycles. Each cycle writes the current readings into the context's `Datoid`s and evaluates the CSM state.
    - In normal cycles, not all readings are high, the `Causaloid` returns `false`, and the CSM does not fire the action.
    - In the high-load cycle, all readings are high, the `CausalState` becomes active, and the CSM fires the warning action.
