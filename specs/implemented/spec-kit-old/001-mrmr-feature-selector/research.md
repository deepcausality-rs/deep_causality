# Research for mRMR Feature Selection

## Summary
The feature requires implementing the mRMR (Maximum Relevance and Minimum Redundancy) feature selection algorithm. Based on the referenced paper (1908.05376v1.pdf), the **FCQ (F-test Correlation Quotient)** variant is the most suitable choice due to its balance of performance, robustness, and computational efficiency.

## Technical Decisions

### Algorithm Variant
- **Decision**: Implement the FCQ (F-test Correlation Quotient) variant of mRMR.
- **Rationale**: The paper's empirical evaluation shows that FCQ performs well across different classification models and is computationally fast. It's a model-free approach, which makes it general-purpose.
- **Alternatives Considered**:
    - **MID/MIQ**: Rejected due to the difficulty and inaccuracy of estimating probability distributions for mutual information calculation.
    - **RFCQ/RFRQ**: These are model-based variants that perform well for Random Forest classifiers but are less general. FCQ is a better starting point for a general-purpose library.

### Core Calculations
- **Relevance (F-statistic)**:
    - **Decision**: Calculate the F-statistic from the Pearson correlation coefficient (`r`). The formula for a single feature is `F = (n-2) * r^2 / (1 - r^2)`.
    - **Rationale**: This is a standard and efficient way to compute the F-statistic for a single predictor, avoiding the need for a full linear regression model fitting at each step.
- **Redundancy (Pearson Correlation)**:
    - **Decision**: Use the standard Pearson correlation coefficient.
    - **Rationale**: This is specified by the FCQ variant and is computationally efficient.

### Data Handling
- **Input Data Structure**:
    - **Decision**: Use `CausalTensor` from the `deep_causality_data_structures` crate.
    - **Rationale**: This aligns with the project's existing data structures and provides the necessary column-based operations.
- **Missing Data**:
    - **Decision**: Replace missing values (NaNs) in each column with the mean of that column.
    - **Rationale**: This is a simple and standard imputation technique that prevents data loss.
- **Non-numeric Data**:
    - **Decision**: Ignore non-numeric columns. The algorithm is defined for numerical data.
    - **Rationale**: The statistical calculations (F-statistic, Pearson correlation) are not applicable to non-numeric data.
