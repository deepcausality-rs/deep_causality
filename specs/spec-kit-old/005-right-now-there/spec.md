# Feature Specification: Implement Data Cleaner and update Feature Selector for Option<f64>

**Feature Branch**: `005-right-now-there`  
**Created**: Monday, September 29, 2025  
**Status**: Draft  
**Input**: User description: "Right now, there is no data cleaner step in the CDL. I want you to 1) Implement the OptionNoneDataCleaner type in @deep_causality_discovery/src/types/data_cleaner/option_none.rs to implemnt the DataCleaner trait so that it takes an CausalTensor<f64> as an input, and replaces f64 values with an Some and all NaN with None. The idea is that the next step, the feature selector, will be updated to take an CausalTensor<Option<f64>> as an input and works only on the Some values while ignoring the None. the wisom here is to prevent bias from NaN substitutioin. The idea is that the next step, the feature selector, will be updated to take an CausalTensor<Option<f64>> as an input and works only on the Some values while ignoring the None. the wisom here is to prevent bias from NaN substitutioin. 2) Update the FeatureSelector trait in @deep_causality_discovery/src/traits/feature_selector.rs to take an CausalTensor<Option<f64>> as input and return an CausalTensor<Option<f64>> aas output. 3) update the FeatureSelector for MrmrFeatureSelector to use the CausalTensor<Option<f64>> as an input and output and use the mrmr_features_selector_cdl instead of the mrmr_features_selector as to process the tensor"

## Execution Flow (main)
```
1. Parse user description from Input
   → If empty: ERROR "No feature description provided"
2. Extract key concepts from description
   → Identify: actors, actions, data, constraints
3. For each unclear aspect:
   → Mark with [NEEDS CLARIFICATION: specific question]
4. Fill User Scenarios & Testing section
   → If no clear user flow: ERROR "Cannot determine user scenarios"
5. Generate Functional Requirements
   → Each requirement must be testable
   → Mark ambiguous requirements
6. Identify Key Entities (if data involved)
7. Run Review Checklist
   → If any [NEEDS CLARIFICATION]: WARN "Spec has uncertainties"
   → If implementation details found: ERROR "Remove tech details"
8. Return: SUCCESS (spec ready for planning)
```

---

## ⚡ Quick Guidelines
- ✅ Focus on WHAT users need and WHY
- ❌ Avoid HOW to implement (no tech stack, APIs, code structure)
- 👥 Written for business stakeholders, not developers

### Section Requirements
- **Mandatory sections**: Must be completed for every feature
- **Optional sections**: Include only when relevant to the feature
- When a section doesn't apply, remove it entirely (don't leave as "N/A")

### For AI Generation
When creating this spec from a user prompt:
1. **Mark all ambiguities**: Use [NEEDS CLARIFICATION: specific question] for any assumption you'd need to make
2. **Don't guess**: If the prompt doesn't specify something (e.g., "login system" without auth method), mark it
3. **Think like a tester**: Every vague requirement should fail the "testable and unambiguous" checklist item
4. **Common underspecified areas**:
   - User types and permissions
   - Data retention/deletion policies  
   - Performance targets and scale
   - Error handling behaviors
   - Integration requirements
   - Security/compliance needs

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
As a user of the CDL pipeline, I want to explicitly handle missing data (NaN values) by converting them to `None` in a `CausalTensor<Option<f64>>` format, so that subsequent feature selection steps can process valid data while ignoring missing values, preventing bias from imputation.

### Acceptance Scenarios
1. **Given** a `CausalTensor<f64>` containing `f64` values and `NaN` values, **When** the `OptionNoneDataCleaner` is applied, **Then** it should return a `CausalTensor<Option<f64>>` where `f64` values are wrapped in `Some()` and `NaN` values are replaced with `None`.
2. **Given** the `FeatureSelector` trait is updated to accept and return `CausalTensor<Option<f64>>`, **When** `MrmrFeatureSelector` is used with a `CausalTensor<Option<f64>>`, **Then** it should correctly select features by operating only on `Some` values and ignoring `None` values, utilizing `mrmr_features_selector_cdl`.

### Edge Cases
- What happens when the input `CausalTensor<f64>` contains no `NaN` values?
- How does the system handle a `CausalTensor<f64>` where all values are `NaN`?
- What happens if `mrmr_features_selector_cdl` encounters a column with only `None` values? If a column contains only `None` values, any pairwise correlation or F-statistic calculation involving that column will result in insufficient samples, leading to an `MrmrError::SampleTooSmall` error. This means such a column cannot be selected as a feature or target.

## Requirements *(mandatory)*

### Functional Requirements
- **FR-001**: The system MUST provide a `DataCleaner` implementation, `OptionNoneDataCleaner`, that transforms a `CausalTensor<f64>` into a `CausalTensor<Option<f64>>`.
- **FR-002**: The `OptionNoneDataCleaner` MUST convert all `f64` values to `Some(f64)` and all `NaN` values to `None`.
- **FR-003**: The `FeatureSelector` trait MUST be updated to accept `CausalTensor<Option<f64>>` as input and return `CausalTensor<Option<f64>>` as output.
- **FR-004**: The `MrmrFeatureSelector` MUST be updated to operate on `CausalTensor<Option<f64>>` inputs and outputs.
- **FR-005**: The `MrmrFeatureSelector` MUST utilize `mrmr_features_selector_cdl` to perform feature selection, processing only `Some` values and ignoring `None` values.
- **FR-006**: The `mrmr_features_selector_cdl` function MUST handle `CausalTensor<Option<f64>>` as input by performing **pairwise deletion** for statistical calculations. Specifically, for any pair of columns, only rows where both values are `Some(f64)` will be included in the calculation of relevance and redundancy. If, after pairwise deletion, there are insufficient samples for a calculation, an appropriate error (`MrmrError::SampleTooSmall`) will be returned.

### Key Entities *(include if feature involves data)*
- **`OptionNoneDataCleaner`**: A component responsible for converting `f64` tensors to `Option<f64>` tensors, specifically handling `NaN` values.
- **`DataCleaner` trait**: A contract defining the interface for data cleaning operations.
- **`FeatureSelector` trait**: A contract defining the interface for feature selection operations, now updated to handle `Option<f64>` tensors.
- **`MrmrFeatureSelector`**: A specific implementation of `FeatureSelector` that uses the MRMR algorithm, adapted to work with `Option<f64>` tensors.
- **`CausalTensor<f64>`**: The original tensor type holding floating-point numbers.
- **`CausalTensor<Option<f64>>`**: The new tensor type capable of representing the presence or absence of floating-point values.

---

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

---

## Execution Status
*Updated by main() during processing*

- [x] User description parsed
- [x] Key concepts extracted
- [x] Ambiguities marked
- [x] User scenarios defined
- [x] Requirements generated
- [x] Entities identified
- [x] Review checklist passed

---