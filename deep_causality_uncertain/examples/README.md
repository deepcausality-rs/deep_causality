# Examples for `deep_causality_uncertain`

This directory contains various examples demonstrating the usage and capabilities of the `deep_causality_uncertain` crate.

To run any of these examples, navigate to the root of the `deep_causality` project and use the `cargo run --example` command.

## Available Examples

### 1. GPS Navigation (`gps_navigation.rs`)

This example simulates a GPS navigation scenario, propagating uncertainty through distance, speed, and time calculations. It showcases:

-   Modeling uncertain inputs (GPS coordinates, speed, traffic).
-   Arithmetic operations and non-linear transformations (`map`).
-   Statistical analysis (`expected_value`, `standard_deviation`).
-   Uncertain comparisons (`lt_uncertain`, `gt_uncertain`, `approx_eq`, `within_range`).
-   Conditional logic (`Uncertain::conditional`) for route planning.
-   Decision-making based on probabilities (`implicit_conditional`, `probability_exceeds`).

To run this example:

```bash
cargo run --example gps_navigation -p deep_causality_uncertain
```

### 2. Sensor Data Processing (`sensor_processing.rs`)

This example demonstrates robust sensor data processing with comprehensive error handling and uncertainty management. It covers:

-   Handling various sensor statuses (healthy, degraded, failed, etc.).
-   Dynamic adjustment of uncertainty based on sensor health.
-   Sensor fusion with uncertainty weighting.
-   Anomaly detection and handling.
-   Fallback strategies and graceful degradation.
-   Decision-making under uncertainty for system reliability and maintenance.

To run this example:

```bash
cargo run --example sensor_processing -p deep_causality_uncertain
```
