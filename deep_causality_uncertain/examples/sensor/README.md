# Sensor Data Processing Example

This example demonstrates how to build a robust sensor data processing pipeline using the `deep_causality_uncertain` crate, focusing on handling uncertainty and various real-world sensor issues.

## Features Demonstrated

-   **Structured Sensor Data:** Defining custom structs and enums (`SensorReading`, `SensorStatus`) to represent sensor data and its quality.
-   **Robust Data Processing:** Implementing logic to process sensor readings with different statuses (healthy, degraded, failed, out-of-range, calibration drift), dynamically adjusting uncertainty based on sensor health.
-   **Data Validation and Quality Assessment:** Using `expected_value` and `standard_deviation` to assess the quality of processed data and the overall system health.
-   **Uncertainty-Aware Sensor Fusion:** Combining multiple uncertain sensor readings using weighted averaging based on their uncertainties.
-   **Anomaly Detection:** Identifying and handling anomalous sensor data based on value ranges and sensor status.
-   **Fallback Strategies and Graceful Degradation:** Demonstrating how to use historical data, cross-validate between sensor types, and define system operational modes based on sensor availability.
-   **Decision Making under Uncertainty:** Employing `implicit_conditional` and `probability_exceeds` for making informed decisions regarding maintenance, risk assessment, and recommendations.
-   **Comparison Operators:** Utilizing `greater_than`, `less_than`, and `within_range` for various checks within the processing logic.

## How to Run

To run this example, navigate to the root of the `deep_causality` project and execute the following command:

```bash
cargo run --example sensor_processing -p deep_causality_uncertain
```

This will compile and run the `sensor_processing.rs` example, displaying the results of the sensor data processing, fusion, anomaly detection, and reliability assessment.
