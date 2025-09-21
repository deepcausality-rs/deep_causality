# Design for a Data Discretization Step in CDL

This document outlines the design for a new `preprocess` step in the Causal Discovery Language (CDL). This step is critical for handling real-world datasets that contain continuous (non-discrete) variables, making them suitable for analysis with information-theoretic algorithms like SURD.

## Pipeline Integration & Optionality

The `preprocess` step must be **optional**. This means the CDL must support two valid execution paths:

1.  **Without Pre-processing:**
    `start` -> `feat_select` -> `causal_discovery` -> `analyze` -> `finalize`

2.  **With Pre-processing:**
    `start` -> **`preprocess`** -> `feat_select` -> `causal_discovery` -> `analyze` -> `finalize`

To achieve this, the `preprocess` step will be implemented as a **self-transitioning state**. It will take `CDL<WithData>` as input and return `CDL<WithData>`, allowing it to be chained multiple times or skipped entirely before proceeding to `feat_select`.

## 1. Identification of Non-Discrete Data

Since the current `DataLoader` converts all numeric data to `f64`, type-checking is insufficient. The identification of continuous data must be done using heuristics and user guidance.

### Design Proposal:

- **Automatic Heuristic-Based Detection:**
    - The pre-processing step will iterate through each column of the loaded data.
    - It will calculate the number of unique values for each column.
    - A column will be flagged as **continuous** if its unique value count exceeds a configurable threshold (e.g., `max_discrete_states: 20`).
    - Columns with a unique value count below this threshold will be assumed to be **already discrete** and will be skipped by the discretization logic.

- **User-Driven Override:**
    - To ensure robustness and accuracy, the user will have the final say.
    - The configuration will allow the user to provide an explicit list of columns (by index or name) that must be treated as continuous and undergo discretization, overriding any automatic detection.

## 2. Discretization Approach

The design will support the two most common binning strategies, allowing the user to choose the most appropriate one for their data.

### Design Proposal:

- **Equal-Width (Uniform) Binning:**
    - **Method:** Divides the data range (`max - min`) into `k` bins of identical width.
    - **Pros:** Simple, intuitive, and is the method referenced in the source SURD paper ("uniform partition").
    - **Cons:** Can be negatively affected by outliers and skewed distributions, potentially creating empty or sparse bins.
    - **Recommendation:** This will be the **default strategy**.

- **Equal-Frequency (Quantile) Binning:**
    - **Method:** Divides the sorted data into `k` bins, each containing an equal number of data points.
    - **Pros:** Robust against outliers and skewed data.
    - **Cons:** Can result in bins with very different widths and may group distinct values if they are close together in a dense region.

The user will configure the desired `strategy` and the `number_of_bins` (`k`).

## 3. CDL Syntax Extension

To integrate this step, new configuration objects, a new CDL typestate, and a new method will be introduced.

### A. New Configuration Objects

A `PreprocessConfig` will be added to the main `CdlConfig`.

```rust
// Configuration for the new preprocessing step
pub struct PreprocessConfig {
    strategy: BinningStrategy,
    num_bins: usize,
    columns: ColumnSelector,
}

// Enum to select the binning method
pub enum BinningStrategy {
    EqualWidth,
    EqualFrequency,
}

// Enum to specify which columns to process
pub enum ColumnSelector {
    All, // A convenience option
    ByIndex(Vec<usize>),
    ByName(Vec<String>),
}

// The main CdlConfig will be extended
pub struct CdlConfig {
    data_loader_config: Option<DataLoaderConfig>,
    preprocess_config: Option<PreprocessConfig>, // <-- NEW
    feature_selector_config: Option<FeatureSelectorConfig>,
    // ...
}
```

### B. New CDL Method and State Logic

The `preprocess` method will be implemented on `CDL<WithData>` and will return `CDL<WithData>`. The `feat_select` method will also remain on `CDL<WithData>`.

```rust
// In types/cdl/mod.rs
impl CDL<WithData> {
    // The preprocess method is a self-transition
    pub fn preprocess<P: DataPreprocessor>(
        self,
        preprocessor: P,
    ) -> Result<CDL<WithData>, CdlError> {
        // ... logic to apply discretization ...
        // Returns a new CDL<WithData>
    }

    // The feat_select method remains on WithData, making preprocess optional
    pub fn feat_select<S: FeatureSelector>(
        self,
        selector: S,
    ) -> Result<CDL<WithFeatures>, CdlError> {
        // ...
    }
}
```

### C. Example Usage

This is how a user would configure and execute the two valid pipelines.

```rust
// SCENARIO 1: Pre-processing is SKIPPED
let discovery_process = CDL::with_config(config)
    .start(CsvDataLoader, &file_path)?
    .feat_select(MrmrFeatureSelector)?
    .causal_discovery(SurdCausalDiscovery)?
    .build()?;

// SCENARIO 2: Pre-processing is INCLUDED
let discovery_process = CDL::with_config(config)
    .start(CsvDataLoader, &file_path)?
    .preprocess(DataDiscretizer)?
    .feat_select(MrmrFeatureSelector)?
    .causal_discovery(SurdCausalDiscovery)?
    .build()?;
```

## 4. Error Handling

To ensure the new step is robust, new error variants will be introduced to handle potential failures during pre-processing.

A new `PreprocessError` enum will be created to represent specific failures within the discretization step:

```rust
pub enum PreprocessError {
    InvalidColumnIdentifier(String), // e.g., "Column 's4' not found"
    BinningError(String),            // e.g., "Cannot bin column with only one unique value"
    ConfigError(String),             // e.g., "Number of bins must be at least 2"
}
```

The main `CdlError` enum, which represents an error in any stage of the pipeline, will be updated to include this new error type:

```rust
pub enum CdlError {
    ReadDataError(DataError),
    PreprocessError(PreprocessError), // <-- NEW
    FeatSelectError(FeatureSelectError),
    CausalDiscoveryError(CausalDiscoveryError),
    AnalyzeError(AnalyzeError),
    FinalizeError(FinalizeError),
    // ... other variants
}
```

This ensures that any issues encountered during the optional `preprocess` step are clearly reported to the user.