# Research: MRMR Feature Score

## Summary
This feature modifies the existing MRMR algorithm to return feature scores alongside their indices. The core logic of the algorithm is already understood. The primary research areas involved clarifying non-functional requirements and edge case handling, which were resolved during the specification clarification phase.

## Key Decisions

- **Error Handling**:
  - **Decision**: If a calculated mRMR score results in `Infinity` or `NaN`, the algorithm will return a `MrmrError::FeatureScoreError`.
  - **Rationale**: This provides a clear, safe failure mode instead of allowing invalid floating-point values to propagate, which could cause unpredictable behavior downstream.

- **Performance**:
  - **Decision**: A minor performance degradation of less than 10% is acceptable.
  - **Rationale**: Capturing and returning scores introduces a small overhead. This budget allows for a clean implementation without requiring extensive micro-optimizations, as correctness and clarity are prioritized.

- **Logging**:
  - **Decision**: No additional logging is required.
  - **Rationale**: The operation is a pure calculation. Existing error-handling is sufficient for diagnostics.

## Alternatives Considered
- No significant alternatives were considered, as the feature request is a direct and necessary extension of the existing algorithm's functionality.