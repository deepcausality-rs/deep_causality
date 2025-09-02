# GPS Navigation Example

This example demonstrates how to use the `deep_causality_uncertain` crate to model and analyze a GPS navigation scenario where various inputs are uncertain.

## Features Demonstrated

-   **Uncertain Value Creation:** Using `Uncertain::normal`, `Uncertain::uniform`, and `Uncertain::<f64>::point` to define uncertain and fixed values.
-   **Arithmetic Operations:** Performing standard arithmetic (`+`, `-`, `*`, `/`) with uncertain values, including unary negation (`-`).
-   **Non-linear Transformations:** Applying custom functions like `sqrt` using the `map` method to propagate uncertainty through complex calculations.
-   **Statistical Analysis:** Calculating `expected_value` (mean) and `standard_deviation` from uncertain results.
-   **Probability Estimation:** Estimating the probability of an uncertain boolean condition being true using `estimate_probability`.
-   **Uncertain Comparisons:** Comparing two uncertain `f64` values using `lt_uncertain` and `gt_uncertain`.
-   **Conditional Logic:** Implementing `if-then-else` branching within the computation graph using `Uncertain::conditional`.
-   **Decision Making:** Using `implicit_conditional` for simple boolean decisions and `probability_exceeds` for more rigorous hypothesis testing.
-   **Specialized Comparisons:** Demonstrating `approx_eq` for approximate equality and `within_range` for checking if a value falls within a specified range.

## How to Run

To run this example, navigate to the root of the `deep_causality` project and execute the following command:

```bash
cargo run --example gps_navigation -p deep_causality_uncertain
```

This will compile and run the `gps_navigation.rs` example, displaying the results of the uncertainty analysis for distance, travel time, route decisions, and fuel consumption.
