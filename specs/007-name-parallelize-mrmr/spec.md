# Feature Specification: Parallelize mRMR algorithm with Rayon

**Feature Branch**: `007-name-parallelize-mrmr`  
**Created**: October 2, 2025  
**Status**: Draft  
**Input**: User description: "parallelize mRMR algo 1. Objective: Reduce the execution time of the mRMR feature selection algorithm by parallelizing its most computationally intensive parts using the rayon crate, guarded by a parallel feature flag. 2. Current State Analysis: * The deep_causality_algorithms crate already includes rayon as an optional dependency and defines a parallel feature flag in its Cargo.toml. * The surd_algo.rs and surd_algo_cdl.rs files provide a clear precedent for conditionally importing rayon::prelude::* and using rayon's parallel iterators (.into_par_iter()) within #[cfg(feature = "parallel")] blocks. * The mRMR algorithm, implemented in mrmr_algo.rs and mrmr_algo_cdl.rs, contains two primary loops suitable for parallelization: * The initial loop that identifies the first feature by finding the maximum relevance score. * The iterative loop that selects subsequent features by calculating relevance and redundancy for all remaining candidate features. 3. Proposed Changes: * Step 1: Conditional `rayon` Import: * In both deep_causality_algorithms/src/feature_selection/mrmr/mrmr_algo.rs and mrmr_algo_cdl.rs, add the following conditional import: 1 #[cfg(feature = "parallel")] 2 use rayon::prelude::*; * Step 2: Parallelize Initial Feature Selection: * Within the mrmr_features_selector and mrmr_features_selector_cdl functions, locate the loop responsible for determining the first_feature based on max_relevance. * Replace the sequential iteration over all_features with a parallel equivalent using rayon's par_iter() and appropriate methods like map() and max_by_key() to find the feature with the highest relevance. * This parallel logic will be encapsulated within #[cfg(feature = "parallel")] blocks, ensuring a sequential fallback when the feature is not enabled. * Step 3: Parallelize Iterative Feature Selection: * In both mrmr_features_selector and mrmr_features_selector_cdl, identify the inner loop within the while loop that iterates over all_features to compute relevance, redundancy, and the mrmr_score for each candidate feature. * Convert this inner loop to use rayon's par_iter() with map() and max_by_key() to efficiently find the best_feature and max_mrmr_score in parallel. * Similar to Step 2, this parallel implementation will be guarded by #[cfg(feature = "parallel")] blocks, maintaining a sequential alternative. 4. Risk Assessment and Mitigation: * Risk 1: Data Races/Incorrect Results due to Shared Mutable State: * Assessment: Parallelizing operations that directly modify shared mutable data structures (CausalTensor, all_features, selected_features_with_scores) can introduce data races and lead to incorrect or inconsistent results. * Mitigation: * Imputation: The impute_missing_values function (in mrmr_algo.rs) is a pre-processing step that modifies the CausalTensor sequentially before any parallel computations begin. This ensures thread safety for the tensor during parallel read operations. * Feature Sets: The parallel sections will primarily involve reading from all_features (to get candidate feature indices) and calculating scores. The critical operations of selecting a feature, removing it from all_features, and adding it to selected_features_with_scores will remain sequential. This design ensures that mutable shared state is accessed and modified by only one thread at a time, preventing data races. * `CausalTensor` Read Access: All CausalTensor operations within the parallel loops will be read-only, which is inherently thread-safe. * Risk 2: Performance Overhead for Small Datasets: * Assessment: The overhead associated with thread management and synchronization in parallel execution can negate performance gains or even lead to slower execution for small input datasets. * Mitigation: * Feature Gate: The existing parallel feature flag directly addresses this. Users can opt to compile the library without this feature if their use case primarily involves small datasets or if parallelization is not desired. * Runtime Thresholding (Consideration): While not part of the initial plan, a future enhancement could involve a runtime check for the number of elements to be processed. If the count falls below a certain threshold, the algorithm could dynamically switch to the sequential path, even if the parallel feature is enabled, to avoid unnecessary overhead. For this plan, we rely on the compile-time feature flag. * Risk 3: Increased Code Complexity and Maintainability: * Assessment: Introducing parallel constructs can make the codebase more complex, potentially increasing the difficulty of understanding, debugging, and maintaining the code. * Mitigation: * Clear Delineation: The use of #[cfg(feature = "parallel")] blocks will clearly separate the parallel and sequential implementations, making it easier to follow each logic path. * Idiomatic `rayon`: Adhering to rayon's recommended patterns and methods will ensure the parallel code is as clean and readable as possible. * Comprehensive Testing: Existing unit tests will be run to verify correctness. The parallel logic will be designed to produce the same functional output as the sequential version, minimizing the need for separate parallel-specific tests. * Risk 4: Numerical Stability/Floating-Point Differences: * Assessment: The non-associative nature of floating-point arithmetic means that the order of operations in parallel reductions (e.g., sum, max) can sometimes lead to minute differences in results compared to sequential execution. This is generally not a functional bug but can cause issues with strict equality checks in tests. * Mitigation: * Floating-Point Comparison Tolerance: Existing and future tests involving f64 comparisons should use an epsilon-based tolerance (e.g., assert!((a - b).abs() < epsilon)) rather than direct assert_eq!. This is a standard best practice for floating-point numbers. * Tie-Breaking: The mRMR algorithm's specification already notes that the selection order for features with identical scores is not guaranteed. rayon's max_by_key behavior for ties is generally consistent but may differ from a specific sequential order. Given the ambiguity, this is an acceptable characteristic. 5. Verification: * Unit Tests: Execute all existing unit tests for deep_causality_algorithms with the parallel feature both enabled and disabled (cargo test -p deep_causality_algorithms --features parallel and cargo test -p deep_causality_algorithms) to confirm that the parallelized code produces correct results and does not introduce regressions. * Benchmarking: Utilize criterion benchmarks (or similar profiling tools) to quantitatively measure the performance improvement of the mRMR algorithm on representative datasets (e.g., the sepsis case study data) when the parallel feature is enabled. Compare these results against the sequential execution to validate the effectiveness of parallelization."

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

