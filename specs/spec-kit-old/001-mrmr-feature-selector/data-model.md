# Data Model for mRMR Feature

This feature primarily operates on existing data structures and introduces one new error type.

## 1. CausalTensor (Existing)

- **Source**: `deep_causality_data_structures` crate.
- **Usage**: Represents the input dataset, containing both features and the target variable in a multi-dimensional array. The mRMR algorithm will treat it as a 2D matrix where rows are samples and columns are features.
- **Fields**:
    - `data: Vec<T>`
    - `shape: Vec<usize>`
    - `strides: Vec<usize>`
- **Constraints**: The input `CausalTensor` is expected to be 2-dimensional for this algorithm.

## 2. MrmrError (New)

- **Purpose**: To provide specific error handling for the mRMR implementation.
- **Type**: `enum`
- **Variants**:
    - `InvalidInput(String)`: For cases like an empty tensor or incorrect dimensions.
    - `CalculationError(String)`: For numerical issues during statistical calculations (e.g., division by zero).
    - `NotEnoughFeatures`: If the number of features requested is greater than the number of available features.

## 3. RankedFeatures (Output)

- **Purpose**: To represent the output of the algorithm.
- **Type**: `Vec<usize>`
- **Description**: A vector of column indices from the input `CausalTensor`, ordered from the most relevant and least redundant feature to the least.
