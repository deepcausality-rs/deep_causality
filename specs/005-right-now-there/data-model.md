# Data Model for Implement Data Cleaner and update Feature Selector for Option<f64>

## Entities

- **`OptionNoneDataCleaner`**:
  - **Description**: A component responsible for converting `f64` tensors to `Option<f64>` tensors, specifically handling `NaN` values by replacing them with `None`.

- **`DataCleaner` trait**:
  - **Description**: A Rust trait defining the interface for data cleaning operations within the CDL pipeline.
  - **Relationship**: Implemented by `OptionNoneDataCleaner`.

- **`FeatureSelector` trait**:
  - **Description**: A Rust trait defining the interface for feature selection operations. This trait will be updated to handle `CausalTensor<Option<f64>>`.
  - **Relationship**: Implemented by `MrmrFeatureSelector`.

- **`MrmrFeatureSelector`**:
  - **Description**: A specific implementation of `FeatureSelector` that uses the MRMR algorithm, adapted to work with `CausalTensor<Option<f64>>` inputs and outputs.

- **`CausalTensor<f64>`**:
  - **Description**: The original tensor type holding floating-point numbers, serving as input to the `DataCleaner`.

- **`CausalTensor<Option<f64>>`**:
  - **Description**: The new tensor type capable of representing the presence (`Some(f64)`) or absence (`None`) of floating-point values. This will be the output of `DataCleaner` and input/output of `FeatureSelector`.
