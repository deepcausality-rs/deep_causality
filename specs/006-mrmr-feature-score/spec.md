# Feature Specification: MRMR Feature Score

**Feature Branch**: `006-mrmr-feature-score`  
**Created**: 2025-09-30  
**Status**: Draft  
**Input**: User description: "In the MRMR algorightm in the deep_causality_algorithms crate, the MRMR algo currently only returns the index of selected feature colums. However, a pair of (index, score) would be needed to determine the relative contribution of each column The mrmr_features_selector_cdl algorithm already calculates an importance score at each step to decide which feature to select next. 1. First Feature: It selects the feature with the highest 'relevance' (calculated using an F-statistic) to the target variable. This relevance score can be returned. 2. Subsequent Features: For the remaining features, it calculates an 'mRMR score' which is relevance / redundancy. The feature with the highest mRMR score is chosen. This score can also be returned. Thus the mrmrm needs to return a vector of tuples (feature_index, score) instead of just the feature indices."

---
## Clarifications
### Session 2025-09-30
- Q: What should happen if a calculated feature score is an invalid floating-point number (e.g., NaN or Infinity)? → A: Return a specific MrmrError::FeatureScoreError with a message indicating whether the score was 'Infinity' or 'NaN'.
- Q: What is the performance expectation for the algorithm after adding the score-capturing logic? → A: A minor performance degradation (e.g., <10%) is acceptable if required for the new functionality.
- Q: What level of logging is required for the feature scoring process? → A: No logging is required for this feature.

## User Scenarios & Testing *(mandatory)*

### Primary User Story
As a data scientist, I want the MRMR feature selection algorithms (`mrmr_features_selector` and `mrmr_features_selector_cdl`) to return not only the selected feature indices but also their corresponding importance scores, so that I can understand the relative contribution of each feature.

### Acceptance Scenarios
1. **Given** a dataset and a target variable, **When** I run an MRMR algorithm, **Then** the output MUST be a vector of `(feature_index, score)` tuples, where `score` is normalized between 0.0 and 1.0.
2. **Given** the selection process starts, **When** the first feature is chosen, **Then** its score MUST be its F-statistic (relevance). This is because redundancy, which measures similarity to already-selected features, cannot be calculated when the selected set is empty.
3. **Given** one or more features have been selected, **When** a subsequent feature is chosen, **Then** its score MUST be the mRMR score (`F-statistic / Redundancy`), where Redundancy is calculated against the set of previously selected features.

### Edge Cases
- **What happens when the input dataset is empty?** The algorithm MUST return an `MrmrError::InvalidInput` or `MrmrError::SampleTooSmall` error, as is current behavior.
- **How does the system handle cases where a score cannot be calculated (e.g., division by zero for redundancy)?** The algorithm MUST return a new `MrmrError::FeatureScoreError` variant containing a descriptive message.

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: The `mrmr_features_selector` function MUST return a `Result<Vec<(usize, f64)>, MrmrError>`.
- **FR-002**: The `mrmr_features_selector_cdl` function MUST return a `Result<Vec<(usize, f64)>, MrmrError>`.
- **FR-003**: The score for the first selected feature in both functions MUST be its F-statistic (relevance value).
- **FR-004**: The score for all subsequent selected features MUST be the calculated mRMR score, where the F-statistic (relevance) is divided by the calculated redundancy against already-selected features.
- **FR-005**: A new error variant `FeatureScoreError(String)` MUST be added to `mrmr_error.rs`.
- **FR-006**: The new error variant MUST be tested in `mrmr_error_tests.rs`.
- **FR-007**: Both `mrmr_features_selector` and `mrmr_features_selector_cdl` MUST return `MrmrError::FeatureScoreError` if a calculated redundancy score is zero, as this would lead to division by zero when calculating the mRMR score.
- **FR-008**: If a calculated mRMR score results in `Infinity` or `NaN`, the algorithm MUST return a `MrmrError::FeatureScoreError` with a message detailing the invalid score and the feature index.
- **FR-009**: The importance scores returned by `mrmr_features_selector` and `mrmr_features_selector_cdl` MUST be normalized to a range between 0.0 and 1.0 (inclusive).

### Non-Functional Requirements
- **NFR-001**: The performance of the updated functions, including score calculation, should not degrade by more than 10% compared to the previous version.
- **NFR-002**: No specific logging is required for the feature scoring process beyond what already exists.

### Key Entities *(include if feature involves data)*
- **Feature Score Pair**: Represents a selected feature and its importance.
  - **Attributes**: 
    - `index`: `usize` - The column index of the feature.
    - `score`: `f64` - The calculated importance score.
- **Constraints**: 
    - `score` must be a valid, finite `f64` number.
    - `score` must be normalized between 0.0 and 1.0 (inclusive).

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [ ] No implementation details (languages, frameworks, APIs)
- [ ] Focused on user value and business needs
- [ ] Written for non-technical stakeholders
- [ ] All mandatory sections completed

### Requirement Completeness
- [ ] No [NEEDS CLARIFICATION] markers remain
- [ ] Requirements are testable and unambiguous  
- [ ] Success criteria are measurable
- [ ] Scope is clearly bounded
- [ ] Dependencies and assumptions identified