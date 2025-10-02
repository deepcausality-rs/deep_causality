# Research: Parallelize mRMR algorithm with Rayon

## Technical Approach Summary

**Decision**: Parallelize the computationally intensive loops within the mRMR feature selection algorithm (`mrmr_features_selector` and `mrmr_features_selector_cdl`) using the `rayon` crate.

**Rationale**: The mRMR algorithm, particularly when processing large datasets, can be time-consuming due to iterative calculations of F-statistics and Pearson correlations across multiple features. `rayon` provides a robust and idiomatic way to introduce data parallelism in Rust, leveraging multi-core processors to significantly reduce execution time. The existing `parallel` feature flag in `deep_causality_algorithms` for the SURD algorithm provides a proven pattern for this integration.

**Alternatives Considered**:
-   **Manual Threading**: Rejected due to increased complexity, potential for errors (data races, deadlocks), and higher maintenance burden compared to `rayon`'s high-level abstractions.
-   **GPU Acceleration (e.g., `ndarray-gpu`)**: Considered too complex and potentially overkill for this specific optimization. It would introduce a new dependency and significantly increase the project's complexity for a performance gain that `rayon` can largely achieve for CPU-bound tasks.

## Implementation Details

1.  **Conditional `rayon` Import**: Add `#[cfg(feature = "parallel")] use rayon::prelude::*;` to `mrmr_algo.rs` and `mrmr_algo_cdl.rs`.
2.  **Parallelize Initial Feature Selection**: In both `mrmr_features_selector` and `mrmr_features_selector_cdl`, the loop that identifies the `first_feature` (based on `max_relevance`) will be converted to use `par_iter()` for parallel computation. This will involve iterating over `all_features` in parallel, calculating relevance, and finding the maximum.
3.  **Parallelize Iterative Feature Selection**: The inner loop within the `while` loop that selects subsequent features will also be parallelized. This loop iterates over remaining `all_features` to calculate `relevance`, `redundancy`, and the `mrmr_score`. `par_iter()` will be used to compute these scores in parallel, followed by finding the `best_feature` with the maximum mRMR score.
4.  **Feature Flag Guarding**: All `rayon`-specific code will be enclosed within `#[cfg(feature = "parallel")]` blocks, ensuring that the parallel implementation is only compiled and used when the `parallel` feature is explicitly enabled.

## Expected Outcomes

-   **Performance Improvement**: Significant reduction in execution time for mRMR on large datasets when the `parallel` feature is enabled.
-   **Correctness**: The parallel implementation will produce results functionally equivalent to the sequential version (within floating-point tolerance).
-   **Maintainability**: The use of `rayon`'s high-level abstractions will keep the code readable and maintainable, with clear separation between parallel and sequential paths.
-   **Safety**: No `unsafe` code will be introduced, and existing safety guarantees will be maintained.
