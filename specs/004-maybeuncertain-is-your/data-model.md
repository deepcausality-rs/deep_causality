# Data Model: MaybeUncertain<T>

Based on the feature specification, the following key entities are identified.

## 1. Key Entities

### 1.1. MaybeUncertain<T>

*   **Description**: A first-class type representing a value that is probabilistically present or absent. If the value is present, its own value is uncertain.
*   **Attributes**: The internal structure will be a sampling function that returns an `Option<T>`, abstracting the source of uncertainty (e.g., Bernoulli trials, direct sampling).
*   **Relationships**: It is related to the existing `Uncertain<T>` type. The `lift_to_uncertain` method provides a direct bridge, converting `MaybeUncertain<T>` to `Result<Uncertain<T>, ...>` under specific statistical conditions.

### 1.2. CausalTensorError::InsufficientEvidenceForPresence

*   **Description**: A new error variant within an existing or new error enum. It signals that a probabilistic value failed to meet the required confidence threshold to be considered definitively present.
*   **Attributes**: None. It is a unit-like enum variant.
*   **Relationships**: This error is returned exclusively by the `lift_to_uncertain` method on `MaybeUncertain<T>`.