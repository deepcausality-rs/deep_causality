# Quickstart: Using MaybeUncertain<T>

This guide demonstrates how to use the `MaybeUncertain<T>` type to model data with uncertain presence, based on the acceptance scenarios in the specification.

## 1. Basic Usage: Modeling Intermittent Data

```rust
use deep_causality_uncertain::{MaybeUncertain, Uncertain, CausalTensorError};

// Example: A sensor that has a 75% chance of providing a reading,
// and the reading itself is a normal distribution centered around 10.0.
let prob_present = 0.75;
let value_if_present = Uncertain::normal(10.0, 2.0);

let maybe_temp = MaybeUncertain::from_bernoulli_and_uncertain(prob_present, value_if_present);

// Sample the value multiple times to observe its probabilistic presence
println!("Sampling 10 times:");
for _ in 0..10 {
    match maybe_temp.sample() {
        Some(value) => println!("  - Value present: {:.2}", value),
        None => println!("  - Value absent (None)"),
    };
}
```

## 2. Lifting to a Definite Value

To use the value in calculations, you might need to assert its presence. This is done with `lift_to_uncertain`.

### Successful Lift

```rust
// Scenario: We are confident if the probability of presence is at least 70%.
let threshold = 0.70;
let confidence = 0.95; // 95% confidence level
let max_samples = 1000;

match maybe_temp.lift_to_uncertain(threshold, confidence, max_samples) {
    Ok(definite_temp) => {
        println!("\nSuccessfully lifted with 70% threshold.");
        // We can now use `definite_temp` as a standard `Uncertain<T>`
        let avg_value = definite_temp.mean(100);
        println!("Average value of definite temp: {:.2}", avg_value);
    }
    Err(_) => {
        println!("This should not happen with a 70% threshold.");
    }
}
```

### Failed Lift

```rust
// Scenario: We require a higher confidence of 80%.
let threshold_high = 0.80;

match maybe_temp.lift_to_uncertain(threshold_high, confidence, max_samples) {
    Err(CausalTensorError::InsufficientEvidenceForPresence) => {
        println!("\nFailed to lift with 80% threshold, as expected.");
    }
    Ok(_) => {
        println!("This should not happen with an 80% threshold.");
    }
    Err(_) => {
        println!("An unexpected error occurred.");
    }
}
```