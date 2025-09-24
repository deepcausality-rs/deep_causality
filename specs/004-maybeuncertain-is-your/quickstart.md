# Quickstart: Using MaybeUncertain<T>

This guide demonstrates how to use the `MaybeUncertain<T>` type to handle data that may be probabilistically absent.

## Core Concept

`MaybeUncertain<T>` is designed for scenarios where the very presence of a data point is uncertain. This is common in
clinical data or sensor readings, where values can be missing for various reasons.

Instead of forcing a choice between a definite value and a definite `None`, `MaybeUncertain<T>` allows you to model the
probability of presence itself.

## Example: Processing Potentially Missing Clinical Data

The following example illustrates how `MaybeUncertain<T>` can be used within a causal model (e.g., inside a Causaloid's
`f_chi` function) to robustly handle a lactate measurement that may or may not have been recorded at a specific time.

```rust
// Assume this code is inside a function that returns a Result
// and has access to a `context` object that can retrieve data.

// 1. Retrieve the lactate value from the context.
// The context returns a `MaybeUncertain<f64>`, acknowledging that the
// reading might not be present for the given time `t`.
let raw_lactate: MaybeUncertain<f64> = context.get("Lactate_at_t")?;

// 2. Probabilistically "unwrap" the value.
// Before using the lactate value, we must first be confident that it's actually present.
// `lift_to_uncertain` acts as a probabilistic gate.
let maybe_present_lactate_dist = raw_lactate.lift_to_uncertain(
    0.8,  // We require at least 80% probability that Lactate was measured.
    0.95, // We want to be 95% confident in this probability assessment.
    5     // Use up to 5 samples from the underlying distribution to decide.
);

// 3. Act based on the presence or absence of the data.
match maybe_present_lactate_dist {
    Ok(present_lactate_dist) => {
        // --- Path A: Data is likely present ---
        // `present_lactate_dist` is a standard `Uncertain<f64>`.
        // We can now safely perform further analysis on its value.
        println!("Lactate is present. Analyzing its value...");

        // For example, check if the lactate value is critically high.
        let is_high = present_lactate_dist.greater_than(2.0);
        if is_high.implicit_conditional()? {
            println!("Alert: Lactate level is likely greater than 2.0.");
            // Trigger high lactate protocol...
        } else {
            println!("Lactate level is within normal range.");
        }
    },
    Err(UncertainError::PresenceError(_)) => {
        // --- Path B: Data is likely absent ---
        // The value's presence did not meet our 80% probability threshold.
        // The causal model should now engage an alternative path.
        println!("Lactate data is too sparse or unreliable for this hour.");
        // Engage alternative causal path (e.g., trigger "MissingDataProtocol").
    },
    Err(e) => {
        // Handle other potential errors during the operation.
        println!("An unexpected error occurred: {:?}", e);
        return Err(e.into());
    }
}
```

### Why This is Valuable

- **Explicit Modeling of Missing Values**: The model doesn't crash or use default values for missing data. It explicitly
  reasons about the probability of a value's presence.
- **Robustness**: The `lift_to_uncertain` gate prevents uncertainly-present data from propagating through a model as if
  it were certainly present, which could lead to incorrect conclusions.
- **Nuanced Decision-Making**: It allows for more sophisticated causal pathways that can adapt to the quality and
  completeness of the available data.