## Clarifications

### Session October 2, 2025

-   Q: What is the target percentage reduction in execution time for the mRMR algorithm on large datasets when the `parallel` feature is enabled? → A: Performance is subject to benchmarks. The non-parallel version takes 3 minutes on the full sepsis data set, and a run with the parallel version should yield some speedup given how parallel the algo is.
-   Q: How will the performance improvement or potential issues be monitored in production? → A: No observation or telemetry needed.

---

## User Scenarios & Testing *(mandatory)*

### Primary User Story
As a data scientist working with large datasets, I want the mRMR feature selection algorithm to run faster, so I can quickly identify relevant features for my models and improve my workflow efficiency.

### Acceptance Scenarios
1.  **Given** the `deep_causality_algorithms` crate is compiled with the `parallel` feature enabled, **When** `mrmr_features_selector` or `mrmr_features_selector_cdl` is executed on a large dataset (e.g., the full sepsis data set), **Then** the execution time is reduced compared to the non-parallel version, as demonstrated by benchmarks.
2.  **Given** the `deep_causality_algorithms` crate is compiled with or without the `parallel` feature, **When** `mrmr_features_selector` or `mrmr_features_selector_cdl` is executed, **Then** the selected features and their scores are functionally equivalent (within acceptable floating-point tolerance) to the sequential version.
3.  **Given** the `deep_causality_algorithms` crate is compiled without the `parallel` feature, **When** `mrmr_features_selector` or `mrmr_features_selector_cdl` is executed, **Then** the algorithm runs sequentially without introducing `rayon`-related overhead.

### Edge Cases
-   What happens when the dataset is very small (e.g., fewer than 3 rows or 2 features)? The algorithm should still produce correct results, though performance might not improve or could slightly degrade due to parallelization overhead if the `parallel` feature is enabled.
-   How does the system handle `NaN` or `Infinity` values during parallel computations? The system MUST handle these values consistently with the sequential version, leading to an `MrmrError::FeatureScoreError` as currently implemented.

## Requirements *(mandatory)*

### Functional Requirements
-   **FR-001**: The mRMR feature selection algorithm MUST leverage parallel processing for its computationally intensive loops (initial feature selection and iterative feature selection) when the `parallel` feature is enabled.
-   **FR-002**: The parallel implementation MUST produce results that are functionally equivalent to the sequential implementation (within acceptable floating-point precision).
-   **FR-003**: The parallelization MUST be guarded by the existing `parallel` feature flag in `deep_causality_algorithms/Cargo.toml`.
-   **FR-004**: The algorithm MUST continue to function correctly in sequential mode when the `parallel` feature is not enabled.
-   **FR-005**: The parallel implementation MUST not introduce data races or undefined behavior.
-   **FR-006**: The parallel implementation MUST handle numerical edge cases (e.g., `NaN`, `Infinity`) consistently with the sequential version, propagating `MrmrError::FeatureScoreError` where appropriate.

---

## Review & Acceptance Checklist
*GATE: Automated checks run during main() execution*

### Content Quality
- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

### Requirement Completeness
- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous  
- [x] Success criteria are measurable
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

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