# Contracts: Causal Discovery DSL

This document outlines the conceptual API contracts for the Causal Discovery DSL, focusing on the traits and the `CQD` type's methods, which define the interaction points of the system.

## CQD Type Methods (Conceptual Signatures)

Based on the Typestate Pattern, the `CQD` type will expose methods that change its internal state, ensuring a valid sequence of operations.

```rust
// Initial state
pub struct CQD<NoData>;

impl CQD<NoData> {
    pub fn new() -> Self { /* ... */ }
    pub fn start<L: ProcessDataLoader>(self, loader: L, config: DataLoaderConfig) -> Result<CQD<WithData>, CqdError> { /* ... */ }
}

// After data is loaded
pub struct CQD<WithData>;

impl CQD<WithData> {
    pub fn feat_select<S: FeatureSelector>(self, selector: S, config: FeatureSelectorConfig) -> Result<CQD<WithFeatures>, CqdError> { /* ... */ }
}

// After features are selected
pub struct CQD<WithFeatures>;

impl CQD<WithFeatures> {
    pub fn causal_discovery<D: CausalDiscovery>(self, discovery: D, config: CausalDiscoveryConfig) -> Result<CQD<WithCausalResults>, CqdError> { /* ... */ }
}

// After causal discovery is performed
pub struct CQD<WithCausalResults>;

impl CQD<WithCausalResults> {
    pub fn analyze<A: ProcessResultAnalyzer>(self, analyzer: A) -> Result<CQD<WithAnalysis>, CqdError> { /* ... */ }
}

// After results are analyzed
pub struct CQD<WithAnalysis>;

impl CQD<WithAnalysis> {
    pub fn finalize<F: ProcessResultFormatter>(self, formatter: F) -> Result<CQD<Finalized>, CqdError> { /* ... */ }
}

// After process is finalized
pub struct CQD<Finalized>;

impl CQD<Finalized> {
    pub fn build(self) -> Result<CQDRunner, CqdError> { /* ... */ }
}

// Runner for the built pipeline
pub struct CQDRunner;

impl CQDRunner {
    pub fn run(self) -> Result<ProcessFormattedResult, CqdError> { /* ... */ }
}
```

## Traits (Conceptual Signatures)

### `ProcessDataLoader`
```rust
pub trait ProcessDataLoader {
    type Config;
    fn load(&self, path: &str, config: Self::Config) -> Result<CausalTensor, DataError>;
}
```

### `FeatureSelector`
```rust
pub trait FeatureSelector {
    type Config;
    fn select(&self, tensor: CausalTensor, config: Self::Config) -> Result<CausalTensor, FeatureSelectError>;
}
```

### `CausalDiscovery`
```rust
pub trait CausalDiscovery {
    type Config;
    fn discover(&self, tensor: CausalTensor, config: Self::Config) -> Result<SurdResult, CausalDiscoveryError>;
}
```

### `ProcessResultAnalyzer`
```rust
pub trait ProcessResultAnalyzer {
    fn analyze(&self, surd_result: SurdResult) -> Result<ProcessAnalysis, AnalyzeError>;
}
```

### `ProcessResultFormatter`
```rust
pub trait ProcessResultFormatter {
    fn format(&self, analysis: ProcessAnalysis) -> Result<ProcessFormattedResult, FinalizeError>;
}
```

## Configuration Structs (Conceptual)

- `DataLoaderConfig` (enum encapsulating `CsvConfig`, `ParquetConfig`)
- `FeatureSelectorConfig` (enum encapsulating `MrmrConfig`)
- `CausalDiscoveryConfig` (enum encapsulating `SurdConfig`)

(Detailed fields for `MrmrConfig`, `SurdConfig`, `CsvConfig`, `ParquetConfig` are in `data-model.md`)
