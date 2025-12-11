# Data Model: MRMR Feature Score

## Entities

### FeatureScorePair
Represents a selected feature and its calculated importance score. This is not a persistent entity but rather the primary data structure for the function's return value.

- **Fields**:
  - `index: usize`: The column index of the feature in the original dataset.
  - `score: f64`: The calculated importance score.
- **Constraints**:
  - `score` must be a valid, finite `f64` number.

## Relationships
- A `Vec<FeatureScorePair>` is the output of the `mrmr_features_selector` and `mrmr_features_selector_cdl` functions.