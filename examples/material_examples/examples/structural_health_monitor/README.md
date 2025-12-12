# High-Stakes Decentralized Structural Health Monitoring

A demonstration of **Autonomous Safety Interventions** using DeepCausality's `Intervenable` trait for critical infrastructure.

## Overview

In extreme environments (Space Stations, Deep-Sea Habitats), structural failures propagate faster than signals can travel to a central server. This example demonstrates a **Decentralized Monitoring System** where:

1.  Each hull plate is an **autonomous agent** (node in a `Graph`).
2.  Local sensors detect stress levels.
3.  When stress exceeds a threshold, the plate **locally intervenes** using the `Intervenable` trait.
4.  The intervention is recorded in a formal **Causal Audit Trail** (Blackbox).

## The Physics

*   **Stress-Strain Relationship**: $\sigma = E \cdot \epsilon$ (Hooke's Law)
*   **Failure Cascade**: When one plate fails, its load redistributes to neighbors, potentially triggering a chain reaction.
*   **Active Dampening**: Smart materials can increase their stiffness (Modulus $E$) in response to overload, absorbing energy and preventing rupture.

## Key Concepts

### `Graph<T>` (Topology)
The hull is modeled as a graph where:
*   **Nodes** = Hull plates (sensors)
*   **Edges** = Structural bonds (load paths)

```rust
let mut hull = Graph::new(num_plates, tensor, 0)?;
hull.add_edge(0, 1)?; // Plate 0 connected to Plate 1
```

### `PropagatingEffect<T>` & `Intervenable`
The stress state is wrapped in a monadic effect. This allows:
1.  **Observation** (Layer 1): Reading the current stress.
2.  **Intervention** (Layer 2): Overriding the stress value to a safe level.
3.  **Counterfactual** (Layer 3): Comparing "what happened" vs "what would have happened".

```rust
// Wrap state in monadic container
let stress_effect = PropagatingEffect::pure(Some(current_stress));

// INTERVENE: Override the dangerous value
let healed_effect = stress_effect.intervene(Some(SAFE_STRESS_LIMIT));
```

### Decentralized Decision Making
*   **Problem**: Central server is 2.4 seconds away. Failure cascade completes in 0.3 seconds.
*   **Solution**: Each node makes its own decision based on local causal rules.
*   **Audit**: All interventions are logged for post-incident analysis.

## Run Command

```bash
cargo run -p material_examples --example structural_health_monitor_example
```

## Expected Output

```text
[1] Initializing Hull Topology...
    Created 6 plates with 8 structural bonds.

[2] Simulating Micrometeoroid Impact on Plate 2...
    Impact Stress: 225.0 MPa
    Warning Threshold: 200.0 MPa

[3] Running Decentralized Monitoring Loop...
    [WARNING] Stress exceeds safety threshold!
    [ACTION]  Initiating LOCAL autonomous intervention...

    > [BLACKBOX AUDIT]: Autonomous Intervention Recorded.
    > Original Stress:  225.0 MPa
    > Intervened Stress: 100.0 MPa

    [SUCCESS] Plate 2 STABLE. Catastrophic failure AVERTED.

[4] Counterfactual Analysis: No Intervention Scenario...
    If Plate 2 had failed (no intervention):
    -> Neighbor stress: 258.8 MPa (>250.0 MPa limit)
    -> [CATASTROPHIC FAILURE] Complete hull breach.
```

## Engineering Value

| Aspect | Without DeepCausality | With DeepCausality |
|--------|----------------------|-------------------|
| **Decision Latency** | 2.4s (round-trip to server) | 0.001s (local) |
| **Fault Tolerance** | Central point of failure | Fully distributed |
| **Audit Trail** | Ad-hoc logging | Formal causal history |
| **Counterfactual Analysis** | N/A | Built-in |
