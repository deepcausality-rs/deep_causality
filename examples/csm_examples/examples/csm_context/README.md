# EPP Example: Server Monitoring with Sensor Fusion

This crate provides a beginner-friendly example of using the `DeepCausality` library to monitor a server's health by fusing data from multiple sensors.

Specifically, this example models a simple server with three sensors:
- Fan Speed
- CPU Temperature
- Power Draw

A warning is triggered only when all three sensors report a "high" reading simultaneously, indicating a potential risk of failure.

This showcases how the EPP's `CausaloidCollection` can be used for sensor fusion, and how a `CausalState` can trigger an `CausalAction` based on the fused data.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p csm_examples --example csm_context_example
```

---

### How It Works: Mapping Concepts to EPP

1.  **Individual Sensor Logic (`Causaloid`s):**
    - The logic for each sensor is encapsulated in a simple `Causaloid`.
    - For example, the `cpu_temp_causaloid` checks if a numerical input (the temperature reading) exceeds a predefined "high" threshold.

2.  **Sensor Fusion (`CausaloidCollection`):
    - The three sensor `Causaloid`s are grouped into a `CausaloidCollection`.
    - The collection's `AggregateLogic` is set to `All`, meaning the collection as a whole will only evaluate to `true` if *all* of its contained `Causaloid`s evaluate to `true`.

3.  **Server State (`Context`):
    - A `BaseContext` is used to represent the server's current state.
    - It holds three `Datoid`s, each storing the latest reading from one of the sensors.

4.  **State-Based Action (`CSM`):
    - A `CausalState` is defined, using the `CausaloidCollection` as its evaluation logic. This state becomes active only when all sensors are high.
    - A `CausalAction` is defined to print a warning message to the console.
    - A `CSM` (Causal State Machine) links the "high load" state to the warning action.

5.  **Simulation:
    - The `main` function simulates two scenarios by creating a `PropagatingEffect::Map` with different sensor readings.
    - In the "Normal Load" scenario, not all readings are high, so the `CausaloidCollection` evaluates to `false`, and the CSM does not fire the action.
    - In the "High Load" scenario, all readings are high, the collection evaluates to `true`, the `CausalState` becomes active, and the CSM fires the warning action.
